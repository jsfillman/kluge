# Kluge specs

These documents are the current design. The early files at the root of the repository are not.

| Doc | Contents |
| --- | --- |
| [decisions.md](decisions.md) | The session's decisions, including the ones that were reversed |
| [language.md](language.md) | Command sentence, batch file, `Klugefile`, abbreviations, completion |
| [checks.md](checks.md) | Parser and server checks, the diagnostic format, `--fix`, security |
| [cluster.md](cluster.md) | What gets installed, reserved nodes, Slinky beside Kluge |
| [roadmap.md](roadmap.md) | Build order |

Execution order is [the ticket board](../tickets/README.md). [AGENTS.md](../../AGENTS.md) is the development process.

If a spec and a sketch in `cli/`, `k8s/`, or `gui/` disagree, the spec wins.
