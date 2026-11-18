use kluge_check::{read_sbatch, render_workload};

fn env_after(yaml: &str, key: &str) -> String {
    let marker = format!("- name: {key}\n");
    let start = yaml.find(&marker).unwrap_or(yaml.len());
    yaml[start..].lines().take(5).collect::<Vec<_>>().join("\n")
}

#[test]
fn one_pod_index_is_zero() {
    let spec = read_sbatch(
        "#SBATCH --job-name=sample\n#SBATCH --output=out/%x-%j-%n.txt\n#SBATCH --time=01:00:00\necho hi\n",
    )
    .unwrap();
    let yaml = render_workload(&spec);
    let rank = env_after(&yaml, "RANK");
    assert!(rank.contains("value: 0"), "{rank}");
    assert!(!rank.contains("valueFrom"), "{rank}");
    assert!(yaml.contains("out/sample-sample-0.txt"), "{yaml}");
    assert!(!yaml.contains("job-index"), "{yaml}");
}

#[test]
fn many_pods_read_the_jobset_index() {
    let spec = read_sbatch(
        "#SBATCH --job-name=sample\n#SBATCH --nodes=4\n#SBATCH --gpus-per-node=h100:8\n#SBATCH --output=out/%n.txt\n#SBATCH --time=01:00:00\necho hi\n",
    )
    .unwrap();
    let yaml = render_workload(&spec);
    for key in ["RANK", "LOCAL_RANK", "SLURM_NODEID", "SLURM_PROCID"] {
        let block = env_after(&yaml, key);
        assert!(
            block.contains("fieldPath: metadata.labels['jobset.sigs.k8s.io/job-index']"),
            "{key}: {block}"
        );
    }
    assert!(yaml.contains("out/$(RANK).txt"), "{yaml}");
    assert!(env_after(&yaml, "MASTER_ADDR").contains("value: sample-0"), "{yaml}");
    assert!(env_after(&yaml, "MASTER_PORT").contains("value: 29500"), "{yaml}");
    assert!(env_after(&yaml, "WORLD_SIZE").contains("value: 4"), "{yaml}");
    assert!(env_after(&yaml, "SLURM_NNODES").contains("value: 4"), "{yaml}");
}
