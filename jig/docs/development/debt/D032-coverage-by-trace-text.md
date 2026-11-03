# D032: Some tests find what happened by searching trace text

- **Kind:** tests
- **Where:** `tests/notes/tests/suite/simulation.rs`, `tests/notes/tests/suite/drop_project.rs`, `tests/core/tests/suite`, `tests/client/src/joint`, `tests/host`
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Checks that prove less than they appear to": coverage by text search over `Debug` output)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list ("Tech-debt cross-checks still open": coverage found by text search; a host-world trace assertion matches `Debug` text)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (assertions that say exactly what they check)
- **Size:** S
- **Status:** open
- **Paid when:** no world test counts or finds events by matching formatted trace lines; each asks a typed record of what was reached, as the tasks world's `Reached` does.
- **Measure:** `rg -n -t rust '\.(contains|starts_with|ends_with|matches)\("[^"]*[A-Za-z]{3,}' jig/tests jig/testing | rg -v 'Find::text|b"' | wc -l` (105 candidate lines at base, to be read one by one; 80 on 10-18)

## What

For example, the notes world counts a store's loads as the trace lines containing
`"out Load"` (`simulation.rs`, lines 124 and 130; `drop_project.rs`, line 122). The tasks
world shows the better form: typed coverage (`Reached` and `MARKERS`, `tests/tasks/src/coverage.rs`).

## Why it matters

A change to a `Debug` format, or a word appearing in another line, changes what the test
proves without failing it.

## What pays it back

Typed observations in each world's trace or referee, and the string searches replaced.
Goes with D033.
