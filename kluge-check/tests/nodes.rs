use kluge_check::{check_pool, lex, nodes_from, resolve, Code, NodeFact};

fn fact(kind: &str, gpus: u32, healthy: bool, free: bool) -> NodeFact {
    NodeFact {
        kind: kind.to_string(),
        gpus,
        healthy,
        free,
    }
}

const ONE: &str = r#"{
  "items": [{
    "metadata": {"name": "n1", "labels": {"kluge.dev/gpu": "h100"}},
    "spec": {"taints": [{"key": "kluge.dev/pool", "value": "reserved", "effect": "NoSchedule"}]},
    "status": {"allocatable": {"nvidia.com/gpu": "8"}}
  }]
}"#;

#[test]
fn reserved_gpu_node_is_a_free_fact() {
    let nodes = nodes_from(ONE, r#"{"items":[]}"#).unwrap();
    assert_eq!(nodes, vec![fact("h100", 8, true, true)]);
}

#[test]
fn a_node_without_the_kluge_taint_is_ignored() {
    let list = r#"{
      "items": [{
        "metadata": {"name": "other", "labels": {"kluge.dev/gpu": "h100"}},
        "spec": {"taints": [{"key": "slinky", "value": "yes", "effect": "NoSchedule"}]},
        "status": {"allocatable": {"nvidia.com/gpu": "8"}}
      }]
    }"#;
    let nodes = nodes_from(list, r#"{"items":[]}"#).unwrap();
    assert!(nodes.is_empty(), "{nodes:?}");
}

#[test]
fn cordoned_or_extra_taint_is_unhealthy() {
    let list = r#"{
      "items": [
        {
          "metadata": {"name": "cordoned", "labels": {"kluge.dev/gpu": "h100"}},
          "spec": {
            "unschedulable": true,
            "taints": [{"key": "kluge.dev/pool", "value": "reserved", "effect": "NoSchedule"}]
          },
          "status": {"allocatable": {"nvidia.com/gpu": "8"}}
        },
        {
          "metadata": {"name": "extra", "labels": {"kluge.dev/gpu": "h100"}},
          "spec": {"taints": [
            {"key": "kluge.dev/pool", "value": "reserved", "effect": "NoSchedule"},
            {"key": "maintenance", "value": "yes", "effect": "NoSchedule"}
          ]},
          "status": {"allocatable": {"nvidia.com/gpu": "8"}}
        }
      ]
    }"#;
    let nodes = nodes_from(list, r#"{"items":[]}"#).unwrap();
    assert_eq!(nodes.len(), 2, "{nodes:?}");
    assert!(nodes.iter().all(|node| !node.healthy && !node.free), "{nodes:?}");
}

#[test]
fn a_running_gpu_pod_makes_the_node_busy() {
    let pods = r#"{
      "items": [
        {
          "spec": {"nodeName": "n1", "containers": [{"resources": {"requests": {"nvidia.com/gpu": "1"}}}]},
          "status": {"phase": "Running"}
        },
        {
          "spec": {"nodeName": "n1", "containers": [{"resources": {"requests": {"nvidia.com/gpu": "1"}}}]},
          "status": {"phase": "Succeeded"}
        }
      ]
    }"#;
    let busy = nodes_from(ONE, pods).unwrap();
    assert_eq!(busy, vec![fact("h100", 8, true, false)]);
    let pending = r#"{
      "items": [{
        "spec": {"nodeName": "n1", "containers": [{"resources": {"requests": {"nvidia.com/gpu": "8"}}}]},
        "status": {"phase": "Pending"}
      }]
    }"#;
    assert!(!nodes_from(ONE, pending).unwrap()[0].free);
}

#[test]
fn an_empty_pool_refuses_and_the_loaded_list_accepts() {
    let spec = resolve(
        &[],
        &lex("run train.py\non 8 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname ok\ndir /scratch/team\n").unwrap(),
        &[],
    )
    .unwrap();
    let empty = check_pool(&spec, &[], 64);
    assert_eq!(empty[0].code, Code::PoolType);
    let loaded = nodes_from(ONE, r#"{"items":[]}"#).unwrap();
    assert!(check_pool(&spec, &loaded, 64).is_empty());
}
