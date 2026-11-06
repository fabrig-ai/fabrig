+++
id = "profile"
part = "profile"
kind = "docs"
needs = []
files = ["hearth/docs/plan/hearth-speed-2/figures.md"]
effort = "high"
minutes = 90
+++

# Where hearth's focused suite spends its CPU, by function

## Goal

No profile of this suite has ever been taken, and `skein/docs/foundation/testing-strategy.md`, 8 says a slow test's cause is found with one. Measure the suite at the base commit and profile representative tests with callgrind, so the repair tasks start from causes and not from guesses. This changes no code.

Acceptance:

- `hearth/docs/plan/hearth-speed-2/figures.md` exists with the sections below, under these headings, because the repair tasks read them by name.
- **`## Before`:** the base commit, the machine, the date, the method's two figures (user, system, CPU, wall, tests). `### By family` with user and system apart, because callgrind sees no kernel time. `### By binary`. `### Twenty most costly tests`.
- **`## Profile`:** one `### ` per family (`Builder suite`, `Tree suite`, `Real git`, `Render suite`, `All the rest`), each with:
  - The tests sampled, with each one's CPU from the collector, its `Ir`, and the share of the family's CPU the sample holds. Sample the three to five most costly tests of the family, its median test and its cheapest. A sample's shares carried over to the family are labelled as estimates.
  - **By bucket,** self `Ir`: hearth's production crates (`hearth_*` under `hearth/crates/`), hearth's test code and kits (`hearth/tests/`, `hearth/testing/`), skein's (`skein_*`), hashing (`sha1`, `sha2`), the allocator (`malloc`, `free`, the counting allocator), Rust's `core`, `alloc` and `std`, the harness (`test::` and what the test `main` runs before the test's function), process startup (`ld-linux`, `_dl_*`, relocations), and anything else over 1%.
  - **The twenty hottest functions by self `Ir`, and the twenty heaviest hearth or skein functions by inclusive `Ir`.** Unoptimized generics show up as `core::...` frames, so the inclusive table is the one that says whose loop it is.
- **`### Real git`** also gives the split by process: the test process, `git`, and the `hearth` binary it runs, in `Ir`; how much of the test process goes before the code under test starts, and how much of `git`'s goes to `git init`, `git add`, `git commit` against the commands the code under test runs.
- **`## A test's process`:** what a test costs before it checks anything, by binary size: the cheapest test of each family's binary and of a typical `--lib` binary, CPU and `Ir`, the `Ir` split into startup, harness and the test's own function, and what that floor comes to over all the tests.
- **`## What it found`:** per family, largest first, one row each: the function and its file and line, what it does that repeats or scans (work a world repeats whatever its seed, setup repeated, a scan where a lookup would do, data copied or encoded again), its estimated share of the family's CPU and of the suite's, and whether it's production code, test code or a shared kit. Read the code of each hot function enough to say this. Facts and shares only: the later tasks choose. Name a cost in skein, smith, jig or temper separately, under `### Shared kits`, with its share: that's a question for the plan, not a repair.
- Anything not measured is `unknown`, with why; a tool the machine lacks is named. No file but `figures.md` changes.

## Inputs

- `hearth/docs/plan/hearth-speed-2/notes/common.md`: the aim, the base commit, the method and its driver
- `hearth/docs/plan/hearth-speed-2/inputs/request.md`, 1
- `skein/docs/foundation/testing-strategy.md`, 8: "A test is a process", "What a test costs the suite", "A slow test is understood, never hidden"
- `hearth/docs/design/domain/testing.md`, 9.1 and 11
- `tools/tiers/design.md`, 7.3 and 10.2

## Owns and may change

- **Owns:** `hearth/docs/plan/hearth-speed-2/figures.md`.
- **Departs from your text:** this writes in a plan's directory, adds no test and changes no code, so the end of `notes/common.md` asks of it only the citations and `git status` showing `figures.md` alone. Your branch starts with the base's code (only `hearth/docs/plan/hearth-speed-2/` differs), so you measure where you are and don't detach.
- **Throwaway, pre-approved:** `.measure.py` and any script that sums `callgrind_annotate`'s output, untracked and deleted before the push. Callgrind's output goes under `target/hs2/`.
- **If `valgrind` isn't on this machine:** first line `Question:`, naming the machine. Nothing gets installed, and no other profiler stands in. The request says the builders have it, and an earlier run on one didn't.
- **A test that fails under callgrind** because a harness deadline passes at callgrind's speed, or a syscall valgrind doesn't know: note it, take the next test in the family. A test that can't be profiled isn't a finding about the test.
- **If `strace` is on the machine,** `--tracer 'strace -c -f'` on two tests of each family gives the syscalls behind the system time, for `## Profile`. If it isn't, system time stays by family and test, and what's behind it is `unknown`.

## Steps

### How to profile

Nextest's tracer mode runs one test with the environment `skein_scratch::paths` reads, and takes off the timeouts. A test runs in its package's directory and its children in their own, so the output path is absolute, and its directory must exist first (`mkdir -p target/hs2/cg`):

```text
NEXTEST_DOUBLE_SPAWN=0 TEMPER_BUNDLE=none heavy cargo nextest run --profile default \
  -p PKG --test BIN \
  --tracer "valgrind --tool=callgrind --trace-children=yes --callgrind-out-file=$PWD/target/hs2/cg/NAME.%p.out" \
  -E 'test(=TEST_NAME)'
```

Use `--lib` in place of `--test BIN` for a library's tests. One test at a time; callgrind runs some fifty times slower, so a 1 s test takes about a minute. `callgrind_annotate --inclusive=no` and `--inclusive=yes` with `--threshold=99` read the results; each `%p` file is one process, and the first line of its `cmd:` names which. Skim, don't dump: `head -60` of an annotation is enough for a table.

### The work

1. `python3 -I -B .measure.py default target/hs2/base-1`, then `base-2`. The tables of `## Before` from the lower run's `result["tests"]`.
2. Profile each family's sample, then the floor's tests. Sum by bucket.
3. Read the hot functions. Write `## What it found`.
4. Delete what's untracked. `git status` shows only `figures.md`. Citations at 0. Commit and push.

## Report

The two figures, the families, the top three things the profile shows for each family, what's in a shared kit if anything, and anything `unknown`.
