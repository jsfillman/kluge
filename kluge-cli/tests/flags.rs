use kluge_check::{Code, Diagnostic, Severity, Span, Suggestion};
use kluge_cli::{execute, CheckClient};
use std::sync::atomic::{AtomicBool, Ordering};

struct PanicClient;
impl CheckClient for PanicClient {
    fn check(&self, _: &kluge_check::Spec) -> Result<Vec<Diagnostic>, String> {
        panic!("server called");
    }
}

struct RecordClient {
    called: AtomicBool,
    diagnostics: Vec<Diagnostic>,
}
impl CheckClient for RecordClient {
    fn check(&self, _: &kluge_check::Spec) -> Result<Vec<Diagnostic>, String> {
        self.called.store(true, Ordering::SeqCst);
        Ok(self.diagnostics.clone())
    }
}

fn warning() -> Diagnostic {
    Diagnostic {
        code: Code::ImageDenied,
        severity: Severity::Warning,
        message: "floating tag".to_string(),
        span: Span {
            file: String::new(),
            line: 1,
            column: 1,
            text: "latest".to_string(),
            label: None,
        },
        notes: Vec::new(),
        suggestions: Vec::<Suggestion>::new(),
    }
}

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|p| (*p).to_string()).collect()
}

#[test]
fn local_error_does_not_call_server() {
    let (code, out) = execute(&args(&["kluge", "run", "train.py", "for", "soon"]), &PanicClient);
    assert_ne!(code, 0);
    assert!(out.contains("parse.duration") || out.contains("for"), "{out}");
}

#[test]
fn dry_run_calls_server_and_prints_sentence() {
    let client = RecordClient {
        called: AtomicBool::new(false),
        diagnostics: Vec::new(),
    };
    let (code, out) = execute(
        &args(&["kluge", "run", "--dry-run", "train.py", "for", "1h"]),
        &client,
    );
    assert_eq!(code, 0);
    assert!(client.called.load(Ordering::SeqCst));
    assert!(out.contains("run train.py"), "{out}");
    assert!(!out.contains("run id"), "{out}");
}

#[test]
fn json_prints_diagnostics() {
    let (code, out) = execute(
        &args(&["kluge", "--json", "run", "train.py", "for", "soon"]),
        &PanicClient,
    );
    assert_ne!(code, 0);
    let value: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    assert!(value.to_string().contains("parse.duration"));
}

#[test]
fn strict_promotes_warning() {
    let loose = RecordClient {
        called: AtomicBool::new(false),
        diagnostics: vec![warning()],
    };
    let (code, _) = execute(&args(&["kluge", "run", "train.py", "for", "1h"]), &loose);
    assert_eq!(code, 0);
    let strict = RecordClient {
        called: AtomicBool::new(false),
        diagnostics: vec![warning()],
    };
    let (code, _) = execute(
        &args(&["kluge", "--strict", "run", "train.py", "for", "1h"]),
        &strict,
    );
    assert_ne!(code, 0);
}

#[test]
fn fix_edits_and_does_not_submit() {
    let path = std::env::temp_dir().join(format!("fix-{}.kluge", std::process::id()));
    std::fs::write(&path, "nods 4\nrun train.py\nfor 1h\n").unwrap();
    let (code, out) = execute(
        &args(&["kluge", "--fix", "run", path.to_str().unwrap()]),
        &PanicClient,
    );
    let body = std::fs::read_to_string(&path).unwrap();
    assert!(body.contains("across"), "{body}");
    assert_eq!(code, 0, "{out}");
    assert!(!out.to_lowercase().contains("submit"), "{out}");
    let _ = std::fs::remove_file(path);
}

#[test]
fn yes_is_not_a_prompt() {
    let (code, out) = execute(
        &args(&["kluge", "--yes", "run", "train.py", "for", "soon"]),
        &PanicClient,
    );
    assert_ne!(code, 0);
    assert!(!out.contains("Apply"), "{out}");
    assert!(!out.contains("[y/N]"), "{out}");
}
