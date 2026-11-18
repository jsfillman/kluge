use kluge_check::{AdmitView, Diagnostic, FsView, NodeFact, Spec};
use std::io::Read;

pub struct Run {
    pub spec: Spec,
    pub view: AdmitView,
    pub nodes: Vec<NodeFact>,
    pub quota: u32,
    pub fs: FsView,
}

pub struct Admission {
    pub accepted: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub summary: String,
}

pub fn handle_check(method: &str, body: &str, run: &Run) -> (u16, String) {
    if method != "POST" {
        return (400, "not a spec".to_string());
    }
    let value: serde_json::Value = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(_) => return (400, "not a spec".to_string()),
    };
    let Some(text) = value.get("text").and_then(|item| item.as_str()) else {
        return (400, "not a spec".to_string());
    };
    let phrases = match kluge_check::lex(text) {
        Ok(phrases) => phrases,
        Err(_) => return (400, "not a spec".to_string()),
    };
    let spec = match kluge_check::resolve(&[], &phrases, &[]) {
        Ok(spec) => spec,
        Err(_) => return (400, "not a spec".to_string()),
    };
    let mut checked = Run {
        spec,
        view: run.view.clone(),
        nodes: run.nodes.clone(),
        quota: run.quota,
        fs: run.fs.clone(),
    };
    let _ = &mut checked;
    let decision = admit(&checked);
    (
        200,
        serde_json::to_string(&decision.diagnostics).unwrap_or_else(|_| "[]".to_string()),
    )
}

pub struct Store {
    pub created: Vec<String>,
}

pub fn reconcile(run: &Run, store: &mut Store) {
    if !admit(run).accepted {
        return;
    }
    let name = run.spec.name.clone().unwrap_or_else(|| "kluge".to_string());
    if store
        .created
        .iter()
        .any(|yaml| yaml.contains(&format!("name: {name}")))
    {
        return;
    }
    store.created.push(kluge_check::render_workload(&run.spec));
}

#[derive(Default)]
pub struct MemoryCluster {
    pub created: Vec<String>,
    pub deleted: Vec<String>,
}

pub struct StoredRun {
    pub name: String,
    pub yaml: String,
    pub log: String,
    pub deleted: bool,
}

pub struct State {
    pub context: Run,
    pub runs: Vec<StoredRun>,
    pub cluster: MemoryCluster,
}

pub struct Handle {
    pub addr: std::net::SocketAddr,
    pub state: std::sync::Arc<std::sync::Mutex<State>>,
}

pub struct HttpCluster {
    pub base: String,
}

impl HttpCluster {
    pub fn create(&self, yaml: &str) -> Result<(), String> {
        for doc in yaml.split("\n---\n") {
            let path = if doc.contains("kind: JobSet") {
                "/apis/jobset.x-k8s.io/v1alpha2/namespaces/kluge/jobsets"
            } else if doc.contains("kind: ConfigMap") {
                "/api/v1/namespaces/kluge/configmaps"
            } else {
                "/apis/batch/v1/namespaces/kluge/jobs"
            };
            post_text(&self.base, path, doc)?;
        }
        Ok(())
    }
}

pub fn sync(run: &Run, cluster: &mut MemoryCluster) -> bool {
    if !admit(run).accepted {
        return false;
    }
    let name = run.spec.name.clone().unwrap_or_else(|| "kluge".to_string());
    let marker = format!("name: {name}\n");
    if cluster.created.iter().any(|yaml| yaml.contains(&marker)) {
        return true;
    }
    cluster
        .created
        .push(kluge_check::render_workload(&run.spec));
    let _ = name;
    true
}

pub fn open(context: Run) -> std::io::Result<Handle> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let state = std::sync::Arc::new(std::sync::Mutex::new(State {
        context,
        runs: Vec::new(),
        cluster: MemoryCluster::default(),
    }));
    let shared = std::sync::Arc::clone(&state);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let shared = std::sync::Arc::clone(&shared);
                let _ = std::thread::spawn(move || {
                    if let Some((method, path, body)) = read_http(&mut stream) {
                        let (status, response) = dispatch(&method, &path, &body, &shared);
                        let reason = match status {
                            200 => "OK",
                            404 => "Not Found",
                            _ => "Bad Request",
                        };
                        let raw = format!(
                            "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{response}",
                            response.len()
                        );
                        let _ = std::io::Write::write_all(&mut stream, raw.as_bytes());
                    }
                });
            }
        }
    });
    Ok(Handle { addr, state })
}

pub fn serve(context: Run) -> std::io::Result<std::net::SocketAddr> {
    Ok(open(context)?.addr)
}

fn dispatch(
    method: &str,
    path: &str,
    body: &str,
    state: &std::sync::Mutex<State>,
) -> (u16, String) {
    let mut state = state.lock().expect("api state");
    match (method, path) {
        ("POST", "/check") => handle_check(method, body, &state.context),
        ("POST", "/admit") => admit_review(body, &state.context),
        ("POST", "/runs") => post_run(body, &mut state),
        ("GET", "/runs") => (
            200,
            state
                .runs
                .iter()
                .filter(|run| !run.deleted)
                .map(|run| run.name.clone())
                .collect::<Vec<_>>()
                .join("\n"),
        ),
        ("GET", path) if path.starts_with("/runs/") && path.ends_with("/logs") => {
            let name = &path["/runs/".len()..path.len() - "/logs".len()];
            match state.runs.iter().find(|run| run.name == name) {
                Some(run) => (200, run.log.clone()),
                None => (404, String::new()),
            }
        }
        ("DELETE", path) if path.starts_with("/runs/") => {
            let name = path.trim_start_matches("/runs/").to_string();
            state.cluster.deleted.push(name.clone());
            if let Some(run) = state.runs.iter_mut().find(|run| run.name == name) {
                run.deleted = true;
            }
            (200, name)
        }
        _ => (404, String::new()),
    }
}

