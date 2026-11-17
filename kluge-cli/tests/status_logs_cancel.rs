use kluge_cli::{execute, CheckClient};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Api {
    names: Vec<String>,
    log: String,
    cancels: AtomicUsize,
}

impl CheckClient for Api {
    fn check(&self, _: &kluge_check::Spec) -> Result<Vec<kluge_check::Diagnostic>, String> {
        Ok(Vec::new())
    }
    fn list(&self) -> Result<Vec<String>, String> {
        Ok(self.names.clone())
    }
    fn logs(&self, name: &str) -> Result<String, String> {
        Ok(format!("{name}:{}", self.log))
    }
    fn cancel(&self, _name: &str) -> Result<(), String> {
        self.cancels.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|p| (*p).to_string()).collect()
}

#[test]
fn status_lists_and_selects_a_prefix() {
    let api = Api {
        names: vec!["ablation".to_string(), "other".to_string()],
        log: String::new(),
        cancels: AtomicUsize::new(0),
    };
    let (code, out) = execute(&args(&["kluge", "status"]), &api);
    assert_eq!(code, 0);
    assert!(out.contains("ablation") && out.contains("other"), "{out}");
    let (code, out) = execute(&args(&["kluge", "status", "ab"]), &api);
    assert_eq!(code, 0);
    assert_eq!(out, "ablation");
    let api = Api {
        names: vec!["ablation".to_string(), "absolute".to_string()],
        log: String::new(),
        cancels: AtomicUsize::new(0),
    };
    let (code, out) = execute(&args(&["kluge", "s", "ab"]), &api);
    assert_ne!(code, 0);
    assert!(out.contains("parse.ambiguous"), "{out}");
}

#[test]
fn logs_prints_the_body() {
    let api = Api {
        names: vec!["ablation".to_string()],
        log: "hello from the pod".to_string(),
        cancels: AtomicUsize::new(0),
    };
    let (code, out) = execute(&args(&["kluge", "logs", "ablation"]), &api);
    assert_eq!(code, 0);
    assert!(out.contains("hello from the pod"), "{out}");
    drop(out);
    assert_eq!(api.cancels.load(Ordering::SeqCst), 0);
}

#[test]
fn cancel_sends_one_cancel() {
    let api = Api {
        names: vec!["ablation".to_string()],
        log: String::new(),
        cancels: AtomicUsize::new(0),
    };
    let (code, out) = execute(&args(&["kluge", "cancel", "ablation"]), &api);
    assert_eq!(code, 0);
    assert_eq!(out, "ablation");
    assert_eq!(api.cancels.load(Ordering::SeqCst), 1);
}
