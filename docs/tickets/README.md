# Tickets

Do the current ticket and stop. [AGENTS.md](../../AGENTS.md) is the process. The specs in `docs/spec/` are the behavior.

The current ticket is the first row that is `open`, whose dependencies are all `done`, and that is not blocked.

| Id | Status | Depends | Title |
| --- | --- | --- | --- |
| [E0-T1](E0-T1.md) | done | — | Cargo workspace |
| [E1-T1](E1-T1.md) | done | E0-T1 | Prefix matching |
| [E1-T2](E1-T2.md) | done | E1-T1 | `on`, `for`, and `with` values |
| [E1-T3](E1-T3.md) | done | E1-T2 | Files and the `Klugefile` |
| [E1-T4](E1-T4.md) | done | E1-T3 | Three layers |
| [E1-T5](E1-T5.md) | done | E1-T4 | Resolved sentence |
| [E1-T6](E1-T6.md) | done | E1-T5 | `#SBATCH` reader |
| [E2-T1](E2-T1.md) | done | E1-T1 | Diagnostic JSON |
| [E2-T2](E2-T2.md) | done | E2-T1 | Text rendering |
| [E2-T3](E2-T3.md) | done | E1-T6, E2-T2 | Local checks |
| [E2-T4](E2-T4.md) | done | E2-T3 | `--fix` |
| [E3-T1](E3-T1.md) | done | E1-T5 | Argv |
| [E3-T2](E3-T2.md) | done | E2-T4, E3-T1 | Flags before any cluster |
| [E3-T3](E3-T3.md) | done | E3-T1 | Completion candidates |
| [E4-T1](E4-T1.md) | done | E1-T4 | Job and JobSet rendering |
| [E4-T2](E4-T2.md) | done | E4-T1 | Fixed pod template |
| [E4-T3](E4-T3.md) | done | E4-T2, E11-T2 | Runtime compatibility |
| [E5-T1](E5-T1.md) | done | E2-T1, E4-T1 | Pool checks |
| [E5-T2](E5-T2.md) | done | E5-T1 | Path checks |
| [E5-T3](E5-T3.md) | done | E5-T1 | Image, name, and auth |
| [E6-T1](E6-T1.md) | done | E5-T3 | Admission uses `kluge-check` |
| [E6-T2](E6-T2.md) | done | E4-T2, E6-T1 | Reconcile a `Run` to a Job |
| [E6-T3](E6-T3.md) | done | E6-T1 | Check endpoint |
| [E7-T1](E7-T1.md) | done | E6-T2 | Helm chart |
| [E8-T1](E8-T1.md) | done | E3-T2, E6-T3 | Submit and dry-run |
| [E8-T2](E8-T2.md) | done | E8-T1 | Status, logs, cancel |
| [E8-T3](E8-T3.md) | blocked-on-cluster | E8-T2, E7-T1 | One real GPU |
| [E9-T1](E9-T1.md) | done | E8-T2 | `watch` model |
| [E9-T2](E9-T2.md) | done | E8-T2, E5-T1 | `explain` |
| [E9-T3](E9-T3.md) | done | E8-T2 | `debug` and `attach` |
| [E10-T1](E10-T1.md) | done | E9-T3 | Output selection |
| [E10-T2](E10-T2.md) | done | E10-T1 | View model, no window |
| [E10-T3](E10-T3.md) | blocked-on-display | E10-T2 | Menu-bar shell |
| [E11-T1](E11-T1.md) | done | E3-T1, E2-T3 | The binary speaks the sentence |
| [E11-T2](E11-T2.md) | done | E1-T6, E11-T1 | Block-quoted scripts |
| [E12-T1](E12-T1.md) | done | E11-T2 | Repeatable `with` |
| [E12-T2](E12-T2.md) | done | E12-T1 | Script and hosts files |
| [E12-T3](E12-T3.md) | done | E12-T2 | Pod index |
| [E12-T4](E12-T4.md) | done | E12-T3 | Launch, wait, and srun |
| [E12-T5](E12-T5.md) | done | E12-T4 | Output files, signal, and export |
| [E12-T6](E12-T6.md) | done | E12-T5 | RDMA request |
| [E12-T7](E12-T7.md) | done | E12-T6 | Directory of the sbatch file |
| [E13-T1](E13-T1.md) | done | E12-T7, E6-T3 | Check and admit over HTTP |
| [E13-T2](E13-T2.md) | done | E13-T1 | Sync |
| [E13-T3](E13-T3.md) | done | E13-T2, E7-T1 | Chart objects |
| [E13-T4](E13-T4.md) | done | E13-T1, E11-T1 | The CLI uses the API |
| [E13-T5](E13-T5.md) | done | E13-T4, E3-T3, E8-T2 | Completion install and Ctrl-C |
| [E14-T1](E14-T1.md) | done | E10-T2 | Menu layout |
| [E14-T2](E14-T2.md) | done | E14-T1 | macOS status item |
| [E15-T1](E15-T1.md) | done | E13-T2, E5-T1 | The pool from node and pod lists |
| [E15-T2](E15-T2.md) | done | E15-T1, E13-T3 | Webhook certificate |
| [E15-T3](E15-T3.md) | open | E15-T1, E13-T2 | Poll nodes and runs |

## Epics

| Epic | Tickets | Starts when |
| --- | --- | --- |
| [E0 Workspace](epics/E0.md) | E0-T1 | Now |
| [E1 Grammar](epics/E1.md) | E1-T1 … E1-T6 | E0 is done |
| [E2 Checks](epics/E2.md) | E2-T1 … E2-T4 | E1 has produced phrases |
| [E3 CLI](epics/E3.md) | E3-T1 … E3-T3 | The library can resolve a sentence |
| [E4 Jobs](epics/E4.md) | E4-T1, E4-T2, E4-T3 | A spec exists without a cluster |
| [E5 Cluster checks](epics/E5.md) | E5-T1 … E5-T3 | Diagnostics and the spec exist |
| [E6 Controller](epics/E6.md) | E6-T1 … E6-T3 | Checks are pure functions |
| [E7 Chart](epics/E7.md) | E7-T1 | The controller's objects are known |
| [E8 Wire-up](epics/E8.md) | E8-T1 … E8-T3 | CLI and check endpoint exist |
| [E9 Daily verbs](epics/E9.md) | E9-T1 … E9-T3 | Submit works against a fake API |
| [E10 Desktop](epics/E10.md) | E10-T1 … E10-T3 | The daily verbs exist. Not the core |
| [E11 The sentence](epics/E11.md) | E11-T1, E11-T2 | The binary speaks it. E11-T2 is a script block |
| [E12 The pod](epics/E12.md) | E12-T1 … E12-T7 | The rendered YAML contains the files and the indexes |
| [E13 The API](epics/E13.md) | E13-T1 … E13-T5 | Localhost API. E8-T3 and E10-T3 stay blocked |
| [E14 Menu bar](epics/E14.md) | E14-T1, E14-T2 | Layout and the Mac shell. E10-T3 stays blocked |
| [E15 The controller reads](epics/E15.md) | E15-T1 … E15-T3 | Recorded node lists and a localhost API. E8-T3 stays blocked |
