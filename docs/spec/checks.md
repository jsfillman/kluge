# Checks

A bad run is refused before it is submitted. The message names the phrase, says what is legal, and ends with a command or a diff that would make it legal. The CLI and the controller return the same diagnostic. The CLI is the only program that edits the researcher's files.

This is also the security boundary. The checks are worthless if a researcher can `kubectl apply` a privileged Pod in the same namespace. Only the controller's service account creates Pods. Everyone else submits a `Run`.

## When a check runs

1. The CLI parses the sentence. Syntax, unknown phrases, ambiguous prefixes, duplicate phrases, a missing `run`, a missing local script, and a duration it cannot read are local errors. The cluster is not contacted.
2. If those pass, the CLI sends the normalized spec to the controller's check endpoint, including on `--dry-run`.
3. The controller repeats the invariants and adds the checks that need cluster state.
4. Any error blocks submit. A warning is printed and the submit continues. `--strict` turns warnings into errors.
5. A queue wait is a note, not an error. "4 of 8 machines are free" still submits. "32 GPUs were requested and the pool has 16" does not. A busy cluster is not a defect. An impossible shape is.

The validating webhook on `Run` calls the same function as the check endpoint. A hand-written resource cannot skip the controller. The webhook's status line carries the error code and the one-line fix. The full rendering is what the CLI prints.

## What an error looks like

```text
error[pool.shape]: each reserved node has 8 h100
  --> ablation.kluge:2
   |
 2 | on 32 h100
   |    ^^^^^^^ no machine has this shape
   |
   = note: 32 GPUs is 4 machines of 8
   |
 2 - on 32 h100
 2 + on 8 h100
 3 + across 4
   |
  kluge run --fix ablation.kluge
```

The same object is the JSON form:

```json
{
  "code": "pool.shape",
  "severity": "error",
  "message": "each reserved node has 8 h100",
  "span": {"file": "ablation.kluge", "line": 2, "column": 4, "text": "on 32 h100"},
  "notes": ["32 GPUs is 4 machines of 8"],
  "suggestions": [{
    "auto": true,
    "replacement": "on 8 h100\nacross 4",
    "command": "kluge run train.py on 8 h100 across 4"
  }]
}
```

`code` is stable. Renderers and `--fix` key off it. Wording can change.

## Fixes

`--fix` applies every suggestion marked `auto` and then runs the checks again. It does not submit. A TTY, after a single unambiguous fix, may ask `Apply and submit? [y/N]`. The default is no. A non-interactive run never prompts. `--yes` with `--fix` applies and submits only when the checks then pass.

`auto` is true only when one edit preserves the requested GPU count, or when the phrase is meaningless on this cluster and dropping it does not change the machines. Everything else is a suggestion the user has to choose.

| Situation | Result |
| --- | --- |
| `nods 4`, one edit away from `across` | `auto`, rewrite to `across` |
| `on 32 h100` and machines have 8, and 32 is an exact multiple | `auto`, rewrite to `on 8 h100` and `across 4` |
| `on 32 h100` and the pool has 16 GPUs in total | error, no `auto`. Suggest the largest legal shape |
| Not enough free machines, but the shape fits the pool | note, submit, `status` explains the wait |
| `#SBATCH --exclusive` | `auto` drop, with a note that reserved nodes are already exclusive |
| Unknown `#SBATCH` directive | error, quote the line, no `auto` |
| Missing `for`, and the `Klugefile` has one | resolve from the file, no error |
| Missing `for`, and nothing else sets it | error. Do not invent a duration |
| Two GPU types share the typed prefix | error listing both. No `auto` |
| `into ../other` or a `dir` outside the team's root | error. No `auto` |
| Image tag `:latest`, admin has not required digests | warning |
| Image outside the allowlist, or digest required and absent | error. No `auto` |
| `name` already running | error suggesting a new name. No `auto` |

The controller never writes to the researcher's disk. It returns suggestions. The CLI applies a suggestion only when the span points at a local file or at the current command line.

## Local checks

- The phrase is a known name, a unique prefix, or a whole-word synonym.
- A phrase is set once per layer.
- `run` is non-empty. If it is a local path, the file exists.
- `for` parses as a duration.
- `on` is a count plus a GPU type, or a type alone (count 1).
- `across` is a positive integer. Omitted means 1.
- `with` is `KEY=VAL` with a non-empty key. The value is kept out of logs and out of error text.
- `#SBATCH` translation either produces a spec or a diagnostic per rejected directive.

## Server checks

The controller knows the pool. It repeats anything that must hold even if the client is old or hostile.

**Identity.** The caller is authenticated. `team` is one they are allowed to charge. The `Run` is created as that user.

**Ceilings.** `for` is at most the team's wall-time limit. `on` does not exceed the GPUs on one machine of that type. `across` times that count does not exceed the team's quota or the number of machines of that type in the reserved pool. `name` matches `^[a-z][a-z0-9-]{0,62}$`.

**Machines.** The GPU type exists on the reserved pool. The count matches a real machine. Enough healthy nodes exist for an eventual admit. Health here means the node is untainted except for the Kluge taint, is not cordoned, and reports the GPUs as allocatable. A shortfall against free nodes is a note. A shortfall against existing nodes is `pool.shape` or `pool.capacity`.

**Files.** `dir` and `into` resolve, after symlinks, to paths inside the team's configured root. The directory exists and the user can write it. `..` does not escape.

**Image.** The reference is in the admin allowlist. The nodes can pull it. A floating tag is a warning unless the admin set `requireDigest`.

**The Pod template is fixed.** The user spec cannot add a volume, a capability, a host namespace, a host port, or a privileged flag. Those fields do not exist on `Run`. The webhook rejects a mutated Pod that is not the controller's.

## Codes

| Code | Meaning |
| --- | --- |
| `parse.unknown` | Phrase or verb is not in the grammar |
| `parse.ambiguous` | Prefix matched more than one name |
| `parse.duplicate` | Phrase set twice in one layer |
| `parse.run` | `run` is missing, or the local script does not exist |
| `parse.duration` | `for` did not parse |
| `parse.sbatch` | A `#SBATCH` directive has no translation |
| `pool.shape` | No machine has this GPU shape |
| `pool.capacity` | Shape is real and larger than the pool or the quota |
| `pool.type` | GPU type is not on the reserved pool |
| `name.taken` | A live run already has this name |
| `name.syntax` | Name does not match the pattern |
| `path.escape` | `dir` or `into` leaves the team root |
| `path.missing` | Resolved directory does not exist |
| `image.denied` | Image is outside the allowlist or unpinned when pins are required |
| `image.pull` | The cluster cannot pull the image |
| `auth.team` | Caller cannot charge this team |
| `auth.identity` | Caller is not authenticated |

Waiting for capacity does not use an error code. It is a note, `pool.waiting`, attached to a successful submit and shown by `status` and `explain`.
