+++
id = "no-workspace"
part = "drafter-run"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-domain/src/boundary.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes.rs",
  "temper/docs/design/domain/agent.status.md",
]
effort = "high"
minutes = 45
+++

# A charter that can't use a checkout is given none

## Goal

A run whose charter names none of `inspect`, `modify` and `shell` is given no
repository, as an asking charter's is (`temper/docs/design/domain/agent.md`, 4.2).
The drafter's charter, 12, is such a charter (`temper/docs/plan/ui-drafter/notes/contracts.md`,
*The charter*): a design session reads the code through the forge's reads and has
no repository workspace of its own. Today `run_workspace` gives every agent run
that isn't asking its project's home and context repositories.

Acceptance:

1. `pub fn no_workspace(policy: &RunPolicy) -> bool` in
   `temper/crates/temper-engine-domain/src/boundary.rs`, beside `asking`, with the
   doc comment of `notes/contracts.md`, *The workspace*: `asking(policy.tools) ||
   !(policy.inspect || policy.modify || policy.shell)`. `boundary.rs` is exported
   whole (`pub use boundary::*`), so it needs no export line.
2. `run_workspace` (`temper/crates/temper-engine-domain/src/route/forge_route.rs`,
   about line 1760) tests `crate::no_workspace(&charter.policy)` where it tests
   `crate::asking(charter.policy.tools)`, returning `status_route::workspace` as
   it does now. That call is the only change in `forge_route.rs`, a file other
   plans change now.
3. Its row, in the existing table story
   `an_asking_charter_gets_no_workspace_and_every_family_keeps_the_home_and_context`
   (`temper/tests/engine/domain/tests/suite/forge_routes.rs`, about line 2151), no
   new test: its rows `(tools, empty)` gain the charter's three flags, and a third
   row is every family with `inspect`, `modify` and `shell` false, `agents` true:
   empty, the home and context adopted as for the others. The asking row keeps its
   flags as `walking::charter()` gives them. You may rename the story to say it
   holds both, such as `an_asking_or_readless_charter_gets_no_workspace_and_every_family_keeps_the_home_and_context`,
   and its failure message likewise; say so.
4. The status: delete the bullet `**Not yet:** 4.2: a run whose charter names none
   of ...` in `temper/docs/design/domain/agent.status.md` (`notes/common.md`,
   *Status files*).

## Inputs

- `temper/docs/design/domain/agent.md`, 4.2, its last two sentences.
- smith's `smith/docs/design/domain/run.md`, 3.3, *No workspace, no word of one*:
  a run with no workspace keeps its sub-agents (`smith-domain-run/src/workspace.rs`,
  `families`).
- `temper/docs/plan/ui-drafter/notes/contracts.md`, *The workspace*.
- `temper/docs/plan/ui-drafter/notes/common.md`.

## Owns and may change

- Owns: `no_workspace` in `boundary.rs`; the one call in `run_workspace`; the
  story's rows; the one bullet of `agent.status.md`.
- Readers (this cut's survey): `run_workspace` is called from `route/now.rs`. The
  stock charters that name none of the three, 9 and 10, return before the call;
  11 is asking; every other stock charter and every charter of
  `docs/examples/reference/settings.json` inspects, so none of them moves. Tests
  that set a charter's `inspect` false: `temper/tests/client/src/front/account_acts.rs`,
  `temper/tests/engine/domain/tests/suite/trials.rs` and
  `temper/crates/temper-engine-domain/src/tests/room.rs`. If one of them, or any
  story, turns off all three on a charter whose run then needs its checkout, its
  fixture is the oracle the rule trips: set `inspect` true on it, its expectation
  alone, and name it in the report.
- `room.rs` and `settings_parts.rs` keep `crate::asking`: they ask what an asking
  charter is declared, not whether a run has a workspace.

## Steps

1. Read the inputs and `run_workspace`'s head (`forge_route.rs`, about lines 1743
   to 1770).
2. Write the row (acceptance 3), then the rule; break it once (`no_workspace`
   returning `asking` alone) and see the new row fail for it; restore.
3. Narrow test: `heavy cargo nextest run -p temper-engine-domain-world --test suite
   -E 'test(=forge_routes::an_asking_charter_gets_no_workspace_and_every_family_keeps_the_home_and_context)'`
   (or its new name).
4. This changes which runs are given a repository: run the focused tests of
   `-p temper-engine-domain`, `-p temper-engine-domain-world`, `-p temper-service`
   and `-p temper-client-world` (`heavy cargo nextest run -p ...`, one at a time).
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain -p temper-engine-domain-world --all-targets -- -D warnings`.
6. Commit (why, in the message) and push.

## Report

As the implementer's text says, and: the row, with the rule broken and the failure
seen; any fixture whose flags you changed.
