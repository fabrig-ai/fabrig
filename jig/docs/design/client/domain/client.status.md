# client: status

2026-10-29. Where jig's code lags or departs from [client.md](client.md).

- **Built:** 1 to 17, 19, 20: public surface, keyed requests, watches, small store, helpers, named send, turn/job reads and deadline spans; 7: clamped windows; 12: held/full sign-ins. 1, 6, 11: memory-only Seen and legacy erase. 14: internal hand-offs uncapped, credential-boundary read fan-out bounded by admitted reads, receiving document envelopes retained.
- **Not yet:** 12, 17, 20: service mode, relayed decisions and one-shot story. 8.3: reasoning/input producers. 10: objects made by turn calls. Independent read-bound and omitted-intent completion unknown; read cancellation is implemented.
