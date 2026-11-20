# macos: status

2026-11-20. Where temper's code lags or departs from [macos.md](macos.md).

- **Not yet:**
  - 2: temper's one story on macOS has not run green
    on the runner, so `../draft/macos.md`'s acceptance stands open.
  - 3, *What both systems have*: tests and kits still run `/usr/bin/kill`
    and `/usr/bin/date`, and list their own descriptors from `/proc/self/fd`
    (`testing/temper-process-guard/src/lib.rs`, `signal`, among others), and
    the last three rows of the table's tests are not gated
    [in flight: change 10631].
  - 3, *What both systems have*: the held login's hook, wrapper and
    rung import take their lock with util-linux's `flock` and check the
    record with GNU `stat -c` (`testing/temper-box-kit/src/held_login.rs`)
    [in flight: change 10632].
  - 3, *What both systems have*: the scripted Forgejo's exec, descriptors
    and standard input are read from `/proc` alone
    (`testing/temper-box-kit/src/owned.rs`, `Programs::marker`)
    [in flight: change 10634].
