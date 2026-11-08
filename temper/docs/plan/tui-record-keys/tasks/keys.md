+++
id = "keys"
part = "keys"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-shell-tui/src/run.rs", "temper/crates/temper-client-shell-tui/tests/suite/run.rs", "temper/docs/design/client/tui/journeys.status.md"]
effort = "high"
minutes = 45
+++

# The record build also takes Alt+m to mark and Alt+s to save

## Goal

The TUI's `record` build takes Alt+Shift+M as its mark key and Alt+Shift+S as its save key. Some terminals drop the Shift from a Meta key they don't bind, and send those keys as Alt+m and Alt+s. The recording run must also take Alt+m as the mark key and Alt+s as the save key. Like the Shift forms, they never reach the host. A run with no recording, which is what the shipped `temper` always runs, gives every key to the host as before, Alt+m and Alt+s included.

Acceptance:

- `Run::record_key` (`temper/crates/temper-client-shell-tui/src/run.rs`) takes `Key::Alt('m')` as it takes `Key::Alt('M')`, and `Key::Alt('s')` as it takes `Key::Alt('S')`. The arms' bodies don't change. No other key changes what it does.
- The run suite's recorder key test, `run::recording_keys_save_whole_and_closing_saves_again` (`temper/crates/temper-client-shell-tui/tests/suite/run.rs`), runs its walk once per row of key forms: the Shift forms, then Alt+m and Alt+s. Both rows pass. The suite's `replay` helper refuses a `key` record of any of the four keys.
- The run's unit test `run::tests::recording_keys_pass_to_the_host_without_a_root` gains the rows `Key::Alt('m')` and `Key::Alt('s')`, beside `Key::Alt('M')` and `Key::Alt('S')`, and passes. Its `Settings` have `record: None`, as `temper/crates/temper/src/tui.rs` gives them in a build without `record`.
- Each new row bites: break its rule by hand once, run its test, and see it fail for that reason (Steps, 3).
- `journeys.status.md`'s **Not yet** entry for 3.1 and 3.3 is gone (Owns and may change).
- The focused tests in Steps, 4 pass.

## Inputs

- The request: `temper/docs/plan/tui-record-keys/inputs/request.md`. Read it whole; it's short. Its `For every task of this plan` section binds this task, and stands in for a `notes/common.md`.
- `temper/docs/design/client/tui/journeys.md`, 3.1 (the two keys, and the bullet *Each key in two forms*, which this plan wrote), 3.3 (*The mark and save keys*) and 8.1 (the walk's `mark` and `end` steps keep the Shift forms).
- `temper/docs/design/client/tui/journeys.status.md`: the **Not yet** entry this task closes.
- The code: `Run::record_key` and its `mod tests` in `temper/crates/temper-client-shell-tui/src/run.rs`. In `temper/crates/temper-client-shell-tui/tests/suite/run.rs`: `World::recording`, `World::keys`, `replay` and `recording_keys_save_whole_and_closing_saves_again`.

Departures from the implementer's text, and why:

- **You edit one design status file,** `temper/docs/design/client/tui/journeys.status.md`, though `docs/design/` is otherwise not yours. The plan wrote its **Not yet** entry for exactly this change. The entry goes when the code lands, in the same branch.
- **The record build's suite runs with `--features record`.** The recorder key test is `#[cfg(feature = "record")]`. `-p temper-client-shell-tui` alone builds the crate without the feature, so that run selects nothing for it.
- **`temper` and `temper-client-world` get `heavy cargo check -p ... --tests`, not their focused tests.** No signature changes. What changes is one match in a function that exists only in the record build and in the crate's own tests. Neither package presses Alt keys, and the shipped binary gives the run `record: None` (`temper/crates/temper/src/tui.rs`). `temper-client-journeys` does read the recorder (`again` and the committed journey's replay), and its focused tests do run.

## Owns and may change

Owns:

- `temper/crates/temper-client-shell-tui/src/run.rs`: the two arms of `record_key`'s match, a doc comment on `record_key` if you add one, and the unit test's key list. Nothing else in the file.
- `temper/crates/temper-client-shell-tui/tests/suite/run.rs`: `replay`'s `matches!` line and `recording_keys_save_whole_and_closing_saves_again`. Nothing else in the file.
- `temper/docs/design/client/tui/journeys.status.md`: remove the **Not yet** bullet, with the blank line after it, and turn `Otherwise no known implementation gap.` back into `No known implementation gap.`. The rest of that line and the date (`2026-11-08`) stay as they are, so the paragraph is again the one on `main`, line for line. Change nothing else in the file.

