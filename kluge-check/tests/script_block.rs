use kluge_check::{check_local_layers, lex, read_sbatch, render_workload, resolve, Code};

#[test]
fn block_is_one_run() {
    let text = "name train\nrun \"\"\"\nmodule load cuda\ntorchrun train.py\n\"\"\"\n";
    let phrases = lex(text).unwrap();
    let runs: Vec<_> = phrases.iter().filter(|phrase| phrase.name == "run").collect();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].value, "module load cuda\ntorchrun train.py");
    let spec = resolve(&[], &phrases, &[]).unwrap();
    let yaml = render_workload(&spec);
    assert!(yaml.contains("- bash\n"), "{yaml}");
    assert!(yaml.contains("- --noprofile\n"), "{yaml}");
    assert!(yaml.contains("- --norc\n"), "{yaml}");
    assert!(yaml.contains("- /kluge/script\n"), "{yaml}");
}

#[test]
fn hash_inside_block_is_kept() {
    let text = "run \"\"\"\necho hello # not a comment\n\"\"\"\n";
    let phrases = lex(text).unwrap();
    assert!(phrases[0].value.contains("# not a comment"), "{}", phrases[0].value);
}

#[test]
fn unclosed_block_is_parse_run() {
    let found = check_local_layers("", "run \"\"\"\necho hi\n", "");
    assert!(found.iter().any(|item| item.code == Code::ParseRun), "{found:?}");
}

#[test]
fn one_line_run_stays_argv() {
    let text = "run train.py --epochs 10\nfor 1h\n";
    let spec = resolve(&[], &lex(text).unwrap(), &[]).unwrap();
    assert!(spec.script.is_none());
    assert_eq!(
        spec.run,
        vec!["train.py".to_string(), "--epochs".to_string(), "10".to_string()]
    );
    let yaml = render_workload(&spec);
    assert!(yaml.contains("- train.py\n"), "{yaml}");
    assert!(!yaml.contains("/kluge/script"), "{yaml}");
}

#[test]
fn sbatch_body_is_a_script() {
    let script = "#SBATCH --job-name=sample\nmodule load cuda\ntorchrun train.py --epochs 10\n";
    let spec = read_sbatch(script).unwrap();
    assert_eq!(
        spec.script.as_deref(),
        Some("module load cuda\ntorchrun train.py --epochs 10")
    );
    assert!(spec.run.is_empty());
    let yaml = render_workload(&spec);
    assert!(yaml.contains("- bash\n"), "{yaml}");
    assert!(yaml.contains("- /kluge/script\n"), "{yaml}");
}
