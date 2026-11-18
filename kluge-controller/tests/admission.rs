use kluge_check::{lex, resolve, AdmitView, Code, FsView, NodeFact};
use kluge_controller::{admit, Run};

fn run(text: &str) -> Run {
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

#[test]
fn rejects_bad_shape_with_pool_shape() {
    let decision = admit(&run("run train.py\non 32 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname ok\ndir /scratch/team\n"));
    assert!(!decision.accepted);
    assert_eq!(decision.diagnostics[0].code, Code::PoolShape);
    assert!(decision.summary.contains("pool.shape"), "{}", decision.summary);
}

#[test]
fn accepts_a_fitting_run() {
    let decision = admit(&run("run train.py\non 8 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname ok\ndir /scratch/team\n"));
    assert!(decision.accepted, "{:?}", decision.diagnostics);
    assert!(decision.summary.is_empty());
}
