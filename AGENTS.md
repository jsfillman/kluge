# AGENTS.md

This file is the process. The specs in `docs/spec/` are the product. The tickets in `docs/tickets/` are the only work. Follow them strictly. Do not deviate.

## What you may do

Take the single ticket named by `docs/tickets/README.md` as the current ticket. It is the lowest-numbered open ticket whose dependencies are `done` and whose status is not `blocked-on-cluster` or `blocked-on-display`.

Finish that ticket in one change:

1. Quote the spec. The ticket names a section. Read it. The proof's first lines are that citation. If the ticket and the spec disagree, stop. Leave the ticket `open` and write the conflict at the top of `docs/proofs/<id>.md`. Do not write Rust. The spec wins.
2. If the spec is silent on a behavior the ticket needs, stop the same way. Do not invent the behavior in code or in a drive-by spec edit.
3. Red. Add only the tests the ticket names. Run the command the ticket names. The tests must fail for the reason the ticket names. Paste that failure into the proof. A compile error counts as red only for a ticket that says so.
4. Green. Write the minimum code that makes those tests pass. Do not add helpers, flags, crates, or error codes for a later ticket.
5. Re-run the same command. Paste the pass, the exit code, and the artifact the ticket names into `docs/proofs/<id>.md`.
6. Set that ticket's `Status` to `done`. Update the status column on `docs/tickets/README.md`. Do not start the next ticket in the same change.

A human must be able to re-run the proof command and get the same pass. Prose that says the tests passed is not a proof.

## What you may not do

- Do not implement a ticket that is not the current ticket.
- Do not skip the red run. A test written after the code does not count.
- Do not mark a ticket `done` without its proof file in the same change.
- Do not change phrase letters, verb letters, diagnostic codes, the pod template rules, or the roadmap unless the current ticket says to edit the spec first. Edit the spec, then the tests.
- Do not extend the Python under `cli/`. Do not add Go. Do not add a second implementation of a check that already lives in `kluge-check`.
- Do not build the desktop app, output sync, Slinky borrowing, a study policy, or an agent API beyond `--json` while those tickets are unopened.
- Do not "improve" the grammar, the diagnostics, or the cluster layout while passing a ticket.
- Do not weaken a test, delete an assertion, or mark a live-cluster ticket done by mocking the cluster.
- Do not prompt for a product decision the spec already made.

`blocked-on-cluster` and `blocked-on-display` stay blocked until the proof command runs against a real cluster or a real display. A fake then does not close them.

## Workspace

Rust, stable toolchain, one Cargo workspace.

| Crate | Owns |
| --- | --- |
| `kluge-check` | Grammar, resolution, diagnostics, local checks, cluster checks over a fixture, Job rendering |
| `kluge-cli` | Binary, completion, file edits for `--fix`, HTTP client |
| `kluge-controller` | `Run` CRD, webhook, reconciler |

The chart under `deploy/kluge` is YAML. It appears only when its ticket is current.

## Proof file

`docs/proofs/<ticket-id>.md` contains, in order:

1. Spec citation (path and heading).
2. Red command, exit code, and the lines that show the failure.
3. Green command, exit code, and the lines that show the pass.
4. The extra artifact the ticket asks for, if any.

Do not commit a proof for a ticket you did not finish.
