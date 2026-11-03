+++
id = "goal-jig-writers"
part = "goal-jig"
kind = "feature"
needs = []
files = ["jig/tests/tasks/tests/suite/holds.rs"]
effort = "medium"
minutes = 35
+++

# goal-jig-writers: writer slots are taken by what a run writes

## Goal

Pin, in jig's tasks world, that a run takes the writer slot of each held resource its
workspace writes and none of a resource it only reads, so that a goal holding its own branch
and its run branch runs while its delegates' effects write its own branch in turn (contract
C3: "G's runs, inspect only, take none"). The code already claims by the writes it is given
(`jig/crates/jig-domain-tasks/src/writers.rs`, `claim(.., writes: &[Name], ..)`); this task
adds the stories that hold it, and changes no crate.

Acceptance, in `jig/tests/tasks/tests/suite/holds.rs`:

- `a_run_writing_one_of_two_held_resources_leaves_the_other_to_its_delegates_effects`: task 1
  holds two write resources (`resource(10)` and `resource(11)`, the helper of that file); its
  run is claimed writing `resource(10)` alone (`world.claim_budget_writing(1, 1, 50,
  Box::new([name10]))` gives `Reply::Done`); a delegate, task 2 made by `Party::Task(1)` with
  no holdings, sends `Event::EffectInFlight { task: 2, resource: name11, entry: 70 }` and gets
  `Reply::Done` while task 1's run is still claimed; a second effect of task 2 on `name11`,
  entry 71, gets `Reply::WriterWaiting(name11)` until `Event::EffectSettled { resource:
  name11, entry: 70 }`, then `Reply::Done`. The stored `Stored::Writer` rows show
  `Writer::Run { task: 1, attempt: 1 }` on `name10` only and `Writer::Effect { entry: .. }` on
  `name11` only.
- `a_run_claimed_reading_only_takes_no_writer_slot`: task 1 holds `resource(12)`; its run is
  claimed with no writes (`Box::new([])`): `Reply::Done`, and no `Stored::Writer` row names
  `resource(12)`; an effect of task 1 on it then gets `Reply::Done` at once.

## Inputs

- `jig/docs/design/domain/tasks.md`, 6.1 ("A held resource a queue orders") and 6.3 ("A run
  takes the slot of each held resource its workspace writes"); `jig/docs/design/domain/
  engine.md`, 7.1 (step 3, "the writer slots of the holds its workspace writes").
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C3.

## Owns and may change

- Owns `jig/tests/tasks/tests/suite/holds.rs`: the two tests, beside the existing writer
  stories (`a_connector_effect_waits_for_a_run_and_owns_the_writer_until_it_settles` is the
  model), reusing `configured`, `resource` and `task`.
- No crate changes. If a test fails because the tasks child takes a slot the claim did not
  name, or refuses a delegate's effect on a resource its requester holds, stop: report it
  with the failing assertion and the log (`Question:`), rather than changing
  `jig-domain-tasks`. If the world lacks a helper the tests need (a claim with an empty write
  list), add it in this file as a private function.
- No design edit: tasks.md 6.3 has no **Not yet** entry for this.

## Steps

1. Read the existing writer stories in `holds.rs` (from
   `a_run_hands_a_held_resource_to_a_procedure_while_it_still_holds_the_writer_slot` down).
2. Write the two tests; run each: `heavy cargo nextest run -p jig-tasks-world --test suite
   -E 'test(=holds::a_run_writing_one_of_two_held_resources_leaves_the_other_to_its_delegates_effects)'`
   and the same for `holds::a_run_claimed_reading_only_takes_no_writer_slot` (check each
   selects one test; adjust the path to what `--list` shows).
3. `cargo fmt`; `heavy cargo clippy -p jig-tasks-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p jig-tasks-world`.
4. Commit ("jig tasks: stories for writer slots taken by what a run writes"), push.

## Report

The two tests and what each asserts, the counts run, and anything the tasks child did that
the design does not say.
