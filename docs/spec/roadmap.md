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

## Not the core

Declared output sync and the menu-bar app are specified in [language](language.md). They do not make Slurm look obsolete. The menu layout and the macOS shell can be built without a display. E10-T3 stays blocked until a screenshot of that shell is taken on a Mac. The core is the sentence in [language](language.md) and the refusal in [checks](checks.md), typed at a real `kluge` binary.

## Tickets for the rest

The board's later epics are the plan for this document. E17 is the rest of the first build. E18 is the second build. E19 is [storage](storage.md). E20 is the list below. E8-T3 and E10-T3 stay blocked until a GPU node and a Mac.

## Later, only with a reason

- Borrowing idle nodes from Slinky above a configured floor, giving them back when Slurm has pending work, never preempting a running Slurm job.
- A run record that can refuse to treat a machine failure as a result. The first build's `Run` is the place that record would attach.
- Arrays and job dependencies in the `#SBATCH` reader.
- An agent-facing API beyond `--json` on the commands, once the commands have stopped changing.
