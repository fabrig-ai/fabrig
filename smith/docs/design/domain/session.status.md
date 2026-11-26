# session: status

2026-11-26. Where smith's code lags or departs from [session.md](session.md).

- **Not yet:** 5: a batch's calls are not yet fixed when it starts: a call answered `invalid` at the tools' entrance lets the reads past the write that ended the batch start in it, with no room reserved for their results. [in flight: change 19267]
