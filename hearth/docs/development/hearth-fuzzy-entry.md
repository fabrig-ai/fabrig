# hearth-fuzzy-entry: retirement audit

2026-11-07. The plan's single part is landed. Its implementation changes
only `tools/tiers/budgets.toml`: `[gate.hearth.fuzzy]` has
`cpu_suite_s = 120`, with the requested comment beneath the focused
suite's comment. The real file is accepted by `budgets()`; hearth's
focused entry remains 30 CPU seconds.

No design was changed by this plan, so no design status file or Later
section needs an update. The request leaves the testing design and its
status unchanged. The fuzzy figure's wall-time versus CPU meaning remains
open in `hearth/docs/design/domain/testing.md`, sections 9.1 and 11.
The new entry does not settle that question or establish a user-approved
CPU figure. No contradiction was corrected.

Retirement validation is incomplete. The plan's status check reports its
part landed, but the required test command fails in the build wrapper's
admission step before running tests. No suite measurement was made.
The retirement call's report carries the operational failure; this audit
does not declare the plan retired.
