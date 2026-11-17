# design: status

2026-11-17. Where the outward scan's code lags or departs from [design.md](design.md).

- **Not yet:** 1, 2, 3, 4 and 5: no list file and no text check: the Rust test `tests/boundary/tests/outward.rs` keeps its names in its own `FIXED` table, with a special case for the name followed by digits and a prefilter of hard-coded prefixes, and no test compares two verdicts.
