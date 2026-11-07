# limits-fuzzy: how a sweep is profiled and written down

`valgrind --tool=callgrind` on one test by name, on the unoptimized test binary: instruction counts, which a loaded machine doesn't move, where CPU seconds swing by a third. Quote both, instructions first.

## Getting valgrind

```sh
command -v valgrind callgrind_annotate
```

If a builder has neither, keep a private copy in your slot, with no root and no change to the machine:

```sh
mkdir -p target/limits-fuzzy/tool && cd target/limits-fuzzy/tool
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
heavy bash -lc 'cd <cwd> && CARGO_MANIFEST_DIR=<cwd> valgrind --tool=callgrind \
  --callgrind-out-file=<slot>/target/limits-fuzzy/<task>/<label>.%p.out \
  <binary-path> --exact <test name> --nocapture'
```

- **The test name is the binary's,** such as `fuzzy::walking::fuzzy_walking_shard_0`. **Check the count:** libtest's last line must say `1 passed`. A run that selected nothing proves nothing.
- **Read it** with `callgrind_annotate --inclusive=yes` and with `--inclusive=no`, for the same file. Keep the top of each. Dependencies have symbols, not lines: a function's name is enough.
- **The figure** is the `PROGRAM TOTALS` Ir. A test that forks writes one file per process (the `%p`): its figure is the sum of the files' totals, and your table says so.
- **Native CPU** is one run of the same binary and name outside valgrind, through `heavy`, under GNU time if it's there. If it isn't, this wrapper, kept untracked and gone before you push, counts user plus system with waited-for descendants:

```python
import os, resource, subprocess, sys, time
before = resource.getrusage(resource.RUSAGE_CHILDREN)
started = time.monotonic()
code = subprocess.run(sys.argv[1:]).returncode
used = resource.getrusage(resource.RUSAGE_CHILDREN)
print("user", used.ru_utime - before.ru_utime, "system", used.ru_stime - before.ru_stime,
      "wall", time.monotonic() - started, "load", os.getloadavg())
sys.exit(code)
```

## How many runs

The engine-domain, one-process and codecs worlds run on simulated time, or on none: once per arm is enough, unless two figures disagree with what you expected.

## The real loop under callgrind

For the real-loop world alone. Callgrind runs a story about 30 to 60 times slower, and that world's patience runs on the real clock.

- **Raise `PATIENCE` locally,** `temper/tests/real-loop/src/lib.rs`, to `from_secs(3000)`, build, and restore it before you commit. `git diff` before each commit shows it gone. The cluster's link deadlines derive from it. Compare only runs built with it. It's the one edit that may sit in your slot while a run goes.
- **A call's own bound** (`temper/tests/real-loop/src/calls.rs`, `argv.1.timeout` and `argv.1.answer`) can also pass. If one does, raise that too, locally, and say which.
- **A poll looks bigger than it is.** Its period stays real time while the work takes 30 to 60 times longer, so a timer-driven path runs more often here than natively. For anything driven by a timer, read its **call count**, and scale it by the run's slowdown (callgrind's wall over native wall) before ranking it. Say that you did.
- **Its counts vary more than a simulated world's.** Run a real-loop story twice per arm, write both, and treat the spread as the figure's error.
- **Children aren't counted.** A real `sh` runs natively and outside the count. CPU seconds do include descendants. Say how many children a story spawned, by the calls of its spawn path in the profile.
- **If a story can't finish under callgrind** with these raised, write down what fired, keep the partial run's figure marked partial, profile the others, and go on. `fuzzy::repair::a_change_failing_ci_is_repaired_and_lands` stopped at the fake forge's observation cap in the last plan's profiles, while passing natively. A story with no profile gets no fix.

## What's written down

Write your figures to `temper/docs/plan/limits-fuzzy/figures/<your task>.md`, a file of its own so that parallel tasks never meet in one, and commit it. Name the machine by its CPU count and model only, never by its host name.

```markdown
# <task>: <what these runs measure>

<date -u>, commit <short sha>, <nproc> CPUs, <lscpu's model name>, valgrind <version>.

| Sweep | Worlds or inputs drawn | Arm | Run | Ir | Native CPU s (user + system, with descendants) | Result |
|---|---:|---|---:|---:|---:|---|

## <sweep>: the largest by inclusive Ir
| Share | Ir | Calls | Function |
|---:|---:|---:|---|

## <sweep>: the largest by self Ir
| Share | Ir | Function |
|---:|---:|---|

## Causes, largest first
| Cause | Where | Share of each sweep | What it saved, or whose it is |
|---|---|---|---|

## Left
<each cause outside your crates: where, its share, what you read of it>

## Stopped at
<the next cause, its share, and what could really go of it>
```

**The worlds or inputs drawn are the same number before and after,** on the same row of the table: that's the request's second point, shown. Keep a table to its top 25 rows; a table is a reading aid, not the profile. **Delete `target/limits-fuzzy/`** once your figures are written: it's yours, and it isn't committed. `close` carries your figures into the account.
