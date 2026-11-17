use kluge_cli::screen_from;

#[test]
fn builds_from_status_json() {
    let raw = r#"{"rows":["ablation"],"explain":"2 of 4 machines are free","into":"results","files":["results/a.txt","/scratch/checkpoint.pt"]}"#;
    let value: serde_json::Value = serde_json::from_str(raw).unwrap();
    let rows = value["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_str().unwrap().to_string())
        .collect();
    let files = value["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item.as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    let screen = screen_from(
        rows,
        value["explain"].as_str().unwrap(),
        value["into"].as_str().unwrap(),
        &files,
    );
    assert_eq!(screen.rows, vec!["ablation".to_string()]);
    assert_eq!(screen.explain, "2 of 4 machines are free");
    assert_eq!(screen.outputs, vec!["results/a.txt".to_string()]);
}
