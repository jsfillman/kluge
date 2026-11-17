use kluge_cli::main_args;

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_string()).collect()
}

#[test]
fn sentence_prints_full_words() {
    let (code, out) = main_args(&args(&[
        "kluge", "run", "train.py", "on", "8", "h100", "across", "4", "for", "6h",
    ]));
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("on 8 h100"), "{out}");
    assert!(out.contains("across 4"), "{out}");
    assert!(out.contains("for 6 hours"), "{out}");
    assert!(!out.contains("run id"), "{out}");
}

#[test]
fn bad_duration_is_refused() {
    let (code, out) = main_args(&args(&["kluge", "run", "train.py", "for", "soon"]));
    assert_ne!(code, 0);
    assert!(out.contains("parse.duration"), "{out}");
}

#[test]
fn sto_names_status_and_cancel() {
    let (code, out) = main_args(&args(&["kluge", "sto"]));
    assert_ne!(code, 0);
    assert!(out.contains("status"), "{out}");
    assert!(out.contains("cancel"), "{out}");
}
