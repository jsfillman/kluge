use kluge_check::{check_pool, lex, resolve, Code, NodeFact, Severity};

fn nodes(free: usize, total: usize) -> Vec<NodeFact> {
    (0..total)
        .map(|index| NodeFact {
            kind: "h100".to_string(),
            gpus: 8,
            healthy: true,
            free: index < free,
        })
        .collect()
}

fn spec(text: &str) -> kluge_check::Spec {
    resolve(&[], &lex(text).unwrap(), &[]).unwrap()
}

#[test]
fn missing_type_is_pool_type() {
    let found = check_pool(&spec("run train.py\non 1 mi300\nfor 1h\n"), &nodes(2, 2), 64);
    assert_eq!(found[0].code, Code::PoolType);
}

#[test]
fn impossible_count_is_pool_shape() {
    let found = check_pool(&spec("run train.py\non 32 h100\nfor 1h\n"), &nodes(8, 8), 64);
    assert_eq!(found[0].code, Code::PoolShape);
    assert!(found[0].suggestions[0].auto);
    assert_eq!(found[0].suggestions[0].replacement, "on 8 h100\nacross 4");
}

#[test]
fn over_pool_is_capacity_and_not_auto() {
    let found = check_pool(&spec("run train.py\non 32 h100\nfor 1h\n"), &nodes(2, 2), 64);
    assert_eq!(found[0].code, Code::PoolCapacity);
    assert!(!found[0].suggestions[0].auto);
}

#[test]
fn waiting_is_a_note() {
    let found = check_pool(&spec("run train.py\non 8 h100\nacross 4\nfor 1h\n"), &nodes(2, 4), 64);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].code, Code::PoolWaiting);
    assert_ne!(found[0].severity, Severity::Error);
}
