+++
id = "ctl-told"
part = "ctl-told"
kind = "feature"
needs = ["link-told"]
files = [
  "temper/crates/temper-client-shell-cli/src/host.rs",
  "temper/crates/temper-client-shell-cli/src/tests.rs",
  "temper/crates/temper-client-shell-cli/src/tests/busy_watches.rs",
  "temper/crates/temper-client-shell-cli/src/tests/fixtures.rs",
  "temper/docs/help/ctl.md",
  "temper/docs/design/client/tools.status.md",
]
effort = "medium"
minutes = 40
+++

# `temper ctl` says the engine's watches are all in use

## Goal

The request's item 3. A `temper ctl` whose watch stays refused Busy gives up at its 30 s deadline with `the engine at ADDRESS did not answer within 30 s`, which sends the person looking in the wrong place. At that deadline it names the cause, beside the streams-full and busy-sign-in lines in `host.rs`. The 30 s stays.

Acceptance:

- With a watch refused Busy at the deadline, `temper ctl` exits 1 with exactly this on stderr, and nothing on stdout:

  ```text
  temper ctl: all the engine's watches are in use, by open pages and waits; close one, or wait and call again
  ```
- It fires one tick after the deadline and not before, within the diagnostic bound of 240 bytes, as `streams_full_exits_one_at_the_link_deadline` holds its line.
- A watch that opens before the deadline gives the call its answer, exit 0, with no such line.
- The busy sign-in's and the full streams' lines still come first. A call with no watch refused keeps its deadline line.

## Inputs

- `temper/docs/design/client/tools.md`, 6.3, "The engine's watches all in use say so".
- `jig/docs/design/client/domain/client.md`, 8.2: what `watches_full` means.
- `temper/docs/plan/watch-room/notes/contracts.md`, "`temper ctl`'s line".
- `temper/docs/plan/watch-room/notes/common.md`.

## Owns and may change

Owns the files in the header. In `host.rs`, your hunk is one `if` in `link_progress`, between the full streams' and the deadline's lines. Nothing else in that file changes.

Reader files: none. Nothing else reads this line, and `docs/help/ctl.md` is read by the docs' tests in step 5.

Other drafts, and the expected catch-up: `ctl-waits/deadline-words` and `ctl-waits/kept-answer` change `host.rs`, `src/tests.rs`, `src/tests/fixtures.rs` and `docs/help/ctl.md`, and `ctl-words/key-words` changes `docs/help/ctl.md` and `tools.status.md`. They hold the deadline's other words. Your tests go in their own file for that reason.

Departures from your text, for this task alone:

- Delete `temper/docs/design/client/tools.status.md`'s `Not yet` line that opens `6.3: a call at its deadline with a watch refused busy`.
- Change the one bullet in `temper/docs/help/ctl.md` named in step 4.

Options, each pre-approved:

- **If your base already holds `ctl-waits`' `waited::waited`,** your `if` goes after the full streams' and before its `let line`, and you restate none of its words. Otherwise your `if` goes before the `the engine at ADDRESS did not answer` line. A merge between the two keeps the order: refused, sign-in busy, streams full, watches full, then the rest.
- **The fixture.** Default: a field on the fixture's frame beside `streams_full`, `busy_watches: bool`, and `Engine::busy_watches(words)` beside `full_streams`. At `/jig/v1/watch`, in `tests/fixtures.rs`, the frame answers a refusal of `Busy` instead of `w::OpenAnswer::Admitted`. Build the refusal as `jig-protocol-engine-clients/src/routes.rs`'s `open_answer` does. If the answer's route doesn't reach `watches_full` in this fixture, a `Stream` end of Busy on `me` is yours, and say so.
- **Which watches.** `me` is the case on record. A call past readiness whose own watch is refused has a live link and the same fact. Cover it with `task show 41 --until changed`, if the fixture reaches it in a few lines. If not, list it under `## Left`.

## Steps

1. In `src/tests/busy_watches.rs`, with one `mod` line in `src/tests.rs` beside `mod keys;`, write the failing tests:
   - `a_call_at_its_deadline_names_the_engines_watches`: for `me` and for `task show 41`, using `DEADLINE_TICK`'s shape from `streams_full_exits_one_at_the_link_deadline`. Before the deadline there's no exit and no stderr. At it, exit 1 and the exact line.
   - `a_watch_room_that_frees_before_the_deadline_answers`: as `streams_full_retries_and_succeeds_when_a_stream_frees_before_the_deadline`. The room frees, the backoff passes, exit 0, no stderr, and `watches_full` is false.
2. Add the `if` in `host.rs`:

   ```rust
   if self.root.client().link().watches_full {
       self.fail(b"all the engine's watches are in use, by open pages and waits; close one, or wait and call again");
       return;
   }
   ```
3. Delete the status line.
4. In `temper/docs/help/ctl.md`, change the bullet that opens `The link's deadline is 30 seconds` so it adds that a call whose engine has all its watches in use, by open pages and waits, says so: close one, or wait and call again. Keep it short. `ctl-waits` changes this file, so touch that bullet alone.
5. Break the rule once by hand: remove the `if`, see the first test fail on the line, and restore. Then `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, `heavy cargo clippy -p temper-client-shell-cli --all-targets -- -D warnings`, `heavy cargo nextest run -p temper-client-shell-cli -p temper-docs`, `python3 -I -B tools/citations/citations.py --base origin/main`. Commit and push.

## Report

The implementer's report, plus: which base you found (with or without `waited::waited`), which fixture you took, and whether `--until` is covered.
