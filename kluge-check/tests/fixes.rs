use kluge_check::{apply_auto, suggestions_for, PoolMachine};

fn pool(machines: u32) -> Vec<PoolMachine> {
    vec![PoolMachine {
        kind: "h100".to_string(),
        gpus: 8,
        count: machines,
    }]
}

#[test]
fn nods_rewrites_to_across() {
    let found = suggestions_for("nods 4\nrun train.py\nfor 1h\n", &pool(8));
    let suggestion = found.iter().flat_map(|d| d.suggestions.iter()).next().unwrap();
    assert!(suggestion.auto);
    assert!(suggestion.replacement.contains("across"), "{}", suggestion.replacement);
}

#[test]
fn thirty_two_splits_across_four() {
    let found = suggestions_for("on 32 h100\nrun train.py\nfor 1h\n", &pool(8));
    let suggestion = found.iter().flat_map(|d| d.suggestions.iter()).next().unwrap();
    assert!(suggestion.auto);
    assert_eq!(suggestion.replacement, "on 8 h100\nacross 4");
}

#[test]
fn pool_too_small_is_not_auto() {
    let found = suggestions_for("on 32 h100\nrun train.py\nfor 1h\n", &pool(2));
    let suggestion = found.iter().flat_map(|d| d.suggestions.iter()).next().unwrap();
    assert!(!suggestion.auto);
    assert!(suggestion.replacement.contains("on 8 h100"));
}

#[test]
fn exclusive_line_is_dropped() {
    let found = suggestions_for("#SBATCH --exclusive\necho hi\n", &pool(8));
    let diagnostic = &found[0];
    assert!(diagnostic.notes.iter().any(|n| n.contains("exclusive")));
    let suggestion = &diagnostic.suggestions[0];
    assert!(suggestion.auto);
    assert!(suggestion.replacement.is_empty());
}

#[test]
fn fix_does_not_submit() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("kluge-fix-{}.kluge", std::process::id()));
    std::fs::write(&path, "on 32 h100\nrun train.py\nfor 1h\n").unwrap();
    let before = std::fs::read_to_string(&path).unwrap();
    let outcome = apply_auto(&path, &pool(8));
    let after = std::fs::read_to_string(&path).unwrap();
    assert!(!outcome.submitted);
    assert_ne!(before, after);
    assert!(after.contains("across 4"));
    let _ = std::fs::remove_file(&path);
}
