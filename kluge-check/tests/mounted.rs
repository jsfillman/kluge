use kluge_check::{lex, render_workload, resolve};

fn yaml(text: &str) -> String {
    render_workload(&resolve(&[], &lex(text).unwrap(), &[]).unwrap())
}

#[test]
fn hosts_list_every_index() {
    let yaml = yaml("run train.py\non 1 h100\nacross 3\nfor 1h\nname sample\n");
    assert!(yaml.contains("mountPath: /kluge/hosts"), "{yaml}");
    assert!(yaml.contains("sample-0"), "{yaml}");
    assert!(yaml.contains("sample-1"), "{yaml}");
    assert!(yaml.contains("sample-2"), "{yaml}");
    assert!(!yaml.contains("script: |"), "{yaml}");
}

#[test]
fn script_is_mounted_at_kluge_script() {
    let yaml = yaml("name train\nfor 1h\nrun \"\"\"\nmodule load cuda\ntorchrun train.py\n\"\"\"\n");
    assert!(yaml.contains("mountPath: /kluge/script"), "{yaml}");
    assert!(yaml.contains("module load cuda"), "{yaml}");
    assert!(yaml.contains("torchrun train.py"), "{yaml}");
    assert!(yaml.contains("- bash\n"), "{yaml}");
    assert!(yaml.contains("- /kluge/script\n"), "{yaml}");
}

#[test]
fn one_line_run_has_no_script_file() {
    let yaml = yaml("run train.py --epochs 10\nfor 1h\nname sample\n");
    assert!(yaml.contains("- train.py\n"), "{yaml}");
    assert!(!yaml.contains("mountPath: /kluge/script"), "{yaml}");
    assert!(!yaml.contains("script: |"), "{yaml}");
    assert!(yaml.contains("sample-0"), "{yaml}");
}
