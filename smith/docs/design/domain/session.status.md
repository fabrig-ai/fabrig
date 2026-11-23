# session: status

2026-11-23. Where smith's code lags or departs from [session.md](session.md).

- **Not yet:** 4: a rate limit retried after its wait without spending a retry, its wait told to the host, and a delay past the session's expiry ending the run limited for its account; `smith-domain-session/src/session/retry.rs` counts a rate limit as any transient failure [in flight: change 12974].
- **Not yet:** 8.2: a carried-over message left with no block keeping the text `[reasoning not carried over]`, its bytes and block within the session's limits. [in flight: change 8576]

