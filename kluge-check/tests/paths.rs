use kluge_check::{check_paths, lex, resolve, Code, FsView};

fn spec(dir: &str, into: &str) -> kluge_check::Spec {
    let text = format!("run train.py\non 1 h100\nfor 1h\ndir {dir}\ninto {into}\n");
    resolve(&[], &lex(&text).unwrap(), &[]).unwrap()
}

fn fs() -> FsView {
    FsView {
        root: "/scratch/team".to_string(),
        existing: vec!["/scratch/team".to_string(), "/scratch/team/results".to_string()],
        links: vec![("/scratch/team/escape".to_string(), "/tmp/secret".to_string())],
    }
}

#[test]
fn dotdot_escapes() {
    let found = check_paths(&spec("/scratch/team/../other", "/scratch/team/results"), &fs());
    assert!(found.iter().any(|item| item.code == Code::PathEscape));
}

#[test]
fn symlink_escapes() {
    let found = check_paths(&spec("/scratch/team/escape", "/scratch/team/results"), &fs());
    assert!(found.iter().any(|item| item.code == Code::PathEscape));
}

#[test]
fn missing_inside_root() {
    let found = check_paths(&spec("/scratch/team/missing", "/scratch/team/results"), &fs());
    assert!(found.iter().any(|item| item.code == Code::PathMissing));
}

#[test]
fn present_is_ok() {
    let found = check_paths(&spec("/scratch/team", "/scratch/team/results"), &fs());
    assert!(found.is_empty(), "{found:?}");
}
