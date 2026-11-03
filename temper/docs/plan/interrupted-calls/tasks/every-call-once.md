+++
id = "every-call-once"
part = "interrupted-calls"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-shell/src/readiness.rs",
  "skein/crates/skein-shell/src/readiness/interrupted.rs",
  "skein/crates/skein-shell/src/linux.rs",
  "skein/crates/skein-shell/src/linux/files.rs",
  "skein/crates/skein-shell/src/linux/sync.rs",
  "skein/crates/skein-shell/src/linux/process.rs",
  "skein/crates/skein-shell/src/darwin.rs",
  "skein/crates/skein-shell/src/darwin/files.rs",
  "skein/crates/skein-shell/src/darwin/process.rs",
  "skein/crates/skein-shell/src/listeners.rs",
  "skein/crates/skein-shell/src/process.rs",
  "skein/docs/design/shell.status.md",
  "skein/docs/design/STATUS.md",
]
effort = "high"
minutes = 60
+++

# Every interrupted operation but a transfer completes `Other(EINTR)`, made once

## Goal

Under the readiness backend, an interrupted `Bind`, `Listen`, `Shutdown`, `Truncate`, `Sync`,
`Stat`, `Rename`, `Remove`, `MakeDirectory`, `List`, `Lock`, `Wait`, `Spawn`, `Signal`, `Usage`
or `Window` completes `Other(EINTR)`, as the ring's does, and is never made again: not by
`attempt_with`'s loop and not by the system's call underneath it. `Socket`, `Accept`, `Connect`,
`Open` and `Close` already do (readiness-eintr); the eight transfers are still reissued.

The retry lives in two places today. `attempt_with` (`readiness.rs`, ~line 298) makes every
operation that `retry_interrupted` (~361) answers `true` for again; and each system's call
wraps its syscall in `retry`, for eleven of the sixteen (below). `List`'s retry is in the shared
part, around `listing.list` (`perform_call`, ~523). `Spawn`, `Signal`, `Usage` and `Window` have
none below `attempt_with`. Both places go, for these sixteen alone.

Acceptance:

- `retry_interrupted` answers `true` for the eight transfers alone; `attempt_with` makes no
  other operation again on `EINTR`, and `Open`'s `EAGAIN` rule is unchanged.
- Each system's call below makes its syscall once on both systems: an `EINTR` is returned as
  the error number, never retried. `List` is made once by the shared part.
- `open_listener` and the public `poll_child` still make their call again on `EINTR`.
- The test of `readiness/interrupted.rs` below passes on Linux, beside the module's three;
  skein-shell's other tests, `skein-readiness-tests`, `skein-ring-tests` and
  `skein-conformance-real` still pass; clippy is clean for Linux and for `aarch64-apple-darwin`
  (`notes/common.md`).
- The 7.4 entry of `shell.status.md` and its clause in `STATUS.md` are gone.

## Inputs

- `temper/docs/plan/interrupted-calls/notes/common.md`.
- `skein/docs/design/shell.md`, 7.4: the bullets *Interrupted transfers are reissued* and
  *Every other interrupted operation completes `Other(EINTR)`* (this cut wrote the second); 7.3
  for `attempt`'s place in a record's path; 7.5 for each system's calls; section 3, *Synchronous
  operations run at submit*, for the ring's calls that Linux shares (`Lock` among them).
- The seam and its tests as they stand: `attempt_with` in `readiness.rs` and
  `readiness/interrupted.rs` (readiness-eintr's task, `temper/docs/plan/readiness-eintr/tasks/interrupted-once.md`,
  for how it was shaped; read only).

## Owns and may change

- **`readiness.rs`, the rule.** `retry_interrupted` keeps its exhaustive `match` over `Op` (no
  `_` arm) and its `unreachable!` for `Op::Cancel`: the eight transfers (`PipeWrite`, `Write`,
  `Append`, `Send`, `PipeRead`, `Read`, `Recv`, `ReadSignal`) answer `true`, every other
  operation `false`. Its doc comment cites `shell.md`, 7.4 alone. The comment above the test in
  `attempt_with` (~lines 295-297) says what is true now, e.g. "Interrupted transfers are
  reissued; every other operation completes with its error (shell.md, 7.4)." Nothing else of
  `attempt_with`'s body changes.
