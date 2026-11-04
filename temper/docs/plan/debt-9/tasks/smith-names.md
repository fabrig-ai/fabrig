+++
id = "smith-names"
part = "smith-names"
kind = "feature"
needs = ["smith-block-walks", "smith-backend-effects"]
files = ["smith/crates/smith-domain-run/src/**", "smith/crates/smith-local-service/src/host_io.rs", "smith/crates/smith-agent-service/src/**", "smith/crates/smith-protocol-llm/src/component.rs", "smith/crates/smith-protocol-llm/src/fixture.rs", "smith/crates/smith-protocol-program/src/backend.rs", "smith/crates/smith-protocol-program/src/backend/**", "smith/crates/smith-domain/src/**", "smith/crates/smith-domain-session/src/**", "temper/crates/temper-engine-smith/src/**", "temper/crates/temper-service/src/tests/**", "jig/tests/channel/src/program_runs.rs", "smith/docs/development/debt/D013-names-left.md", "smith/docs/development/debt/README.md"]
effort = "medium"
minutes = 90
+++

# smith: pay D013, the names the review left

## Goal

Pay `smith/docs/development/debt/D013-names-left.md`, three renames in three commits. The run's `Failure` is
`RunFailure`, as the host calls it. `adapter` stops naming the local host's io hub and the LLM component's limits.
"checkout" stays only where git's checkout is meant. No behaviour changes: no rendered text, reported limit name,
wire form or test outcome moves.

Acceptance:

- `rg -n '\benum Failure\b' smith/crates/smith-domain-run/src` finds nothing;
  `rg -n '\b(run|smith_domain_run)::Failure\b' smith jig temper -g '*.rs' -g '!**/docs/plan/**'` finds nothing.
- `rg -nw 'adapter' smith/crates/smith-local-service/src/host_io.rs` finds nothing.
- `rg -n '\.adapter\b|\badapter:' smith jig -g '*.rs'` finds nothing but what **Exempt** lists.
- `rg -n -i 'checkout' smith/crates -g '*.rs' -g '!*test*'` finds only what **Exempt** lists. Your report gives
  the list that's left, with a word on each.
- The item's file and its README row are gone.

## Inputs

- `temper/docs/plan/debt-9/notes/common.md`, `AGENTS.md`, `smith/AGENTS.md`.
- `smith/docs/design/domain/README.md` (the workspace's words), `smith/docs/design/protocol/limits.md`, 9.
- `smith/docs/development/debt/D013-names-left.md`.

## Owns and may change

- Owns, for these renames only: `smith-domain-run/src/**`, `smith-local-service/src/host_io.rs`.
- The public names' break is the item's own, so it is approved, for the names below. Their readers, each edited
  only for the name: `smith-agent-service/src/**`, `smith-protocol-llm/src/{component,fixture}.rs`,
  `smith-protocol-program/src/backend.rs` and `backend/`, `smith-domain/src/**`, `smith-protocol-channel`,
  `temper-engine-smith/src/{finish,tests}.rs`, `temper-service/src/tests/` (those that name the run's `Failure`:
  read the `use`, as `Failure` there often means another enum), `jig/tests/channel/src/program_runs.rs`.
  `heavy cargo check --tests` finds the rest, which are allowed the same one-word edit.
- **Doc comments** in any smith crate may change for "checkout".

## Exempt

- **The other `Failure`s stay:** the session's, the agent service's, the local domain's and the local shell's two.
  The item's **Paid when** asks only for the run's.
- **The rendered prompt stays byte for byte:** `smith-domain-run/src/prompt.rs`'s text (`## Checkout`, "the files in
  the checkout", "There is no checkout.") and its tests, and the private `render_checkout`, which renders that
  section. It is what the LLM reads, so changing it is a change of behaviour. Comments in that file do change.
- **Reported names stay:** `effective/llm.rs`' `"adapter.tool_bytes"` and `"adapter.history_bytes"` are limits' names in
  a report (`smith/docs/design/protocol/limits.md`, 9), so they're output. The private `fn adapter` beside them
  takes the new word; the two strings don't.
- **Another meaning stays:** temper's `Adapter` and `adapter:` fields (its workers' program), "the run's program
  adapter", the shell's `Agent` adapter, the fake checkout (`skein-fake-checkout`), git's own checkout
  (`smith-local-domain/src/boundary.rs`' "since this checkout began", the local host's git operations).
- `smith-agent-service/src/tests/line.rs`' `use smith_protocol_llm as adapter;` names no limit: leave it.

## Steps

1. **`RunFailure`.** `pub enum Failure` in `smith-domain-run/src/boundary.rs` becomes `RunFailure`, with its
   `impl` in `facts.rs`, the export in `lib.rs`, and its readers. Where a file already has another `RunFailure` in
   scope (the host's or the wire's), use the path. Its doc comment keeps its words. Commit.
2. **`adapter`.** In `host_io.rs`' tests the binding and parameter become `host_io`. `ComponentLimits::adapter`
   (`smith-protocol-llm/src/component.rs`, doc "Translation and receiving bounds") and
   `Settings::adapter` (`smith-protocol-program/src/backend.rs`) become `translation`, as `fixture.rs`' parameter
   already reads; `profile.rs`' literal, `service/program.rs`' `let mut adapter`, `projection.rs`, and each
   `limits.llm.adapter` follow. Commit.
3. **"checkout".** In production code, where it means what the design calls the workspace or its directories, write
   "workspace": `charter.rs`' three comments, `boundary.rs`, `prepare.rs`, `run.rs`, `opening.rs`, `agent.rs`,
   `facts.rs`, `limits.rs`, `outcome.rs`, the session's and the domain's. A field or function so named follows only
   if private. Commit.
4. `cargo fmt`; for each tree `old_forms.py`; `heavy cargo clippy -p smith-domain-run -p smith-domain -p smith-local-service
   -p smith-agent-service -p smith-protocol-llm -p smith-protocol-program -p temper-engine-smith --all-targets -- -D
   warnings`; `heavy cargo nextest run` on those and `smith-protocol-channel`; `heavy cargo check --tests` on
   `temper-service` and `jig/tests/channel`'s package. A doc comment that moves past the width of a line takes
   `cargo fmt`'s word.
5. Delete the item and its row; the citations check. `smith/docs/design/**` is read by tests and this task doesn't
   touch it. Commit and push.

## Report

The common note's items. The renames only move, so no break is owed. Give the lists **Acceptance** asks for.
