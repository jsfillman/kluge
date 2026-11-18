use kluge_check::{lex, resolve, AdmitView, FsView, NodeFact};
use kluge_cli::main_with;
use kluge_controller::{open, Run};

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_string()).collect()
}

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

const SUBMIT: &[&str] = &[
    "kluge",
    "run",
    "train.py",
    "on",
    "8",
    "h100",
    "for",
    "1h",
    "name",
    "sample",
    "use",
    "registry.example.com/train:1",
    "team",
    "kernel-search",
    "dir",
    "/scratch/team",
];

#[test]
fn api_submits_lists_logs_and_cancels() {
    let handle = open(context()).unwrap();
    let base = format!("http://{}", handle.addr);
    let (code, text) = main_with(&args(SUBMIT), Some(&base));
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("run train.py"), "{text}");
    assert!(text.contains("sample"), "{text}");
    let (code, text) = main_with(&args(&["kluge", "status"]), Some(&base));
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("sample"), "{text}");
    handle.state.lock().unwrap().runs[0].log = "epoch 1".to_string();
    let (code, text) = main_with(&args(&["kluge", "logs", "sample"]), Some(&base));
    assert_eq!(code, 0, "{text}");
    assert_eq!(text, "epoch 1");
    let (code, text) = main_with(&args(&["kluge", "cancel", "sample"]), Some(&base));
    assert_eq!(code, 0, "{text}");
    assert_eq!(text, "sample");
    assert!(handle
        .state
        .lock()
        .unwrap()
        .cluster
        .deleted
        .iter()
        .any(|name| name == "sample"));
}

#[test]
fn api_refuses_a_bad_sentence() {
    let handle = open(context()).unwrap();
    let base = format!("http://{}", handle.addr);
    let (code, text) = main_with(
        &args(&[
            "kluge", "run", "train.py", "on", "32", "h100", "for", "1h", "name", "sample", "use",
            "registry.example.com/train:1", "team", "kernel-search", "dir", "/scratch/team",
        ]),
        Some(&base),
    );
    assert_eq!(code, 1, "{text}");
    assert!(handle.state.lock().unwrap().runs.is_empty(), "{text}");
}

#[test]
fn no_address_does_not_submit() {
    let (code, text) = main_with(&args(&["kluge", "run", "train.py", "on", "1", "h100", "for", "1h"]), None);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("run train.py"), "{text}");
    let (code, text) = main_with(&args(&["kluge", "status"]), None);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("needs a cluster"), "{text}");
}
