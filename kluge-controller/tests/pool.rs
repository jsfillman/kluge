use kluge_check::{lex, resolve, AdmitView, FsView};
use kluge_controller::{apply_pool, Run, State};

#[test]
fn apply_pool_replaces_an_empty_list() {
    let mut state = State {
        context: Run {
            spec: resolve(&[], &lex("run t\nfor 1h\n").unwrap(), &[]).unwrap(),
            view: AdmitView {
                allow: Vec::new(),
                require_digest: false,
                pull_ok: true,
                live_names: Vec::new(),
                teams: Vec::new(),
                caller: Some("ada".to_string()),
                max_wall_secs: 3600,
            },
            nodes: Vec::new(),
            quota: 8,
            fs: FsView {
                root: "/scratch".to_string(),
                existing: vec!["/scratch".to_string()],
                links: Vec::new(),
            },
        },
        runs: Vec::new(),
        cluster: kluge_controller::MemoryCluster::default(),
        apiserver: None,
    };
    let nodes = r#"{
      "items": [{
        "metadata": {"name": "n1", "labels": {"kluge.dev/gpu": "h100"}},
        "spec": {"taints": [{"key": "kluge.dev/pool", "value": "reserved", "effect": "NoSchedule"}]},
        "status": {"allocatable": {"nvidia.com/gpu": "8"}}
      }]
    }"#;
    apply_pool(&mut state, nodes, r#"{"items":[]}"#).unwrap();
    assert_eq!(state.context.nodes.len(), 1);
    assert!(state.context.nodes[0].healthy && state.context.nodes[0].free);
}
