use kluge_cli::parse_invocation;

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_string()).collect()
}

#[test]
fn short_sentence_equals_long_sentence() {
    let short = parse_invocation(&args(&[
        "kluge", "r", "train.py", "o", "8", "h100", "a", "4", "f", "6h",
    ]))
    .unwrap();
    let long = parse_invocation(&args(&[
        "kluge", "run", "train.py", "on", "8", "h100", "across", "4", "for", "6", "hours",
    ]))
    .unwrap();
    assert_eq!(short.run, long.run);
    assert_eq!(short.on, long.on);
    assert_eq!(short.across, long.across);
    assert_eq!(short.duration, long.duration);
}

#[test]
fn kluge_file_is_loaded() {
    let path = std::env::temp_dir().join(format!("argv-{}.kluge", std::process::id()));
    std::fs::write(&path, "run train.py\non 1 h100\nfor 1h\n").unwrap();
    let spec = parse_invocation(&args(&["kluge", "run", path.to_str().unwrap()])).unwrap();
    assert_eq!(spec.run, vec!["train.py".to_string()]);
    assert_eq!(spec.on.unwrap().count, 1);
    let _ = std::fs::remove_file(path);
}

#[test]
fn sbatch_file_is_loaded() {
    let path = std::env::temp_dir().join(format!("argv-{}.sbatch", std::process::id()));
    std::fs::write(&path, "#SBATCH --job-name=sample\necho hi\n").unwrap();
    let spec = parse_invocation(&args(&["kluge", "run", path.to_str().unwrap()])).unwrap();
    assert_eq!(spec.name.as_deref(), Some("sample"));
    assert_eq!(spec.run, vec!["echo".to_string(), "hi".to_string()]);
    let _ = std::fs::remove_file(path);
}

#[test]
fn sto_offers_status_and_cancel() {
    let err = parse_invocation(&args(&["kluge", "sto"])).unwrap_err();
    assert!(err.contains("status"), "{err}");
    assert!(err.contains("cancel"), "{err}");
}
