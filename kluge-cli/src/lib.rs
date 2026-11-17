use kluge_check::{lex, match_phrase, match_verb, read_sbatch, resolve, Match, Phrase, Spec};

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
