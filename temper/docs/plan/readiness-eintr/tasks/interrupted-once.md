+++
id = "interrupted-once"
part = "interrupted-calls"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-shell/src/readiness.rs",
  "skein/crates/skein-shell/src/readiness/interrupted.rs",
  "skein/crates/skein-shell/src/linux.rs",
  "skein/crates/skein-shell/src/linux/files.rs",
  "skein/crates/skein-shell/src/darwin.rs",
  "skein/crates/skein-shell/src/darwin/files.rs",
  "skein/crates/skein-shell/src/listeners.rs",
  "skein/docs/design/shell.status.md",
  "skein/docs/design/STATUS.md",
]
effort = "high"
minutes = 50
+++

# The readiness backend completes an interrupted `Socket`, `Accept`, `Connect` or `Open` `Other(EINTR)`

## Goal

Under the readiness backend, an interrupted `Socket`, `Accept`, `Connect` or `Open` completes
`Other(EINTR)`, as the ring's does, and is never made again: not by `attempt`'s loop and not by
the system's call underneath it. A `Connect` then never calls `connect` twice on its socket.

The retry lives in two places today. `attempt` (`readiness.rs`, ~line 276) makes every
operation but `Open` and `Close` again on `EINTR`; and each system's call wraps its syscall in
`retry` (`linux.rs`: `socket` ~67, `accept` ~136, `connect` ~157; `linux/files.rs`: `open` ~20;
`darwin.rs`: `socket` ~92, `connect` ~169, `accept` ~186; `darwin/files.rs`: `open` ~32 through
`open_flags` ~53). Dropping only `attempt`'s would change nothing: the call underneath absorbs
the `EINTR` first. Both go, for these four alone.

Acceptance:

- `system::socket`, `system::accept`, `system::connect` and `system::files::open` make their
  syscall once on both systems: an `EINTR` is returned as the error number, never retried.
- `attempt` never makes a `Socket`, `Accept`, `Connect`, `Open` or `Close` again on `EINTR`; it
  still makes again every other operation it makes again today (the eight transfers 7.4
  reissues, and the operations section 10 leaves open), and `Open`'s `EAGAIN` rule is unchanged.
- The tests of `readiness/interrupted.rs` below pass on Linux; skein-shell's other tests, the
  readiness backend's (`skein-readiness-tests`) and the conformance suite
  (`skein-conformance-real`) still pass; clippy is clean for Linux and for
  `aarch64-apple-darwin` (`notes/common.md`).
- The 7.4 entry of `shell.status.md` and its clause in `STATUS.md` are gone.

## Inputs

- `temper/docs/plan/readiness-eintr/notes/common.md`.
- `skein/docs/design/shell.md`, 7.4: the bullets *Interrupted transfers are reissued* and *An
  interrupted `Socket`, `Accept`, `Connect` or `Open` completes `Other(EINTR)`* (this cut wrote
  the second); 7.3 for `attempt`'s place in a record's path; 7.5 for each system's calls.
- `skein/docs/design/shell.md`, 10: *Every other interrupted operation* (open: leave those
  operations as they are).
- The ring's seam tests, for the shape of a seam test: `linux/ring/tests/write_retries.rs`
  (read only).

## Owns and may change

