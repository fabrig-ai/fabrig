# macos-box-start: the plan

Follow-up to `macos-cheap`, under `temper/docs/design/draft/macos.md`'s rule: keep it cheap. The bar is
that draft's section 6, step 5: the story green on the runner. It lands nothing until the trunk has
run there.

## The cause

`temper box: LISTEN_FDS: fd 4: errno 42` is `getsockopt(SOL_SOCKET, SO_ACCEPTCONN)` answering
`ENOPROTOOPT`, in `inspect` (`skein/crates/skein-shell/src/listener_calls.rs`).

- **Why that call.** Published XNU (`bsd/kern/uipc_socket.c`, `sogetoptlock`) has no `case SO_ACCEPTCONN`,
  so the switch's `default:` gives `ENOPROTOOPT`, for every socket however it was made. `SO_TYPE` has its
  case. `fstat` and `getsockname` don't answer `ENOPROTOOPT`. Darwin's `getsockopt(2)` lists no
  `SO_ACCEPTCONN` either; `sys/socket.h` has it only as a bit of `so_options`.
- **What fd 4 is.** The first of `held`: `temper init`'s reservation for the clients' port, made by
  `Io::port` (`temper/crates/temper/src/init.rs`) on a fresh box. A plain listening TCP socket:
  nothing about it differs. Fd 3 was the kernel's kqueue, opened first and closed since.
- **The request's counter-evidence doesn't hold.** It says skein's listener tests ran green on macos-26
  on 10-23. `listener_at`, `SO_ACCEPTCONN` and those tests came in `7a255a4b5e`, on 2026-10-24. The last
  green the repository records is run 40019066645, on `19cbc77b12` of 2026-10-23. The workflow still ran
  on each push to skein's `main` until 2026-10-28, so later runs may have gone red; none is recorded.

**How sure:** high, not certain. It rests on reading published XNU, whose `main` may lag macos-26's
kernel, and nothing here has run on a Mac. The run will tell at once: if the belief is wrong, the
same stop now reads `held listener: fd 4: getsockopt ...`, naming the call.

## The parts and tasks

One part, `listeners`, one slice, two tasks. The critical path is both, in order:

1. `listening-check` (skein): on macOS `ENOPROTOOPT` for `SO_ACCEPTCONN` passes as listening. A failed
   check names its call. The two test answers that differ on Darwin are written down. **This one fixes
   the red.**
2. `held-refusal` (temper, needs 1): a held descriptor reads `held listener: ...`, not `LISTEN_FDS`.

Two tasks because a branch stays in its tree: the first carries only the one temper arm its new
variant forces.

## What was decided

- **Accept the answer, don't `cfg` the call out.** The check still asks, so a kernel that gains the
  option gets the real check. A bound socket that never listened now passes on macOS and fails at its
  first accept. Only a parent's mistake reaches that, so the degraded check is fine by the user's rule.
  `shell.md`, section 10 keeps the question of reading it another way.
- **`skein-shell`'s public change is additive:** a new variant `ListenersRefusal::Check { fd, check, errno
  }` and a new enum `ListenerCheck`. No variant changes its fields. `Kernel` keeps its fields and now
  means the close-on-exec update alone. hearth, smith and jig name neither the enum nor its functions;
  its one match outside the crate is temper's `listeners_refusal`, which takes one more arm.
- **Darwin's path is reached on Linux:** `listening` takes the system's answer as an argument, so a Linux
  test holds both. Only the constant's value on macOS goes unrun.
- **`open_listener_port_zero` loses its `Ok(1)`:** XNU answers `SO_REUSEADDR` with the flag, 4. The
  workflow doesn't run it yet, but step 6 would.
- **No workflow change, no new Darwin call:** the story itself proves it.

## Overlaps

No other draft's listed files are touched. `temper/crates/temper/src/lib.rs` and `box_command.rs` are in
no other draft's list here, and `skein/docs/design/shell.md` and its status neither. Nothing unavoidable.

## Proving it

`macos-box-start/integration` holds the fix once both tasks merge: dispatch `macos.yml` there. A next
failure in the story joins this plan as a further small task.
