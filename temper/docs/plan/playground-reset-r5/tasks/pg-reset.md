+++
id = "pg-reset"
part = "playground"
kind = "feature"
needs = ["pg-start", "jp-close"]
files = ["temper/crates/temper-engine-domain-forge/src/**", "temper/crates/temper-engine-domain/src/**", "temper/tests/engine/domain/**"]
effort = "high"
minutes = 110
+++

## Goal

The playground's reset on the engine's side (draft
`temper/docs/design/draft/playground-reset.md`, 6.2, and section 9's
first default): `EndPlayground` turns the row `Resetting` and keeps the
reset's acts in order (the playground's project deleted, its fake forge
removed, its scripted account removed, then the row erased), each kept
until answered across restarts; and every start resets a playground it
finds running.

Acceptance:

- `Ask::EndPlayground` (answered *not offered* since `pg-start`) runs
  `notes/playground.md`, section 4, in the forge connector
  (`src/playground.rs`): in one decision the row saved `Resetting` and the
  first step's `Kept` row saved and sent (`Request::SendKept`, the root
  sending every kept row's act, `notes/remove-forge.md`, section 4):
  1. `KeptAct::RemoveProject { project }` (key: step 5), jig's
     `Act::RemoveProject`; its answer `ProjectRemoved`, or `Refused(Unknown)`
     (a deletion already finished, or a project never made: jig's
     `notes/jig-remove-project.md`, section 7), erases the kept row and
     clears `project`, and in that decision
  2. `remove_forge(forge, out)` (`rf-domain`) with `forge` cleared (a
     `NoSuchForge` refusal counts as done), then
  3. `KeptAct::RemoveAccount { account }` (key: step 6) saved and sent in
     the same decision; its answer (removed, or unknown) erases it and
     clears `account`;
  4. all three `None`: the row erased; every waiting `EndPlayground`
     answered (`Answered::EndPlayground(EndAnswer::Reset)`, a new
     `EndAnswer { Reset }`; its wire form `reset` is `pg-svc`'s).
  A kept act refused for retry (`Again`, the grant missing, no first
  owner) stays kept and is named in a note, as the demo's end acts were
  (`end_acted`'s policy moves here). Steps whose field is `None` are
  skipped. `EndPlayground` with no row answers `Reset` at once; with a
  `Resetting` row, it waits with the others (bounded as starts are).
- While `Resetting`, a `StartPlayground` is answered *not offered*; the
  playground's forge is ended from step 2 (`forge_ended`), and its chats
  end with the project's deletion (jig's).
- At a start (`restart`, replacing `pg-start`'s group ask): a `Running`
  row is reset as `EndPlayground` resets it, after learning its unknown
  numbers: `project: None` resends step 1's `MakeProject` with its key and
  the row's `maker` (its retained answer names the project; *unknown* or
  a refusal means none was made); `account: None` takes the
  deployment-held account of the scripted provider, if any. A
  `Resetting` row's kept rows are resent as every kept row is, and the
  step in progress goes on from their answers.
- Tests, `temper-engine-domain-world`'s `suite`, `tests/suite/playground.rs`:
  `end_playground_deletes_the_project_removes_the_forge_and_the_account_then_erases_the_row`,
  `the_reset_keeps_its_acts_in_order_one_decision_each`,
  `a_reset_cut_after_each_kept_act_ends_whole_at_the_next_start` (a crash
  after each step's commit, resumed to the same end: no row, no kept row,
  the project unknown to jig, the forge's record removing or gone, no
  deployment account of the scripted provider),
  `a_start_resets_a_running_playground_and_learns_an_unrecorded_project`,
  `end_playground_with_no_row_answers_reset`,
  `a_start_while_resetting_is_not_offered`,
  `an_ungranted_reset_act_stays_kept_and_is_named`; after the reset a
  start makes a new playground with new numbers and keys. The forge
  crate's library tests on the row's steps.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (sections 1
  to 3), `temper/docs/plan/playground-reset-r5/notes/playground.md`
  (sections 1, 2, 4: binding), `temper/docs/plan/playground-reset-r5/notes/remove-forge.md`
  (sections 2 to 4), `temper/docs/plan/playground-reset-r5/notes/jig-remove-project.md`
  (sections 2 and 7: what `RemoveProject` answers).
- The draft, 6.2 and section 9. `temper/docs/design/protocol/setup.md`,
  8.4 (today's kept end acts, the model for kept rows).
- `temper/docs/plan/playground-reset-r5/tasks/pg-start.md` (what the
  start left).

## Owns and may change

- Owns: `temper-engine-domain-forge`, `temper-engine-domain`, the
  engine-domain world.
- May change, call sites only: `temper-service`'s exhaustive arms for
  `EndAnswer` (answered as `pg-start` answered `EndPlayground` there,
  until `pg-svc`), the other engine worlds' arms.
- Options: if `remove_forge` refuses the playground's forge because a
  kept `Cancel` of its own is outstanding, retry it at that act's answer
  and say so; if a crash between a jig answer and its row save makes
  step 1's resend answer `Refused(Unknown)`, take it as done (the
  deletion is jig's and whole).

## Steps

1. The reset's steps in the connector; the root's answers by key.
2. The start's reset; the world tests; narrow runs: `heavy cargo nextest
   run -p temper-engine-domain-world --test suite -E
   'test(/playground/)'`, `heavy cargo nextest run -p
   temper-engine-domain-forge --lib -E 'test(/playground/)'`.
3. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge -p
   temper-engine-domain -p temper-engine-domain-world --all-targets -- -D
   warnings`; `heavy cargo check -p temper-service --tests`; the focused
   tests of those and `temper-engine-forge-world`. Old-forms from
   `temper/`. Commit and push.

## Report

As `notes/common.md` says.
