pub fn crate_name() -> &'static str {
    "kluge-check"
}

pub fn select_outputs(into: &str, files: &[String]) -> (Vec<String>, Vec<Diagnostic>) {
    let mut kept = Vec::new();
    let mut diagnostics = Vec::new();
    let prefix = format!("{}/", into.trim_end_matches('/'));
    for file in files {
        if file.split('/').any(|part| part == "..") {
            diagnostics.push(plain(
                Code::PathEscape,
                "output path leaves into",
                1,
                file,
            ));
            continue;
        }
        if file == into || file.starts_with(&prefix) {
            kept.push(file.clone());
        }
    }
    (kept, diagnostics)
}

#[derive(Debug, Clone)]
pub struct AdmitView {
    pub allow: Vec<String>,
    pub require_digest: bool,
    pub pull_ok: bool,
    pub live_names: Vec<String>,
    pub teams: Vec<String>,
    pub caller: Option<String>,
    pub max_wall_secs: u64,
}

pub fn check_admit(spec: &Spec, view: &AdmitView) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if view.caller.is_none() {
        diagnostics.push(plain(Code::AuthIdentity, "caller is not authenticated", 1, "team"));
    }
    if let Some(team) = &spec.team {
        if !view.teams.iter().any(|allowed| allowed == team) {
            diagnostics.push(plain(Code::AuthTeam, "caller cannot charge this team", 1, team));
        }
    }
    if let Some(name) = &spec.name {
        if !valid_name(name) {
            diagnostics.push(plain(Code::NameSyntax, "name does not match the pattern", 1, name));
        } else if view.live_names.iter().any(|live| live == name) {
            diagnostics.push(suggestion_diag(
                Code::NameTaken,
                "a live run already has this name",
                1,
                name,
                None,
                Suggestion {
                    auto: false,
                    replacement: format!("{name}-2"),
                    command: String::new(),
                },
            ));
        }
    }
    if let Some(image) = &spec.image {
        let allowed = view.allow.iter().any(|prefix| image.starts_with(prefix));
        let pinned = image.contains('@') || (!image.ends_with(":latest") && image.contains(':'));
        if !allowed || (view.require_digest && !image.contains('@')) {
            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                ..plain(Code::ImageDenied, "image is not allowed", 1, image)
            });
        } else if image.ends_with(":latest") {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                ..plain(Code::ImageDenied, "floating tag", 1, image)
            });
        }
        if !view.pull_ok {
            diagnostics.push(plain(Code::ImagePull, "the cluster cannot pull the image", 1, image));
        }
        let _ = pinned;
    }
    if let Some(duration) = spec.duration {
        if duration.as_secs() > view.max_wall_secs {
            diagnostics.push(plain(
                Code::TimeCeiling,
                format!("for exceeds the ceiling of {} seconds", view.max_wall_secs),
                1,
                "for",
            ));
        }
    }
    diagnostics
}

fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_lowercase()
        && name.len() <= 63
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

#[derive(Debug, Clone)]
pub struct FsView {
    pub root: String,
    pub existing: Vec<String>,
    pub links: Vec<(String, String)>,
}

pub fn check_paths(spec: &Spec, fs: &FsView) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for (label, value) in [("dir", spec.dir.as_deref()), ("into", spec.into.as_deref())] {
        let Some(value) = value else { continue };
        let resolved = resolve_fs_path(value, fs);
        if !within_root(&resolved, &fs.root) {
            diagnostics.push(plain(Code::PathEscape, format!("{label} leaves the team root"), 1, value));
        } else if !fs.existing.iter().any(|path| path == &resolved) {
            diagnostics.push(plain(Code::PathMissing, format!("{label} does not exist"), 1, value));
        }
    }
    diagnostics
}

fn resolve_fs_path(path: &str, fs: &FsView) -> String {
    let path = fs
        .links
        .iter()
        .find(|(from, _)| from == path)
        .map(|(_, target)| target.clone())
        .unwrap_or_else(|| path.to_string());
    let mut stack = Vec::new();
    for part in path.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        if part == ".." {
            stack.pop();
        } else {
            stack.push(part);
        }
    }
    format!("/{}", stack.join("/"))
}

fn within_root(path: &str, root: &str) -> bool {
    let root = root.trim_end_matches('/');
    path == root || path.starts_with(&format!("{root}/"))
}

#[derive(Debug, Clone)]
pub struct NodeFact {
    pub kind: String,
    pub gpus: u32,
    pub healthy: bool,
    pub free: bool,
}

