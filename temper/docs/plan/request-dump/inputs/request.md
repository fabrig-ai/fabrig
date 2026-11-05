# request-dump: a worker can dump each model request to a file, for the comparison

A temporary diagnostic, off by default, as cheap as possible. Dogfooding compares temper with another pipeline, and prompts, models and effort must match exactly. Today nobody can read what a run was sent: the rendered prompt isn't kept, and a Claude run's effort can't be read. This goes before the next comparison.

## What

An environment variable on the worker, `TEMPER_DUMP_LLM=DIR`, read once at its start like `TEMPER_WORKSPACE_REUSE` (`temper/crates/temper/src/lib.rs`, `workspace_reuse`). Unset, nothing changes and nothing is paid but one branch.

Set, each model request a run makes writes one file to `DIR`, before it's sent:

- **The body as the provider gets it,** byte for byte: system and developer prompts, messages, tools, model, effort or reasoning, token limits, temperature. Not a re-rendering of our own types: the point is to see what left.
- **Named so it sorts and says whose it is:** task, run and request number, whichever of those the seam knows, then the provider. A sidecar is fine if the name can't hold them.
- **Never headers, tokens or URLs with a credential.** Say in the report how the seam proves it. A test checks that a fake provider's bearer token isn't in the file.
- **Mode 0600 in a directory made 0700,** written whole by rename.
- **Bounded:** stop writing after a total of 4 GiB, with one line in the log saying so; the run goes on. A failed write logs one line and never fails a run.

Every provider path the worker uses must be covered: the API with an OpenAI-style account, and Claude, including a Claude Code subscription, if temper's path for it goes through `temper/crates/temper/src/claude_code.rs` or a child process. If one path can't be seen, because a CLI makes the call and not temper, say which, say what can be read instead (its argv, its config, its stream), dump that, and flag it plainly in the report: a gap here is the user's to know.

Also one file per run at its start, if the seam makes that cheap: what the run was given beyond the request: charter, turn and spend limits, tools granted. Otherwise say where each can be read today.

## Where

Find the seam nearest the wire. The kit's rule holds: a missing generic mechanism goes into skein before its consumer, and smith names neither jig nor temper. If the agent runs in a child or a sandbox, the path must work from there: carry it down as the worker's other settings go, and the sandbox must allow the write. If the smallest correct hack crosses a kit's boundary, keep it behind a narrow hook, not a name; a boundary you can't keep is a `Question:`.

Keep it small and easy to delete: the whole of it should come out as one commit's revert after v0.1.0.

## `dogfood-hacks.md`

Add its entry to `temper/docs/design/draft/dogfood-hacks.md` (what, why, cost and risk, what the other pipeline does, a real design), and its status file, as the document's first lines ask. Its risk: the files hold whatever the run was sent, so repository text and anything an agent read. They stay on the worker's disk. The real design is a question: what a deployment's operators may read of a run's prompts.

## Tests

The narrowest that proves it, in the tier that holds the seam, with a fake provider and no real host: with the variable set, a request's file exists, its bytes equal what the fake received, it has no credential, its mode is 0600; unset, no file; a failing write leaves the run going. Add a test that the 4 GiB bound stops the writing only if it can be done with a small injected bound.

## Size

One or two tasks. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1). fabrig/ui has priority: leave its files alone. No protocol, store or client change. Formats change in place before v0.1.0.

## Walk

`walk = false`. fabrig/qa uses it at the next run, and that's the proof.
