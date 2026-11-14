use kluge_check::{sample_pool_shape, Code, Diagnostic};

#[test]
fn json_matches_spec_fixture() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/pool-shape.json")).unwrap();
    let rendered = serde_json::to_value(sample_pool_shape()).unwrap();
    assert_eq!(rendered, fixture);

    let mut with_future = fixture.clone();
    with_future
        .as_object_mut()
        .unwrap()
        .insert("future".to_string(), serde_json::json!(true));
    let parsed: Diagnostic = serde_json::from_value(with_future).unwrap();
    assert_eq!(parsed.code, Code::PoolShape);
}
