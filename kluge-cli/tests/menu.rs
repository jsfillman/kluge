use kluge_cli::{main_with, menu_from_json, Menu, Screen};

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_string()).collect()
}

#[test]
fn empty_menu_is_kluge_zero() {
    let menu = Menu::from_screen(Screen {
        rows: Vec::new(),
        explain: String::new(),
        outputs: Vec::new(),
    });
    assert_eq!(menu.render(), "kluge 0");
}

#[test]
fn one_row_starts_selected() {
    let menu = menu_from_json(
        r#"{"rows":["ablation"],"explain":"2 of 4 machines are free","into":"results","files":["results/a.txt","/scratch/checkpoint.pt"]}"#,
    )
    .unwrap();
    let text = menu.render();
    assert_eq!(
        text,
        "kluge 1\n* ablation\n  2 of 4 machines are free\n  results/a.txt"
    );
    assert!(!text.to_lowercase().contains("cancel"), "{text}");
    assert!(!text.to_lowercase().contains("submit"), "{text}");
    assert!(!text.to_lowercase().contains("sync"), "{text}");
}

#[test]
fn choosing_a_row_selects_it() {
    let mut menu = menu_from_json(
        r#"{"rows":["ablation","train"],"explain":"waiting for a machine","into":"results","files":[]}"#,
    )
    .unwrap();
    assert_eq!(menu.render(), "kluge 2\n  ablation\n  train");
    assert!(menu.choose(1));
    assert_eq!(
        menu.render(),
        "kluge 2\n  ablation\n* train\n  waiting for a machine"
    );
    assert!(!menu.choose(3));
}

#[test]
fn command_prints_a_screen_file() {
    let dir = std::env::temp_dir().join(format!("kluge-menu-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("screen.json");
    std::fs::write(
        &path,
        r#"{"rows":["ablation"],"explain":"running","into":"results","files":["results/a.txt"]}"#,
    )
    .unwrap();
    let (code, text) = main_with(&args(&["kluge", "menu"]), None);
    assert_eq!(code, 0, "{text}");
    assert_eq!(text, "kluge 0");
    let (code, text) = main_with(
        &args(&["kluge", "menu", "--screen", path.to_str().unwrap()]),
        None,
    );
    assert_eq!(code, 0, "{text}");
    assert_eq!(text, "kluge 1\n* ablation\n  running\n  results/a.txt");
    let _ = std::fs::remove_dir_all(dir);
}
