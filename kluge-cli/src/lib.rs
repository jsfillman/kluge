use kluge_check::{lex, match_phrase, match_verb, read_sbatch, resolve, Match, Phrase, Spec};

#[derive(Debug, PartialEq, Eq)]
pub struct Screen {
    pub rows: Vec<String>,
    pub explain: String,
    pub outputs: Vec<String>,
}

pub fn screen_from(rows: Vec<String>, explain: &str, into: &str, files: &[String]) -> Screen {
    let (outputs, _) = kluge_check::select_outputs(into, files);
    Screen {
        rows,
        explain: explain.to_string(),
        outputs,
    }
}

pub struct WatchModel {
    pub rows: Vec<String>,
    pub detached: bool,
    pub cancelled: bool,
}

impl WatchModel {
    pub fn new(rows: Vec<String>) -> Self {
        Self {
            rows,
            detached: false,
            cancelled: false,
        }
    }
    pub fn press_q(&mut self) {
        self.detached = true;
    }
    pub fn render(&self) -> String {
        self.rows.join("\n")
    }
}

pub fn complete(
    previous: &[String],
    files: &[String],
    gpus: &[String],
    runs: &[String],
) -> Vec<String> {
    let last = previous.last().map(String::as_str).unwrap_or("");
    if previous.len() <= 1 {
        return kluge_check::VERBS.iter().map(|verb| (*verb).to_string()).collect();
    }
    match match_verb(last) {
        Match::One("run") => files
            .iter()
            .filter(|file| {
                file.ends_with(".py")
                    || file.ends_with(".sh")
                    || file.ends_with(".kluge")
                    || file.ends_with(".sbatch")
            })
            .cloned()
            .collect(),
        Match::One("logs") | Match::One("cancel") => runs.to_vec(),
        _ => match match_phrase(last) {
            Match::One("on") => gpus.to_vec(),
            Match::One("for") => ["30m", "6h", "6 hours"]
                .into_iter()
                .map(str::to_string)
                .collect(),
            _ => Vec::new(),
        },
    }
}

pub fn completion_script(shell: &str) -> String {
    format!("# kluge completion for {shell}\nkluge complete\n")
}

pub trait CheckClient {
    fn check(&self, spec: &Spec) -> Result<Vec<kluge_check::Diagnostic>, String>;
    fn submit(&self, spec: &Spec) -> Result<String, String> {
        Ok(spec.name.clone().unwrap_or_else(|| "run".to_string()))
    }
    fn list(&self) -> Result<Vec<String>, String> {
        Ok(Vec::new())
    }
    fn logs(&self, _name: &str) -> Result<String, String> {
        Ok(String::new())
    }
    fn cancel(&self, _name: &str) -> Result<(), String> {
        Ok(())
    }
    fn explain(&self, name: &str) -> Result<kluge_check::Diagnostic, String> {
        Err(format!("unknown run {name}"))
    }
    fn attach(&self, name: &str) -> Result<String, String> {
        Ok(format!("pod/{name}"))
    }
}

pub fn execute(args: &[String], client: &dyn CheckClient) -> (i32, String) {
    if let Some((code, out)) = dispatch_read(args, client) {
        return (code, out);
    }
    let mut dry_run = false;
    let mut json = false;
    let mut strict = false;
    let mut fix = false;
    let mut positional = Vec::new();
    for (index, arg) in args.iter().enumerate() {
        if index == 0 {
            positional.push(arg.clone());
            continue;
        }
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            "--json" => json = true,
            "--strict" => strict = true,
            "--fix" => fix = true,
            "--yes" => {}
            _ => positional.push(arg.clone()),
        }
    }
    if fix {
        if let Some(path) = positional
            .iter()
            .find(|arg| arg.ends_with(".kluge") || arg.ends_with(".sbatch"))
        {
            let pool = vec![kluge_check::PoolMachine {
                kind: "h100".to_string(),
                gpus: 8,
                count: 8,
            }];
            let outcome = kluge_check::apply_auto(std::path::Path::new(path), &pool);
            let code = if outcome
                .diagnostics
                .iter()
                .any(|item| item.severity == kluge_check::Severity::Error)
            {
                1
            } else {
                0
            };
            return (code, render_list(&outcome.diagnostics, json));
        }
    }
    let local = local_diagnostics(&positional);
    if local
        .iter()
        .any(|item| item.severity == kluge_check::Severity::Error)
    {
        return (1, render_list(&local, json));
    }
    let spec = match parse_invocation(&positional) {
        Ok(spec) => spec,
        Err(err) => return (1, err),
    };
    let remote = match client.check(&spec) {
        Ok(diagnostics) => diagnostics,
        Err(err) => return (1, err),
    };
    let failed = remote
        .iter()
        .any(|item| item.severity == kluge_check::Severity::Error)
        || (strict
            && remote
                .iter()
                .any(|item| item.severity == kluge_check::Severity::Warning));
    if failed {
        return (1, render_list(&remote, json));
    }
    let mut out = render_list(&remote, json);
    if !json {
        out.push_str(&kluge_check::render_sentence(&spec));
    }
    if !dry_run {
        match client.submit(&spec) {
            Ok(name) => {
                if !json {
                    out.push_str(&name);
                    out.push('\n');
                }
            }
            Err(err) => return (1, err),
        }
    }
    (0, out)
}

