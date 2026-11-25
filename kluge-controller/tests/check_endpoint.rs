use kluge_check::{lex, resolve, AdmitView, FsView, NodeFact};
use kluge_controller::{handle_check, Run};

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

#[test]
fn post_returns_pool_shape_json() {
    let body = r#"{"text":"run train.py\non 32 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname ok\ndir /scratch/team\n"}"#;
    let (status, response) = handle_check("POST", body, &context());
    assert_eq!(status, 200);
    assert!(response.contains("\"pool.shape\""), "{response}");
    let empty = r#"{"text":"run train.py\non 8 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname ok\ndir /scratch/team\n"}"#;
    let (status, response) = handle_check("POST", empty, &context());
    assert_eq!(status, 200);
    assert_eq!(response, "[]");
}

#[test]
fn bad_body_is_400() {
    let (status, _) = handle_check("POST", "not-json", &context());
    assert_eq!(status, 400);
}
