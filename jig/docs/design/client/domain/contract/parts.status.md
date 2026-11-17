# parts: status

2026-11-17. Where jig's code lags or departs from [parts.md](parts.md).

- **Not yet:** 3, *The client's contract*: `jig_client_domain::contract` has no `PartAsk`, `RowPart` or `PartFailure`, and its task queries, lines, `Tasks` and `TaskPage` no parts or ended mark [in flight: change 6292].
- **Not yet:** 2, 3 and 7: jig's client wire is at version 27, with no part on a task query or a row and no ended mark; the clients' component answers no part `not_served`, and the link checks no row's parts [in flight: change 6298].
- **Not yet:** 4, 5 and 6: the core asks no connector for a row's parts, holds no answer for them, marks no row by a part's input, a note marks nothing, and it keeps no ended mark [in flight: change 6297].