fn dispatch_read(args: &[String], client: &dyn CheckClient) -> Option<(i32, String)> {
    let verb = args.get(1).map(String::as_str).unwrap_or("");
    match match_verb(verb) {
        Match::One("status") => Some(show_status(args.get(2).map(String::as_str), client)),
        Match::One("logs") => Some(show_logs(args.get(2).map(String::as_str), client)),
        Match::One("cancel") => Some(cancel_run(args.get(2).map(String::as_str), client)),
        Match::One("explain") => Some(explain_run(args.get(2).map(String::as_str), client)),
        Match::One("debug") => Some(debug_run(args, client)),
        Match::One("attach") => Some(attach_run(args.get(2).map(String::as_str), client)),
        _ => None,
    }
}

fn show_status(name: Option<&str>, client: &dyn CheckClient) -> (i32, String) {
    let names = client.list().unwrap_or_default();
    let Some(name) = name else {
        return (0, names.join("\n"));
    };
    let hits: Vec<_> = names.into_iter().filter(|item| item.starts_with(name)).collect();
    match hits.len() {
        1 => (0, hits[0].clone()),
        0 => (1, format!("unknown run {name}")),
        _ => (1, format!("parse.ambiguous {}", hits.join(" "))),
    }
}

fn show_logs(name: Option<&str>, client: &dyn CheckClient) -> (i32, String) {
    let Some(name) = name else {
        return (1, "logs needs a run".to_string());
    };
    match client.logs(name) {
        Ok(body) => (0, body),
        Err(err) => (1, err),
    }
}

fn cancel_run(name: Option<&str>, client: &dyn CheckClient) -> (i32, String) {
    let Some(name) = name else {
        return (1, "cancel needs a run".to_string());
    };
    match client.cancel(name) {
        Ok(()) => (0, name.to_string()),
        Err(err) => (1, err),
    }
}

fn debug_run(args: &[String], client: &dyn CheckClient) -> (i32, String) {
    let mut positional = vec![args[0].clone(), "run".to_string()];
    positional.extend(args.iter().skip(2).cloned());
    let spec = match parse_invocation(&positional) {
        Ok(spec) => spec,
        Err(err) => return (1, err),
    };
    let kind = spec
        .on
        .as_ref()
        .map(|shape| shape.kind.clone())
        .unwrap_or_else(|| "h100".to_string());
    let mut rewritten = spec;
    rewritten.on = Some(kluge_check::GpuShape { count: 1, kind });
    rewritten.across = 1;
    let sentence = kluge_check::render_sentence(&rewritten);
    match client.submit(&rewritten) {
        Ok(_) => (0, sentence),
        Err(err) => (1, err),
    }
}

fn attach_run(name: Option<&str>, client: &dyn CheckClient) -> (i32, String) {
    let Some(name) = name else {
        return (1, "attach needs a run".to_string());
    };
    match client.attach(name) {
        Ok(target) => (0, target),
        Err(err) => (1, err),
    }
}

fn explain_run(name: Option<&str>, client: &dyn CheckClient) -> (i32, String) {
    let Some(name) = name else {
        return (1, "explain needs a run".to_string());
    };
    match client.explain(name) {
        Ok(diagnostic) => {
            let code = if diagnostic.severity == kluge_check::Severity::Error {
                1
            } else {
                0
            };
            (code, kluge_check::render(&diagnostic))
        }
        Err(err) => (1, err),
    }
}

