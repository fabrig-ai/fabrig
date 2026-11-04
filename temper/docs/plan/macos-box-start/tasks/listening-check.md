+++
id = "listening-check"
part = "listeners"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-shell/src/listener_calls.rs",
  "skein/crates/skein-shell/src/listener_tests.rs",
  "skein/crates/skein-shell/src/listeners.rs",
  "skein/crates/skein-shell/src/lib.rs",
  "temper/crates/temper/src/lib.rs",
  "skein/docs/design/shell.status.md",
]
effort = "high"
minutes = 45
+++

# A socket on macOS that can't say it listens passes, and a failed call names itself

## Goal

`temper box` on macOS stops at `temper box: LISTEN_FDS: fd 4: errno 42`. Errno 42 is Darwin's
`ENOPROTOOPT`, from `getsockopt(SOL_SOCKET, SO_ACCEPTCONN)` in `inspect`
(`skein/crates/skein-shell/src/listener_calls.rs`): XNU's `sogetoptlock` has no case for that
option, so its `default:` answers `ENOPROTOOPT` for every socket, however it was made. Fd 4 is the
first of the box's held listeners, a plain listening TCP socket. `listener_at` landed on 2026-10-24,
after the last macOS run the repository records as green (2026-10-23), so that green says nothing
about it.

Take that answer as listening, on macOS alone, and make a failed check name its call, so that the
next red says more. Linux's behaviour does not change.

Acceptance:

- `listening` (below) passes `ENOPROTOOPT` only when told the system has no such option; `inspect`
  tells it so on macOS alone, and a Linux test holds both answers.
- A failed check is `ListenersRefusal::Check` with its `ListenerCheck`; a Linux test holds each of
  the four, and `EBADF` as `NotOpen` and `EAFNOSUPPORT` as `NotIp` from each of the four.
- `listener_tests.rs` says what macOS answers where the two systems differ (below).
- skein-shell's lib tests and temper's `--test suite` binary's startup tests are green on Linux, the
  new tests bite, and `skein-shell` passes Darwin's clippy (`notes/common.md`).
- The 6.6 sentence you built is gone from `skein/docs/design/shell.status.md`. That is this task's
  one departure from "Not yours: any `docs/design/`": that sentence alone, nothing else there.

## Inputs

- `skein/docs/design/shell.md`, 6.6: "Refused", its two new bullets, and `listener_at`.
- `temper/docs/plan/macos-box-start/notes/common.md`
- `temper/docs/plan/macos-box-start/notes/contracts.md`: the types, as binding.

## Owns and may change

- Owns: `skein/crates/skein-shell/src/listener_calls.rs`, `listener_tests.rs`, `listeners.rs`, and
  the one export line in `lib.rs`.
- May change: `temper/crates/temper/src/lib.rs`, in `listeners_refusal` only: the one arm the new
  variant forces, `Why::Check { fd, check, errno } => format!("LISTEN_FDS: fd {}: {}: errno {errno}",
  fd.raw(), check.name())`. Task `held-refusal` reshapes that function; leave it otherwise as it is.
- May change: `skein/docs/design/shell.status.md`, as in Goal.
- If `Kernel`'s doc comment or a test still says it holds a check's failure, change it to the
  close-on-exec update alone. If any other crate turns out to match the enum, add the same arm there
  and say so in the report.

## Steps

In `listener_calls.rs`, `inspect`'s error carries its step, and one function decides the refusal,
so a Linux test reaches it:

```rust
pub(crate) fn listener_at(fd: Fd) -> Result<SocketAddr, ListenersRefusal> {
    inspect(fd).map_err(|(check, errno)| refused(fd, check, errno))?
}

fn refused(fd: Fd, check: ListenerCheck, errno: i32) -> ListenersRefusal;
// EBADF -> NotOpen, EAFNOSUPPORT -> NotIp, else Check { fd, check, errno }

fn inspect(fd: Fd) -> Result<Result<SocketAddr, ListenersRefusal>, (ListenerCheck, i32)>;

/// XNU's getsockopt has no case for SO_ACCEPTCONN (bsd/kern/uipc_socket.c,
/// sogetoptlock), so it answers ENOPROTOOPT for every socket; there a stream
/// socket passes (shell.md, 6.6).
const NO_ACCEPTCONN: bool = cfg!(target_os = "macos");

fn listening(answer: Result<i32, i32>, no_option: bool) -> Result<bool, i32>;
// Ok(0) -> Ok(false); Ok(_) -> Ok(true);
// Err(ENOPROTOOPT) with no_option -> Ok(true); any other Err(errno) -> Err(errno)
```

`inspect` calls `listening(option(fd, libc::SO_ACCEPTCONN), NO_ACCEPTCONN)`. `SO_TYPE` stays a
refusal on every errno. `listeners.rs` holds `ListenerCheck`, `name` and the new variant, and
`inherited_listeners` keeps `Kernel` for `close_on_exec`.

Tests, in `listener_tests.rs`:

- `listening_answers`: `Ok(0)`, a nonzero value, `Err(EINVAL)` with `no_option` true and false, and
  `Err(ENOPROTOOPT)` with `no_option` true (`Ok(true)`) and false (`Err(ENOPROTOOPT)`). Add
  `assert!(!NO_ACCEPTCONN)` under `cfg(target_os = "linux")`, so that Linux keeps its check.
- `refused_names_its_call`: over the four `ListenerCheck`s, an errno such as `ENOTSOCK` gives `Check`,
  `EBADF` gives `NotOpen` and `EAFNOSUPPORT` gives `NotIp`; and each `name()` as in the contract.
- `inherited_bound_not_listening`: `NotListening` where the system can say so, and on macOS `Ok` with
  the one listener at fd 3 and its bound address, chosen by `NO_ACCEPTCONN`. It stays one test.
- `open_listener_port_zero`: `SO_REUSEADDR` reads as nonzero, not `Ok(1)`: XNU answers the option's
  flag (`optval = so->so_options & sopt->sopt_name`), 4. Leave the rest of that test as it is.

Then the checks your implementer text names and Darwin's clippy, commit and push.

## Report

As your implementer text says. Name what no Linux run reaches (the `NO_ACCEPTCONN` constant's value
on macOS) and any Darwin check you could not run.
