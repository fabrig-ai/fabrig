+++
id = "subagents-profile"
part = "subagents"
kind = "feature"
needs = []
files = ["temper/crates/temper-init/template/agent.json", "temper/docs/examples/owned/agent.json", "temper/crates/temper-init/src/tests/**", "temper/docs/design/protocol/setup.status.md"]
effort = "medium"
minutes = 45
+++

# subagents-profile: the owned example's agent runs a run and two sub-agents side by side

## Goal

The agent configuration `temper init` writes, and the owned example, declare smith's
`conversations` 3 rather than 1, so that a run on Claude Code may ask two sub-agents side by
side instead of having every `sub_agent` refused as too many (temper's `protocol/setup.md`,
8.1; smith's `protocol/limits.md`, 2.2; smith's `protocol/programs.md`, 5.10).

Acceptance:

- `temper/crates/temper-init/template/agent.json` and `temper/docs/examples/owned/agent.json`
  give `profile.declared.conversations` 3; nothing else in them changes.
- `temper-init`'s focused tests pass, a test there asserting the template's declared
  `conversations` is 3 (add one to `src/tests/documents.rs` if none asserts the profile).
- The agent configuration still loads: the tests that decode the template or the owned example
  as smith's agent configuration (find them with `rg -l 'agent.json' temper/crates temper/tests`)
  pass unchanged but for a literal 1 they assert, which becomes 3.
- `temper/docs/design/protocol/setup.status.md`: the Not yet entry beginning "8.1: the owned
  example's agent declares `conversations` 1" is removed.

## Inputs

- `temper/docs/design/protocol/setup.md`, 8.1 (the paragraph "Its agent's configuration").
- smith's `smith/docs/design/protocol/limits.md`, 2.2 (`conversations`) and section 3 (what it
  sizes).
- `temper/docs/plan/reference-m3/notes/common.md`; contracts C6.

## Owns and may change

- Owns: the two `agent.json` files above; `temper-init`'s `src/tests/`; the status entry.
- Neighbours, a literal's value only: the test fixtures that copy the template's profile
  (`temper/crates/temper/tests/fixtures/cluster/agent.json`,
  `temper/crates/temper/tests/support/github/agent.json`,
  `temper/crates/temper/tests/support/configuration.rs`) where a test holds them equal to the
  template; otherwise leave them at 1 and say so in the report.
- `temper/tests/real-loop/src/fixtures.rs` is a live tier's: leave it.
- Options:
  - If smith refuses the profile at start because 3 conversations do not fit the declared
    `memory` (2,147,483,648), do not raise `memory`: it is the user's sizing. Stop with a
    `Question:` giving the smallest `memory` that fits and the error.
  - If smith's `standard` profile's `policy` or another declared quantity must change with
    `conversations`, the same: ask, with the figures.

## Steps

1. Change the two files' `conversations` to 3.
2. Find each reader: `rg -n 'conversations' temper/crates/temper-init temper/crates/temper/tests
   temper/docs/examples`; update the literals the options allow.
3. Add or extend the assertion in `temper/crates/temper-init/src/tests/documents.rs`.
4. `heavy cargo nextest run -p temper-init`, then the tests that decode the owned example (the
   `temper` package's test binary that reads `docs/examples/owned`, by `rg`), narrowly by name.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-init
   --all-targets -- -D warnings`; the citations check (common.md). Commit and push.

## Report

As fab's implementer text says; name each test that read the profile and whether it changed.
