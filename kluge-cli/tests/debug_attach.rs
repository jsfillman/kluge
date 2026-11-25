use kluge_cli::{execute, CheckClient};
use std::sync::Mutex;

struct Submitted {
    across: u32,
    count: u32,
    run: Vec<String>,
}

struct Api {
    submitted: Mutex<Option<Submitted>>,
    target: Mutex<String>,
}

impl CheckClient for Api {
    fn check(&self, _: &kluge_check::Spec) -> Result<Vec<kluge_check::Diagnostic>, String> {
        Ok(Vec::new())
    }
    fn submit(&self, spec: &kluge_check::Spec) -> Result<String, String> {
        *self.submitted.lock().unwrap() = Some(Submitted {
            across: spec.across,
            count: spec.on.as_ref().map(|shape| shape.count).unwrap_or(0),
            run: spec.run.clone(),
        });
        Ok("debug".to_string())
    }
    fn attach(&self, name: &str) -> Result<String, String> {
        let target = format!("pod/{name}");
        *self.target.lock().unwrap() = target.clone();
        Ok(target)
    }
}

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|p| (*p).to_string()).collect()
}

#[test]
fn debug_rewrites_and_submits() {
    let api = Api {
        submitted: Mutex::new(None),
        target: Mutex::new(String::new()),
    };
    let (code, out) = execute(
        &args(&["kluge", "debug", "train.py", "on", "8", "h100", "across", "4", "for", "1h"]),
        &api,
    );
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("on h100") || out.contains("on 1 h100"), "{out}");
    assert!(out.contains("across 1"), "{out}");
    let guard = api.submitted.lock().unwrap();
    let spec = guard.as_ref().unwrap();
    assert_eq!(spec.across, 1);
    assert_eq!(spec.count, 1);
    assert_eq!(spec.run, vec!["train.py".to_string()]);
}

#[test]
fn attach_targets_the_pod() {
    let api = Api {
        submitted: Mutex::new(None),
        target: Mutex::new(String::new()),
    };
    let (code, out) = execute(&args(&["kluge", "ssh", "ablation"]), &api);
    assert_eq!(code, 0, "{out}");
    assert_eq!(out, "pod/ablation");
    let (code, out) = execute(&args(&["kluge", "ss", "ablation"]), &api);
    assert_ne!(code, 0, "{out}");
}
