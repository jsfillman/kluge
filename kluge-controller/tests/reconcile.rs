use kluge_check::{lex, resolve, AdmitView, FsView, NodeFact};
use kluge_controller::{reconcile, Run, Store};

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
fn accepted_run_creates_one_job() {
    let mut store = Store { created: Vec::new() };
    let run = run("run train.py\non 8 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname ok\ndir /scratch/team\n");
    reconcile(&run, &mut store);
    reconcile(&run, &mut store);
    assert_eq!(store.created.len(), 1);
    assert!(store.created[0].contains("kind: Job"), "{}", store.created[0]);
    assert!(store.created[0].contains("name: ok"), "{}", store.created[0]);
}

#[test]
fn rejected_run_creates_nothing() {
    let mut store = Store { created: Vec::new() };
    let run = run("run train.py\non 32 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname ok\ndir /scratch/team\n");
    reconcile(&run, &mut store);
    assert!(store.created.is_empty(), "{:?}", store.created);
}
