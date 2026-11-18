use kluge_check::{lex, resolve, AdmitView, FsView};
use kluge_controller::{open_on, Run};

fn main() {
    let bind = std::env::var("KLUGE_LISTEN").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let root = std::env::var("KLUGE_TEAM_ROOT").unwrap_or_else(|_| "/".to_string());
    let wall = std::env::var("KLUGE_TEAM_MAX_WALL").unwrap_or_else(|_| "8h".to_string());
    let digest = std::env::var("KLUGE_IMAGE_REQUIRE_DIGEST").unwrap_or_default() == "true";
    let allow = std::env::var("KLUGE_IMAGE_ALLOW").unwrap_or_default();
    let allow: Vec<String> = if allow.is_empty() {
        vec![String::new()]
    } else {
        allow.split(',').map(|item| item.to_string()).collect()
    };
    let teams = std::env::var("KLUGE_TEAMS").unwrap_or_default();
    let teams: Vec<String> = teams
        .split(',')
        .filter(|item| !item.is_empty())
        .map(|item| item.to_string())
        .collect();
    let spec = resolve(&[], &lex("run kluge\nfor 1h\n").expect("context"), &[])
        .expect("context");
    let context = Run {
        spec,
        view: AdmitView {
            allow,
            require_digest: digest,
            pull_ok: true,
            live_names: Vec::new(),
            teams,
            caller: Some("kluge".to_string()),
            max_wall_secs: wall_seconds(&wall),
        },
        nodes: Vec::new(),
        quota: u32::MAX,
        fs: FsView {
            root: root.clone(),
            existing: vec![root],
            links: Vec::new(),
        },
    };
    let handle = open_on(&bind, context).expect("listen");
    println!("{}", handle.addr);
    std::thread::park();
}

fn wall_seconds(value: &str) -> u64 {
    value
        .strip_suffix('h')
        .and_then(|hours| hours.parse::<u64>().ok())
        .map(|hours| hours * 3600)
        .unwrap_or(8 * 3600)
}
