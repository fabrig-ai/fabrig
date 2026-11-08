+++
id = "delegate-held"
part = "drafter-run"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-smith/src/calls.rs",
  "temper/crates/temper-engine-smith/src/tests.rs",
  "temper/docs/design/domain/agent.status.md",
]
effort = "high"
minutes = 50
+++

# The delegate family's tools need the charter's family

## Goal

A run is declared `goal`, `change` and `delegate` only where its charter's
`tools` name jig's delegate family (`temper/docs/design/domain/agent.md`, 4.3).
Today `tools_for` holds only the settings family to the charter and declares every
other host tool to every charter that isn't asking. The drafter's charter, 12,
names every family but delegate (`temper/docs/plan/ui-drafter/notes/contracts.md`,
*The declarations*), so its runs must not be declared them. Stock charters 2, 4,
6, 9 and 10, which don't name the family either, lose the three declarations with
it, as `temper/docs/design/protocol/configuration.md`, 13.4, now says.

Acceptance:

1. `tools_for` (`calls.rs`) leaves out `goal`, `change` (its plan form,
   `PLAN_CHANGE`, included) and `delegate` where `charter.0 &
   jig_domain_core::ToolFamilies::JIG.delegate.0 == 0`, whatever `authority`
   holds. Where the charter names the family, nothing changes: the authority
   still decides each call. The asking branch, `tools(timeout)` (the whole
   registry, which the generated host-tools page and `room.rs`'s measure read) and
   every other family's gate stay as they are. Keep the order of the declarations
   that remain.
2. Its rows, in the existing table test `settings_tools_need_the_charters_family_too`
   (`temper/crates/temper-engine-smith/src/tests.rs`, about line 185), no new test
   function. You may rename it to say it holds both families, such as
   `settings_and_delegate_tools_need_the_charters_family`, and say so:
   - its head, charter 512 over authority `1_539 | 1024`: the expected list loses
     `goal`, `change` and `delegate` beside `settings` and `propose_settings`;
   - its `rows` table: the non-asking expectation (`tools_for(EVERY_FAMILY, ..)`
     with the settings mask) also leaves out the three where `charter & 1 == 0`,
     so row `(1_920, u64::MAX)` loses them and `(1_023, u64::MAX)` keeps them;
   - a new row `(1_022, u64::MAX, None)`, the drafter's mask: no `goal`,
     `change` or `delegate`, and `note`, `recall`, `read_branch` and `read_file`
     declared. Assert those seven names by name in that row, not only through the
     computed expectation.
3. Any other test of this package that pins a declared list for a charter
   without the delegate family (`message_is_offered_only_with_the_runs_message_family`,
   about line 263, builds its assignment's charter policy; check which mask it
   carries) takes the three out of its expectation, keeping what it checks.
4. The status: delete the bullet `**Not yet:** 4.3: the delegate family's ...` in
   `temper/docs/design/domain/agent.status.md` (`notes/common.md`, *Status files*).

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3, the bullet *Every host tool below*,
  its last sentence; and section 10, *Which host tools a charter offers*.
- `temper/docs/design/protocol/configuration.md`, 13.4, *Each charter's host
  families*, its numbers and its last sentence.
- jig's `jig/docs/design/domain/engine.md`, 7.1, step 4, *its charter*: a family
  neither names is not offered.
- `temper/docs/plan/ui-drafter/notes/contracts.md`, *The declarations*.
- `temper/docs/plan/ui-drafter/notes/common.md`.

## Owns and may change

- Owns: `temper/crates/temper-engine-smith/src/calls.rs`, its `tools_for` and
  `offered_tools`; `temper/crates/temper-engine-smith/src/tests.rs`; the one bullet
  of `temper/docs/design/domain/agent.status.md`.
- Readers (this cut's survey): `crate::charter` (`start.rs`) and `room.rs` call
  `tools_for`; `temper-service` (`assignment.rs`, `tests/workers.rs`,
  `tests/stock_brief.rs`, `tests/ask_bound.rs`), `temper`'s
  `tests/suite/producer_policy.rs` and the engine world's
  `tests/suite/configuring.rs`, `configuring/fixture.rs` and
  `forge_routes/{goal_changes,b2_tree,goal_branch}.rs` call
  `temper_engine_smith::charter`. You may change a reader's expectation where it
  pins `goal`, `change` or `delegate` declared to a charter that names no delegate
  family (2, 4, 6, 9, 10, or a fixture's mask), the expectation alone.
- If a scripted model in a story calls `goal`, `change` or `delegate` from a run
  whose charter names no delegate family (a coder's, a reviewer's), the script is
  the oracle the correct behaviour trips: give the fixture charter the family
  (its `tools` mask) if the story is about the call, or take the call out if it
  isn't, and name it in the report.
- If a refusal's hint (`answers.rs`, `change_hint`: "For a change to a repository,
  call change ...") would name `change` to a run that isn't declared it, and the
  charter's `tools` reach that code with no new parameter, leave the hint out
  there; otherwise leave it as it is and say so in the report.

## Steps

1. Read the inputs, then `tools_for` and `offered_tools` (`calls.rs`, about lines
   239 to 290). Thread the charter's delegate bit into `offered_tools` beside its
   other gates; `tools(timeout)` passes every family, so it keeps them.
2. Write the rows (acceptance 2), then the rule; break the rule once (declare the
   three again) and see the new row fail for it; restore.
3. Narrow tests: `heavy cargo nextest run -p temper-engine-smith --lib -E
   'test(=tests::settings_tools_need_the_charters_family_too)'` (or its new
   name), then `heavy cargo nextest run -p temper-engine-smith`.
4. This changes what a run is declared, a meaning its readers see: run the
   focused tests of `-p temper-service`, `-p temper-engine-domain-world`,
   `-p temper-client-world` and `-p temper` (`heavy cargo nextest run -p ...`, one
   at a time), and `heavy cargo check --tests -p temper-one-process-world -p
   temper-real-loop-world`.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy
   -p temper-engine-smith --all-targets -- -D warnings`.
6. Commit (why, in the message) and push.

## Report

As the implementer's text says, and: the rows added or changed, with the rule
broken and the failure seen; each reader whose expectation changed, and why; what
you did with `change_hint`.
