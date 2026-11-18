use kluge_cli::WatchModel;

#[test]
fn q_detaches_without_cancel() {
    let mut model = WatchModel::new(vec!["ablation".to_string()]);
    model.press_q();
    assert!(model.detached);
    assert!(!model.cancelled);
}

#[test]
fn rows_render() {
    let model = WatchModel::new(vec!["ablation".to_string(), "other".to_string()]);
    assert_eq!(model.render(), "ablation\nother");
}