- **`readiness.rs`, `List`:** in `perform_call`, `system::retry(|| listing.list(*fd, entries,
  names))` becomes `listing.list(*fd, entries, names)`. The `system::retry` around
  `setsockopt` in `Bind` and `Accept`, and in `attempt_with` after a `Socket`, stays.
- **Each system's calls**, made once, with no `retry`; the code inside the closure stays as it
  is. Each gains (or keeps) the one-line doc comment the four have: "An `EINTR` is returned, not
  retried (shell.md, 7.4)."
  - Linux, `linux.rs`: `bind` (~76), `listen` (~123), `shutdown` (~177). `linux/files.rs`:
    `truncate`, `sync`, `stat`, `rename`, `remove`, `mkdir`. `linux/sync.rs`: `lock` (~93),
    which the ring's `Lock` shares (`linux/ring.rs` ~669): the ring's then completes
    `Other(EINTR)` too, as 7.4 says; `ring.rs` itself does not change. `linux/process.rs`:
    `poll_child` (~629).
  - Darwin, `darwin.rs`: `bind` (~160), `listen` (~237), `shutdown` (~278).
    `darwin/files.rs`: `truncate`, `sync`, `rename`, `remove` (its `unlinkat`; the `fstatat`
    that inspects an `EPERM` keeps its `retry`), `mkdir`, `lock`. `darwin/process.rs`:
    `poll_child` (~379); its `ECHILD` mapping and the rest stay.
  - Darwin's `stat`: `raw_stat` has two other readers, `opened` (an `Open`'s check) and
    `prepare_output` (startup), which keep their retry. Split it so `stat` makes one `fstat`,
    e.g.

    ```rust
    /// One `fstat`: an `EINTR` is returned, not retried (shell.md, 7.4).
    fn stat_once(fd: i32) -> Result<libc::stat, i32>
    fn raw_stat(fd: i32) -> Result<libc::stat, i32> {
        retry(|| stat_once(fd))
    }
    ```

    The names inside are yours. Darwin's `open_flags`, `opened` and `open_append` stay as they
    are; so do every `retry` in `darwin/process.rs` but `poll_child`'s (`reap_pid`, the exec
    report's read and write) and every `retry` this list does not name.
- **The callers outside the kernel keep their behaviour:**
  - `listeners.rs`, `open_listener` (~83): `system::bind(fd, &addr)` becomes
    `system::retry(|| system::bind(fd, &addr))`, and `system::listen(fd, backlog)` likewise.
    Nothing else there changes.
  - `process.rs`, the public `poll_child` (~69), which the harness calls (`skein-world`'s
    `tree.rs`, `tests/world`): it makes `system::process::poll_child` again while it answers
    `Err(Error::Other(libc::EINTR))`, e.g.

    ```rust
    pub fn poll_child(pidfd: Fd, reap: bool) -> Result<Option<Exit>, Error> {
        loop {
            match system::process::poll_child(pidfd, reap) {
                Err(Error::Other(libc::EINTR)) => {}
                result => return result,
            }
        }
    }
    ```

    Its signature and doc stay; one line of doc may say that an interrupted wait is made again.
  - `signal_kept_child` and Linux's `abandon_binary` call `signal_child`, which has no retry
    today and gains none.
