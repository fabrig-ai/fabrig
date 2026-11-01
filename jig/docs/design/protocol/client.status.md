# client: status

2026-11-01. Where jig's code lags or departs from [client.md](client.md).

- **Built:** 1 to 14, 16, 17: transport-independent messages, default HTTP mapping, whole authorities and bounded streams, typed versions, turn reads, spans, projects, moves, pasted sign-in and Seen ordering. 3: project resources carry their adoption role, including fork, and an independent home flag. 17.3: public Host controls Secure cookies. 3, 13: of the remaining settings/revision records, `MoveToNewest` and `Task.settings` are built. 13: internal hand-offs uncapped, read cancellation bounded by admitted reads, transport edge limits retained.
- **Not yet:** 6: gone answer. 7.3: reasoning/input producers. 17.4: reference service reporting peer end during an engine call. 17.1, 18: OpenID provider.

- **Built:** 4, 17.2: both stream admission bounds answer 409 with `StreamsFull { bound, size }`; native and browser HTTP clients decode it, retry with the full fact in the link view and tell the caller, clearing it when `me` returns or the sign-in ends. Empty or malformed 409 bodies remain gone.
