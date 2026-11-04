+++
id = "guard"
part = "guard"
kind = "feature"
needs = []
files = [
  "temper/crates/temper/tests/support/held_login.rs",
  "temper/crates/temper/tests/support/forge_land/rung.rs",
  "temper/crates/temper/tests/live.rs",
  "temper/crates/temper/tests/process.rs",
  "temper/crates/temper/tests/suite/startup.rs",
  "temper/docs/design/domain/testing.status.md",
]
effort = "high"
minutes = 75
+++

# A held record comes back from every ending of the landing story

## Goal

`a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge` moves a held record into its store.
After a good first sitting it leaves the record there with the box stopped, since the second
sitting proves the restart on it. A panic in that gap (`temper/crates/temper/tests/live.rs`, ~114,
`landed_identity`, then `support/ctl.rs:38`) finds no box up. The story's return is behind it or
ahead of it, the launcher's `give-back` needs a box to run `temper ctl`, and nothing exports the
record. A live run lost a login this way.

Build the guard of `temper/docs/design/domain/testing.md`, 2.7, "The held record's guard": `Lent`,
standing from before the first sitting to the test's end. Its `Drop` settles what is owed on every
ending, and starts a box of its own where none is up.

This is test support only: no product change, and no change to what a story asserts. The launcher
(`temper/scripts/proof-live.sh`), `held_login::due_back`, `LiveJudge`'s `Returning` phase and the
rung's `catch_unwind` in `run_walk` stay as they are.

Acceptance:

- With the box of a sitting stopped and the record moved in, a panic leaves the record at its lent
  path: a regular file, mode 0600, not empty. The panic that reaches the caller is the story's own.
  The box the guard started has stopped. `process.rs` shows this on a synthetic record and the fake
  issuer, with no live login.
- A record at its lent path owes nothing: no hook runs, no box starts. That is how the story's own
  return and an import that never ran both disarm the guard, so it exports once. A path holding
  anything but a private regular file with bytes in it is reported and left alone.
- A box still up returns the record through the installed `give-back`, and no box starts.
- A record still not back gives one notice, `LOGIN NOT BACK: {message}; store {store} retained for
  recovery`, with the store left in place. While the test unwinds, the guard does not panic and the
  story's panic stays the one reported. On a test that is not unwinding, the guard panics with that
  message.
- Each test you add bites, as your implementer text says, and the report names the rule broken and
  the failure seen for each.
- The 2.7 sentence on the guard is gone from `testing.status.md`.

## Inputs

- `temper/docs/design/domain/testing.md`, 2.7: "The held record's guard", and "The runner's part"
  in the bullet before it, which stays the cover for a signal. Section 11, "A signal between the
  landing story's sittings", is deferred and not yours.
- `temper/crates/temper/tests/support/held_login.rs`: `return_hook`, whose checks `at_rest` matches.
- `temper/crates/temper/tests/support/forge_land/rung.rs`: `Driver`, `start_box`, `stop_box`,
  `run_walk`, `setup`.
- `temper/crates/temper/tests/process.rs`: `account_transfer_restart` and
  `imported_restart_account`, the shape of the new test.
- `temper/crates/temper/tests/suite/startup.rs`:
  `a_held_login_return_hook_is_idempotent_and_refuses_unsafe_metadata_without_reading_the_file`,
  whose stub `temper` the new suite tests copy.
- `skein/docs/foundation/testing-strategy.md`, 2.10 and 8.1, on harness deadlines and budgets.

Rules for this task:

- Never read a record: `stat` alone, on the synthetic fixtures too. The guard's words never name
  the record's path; they name the store.
- Never run a live test and never set `TEMPER_LIVE_ACCOUNT` or `TEMPER_LIVE_CLAUDE`. `live.rs` is
  checked by clippy's `--all-targets`; nothing offline runs its wiring, and the report says so.

## Owns and may change

Owned:

