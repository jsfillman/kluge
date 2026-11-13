# Kluge

<p align="center">
  <img src="./assets/kluge-logo-8122426.png" alt="Kluge" width="240">
</p>

A researcher-facing way to run GPU jobs on a Kubernetes cluster that already exists.

```bash
kluge run train.py on 8 h100 across 4 for 6 hours into results
```

The systems team installs one chart and taints a pool of GPU nodes. Researchers never write a manifest. Slurm and Slinky can keep running on the other nodes. The name is a joke, in the same spirit as `git`.

The design is in [docs/spec](docs/spec/README.md). This repository still contains the early sketches (container builds, static manifests, a prototype submitter, a desktop mock). Those files are not the system described here.

## For researchers

A run is one sentence. The same words are a command, a `*.kluge` file, or the defaults in a `Klugefile`. Each word has a unique first letter, so `kluge r train.py o 8 h100 a 4 f 6h` is the same command. Closing the terminal does not stop the job.

```text
kluge run train.py on 8 h100 across 4 for 6 hours
kluge status
kluge logs ablation
kluge cancel ablation
kluge watch
```

An existing `#SBATCH` script is a reader input: `kluge run job.sbatch` translates the headers it understands and refuses the rest with the line quoted.

## For the cluster

Kluge is a namespace, a queue, and a controller on stock Kubernetes. The reserved nodes carry a taint that only Kluge's service account tolerates. Slinky does not schedule there. Researchers cannot create Pods directly, so the checks cannot be skipped with `kubectl`.

```bash
helm install kluge ./deploy/kluge
```

The chart is part of the design, not of this tree yet.

## Specs

| Doc | What it decides |
| --- | --- |
| [Decisions](docs/spec/decisions.md) | What this session kept, and what it set aside |
| [Language](docs/spec/language.md) | The sentence, the files, abbreviations, completion |
| [Checks](docs/spec/checks.md) | Client and server diagnostics, fixes, security |
| [Cluster](docs/spec/cluster.md) | Install, the reserved pool, how Slinky coexistence works |
| [Roadmap](docs/spec/roadmap.md) | What is in the first build, and what waits |
