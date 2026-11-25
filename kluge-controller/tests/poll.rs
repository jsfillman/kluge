use kluge_check::{lex, resolve, AdmitView, FsView};
use kluge_controller::{poll_once, Run, State};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

fn state() -> State {
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
        apiserver: None,
    }
}

fn serve(nodes: &'static str, runs: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut buf = [0u8; 2048];
            let n = stream.read(&mut buf).unwrap_or(0);
            let text = String::from_utf8_lossy(&buf[..n]);
            let path = text.split_whitespace().nth(1).unwrap_or("");
            let body = if path == "/api/v1/nodes" {
                nodes
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
    format!("http://{addr}")
}

const NODES: &str = r#"{"items":[{"metadata":{"name":"n1","labels":{"kluge.dev/gpu":"h100"}},"spec":{"taints":[{"key":"kluge.dev/pool","value":"reserved","effect":"NoSchedule"}]},"status":{"allocatable":{"nvidia.com/gpu":"8"}}}]}"#;

#[test]
fn poll_loads_the_pool_and_syncs_one_run() {
    let runs = r#"{"items":[{"metadata":{"name":"sample"},"spec":{"text":"run train.py\non 8 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname sample\ndir /scratch/team\n"}}]}"#;
    let base = serve(NODES, runs);
    let mut state = state();
    poll_once(&base, &mut state).unwrap();
    assert_eq!(state.context.nodes.len(), 1);
    assert_eq!(state.cluster.created.len(), 1, "{:?}", state.cluster.created);
    poll_once(&base, &mut state).unwrap();
    assert_eq!(state.cluster.created.len(), 1);
}

#[test]
fn poll_skips_a_run_the_pool_cannot_hold() {
    let runs = r#"{"items":[{"metadata":{"name":"too-big"},"spec":{"text":"run train.py\non 32 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname too-big\ndir /scratch/team\n"}}]}"#;
    let base = serve(NODES, runs);
    let mut state = state();
    poll_once(&base, &mut state).unwrap();
    assert!(state.cluster.created.is_empty(), "{:?}", state.cluster.created);
}