- `temper/crates/temper/tests/support/held_login.rs`: adds `Lent`, `not_back` and `at_rest`.
  `return_hook` and `import_guard` keep their signatures and bodies; `return_hook`'s doc comment
  may name the guard beside the launcher's trap.
- `temper/crates/temper/tests/support/forge_land/rung.rs`: adds the two `return_*` functions and
  their helpers. `start_box` takes `binary: &Path` first. `run_walk`, its one caller (~559), passes
  `&driver.binary` and changes in nothing else. `setup` is not touched: the new box has its own
  link wait, because it must also see the box exit.
- `temper/crates/temper/tests/live.rs`: `owned_forge_landing` gains the guard, nothing else.
- `temper/crates/temper/tests/process.rs`: one new test, using `imported_restart_account` and
  `no_model_scripts` as they are.
- `temper/crates/temper/tests/suite/startup.rs`: the three new tests.
- `temper/docs/design/domain/testing.status.md`: remove the sentence beginning "2.7: the held
  record's guard". This departs from your implementer text, which keeps you out of `docs/design/`:
  the status records only gaps, and this one closes with your branch. Nothing else under
  `docs/design/` is yours.

Not changed: `support/account_move.rs` (a synthetic record between two fake deployments, nothing
lent) and `support/forge_land.rs` (`LivePlan` keeps installing the hook each sitting; its
`return_failed` words stay).

Pre-approved options:

- **The new box needs the issuer.** If `account list` or `account export` fails on the box the
  guard starts because the first sitting's fake issuer has gone, keep one up across the guard's
  drop in an independent world on `std::thread::scope`, as `account_move::run` (`Peers`) and
  `rung::offline_walk` do, built from `support::sign_in::restart_issued_issuer`. The change stays
  in the test, not in the guard. Say so in the report.
- **A bound.** The 240 s for the new box to answer, 60 s for each `give-back` and 30 s for its stop
  are this act's own. If a process run shows one is wrong, change that figure and give the
  measurement in the report; the 60 s return is the story's figure and stays.
- **The link.** `setup` finds it as a line of `box.out` starting `http://` and containing
  `secret=`. If the fake deployment prints it otherwise, match both forms.
- **Stopping the new box.** A box that exits non-zero or needs killing after the record is back is
  a `live::notice`, not `LOGIN NOT BACK`: the record is what the guard answers for.

## Steps

### 1. rung.rs

```rust
/// The guard's return through the installed `give-back`, which a box still up answers
/// (temper/docs/design/domain/testing.md, 2.7).
pub fn return_through_hook(binary: &Path, store: &Path) -> Result<(), String>;

/// The guard's return with no box up: a box over the store for this alone, a new client signed
/// in by the box's link, then `give-back`. The box stops on any ending; the store stays.
pub fn return_on_a_new_box(
    binary: &Path,
    store: &Path,
    paths: &[PathBuf; 3],
    account: &Path,
) -> Result<(), String>;

fn start_box(binary: &Path, store: &Path, paths: &[PathBuf; 3]) -> Child;
```

- Both use `Driver::owned(store, &client)` with `driver.binary = binary.into()`, so `execute` keeps
  the bounds, the outside-a-world `Deadline` and the `call-N` files. Each stage calls `driver.arm`
  with its own phase name.
- `return_through_hook` runs `Command::new(store.join("give-back"))` within 60 s. A missing hook
  is an `Err`, not a panic.
- `return_on_a_new_box`, in this order, so that the calls that may panic come before the box does:
  1. `client = store.join("return-client")`, made with `create_dir_all`, mode 0700;
  2. `return_hook(binary, store, &client, account)`, which rewrites `give-back` for this client;
  3. arm 240 s, then `start_box`;
  4. wait for the link in `box.out`, each round checking `boxed.try_wait()` (a box that exited is
     an `Err` at once, naming `box.err`) and `driver.check_wait()`;
  5. `driver.ctl(&["sign-in", "-"], Some(&link))`;
  6. arm 60 s and run `give-back`;
  7. stop the box, on every result of 3 to 6: TERM by `parent::signal`, poll `try_wait` under a
     30 s `Deadline`, then `parent::end_child`.
