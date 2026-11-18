use kluge_check::render_workload;
use kluge_cli::parse_invocation;

fn args(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_string()).collect()
}

fn write_script(body: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let dir = std::env::temp_dir().join(format!(
        "sbatch-dir-{}-{}",
        std::process::id(),
        body.len()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("job.sbatch");
    std::fs::write(&path, body).unwrap();
    (dir, path)
}

#[test]
fn missing_chdir_uses_the_file_directory() {
    let (dir, path) = write_script("#SBATCH --job-name=sample\n#SBATCH --time=00:01:00\necho hi\n");
    let spec = parse_invocation(&args(&["kluge", "run", path.to_str().unwrap()])).unwrap();
    assert_eq!(spec.dir.as_deref(), Some(dir.to_str().unwrap()));
    let yaml = render_workload(&spec);
    assert!(
        yaml.contains(&format!("value: {}", dir.display())),
        "{yaml}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn chdir_wins() {
    let (dir, path) =
        write_script("#SBATCH --chdir=/scratch/work\n#SBATCH --time=00:01:00\necho hi\n");
    let spec = parse_invocation(&args(&["kluge", "run", path.to_str().unwrap()])).unwrap();
    assert_eq!(spec.dir.as_deref(), Some("/scratch/work"));
    let _ = std::fs::remove_dir_all(dir);
}
