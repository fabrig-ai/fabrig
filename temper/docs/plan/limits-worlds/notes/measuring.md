# limits-worlds: how a story is profiled and written down

`valgrind --tool=callgrind` on one test by name, on the unoptimized test binary: instruction counts, which a loaded machine doesn't move, where CPU seconds swing by a third. Quote both, instructions first.

## Getting valgrind

```sh
command -v valgrind callgrind_annotate
```

An earlier run found neither on a builder. Then keep a private copy in your slot, with no root and no change to the machine:

```sh
mkdir -p target/limits-worlds/tool && cd target/limits-worlds/tool
apt-get download valgrind
dpkg -x valgrind_*.deb root
# VALGRIND_LIB is the directory that holds `callgrind-amd64-linux`; on Debian, root/usr/libexec/valgrind.
```

Call it by absolute path, with `VALGRIND_LIB` set in the command itself, since a login shell resets what you exported. `callgrind_annotate` from the same copy needs only perl. **If you can't get it this way, your first line is `Question:`, with the error:** nothing in this plan changes without a profile. Don't install anything on the machine.

## One run

Every run goes through `heavy`, one at a time. Find the binary, which also builds it:

```sh
heavy cargo nextest list -p <package> --test suite --message-format json
```

Its `rust-suites` entry holds `binary-path` and `cwd`. A test reads only `CARGO_MANIFEST_DIR` from its environment, so run it as nextest would, from `cwd`:

```sh
heavy bash -lc 'cd <cwd> && CARGO_MANIFEST_DIR=<cwd> VALGRIND_LIB=<lib> <valgrind> --tool=callgrind \
  --callgrind-out-file=<slot>/target/limits-worlds/<task>/<label>.%p.out \
  <binary-path> --exact <test name> --nocapture'
```

- **The test name is the binary's,** such as `fuzzy::repair::a_change_failing_ci_is_repaired_and_lands`. **Check the count:** libtest's last line must say `1 passed`. A run that selected nothing proves nothing.
- **Read it** with `callgrind_annotate --inclusive=yes` and with `--inclusive=no`, for the same file. Keep the top of each. Dependencies have symbols, not lines: a function's name is enough.
- **The figure** is the `PROGRAM TOTALS` Ir. After `cuts-fork`, a sweep writes one file per cut (the `%p`): its figure is the sum of the files' totals, and your table says so.

## The real loop under callgrind

Callgrind runs a story about 30 to 60 times slower, and the real-loop world's patience runs on the real clock. So:

- **Raise `PATIENCE` locally,** `temper/tests/real-loop/src/lib.rs`, to `from_secs(3000)`, build, and restore it before you commit. `git diff` before each commit shows it gone. The cluster's link deadlines derive from it. Compare only runs built with it.
- **A call's own bound** (`temper/tests/real-loop/src/calls.rs`, `argv.1.timeout` and `argv.1.answer`) can also pass. If one does, raise that too, locally, and say which.
- **A poll looks bigger than it is.** Its period stays real time while the work takes 30 to 60 times longer, so a timer-driven path runs more often here than natively. For anything driven by a timer, read its **call count**, and scale it by the run's slowdown (callgrind's wall over native wall) before ranking it. Say that you did.
- **Its counts vary more than a simulated world's.** Run a real-loop story twice per arm, write both, and treat the spread as the figure's error. The conformance and one-process worlds run on simulated time: once per arm is enough, unless two figures disagree with what you expected.
- **Children aren't counted.** A real `sh` runs natively and outside the count. The request's CPU seconds do include descendants. Say how many children a story spawned, by the calls of its spawn path in the profile, and take one native `/usr/bin/time -v` of the same test for CPU seconds, which counts them.
- **If a story can't finish under callgrind** with these raised, write down what fired, keep the partial run's figure marked partial, profile the others, and go on. A story with no profile gets no fix.

## What's written down

Write your figures to `temper/docs/plan/limits-worlds/figures/<your task>.md`, a file of its own so that parallel tasks never meet in one, and commit it. Name the machine by its CPU count and model only, never by its host name.

```markdown
# <task>: <what these runs measure>

<date -u>, commit <short sha>, <nproc> CPUs, <lscpu's model name>, valgrind <version>.

| Story | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---|---:|---:|---:|---|

## <story>: the largest by inclusive Ir
| Share | Ir | Calls | Function |
|---:|---:|---:|---|

## <story>: the largest by self Ir
| Share | Ir | Function |
|---:|---:|---|

## Causes, largest first
| Cause | Where | Share of each story | Fixed by this task, or whose it is |
|---|---|---|---|
```

Keep a table to its top 25 rows; a table is a reading aid, not the profile. **Delete `target/limits-worlds/`** once your figures are written: it's yours, and it isn't committed. `close` carries your figures into the account.
