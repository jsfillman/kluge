use kluge_check::{lex, resolve, AdmitView, FsView, NodeFact};
use kluge_controller::{open, sync, HttpCluster, MemoryCluster, Run};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

fn context(text: &str) -> Run {
    Run {
        spec: resolve(&[], &lex(text).unwrap(), &[]).unwrap(),
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

const GOOD: &str = "run train.py\non 8 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname sample\ndir /scratch/team\n";
const BAD: &str = "run train.py\non 32 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname sample\ndir /scratch/team\n";

fn exchange(addr: std::net::SocketAddr, method: &str, path: &str, body: &str) -> (u16, String) {
    let mut stream = std::net::TcpStream::connect(addr).unwrap();
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(request.as_bytes()).unwrap();
    let mut buf = String::new();
    stream.read_to_string(&mut buf).unwrap();
    let (head, body) = buf.split_once("\r\n\r\n").unwrap_or(("", &buf));
    let status = head.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
    (status, body.to_string())
}

#[test]
fn sync_creates_one_yaml_and_skips_a_refusal() {
    let mut cluster = MemoryCluster::default();
    assert!(sync(&context(GOOD), &mut cluster));
    sync(&context(GOOD), &mut cluster);
    assert_eq!(cluster.created.len(), 1, "{:?}", cluster.created);
    assert!(cluster.created[0].contains("kind: Job"), "{}", cluster.created[0]);
    let mut refused = MemoryCluster::default();
    assert!(!sync(&context(BAD), &mut refused));
    assert!(refused.created.is_empty());
}

#[test]
fn runs_round_trip() {
    let handle = open(context(GOOD)).unwrap();
    let spec = serde_json::to_string(&context(GOOD).spec).unwrap();
    let (status, body) = exchange(handle.addr, "POST", "/runs", &spec);
    assert_eq!(status, 200, "{body}");
    assert!(body.contains("sample"), "{body}");
    let (status, listed) = exchange(handle.addr, "GET", "/runs", "");
    assert_eq!(status, 200, "{listed}");
    assert!(listed.contains("sample"), "{listed}");
    handle.state.lock().unwrap().runs[0].log = "epoch 1".to_string();
    let (status, log) = exchange(handle.addr, "GET", "/runs/sample/logs", "");
    assert_eq!(status, 200, "{log}");
    assert_eq!(log, "epoch 1");
    let (status, name) = exchange(handle.addr, "DELETE", "/runs/sample", "");
    assert_eq!(status, 200, "{name}");
    assert_eq!(name, "sample");
    assert!(handle.state.lock().unwrap().cluster.deleted.iter().any(|item| item == "sample"));
}

#[test]
fn refused_run_is_not_stored() {
    let handle = open(context(GOOD)).unwrap();
    let spec = serde_json::to_string(&context(BAD).spec).unwrap();
    let (status, body) = exchange(handle.addr, "POST", "/runs", &spec);
    assert_eq!(status, 400, "{body}");
    let (status, listed) = exchange(handle.addr, "GET", "/runs", "");
    assert_eq!(status, 200, "{listed}");
    assert!(!listed.contains("sample"), "{listed}");
}

#[test]
fn http_cluster_posts_each_document() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let got = Arc::new(Mutex::new(Vec::new()));
    let record = got.clone();
    thread::spawn(move || {
        for stream in listener.incoming().take(2) {
            let mut stream = stream.unwrap();
            let mut buf = [0u8; 8192];
            let n = stream.read(&mut buf).unwrap_or(0);
            let text = String::from_utf8_lossy(&buf[..n]).to_string();
            record.lock().unwrap().push(text);
            let _ = stream.write_all(b"HTTP/1.1 201 Created\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
        }
    });
    let cluster = HttpCluster { base: format!("http://{addr}") };
    let yaml = "apiVersion: batch/v1\nkind: Job\nmetadata:\n  name: sample\n---\napiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: sample-kluge\ndata:\n  hosts: |\n    sample-0\n";
    cluster.create(yaml).unwrap();
    let got = got.lock().unwrap();
    assert!(got[0].contains("/apis/batch/v1/namespaces/kluge/jobs"), "{}", got[0]);
    assert!(got[0].contains("kind: Job"), "{}", got[0]);
    assert!(got[1].contains("/api/v1/namespaces/kluge/configmaps"), "{}", got[1]);
}