Keep the edits to these lines. `tui-link-colours` is in flight beside this plan, and may change the run's start facts (the scheme) in both `run.rs` files and in the status file. Don't tidy lines this task doesn't own. If `cargo fmt` rewraps an owned line, that's fine.

Not yours, and why:

- `temper/docs/design/client/tui/journeys.md`: the plan has written it.
- `temper/tools/journeys/**`: `again` keeps pressing Alt+Shift+M and Alt+Shift+S (`journeys.md`, 8.1). Its key table (`tui_again/tests.rs`) doesn't change.
- `temper/docs/design/client/tui/journeys/**`: the committed journey. It holds no `key alt` record, and it isn't recorded again.
- The browser's journeys, `temper-client-shell-web`: its keys don't change.

Pre-approved options:

- **The arms' patterns:** `Key::Alt('M' | 'm')` and `Key::Alt('S' | 's')` by default. A `match` must list every variant; the `Key::Alt(_)` arm that follows stays for every other character.
- **A doc comment on `record_key`** citing `temper/docs/design/client/tui/journeys.md`, 3.1, is welcome. If you add one, keep it to a line.
- **The suite test's table:** by default, `for (mark, save) in [(b"\x1bM", b"\x1bS"), (b"\x1bm", b"\x1bs")]`. Each row gets a fresh `World::recording(false, true, false, Outcome::Success, Recording::Root)` and the same walk and assertions as now, with the row's bytes wherever the test presses `\x1bM` or `\x1bS`. That's the first press of both keys, the second save after `\t`, and nothing else. Each assertion's message names the row: `{mark:?}` is enough, or a name column if you prefer. Never share a world across rows.
- **If `cargo fmt` or clippy wants a change** in an owned item, take it.

## Steps

1. **The key match.** In `Run::record_key`, widen the mark arm to `Key::Alt('M' | 'm')` and the save arm to `Key::Alt('S' | 's')`. The bodies don't change.

2. **The test rows.**
   - `run.rs`'s `mod tests`: the loop becomes `for key in [Key::Alt('M'), Key::Alt('S'), Key::Alt('m'), Key::Alt('s')]`. Its assertion and message stay. This is the row showing that a run without a recording, the shipped binary's, gives Alt+m and Alt+s to the host.
   - `tests/suite/run.rs`:
     - `replay`'s refusal becomes `Key::Alt('M' | 'S' | 'm' | 's')`.
     - `recording_keys_save_whole_and_closing_saves_again` loops over the key-form rows (Owns and may change, *The suite test's table*). The Shift row keeps exactly today's assertions.

   Narrowest runs, each checking the count is 1:

   ```sh
   heavy cargo nextest run -p temper-client-shell-tui --lib -E 'test(=run::tests::recording_keys_pass_to_the_host_without_a_root)'
   heavy cargo nextest run -p temper-client-shell-tui --features record --test suite -E 'test(=run::recording_keys_save_whole_and_closing_saves_again)'
   ```

3. **Each new row bites.** Break each rule once by hand, run its narrow test, see it fail for the rule's reason, and restore:
   - Take `'m'` out of the mark arm. The suite test's second row must fail: no `Mark`, or `replay`'s refusal of a `key alt "m"` record.
   - Take `'s'` out of the save arm. The suite test's second row must fail at its first `recording_file()` (`save key writes recording`).
   - Make `record_key` return `true` for `Key::Alt('m')` before its `if let`. The unit test must fail on its `Alt('m')` row.

   The report names each break and the failure seen.

4. **Check.**
   - `cargo fmt`.
   - `python3 -I -B temper/scripts/old_forms.py`. No count rises.
   - `heavy cargo clippy -p temper-client-shell-tui --all-targets -- -D warnings`, then again with `--features record`.
   - `heavy cargo nextest run -p temper-client-shell-tui`, then `heavy cargo nextest run -p temper-client-shell-tui --features record`.
   - `heavy cargo nextest run -p temper-client-journeys`. It enables `record`, replays the committed TUI journey and checks the walk.
   - `heavy cargo check -p temper --tests` and `heavy cargo check -p temper-client-world --tests`.
   - `python3 -I -B tools/citations/citations.py --base origin/main`.

5. **Commit and push.** One commit: `tui-record-keys: the record build takes Alt+m and Alt+s too`. Its body says why: some terminals drop the Shift from Alt+Shift+M and Alt+Shift+S, and the shipped build is unchanged.

## Report

Follow the implementer's text. Also give:

- each break of Steps, 3 and the failure it gave;
- the suite test's time per row, from nextest's output;
- every file you changed outside `files`, and why (none is expected).
