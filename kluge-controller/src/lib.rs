use kluge_check::{AdmitView, Diagnostic, FsView, NodeFact, Spec};

pub struct Run {
    pub spec: Spec,
    pub view: AdmitView,
    pub nodes: Vec<NodeFact>,
    pub quota: u32,
    pub fs: FsView,
}

pub struct Admission {
    pub accepted: bool,
    pub diagnostics: Vec<Diagnostic>,
    pub summary: String,
}

pub fn handle_check(method: &str, body: &str, run: &Run) -> (u16, String) {
    if method != "POST" {
        return (400, "not a spec".to_string());
    }
    let value: serde_json::Value = match serde_json::from_str(body) {
        Ok(value) => value,
        Err(_) => return (400, "not a spec".to_string()),
    };
    let Some(text) = value.get("text").and_then(|item| item.as_str()) else {
        return (400, "not a spec".to_string());
    };
    let phrases = match kluge_check::lex(text) {
        Ok(phrases) => phrases,
        Err(_) => return (400, "not a spec".to_string()),
    };
    let spec = match kluge_check::resolve(&[], &phrases, &[]) {
        Ok(spec) => spec,
        Err(_) => return (400, "not a spec".to_string()),
    };
    let mut checked = Run {
        spec,
        view: run.view.clone(),
        nodes: run.nodes.clone(),
        quota: run.quota,
        fs: run.fs.clone(),
    };
    let _ = &mut checked;
    let decision = admit(&checked);
    (
        200,
        serde_json::to_string(&decision.diagnostics).unwrap_or_else(|_| "[]".to_string()),
    )
}

pub struct Store {
    pub created: Vec<String>,
}

pub fn reconcile(run: &Run, store: &mut Store) {
    if !admit(run).accepted {
        return;
    }
    let name = run.spec.name.clone().unwrap_or_else(|| "kluge".to_string());
    if store
        .created
        .iter()
        .any(|yaml| yaml.contains(&format!("name: {name}")))
    {
        return;
    }
    store.created.push(kluge_check::render_workload(&run.spec));
}

pub fn admit(run: &Run) -> Admission {
    let mut diagnostics = Vec::new();
    diagnostics.extend(kluge_check::check_pool(&run.spec, &run.nodes, run.quota));
    diagnostics.extend(kluge_check::check_paths(&run.spec, &run.fs));
    diagnostics.extend(kluge_check::check_admit(&run.spec, &run.view));
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|item| item.severity == kluge_check::Severity::Error)
        .collect();
    let summary = errors
        .first()
        .map(|item| {
            let code = serde_json::to_value(item.code)
                .ok()
                .and_then(|value| value.as_str().map(str::to_string))
                .unwrap_or_default();
            let command = item
                .suggestions
                .first()
                .map(|suggestion| suggestion.command.clone())
                .unwrap_or_default();
            format!("{code} {command}").trim().to_string()
        })
        .unwrap_or_default();
    Admission {
        accepted: errors.is_empty(),
        diagnostics,
        summary,
    }
}
