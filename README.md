# Kluge

<p align="center">
  <img src="./assets/kluge-logo-8122426.png" alt="Kluge" width="240">
</p>

The batch system for GPU research. Researchers and agents submit attempts. Systems people run an ordinary Kubernetes cluster. Only the systems team has to know that.

The name is a joke, in the same spirit as `git`.

## Three doors, one cluster

**Researchers** submit work and read results in batch language: jobs, GPUs, cost, and why a run stopped. `sbatch` still works. Pods, manifests, and `kubectl` stay behind the door.

**Agents** call a few stable tools and get JSON back. The same actions a person runs. Idempotent submits, structured errors, and a next step they can apply. They never assemble a manifest, and they never log in to a node to scrape `squeue`.

**Systems** run stock Kubernetes. GPU drivers, quotas, node pools, upgrades — the cluster they already know how to operate. Kluge is a controller, a queue, and a catalog installed the way any other controller is installed. There is no Slurm daemon per user and no second control plane to feed.

An existing Slurm or AWS ParallelCluster queue is another place a run can land. Jobs submitted there with raw `sbatch` are adopted into the same catalog. That door is for sites that already run Slurm. The thing everyone shares is the attempt.

## Submit

From the directory you are already working in:

```bash
kluge run train.py --nodes 8
```

```text
kernel-search/ablation-44
14 files changed, uncommitted, bundled as 9f3c1a2
8×H100  rail-aligned  warm pool  starts now
estimate $420  cap $2,000  parent ablation-31 (best, 1.12 step/s)
https://kluge.example/kernel-search/ablation-44
```

Kluge hashes the working tree, including uncommitted edits, and runs those bytes. The image is content-addressed from that bundle plus the project base image. The project supplies the account, the output location, the budget cap, and the GPU. Flags cover the things you actually change:

```bash
kluge run train.py --nodes 8 --gpus 8 --time 6h --gpu h100
kluge run train.py --gpu mi300
kluge run train.py --debug          # one GPU, same bundle, same image, same environment
kluge run --print train.py --nodes 8   # show the job, the launch environment, and the digest
```

`#SBATCH` headers are accepted too. Kluge translates them into the same run. A script handed to `sbatch` on a linked Slurm cluster shows up in the catalog on its own.

```bash
kluge run job.sbatch
```

## The attempt

A run records the code that actually executed, the parent attempt, the GPUs it landed on, what it cost, and why it stopped. The URL holds the spec, the rendered job, the bundle hash, the metrics, and the reason. `kluge replay <url>` runs those bytes again, or tells you the digest is gone.

When it stops, the next command is already decided:

```text
ablation-44  stopped  xid 79  rank 37  node p5-014
node drained
11 gpu-hours  $210
retry ready: same bundle, that node excluded
```

```bash
kluge explain 44
kluge retry
kluge logs 44              # the anomaly first: the warning, the slow rank, then rank 0
kluge logs 44 --rank 37
kluge diff 44              # code bundle and metrics against the parent
kluge diff 44 31           # against the best attempt in the study
```

Infrastructure outcomes and scientific outcomes stay distinct. An Xid, an out-of-memory kill, an NCCL or RCCL timeout, a hang, a preemption, a drained node, or a hit budget is an infrastructure result: the bytes did not get a fair measurement. A completed run with a worse metric is a scientific result. The archive stores the difference, so a bad GPU is never written down as a worse kernel.

`kluge doctor` probes each GPU pool and stores a launch profile: CPU affinity, fabric devices, NCCL or RCCL settings, and the bus-bandwidth baseline for that shape. Runs inherit the profile. `explain` prints the bandwidth the job achieved next to that baseline. Overrides are allowed and recorded.

Before a job is admitted, preflight checks the quota, the image digest, the output location, memory headroom, and whether the requested nodes fit one rail-aligned placement. A run that would exhaust GPU memory, or land across two fabrics, comes back in seconds with a shape that will start.

Pending tells you the lever. How many GPUs are free, what is ahead of you, when a warm node can take a smaller shape, and what waiting costs.

