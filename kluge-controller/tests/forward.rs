use kluge_check::{lex, resolve, AdmitView, FsView};
use kluge_controller::{poll_once, Run, State};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

fn state(base: String) -> State {
    State {
        context: Run {
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
            nodes: Vec::new(),
            quota: 64,
            fs: FsView {
                root: "/scratch/team".to_string(),
                existing: vec!["/scratch/team".to_string()],
                links: Vec::new(),
            },
        },
        runs: Vec::new(),
        cluster: kluge_controller::MemoryCluster::default(),
        apiserver: Some(base),
    }
}

fn serve(runs: &'static str) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let posted = Arc::new(Mutex::new(Vec::new()));
    let record = posted.clone();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut buf = [0u8; 65536];
            let n = stream.read(&mut buf).unwrap_or(0);
            let text = String::from_utf8_lossy(&buf[..n]).to_string();
            let path = text.split_whitespace().nth(1).unwrap_or("");
            let method = text.split_whitespace().next().unwrap_or("");
            let body = if method == "POST" {
                record.lock().unwrap().push(text);
                ""
            } else if path == "/api/v1/nodes" {
                r#"{"items":[{"metadata":{"name":"n1","labels":{"kluge.dev/gpu":"h100"}},"spec":{"taints":[{"key":"kluge.dev/pool","value":"reserved","effect":"NoSchedule"}]},"status":{"allocatable":{"nvidia.com/gpu":"8"}}}]}"#
            } else if path == "/api/v1/pods" {
                r#"{"items":[]}"#
            } else {
                runs
            };
            let raw = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(raw.as_bytes());
        }
    });
    (format!("http://{addr}"), posted)
}

const GOOD: &str = r#"{"items":[{"metadata":{"name":"sample"},"spec":{"text":"run train.py\non 8 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname sample\ndir /scratch/team\n"}}]}"#;

#[test]
fn poll_posts_a_new_run_once() {
    let (base, posted) = serve(GOOD);
    let mut state = state(base);
    poll_once(state.apiserver.clone().unwrap().as_str(), &mut state).unwrap();
    assert_eq!(state.runs.len(), 1);
    assert_eq!(state.runs[0].name, "sample");
    let first = posted.lock().unwrap().iter().filter(|item| item.contains("kind: Job")).count();
    assert!(first >= 1, "{:?}", posted.lock().unwrap());
    poll_once(state.apiserver.clone().unwrap().as_str(), &mut state).unwrap();
    let second = posted.lock().unwrap().iter().filter(|item| item.contains("kind: Job")).count();
    assert_eq!(first, second);
}

#[test]
fn poll_does_not_post_a_run_the_pool_cannot_hold() {
    let runs = r#"{"items":[{"metadata":{"name":"too-big"},"spec":{"text":"run train.py\non 32 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname too-big\ndir /scratch/team\n"}}]}"#;
    let (base, posted) = serve(runs);
    let mut state = state(base);
    poll_once(state.apiserver.clone().unwrap().as_str(), &mut state).unwrap();
    assert!(state.runs.is_empty());
    assert!(posted.lock().unwrap().is_empty(), "{:?}", posted.lock().unwrap());
}
