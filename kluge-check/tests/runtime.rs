use kluge_check::{check_local_sbatch, lex, read_sbatch, render_workload, resolve, Code};

fn rendered(text: &str) -> String {
    let spec = resolve(&[], &lex(text).unwrap(), &[]).unwrap();
    render_workload(&spec)
}

#[test]
fn shm_is_half_the_memory_limit() {
    let mut spec = resolve(&[], &lex("run train.py\non 1 h100\nfor 1h\n").unwrap(), &[]).unwrap();
    spec.memory = Some("16Gi".to_string());
    let yaml = render_workload(&spec);
    assert!(yaml.contains("mountPath: /dev/shm"), "{yaml}");
    assert!(yaml.contains("medium: Memory"), "{yaml}");
    assert!(yaml.contains("sizeLimit: 8Gi"), "{yaml}");
}

#[test]
fn shm_default_when_memory_is_unset() {
    let yaml = rendered("run train.py\non 1 h100\nfor 1h\n");
    assert!(yaml.contains("memory: 16Gi"), "{yaml}");
    assert!(yaml.contains("sizeLimit: 8Gi"), "{yaml}");
}

#[test]
fn limits_match_requests() {
    let yaml = rendered("run train.py\non 1 h100\nfor 1h\n");
    let requests = yaml.split("limits:").next().unwrap_or("");
    let limits = yaml.split("limits:").nth(1).unwrap_or("");
    assert!(requests.contains("memory: 16Gi"), "{yaml}");
    assert!(limits.contains("memory: 16Gi"), "{yaml}");
}

#[test]
fn output_patterns_are_expanded() {
    let spec = read_sbatch(
        "#SBATCH --job-name=ablation\n#SBATCH --output=results/%x-%j-%n.out\n#SBATCH --error=results/err-%n.txt\n#SBATCH --time=01:00:00\necho hi\n",
    )
    .unwrap();
    assert_eq!(spec.into, None);
    let yaml = render_workload(&spec);
    assert!(yaml.contains("results/ablation-ablation-0.out"), "{yaml}");
    assert!(yaml.contains("results/err-0.txt"), "{yaml}");
}

#[test]
fn allocation_env_is_present() {
    let yaml = rendered("run train.py\non 8 h100\nacross 4\nfor 1h\nname train\ndir /scratch/team\n");
    for key in [
        "SLURM_JOB_ID",
        "SLURM_JOB_NAME",
        "SLURM_NNODES",
        "SLURM_NODEID",
        "SLURM_PROCID",
        "SLURM_NTASKS",
        "SLURM_CPUS_PER_TASK",
        "SLURM_GPUS_ON_NODE",
        "SLURM_SUBMIT_DIR",
        "RANK",
        "WORLD_SIZE",
        "LOCAL_RANK",
        "MASTER_ADDR",
        "MASTER_PORT",
    ] {
        assert!(
            yaml.contains(&format!("name: {key}")),
            "{key} missing from {yaml}"
        );
    }
}

#[test]
fn script_and_argv_commands_differ() {
    let argv = rendered("run train.py --epochs 10\nfor 1h\n");
    assert!(argv.contains("- train.py\n"), "{argv}");
    assert!(!argv.contains("/kluge/script"), "{argv}");
    let script = resolve(
        &[],
        &lex("run \"\"\"\necho hi\n\"\"\"\nfor 1h\n").unwrap(),
        &[],
    )
    .unwrap();
    let yaml = render_workload(&script);
    assert!(yaml.contains("- bash\n"), "{yaml}");
    assert!(yaml.contains("- --noprofile\n"), "{yaml}");
    assert!(yaml.contains("- --norc\n"), "{yaml}");
    assert!(yaml.contains("- /kluge/script\n"), "{yaml}");
}

#[test]
fn chdir_maps_to_dir() {
    let spec = read_sbatch("#SBATCH --chdir=/scratch/work\n#SBATCH --time=00:01:00\necho hi\n").unwrap();
    assert_eq!(spec.dir.as_deref(), Some("/scratch/work"));
}

#[test]
fn export_all_is_refused() {
    let diagnostics = check_local_sbatch("#SBATCH --export=ALL\necho hi\n");
    assert!(
        diagnostics.iter().any(|item| {
            item.code == Code::ParseSbatch && item.message.to_lowercase().contains("clean")
        }),
        "{diagnostics:?}"
    );
}
