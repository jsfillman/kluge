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
