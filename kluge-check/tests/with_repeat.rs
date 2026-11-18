use kluge_check::{check_local_layers, lex, resolve, Code};

#[test]
fn two_withs_in_one_layer_are_kept() {
    let text = "run train.py\nfor 1h\nwith A=1\nwith B=2\n";
    let spec = resolve(&[], &lex(text).unwrap(), &[]).unwrap();
    assert_eq!(
        spec.with,
        vec![
            ("A".to_string(), "1".to_string()),
            ("B".to_string(), "2".to_string())
        ]
    );
}

#[test]
fn later_layer_replaces_with() {
    let base = lex("run train.py\nfor 1h\nwith A=1\n").unwrap();
    let file = lex("with B=2\nwith C=3\n").unwrap();
    let spec = resolve(&base, &file, &[]).unwrap();
    assert_eq!(
        spec.with,
        vec![
            ("B".to_string(), "2".to_string()),
            ("C".to_string(), "3".to_string())
        ]
    );
}

#[test]
fn second_on_is_still_duplicate() {
    let found = check_local_layers("", "on h100\non 8 h100\nrun train.py\nfor 1h\n", "");
    assert!(found.iter().any(|item| item.code == Code::ParseDuplicate));
}