pub fn check_pool(spec: &Spec, nodes: &[NodeFact], quota: u32) -> Vec<Diagnostic> {
    let Some(shape) = spec.on.as_ref() else {
        return Vec::new();
    };
    let matching: Vec<&NodeFact> = nodes.iter().filter(|node| node.kind == shape.kind).collect();
    if matching.is_empty() {
        return vec![plain(Code::PoolType, "gpu type is not on the reserved pool", 1, &shape.kind)];
    }
    let per_node = matching[0].gpus;
    let healthy: Vec<&NodeFact> = matching.iter().copied().filter(|node| node.healthy).collect();
    let healthy_count = healthy.len() as u32;
    if shape.count > per_node {
        if per_node == 0 || shape.count % per_node != 0 {
            return vec![plain(Code::PoolShape, "no machine has this shape", 1, &format!("on {} {}", shape.count, shape.kind))];
        }
        let wanted = shape.count / per_node;
        let total_gpus = per_node * healthy_count;
        let quota_machines = if per_node == 0 { 0 } else { quota / per_node };
        let legal = healthy_count.min(quota_machines);
        if wanted <= legal {
            return vec![suggestion_diag(
                Code::PoolShape,
                format!("each reserved node has {per_node} {}", shape.kind),
                1,
                &format!("on {} {}", shape.count, shape.kind),
                None,
                Suggestion {
                    auto: true,
                    replacement: format!("on {per_node} {}\nacross {wanted}", shape.kind),
                    command: String::new(),
                },
            )];
        }
        return vec![suggestion_diag(
            Code::PoolCapacity,
            "shape is larger than the pool",
            1,
            &format!("on {} {}", shape.count, shape.kind),
            None,
            Suggestion {
                auto: false,
                replacement: format!("on {per_node} {}\nacross {legal}", shape.kind),
                command: String::new(),
            },
        )];
    }
    let requested = spec.across;
    let total_requested = requested * shape.count;
    if requested > healthy_count || total_requested > quota {
        let legal = healthy_count.min(if shape.count == 0 { 0 } else { quota / shape.count });
        return vec![suggestion_diag(
            Code::PoolCapacity,
            "shape is larger than the pool",
            1,
            &format!("across {requested}"),
            None,
            Suggestion {
                auto: false,
                replacement: format!("on {} {}\nacross {legal}", shape.count, shape.kind),
                command: String::new(),
            },
        )];
    }
    let free = healthy.iter().filter(|node| node.free).count() as u32;
    if requested > free {
        return vec![Diagnostic {
            code: Code::PoolWaiting,
            severity: Severity::Note,
            message: format!("{free} of {healthy_count} machines are free"),
            span: Span {
                file: String::new(),
                line: 1,
                column: 1,
                text: format!("across {requested}"),
                label: None,
            },
            notes: Vec::new(),
            suggestions: Vec::new(),
        }];
    }
    Vec::new()
}

pub fn render_workload(spec: &Spec) -> String {
    let name = spec.name.as_deref().unwrap_or("kluge");
    let deadline = spec.duration.map(|d| d.as_secs()).unwrap_or(0);
    let pod = pod_template(spec, if spec.across <= 1 { 6 } else { 12 });
    if spec.across <= 1 {
        format!(
            "apiVersion: batch/v1\nkind: Job\nmetadata:\n  name: {name}\nspec:\n  activeDeadlineSeconds: {deadline}\n  template:\n    spec:\n{pod}"
        )
    } else {
        format!(
            "apiVersion: jobset.x-k8s.io/v1alpha2\nkind: JobSet\nmetadata:\n  name: {name}\nspec:\n  replicatedJobs:\n  - name: workers\n    replicas: {}\n    template:\n      spec:\n        parallelism: 1\n        completions: 1\n        activeDeadlineSeconds: {deadline}\n        template:\n          spec:\n{pod}",
            spec.across
        )
    }
}

fn pod_template(spec: &Spec, indent: usize) -> String {
    let i = " ".repeat(indent);
    let i2 = " ".repeat(indent + 2);
    let i4 = " ".repeat(indent + 4);
    let i6 = " ".repeat(indent + 6);
    let image = spec.image.as_deref().unwrap_or("");
    let dir = spec.dir.as_deref().unwrap_or("");
    let gpus = spec.on.as_ref().map(|on| on.count).unwrap_or(0);
    let mut body = String::new();
    body.push_str(&format!("{i}restartPolicy: Never\n"));
    body.push_str(&format!("{i}containers:\n"));
    body.push_str(&format!("{i}- name: kluge\n"));
    body.push_str(&format!("{i2}image: {image}\n"));
    body.push_str(&format!("{i2}workingDir: {dir}\n"));
    body.push_str(&format!("{i2}command:\n"));
    if spec.script.is_some() {
        for arg in ["bash", "--noprofile", "--norc", "/kluge/script"] {
            body.push_str(&format!("{i2}- {arg}\n"));
        }
    } else {
        for arg in &spec.run {
            body.push_str(&format!("{i2}- {arg}\n"));
        }
    }
    let memory = spec.memory.clone().unwrap_or_else(|| "16Gi".to_string());
    let shm = shm_size(&memory);
    let job = spec.name.as_deref().unwrap_or("kluge");
    body.push_str(&format!("{i2}env:\n"));
    for (key, value) in &spec.with {
        push_env(&mut body, &i2, &i4, key, value);
    }
    for (key, value) in allocation_env(spec, job, gpus) {
        push_env(&mut body, &i2, &i4, &key, &value);
    }
    if let Some(stdout) = &spec.stdout {
        let expanded = expand_file(stdout, job);
        let stderr = spec
            .stderr
            .as_deref()
            .map(|path| expand_file(path, job))
            .unwrap_or_else(|| expanded.clone());
        push_env(&mut body, &i2, &i4, "KLUGE_STDOUT", &expanded);
        push_env(&mut body, &i2, &i4, "KLUGE_STDERR", &stderr);
    } else if let Some(stderr) = &spec.stderr {
        let expanded = expand_file(stderr, job);
        push_env(&mut body, &i2, &i4, "KLUGE_STDERR", &expanded);
    }
    body.push_str(&format!("{i2}resources:\n"));
    body.push_str(&format!("{i4}requests:\n"));
    if let Some(cpu) = &spec.cpus {
        body.push_str(&format!("{i6}cpu: \"{cpu}\"\n"));
    }
    body.push_str(&format!("{i6}memory: {memory}\n"));
    body.push_str(&format!("{i6}nvidia.com/gpu: \"{gpus}\"\n"));
    body.push_str(&format!("{i4}limits:\n"));
    if let Some(cpu) = &spec.cpus {
        body.push_str(&format!("{i6}cpu: \"{cpu}\"\n"));
    }
    body.push_str(&format!("{i6}memory: {memory}\n"));
    body.push_str(&format!("{i6}nvidia.com/gpu: \"{gpus}\"\n"));
    body.push_str(&format!("{i2}volumeMounts:\n"));
    body.push_str(&format!("{i2}- name: team\n"));
    body.push_str(&format!("{i4}mountPath: {dir}\n"));
    body.push_str(&format!("{i2}- name: dshm\n"));
    body.push_str(&format!("{i4}mountPath: /dev/shm\n"));
    if let Some(on) = &spec.on {
        if !on.kind.is_empty() {
            body.push_str(&format!("{i}nodeSelector:\n"));
            body.push_str(&format!("{i2}kluge.dev/gpu: {}\n", on.kind));
        }
    }
    body.push_str(&format!("{i}tolerations:\n"));
    body.push_str(&format!("{i}- key: kluge.dev/pool\n"));
    body.push_str(&format!("{i2}operator: Equal\n"));
    body.push_str(&format!("{i2}value: reserved\n"));
    body.push_str(&format!("{i2}effect: NoSchedule\n"));
    body.push_str(&format!("{i}volumes:\n"));
    body.push_str(&format!("{i}- name: team\n"));
    body.push_str(&format!("{i2}persistentVolumeClaim:\n"));
    body.push_str(&format!("{i4}claimName: team\n"));
    body.push_str(&format!("{i}- name: dshm\n"));
    body.push_str(&format!("{i2}emptyDir:\n"));
    body.push_str(&format!("{i4}medium: Memory\n"));
    body.push_str(&format!("{i4}sizeLimit: {shm}\n"));
    body
}

