# Language

One grammar. A command, a `*.kluge` file, and a `Klugefile` are the same phrases. The long form reads as a sentence. The short form is a unique prefix.

```bash
kluge run train.py on 8 h100 across 4 for 6 hours into results
kluge r train.py o 8 h100 a 4 f 6h i results
```

Both submit the same run. Kluge prints the resolved sentence in full words before it submits, so a file of single letters is never the thing a person has to read later.

## Verbs

Each verb owns its first letter. A new verb has to take a free letter or it does not ship. That is what keeps one-letter forms stable.

| Letters | Verb | Meaning |
| --- | --- | --- |
| `r` | `run` | Submit a run |
| `s` | `status` | List runs, or show one |
| `l` | `logs` | Show or follow output |
| `c` | `cancel` | End a run |
| `w` | `watch` | Attach the status TUI. `q` closes the view and leaves the jobs running |
| `e` | `explain` | Why a run is waiting, or why it stopped |
| `d` | `debug` | Resubmit on one GPU, one machine, and say what was rewritten |
| `a` | `attach` | Shell in the run's pod. The shell dies with the pod |

`ssh` is a whole-word synonym of `attach`. `stop` is a whole-word synonym of `cancel`. Synonyms are not in the prefix tree. `s` and `st` are `status`. `sto` matches nothing, and the error offers `status` and `cancel`.

Free first letters for later verbs: `b f g h i j k m n o p q t u v x y z`.

## Phrases

Phrases have their own prefix tree. `a` is `attach` after `kluge`, and `across` after `run`.

| Letters | Phrase | Sets |
| --- | --- | --- |
| `r` | `run` | The command and its arguments |
| `o` | `on` | GPUs on one machine: a count and a type. `on h100` means one GPU |
| `a` | `across` | How many machines |
| `f` | `for` | Wall time. `6h`, `90m`, and `6 hours` are the same value |
| `u` | `use` | Container image |
| `n` | `name` | Run name |
| `i` | `into` | Output directory |
| `w` | `with` | One `KEY=VAL`. Repeatable |
| `t` | `team` | The account that is charged. Slurm's account maps here |
| `d` | `dir` | Working directory on the shared filesystem |

`from` is a whole-word synonym of `use`. `as` is a whole-word synonym of `name`. `f` stays `for`.

There is no second phrase for "GPUs per node." `on 8 h100` is one machine. `across 4` is four of those machines. A request for `on 32 h100` on machines that have eight GPUs is a check failure, not a silent reinterpretation. See [checks](checks.md).

Leading dashes are ignored on a phrase name, so `--on`, `-on`, `on`, and `o` match the same way. There is no separate short-flag alphabet.

Free first letters for later phrases: `b e g h j k l m p q v x y z`.

## Where a value comes from

Later layers win.

1. `Klugefile` at the repo root, the project defaults.
2. A `*.kluge` file named on the command.
3. Phrases typed after `kluge run`.

When the command overrides the file, the submit output says which phrase moved. A phrase set twice in the same layer is an error that quotes both places.

A `Klugefile`:

```text
use registry.example.com/train:latest
on h100
for 6 hours
team kernel-search
dir /scratch/kernel-search
```

A run file, `ablation.kluge`:

```text
name ablation-17
on 8 h100
across 4
for 6 hours
into results
with NCCL_DEBUG=INFO
run torchrun train.py --epochs 10
```

`kluge run ablation.kluge` reads it. `kluge run train.py on 8 h100` is a command whose `run` phrase is `train.py`. Extra phrases on that command line add to it.

`#` starts a comment. Blank lines are ignored. The `run` phrase is the argv of the container, not a shell string. Kluge does not wrap it in `sh -c`.

## `#SBATCH`

`kluge run job.sbatch` is a reader. It understands:

- job name
- nodes
- GPUs, whether written as `--gpus`, `--gpus-per-node`, or `--gres=gpu:`
- CPUs
- memory
- time
- account, mapped to `team`
- output, mapped to `into`

The script body becomes `run`. Any other `#SBATCH` directive refuses the submit, quotes the line, and prints the sentence it had understood so far. Arrays and dependencies are in that refused set until a spec adds them.

## Abbreviations beyond the grammar

GPU types and run names use the same prefix rule.

- `kluge logs ab` selects the single run whose name starts with `ab`. Two matches is an error that lists them.
- `on 8 h` selects `h100` when that is the only type on the reserved pool. Two types that share the prefix is an error that lists both.

## Completion

`kluge completion install` registers bash, zsh, and fish. The completer is the Kluge binary, so the grammar and the cluster cannot drift apart from a hand-written script.

- After `kluge`, the verbs.
- After `run`, local scripts, `*.kluge`, `*.sbatch`, and `Klugefile` phrases.
- After `on`, counts, then GPU types reported by the cluster.
- After `across`, machine counts that fit the reserved pool.
- After `for`, `30m`, `6h`, `6 hours`.
- After `name`, `logs`, `cancel`, `attach`, and `explain`, the caller's run names.
- Inside a `*.kluge` file or a `Klugefile`, the same phrase names and values.

## Output and the TUI

`kluge run` submits, prints the run name and the resolved sentence, and streams logs. Ctrl-C stops the stream. The job keeps running.

`kluge status` with no name prints a table. `kluge status NAME` prints one run. `kluge watch` is the TUI over the same information: name, state, GPUs, age, and one line of why a run is waiting. `q` detaches the view.

`kluge debug train.py` is `run` rewritten to `on 1 <type> across 1`, keeping the image and the command. It prints the rewrite.

`--dry-run` resolves and checks, then does not submit. `--json` prints the spec and any diagnostics as data. Human output stays the sentence.

## Matching rules

1. A token is compared to canonical names in that position.
2. One name with that prefix wins. `c`, `ca`, and `cancel` are `cancel`.
3. Two names with that prefix is an error that lists the names. The command does not run.
4. A whole-word synonym matches only when the token is complete.
5. Edit distance is used only in the error text ("did you mean"), never to pick a meaning and continue.
