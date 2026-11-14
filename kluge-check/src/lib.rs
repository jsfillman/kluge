pub fn crate_name() -> &'static str {
    "kluge-check"
}

#[derive(Debug, PartialEq, Eq)]
pub struct Phrase {
    pub line: usize,
    pub name: &'static str,
    pub value: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LexError {
    Unknown { line: usize, token: String },
    Ambiguous { line: usize, names: Vec<&'static str> },
    Duplicate {
        phrase: &'static str,
        first_line: usize,
        second_line: usize,
    },
}

pub fn lex(text: &str) -> Result<Vec<Phrase>, LexError> {
    let mut phrases = Vec::new();
    let mut seen: Vec<(&'static str, usize)> = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let code = raw.split('#').next().unwrap_or("").trim();
        if code.is_empty() {
            continue;
        }
        let (token, value) = code.split_once(char::is_whitespace).unwrap_or((code, ""));
        let name = match match_phrase(token) {
            Match::One(name) => name,
            Match::None => {
                return Err(LexError::Unknown {
                    line,
                    token: token.to_string(),
                })
            }
            Match::Ambiguous(names) => return Err(LexError::Ambiguous { line, names }),
        };
        if let Some((_, first_line)) = seen.iter().find(|(seen_name, _)| *seen_name == name) {
            return Err(LexError::Duplicate {
                phrase: name,
                first_line: *first_line,
                second_line: line,
            });
        }
        seen.push((name, line));
        phrases.push(Phrase {
            line,
            name,
            value: value.trim().to_string(),
        });
    }
    Ok(phrases)
}

#[derive(Debug, PartialEq, Eq)]
pub struct GpuShape {
    pub count: u32,
    pub kind: String,
}

pub fn parse_for(input: &str) -> Result<std::time::Duration, ()> {
    let input = input.trim();
    let (number, unit) = input
        .split_once(' ')
        .map(|(n, u)| (n, u))
        .or_else(|| {
            let split = input.find(|c: char| c.is_ascii_alphabetic())?;
            Some((&input[..split], &input[split..]))
        })
        .ok_or(())?;
    let count: u64 = number.parse().map_err(|_| ())?;
    let seconds = match unit {
        "h" | "hour" | "hours" => count.checked_mul(60 * 60).ok_or(())?,
        "m" | "minute" | "minutes" => count.checked_mul(60).ok_or(())?,
        _ => return Err(()),
    };
    Ok(std::time::Duration::from_secs(seconds))
}

pub fn parse_on(input: &str) -> Result<GpuShape, ()> {
    let input = input.trim();
    if let Some((count, kind)) = input.split_once(|c: char| c == 'x' || c == 'X') {
        if !kind.is_empty() && !kind.contains('x') && !kind.contains('X') {
            let count: u32 = count.parse().map_err(|_| ())?;
            if count > 0 {
                return Ok(GpuShape {
                    count,
                    kind: kind.to_string(),
                });
            }
        }
    }
    let mut parts = input.split_whitespace();
    let first = parts.next().ok_or(())?;
    match parts.next() {
        None => Ok(GpuShape {
            count: 1,
            kind: first.to_string(),
        }),
        Some(kind) if parts.next().is_none() => {
            let count: u32 = first.parse().map_err(|_| ())?;
            if count == 0 || kind.is_empty() {
                return Err(());
            }
            Ok(GpuShape {
                count,
                kind: kind.to_string(),
            })
        }
        _ => Err(()),
    }
}

pub fn parse_with(input: &str) -> Result<(String, String), ()> {
    let (key, value) = input.split_once('=').ok_or(())?;
    if key.is_empty() {
        return Err(());
    }
    Ok((key.to_string(), value.to_string()))
}

pub fn parse_across(input: &str) -> Result<u32, ()> {
    let count: u32 = input.trim().parse().map_err(|_| ())?;
    if count == 0 {
        Err(())
    } else {
        Ok(count)
    }
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
