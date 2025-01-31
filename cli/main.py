import yaml
import subprocess
import os
from jinja2 import Template

class KlugeBatch:
    def __init__(self, batch_script: str):
        self.job_name = "kluge-job"
        self.cpus = 1
        self.mem = "1Gi"
        self.gpus = 0
        self.nodes = 1
        self.image = "ubuntu:latest"
        self.workdir = "/workspace"
        self.run_commands = []  # Support for multiple run commands
        self.outdir = "Desktop"  # Default output directory

        self.parse_script(batch_script)
        self.ensure_outdir_exists()  # Ensure output directory exists

    def parse_script(self, script: str):
        """Parses the Kluge Batch DSL and extracts job parameters."""
        for line in script.strip().split("\n"):
            parts = line.split(maxsplit=1)
            if len(parts) < 2:
                continue
            key, value = parts
            if key == "job":
                self.job_name = value
            elif key == "cpus":
                self.cpus = int(value)
            elif key == "mem":
                self.mem = value
            elif key == "gpus":
                self.gpus = int(value)
            elif key == "nodes":
                self.nodes = int(value)
            elif key == "image":
                self.image = value
            elif key == "workdir":
                self.workdir = value  # Set working directory
            elif key == "run":
                self.run_commands.append(value)  # Support multiple run statements
            elif key == "outdir":
                self.outdir = value  # Custom output directory

    def ensure_outdir_exists(self):
        """Ensures the output directory exists on the host."""
        host_path = os.path.expanduser(f"~/ {self.outdir}")  # Expand ~ to user's home
        if not os.path.exists(host_path):
            print(f"📂 Creating output directory: {host_path}")
            os.makedirs(host_path)

    def generate_k8s_yaml(self):
        """Loads a base YAML template and fills in values dynamically."""
        with open("job_template.yaml") as f:
            template = Template(f.read())

        formatted_run_commands = " && ".join(
            [cmd.replace("job_output_", f"/mnt/output/{self.outdir}/job_output_") for cmd in self.run_commands]
        )

        filled_yaml = template.render(
            job_name=self.job_name,
            cpus=self.cpus,
            mem=self.mem,
            nodes=self.nodes,
            image=self.image,
            workdir=self.workdir,
            outdir=self.outdir,
            run_command=formatted_run_commands  # Supports multiple run commands
        )

        return filled_yaml

    def run(self):
        """Creates and runs the Kubernetes Job."""
        k8s_yaml = self.generate_k8s_yaml()
        yaml_filename = f"{self.job_name}.yaml"
        
        with open(yaml_filename, "w") as f:
            f.write(k8s_yaml)
        
        print(f"✅ Generated {yaml_filename}")
        
        # Apply the Kubernetes job
        subprocess.run(["kubectl", "apply", "-f", yaml_filename], check=True)


# Example Usage
if __name__ == "__main__":
    job_definition = """
    job kluge_image_batch
    cpus 2
    mem 1Gi
    gpus 0
    nodes 2
    workdir /workspace/kluge_jobs
    outdir kluge_results

    image python:3.9-slim
    run pip3 install --user pillow opencv-python-headless numpy
    run python3 logo-overlay.py
    """

    job = KlugeBatch(job_definition)
    job.run()
