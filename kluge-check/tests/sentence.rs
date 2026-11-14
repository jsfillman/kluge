use kluge_check::{lex, render_sentence, resolve};

#[test]
fn opening_example_prints_full_words() {
    let cli = lex("run train.py\non 8 h100\nacross 4\nfor 6h\ninto results").unwrap();
    let spec = resolve(&[], &[], &cli).unwrap();
    let expected = std::fs::read_to_string("tests/fixtures/opening-example.txt").unwrap();
    assert_eq!(render_sentence(&spec), expected);
}
