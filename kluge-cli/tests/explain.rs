use kluge_check::{Code, Diagnostic, Severity, Span};
use kluge_cli::{execute, CheckClient};

struct Api {
    names: Vec<String>,
    note: bool,
}

impl CheckClient for Api {
    fn check(&self, _: &kluge_check::Spec) -> Result<Vec<Diagnostic>, String> {
        Ok(Vec::new())
    }
    fn list(&self) -> Result<Vec<String>, String> {
        Ok(self.names.clone())
    }
    fn explain(&self, name: &str) -> Result<Diagnostic, String> {
        if !self.names.iter().any(|item| item == name) {
            return Err(self.names.join(" "));
        }
        Ok(Diagnostic {
            code: Code::PoolWaiting,
            severity: if self.note { Severity::Note } else { Severity::Error },
            message: "2 of 4 machines are free".to_string(),
            span: Span {
                file: String::new(),
                line: 1,
                column: 1,
                text: name.to_string(),
                label: None,
            },
            notes: vec!["waiting".to_string()],
            suggestions: Vec::new(),
        })
    }
}

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|p| (*p).to_string()).collect()
}

#[test]
fn waiting_note_is_success() {
    let api = Api {
        names: vec!["ablation".to_string()],
        note: true,
    };
    let (code, out) = execute(&args(&["kluge", "explain", "ablation"]), &api);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("pool.waiting") || out.contains("waiting"), "{out}");
}

#[test]
fn unknown_name_lists_runs() {
    let api = Api {
        names: vec!["ablation".to_string(), "other".to_string()],
        note: false,
    };
    let (code, out) = execute(&args(&["kluge", "explain", "missing"]), &api);
    assert_ne!(code, 0);
    assert!(out.contains("ablation") && out.contains("other"), "{out}");
}
