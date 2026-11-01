+++
id = "bx-story-reset-cut"
part = "pg"
kind = "feature"
needs = ["bx-story-playground"]
files = ["temper/crates/temper/tests/reset/**", "temper/crates/temper/tests/support/boxed.rs", "temper/crates/temper/tests/support/ctl.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/adoption.rs", "temper/crates/temper-engine-domain-forge/src/tests/adoption.rs"]
effort = "high"
minutes = 60
+++

## Goal

Draft section 8's second process story: the box killed during a reset, which ends whole at
the next start. The reset's kept acts are sent again at the next start (draft 6.2, "Cut by
a restart"), and the store reads as after an uncut reset.

Acceptance:
- `tests/reset/reset_cut.rs` (a `mod` line in `tests/reset/mod.rs`) holding
  `the_box_killed_during_a_reset_ends_it_whole_at_its_next_start`: as
  `a_first_run_into_the_playground_and_out_by_its_reset_leaves_the_store_empty` up to the
  playground running; then `playground reset` started without waiting
  (`support::ctl::Step::Start`), `KillBox` as soon as the box's stderr or the ctl's output
  shows the reset under way (the playground reading `resetting`), `StartBox`; then
  `playground` read until `not_started`; then the same empty reads as the first story (no
  forge, no scripted account, no `Playground` project, no `box/groups/N`, `box/started/N`
  or `forge-N`); the box exits 0 at the end with nothing left.
- A second case, `the_box_killed_twice_during_a_reset_ends_it_whole`: `KillBox` and
  `StartBox` again once the playground's forge reads removing (or is gone from the list
  while the row still reads resetting), then as above.
- Run each alone:
  `heavy cargo nextest run -p temper --profile process-temper --test process -E 'test(=reset::reset_cut::NAME)'`.

## Inputs

- Draft `temper/docs/design/draft/playground-reset.md`, sections 4 ("Kept until whole"),
  6.2 and 8.
- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T5, T6.
- `tests/reset/playground.rs` (bx-story-playground), bx-harness's report.

## Owns and may change

- Owns: `temper/crates/temper/tests/reset/reset_cut.rs` and its `mod` line.
- May change: `tests/reset/playground.rs`, to share the story's first half as a function.
- Options, pre-approved:
  - If no read shows the reset under way before it ends (it ends too fast to cut), cut at
    the first ctl output of the `playground reset` call instead, and say whether the
    restart found a reset to resume (the box's stderr, or the playground read `resetting`
    after `StartBox`).

## Steps

1. The two cases; `heavy cargo check -p temper --tests`.
2. Run each alone; read a failure's log before anything reruns.
3. `cargo fmt`; `heavy cargo clippy -p temper --all-targets`. Commit and push.

## Report

For each case: where the kill landed (the reset's step as read), and what the next start
resumed.

## Answers

2026-11-01. Q: ctl has no playground-state read, so `resetting` and `not_started` cannot be
read: add a reader first, or revise acceptance?

A: Revise acceptance; no ctl tool or production code is added (as bx-story-playground's
answer). Each state is read through what ctl already lists, and T6's order of steps
(`Project`, `Forge`, `Account`, then the row erased) tells which state they prove:
- **`resetting`, first case:** after `playground reset` is started (`Step::Start`), poll
  `me --json` and the forge list; kill as soon as the `Playground` project has left the
  person's projects while the fake forge still lists. Only the reset deletes that project,
  so the row then reads resetting at `Project` or `Forge`, and the next start resumed it.
- **`resetting`, second case:** the first kill as above, `StartBox`; then kill again as soon
  as the fake forge has left the forge list while the scripted account still lists (or, if
  the account has gone first, while `box/groups/N`, `box/started/N` or `forge-N` still
  exists). The account step comes after the forge's, so the row still reads resetting.
- **`not_started`:** as in bx-story-playground's answer: after the restart, call
  `playground reset` under a key of its own (never the cut call's) until it prints
  `refused: not offered` and exits 2 ("no playground runs"); a `reset` answer meanwhile is
  the resumed reset finishing, so call again under another new key. Then the same empty
  reads and filesystem checks as the first story, and the box exits 0.
- **A cut missed** (the cut call answers `reset` before the read shows its phase): the case
  fails naming what was last read. If that happens in your runs, use the pre-approved
  fallback for that kill: kill right after the call is sent; a row still `Running` at the
  kill is reset by the next start anyway (contracts T6, "A start of the engine that finds
  the row Running resets it"), so the case still checks the whole end; report which kill
  fell back and that resumption was then not shown.
- **The shared first half** stops once the playground runs and its forge is connected; it
  does not take the story's `forge-N/admin_token` check, which bx-story-playground's answer
  replaces. Its failing assertion on the prerequisite branch is that task's, not yours.

2026-11-01. Q: the immediate cut dies by signal 15, which the harness judges a failure, and
the restart after a probed cut panics in the forge connector's reconcile (`forge link
Application -> Repository(...)`): wait for the harness and forge owners, or widen this task?

A: Widen this task; do not wait. `bx-harness` and `fe-remove` have landed, so no other run
takes these. Two repairs, then the stories as the answer above says.

1. **The cut is the probe's, not the immediate fallback's.** Your phase-probe run cut where
   the answer above asks (the `Playground` project gone, the fake forge still listed); the
   restart failed only on item 2. Go back to the probe for every kill. The immediate
   fallback stays only for a kill whose probe is missed, as the answer above says.
2. **The connector erases an adoption's retained answer with its repository.** A
   `Stored::Application` whose outcome is `Adopted` names a repository; when the connector
   erases that `Repository` row (the project's release, `domain.rs` near 273; a forgotten
   adoption, `Event::ForgetAdoption`'s `None` arm; and forge removal, if it erases
   repository rows by another path), it also erases each kept `Application` answer naming
   it: removed from `d.application.kept` and `Request::Erase { key: Key::Application(..) }`
   in the same output. That is draft section 5's "an act's retained answer that named it,
   which answers unknown, as one past its retention does"; reconcile's panic stays, since
   a dangling link is then a writer bug again. One helper beside `adoption.rs`'s
   retention erase, called from each site. Unit test in
   `temper/crates/temper-engine-domain-forge/src/tests/adoption.rs`: an adoption answered
   `Adopted`, its project released, then a restore of the saved rows reconciles without a
   panic and the answer reads unknown. Run `heavy cargo nextest run -p
   temper-engine-domain-forge --lib` and its clippy. `fe-world-remove` is running in the
   same crate on `removal.rs` and `tests/removal.rs`: stay out of both.
3. **A cut call that dies by SIGTERM before ctl's handler is a cut.** In
   `tests/support/boxed.rs` (both judgments, near 686 and 750) and `tests/support/ctl.rs`'s
   `check_adoption`, accept `Exit::Signal(15)` beside `Exit::Code(143)` for a call
   `was_cut`. Nothing else in the harness changes; no sleep, and no new readiness point.

Owns and may change, widened: the files in the front matter, for these items only. The
inherited `fake::check::tests::verb` failure is not yours: leave it under `## Left`. The run
is `Done` when both stories pass run alone. A further fault outside these files that keeps
a story red ends the run `Failed:` with the story, the cut's step and the log, not a
question.
