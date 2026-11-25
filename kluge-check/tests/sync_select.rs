use kluge_check::{select_outputs, Code};

#[test]
fn only_declared_outputs_are_selected() {
    let files = vec![
        "results/a.txt".to_string(),
        "/scratch/team/checkpoint.pt".to_string(),
        "results/../secret".to_string(),
    ];
    let (kept, diagnostics) = select_outputs("results", &files);
    assert_eq!(kept, vec!["results/a.txt".to_string()]);
    assert!(diagnostics.iter().any(|item| item.code == Code::PathEscape));
    assert!(!kept.iter().any(|path| path.contains("checkpoint")));
}
