use kluge_check::{lex, LexError};

fn fixture(name: &str) -> String {
    let path = format!("tests/fixtures/{name}");
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("missing {path}"))
}

#[test]
fn klugefile_sample_parses() {
    let phrases = lex(&fixture("Klugefile")).unwrap();
    let got: Vec<_> = phrases
        .iter()
        .map(|p| (p.name, p.value.as_str()))
        .collect();
    assert_eq!(
        got,
        vec![
            ("use", "registry.example.com/train:latest"),
            ("on", "h100"),
            ("for", "6 hours"),
            ("team", "kernel-search"),
            ("dir", "/scratch/kernel-search"),
        ]
    );
}

#[test]
fn run_file_sample_parses() {
    let phrases = lex(&fixture("ablation.kluge")).unwrap();
    let got: Vec<_> = phrases
        .iter()
        .map(|p| (p.name, p.value.as_str()))
        .collect();
    assert_eq!(
        got,
        vec![
            ("name", "ablation-17"),
            ("on", "8 h100"),
            ("across", "4"),
            ("for", "6 hours"),
            ("into", "results"),
            ("with", "NCCL_DEBUG=INFO"),
            ("run", "torchrun train.py --epochs 10"),
        ]
    );
}

#[test]
fn comment_and_blank_lines_ignored() {
    let phrases = lex(&fixture("comments.kluge")).unwrap();
    assert_eq!(phrases.len(), 1);
    assert_eq!(phrases[0].name, "on");
    assert_eq!(phrases[0].value, "h100");
}

#[test]
fn duplicate_phrase_quotes_both_lines() {
    let err = lex(&fixture("duplicate.kluge")).unwrap_err();
    match err {
        LexError::Duplicate {
            phrase: "on",
            first_line: 1,
            second_line: 2,
        } => {}
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn abbreviation_in_file() {
    let phrases = lex(&fixture("abbrev.kluge")).unwrap();
    assert_eq!(phrases.len(), 1);
    assert_eq!(phrases[0].name, "on");
    assert_eq!(phrases[0].value, "8 h100");
}