fn push_env(body: &mut String, i2: &str, i4: &str, key: &str, value: &str) {
    body.push_str(&format!("{i2}- name: {key}\n"));
    body.push_str(&format!("{i4}value: {value}\n"));
}

fn allocation_env(spec: &Spec, job: &str, gpus: u32) -> Vec<(String, String)> {
    let nodes = spec.across.to_string();
    let cpus = spec.cpus.clone().unwrap_or_else(|| "1".to_string());
    let submit = spec.dir.clone().unwrap_or_default();
    let gpus = gpus.to_string();
    let pairs = [
        ("SLURM_JOB_ID", job.to_string()),
        ("SLURM_JOB_NAME", job.to_string()),
        ("SLURM_NNODES", nodes.clone()),
        ("SLURM_JOB_NUM_NODES", nodes.clone()),
        ("SLURM_NODEID", "0".to_string()),
        ("SLURM_PROCID", "0".to_string()),
        ("SLURM_LOCALID", "0".to_string()),
        ("SLURM_NTASKS", nodes.clone()),
        ("SLURM_CPUS_PER_TASK", cpus),
        ("SLURM_GPUS_ON_NODE", gpus),
        ("SLURM_SUBMIT_DIR", submit),
        ("RANK", "0".to_string()),
        ("WORLD_SIZE", nodes),
        ("LOCAL_RANK", "0".to_string()),
        ("MASTER_ADDR", job.to_string()),
        ("MASTER_PORT", "29500".to_string()),
    ];
    pairs
        .into_iter()
        .map(|(key, value)| (key.to_string(), value))
        .collect()
}

fn expand_file(pattern: &str, job: &str) -> String {
    pattern
        .replace("%j", job)
        .replace("%x", job)
        .replace("%n", "0")
}

fn shm_size(memory: &str) -> String {
    let bytes = parse_memory_bytes(memory).unwrap_or(16 * 1024 * 1024 * 1024);
    let gi = bytes / 2 / (1024 * 1024 * 1024);
    format!("{gi}Gi")
}

fn parse_memory_bytes(value: &str) -> Option<u64> {
    let value = value.trim();
    let split = value
        .find(|c: char| c.is_ascii_alphabetic())
        .unwrap_or(value.len());
    let (number, suffix) = value.split_at(split);
    let number: u64 = number.parse().ok()?;
    let mult = match suffix.to_ascii_lowercase().as_str() {
        "k" | "ki" | "kb" => 1024,
        "m" | "mi" | "mb" => 1024 * 1024,
        "g" | "gi" | "gb" => 1024 * 1024 * 1024,
        "t" | "ti" | "tb" => 1024 * 1024 * 1024 * 1024,
        _ => return None,
    };
    Some(number.saturating_mul(mult))
}

#[derive(Debug, Clone)]
pub struct PoolMachine {
    pub kind: String,
    pub gpus: u32,
    pub count: u32,
}

pub struct ApplyOutcome {
    pub diagnostics: Vec<Diagnostic>,
    pub submitted: bool,
}

pub fn suggestions_for(text: &str, pool: &[PoolMachine]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let trimmed = raw.trim();
        if trimmed == "#SBATCH --exclusive" {
            diagnostics.push(suggestion_diag(
                Code::ParseSbatch,
                "directive has no translation",
                line,
                trimmed,
                Some("reserved nodes are already exclusive".to_string()),
                Suggestion {
                    auto: true,
                    replacement: String::new(),
                    command: String::new(),
                },
            ));
            continue;
        }
        let Some((token, value)) = trimmed.split_once(char::is_whitespace) else {
            continue;
        };
        if trimmed.starts_with('#') {
            continue;
        }
        if matches!(match_phrase(token), Match::None) {
            if let Some(phrase) = closest_phrase(token) {
                diagnostics.push(suggestion_diag(
                    Code::ParseUnknown,
                    "unknown phrase",
                    line,
                    trimmed,
                    None,
                    Suggestion {
                        auto: true,
                        replacement: format!("{phrase} {value}"),
                        command: String::new(),
                    },
                ));
            } else if parse_across(value).is_ok() {
                diagnostics.push(suggestion_diag(
                    Code::ParseUnknown,
                    "unknown phrase",
                    line,
                    trimmed,
                    None,
                    Suggestion {
                        auto: true,
                        replacement: format!("across {value}"),
                        command: String::new(),
                    },
                ));
            }
        }
        if token == "on" || matches!(match_phrase(token), Match::One("on")) {
            if let Ok(shape) = parse_on(value) {
                if let Some(suggestion) = shape_suggestion(&shape, pool) {
                    diagnostics.push(suggestion_diag(
                        Code::PoolShape,
                        format!("each reserved node has {} {}", pool_gpus(pool, &shape.kind), shape.kind),
                        line,
                        trimmed,
                        None,
                        suggestion,
                    ));
                }
            }
        }
    }
    diagnostics
}

