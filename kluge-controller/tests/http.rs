use kluge_check::{lex, resolve, AdmitView, FsView, NodeFact};
use kluge_controller::{serve, Run};
use std::io::{Read, Write};
use std::net::TcpStream;

fn context() -> Run {
    Run {
        spec: resolve(&[], &lex("run t\nfor 1h\n").unwrap(), &[]).unwrap(),
        view: AdmitView {
            allow: vec!["registry.example.com/".to_string()],
            require_digest: false,
            pull_ok: true,
            live_names: Vec::new(),
            teams: vec!["kernel-search".to_string()],
            caller: Some("ada".to_string()),
            max_wall_secs: 8 * 60 * 60,
        },
        nodes: (0..8)
            .map(|_| NodeFact {
                kind: "h100".to_string(),
                gpus: 8,
                healthy: true,
                free: true,
            })
            .collect(),
        quota: 64,
        fs: FsView {
            root: "/scratch/team".to_string(),
            existing: vec!["/scratch/team".to_string()],
            links: Vec::new(),
        },
    }
}

fn post(addr: std::net::SocketAddr, path: &str, body: &str) -> (u16, String) {
    let mut stream = TcpStream::connect(addr).unwrap();
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(request.as_bytes()).unwrap();
    let mut buf = String::new();
    stream.read_to_string(&mut buf).unwrap();
    let (head, body) = buf.split_once("\r\n\r\n").unwrap_or(("", &buf));
    let status = head
        .split_whitespace()
        .nth(1)
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);
    (status, body.to_string())
}

const BAD: &str = "run train.py\non 32 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname ok\ndir /scratch/team\n";
const GOOD: &str = "run train.py\non 8 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname ok\ndir /scratch/team\n";

#[test]
fn check_returns_the_diagnostic() {
    let addr = serve(context()).unwrap();
    let (status, body) = post(addr, "/check", &format!("{{\"text\":{}}}", serde_json::to_string(BAD).unwrap()));
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("pool.shape"), "{body}");
}

#[test]
fn admit_refuses_in_the_status_line() {
    let addr = serve(context()).unwrap();
    let review = serde_json::json!({
        "apiVersion": "admission.k8s.io/v1",
        "kind": "AdmissionReview",
        "request": {"uid": "abc", "object": {"spec": {"text": BAD}}}
    });
    let (status, body) = post(addr, "/admit", &review.to_string());
    assert_eq!(status, 200, "{body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["response"]["allowed"], false);
    assert_eq!(parsed["response"]["uid"], "abc");
    assert!(
        parsed["response"]["status"]["message"]
            .as_str()
            .unwrap_or("")
            .contains("pool.shape"),
        "{body}"
    );
}

#[test]
fn admit_allows_a_fitting_run() {
    let addr = serve(context()).unwrap();
    let review = serde_json::json!({
        "request": {"uid": "ok", "object": {"spec": {"text": GOOD}}}
    });
    let (status, body) = post(addr, "/admit", &review.to_string());
    assert_eq!(status, 200, "{body}");
    let parsed: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(parsed["response"]["allowed"], true, "{body}");
}