fn render_list(diagnostics: &[kluge_check::Diagnostic], json: bool) -> String {
    if json {
        serde_json::to_string(diagnostics).unwrap_or_default()
    } else {
        diagnostics.iter().map(kluge_check::render).collect()
    }
}

fn local_diagnostics(args: &[String]) -> Vec<kluge_check::Diagnostic> {
    if !matches!(args.get(1).map(String::as_str).map(match_verb), Some(Match::One("run"))) {
        return Vec::new();
    }
    let rest = &args[2..];
    let Some(head) = rest.first() else {
        return kluge_check::check_local_layers("", "", "");
    };
    if head.ends_with(".sbatch") {
        let text = std::fs::read_to_string(head).unwrap_or_default();
        return kluge_check::check_local_sbatch(&text);
    }
    if head.ends_with(".kluge") {
        let text = std::fs::read_to_string(head).unwrap_or_default();
        return kluge_check::check_local_layers("", &text, "");
    }
    let mut text = format!("run {head}\n");
    let mut index = 1;
    while index < rest.len() {
        text.push_str(&rest[index]);
        text.push(' ');
        index += 1;
        while index < rest.len() && !is_phrase(&rest[index]) {
            text.push_str(&rest[index]);
            text.push(' ');
            index += 1;
        }
        text.push('\n');
    }
    kluge_check::check_local_layers("", &text, "")
}

pub fn parse_invocation(args: &[String]) -> Result<Spec, String> {
    let verb = args.get(1).map(String::as_str).unwrap_or("");
    match match_verb(verb) {
        Match::One("run") => parse_run(&args[2..]),
        Match::One(_) => Err(format!("verb {verb} is not run")),
        Match::Ambiguous(names) => Err(names.join(", ")),
        Match::None => Err(unmatched_verb(verb)),
    }
}

fn unmatched_verb(token: &str) -> String {
    let mut names = Vec::new();
    if token.len() >= 2 {
        let prefix = &token[..2];
        names.extend(kluge_check::VERBS.iter().copied().filter(|name| name.starts_with(prefix)));
    }
    for synonym in ["stop", "ssh"] {
        if synonym.starts_with(token) {
            let canonical = if synonym == "stop" { "cancel" } else { "attach" };
            if !names.contains(&canonical) {
                names.push(canonical);
            }
        }
    }
    if names.is_empty() {
        format!("unknown verb {token}")
    } else {
        names.join(", ")
    }
}

fn parse_run(args: &[String]) -> Result<Spec, String> {
    let head = args.first().ok_or_else(|| "run is missing".to_string())?;
    if head.ends_with(".sbatch") {
        let text = std::fs::read_to_string(head).map_err(|err| err.to_string())?;
        return read_sbatch(&text).map_err(|err| format!("{}: {}", err.line, err.text));
    }
    if head.ends_with(".kluge") {
        let text = std::fs::read_to_string(head).map_err(|err| err.to_string())?;
        let file = lex(&text).map_err(|err| format!("{err:?}"))?;
        let cli = phrase_lines(&args[1..])?;
        return resolve(&[], &file, &cli).map_err(|err| format!("{err:?}"));
    }
    let mut lines = vec![format!("run {head}")];
    lines.extend(phrase_lines(&args[1..])?.into_iter().map(|phrase| format!("{} {}", phrase.name, phrase.value)));
    let phrases = lex(&lines.join("\n")).map_err(|err| format!("{err:?}"))?;
    resolve(&[], &[], &phrases).map_err(|err| format!("{err:?}"))
}

fn phrase_lines(args: &[String]) -> Result<Vec<Phrase>, String> {
    let mut text = String::new();
    let mut index = 0;
    while index < args.len() {
        let token = &args[index];
        let name = match match_phrase(token) {
            Match::One(name) => name,
            Match::None => return Err(format!("unknown phrase {token}")),
            Match::Ambiguous(names) => return Err(names.join(", ")),
        };
        index += 1;
        let mut value = Vec::new();
        while index < args.len() && !is_phrase(&args[index]) {
            value.push(args[index].as_str());
            index += 1;
        }
        text.push_str(name);
        text.push(' ');
        text.push_str(&value.join(" "));
        text.push('\n');
    }
    lex(&text).map_err(|err| format!("{err:?}"))
}

fn is_phrase(token: &str) -> bool {
    matches!(match_phrase(token), Match::One(_))
}
