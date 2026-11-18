use kluge_cli::{complete, completion_script};

fn s(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|p| (*p).to_string()).collect()
}

#[test]
fn verbs_after_kluge() {
    let got = complete(&s(&["kluge"]), &[], &[], &[]);
    for verb in ["run", "status", "logs", "cancel", "watch", "explain", "debug", "attach"] {
        assert!(got.iter().any(|item| item == verb), "{got:?}");
    }
}

#[test]
fn files_after_run() {
    let files = s(&["train.py", "notes.txt", "job.sbatch", "run.kluge", "go.sh"]);
    let got = complete(&s(&["kluge", "run"]), &files, &[], &[]);
    assert_eq!(
        got,
        s(&["train.py", "job.sbatch", "run.kluge", "go.sh"])
    );
}

#[test]
fn gpus_after_on() {
    let got = complete(&s(&["kluge", "run", "train.py", "on"]), &[], &s(&["h100", "mi300"]), &[]);
    assert_eq!(got, s(&["h100", "mi300"]));
}

#[test]
fn durations_after_for() {
    let got = complete(&s(&["kluge", "run", "train.py", "for"]), &[], &[], &[]);
    assert_eq!(got, s(&["30m", "6h", "6 hours"]));
}

#[test]
fn runs_after_logs_or_cancel() {
    let runs = s(&["ablation", "other"]);
    assert_eq!(complete(&s(&["kluge", "logs"]), &[], &[], &runs), runs);
    assert_eq!(complete(&s(&["kluge", "cancel"]), &[], &[], &runs), runs);
}

#[test]
fn scripts_name_the_shell() {
    for shell in ["bash", "zsh", "fish"] {
        let script = completion_script(shell);
        assert!(script.contains("kluge"), "{script}");
        assert!(script.contains(shell), "{script}");
        assert!(script.contains("complete"), "{script}");
    }
}
