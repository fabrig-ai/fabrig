+++
id = "io-on-stage"
part = "io-worlds"
kind = "feature"
needs = ["io-output-loop"]
files = ["skein/crates/skein-io/src/stage.rs", "skein/tests/io/src/world.rs", "skein/tests/io/src/output.rs", "skein/tests/io/src/owner.rs", "skein/tests/io/src/ledger.rs", "skein/tests/io/src/lib.rs", "skein/tests/io/tests/suite/stage.rs", "skein/docs/design/io.status.md", "skein/docs/design/testing.md", "skein/docs/design/testing.status.md", "skein/scripts/old-forms.txt", "skein/docs/development/debt/D032-world-harness-copies.md"]
effort = "high"
minutes = 70
+++

# skein: io's worlds run on the stage a service runs on

## Goal

The last step of `skein/docs/development/debt/D032-world-harness-copies.md`: **Paid when** holds, and the item goes. `Proc::iterate` and `Driver::iterate` call `skein_io::stage::{up, down}`, so these worlds hold io to the drive a service uses. The per-call `MAX_OUT` check moves into the stage, where every service gets it too, and the ledger still reads each request taken.

Acceptance:

- `rg -n 'skein_io::(resume|up|fire|down)\(' skein/tests/io/src/world.rs skein/tests/io/src/output.rs skein/tests/io/src/joined.rs` finds nothing (8 at base), and `rg -n 'fn within|fn marks|fn mark' skein/tests/io/src/world.rs skein/tests/io/src/output.rs` finds nothing.
- The digest of `temper/docs/plan/debt-19/notes/contracts.md`, 2 is the same at your base and at your tip. **By reading it should be** (below). If it isn't, stop and ask, naming the world and the seed.
- `heavy cargo nextest run -p skein-io -p skein-io-world` passes, all assertions unchanged, and the tests of the stage's other readers (step 5), since a new assert changes what a shared item may do.
- By hand: lower `MAX_OUT_UP` to 1 (a tests/io test goes red on the stage's assert); make `Ledger::taken` skip the first request (a ledger test goes red, on a `Bind` with no owner or a demand out of turn). Restore each.
- `rg -n 'debug_assert' skein/crates/skein-io/src/stage.rs` finds nothing: `debug_assert!` is out of step code.
- Production rises by at most 20 lines, in `stage.rs` alone (estimate +14). **This item can't end with no more production code than it started,** and the cut says why (`temper/docs/plan/debt-19/plan.md`). Test lines fall. The item is deleted, and `skein/docs/design/io.status.md`'s **Not yet** entries for 2 and 8 go.

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `temper/docs/plan/debt-19/notes/contracts.md`, 2: binding
- `skein/docs/development/debt/D032-world-harness-copies.md`
- `skein/docs/design/io.md`, 2 and 8: written, and binding
- `skein/docs/foundation/programming-model.md`, 10.2 (`assert!`, the invariant relied on) and 10.3

## Owns and may change

- **Owns:** the files in `files`.
- **The stage's assert:**

  ```rust
  /// What an entry point submitted is within what it declared, which `room` relies on.
  fn within(submissions: &Queue<Submit>, before: u32, max: MaxOut) {
      let submitted = submissions.len().checked_sub(before).expect("an entry point only adds submissions");
      assert!(submitted <= max.submissions, "an entry point submits no more than it declares");
  }
  ```

  Called after each of the stage's four entry calls. It fires only if io is wrong. `up`'s and `down`'s signatures don't change: **no witness, no trait, no feature**. `MAX_OUT_*` are exempt in `scripts/old_forms.py`, so the count holds.
- **The ledger reads what the stage took, from a description.** `Request` isn't `Clone`, `stage::down` pops, and `Ledger::request` only reads it. So: `Ledger::asked(&Request)` records, by reference, a small private description of each request as the owner queues it (what `request`'s match reads: the variant, `owner`, `socket`, `stream`, `entity`, a demand's `read` and `room`), and `Ledger::taken(count)` runs today's checks over the first `count` descriptions. In `iterate`: `let before = requests.len(); stage::down(..); ledger.taken(before - requests.len());`. Nothing runs between the stage and `taken`, and `down` tells no events, so the ledger's order with `event` holds. Recording at queueing would be unsound without this split: a held-back `Close` sets `closing`, and later legitimate `Bytes` would trip it.
- **The owner's queue:** `Owner`'s `&mut VecDeque<Request>` becomes `&mut Queue<Request>` (nine `push_back`s), with a capacity no scenario reaches, so the stage takes it. `held_back` and the accept batch's assert stay, unchanged, in `iterate`.
- **Why the digest should hold.** The stage differs from the hand loops in three ways: it returns before firing while completions or ready work remain, and it bounds resumes at `sockets + refusals` and fires at `2 * sockets`. By reading, none is reachable here: `room` is short only past 62 of 64 submissions, and the largest world holds at most 52 (`8 * sockets + 4` flights, and `sim.submit` drains the queue each turn); `mark` writes to `next`, and only `reclaim` promotes it, so a stage can't resume more than the bound; a fire that re-arms lands strictly later. This is a reading, not a proof: the digest decides.
- **`suite/stage.rs` doesn't change;** nor do `append.rs` and `usage.rs`, which aren't in **Paid when**.
- **`skein/tests/io/src/lib.rs`'s doc** says what's true: the stage holds each call to its `MAX_OUT_*`.
- **`Ledger::request`'s words stay;** its asserts' messages are read when they fire.

## Steps

1. The probe at your base.
2. `stage.rs`'s assert.
3. `Ledger::asked` and `taken`; the owner's queue; `Proc::iterate` on the stage. `Driver::iterate` on the stage.
4. The probe at your tip; `diff`; delete the probe. Break the rules by hand, as the acceptance says.
5. Delete the item and the status entries. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-io -p skein-io-world --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-io -p skein-io-world -p skein-sim -p skein-playground -p skein-fake-peers -p smith-agent-shell -p smith-local-shell -p temper-service`. The citations check. Commit and push.

## Report

The common note's items, the digests' line count and that they're equal, what you broke and what failed, and the production and test lines apart.

## Answers

2026-11-05. **May this task remove the paid D032 paragraph and status gap from `skein/docs/design/testing.md` and `testing.status.md`?** Yes, and it must: the request has each part update "any citation that breaks", and `tools/citations/citations.py --all` can't read 0 otherwise. Both files are now in `files`, for these two removals only:

- `testing.md`, 9: delete the whole bullet **tests/io's worlds on the stage**. The question is settled and `skein/docs/design/io.md`, 2 and 8 already say the answer. Section 9 keeps its number and its other bullets.
- `testing.status.md`: delete the one sentence from "6: `tests/io`'s socket, output and either worlds" to its D032 citation. The entry's earlier text, ending "no reason is recorded.", stays.

Change nothing else in either file, and leave the rest of the pushed branch as it is. `skein/scripts/old-forms.txt` is in `files` too: the run's lowered count stands, as `common.md` requires.

2026-11-05. **The by-hand `MAX_OUT_UP` line (the run's report: the worlds' tests pass at 1; only a deleted probe reached the assert).** That probe doesn't meet the acceptance, which asks for a tests/io test: nothing kept in the tree reaches the assert. Keep the probe as a focused test in `skein/tests/io/tests/suite/stage.rs`, which this answer lets change for that one addition (one `up` with a full intake, a short send completion and a pending read, so it continues the send and re-arms the receive). It passes at 2. Rerun the break and report that it goes red on the stage's assert. Compact it; if it stops test lines falling, report the measured size and go on: production's cap is untouched. Rerun the task's nextest and citations checks.
