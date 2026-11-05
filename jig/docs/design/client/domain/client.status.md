# client: status

2026-11-04. Where jig's code lags or departs from [client.md](client.md).

- **Not yet:** 14: `long_text`, and an outcome before its input.
- **Not yet:** 19: `set_agreed_goal_from`.
- **Not yet:** 10: a committed turn's calls carry no objects or parts: `jig-protocol-transcript` decodes them empty. 12: no story relays a person's decision with their own credential. 17, 20: the one-shot mode and its story, and a service in a host or world (the client takes `With::Service`).
- **Departs:** 19: `words` keys its phrases under skein-ui owner 65537 (`jig/crates/jig-client/src/words.rs`, `OWNER`), beside jig's 65536 (root.md, 15); no reason is written. 14: limits above the wire's rows ceiling are refused at construction; the section has no rule for other over-ceiling limits. `jig-client-protocol-link/src/limits.rs`, `wire`, silently caps fifteen mapped limits with `.min(w::CEILINGS.…)`; no reason is recorded. The backfill recommends refusing them (the code's side is `jig/docs/development/debt/D009-limits-mapped-by-hand.md`).
