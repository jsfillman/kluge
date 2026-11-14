use kluge_check::{match_phrase, match_verb, Match, PHRASES, VERBS};

#[test]
fn verb_s_is_status() {
    assert_eq!(match_verb("s"), Match::One("status"));
}

#[test]
fn verb_st_is_status() {
    assert_eq!(match_verb("st"), Match::One("status"));
}

#[test]
fn verb_sto_matches_nothing() {
    assert_eq!(match_verb("sto"), Match::None);
}

#[test]
fn verb_ssh_is_attach_only_when_complete() {
    assert_eq!(match_verb("ssh"), Match::One("attach"));
}

#[test]
fn verb_ss_is_not_ssh() {
    assert_eq!(match_verb("ss"), Match::None);
}

#[test]
fn phrase_f_is_for() {
    assert_eq!(match_phrase("f"), Match::One("for"));
}

#[test]
fn phrase_from_is_use_only_when_complete() {
    assert_eq!(match_phrase("from"), Match::One("use"));
}

#[test]
fn phrase_dashes_ignored() {
    assert_eq!(match_phrase("--on"), Match::One("on"));
    assert_eq!(match_phrase("-on"), Match::One("on"));
    assert_eq!(match_phrase("on"), Match::One("on"));
    assert_eq!(match_phrase("o"), Match::One("on"));
}

#[test]
fn first_letters_are_unique_in_each_table() {
    let verbs = [
        "run", "status", "logs", "cancel", "watch", "explain", "debug", "attach",
    ];
    let phrases = [
        "run", "on", "across", "for", "use", "name", "into", "with", "team", "dir",
    ];
    assert_eq!(VERBS, verbs);
    assert_eq!(PHRASES, phrases);
    assert!(unique_first_letters(&verbs));
    assert!(unique_first_letters(&phrases));
}

fn unique_first_letters(names: &[&str]) -> bool {
    let mut seen = std::collections::BTreeSet::new();
    names.iter().all(|name| seen.insert(name.chars().next().unwrap()))
}