- **Readers found later.** If `rg -n
  'system::(bind|listen|shutdown|lock)\b|files::(truncate|sync|stat|rename|remove|mkdir)\b|poll_child\(|[^.]\block\('
  skein/crates/skein-shell/src` finds a reader outside the readiness backend and the ring that
  this spec does not name, wrap its call in `system::retry` the same way (or loop as
  `poll_child` does, for an `Error`) and name it in the report. The tests' helpers that
  `expect` a bound or listening socket (`readiness.rs`'s `listener`, `listener_tests.rs`) stay as
  they are.
- **The test**, in `readiness/interrupted.rs`, beside its three, through its `attempted`
  helper unchanged:

  - `every_other_interrupted_operation_completes_eintr_after_one_call`: a table of the sixteen,
    each answered `Err(Error::Other(libc::EINTR))` at every call: each made one call and
    completed `Err(Error::Other(libc::EINTR))`. `Op::Bind` (a loopback address), `Op::Listen`,
    `Op::Shutdown`, `Op::Truncate`, `Op::Sync`, `Op::Stat`, `Op::Rename`, `Op::Remove`,
    `Op::MakeDirectory`, `Op::List` (one `Entry` and a short `names`, or empty boxes, as `Op`
    allows), `Op::Lock`, `Op::Wait` at submit (`resumed` false) and on its readiness (`resumed`
    true), `Op::Spawn` (a `Spawn` with a short `program`, empty `args`, `env`, `dir` and
    `pipes`, `root` the constant `FD`), `Op::Signal` (`Signal::Terminate`, `Target::Child`),
    `Op::Usage` and `Op::Window`. Descriptors are the module's `FD`: the fake makes no call.
    Each assertion names the operation (and `resumed`) it failed on, as the module's do.

  Import what the table needs (`Spawn`, `Signal`, `Target`, `Entry`) from `skein_io::kernel`
  in the module. The existing three tests do not change.

  No seam exists below the systems' calls (each makes its libc call directly), and none is
  added: the once-ness there is held by the code, one call with no `retry`, and by the Darwin
  clippy.
- **Status**, after the code (`notes/common.md`):
  - `skein/docs/design/shell.status.md`: in **Not yet**, remove the sentence that begins `7.4:
    the readiness backend makes an interrupted` through `is made again too.`; the 6, 6.4 and 9
    items stay word for word, re-wrapped; the date line is today's.
  - `skein/docs/design/STATUS.md`: remove the clause `, and the readiness backend still makes
    most interrupted operations again (\`shell.md\`)`, so the sentence reads ``Single items lag
    in `lib.md`, `examples.md` and `ui.md`.``; re-wrap the paragraph's lines as they are
    wrapped; the date line is today's.

Pre-approved: if clippy asks for a reshaped loop or closure in `poll_child` or Darwin's split,
take the reshape it asks for, or a scoped `#[expect(..., reason = "...")]`, and say so. If a
function this spec names has lost its `retry` already or has moved, change what stands and say
so; if a call it names has a reader in another crate, stop and ask (none does on the trunk:
each is crate-private).

## Steps

1. The test above. It fails today for every operation in its table (each is made again until
   the helper's bound panics, naming it).
2. The rule in `retry_interrupted` and `List`'s line; the test passes.
3. Each system's calls, Linux, then Darwin; `listeners.rs` and `process.rs`.
4. Narrow: `heavy cargo nextest run -p skein-shell --lib -E 'test(/^readiness::/)'` (check the
   count: the new test and the module's others run).
5. The change of meaning (these calls now return `EINTR`) reaches skein-shell, whose items are
   crate-private, and the ring's `Lock`: `heavy cargo nextest run -p skein-shell`, then `-p
   skein-readiness-tests`, then `-p skein-ring-tests`, then `-p skein-conformance-real`. No other
   dependent: this departs from the implementer's text ("their direct dependents"), since no
   public item changes meaning and no test provokes an `EINTR`.
6. `cargo fmt`; `cd skein && python3 -I -B scripts/old_forms.py`; `heavy cargo clippy -p
   skein-shell --all-targets -- -D warnings`; the Darwin clippy of `notes/common.md`.
7. The status edits; the citations check; commit (code and tests, then status) and push.

## Report

As the implementer's text says, and: each call you made once, by file; Darwin's split; every
reader you wrapped in `system::retry` or a loop; the counts of steps 4 and 5's runs; the Darwin
clippy's result or why it could not run.
