import yaml
import sys
import subprocess
from jinja2 import Template

# Read batch file and extract job details
def parse_batch_file(batch_file):
    job_data = {
        "env": {},
        "command": [],
    }

    with open(batch_file, "r") as f:
        lines = f.readlines()

    in_env = False
    in_run = False
    run_commands = []

    for line in lines:
        line = line.strip()
        if not line or line.startswith("#"):  # Skip comments and empty lines
            continue

        if line.startswith("ENV {"):
            in_env = True
            continue
        elif in_env and line == "}":
            in_env = False
            continue
        elif in_env:
            key, value = (line.split("=", 1) if "=" in line else line.split(" ", 1))
            job_data["env"][key.strip()] = value.strip()
            continue

        if line.startswith("RUN {"):
            in_run = True
            continue
        elif in_run and line == "}":
            in_run = False
            job_data["command"] = run_commands
            continue
        elif in_run:
            run_commands.append(line.strip())
            continue

        if line.startswith("RUN "):
            job_data["command"] = line.replace("RUN ", "").strip().split()
        elif line.startswith("ENV "):
            key, value = (line.replace("ENV ", "").split("=", 1) if "=" in line else line.replace("ENV ", "").split(" ", 1))
            job_data["env"][key.strip()] = value.strip()
        else:
            key, value = line.split(" ", 1)
            job_data[key.lower()] = value.strip()
    
    # 🔍 Debugging: Print parsed job data before validation
    print("\n🔍 Parsed Job Data:")
    for key, value in job_data.items():
        print(f"{key}: {value}")

    # 🔥 Ensure NAME is properly set
    if "name" not in job_data or not isinstance(job_data["name"], str) or not job_data["name"].strip():
        raise ValueError("❌ Error: `NAME` is missing or empty in the batch file.")
    return job_data

# Read and render the YAML template
def render_template(job_data):
    with open("job-template.yaml", "r") as f:
        template = Template(f.read())
    return template.render(job_data)



# Write the YAML file and apply it
def apply_yaml(yaml_content, job_name):
    yaml_file = f"{job_name}.yaml"
    with open(yaml_file, "w") as f:
        f.write(yaml_content)

    print(f"Generated {yaml_file}, now applying...")
    subprocess.run(["kubectl", "apply", "-f", yaml_file], check=True)

# Main function
if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python generate_volcano_job.py job.batch")
        sys.exit(1)

    batch_file = sys.argv[1]
    job_data = parse_batch_file(batch_file)
    job_yaml = render_template(job_data)
    apply_yaml(job_yaml, job_data["name"])
