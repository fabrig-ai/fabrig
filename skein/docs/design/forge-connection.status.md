# forge-connection: status

2026-10-31. Where skein's code lags or departs from [forge-connection.md](forge-connection.md).

- **Built:** 2, 7: uncapped handoffs and edge/derived-capacity checks without a process sum. 2 to 5, 7, 8, 10: runtime access; installation routing, token renewal, GitHub file/compare/merge/reviewer calls and queue; rate/304 accounting; bounded credential-free job-log redirects and delivery-log polling; heap checks and machine/protocol stories. 2.1, 8.5: the abort, a close that does not wait, aborting every socket, a graceful close already sent among them.
- **Not yet:** 6: TLS listener. 10: remaining GitHub cases, details unknown. 12, Later: the TLS close, with the shared transport.
