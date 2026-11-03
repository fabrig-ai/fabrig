+++
id = "unname-at-erasure"
part = "kept-names"
kind = "feature"
# kept-names maps Ask::Unname in temper's root (today an unreachable! arm there), so jig may send
# it only after; deletion-grace is jig-domain-core's first writer in this plan.
needs = ["kept-names", "deletion-grace"]
files = [
  "jig/crates/jig-domain-core/src/routing/task/erase.rs",
  "jig/crates/jig-domain-core/src/reads/forget.rs",
  "jig/crates/jig-domain-core/src/effects/connector.rs",
  "jig/crates/jig-domain-core/src/tests/routing/erase.rs",
  "jig/crates/jig-domain-core/src/tests/reads/forget.rs",
  "jig/testing/jig-test-domain/src/routes.rs",
  "jig/docs/design/domain/connectors.status.md",
]
effort = "medium"
minutes = 45
+++

# jig tells every connector when a task is erased

## Goal

A connector may keep what an ended task named until the task is erased. temper's forge keeps a
removed forge's repository names for its kept chats that way (`kept-names`). Nothing tells a
connector of an erasure today: `connector::Ask::Unname` is sent by no code. This task makes the
core ask every numbered connector `Ask::Unname { task }` in the decision that erases a task whole
(`notes/contracts.md`, C1). That is the last decision of a deleted project's erase walk for that
task, and the decision that forgets a task forgotten once read. Retention at the horizon is not
built; when it is, it shares the erase walk's last decision.

Acceptance:

- The decision that erases an ended task's record in the erase walk (`routing/task/erase.rs`,
  `Phase::Record`) carries one `Request::Ask { connector, ask:
  Ask::Effect(connector::Ask::Unname { task }) }` per connector in `core.connectors`. The rows'
  and cleanup's decisions carry none.
- `reads::forget::forget`, when it erases the task (its `if` holds), carries the same asks beside
  the record's erases. When it erases nothing, it asks nothing.
- jig's test domain routes `Ask::Unname` to no event. Its connector's `Unnamed` stays mapped from
  `Ask::Release`, as today. jig's ops example already ignores it. temper's root maps it since
  `kept-names`.
- The status entry below is gone. The tests below pass, and so do the readers' tests in Steps.

## Inputs

- `temper/docs/plan/playground-decisions/notes/common.md`;
  `temper/docs/plan/playground-decisions/notes/contracts.md`, C1.
