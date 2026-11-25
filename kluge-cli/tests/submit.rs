use kluge_check::{Code, Diagnostic, Severity, Span};
use kluge_cli::{execute, CheckClient};
use std::sync::atomic::{AtomicUsize, Ordering};

struct CountClient {
    checks: AtomicUsize,
    posts: AtomicUsize,
    diagnostics: Vec<Diagnostic>,
}

impl CheckClient for CountClient {
    fn check(&self, _: &kluge_check::Spec) -> Result<Vec<Diagnostic>, String> {
        self.checks.fetch_add(1, Ordering::SeqCst);
        Ok(self.diagnostics.clone())
    }
    fn submit(&self, spec: &kluge_check::Spec) -> Result<String, String> {
        self.posts.fetch_add(1, Ordering::SeqCst);
        Ok(spec.name.clone().unwrap_or_else(|| "run".to_string()))
    }
}

fn note() -> Diagnostic {
    Diagnostic {
        code: Code::PoolWaiting,
        severity: Severity::Note,
        message: "2 of 4 machines are free".to_string(),
        span: Span {
            file: String::new(),
            line: 1,
            column: 1,
            text: "across 4".to_string(),
            label: None,
        },
        notes: Vec::new(),
        suggestions: Vec::new(),
    }
}

fn shape() -> Diagnostic {
    Diagnostic {
        code: Code::PoolShape,
        severity: Severity::Error,
        message: "no machine has this shape".to_string(),
        span: Span {
            file: String::new(),
            line: 1,
            column: 1,
            text: "on 32 h100".to_string(),
            label: None,
        },
        notes: Vec::new(),
        suggestions: Vec::new(),
    }
}

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|p| (*p).to_string()).collect()
}

#[test]
fn local_error_skips_endpoint() {
    let client = CountClient {
        checks: AtomicUsize::new(0),
        posts: AtomicUsize::new(0),
        diagnostics: Vec::new(),
    };
    let (code, _) = execute(&args(&["kluge", "run", "train.py", "for", "soon"]), &client);
    assert_ne!(code, 0);
    assert_eq!(client.checks.load(Ordering::SeqCst), 0);
    assert_eq!(client.posts.load(Ordering::SeqCst), 0);
}

#[test]
fn dry_run_checks_and_does_not_post() {
    let client = CountClient {
        checks: AtomicUsize::new(0),
        posts: AtomicUsize::new(0),
        diagnostics: Vec::new(),
    };
    let (code, out) = execute(
        &args(&["kluge", "run", "--dry-run", "train.py", "for", "1h", "name", "ablation"]),
        &client,
    );
    assert_eq!(code, 0, "{out}");
    assert_eq!(client.checks.load(Ordering::SeqCst), 1);
    assert_eq!(client.posts.load(Ordering::SeqCst), 0);
    assert!(!out.contains("ablation\n") || out.contains("name ablation"));
}

#[test]
fn success_posts_one_run() {
    let client = CountClient {
        checks: AtomicUsize::new(0),
        posts: AtomicUsize::new(0),
        diagnostics: Vec::new(),
    };
    let (code, out) = execute(
        &args(&["kluge", "run", "train.py", "for", "1h", "name", "ablation"]),
        &client,
    );
    assert_eq!(code, 0, "{out}");
    assert_eq!(client.posts.load(Ordering::SeqCst), 1);
    assert!(out.contains("ablation"), "{out}");
}

#[test]
fn server_error_does_not_post() {
    let client = CountClient {
        checks: AtomicUsize::new(0),
        posts: AtomicUsize::new(0),
        diagnostics: vec![shape()],
    };
    let (code, _) = execute(&args(&["kluge", "run", "train.py", "for", "1h"]), &client);
    assert_ne!(code, 0);
    assert_eq!(client.posts.load(Ordering::SeqCst), 0);
}

#[test]
fn waiting_note_posts() {
    let client = CountClient {
        checks: AtomicUsize::new(0),
        posts: AtomicUsize::new(0),
        diagnostics: vec![note()],
    };
    let (code, _) = execute(&args(&["kluge", "run", "train.py", "for", "1h"]), &client);
    assert_eq!(code, 0);
    assert_eq!(client.posts.load(Ordering::SeqCst), 1);
}
