use kluge_check::{lex, resolve, AdmitView, FsView};
use kluge_controller::{mint_webhook, open_on, open_tls, Run};

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
    let nodes = match (std::env::var("KLUGE_NODES"), std::env::var("KLUGE_PODS")) {
        (Ok(node_path), Ok(pod_path)) => {
            let node_list = std::fs::read_to_string(node_path).unwrap_or_else(|_| "{\"items\":[]}".into());
            let pod_list = std::fs::read_to_string(pod_path).unwrap_or_else(|_| "{\"items\":[]}".into());
            kluge_check::nodes_from(&node_list, &pod_list).unwrap_or_default()
        }
        _ => Vec::new(),
    };
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
        nodes,
        quota: u32::MAX,
        fs: FsView {
            root: root.clone(),
            existing: vec![root],
            links: Vec::new(),
        },
    };
    let material = mint_webhook();
    println!("{}", material.ca_bundle);
    let handle = open_on(&bind, context).expect("listen");
    let webhook = std::env::var("KLUGE_WEBHOOK_LISTEN").unwrap_or_else(|_| "0.0.0.0:8443".to_string());
    open_tls(&webhook, &material, std::sync::Arc::clone(&handle.state)).expect("webhook");
    if let Ok(base) = std::env::var("KLUGE_APISERVER") {
        handle.state.lock().expect("state").apiserver = Some(base);
        let state = std::sync::Arc::clone(&handle.state);
        std::thread::spawn(move || loop {
            let base = state.lock().expect("state").apiserver.clone();
            if let Some(base) = base {
                let mut guard = state.lock().expect("state");
                let _ = kluge_controller::poll_once(&base, &mut guard);
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
        });
    }
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
