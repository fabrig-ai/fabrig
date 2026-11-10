# hearth-fuzzy-entry: retirement

2026-11-07.

The sole part, `fuzzy-entry`, is landed. `tools/tiers/budgets.toml` holds
`[gate.hearth.fuzzy]` with `cpu_suite_s = 120` beside the focused suite's
entry, with the comment required by the request. The landed change adds
only those four lines.

No design document changed for this plan, so no design status file or
Later section needs an update. `tools/tiers/design.md`, 7.3 already
describes the table. The unrelated design changes made while this plan
was running need no reconciliation for this entry.

The question in `hearth/docs/design/testing/README.md`, 6 remains open:
whether the fuzzy suite's 120 seconds stay wall time or become a CPU
budget. Section 9.1 and `../design/testing/README.status.md` remain unchanged, as the
request requires. The entry's comment preserves that question; the entry
does not settle the figure. No contradiction was corrected.

Validation: the plan's record reports the task landed, the development
tooling's own test run passes all 3,314 tests, and `budgets()` accepts the
real file with hearth's focused entry at 30 and fuzzy entry at 120. Nothing
was measured for this plan, and no hearth suite or slow tier was run.
