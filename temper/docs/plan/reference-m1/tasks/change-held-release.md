+++
id = "change-held-release"
part = "change"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/tests/engine/domain/tests/suite/forge_routes/held_change.rs", "temper/docs/design/domain/forge.status.md"]
effort = "high"
minutes = 60
+++

# change-held-release: a change held before its producer is asked gets one once released

## Goal

A change held for effects (jig's `Hold::Effects`) while its first step's reads are still
outstanding, then released, is left Active and idle with no producer, on main today. The
reads return after the hold has committed; the change procedure (forge-change `producing`)
sets `Producing { requested: true }` and decides `Delegate(Produce)`; the forge saves that
row; the root's `change_decision` finds no step due (`procedure_due` is `None` while held)
and returns without making the delegate, so nothing resets `requested`. On release the
forge's state is `Producing`, not `Held`, so `release_change` is skipped, and the next step
waits forever on a producer never made. forge.md, 8.4 (a release decides afresh; at
producing with no head pushed, a fresh producer) is the rule it breaks.

Acceptance: a change held for effects before its producer is asked, then released, asks
exactly one producer, and lands once the producer pushes, in the story below; every story of
`temper-engine-domain-world`'s `forge_routes` passes unchanged.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.4 (the paragraph on a release: the change decides
  afresh, and at producing with no head pushed makes a fresh producer).
- jig's `jig/docs/design/domain/connectors.md`, 6.3 (one decision, committed with its new
  state): a decision the root drops must not leave its state behind.
- `temper/docs/design/domain/forge.status.md`: its **Not yet** entry for 8.4 ("a change held
  for effects before its producer was asked ..."), which this task removes.
- `temper/docs/plan/reference-m1/notes/common.md`.

## Owns and may change

Owns the files in the header.

- `forge_route.rs`, `change_decision`: where `procedure_due(task)` gives no step (or not the
  change procedure's), and the forge's choice is `forge_change::Decision::Delegate(kind)`,
  tell the forge the delegate was not made, as a refused admission does today:
  `domain.work.push(Work::Forge(forge::Event::DelegateAdmissionRefused { task, kind }))`
  (the forge's `delegate_admission_refused` resets `requested`, so the next step asks again).
  Read `delegate_admission_refused` (`temper-engine-domain-forge/src/domain.rs`, near line
  2349) first: if it needs an entry in `domain.forge_delegating` or a `kind` that this path
  does not have, add the least that makes it reset this change's `requested` (an event of
  the forge's own, `Event::DelegateUnmade { task, kind }`, in
  `temper-engine-domain-forge/src/boundary.rs` and `domain.rs`, is pre-approved, its body the
  reset alone) and say so in the report.
- Keep the hunk to `change_decision` (and the forge's event, if needed): `forge_route.rs` is
  changed on plan reference-m8's trunk too.
- Options: if the story shows another decision dropped the same way leaving state behind (an
  `Effect`, a `Hold`, `Ready`), do not fix it here: name it under `## Left` with the decision
  and what is left behind. A full admission fence (the forge's state made durable only once
  jig admits the step) is not this task.

## Steps

1. The fix, as above.
2. The story, in `temper/tests/engine/domain/tests/suite/forge_routes/held_change.rs`:
   `a_change_held_for_effects_before_its_producer_asks_its_producer_once_released`.
   - `World::configured(true, true)`, `world.adopt()`, then
     `world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])))`; a chat from
     `adoption_chat` and `assigned_chat(&mut world, 92, ask)` (`forge_routes.rs`); the chat's
     `change` call through `change_tool::change(&mut world, &chat, 93, None, b"Small fix")`.
   - Before the change's first step's reads return, hold it for effects through a party's
     `ProcedureStep` input naming the change task's next step with
     `ProcedureStep::new(StepEnd::Hold(tasks::Hold::Effects))` (the shape the root stories
     send, `rg 'StepEnd::Hold\(tasks::Hold::Effects' temper/tests/engine`); tick; assert the
     change is `Phase::Held` for effects with no delegate.
   - Let the reads return; assert still no delegate.
   - Release with `people::Ask::Release { project: 1, task, revision, again: None }` (the
     plain release; `reviews.rs`'s `release` helper asserts a forge-made hold and does not
     fit); tick.
   - Assert exactly one producer task delegated by the change; drive it as the stories do
     (`world.external` and `fake::advance` for its push, its `Finished` change result); assert
     the change lands (`Landed`) with one merge.
   - Write it first and see it fail on the trunk's tip (no producer after the release); the
     report says it did.
3. Narrow: `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(=forge_routes::held_change::a_change_held_for_effects_before_its_producer_asks_its_producer_once_released)'`
   (adjust the path to the module's real one; check it selected 1). Then `heavy cargo nextest
   run -p temper-engine-domain-world --test suite -E 'test(/forge_routes::/)'` and `heavy
   cargo nextest run -p temper-engine-domain --lib`.
4. Remove the 8.4 **Not yet** entry from `forge.status.md`. `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain -p
   temper-engine-domain-world --all-targets -- -D warnings` (and `-p
   temper-engine-domain-forge` if you added the event). Commit ("Ask a released change's
   producer when its hold dropped the delegate") and push.

## Report

As the implementer's text says, with the story's failure on the tip before the fix.