## The study

The unit of work is the study: a tree of attempts with a budget and a policy.

```bash
kluge status
```

```text
kernel-search   best ablation-31  +6.2% tokens/s   $18.4k of $25k
  running   45, 46
  held      47  tile size 128→256, waiting on policy
  today     xid 79 drained p5-014
```

```bash
kluge best
kluge frontier          # non-dominated set across the metrics the training process wrote
kluge top               # live spend against the cluster’s budget
```

People set the policy. Agents work inside it.

```yaml
concurrent: 4
budget: 25000
retry: infra
stop: metric.tokens_per_s < 0.9 * best after 20m
```

Infrastructure failures retry once, on the same bytes, with the bad node excluded. A regression against the best attempt stops, and the checkpoint is kept. The agent cannot raise the cap, read another study, or drain a node.

Training code writes `metrics.json` to the output prefix. `diff`, `best`, and `frontier` read that file. The search algorithm stays yours. The archive is Kluge’s.

## Agents

The agent surface is the same set of actions, as tools:

| Tool | What it returns |
| --- | --- |
| `submit` | A run id, or the existing run when the spec hash matches |
| `status` | Phase, cost, placement, parent |
| `logs` | The anomaly first, then the rank you asked for |
| `explain` | Reason, node, rank, bandwidth against baseline, and the prepared retry |
| `retry` | A child run with the infrastructure patch applied |
| `diff` | Code-bundle diff and metric diff against a parent or the best |
| `watch` | A stream of sentences: started, stopped, drained, policy hold |

The idempotency key is a hash of the spec and the code bundle. Submitting the same attempt again returns the run that is already there.

Terminal reasons come from a closed list: `completed`, `oom`, `xid`, `nccl_timeout`, `rccl_timeout`, `hang`, `preempted`, `budget`, `node_fail`, `bad_image`, `cancelled`. Each error says whether it is retryable and what the next call is.

A study watch looks like this:

```text
02:14  ablation-44  xid 79 on p5-014  drained  retry queued
02:16  ablation-45  started  warm  8×h100
02:31  ablation-45  tokens/s 0.71 of best  policy stopping
```

## Systems

Kluge adds one controller and one catalog to a standard cluster.

- **Queue.** [Kueue](https://kueue.sigs.k8s.io/) for quota, priority, and gang scheduling.
- **Workloads.** Indexed Jobs and [JobSet](https://jobset.sigs.k8s.io/) for multi-node training and eval arrays.
- **Machines.** Node pools per GPU type, a warm pool for short attempts, and labels for rail-aligned placement. Cold capacity scales through the cluster autoscaler and says so in the run: “cold start, about eight minutes.”
- **Health.** A node agent watches Xid, ECC, fabric errors, disk, and clocks, and removes a bad node from the pool before the next attempt lands on it.
- **Catalog.** Every run, including one adopted from raw `sbatch`, is a row: bundle hash, image digest, parent, nodes, GPU UUIDs, cost, metrics, reason.

`kluge run --print` shows the job the cluster will execute. Every automatic act — injected launch environment, placement, drain, retry, policy stop — is an event on the run. `scontrol` and `kubectl` remain available to the people who operate the cluster. The books match the queue.

Sites with Slurm keep it. Kluge submits into a partition and reads back state, so a ParallelCluster queue and a Kubernetes pool show up as one study. Researchers use the same commands either way.

## Commands

```text
kluge run <script> [--nodes N] [--gpus N] [--time T] [--gpu TYPE] [--debug] [--print]
kluge status [study]
kluge explain <run>
kluge retry <run>
kluge logs <run> [--rank N]
kluge diff <run> [other]
kluge best [study]
kluge frontier [study]
kluge top
kluge doctor
kluge replay <url>
```

## Status of this repository

The interface above is the product. This repository is the early sketch that led there: container builds, static manifests, a prototype submitter, and a desktop mock. Those files are not the system described here. The acronym expansion “Kubernetes-Linux Unified Grid Engine” is retired; it described that sketch, and the name does not need a replacement.
