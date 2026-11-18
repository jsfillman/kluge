use kluge_check::{check_local_layers, check_local_sbatch, render, Code};

fn codes(diagnostics: &[kluge_check::Diagnostic]) -> Vec<Code> {
    diagnostics.iter().map(|d| d.code).collect()
}

#[test]
fn unknown_phrase() {
    let found = codes(&check_local_layers("", "nods 4\nrun train.py\nfor 1h\n", ""));
    assert_eq!(found, vec![Code::ParseUnknown]);
}

#[test]
fn ambiguous_prefix() {
    let found = codes(&check_local_layers("", "---\nrun train.py\nfor 1h\n", ""));
    assert_eq!(found, vec![Code::ParseAmbiguous]);
}

#[test]
fn duplicate_phrase() {
    let found = codes(&check_local_layers("", "on h100\non 8 h100\nrun train.py\nfor 1h\n", ""));
    assert_eq!(found, vec![Code::ParseDuplicate]);
}

#[test]
fn missing_run() {
    let found = codes(&check_local_layers("", "on h100\nfor 1h\n", ""));
    assert_eq!(found, vec![Code::ParseRun]);
    let missing = codes(&check_local_layers("", "run ./does-not-exist.py\nfor 1h\n", ""));
    assert_eq!(missing, vec![Code::ParseRun]);
}

#[test]
fn bad_duration() {
    let found = codes(&check_local_layers("", "run train.py\nfor soon\n", ""));
    assert_eq!(found, vec![Code::ParseDuration]);
}

#[test]
fn refused_sbatch() {
    let found = codes(&check_local_sbatch("#SBATCH --array=0-3\necho hi\n"));
    assert_eq!(found, vec![Code::ParseSbatch]);
}

#[test]
fn with_value_not_in_text() {
    let diagnostics = check_local_layers("", "with SECRET=hunter2\nfor soon\nrun train.py\n", "");
    let rendered = diagnostics.iter().map(render).collect::<String>();
    assert!(!rendered.contains("hunter2"), "{rendered}");
    assert!(codes(&diagnostics).contains(&Code::ParseDuration));
}

#[test]
fn klugefile_for_satisfies_duration() {
    let found = codes(&check_local_layers(
        "for 6 hours\nrun train.py\n",
        "on h100\n",
        "",
    ));
    assert_eq!(found, Vec::<Code>::new());
}