- Each return sends `give-back`'s stdout, its `LOGIN BACK` line, to `live::notice`, as
  `finish_story` does.
- A starved wait comes back from `check_wait` as an `Err`. Treat it as any other: the record is
  not back, whatever the load. Never call `Starved::pass` here.

### 2. held_login.rs

```rust
/// A held record lent to a story's store, from before its import to the test's end
/// (temper/docs/design/domain/testing.md, 2.7). A record at its path owes nothing, so the
/// story's own return disarms this, and an import that never ran leaves it nothing to do.
pub struct Lent {
    binary: PathBuf,
    paths: [PathBuf; 3],
    account: PathBuf,
}

impl Lent {
    #[must_use]
    pub fn new(binary: &Path, paths: &[PathBuf; 3], account: &Path) -> Self;
}

/// What a record that did not come back leaves in the notices.
#[must_use]
pub fn not_back(message: &str, store: &Path) -> String; // "LOGIN NOT BACK: {message}; store {} retained for recovery"

/// A record's path by `stat` alone: a regular file, 0600, not empty, as `give-back` checks it.
fn at_rest(account: &Path) -> Result<(), String>;

impl Drop for Lent {
    fn drop(&mut self) {
        let settled = catch_unwind(AssertUnwindSafe(|| self.settle()))
            .unwrap_or_else(|_| Err("the return panicked".to_owned()));
        if let Err(message) = settled {
            let message = not_back(&message, store);
            super::live::notice(&message);
            if !std::thread::panicking() {
                panic!("{message}");
            }
        }
    }
}
```

The store is `paths[0].parent()`. `settle`, private:

1. `symlink_metadata(account)` succeeds: return `at_rest(account)`, running nothing.
2. `return_through_hook`. `Ok` ends it.
3. Its `Err`, and the path now holds something: `at_rest`, with no box.
4. Otherwise `return_on_a_new_box`, its `Err` joined to the hook's in the message.

There is no armed flag: the `stat` is the disarm, and the design says why.

### 3. live.rs

In `owned_forge_landing`, right before `let result = std::panic::catch_unwind(...)`, so that it
outlives the `catch_unwind`, is dropped while `resume_unwind` unwinds (after the `FAILURE` notice)
and, on a pass, stats the path at the function's end:

```rust
let _lent = if let forge_land::LiveModel::ChatGpt { account: Some(account) } = model {
    Some(support::held_login::Lent::new(&skein_scratch::paths::binary("temper"), &paths, account))
} else {
    None
};
```

A Claude login is used in place and never moves, and a person's sign-in holds nothing lent, so
neither gets a guard. `rung_b0...` needs none.

### 4. The tests

In `process.rs`, beside `a_restart_over_an_imported_then_exported_account_answers_ctl`. Run it
under `--profile process-temper`, since the default profile leaves `process` out:

```rust
/// temper/docs/design/domain/testing.md, 2.7: the landing story's panic between its sittings.
#[test]
fn a_story_panicking_after_its_box_stopped_leaves_the_held_record_at_its_path() {
    use support::ctl::{Call, Step};
    let scratch = support::Scratch::new("held-login-guard");
    let paths = ["config", "data", "cache"].map(|name| scratch.path().join(name));
    imported_restart_account(scratch.path(), &paths);
    let held = scratch.path().join("incoming-account.move");
    let temper = skein_scratch::paths::binary("temper");
    // As `run_live` does: the sitting's hook, which fails on a stopped box as the launcher's does.
    support::held_login::return_hook(&temper, scratch.path(), &scratch.path().join("client"), &held);
    let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _lent = support::held_login::Lent::new(&temper, &paths, &held);
        let sitting = support::boxed::run_ctl_with_account(
            scratch.path(),
            support::boxed::echo_steps(deployment_flags(&paths)),
            scratch.path(),
            vec![
                Step::Run(Call::sign_in()),
                Step::Run(Call::new(&["account", "import", held.to_str().expect("move file"), "--key", "guard"])),
                Step::Run(Call::new(&["account", "list", "--until", "usable"])),
            ],
            BOX_DEADLINE,
            || (),
            support::forge_land::fake_face(support::sign_in::restart_issued_issuer(&paths[0]), no_model_scripts()),
        );
        ctl_assert_success(&sitting.calls);
        assert_eq!(sitting.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&sitting.stderr));
        assert!(!held.exists(), "the record moved in");
        panic!("the story's assertion after its box stopped");
    }));
    // The panic is the story's; the record is back by stat; the guard's box has stopped: the
    // deployment's client listener (`application.clients.listener.socket`) refuses a connection.
}
```