fn post_run(body: &str, state: &mut State) -> (u16, String) {
    let spec: Spec = match serde_json::from_str(body) {
        Ok(spec) => spec,
        Err(err) => return (400, err.to_string()),
    };
    let name = spec.name.clone().unwrap_or_else(|| "kluge".to_string());
    let run = Run {
        spec,
        view: state.context.view.clone(),
        nodes: state.context.nodes.clone(),
        quota: state.context.quota,
        fs: state.context.fs.clone(),
    };
    if !sync(&run, &mut state.cluster) {
        let decision = admit(&run);
        return (
            400,
            serde_json::to_string(&decision.diagnostics).unwrap_or_else(|_| "[]".to_string()),
        );
    }
    if !state.runs.iter().any(|item| item.name == name) {
        let yaml = state.cluster.created.last().cloned().unwrap_or_default();
        state.runs.push(StoredRun {
            name: name.clone(),
            yaml,
            log: String::new(),
            deleted: false,
        });
    }
    (200, serde_json::json!({ "name": name }).to_string())
}

fn post_text(base: &str, path: &str, body: &str) -> Result<(), String> {
    let host = base.trim_start_matches("http://");
    let mut stream = std::net::TcpStream::connect(host).map_err(|err| err.to_string())?;
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    use std::io::{Read, Write};
    Write::write_all(&mut stream, request.as_bytes()).map_err(|err| err.to_string())?;
    let mut buf = [0u8; 64];
    let _ = stream.read(&mut buf);
    Ok(())
}

fn admit_review(body: &str, context: &Run) -> (u16, String) {
    let value: serde_json::Value = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(_) => return (400, "not a review".to_string()),
    };
    let uid = value["request"]["uid"].as_str().unwrap_or("").to_string();
    let text = value["request"]["object"]["spec"]["text"]
        .as_str()
        .unwrap_or("");
    let decision = decide_text(text, context);
    let response = serde_json::json!({
        "apiVersion": "admission.k8s.io/v1",
        "kind": "AdmissionReview",
        "response": {
            "uid": uid,
            "allowed": decision.accepted,
            "status": {"message": decision.summary}
        }
    });
    (200, response.to_string())
}

fn decide_text(text: &str, context: &Run) -> Admission {
    let spec = match kluge_check::lex(text) {
        Ok(phrases) => match kluge_check::resolve(&[], &phrases, &[]) {
            Ok(spec) => spec,
            Err(_) => return refused("parse.run"),
        },
        Err(_) => return refused("parse.run"),
    };
    admit(&Run {
        spec,
        view: context.view.clone(),
        nodes: context.nodes.clone(),
        quota: context.quota,
        fs: context.fs.clone(),
    })
}

fn refused(message: &str) -> Admission {
    Admission {
        accepted: false,
        diagnostics: Vec::new(),
        summary: message.to_string(),
    }
}

fn read_http(stream: &mut std::net::TcpStream) -> Option<(String, String, String)> {
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(2)))
        .ok()?;
    let mut buf = Vec::new();
    let mut tmp = [0u8; 8192];
    let mut header_end = None;
    loop {
        let n = stream.read(&mut tmp).unwrap_or(0);
        if n == 0 && buf.is_empty() {
            return None;
        }
        if n > 0 {
            buf.extend_from_slice(&tmp[..n]);
        }
        if header_end.is_none() {
            header_end = buf.windows(4).position(|window| window == b"\r\n\r\n");
        }
        if let Some(pos) = header_end {
            let headers = String::from_utf8_lossy(&buf[..pos]);
            if buf.len() >= pos + 4 + content_length(&headers) || n == 0 {
                let head = headers.to_string();
                let body = String::from_utf8_lossy(&buf[pos + 4..]).to_string();
                let mut parts = head.lines().next().unwrap_or("").split_whitespace();
                let method = parts.next().unwrap_or("").to_string();
                let path = parts.next().unwrap_or("").to_string();
                return Some((method, path, body));
            }
        }
        if n == 0 {
            return None;
        }
    }
}

fn content_length(headers: &str) -> usize {
    headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("content-length") {
                value.trim().parse().ok()
            } else {
                None
            }
        })
        .unwrap_or(0)
}

pub fn admit(run: &Run) -> Admission {
    let mut diagnostics = Vec::new();
    diagnostics.extend(kluge_check::check_pool(&run.spec, &run.nodes, run.quota));
    diagnostics.extend(kluge_check::check_paths(&run.spec, &run.fs));
    diagnostics.extend(kluge_check::check_admit(&run.spec, &run.view));
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|item| item.severity == kluge_check::Severity::Error)
        .collect();
    let summary = errors
        .first()
        .map(|item| {
            let code = serde_json::to_value(item.code)
                .ok()
                .and_then(|value| value.as_str().map(str::to_string))
                .unwrap_or_default();
            let command = item
                .suggestions
                .first()
                .map(|suggestion| suggestion.command.clone())
                .unwrap_or_default();
            format!("{code} {command}").trim().to_string()
        })
        .unwrap_or_default();
    Admission {
        accepted: errors.is_empty(),
        diagnostics,
        summary,
    }
}
