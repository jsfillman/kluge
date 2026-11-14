pub fn crate_name() -> &'static str {
    "kluge-check"
}

#[derive(Debug, PartialEq, Eq)]
pub enum Match {
    One(&'static str),
    None,
    Ambiguous(Vec<&'static str>),
}

pub const VERBS: &[&str] = &[
    "run", "status", "logs", "cancel", "watch", "explain", "debug", "attach",
];

pub const PHRASES: &[&str] = &[
    "run", "on", "across", "for", "use", "name", "into", "with", "team", "dir",
];

const VERB_SYNONYMS: &[(&str, &str)] = &[("ssh", "attach"), ("stop", "cancel")];

const PHRASE_SYNONYMS: &[(&str, &str)] = &[("from", "use"), ("as", "name")];

pub fn match_verb(token: &str) -> Match {
    match_token(token, VERBS, VERB_SYNONYMS)
}

pub fn match_phrase(token: &str) -> Match {
    match_token(token.trim_start_matches('-'), PHRASES, PHRASE_SYNONYMS)
}

fn match_token(token: &str, names: &[&'static str], synonyms: &[(&str, &'static str)]) -> Match {
    let mut hits: Vec<&'static str> = names
        .iter()
        .copied()
        .filter(|name| name.starts_with(token))
        .collect();
    match hits.len() {
        1 => Match::One(hits.remove(0)),
        n if n > 1 => Match::Ambiguous(hits),
        _ => synonyms
            .iter()
            .find(|(word, _)| *word == token)
            .map(|(_, canonical)| Match::One(*canonical))
            .unwrap_or(Match::None),
    }
}