In `suite/startup.rs`, after `a_held_login_import_refuses_after_returning_without_starting_ctl`,
each on `Scratch::startup` with `paths` under its root and `support::held_login` as its surface:

- `a_held_record_at_its_path_owes_the_guard_nothing`: a synthetic record at 0600 and
  `Lent::new(Path::new("/must-not-export"), ..)` dropped: no `give-back` ran, no `returning`,
  `return-client` or `box.out` appeared. The same record at 0644: the drop panics with
  `LOGIN NOT BACK`, and the file's mode and length are unchanged.
- `a_lent_record_returns_through_the_box_still_up_and_once`: the stub `temper` of
  `a_held_login_return_hook_is_idempotent_...`, also appending a line to a `seen` file for each
  call. `return_hook` over the stub, the record removed, a `Lent` dropped: the record is back at
  0600, no `return-client` or `box.out` appeared. A second `Lent` dropped: `seen` holds one
  `export`.
- `a_lent_record_not_back_keeps_its_store_and_the_storys_panic`: a stub `temper` that only exits 1,
  the record absent. A `Lent` dropped inside `catch_unwind` with `panic!("the story's own failure")`
  after it: the payload is that panic, and the process did not abort. A `Lent` dropped alone inside
  `catch_unwind`: the payload starts `LOGIN NOT BACK: ` and ends
  `; store {root} retained for recovery`. The root and its files are still there both times. The
  stub's `box` exits at once, so neither waits on the 240 s bound.

The rule each breaks by hand: `return_on_a_new_box` removed (the process test: no record at the
path); step 1 of `settle` removed (nothing owed: `give-back` runs); the hook step moved after the
new box (box still up: `box.out` appears); the `thread::panicking` check removed (the process
aborts).

### 5. Checks

Narrowest first:

```sh
heavy cargo nextest run -p temper --test suite -E 'test(~a_held_record_at_its_path) | test(~a_lent_record)'
heavy cargo nextest run -p temper --profile process-temper --test process \
  -E 'test(=a_story_panicking_after_its_box_stopped_leaves_the_held_record_at_its_path)'
```

Then, once, as your text says: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py` (the words
`actual` and `genuine` in a comment count); `heavy cargo clippy -p temper --all-targets -- -D
warnings`, which is what checks `live.rs`; `heavy cargo nextest run -p temper`. `start_box` has no
reader outside `rung.rs`, and nothing in `temper` has a dependent reading this support code.

`rung.rs`'s walk changes only in one argument, so `proof_live` need not run. If you touch more of
it, run `proof_live` through `temper/scripts/proof-live.sh rung-b0-offline`.

Commit and push as your text says.

## Report

As the implementer's text says, and also:

- the process test's wall time and CPU seconds, one run: the tier's budget is 60 s of CPU a test
  (`tools/tiers/budgets.toml`, `process-temper`). If it is over, ask as your text says, with the
  measurements.
- whether the new box needed the issuer, and what the box took to answer.
- each bite: the rule broken and the failure seen.
- what no offline run reaches: `live.rs`'s five lines, and the real store's restart with an owned
  Forgejo.
