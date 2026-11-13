# Decisions

A record of what this design is, and of the earlier shapes that were dropped. Dates below are the conversation, not release dates.

## What Kluge is

Kluge is the front door to GPU work on a Kubernetes cluster the systems team already runs. A researcher types a sentence. A controller turns the sentence into an ordinary Job and admits it through a queue. The reserved GPU nodes are tainted so only that controller's service account can schedule there.

Slinky, or any other Slurm-on-Kubernetes install, can keep the rest of the cluster. Kluge does not replace `slurmctld`, and it does not borrow Slurm's nodes in the first build. A lab that wants to leave Slurm later shrinks the Slinky pool and grows the Kluge pool.

## What we kept

- The name Kluge. It is a joke in the spirit of `git`. The expansion "Kubernetes-Linux Unified Grid Engine" is retired. It described the first sketch and it is no longer true of the architecture.
- Researchers do not have to learn Pods, manifests, or `kubectl`.
- `#SBATCH` is accepted as an input format so existing scripts move over. It is parsed into the Kluge sentence. It is not a second language we extend.
- Jobs run detached. Ctrl-C stops log streaming. `kluge cancel` is what ends a job.
- The backend stays small: namespace, quota, Kueue for gang admission, standard Jobs, the cluster's existing shared filesystem for code. No custom scheduler.

## What we set aside

- **Ephemeral Slurm per user.** The original README's mechanism. It gives the systems team a second control plane. Slinky is that product and it is already open source.
- **Scale-from-zero as the reason to exist.** ParallelCluster has done this for years. Slinky NodeSets scale to zero with KEDA, and Karpenter removes the instances. Rebuilding it is not a product.
- **A measurement archive and a study policy as the first build.** Separating a bad GPU from a worse result matters for an automated research loop. It is not the thing a researcher feels on day one. The first build is the front door. The archive can sit on the run record later.
- **Putting Slurm in the path of someone who does not want Slurm.** Running Slinky underneath Kluge would mean operating Slurm so the interface can hide it. On a cluster that already has Slinky, Kluge is a second pool, not a client of `slurmrestd`.
- **Borrowing idle GPUs from Slinky.** Right, and easy to get wrong. A later controller can drain idle nodes above a floor and hand them back when Slurm has pending work. Kluge never preempts a running Slurm job. Not in the first build.
- **A new batch grammar that is not the command.** The prototype's `NAME` / `ENV {}` / `RUN` dialect, and `# kluge:` comments inside Python, both split the language from the CLI. The file is the sentence written one phrase per line.
- **A desktop UI before the CLI.** The PyQt window in `gui/` is a mock with hardcoded numbers. A menu-bar app comes after submit, logs, and declared output sync work in the CLI.
- **Invisible sync of the whole filesystem.** Later, and only for paths the run declares as outputs. Checkpoints stay on the cluster filesystem.

## Current scope

1. Install onto an existing cluster in a way an infra team will accept: one chart, a tainted pool, Slinky untouched.
2. The sentence, the files, abbreviations, completion, and errors that refuse a bad submit and can apply an unambiguous fix.
3. After that, a detachable status TUI, `debug`, and `attach`.
4. After that, a desktop app and output sync.

Agents use the same commands and `--json`. There is no separate agent API while the commands are still changing.
