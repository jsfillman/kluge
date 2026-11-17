use kluge_check::{render, sample_pool_shape};

#[test]
fn pool_shape_transcript() {
    let expected = include_str!("fixtures/pool-shape.txt");
    assert_eq!(render(&sample_pool_shape()), expected);
}