pub fn apply_auto(path: &std::path::Path, pool: &[PoolMachine]) -> ApplyOutcome {
    let original = std::fs::read_to_string(path).unwrap_or_default();
    let first = suggestions_for(&original, pool);
    let mut text = original;
    for diagnostic in &first {
        for suggestion in &diagnostic.suggestions {
            if !suggestion.auto {
                continue;
            }
            if suggestion.replacement.is_empty() {
                text = text
                    .lines()
                    .filter(|line| line.trim() != diagnostic.span.text.trim())
                    .collect::<Vec<_>>()
                    .join("\n");
                if !text.is_empty() && !text.ends_with('\n') {
                    text.push('\n');
                }
            } else if let Some(start) = text.find(&diagnostic.span.text) {
                let end = start + diagnostic.span.text.len();
                text.replace_range(start..end, &suggestion.replacement);
            }
        }
    }
    let _ = std::fs::write(path, &text);
    ApplyOutcome {
        diagnostics: suggestions_for(&text, pool),
        submitted: false,
    }
}

fn closest_phrase(token: &str) -> Option<&'static str> {
    let mut best: Vec<&str> = Vec::new();
    let mut best_distance = usize::MAX;
    for phrase in PHRASES {
        let distance = edit_distance(token, phrase);
        if distance == 0 || distance > 1 {
            continue;
        }
        if distance < best_distance {
            best_distance = distance;
            best.clear();
            best.push(phrase);
        } else if distance == best_distance {
            best.push(phrase);
        }
    }
    if best.len() == 1 {
        Some(best[0])
    } else {
        None
    }
}

fn edit_distance(left: &str, right: &str) -> usize {
    let left: Vec<char> = left.chars().collect();
    let right: Vec<char> = right.chars().collect();
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    for (i, left_char) in left.iter().enumerate() {
        let mut current = vec![i + 1];
        for (j, right_char) in right.iter().enumerate() {
            let cost = if left_char == right_char { 0 } else { 1 };
            current.push(
                (previous[j + 1] + 1)
                    .min(current[j] + 1)
                    .min(previous[j] + cost),
            );
        }
        previous = current;
    }
    previous[right.len()]
}

fn shape_suggestion(shape: &GpuShape, pool: &[PoolMachine]) -> Option<Suggestion> {
    let per_node = pool_gpus(pool, &shape.kind);
    if per_node == 0 || shape.count <= per_node || shape.count % per_node != 0 {
        return None;
    }
    let wanted = shape.count / per_node;
    let have = pool
        .iter()
        .filter(|machine| machine.kind == shape.kind)
        .map(|machine| machine.count)
        .sum::<u32>();
    if have == 0 {
        return None;
    }
    let (across, auto) = if wanted <= have {
        (wanted, true)
    } else {
        (have, false)
    };
    Some(Suggestion {
        auto,
        replacement: format!("on {per_node} {}\nacross {across}", shape.kind),
        command: String::new(),
    })
}

fn pool_gpus(pool: &[PoolMachine], kind: &str) -> u32 {
    pool.iter()
        .find(|machine| machine.kind == kind)
        .map(|machine| machine.gpus)
        .unwrap_or(0)
}

fn suggestion_diag(
    code: Code,
    message: impl Into<String>,
    line: usize,
    text: &str,
    note: Option<String>,
    suggestion: Suggestion,
) -> Diagnostic {
    Diagnostic {
        code,
        severity: Severity::Error,
        message: message.into(),
        span: Span {
            file: String::new(),
            line,
            column: 1,
            text: text.to_string(),
            label: None,
        },
        notes: note.into_iter().collect(),
        suggestions: vec![suggestion],
    }
}

pub fn check_local_layers(klugefile: &str, file: &str, cli: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let base = match lex_layer(klugefile, &mut diagnostics) {
        Some(phrases) => phrases,
        None => return diagnostics,
    };
    let file_phrases = match lex_layer(file, &mut diagnostics) {
        Some(phrases) => phrases,
        None => return diagnostics,
    };
    let cli_phrases = match lex_layer(cli, &mut diagnostics) {
        Some(phrases) => phrases,
        None => return diagnostics,
    };
    if !diagnostics.is_empty() {
        return diagnostics;
    }
    match resolve(&base, &file_phrases, &cli_phrases) {
        Err(ResolveError::MissingRun) => diagnostics.push(plain(Code::ParseRun, "run is missing", 1, "run")),
        Err(ResolveError::Value { phrase: "for" }) => {
            diagnostics.push(plain(Code::ParseDuration, "for did not parse", 1, "for"))
        }
        Err(ResolveError::Value { phrase }) => {
            diagnostics.push(plain(Code::ParseUnknown, "value is not valid", 1, phrase))
        }
        Ok(spec) => {
            if spec.duration.is_none() {
                diagnostics.push(plain(Code::ParseDuration, "for is missing", 1, "for"));
            }
            if spec.run.len() == 1 && is_local_script(&spec.run[0]) && !std::path::Path::new(&spec.run[0]).is_file()
            {
                diagnostics.push(plain(
                    Code::ParseRun,
                    "local script does not exist",
                    1,
                    spec.run[0].clone(),
                ));
            }
        }
    }
    diagnostics
}

