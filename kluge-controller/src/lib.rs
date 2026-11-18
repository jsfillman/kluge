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

pub fn serve(context: Run) -> std::io::Result<std::net::SocketAddr> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let context = std::sync::Arc::new(std::sync::Mutex::new(context));
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let context = std::sync::Arc::clone(&context);
                let _ = std::thread::spawn(move || {
                    if let Some((method, path, body)) = read_http(&mut stream) {
                        let (status, response) = dispatch(&method, &path, &body, &context);
                        let reason = if status == 200 { "OK" } else { "Bad Request" };
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
    Ok(addr)
}

fn dispatch(
    method: &str,
    path: &str,
    body: &str,
    context: &std::sync::Mutex<Run>,
) -> (u16, String) {
    let context = context.lock().expect("run context");
    match (method, path) {
        ("POST", "/check") => handle_check(method, body, &context),
        ("POST", "/admit") => admit_review(body, &context),
        _ => (404, String::new()),
    }
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
