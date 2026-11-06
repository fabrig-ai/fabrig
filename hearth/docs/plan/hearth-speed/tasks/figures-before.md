+++
id = "figures-before"
part = "before"
kind = "docs"
needs = []
files = ["hearth/docs/plan/hearth-speed/figures.md"]
effort = "high"
minutes = 40
+++

# The focused suite's cost, measured and profiled

## Goal

Record what hearth's focused suite costs at the base commit, and what those CPU seconds are spent on, so that the later tasks start from a profile and not from guesses. This changes no code.

Acceptance:

- `hearth/docs/plan/hearth-speed/figures.md` exists, with a "Before" section: the base commit, the machine, the date, and the method's two figures (user, system, wall, tests), from `notes/common.md`.
- **By family and by binary:** CPU seconds and tests per family, and the twenty most costly tests with their CPU seconds.
- **The floor:** the CPU of the cheapest test in each of the four families' binaries, and in a typical crate's `--lib` binary, as an estimate of what a test costs before it does anything. What that comes to over all the tests.
- **A "Profile" section,** per family: the share of self time by crate (from the demangled symbol's first path segment, with the kernel and libc apart), and the ten hottest functions. For the real-git binary, the split between `git` processes and the test process, and how much of the test process goes before the first `git` it runs for the code under test.
- **What it found:** a short list of what repeats, with its measured share: work done per seed or per test that gives the same result each time, and any inefficiency in hearth's own code. Facts and shares only; the later tasks choose.
- No file but `figures.md` changes.

## Inputs

- `hearth/docs/plan/hearth-speed/notes/common.md`: the base commit and the method
- `hearth/docs/plan/hearth-speed/inputs/request.md`, "Where it goes"
- `hearth/docs/design/domain/testing.md`, 9.1, and `hearth/docs/design/domain/testing.status.md`, its **Departs**
- `tools/tiers/design.md`, 7.3 and 10.2

## Owns and may change

- **Owns:** `hearth/docs/plan/hearth-speed/figures.md`.
- **Departs from your text:** this writes in a plan's directory, and adds no test.
- **Profiling, pre-approved:** `perf` through heavy, for example `heavy perf record -F 999 -o <untracked file> -- cargo nextest run -p PKG --test BIN`, which follows the tests it spawns, or a test binary run directly. If `perf` refuses without root, `sudo perf` for that command alone, changing no setting. If `perf` is absent, `valgrind --tool=callgrind` on a sample of each family's tests, labelled. If neither is on the machine, nothing gets installed: the profile is `unknown`, and the figures by test and family stand.
- **Throwaway scripts** that sum `perf report` by crate stay untracked and are deleted before the push.

## Steps

1. In your slot, `git checkout --detach` the base commit; build; measure twice; keep each DIR for the by-test sums.
2. Profile each family; sum by crate; read the hottest functions' code enough to say whether they repeat work.
3. Return to your branch, write `figures.md`, delete what's untracked. `git status` shows only `figures.md`.
4. Citations at 0. Commit and push.

## Report

The two figures, the shares by family, the top three things the profile shows, and anything `unknown`.
