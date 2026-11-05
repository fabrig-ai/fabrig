# lib: status

2026-11-05. Where skein's code lags or departs from [lib.md](lib.md).

- **Not yet:** 3: `Slab::get2_mut`. 5: `Lru`. 8: `Civil` and `Wall::civil`.
- **Departs:** 12: `skein/docs/foundation/programming-model.md`, 5.4
  asks for one explicit state enum per lifecycle, each state holding
  exactly its data, no lifecycle in flags or `Option`s beside it,
  exhaustive matches and one handler per cell;
  `skein-lib/src/requests.rs`'s `RequestTable` keeps `Entry`'s
  `restored`, `save` and `changed` flags and `progress` and `anchor`
  options beside `state`, rather than in lifecycle states; no reason is
  recorded (the code's side is
  `skein/docs/development/debt/D025-request-table-in-lib.md`).