- `jig/docs/design/domain/connectors.md`, section 2 (the table's first row) and 3.4.
- `jig/docs/design/domain/people.md`, 5.2 ("Its tasks ended, then erased") and section 6 (the
  retention class, forgotten once read): read only.

## Owns and may change

- `jig-domain-core/src/routing/task/erase.rs`, `loaded`, the `Phase::Record` arm (~165): after
  its `Write::Erase(Key::Tasks(tasks::Key::Ended(number)))`, one `Request::Ask` per `&connector
  in core.connectors.as_ref()`, in that order, with `Ask::Effect(crate::effects::connector::Ask::
  Unname { task: number })`. Nothing else in the file.
- `jig-domain-core/src/reads/forget.rs`, `forget` (~88): the same asks for `owner.task`, inside
  the `if`, after the record keys' erases. The module doc may add `connectors.md, 3.4`.
- `jig-domain-core/src/effects/connector.rs` (~230-236), doc comments only. On `Names`: "The live
  task names these resources." On `Unname`: "The task is erased whole: nothing it named need be
  kept for it (jig/docs/design/domain/connectors.md, 3.4)." Touch nothing near line 430 and
  after: the `reference-m8` branches edit there.
- `jig/testing/jig-test-domain/src/routes.rs`, `route_effect_ask` (~780): move
  `core::effects::connector::Ask::Unname { .. }` out of the `unreachable!("effect handoff
  vocabulary")` arm (~835). It pushes no `Work::Connector`: return before the push, or wrap the
  match's value in an `Option`. Use whichever reads better and say which. `Ask::Names` stays in
  the `unreachable!` arm. Change nothing else in the file; the `reference-m1` branches edit lines
  ~186-215.
- If a test of `jig-domain-core` asserts the exact requests of an erase walk's record decision
  or a forget's decision, and now meets the new asks, extend its expectation by those asks alone
  and say which.

Not yours: `jig/examples/ops/domain/src/route.rs` (it already ignores `Unname`); temper's root
(`kept-names` maps it); `admission/deletion.rs` and its tests (`deletion-grace`).

## Steps

1. The erase walk, test first, in `src/tests/routing/erase.rs` with its helpers (`ended`,
   `page`, `finish_rows`, `erases`). The tree core's connectors are `[1, 2]`
   (`tests.rs`, `connectors: Box::new([1, 2])`):

   ```rust
   fn unnamed(out: &Queue<Request>) -> Box<[(u16, u64)]> {
       // Each Request::Ask { connector, ask: Ask::Effect(connector::Ask::Unname { task }) }, in order.
   }

   #[test]
   fn every_connector_is_told_in_the_decision_that_erases_the_record() {
       let (mut core, limits) = make_tree();
       let record = fixtures::ended_report(9, tasks::Party::Person(1));
       let rows = ended(&mut core, &limits, record);
       assert!(unnamed(&rows).is_empty());
       let rows = page(&mut core, &limits, &rows, Box::new([]), None);
       let (cleanup, record_out) = finish_rows(&mut core, &limits, &rows);
       assert!(unnamed(&cleanup).is_empty());
       assert_eq!(unnamed(&record_out).as_ref(), &[(1, 9), (2, 9)]);
   }
   ```

   Fit the page sequence to what `an_ended_task_is_erased_whole_and_its_record_last` drives
   (its rows page, then `finish_rows`). The assertion that matters: the asks come in the decision
   whose erases are `[Key::Tasks(tasks::Key::Ended(9))]`, and in no other.
   `heavy cargo nextest run -p jig-domain-core --lib -E 'test(erase)'`.
2. The forget, in `src/tests/reads/forget.rs`:

   ```rust
   #[test]
   fn a_forget_tells_every_connector_the_task_is_erased() {
       // make_tree(); fixtures::ended_report(TASK, Person(PARTY)) with its retention
       // ForgottenOnceRead, as this file's CASES set it, and its Owner::of_row (result_position 7);
       // core.counters.deployment.messages = 7, so the forget's condition holds;
       // crate::reads::forget::forget(&mut core, &mut Queue::uncapped(), &owner):
       // its output erases Key::Tasks(Ended(TASK)) and asks Unname { task: TASK } of 1 and 2.
       // With messages = 6 it erases nothing and asks nothing.
   }
   ```

   `heavy cargo nextest run -p jig-domain-core --lib -E 'test(forget)'`.
3. The test domain's arm; `heavy cargo nextest run -p jig-test-domain`.
4. Status, as `notes/common.md` says: `jig/docs/design/domain/connectors.status.md`, **Not yet**:
   remove `3.4: a task's erasure told to every connector: nothing sends \`Unname\`
   (\`jig-domain-core/src/effects/connector.rs\`), neither a deleted project's erasure of its
   tasks (\`routing/task/erase.rs\`) nor the forget of a task forgotten once read
   (\`reads/forget.rs\`).` (its four lines) and nothing else but the date. The citations check.
5. `cargo fmt`; old forms for jig; `heavy cargo clippy -p jig-domain-core -p jig-test-domain
   --all-targets -- -D warnings`. This changes what an erasure's decision holds, which every
   application root reads, so run the focused tests of `jig-domain-core`, `jig-test-domain`,
   `jig-core-world`, `jig-conformance-world`, `jig-test-connector-world`, `jig-ops-domain`,
   `jig-ops-world`, `temper-engine-domain` (`--lib`) and `temper-engine-domain-world` (its
   `removal`, `reset` and `playground` suites at least:
   `-E 'test(removal) | test(reset) | test(playground)'`). Commit the code and tests, then the
   status text; push.

## Report

As the implementer's text says. Also: how the test domain's arm routes nothing, and every test
whose expectation you extended.
