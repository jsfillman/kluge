use kluge_check::{lex, render_workload, resolve};

fn yaml(text: &str) -> String {
    render_workload(&resolve(&[], &lex(text).unwrap(), &[]).unwrap())
}

#[test]
fn multi_node_script_uses_launch() {
    let yaml = yaml("name train\nacross 4\nfor 1h\nrun \"\"\"\necho hi\n\"\"\"\n");
    assert!(yaml.contains("- /kluge/launch\n"), "{yaml}");
    assert!(yaml.contains("mountPath: /kluge/launch"), "{yaml}");
    assert!(yaml.contains("mountPath: /kluge/wait"), "{yaml}");
    assert!(yaml.contains("mountPath: /kluge/srun"), "{yaml}");
    assert!(yaml.contains("export PATH=\"/kluge:${PATH}\""), "{yaml}");
    assert!(yaml.contains("exec /kluge/wait"), "{yaml}");
    assert!(yaml.contains("printf '%s\\n' \"$*\" > \"${dir}/srun\""), "{yaml}");
    assert!(yaml.contains("exec ${line}"), "{yaml}");
}

#[test]
fn single_script_stays_on_the_script() {
    let yaml = yaml("name train\nfor 1h\nrun \"\"\"\necho hi\n\"\"\"\n");
    assert!(yaml.contains("- /kluge/script\n"), "{yaml}");
    assert!(!yaml.contains("- /kluge/launch\n"), "{yaml}");
    assert!(!yaml.contains("mountPath: /kluge/launch"), "{yaml}");
}

#[test]
fn argv_across_machines_stays_argv() {
    let yaml = yaml("run train.py --epochs 10\non 8 h100\nacross 4\nfor 1h\nname train\n");
    assert!(yaml.contains("- train.py\n"), "{yaml}");
    assert!(!yaml.contains("/kluge/launch"), "{yaml}");
    assert!(!yaml.contains("/kluge/srun"), "{yaml}");
}