- **`readiness.rs`, the seam.** `attempt` makes its calls through a parameter, so a test can
  answer a call as the kernel would:

  ```rust
  /// True once complete; false when the record still needs readiness.
  fn attempt(&mut self, index: u32, resumed: bool, waitable: bool) -> bool {
      self.attempt_with(index, resumed, waitable, perform)
  }

  /// `attempt`, each call made through `call` (`perform` outside the tests).
  fn attempt_with(
      &mut self,
      index: u32,
      resumed: bool,
      waitable: bool,
      mut call: impl FnMut(&mut Op, bool, bool, &mut system::Listings) -> Result<Done, Error>,
  ) -> bool
  ```

  `attempt_with` holds today's body of `attempt`, `perform(kind, resumed, nonblocking, &mut
  self.listing)` becoming `call(kind, resumed, nonblocking, &mut self.listing)`. Every caller
  of `attempt` stays as it is.
- **`readiness.rs`, the rule.** The `EINTR` test in `attempt_with`'s loop makes the call again
  for every operation but `Socket`, `Accept`, `Connect`, `Open` and `Close`. Write it as an
  exhaustive `match` over `Op` in a small function (no `_` arm), so a new operation must choose;
  `Op::Cancel` never reaches it (`unreachable!`, as `perform_call` has). Its doc comment cites
  `shell.md`, 7.4 and 10. Replace the comment above the test (~lines 283-286, "Open retries at
  openat2 ...") with one that says what is true now: these five complete with their error, the
  transfers are reissued (7.4), and the others' rule is open (section 10). Nothing else of
  `attempt`'s body changes.
- **Each system's four calls**, Linux and Darwin: `socket`, `accept`, `connect` and
  `files::open` make their syscall once, with no `retry`; the code inside the closure stays as
  it is (Darwin's `configure` and its close on a failed `decode`, Linux's `accept4` flags). Each
  gains a one-line doc comment saying an `EINTR` is returned, not retried (`shell.md`, 7.4).
  On Darwin, `open_flags` keeps `retry` for its other two readers (`darwin/process.rs`,
  `directory`, ~line 88; `darwin/files.rs` ~204): split it so `open` makes one `openat` and
  `open_flags` retries that same one-call function, e.g.

  ```rust
  /// One `openat` beneath `root`: an `EINTR` is returned, not retried (shell.md, 7.4).
  fn open_once(root: Fd, path: &[u8], flags: i32, mode: u32) -> Result<i32, i32>
  pub(super) fn open_flags(root: Fd, path: &[u8], flags: i32, mode: u32) -> Result<i32, i32> {
      retry(|| open_once(root, path, flags, mode))
  }
  ```

  The names inside are yours.
- **`listeners.rs`, `open_listener` (~line 85):** a held listener opened at startup is no
  record, and keeps its behaviour: its `system::socket(family)` becomes
  `system::retry(|| system::socket(family))`. Nothing else there changes.
- **Readers found later.** If `rg -n 'system::(socket|accept|connect)\b|files::open\b|open_flags\('
  skein/crates/skein-shell/src` finds a reader outside the readiness backend that this spec does
  not name, wrap its call in `system::retry` the same way and name it in the report. The tests'
  helpers that `expect` a socket (`readiness.rs`'s `listener`, `listener_tests.rs`) stay as they
  are.
- **The tests**, a new module `readiness/interrupted.rs`, declared beside `guards` at the end of
  `readiness.rs` (`#[cfg(test)] mod interrupted;`). Not Linux's alone: it runs on both systems
  and names no Linux call. Sketch:

  ```rust
  //! Interrupted calls at the readiness backend's seam (shell.md, 7.4), each
  //! answered as the kernel would, without interrupting a real call.

  /// Puts `kind` in a slot as `start` does (a free slot, its token, the
  /// record), attempts it through `answer`, which receives the call's number
  /// from 1, then reaps it; returns the calls made and the completion's result.
  fn attempted(
      kind: Op,
      resumed: bool,
      answer: impl FnMut(u32) -> Result<Done, Error>,
  ) -> (u32, Result<Done, Error>)
  ```

  It opens `Readiness::open(Config { operations: 1 })` as the module's tests do, calls
  `attempt_with(index, resumed, true, ..)` and asserts it returned `true` (complete), then
  asserts no waiter was added (`kernel.waiters.is_empty()`) and, after `reap`, nothing is in
  flight. Descriptors are a constant (`Fd::new(123)`): the fake makes no call, so none is real.

  - `an_interrupted_socket_accept_connect_or_open_completes_eintr_after_one_call`: a table of
    `Op::Socket { family: Family::Ipv4 }`, `Op::Accept`, `Op::Connect` at submit (`resumed`
    false) and on its readiness (`resumed` true), and `Op::Open` (`OpenHow::Read`, a short
    path), each answered `Err(Error::Other(libc::EINTR))` at every call: each made one call and
    completed `Err(Error::Other(libc::EINTR))`.
  - `an_interrupted_close_completes_eintr_after_one_call`: the same for `Op::Close`.
  - `an_interrupted_transfer_is_made_again_until_it_completes`: each of the eight transfers 7.4
    names (`PipeWrite`, `Write`, `Append`, `Send`, `PipeRead`, `Read`, `Recv`, `ReadSignal`),
    answered `EINTR` on calls 1 and 2 and `Ok(Done::Count(1))` on call 3: three calls, and
    `Ok(Done::Count(1))`.

  Each assertion names the operation it failed on (the table's `Op` in the message).
- **Status**, after the code (`notes/common.md`):
  - `skein/docs/design/shell.status.md`: in **Not yet**, remove the sentence that begins `7.4:
    the readiness backend makes an interrupted` through `answers \`EALREADY\`.`; the 6, 6.4 and
    9 items stay word for word, re-wrapped; the date line is today's.
  - `skein/docs/design/STATUS.md`: remove the clause `, and the readiness backend still makes an
    interrupted \`Socket\`, \`Accept\`, \`Connect\` or \`Open\` again (\`shell.md\`)`, so the
    sentence reads ``Single items lag in `lib.md`, `examples.md` and `ui.md`.``; re-wrap the
    paragraph's lines as they are wrapped; the date line is today's.

Pre-approved: if clippy asks for a reshaped closure or a `#[expect]` on the seam (for example
`impl FnMut` in a private method), take the scoped `#[expect(..., reason = "...")]` and say so.
If `attempt_with`'s borrow of `self.listing` beside the slot's `kind` will not compile as
written, take the slot's fields apart as `attempt` does today; the seam's signature stays.

## Steps

1. The seam in `readiness.rs`, with no change of behaviour; then the tests above. The first
   fails today for `Socket`, `Accept` and `Connect`, which `attempt` makes again; `Open` and
   `Close` pass already at this level. Since the fake answers `EINTR` every time, a call made
   again would loop for good: give `answer` a bound (panic past 3 calls, naming the operation),
   so a red is a clear failure, not a hang.
2. The rule in `attempt_with`; the four calls on Linux, then on Darwin; `listeners.rs`.
3. Narrow: `heavy cargo nextest run -p skein-shell --lib -E 'test(/^readiness::/)'` (check the
   count: the three new tests and the module's others run).
4. The change of meaning (the four calls now return `EINTR`) reaches only skein-shell, whose
   items are crate-private: `heavy cargo nextest run -p skein-shell`, then `-p
   skein-readiness-tests`, then `-p skein-conformance-real`. No other dependent: this departs
   from the implementer's text ("their direct dependents"), since no public item changes and no
   test provokes an `EINTR`.
5. `cargo fmt`; `cd skein && python3 -I -B scripts/old_forms.py`; `heavy cargo clippy -p
   skein-shell --all-targets -- -D warnings`; the Darwin clippy of `notes/common.md`.
6. The status edits; the citations check; commit (code and tests, then status) and push.

## Report

As the implementer's text says, and: the shape of the rule's function and of Darwin's split;
every reader you wrapped in `system::retry`; the counts of step 3 and 4's runs; the Darwin
clippy's result or why it could not run.
