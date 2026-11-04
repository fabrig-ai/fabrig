# D035: World tests speak in bare numbers, and each file names its own fixtures

- **Kind:** tests
- **Where:** `jig/tests`, for example `tests/core/tests/suite/stories.rs` and `tests/core/tests/suite/reads.rs`
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Readability of the worlds' tests")
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (A1 F10: `ask` defined 7 times; A12 F15: stories in bare numbers)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 19: the core world's "verbs in the kit" never planned)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (a reader sees the story a test tells)
- **Size:** M
- **Status:** open
- **Paid when:** each world names its parties, projects, tasks and connectors once, in the world or its kit, and its common acts are helpers named for the design's verbs; a bare number is left only where the number is the point.
- **Measure:** `rg -o -t rust '\b[a-z_]+\((&mut [a-z_]+,\s*)?[0-9]{2,}(_[0-9]+)?,\s*[0-9]+' jig/tests | wc -l` (262 at base; 137 on 10-18)

## What

- calls with two or more bare integers, the first of two digits or more: 262 (137 on 10-18); `ask(&mut world, <n>, …)` 100 times (66 on 10-18);
- named constants are per file: `PROJECT` is defined in 49 files (`rg -l -t rust '^\s*(pub(\(crate\))? )?const PROJECT:' jig/tests | wc -l`; 17 on 10-18), `SEED` in 53, `TASK` in 22, `PARTY` in 20.

## Why it matters

A test that reads `ask(&mut world, 43, …)` needs its setup read first; a fixture named in
49 files can mean 49 things.

## What pays it back

Fixtures and verbs in each world's kit (the core world's first: `ask` is defined several
times), then the files converted as they are touched; group tests that differ only in data
into table tests.
