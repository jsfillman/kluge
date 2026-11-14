use kluge_check::{parse_across, parse_for, parse_on, parse_with};
use std::time::Duration;

#[test]
fn six_hours_equals_6h() {
    assert_eq!(
        parse_for("6h").unwrap(),
        parse_for("6 hours").unwrap()
    );
    assert_eq!(parse_for("6h").unwrap(), Duration::from_secs(6 * 60 * 60));
}

#[test]
fn ninety_minutes() {
    assert_eq!(parse_for("90m").unwrap(), Duration::from_secs(90 * 60));
}

#[test]
fn for_rejects_soon() {
    assert!(parse_for("soon").is_err());
}

#[test]
fn on_type_alone_is_one() {
    let shape = parse_on("h100").unwrap();
    assert_eq!(shape.count, 1);
    assert_eq!(shape.kind, "h100");
}

#[test]
fn on_packed_equals_split() {
    assert_eq!(parse_on("8xh100").unwrap(), parse_on("8 h100").unwrap());
}

#[test]
fn with_rejects_bare_key() {
    assert!(parse_with("FOO").is_err());
    let (key, value) = parse_with("FOO=").unwrap();
    assert_eq!(key, "FOO");
    assert_eq!(value, "");
}

#[test]
fn across_rejects_zero() {
    assert!(parse_across("0").is_err());
    assert_eq!(parse_across("4").unwrap(), 4);
}
