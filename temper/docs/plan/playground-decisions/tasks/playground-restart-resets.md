+++
id = "playground-restart-resets"
part = "playground-restart"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-domain-forge/src/playground.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/playground.rs",
  "temper/tests/engine/domain/tests/suite/playground.rs",
  "temper/crates/temper-service/src/clients/playground_tests.rs",
  "temper/docs/design/protocol/setup.status.md",
  "temper/docs/design/STATUS.md",
]
effort = "high"
minutes = 60
+++

# A start that finds the playground running resets it

## Goal

The fake forge keeps no state across its starts, so after a box restart the engine's records of
the branches it lost would outlive it. The user decided (`inputs/request.md`, decision 1) that a
start that finds the playground running resets it, as the banner's "set up temper" does, and the
person meets the welcome (`temper/docs/design/protocol/setup.md`, 8.4). Today `restart` asks the
box for the fake forge's group again and keeps it running; only a *starting* row is reset.

Acceptance:

- At `Event::ReadAfresh`, a restored `Playground` row *running* is reset exactly as one
  *starting* is: the row reads `Resetting(ResetStep::Project)`, is saved, and the reset's kept
  acts follow in order (the project deleted, the fake forge removed, the scripted account
  removed, then the row erased), each under its reset key. No `BoxGroup` is asked for the
  playground's forge at that start.
- In the engine's world, a start with the playground running ends with no `Playground` row, no
  scripted account and the fake forge removed, once the forge's two kept acts are answered.
- The status entries below are gone; the tests below pass.

## Inputs

- `temper/docs/plan/playground-decisions/notes/common.md`.
- `temper/docs/design/protocol/setup.md`, 8.4: "A start a restart cut short" and "A box restarted
  while the playground runs" (lines ~2244-2251), and "Its reset" (the acts in order, ~2206-2240).
- `temper/docs/design/domain/forge.md`, 4.1: a forge's removal (read only).

## Owns and may change

- `temper-engine-domain-forge/src/playground.rs`, `restart` (line ~309): the `Starting` and
  `Running` arms become one, the reset arm as `Starting`'s is now (`playground_request = None`,
  `Resetting(ResetStep::Project)`, `save`, `reset`); the running arm's `BoxGroup { .., request:
  Token::new(u64::MAX) }` goes. A one-line doc comment on `restart` cites
  `temper/docs/design/protocol/setup.md, 8.4`. `reset`, `end`, `needs_group` and `start_effects`
  stay as they are (`needs_group` is still read by `start_effects`; if the compiler says it is
  unused, remove it and say so).
- `temper-engine-domain-forge/src/tests/playground.rs`: `running_restarts_the_group_without_start_acts`
  (line ~345) encodes the departure; it becomes `a_restart_with_the_playground_running_resets_it`,
  in the shape of `a_start_cut_by_a_restart_resets_the_playground` (line ~318): a running row
  restored with its project and account, then `ReadAfresh` sends the project's `RemoveProject`
  reset act, the row reads `Resetting(ResetStep::Project)`, no `BoxGroup` is asked, and a later
  `connect` makes no `PlaygroundActs`.
- `temper/tests/engine/domain/tests/suite/playground.rs`: a new story beside
  `a_start_cut_by_a_restart_resets_the_playground`, in this file's helpers (`driver`, `start`,
  `row`, `account_rows`, `restart`, `finish_forge_removal`):

  ```rust
  #[test]
  fn a_restart_with_the_playground_running_resets_it() {
      let (mut d, sign_in) = driver(true);
      start(&mut d, sign_in);
      assert_eq!(row(&d).expect("playground fixture").state, forge::PlaygroundState::Running);
      while !d.store.pending.is_empty() {
          d.store.apply();
      }
      restart(&mut d, true);
      finish_forge_removal(&mut d);
      assert!(row(&d).is_none());
      assert!(account_rows(&d).is_empty());
      // and the playground's project is gone from the projects the store lists, as
      // the_reset_removes_the_project_then_the_forge_then_the_account reads it
  }
  ```

  Assert the project's deletion the way an existing story of this file or `reset.rs` reads it; if
  none reads it, assert the `RemoveProject` act's answer instead and say so.
- Reader test, `temper-service/src/clients/playground_tests.rs`,
  `the_playground_read_uses_the_restored_engine_row` (line ~135): it restores a *running* row and
  asserts the domain's and the clients' rows equal it after `fixtures::restart`. If it now fails
  because the start resets the row, keep its purpose (the clients read the restored engine row):
  assert the domain's and the clients' rows are equal and read `Resetting(ResetStep::Project)`.
  Change nothing else in the file.
- Status, as `notes/common.md` says:
  - `temper/docs/design/protocol/setup.status.md`, **Departs**: remove the sentence `8.4, 16: a
    start that finds the playground running asks the box for its group again and keeps it
    running, in place of resetting it; only a start that finds it starting resets it
    (\`temper-engine-domain-forge/src/playground.rs\`, \`restart\`).`; the rest of the line word for
    word. Touch no other line but the date (another task writes the **Not yet** line).
  - `temper/docs/design/STATUS.md`: in its summary paragraph, remove ` and where a box restart
    resumes a running playground (\`protocol/setup.md\`)` and nothing else, so the sentence reads
    "the code departs where the box's group file holds a forge's kind alone (…), and lags in a
    removal without the box …".

Without the box, a removal's group act waits for a pipe that is not there
(`temper/docs/design/domain/forge.status.md`, 4.1, **Not yet**), so a reset without the box
stops at its forge step as `EndPlayground`'s does: not this task's. If a test you run restarts a
running playground with no pipe and now stalls there, list it under `## Left` with that reason.

## Steps

1. `restart` and the forge unit test; `heavy cargo nextest run -p temper-engine-domain-forge
   --lib -E 'test(playground)'`.
2. The world story; `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(playground)'`.
3. `heavy cargo nextest run -p temper-service --lib -E 'test(playground)'`; the reader test as
   above if it fails.
4. The two status edits; the citations check (`notes/common.md`).
5. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-engine-domain-forge -p
   temper-engine-domain-world -p temper-service --all-targets -- -D warnings`; the focused tests
   of `temper-engine-domain-forge`, `temper-engine-domain` (its direct dependent) and
   `temper-engine-domain-world`. This changes what a restored row means, so also run
   `heavy cargo nextest run -p temper-service --lib`. Commit the code and tests, then the status
   text; push.

## Report

As the implementer's text says, and whether the service's reader test needed its change.
