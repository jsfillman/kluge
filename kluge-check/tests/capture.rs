use kluge_check::{check_local_sbatch, fill_export, lex, read_sbatch, render_workload, resolve, Code};

fn yaml_of(text: &str) -> String {
    render_workload(&resolve(&[], &lex(text).unwrap(), &[]).unwrap())
}

#[test]
fn into_sets_the_output_file() {
    let yaml = yaml_of("run train.py\nfor 1h\nname ablation\ninto results\n");
    assert!(yaml.contains("results/ablation-ablation.out"), "{yaml}");
    assert!(yaml.contains("- train.py\n"), "{yaml}");
    assert!(!yaml.contains("/kluge/launch"), "{yaml}");
}

#[test]
fn script_with_output_uses_launch() {
    let spec = read_sbatch(
        "#SBATCH --job-name=sample\n#SBATCH --output=out/%x.txt\n#SBATCH --time=01:00:00\necho hi\n",
    )
    .unwrap();
    let yaml = render_workload(&spec);
    assert!(yaml.contains("- /kluge/launch\n"), "{yaml}");
    assert!(yaml.contains("out/sample.txt"), "{yaml}");
}

#[test]
fn signal_is_seconds_before_the_deadline() {
    let spec = read_sbatch(
        "#SBATCH --job-name=sample\n#SBATCH --signal=B:TERM@60\n#SBATCH --time=01:00:00\necho hi\n",
    )
    .unwrap();
    let yaml = render_workload(&spec);
    assert!(yaml.contains("value: TERM"), "{yaml}");
    assert!(yaml.contains("value: 3540"), "{yaml}");
    assert!(yaml.contains("- /kluge/launch\n"), "{yaml}");
}

#[test]
fn other_signal_syntax_is_refused() {
    let found = check_local_sbatch("#SBATCH --signal=TERM@60\necho hi\n");
    assert!(found.iter().any(|item| item.code == Code::ParseSbatch), "{found:?}");
}

#[test]
fn export_names_are_copied() {
    let mut spec = read_sbatch("#SBATCH --export=NONE\n#SBATCH --time=00:01:00\necho hi\n").unwrap();
    assert!(spec.export_names.is_empty(), "{:?}", spec.export_names);
    let spec = read_sbatch("#SBATCH --export=FOO,BAR\n#SBATCH --time=00:01:00\necho hi\n").unwrap();
    assert_eq!(spec.export_names, vec!["FOO".to_string(), "BAR".to_string()]);
    let mut spec = spec;
    fill_export(&mut spec, &[("FOO", "1")]);
    assert!(spec.with.iter().any(|(key, value)| key == "FOO" && value == "1"));
    assert!(!spec.with.iter().any(|(key, _)| key == "BAR"));
    let found = check_local_sbatch("#SBATCH --export=ALL\necho hi\n");
    assert!(found.iter().any(|item| {
        item.code == Code::ParseSbatch && item.message.to_lowercase().contains("clean")
    }));
    let _ = &mut spec;
}
