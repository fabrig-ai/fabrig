+++
id = "box-readers-early"
part = "box-readers"
kind = "migration"
needs = ["box-readers-log"]
files = ["temper/crates/temper/tests/support/boxed.rs", "temper/crates/temper/tests/support/ctl.rs", "temper/crates/temper/tests/support/person.rs", "temper/crates/temper/tests/process.rs", "temper/crates/temper/tests/suite/startup.rs"]
effort = "high"
minutes = 60
+++

# A harness no longer takes the first link for serve's being up

## Goal

The first link now prints before serve starts. Each harness that used the link to mean "serve is up" waits for that fact itself, with one helper, so that every existing story keeps today's timing and its proof. No bound's value changes: a bound starts its clock where the fact now is.

Paths `tests/...` here are under `temper/crates/temper/`.

Acceptance:

- The helper, in `tests/support/boxed.rs`:

  ```rust
  /// Serve has listened on the clients' listener this run: the moment the first link used to print (protocol/setup.md, 7.2).
  pub fn serve_listening(data: &Path) -> bool;   // serve.log holds "listening clients "
  ```

- `tests/support/boxed.rs`, about lines 64-70 and 120, and the `Judge`'s phases: `Phase::Linked` is reached once the link is read **and** `serve_listening`. `Plan::Stop`'s signal and `forgejo_at_link` come after that.
- `tests/process.rs`, about lines 388, 417 and 437, hold again by that gate, unchanged.
- `tests/process.rs`, about line 1980, the no-DNS test: it waits for `serve_listening`, since its words "must reach its printed link" no longer prove it.
- `tests/suite/startup.rs`, about lines 498-501: it shows serve's start through the gate: the test's assertion on the link stays, and `setup.md`, 8.2's "reaches `temper serve`'s listening" holds of it.
- `tests/support/boxed.rs`, about line 668, each `on_link` callback, read one by one: it runs after the gate unless it is about the link alone.
- `tests/support/ctl.rs`, about lines 457 and 892: a `sign-in -` starts after the gate, as it did.
- `tests/support/person.rs`, about lines 534 and 544: `START_WITHIN` keeps its value, and starts once serve listens. If `person.rs` has no data directory to hand, pass one in; if a caller has none, say so in the report and leave that caller.
- With `heavy`, `-p temper --test suite` passes, and each story in `tests/process.rs` that used `Plan::Stop` passes run alone under `process-temper`.

## Inputs

- `temper/docs/design/protocol/setup.md`, 7.2, *The sign-in block*.
- `temper/docs/plan/box-link-first/notes/contracts.md`, section *What a reader of the box's output may rely on*.
- `temper/docs/plan/box-link-first/notes/common.md`.

## Owns and may change

Owns the files in the header.

Read, and change only as follows:

- `temper/tests/client-browser/src/lib.rs`, about lines 163 and 284, `src/public.rs`, about line 10, and `temper/tools/journeys/src/again.rs`, about lines 36-44: Chromium goes to the link at once and is held. Its stories' bounds now cover serve's start. Leave them if they pass; you need not run them. If one's bound clearly cannot hold, gate it as above, a call site only, and say so.
- `tests/support/ctl.rs`, about lines 796-803: stdout holds no blank line (contracts), so `printed_link` needs nothing. Check that, and leave it.

Options, each pre-approved:

- `tests/support/ctl.rs`'s steps have no data directory: add a step that waits for it, or pass it in, whichever is one call site.
- A story that truly means to catch the link early (none today, `box-link-stories` adds them) skips the gate with a named `Plan` or argument.
- `serve.log`'s line is from `temper/crates/temper/src/host.rs`, `note_line`: `listening clients ADDRESS`. If the file is absent, it is false. Once the link is read, the last run's logs have moved, so it cannot be stale.

**A change of meaning runs its readers' tests:** `boxed.rs` changes what `Linked` means, so run every test of `--test suite` that uses `support::boxed`.

## Steps

Find the rest of what `on_link` and `Linked` drive with `rg`, then read their assertions. Run each moved test alone, then the suite. Then fmt, clippy on `temper`. Commit and push.

## Report

As your text says, listing each `on_link` callback and what you decided for it, and the wall time `--test suite` took before and after, as heavy reports them.
