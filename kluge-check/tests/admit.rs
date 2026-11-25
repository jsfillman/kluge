use kluge_check::{check_admit, lex, resolve, AdmitView, Code, Severity};
use std::time::Duration;

fn view() -> AdmitView {
    AdmitView {
        allow: vec!["registry.example.com/".to_string()],
        require_digest: false,
        pull_ok: true,
        live_names: vec!["taken".to_string()],
        teams: vec!["kernel-search".to_string()],
        caller: Some("ada".to_string()),
        max_wall_secs: 2 * 60 * 60,
    }
}

fn spec(text: &str) -> kluge_check::Spec {
    resolve(&[], &lex(text).unwrap(), &[]).unwrap()
}

#[test]
fn latest_is_a_warning_until_digests_are_required() {
    let mut loose = view();
    let found = check_admit(&spec("run t\non 1 h100\nfor 1h\nuse registry.example.com/train:latest\nteam kernel-search\nname ok\n"), &loose);
    assert!(found.iter().any(|item| item.severity == Severity::Warning));
    loose.require_digest = true;
    let found = check_admit(&spec("run t\non 1 h100\nfor 1h\nuse registry.example.com/train:latest\nteam kernel-search\nname ok\n"), &loose);
    assert!(found.iter().any(|item| item.code == Code::ImageDenied && item.severity == Severity::Error));
}

#[test]
fn outside_allowlist_is_denied() {
    let found = check_admit(&spec("run t\non 1 h100\nfor 1h\nuse evil.example/train:1\nteam kernel-search\nname ok\n"), &view());
    assert!(found.iter().any(|item| item.code == Code::ImageDenied));
}

#[test]
fn pull_failure_is_image_pull() {
    let mut view = view();
    view.pull_ok = false;
    let found = check_admit(&spec("run t\non 1 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname ok\n"), &view);
    assert!(found.iter().any(|item| item.code == Code::ImagePull));
}

#[test]
fn bad_and_taken_names() {
    let syntax = check_admit(&spec("run t\non 1 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname Bad_Name\n"), &view());
    assert!(syntax.iter().any(|item| item.code == Code::NameSyntax));
    let taken = check_admit(&spec("run t\non 1 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname taken\n"), &view());
    let item = taken.iter().find(|item| item.code == Code::NameTaken).unwrap();
    assert!(!item.suggestions[0].auto);
}

#[test]
fn team_and_identity() {
    let mut view = view();
    let denied = check_admit(&spec("run t\non 1 h100\nfor 1h\nuse registry.example.com/train:1\nteam other\nname ok\n"), &view);
    assert!(denied.iter().any(|item| item.code == Code::AuthTeam));
    view.caller = None;
    let found = check_admit(&spec("run t\non 1 h100\nfor 1h\nuse registry.example.com/train:1\nteam kernel-search\nname ok\n"), &view);
    assert!(found.iter().any(|item| item.code == Code::AuthIdentity));
}

#[test]
fn wall_time_ceiling() {
    let found = check_admit(&spec("run t\non 1 h100\nfor 6 hours\nuse registry.example.com/train:1\nteam kernel-search\nname ok\n"), &view());
    assert!(found.iter().any(|item| item.code == Code::TimeCeiling));
    assert!(found[0].message.contains("2") || found.iter().any(|item| item.message.contains("7200") || item.notes.iter().any(|n| n.contains("2"))));
    let _ = Duration::from_secs(1);
}
