use kluge_check::{render_workload, resolve, lex};

fn spec(extra: &str) -> kluge_check::Spec {
    let text = format!(
        "run train.py\non 8 h100\nfor 1h\nname ablation\nuse registry.example.com/train:latest\ndir /scratch/kernel-search\nwith NCCL_DEBUG=INFO\n{extra}"
    );
    let mut spec = resolve(&[], &lex(&text).unwrap(), &[]).unwrap();
    spec.cpus = Some("4".to_string());
    spec.memory = Some("16G".to_string());
    spec
}

#[test]
fn job_one_matches_fixture() {
    let expected = include_str!("fixtures/job-one.yaml");
    assert_eq!(render_workload(&spec("")), expected);
}

#[test]
fn jobset_four_matches_fixture() {
    let expected = include_str!("fixtures/jobset-four.yaml");
    assert_eq!(render_workload(&spec("across 4\n")), expected);
}
