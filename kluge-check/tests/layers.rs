use kluge_check::{lex, resolve, Layer, ResolveError};
use std::time::Duration;

fn phrases(text: &str) -> Vec<kluge_check::Phrase> {
    lex(text).unwrap()
}

#[test]
fn cli_overrides_file() {
    let file = phrases("on h100\nrun train.py");
    let cli = phrases("on 8 h100");
    let spec = resolve(&[], &file, &cli).unwrap();
    let on = spec.on.unwrap();
    assert_eq!(on.count, 8);
    assert_eq!(on.kind, "h100");
}

#[test]
fn file_overrides_klugefile() {
    let base = phrases("for 6 hours\nrun train.py");
    let file = phrases("for 2 hours");
    let spec = resolve(&base, &file, &[]).unwrap();
    assert_eq!(spec.duration, Some(Duration::from_secs(2 * 60 * 60)));
}

#[test]
fn override_records_the_phrase() {
    let file = phrases("on h100\nrun train.py");
    let cli = phrases("on 8 h100");
    let spec = resolve(&[], &file, &cli).unwrap();
    assert!(spec.overrides.iter().any(|note| {
        note.phrase == "on" && note.from == Layer::File && note.to == Layer::Cli
    }));
}

#[test]
fn missing_run_errors() {
    let file = phrases("on h100");
    let err = resolve(&[], &file, &[]).unwrap_err();
    assert_eq!(err, ResolveError::MissingRun);
}

#[test]
fn omitted_across_is_one() {
    let file = phrases("run train.py");
    let spec = resolve(&[], &file, &[]).unwrap();
    assert_eq!(spec.across, 1);
    assert_eq!(spec.run, vec!["train.py".to_string()]);
}