pub fn check_local_sbatch(text: &str) -> Vec<Diagnostic> {
    match read_sbatch(text) {
        Err(error) => vec![plain(Code::ParseSbatch, error.message, error.line, error.text)],
        Ok(spec) => {
            let mut diagnostics = Vec::new();
            if spec.duration.is_none() {
                diagnostics.push(plain(Code::ParseDuration, "for is missing", 1, "for"));
            }
            diagnostics
        }
    }
}

fn lex_layer(text: &str, diagnostics: &mut Vec<Diagnostic>) -> Option<Vec<Phrase>> {
    match lex(text) {
        Ok(phrases) => Some(phrases),
        Err(LexError::Unknown { line, token }) => {
            diagnostics.push(plain(Code::ParseUnknown, "unknown phrase", line, token));
            None
        }
        Err(LexError::Ambiguous { line, names }) => {
            diagnostics.push(plain(
                Code::ParseAmbiguous,
                format!("matches {}", names.join(", ")),
                line,
                names.join(" "),
            ));
            None
        }
        Err(LexError::Duplicate {
            phrase,
            first_line,
            second_line,
        }) => {
            diagnostics.push(plain(
                Code::ParseDuplicate,
                format!("{phrase} is set on lines {first_line} and {second_line}"),
                second_line,
                phrase,
            ));
            None
        }
        Err(LexError::Unclosed { line }) => {
            diagnostics.push(plain(Code::ParseRun, "run block is not closed", line, "\"\"\""));
            None
        }
    }
}

fn is_local_script(token: &str) -> bool {
    token.starts_with("./") || token.starts_with('/') || token.starts_with("../")
}

