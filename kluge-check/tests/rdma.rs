use kluge_check::{lex, render_workload, request_rdma, resolve};

fn spec(text: &str) -> kluge_check::Spec {
    resolve(&[], &lex(text).unwrap(), &[]).unwrap()
}

#[test]
fn requests_one_device_when_the_job_spans_nodes() {
    let mut spec = spec("run train.py\non 8 h100\nacross 4\nfor 1h\nname train\n");
    request_rdma(&mut spec, Some("rdma/hca"));
    assert_eq!(spec.rdma.as_deref(), Some("rdma/hca"));
    let yaml = render_workload(&spec);
    let requests = yaml.split("limits:").next().unwrap_or("");
    let limits = yaml.split("limits:").nth(1).unwrap_or("");
    assert!(requests.contains("rdma/hca: \"1\""), "{yaml}");
    assert!(limits.contains("rdma/hca: \"1\""), "{yaml}");
}

#[test]
fn one_pod_does_not_request_it() {
    let mut spec = spec("run train.py\non 1 h100\nfor 1h\nname train\n");
    request_rdma(&mut spec, Some("rdma/hca"));
    assert_eq!(spec.rdma, None);
    assert!(!render_workload(&spec).contains("rdma/hca"));
}

#[test]
fn a_missing_device_does_not_request_it() {
    let mut spec = spec("run train.py\non 8 h100\nacross 4\nfor 1h\nname train\n");
    request_rdma(&mut spec, None);
    assert_eq!(spec.rdma, None);
    assert!(!render_workload(&spec).contains("rdma/hca"));
}
