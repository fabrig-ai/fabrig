# tools: status

2026-11-01. Where smith's code lags or departs from [tools.md](tools.md).

- **Built:** 3 to 5: jobs and waits that yield to other calls. Other implementation detail unknown.
- **Built:** 3, 4 and 10: `remove` on the direct path and the program backend; native file snapshots feed the session's known versions.
- **Built:** 3 and 5: native list and search include dot-named paths, honour ignore files and exclude git directories, including when a glob selects git paths.
- **Not yet:** 5: containment.
