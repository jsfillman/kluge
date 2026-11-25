use kluge_check::{lex, render_workload, resolve};

#[test]
fn template_is_fixed() {
    let text = "run train.py\non 8 h100\nfor 1h\nname ablation\nuse registry.example.com/train:latest\ndir /scratch/kernel-search\n";
    let spec = resolve(&[], &lex(text).unwrap(), &[]).unwrap();
    let yaml = render_workload(&spec);
    assert!(yaml.contains("key: kluge.dev/pool"), "{yaml}");
    assert!(yaml.contains("value: reserved"), "{yaml}");
    assert!(yaml.contains("effect: NoSchedule"), "{yaml}");
    assert!(yaml.contains("kluge.dev/gpu: h100"), "{yaml}");
    assert!(!yaml.contains("privileged"));
    assert!(!yaml.contains("hostNetwork"));
    assert!(!yaml.contains("hostPID"));
    assert!(!yaml.contains("hostIPC"));
    assert!(!yaml.contains("sh -c"));
}
