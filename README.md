# Kluge

#### (Kubernetes-Linux Unified Grid Engine)

<p align="center">
  <img src="./assets/kluge-logo-8122426.png" alt="Kluge Logo" width="300">
</p>

---

## 🚀 Overview

**Kluge** is a **modern, ephemeral Slurm-on-Kubernetes** framework designed for **batch job execution** with seamless `sbatch` compatibility. It dynamically provisions isolated Slurm clusters per user, optimizing cloud and bare-metal resources for **HPC workloads**.

## 🎯 Features

✅ **Ephemeral Slurm Clusters** - Each user gets an isolated Slurm instance, spun up on demand.  
✅ **Kubernetes Native** - Deploys via Helm, scales with Kubernetes.  
✅ **Automatic Resource Allocation** - Creates `slurmd` nodes dynamically based on job specs.  
✅ **Cloud & Bare-Metal Ready** - Works across Kubernetes-managed clusters, from cloud to on-prem.  
✅ **Native `sbatch` Support** - Submit jobs without modifying existing scripts.  
✅ **Persistent Storage (Optional)** - Mounts `/home/$USER` for easy result retrieval.  

---

## 📦 Installation

1️⃣ **Install Helm** (if not already installed):  
   ```bash
   curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash
   ```

2️⃣ **Clone the repo & deploy Kluge**:  
   ```bash
   git clone https://github.com/jsfillman/kluge.git
   cd kluge
   helm install kluge ./helm/kluge --namespace $USER
   ```

3️⃣ **Verify deployment**:  
   ```bash
   kubectl get pods -n $USER
   ```

---

## 🛠️ Usage

### 🏗️ Submitting a Batch Job
Submit a job using the `kluge` CLI:

```bash
kluge batch myjob.sbatch
```

Example `myjob.sbatch`:
```bash
#!/bin/bash
#SBATCH --job-name=test_job
#SBATCH --nodes=2
#SBATCH --time=00:10:00
#SBATCH --output=output.log

echo "Running on $(hostname)"
sleep 60
```

### 📜 Monitoring Jobs
View running jobs:

```bash
kluge queue
```

Cancel a job:

```bash
kluge cancel <job_id>
```

---

## 🔧 Development

### 🔹 Deploy Locally
Run in minikube or kind:

```bash
helm install kluge ./helm/kluge --set local=true
```

### 🔹 Debugging
Check logs for the controller:

```bash
kubectl logs -n $USER deployment/kluge-controller
```

---

## 🎯 Roadmap

- ✅ **MVP: Helm-based deployment**  
- 🔜 **Multi-user job scheduling**  
- 🔜 **Web dashboard for job monitoring**  
- 🔜 **Cloud autoscaling**  

---

## 🏆 Contributing

Contributions are welcome! Open an issue or submit a PR to help shape **Kluge**. 🚀