fn plain(code: Code, message: impl Into<String>, line: usize, text: impl Into<String>) -> Diagnostic {
    Diagnostic {
        code,
        severity: Severity::Error,
        message: message.into(),
        span: Span {
            file: String::new(),
            line,
            column: 1,
            text: text.into(),
            label: None,
        },
        notes: Vec::new(),
        suggestions: Vec::new(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    pub code: Code,
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub notes: Vec<String>,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Code {
    #[serde(rename = "parse.unknown")]
    ParseUnknown,
    #[serde(rename = "parse.ambiguous")]
    ParseAmbiguous,
    #[serde(rename = "parse.duplicate")]
    ParseDuplicate,
    #[serde(rename = "parse.run")]
    ParseRun,
    #[serde(rename = "parse.duration")]
    ParseDuration,
    #[serde(rename = "parse.sbatch")]
    ParseSbatch,
    #[serde(rename = "pool.shape")]
    PoolShape,
    #[serde(rename = "pool.capacity")]
    PoolCapacity,
    #[serde(rename = "time.ceiling")]
    TimeCeiling,
    #[serde(rename = "pool.type")]
    PoolType,
    #[serde(rename = "pool.waiting")]
    PoolWaiting,
    #[serde(rename = "name.taken")]
    NameTaken,
    #[serde(rename = "name.syntax")]
    NameSyntax,
    #[serde(rename = "path.escape")]
    PathEscape,
    #[serde(rename = "path.missing")]
    PathMissing,
    #[serde(rename = "image.denied")]
    ImageDenied,
    #[serde(rename = "image.pull")]
    ImagePull,
    #[serde(rename = "auth.team")]
    AuthTeam,
    #[serde(rename = "auth.identity")]
    AuthIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Span {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub text: String,
    #[serde(default, skip_serializing)]
    pub label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Suggestion {
    pub auto: bool,
    pub replacement: String,
    pub command: String,
}

pub fn render(diagnostic: &Diagnostic) -> String {
    let code = serde_json::to_value(diagnostic.code)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .unwrap_or_default();
    let severity = match diagnostic.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Note => "note",
    };
    let width = diagnostic.span.line.to_string().len().max(2);
    let gutter = " ".repeat(width);
    let mut out = String::new();
    out.push_str(&format!("{severity}[{code}]: {}\n", diagnostic.message));
    out.push_str(&format!("  --> {}:{}\n", diagnostic.span.file, diagnostic.span.line));
    out.push_str(&format!("{gutter} |\n"));
    out.push_str(&format!(
        "{:>width$} | {}\n",
        diagnostic.span.line, diagnostic.span.text, width = width
    ));
    let caret_at = diagnostic.span.column.saturating_sub(1);
    let caret_len = diagnostic.span.text.len().saturating_sub(caret_at).max(1);
    let mut caret = format!("{gutter} | {:>pad$}", "", pad = caret_at);
    caret.push_str(&"^".repeat(caret_len));
    if let Some(label) = &diagnostic.span.label {
        caret.push(' ');
        caret.push_str(label);
    }
    out.push_str(&caret);
    out.push('\n');
    for note in &diagnostic.notes {
        out.push_str(&format!("{gutter} |\n"));
        out.push_str(&format!("{gutter} = note: {note}\n"));
    }
    if let Some(suggestion) = diagnostic.suggestions.first() {
        out.push_str(&format!("{gutter} |\n"));
        let mut replacement = suggestion.replacement.split('\n');
        if let Some(first) = replacement.next() {
            out.push_str(&format!(
                "{:>width$} - {}\n",
                diagnostic.span.line, diagnostic.span.text, width = width
            ));
            out.push_str(&format!(
                "{:>width$} + {first}\n",
                diagnostic.span.line, width = width
            ));
            let mut line = diagnostic.span.line;
            for extra in replacement {
                line += 1;
                out.push_str(&format!("{:>width$} + {extra}\n", line, width = width));
            }
        }
        out.push_str(&format!("{gutter} |\n"));
        out.push_str(&format!("  kluge run --fix {}\n", diagnostic.span.file));
    }
    out
}

pub fn sample_pool_shape() -> Diagnostic {
    Diagnostic {
        code: Code::PoolShape,
        severity: Severity::Error,
        message: "each reserved node has 8 h100".to_string(),
        span: Span {
            file: "ablation.kluge".to_string(),
            line: 2,
            column: 4,
            text: "on 32 h100".to_string(),
            label: Some("no machine has this shape".to_string()),
        },
        notes: vec!["32 GPUs is 4 machines of 8".to_string()],
        suggestions: vec![Suggestion {
            auto: true,
            replacement: "on 8 h100\nacross 4".to_string(),
            command: "kluge run train.py on 8 h100 across 4".to_string(),
        }],
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct SbatchError {
    pub line: usize,
    pub text: String,
    pub message: String,
}

pub fn read_sbatch(text: &str) -> Result<Spec, SbatchError> {
    let mut spec = Spec {
        run: Vec::new(),
        on: None,
        across: 1,
        duration: None,
        image: None,
        name: None,
        into: None,
        with: Vec::new(),
        team: None,
        dir: None,
        cpus: None,
        memory: None,
        script: None,
        stdout: None,
        stderr: None,
        overrides: Vec::new(),
    };
    let mut body = Vec::new();
    let mut in_body = false;
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let trimmed = raw.trim();
        if !in_body {
            if trimmed.is_empty() || trimmed.starts_with("#!") {
                continue;
            }
            if let Some(directive) = trimmed.strip_prefix("#SBATCH") {
                apply_sbatch(&mut spec, line, directive.trim())?;
                continue;
            }
            if trimmed.starts_with('#') {
                continue;
            }
            in_body = true;
        }
        body.push(raw);
    }
    let script = body.join("\n");
    if !script.is_empty() {
        spec.script = Some(script);
        spec.run.clear();
    }
    Ok(spec)
}

fn apply_sbatch(spec: &mut Spec, line: usize, directive: &str) -> Result<(), SbatchError> {
    let reject = || SbatchError {
        line,
        text: format!("#SBATCH {directive}"),
        message: "directive has no translation".to_string(),
    };
    let (flag, value) = split_sbatch_flag(directive).ok_or_else(reject)?;
    match flag.as_str() {
        "--job-name" => spec.name = Some(value),
        "--nodes" => {
            spec.across = value.parse().map_err(|_| reject())?;
        }
        "--gpus" | "--gpus-per-node" => {
            spec.on = Some(parse_gpu_request(&value).map_err(|_| reject())?);
        }
        "--gres" => {
            let rest = value.strip_prefix("gpu:").ok_or_else(reject)?;
            spec.on = Some(parse_gpu_request(rest).map_err(|_| reject())?);
        }
        "--cpus-per-task" => spec.cpus = Some(value),
        "--mem" => spec.memory = Some(value),
        "--time" => spec.duration = Some(parse_slurm_time(&value).map_err(|_| reject())?),
        "--account" => spec.team = Some(value),
        "--output" => spec.stdout = Some(value),
        "--error" => spec.stderr = Some(value),
        "--chdir" => spec.dir = Some(value),
        "--export" => {
            if value.split(',').any(|name| name.trim() == "ALL") {
                return Err(SbatchError {
                    line,
                    text: format!("#SBATCH {directive}"),
                    message: "the container starts clean".to_string(),
                });
            }
        }
        _ => return Err(reject()),
    }
    Ok(())
}

fn split_sbatch_flag(directive: &str) -> Option<(String, String)> {
    let directive = directive.trim();
    if !directive.starts_with("--") {
        return None;
    }
    if let Some((flag, value)) = directive.split_once('=') {
        Some((flag.to_string(), value.trim().to_string()))
    } else if let Some((flag, value)) = directive.split_once(char::is_whitespace) {
        Some((flag.to_string(), value.trim().to_string()))
    } else {
        None
    }
}

fn parse_gpu_request(value: &str) -> Result<GpuShape, ()> {
    let parts: Vec<&str> = value.split(':').collect();
    match parts.as_slice() {
        [count] => Ok(GpuShape {
            count: count.parse().map_err(|_| ())?,
            kind: String::new(),
        }),
        [kind, count] => Ok(GpuShape {
            count: count.parse().map_err(|_| ())?,
            kind: (*kind).to_string(),
        }),
        _ => Err(()),
    }
}

fn parse_slurm_time(value: &str) -> Result<std::time::Duration, ()> {
    let (days, clock) = match value.split_once('-') {
        Some((days, clock)) => (days.parse::<u64>().map_err(|_| ())?, clock),
        None => (0, value),
    };
    let parts: Vec<&str> = clock.split(':').collect();
    let seconds = match parts.as_slice() {
        [minutes] => minutes.parse::<u64>().map_err(|_| ())? * 60,
        [minutes, seconds] => {
            minutes.parse::<u64>().map_err(|_| ())? * 60
                + seconds.parse::<u64>().map_err(|_| ())?
        }
        [hours, minutes, seconds] => {
            hours.parse::<u64>().map_err(|_| ())? * 3600
                + minutes.parse::<u64>().map_err(|_| ())? * 60
                + seconds.parse::<u64>().map_err(|_| ())?
        }
        _ => return Err(()),
    };
    Ok(std::time::Duration::from_secs(
        days * 24 * 60 * 60 + seconds,
    ))
}

pub fn render_sentence(spec: &Spec) -> String {
    let mut lines = Vec::new();
    if !spec.run.is_empty() {
        lines.push(format!("run {}", spec.run.join(" ")));
    }
    if let Some(on) = &spec.on {
        if on.count == 1 {
            lines.push(format!("on {}", on.kind));
        } else {
            lines.push(format!("on {} {}", on.count, on.kind));
        }
    }
    lines.push(format!("across {}", spec.across));
    if let Some(duration) = spec.duration {
        lines.push(format!("for {}", format_duration(duration)));
    }
    if let Some(image) = &spec.image {
        lines.push(format!("use {image}"));
    }
    if let Some(name) = &spec.name {
        lines.push(format!("name {name}"));
    }
    if let Some(into) = &spec.into {
        lines.push(format!("into {into}"));
    }
    for (key, value) in &spec.with {
        lines.push(format!("with {key}={value}"));
    }
    if let Some(team) = &spec.team {
        lines.push(format!("team {team}"));
    }
    if let Some(dir) = &spec.dir {
        lines.push(format!("dir {dir}"));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn format_duration(duration: std::time::Duration) -> String {
    let seconds = duration.as_secs();
    if seconds % 3600 == 0 {
        let hours = seconds / 3600;
        if hours == 1 {
            "1 hour".to_string()
        } else {
            format!("{hours} hours")
        }
    } else {
        let minutes = seconds / 60;
        if minutes == 1 {
            "1 minute".to_string()
        } else {
            format!("{minutes} minutes")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Klugefile,
    File,
    Cli,
}

#[derive(Debug, PartialEq, Eq)]
pub struct OverrideNote {
    pub phrase: &'static str,
    pub from: Layer,
    pub to: Layer,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Spec {
    pub run: Vec<String>,
    pub on: Option<GpuShape>,
    pub across: u32,
    pub duration: Option<std::time::Duration>,
    pub image: Option<String>,
    pub name: Option<String>,
    pub into: Option<String>,
    pub with: Vec<(String, String)>,
    pub team: Option<String>,
    pub dir: Option<String>,
    pub cpus: Option<String>,
    pub memory: Option<String>,
    pub script: Option<String>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub overrides: Vec<OverrideNote>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ResolveError {
    MissingRun,
    Value { phrase: &'static str },
}

pub fn resolve(
    klugefile: &[Phrase],
    file: &[Phrase],
    cli: &[Phrase],
) -> Result<Spec, ResolveError> {
    let mut spec = Spec {
        run: Vec::new(),
        on: None,
        across: 1,
        duration: None,
        image: None,
        name: None,
        into: None,
        with: Vec::new(),
        team: None,
        dir: None,
        cpus: None,
        memory: None,
        script: None,
        stdout: None,
        stderr: None,
        overrides: Vec::new(),
    };
    let mut origin: Vec<(&'static str, Layer)> = Vec::new();
    apply(&mut spec, &mut origin, klugefile, Layer::Klugefile)?;
    apply(&mut spec, &mut origin, file, Layer::File)?;
    apply(&mut spec, &mut origin, cli, Layer::Cli)?;
    if spec.run.is_empty() && spec.script.is_none() {
        return Err(ResolveError::MissingRun);
    }
    Ok(spec)
}

fn apply(
    spec: &mut Spec,
    origin: &mut Vec<(&'static str, Layer)>,
    phrases: &[Phrase],
    layer: Layer,
) -> Result<(), ResolveError> {
    let mut with_from_this_layer = false;
    for phrase in phrases {
        if phrase.name == "with" && !with_from_this_layer {
            if origin
                .iter()
                .any(|(name, from)| *name == "with" && *from != layer)
            {
                spec.with.clear();
            }
            with_from_this_layer = true;
        }
        if let Some((_, from)) = origin.iter().find(|(name, _)| *name == phrase.name) {
            if *from != layer {
                spec.overrides.push(OverrideNote {
                    phrase: phrase.name,
                    from: *from,
                    to: layer,
                });
            }
        }
        if let Some(slot) = origin.iter_mut().find(|(name, _)| *name == phrase.name) {
            slot.1 = layer;
        } else {
            origin.push((phrase.name, layer));
        }
        set_phrase(spec, phrase)?;
    }
    Ok(())
}

fn set_phrase(spec: &mut Spec, phrase: &Phrase) -> Result<(), ResolveError> {
    let bad = || ResolveError::Value {
        phrase: phrase.name,
    };
    match phrase.name {
        "run" => {
            if phrase.script {
                spec.script = Some(phrase.value.clone());
                spec.run.clear();
            } else {
                spec.script = None;
                spec.run = phrase.value.split_whitespace().map(str::to_string).collect();
            }
        }
        "on" => spec.on = Some(parse_on(&phrase.value).map_err(|_| bad())?),
        "across" => spec.across = parse_across(&phrase.value).map_err(|_| bad())?,
        "for" => spec.duration = Some(parse_for(&phrase.value).map_err(|_| bad())?),
        "use" => spec.image = Some(phrase.value.clone()),
        "name" => spec.name = Some(phrase.value.clone()),
        "into" => spec.into = Some(phrase.value.clone()),
        "with" => spec.with.push(parse_with(&phrase.value).map_err(|_| bad())?),
        "team" => spec.team = Some(phrase.value.clone()),
        "dir" => spec.dir = Some(phrase.value.clone()),
        _ => {}
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub struct Phrase {
    pub line: usize,
    pub name: &'static str,
    pub value: String,
    pub script: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LexError {
    Unknown { line: usize, token: String },
    Ambiguous { line: usize, names: Vec<&'static str> },
    Duplicate {
        phrase: &'static str,
        first_line: usize,
        second_line: usize,
    },
    Unclosed { line: usize },
}

pub fn lex(text: &str) -> Result<Vec<Phrase>, LexError> {
    let lines: Vec<&str> = text.lines().collect();
    let mut phrases = Vec::new();
    let mut seen: Vec<(&'static str, usize)> = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = index + 1;
        let code = lines[index].split('#').next().unwrap_or("").trim();
        if code.is_empty() {
            index += 1;
            continue;
        }
        let (token, value) = code.split_once(char::is_whitespace).unwrap_or((code, ""));
        let name = match match_phrase(token) {
            Match::One(name) => name,
            Match::None => {
                return Err(LexError::Unknown {
                    line,
                    token: token.to_string(),
                })
            }
            Match::Ambiguous(names) => return Err(LexError::Ambiguous { line, names }),
        };
        if name == "run" && value.trim() == "\"\"\"" {
            let start = line;
            index += 1;
            let mut body = Vec::new();
            let mut closed = false;
            while index < lines.len() {
                if lines[index].trim() == "\"\"\"" {
                    closed = true;
                    index += 1;
                    break;
                }
                body.push(lines[index]);
                index += 1;
            }
            if !closed {
                return Err(LexError::Unclosed { line: start });
            }
            remember(&mut seen, name, start)?;
            phrases.push(Phrase {
                line: start,
                name,
                value: body.join("\n"),
                script: true,
            });
            continue;
        }
        remember(&mut seen, name, line)?;
        phrases.push(Phrase {
            line,
            name,
            value: value.trim().to_string(),
            script: false,
        });
        index += 1;
    }
    Ok(phrases)
}

fn remember(seen: &mut Vec<(&'static str, usize)>, name: &'static str, line: usize) -> Result<(), LexError> {
    if name == "with" {
        return Ok(());
    }
    if let Some((_, first_line)) = seen.iter().find(|(seen_name, _)| *seen_name == name) {
        return Err(LexError::Duplicate {
            phrase: name,
            first_line: *first_line,
            second_line: line,
        });
    }
    seen.push((name, line));
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
pub struct GpuShape {
    pub count: u32,
    pub kind: String,
}

pub fn parse_for(input: &str) -> Result<std::time::Duration, ()> {
    let input = input.trim();
    let (number, unit) = input
        .split_once(' ')
        .map(|(n, u)| (n, u))
        .or_else(|| {
            let split = input.find(|c: char| c.is_ascii_alphabetic())?;
            Some((&input[..split], &input[split..]))
        })
        .ok_or(())?;
    let count: u64 = number.parse().map_err(|_| ())?;
    let seconds = match unit {
        "h" | "hour" | "hours" => count.checked_mul(60 * 60).ok_or(())?,
        "m" | "minute" | "minutes" => count.checked_mul(60).ok_or(())?,
        _ => return Err(()),
    };
    Ok(std::time::Duration::from_secs(seconds))
}

pub fn parse_on(input: &str) -> Result<GpuShape, ()> {
    let input = input.trim();
    if let Some((count, kind)) = input.split_once(|c: char| c == 'x' || c == 'X') {
        if !kind.is_empty() && !kind.contains('x') && !kind.contains('X') {
            let count: u32 = count.parse().map_err(|_| ())?;
            if count > 0 {
                return Ok(GpuShape {
                    count,
                    kind: kind.to_string(),
                });
            }
        }
    }
    let mut parts = input.split_whitespace();
    let first = parts.next().ok_or(())?;
    match parts.next() {
        None => Ok(GpuShape {
            count: 1,
            kind: first.to_string(),
        }),
        Some(kind) if parts.next().is_none() => {
            let count: u32 = first.parse().map_err(|_| ())?;
            if count == 0 || kind.is_empty() {
                return Err(());
            }
            Ok(GpuShape {
                count,
                kind: kind.to_string(),
            })
        }
        _ => Err(()),
    }
}

pub fn parse_with(input: &str) -> Result<(String, String), ()> {
    let (key, value) = input.split_once('=').ok_or(())?;
    if key.is_empty() {
        return Err(());
    }
    Ok((key.to_string(), value.to_string()))
}

pub fn parse_across(input: &str) -> Result<u32, ()> {
    let count: u32 = input.trim().parse().map_err(|_| ())?;
    if count == 0 {
        Err(())
    } else {
        Ok(count)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Match {
    One(&'static str),
    None,
    Ambiguous(Vec<&'static str>),
}

pub const VERBS: &[&str] = &[
    "run", "status", "logs", "cancel", "watch", "explain", "debug", "attach",
];

pub const PHRASES: &[&str] = &[
    "run", "on", "across", "for", "use", "name", "into", "with", "team", "dir",
];

const VERB_SYNONYMS: &[(&str, &str)] = &[("ssh", "attach"), ("stop", "cancel")];

const PHRASE_SYNONYMS: &[(&str, &str)] = &[("from", "use"), ("as", "name")];

pub fn match_verb(token: &str) -> Match {
    match_token(token, VERBS, VERB_SYNONYMS)
}

pub fn match_phrase(token: &str) -> Match {
    match_token(token.trim_start_matches('-'), PHRASES, PHRASE_SYNONYMS)
}

fn match_token(token: &str, names: &[&'static str], synonyms: &[(&str, &'static str)]) -> Match {
    let mut hits: Vec<&'static str> = names
        .iter()
        .copied()
        .filter(|name| name.starts_with(token))
        .collect();
    match hits.len() {
        1 => Match::One(hits.remove(0)),
        n if n > 1 => Match::Ambiguous(hits),
        _ => synonyms
            .iter()
            .find(|(word, _)| *word == token)
            .map(|(_, canonical)| Match::One(*canonical))
            .unwrap_or(Match::None),
    }
}
