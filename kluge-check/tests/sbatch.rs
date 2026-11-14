use kluge_check::read_sbatch;

#[test]
fn common_headers_translate() {
    let script = "\
#!/bin/bash
#SBATCH --job-name=ablation-17
#SBATCH --nodes=2
#SBATCH --gpus-per-node=h100:8
#SBATCH --cpus-per-task=4
#SBATCH --mem=16G
#SBATCH --time=01:00:00
#SBATCH --account=kernel-search
#SBATCH --output=results/out.txt

torchrun train.py --epochs 10
";
    let spec = read_sbatch(script).unwrap();
    assert_eq!(spec.name.as_deref(), Some("ablation-17"));
    assert_eq!(spec.across, 2);
    let on = spec.on.unwrap();
    assert_eq!(on.count, 8);
    assert_eq!(on.kind, "h100");
    assert_eq!(spec.cpus.as_deref(), Some("4"));
    assert_eq!(spec.memory.as_deref(), Some("16G"));
    assert_eq!(spec.duration, Some(std::time::Duration::from_secs(3600)));
    assert_eq!(spec.team.as_deref(), Some("kernel-search"));
    assert_eq!(spec.into.as_deref(), Some("results/out.txt"));
}

#[test]
fn gres_translates() {
    let script = "#SBATCH --gres=gpu:h100:8\necho hi\n";
    let spec = read_sbatch(script).unwrap();
    let on = spec.on.unwrap();
    assert_eq!(on.count, 8);
    assert_eq!(on.kind, "h100");
}

#[test]
fn body_is_argv() {
    let script = "#SBATCH --job-name=sample\ntorchrun train.py --epochs 10\n";
    let spec = read_sbatch(script).unwrap();
    assert_eq!(
        spec.run,
        vec![
            "torchrun".to_string(),
            "train.py".to_string(),
            "--epochs".to_string(),
            "10".to_string()
        ]
    );
    assert!(!spec.run.iter().any(|arg| arg == "sh"));
}

#[test]
fn array_is_refused_and_quotes_the_line() {
    let script = "#SBATCH --array=0-3\necho hi\n";
    let err = read_sbatch(script).unwrap_err();
    assert_eq!(err.line, 1);
    assert!(err.text.contains("--array"), "{}", err.text);
}

#[test]
fn unknown_directive_quotes_the_line() {
    let script = "#!/bin/bash\n#SBATCH --exclusive\necho hi\n";
    let err = read_sbatch(script).unwrap_err();
    assert_eq!(err.line, 2);
    assert!(err.text.contains("--exclusive"), "{}", err.text);
}
