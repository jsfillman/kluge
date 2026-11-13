# Roadmap

The order is the product order. A later phase does not start by mocking a UI over commands that do not exist.

## First build

A researcher can submit, watch logs, and cancel, on a reserved pool, with checks that refuse a bad sentence.

- Helm chart, namespace, taint documentation, Kueue queue, `Run` CRD, controller, webhook.
- Parser for the sentence, `*.kluge`, `Klugefile`, and the `#SBATCH` subset in [language](language.md).
- Verbs `run`, `status`, `logs`, `cancel`.
- Prefix abbreviations, whole-word synonyms, and `kluge completion install`.
- Local checks, server checks, the diagnostic format, and `--fix` from [checks](checks.md).
- `--dry-run` and `--json`.
- Ctrl-C detaches the log stream and leaves the Job running.
- One shared filesystem path, configured by the admin. The laptop is not the source of the code yet.
- Acceptance: one GPU from a laptop, then eight GPUs on the reserved nodes, including one `#SBATCH` file and one file that `--fix` rewrites.

## Second build

The same API, with the rest of the daily verbs.

- `watch` TUI. `q` detaches.
- `explain` renders the waiting note or the terminal reason in the same diagnostic style.
- `debug` and `attach`.
- Pending text comes from Kueue and the Pod: quota, gang, image pull, or the pool being full.

## Third build

- Declared outputs under `into` sync to the laptop. Checkpoints stay on the shared filesystem. Nothing else is copied.
- A menu-bar app that is a view of `status`, `logs`, and `explain`. It is not a second control path.

## Later, only with a reason

- Borrowing idle nodes from Slinky above a configured floor, giving them back when Slurm has pending work, never preempting a running Slurm job.
- A run record that can refuse to treat a machine failure as a result. The first build's `Run` is the place that record would attach.
- Arrays and job dependencies in the `#SBATCH` reader.
- An agent-facing API beyond `--json` on the commands, once the commands have stopped changing.
