# llm-connection: status

2026-11-07. Where skein's code lags or departs from [llm-connection.md](llm-connection.md).

- **Departs:** 5: `skein/docs/foundation/programming-model.md`, 5.4 asks
  for one explicit state enum per lifecycle, each state holding exactly
  its data, no lifecycle in flags or `Option`s beside it, exhaustive
  matches and one handler per cell; `skein-llm-connection/src/call.rs`
  keeps `waiting` beside `Phase`, because the component recomputes it for
  every connection in every phase, and `reservation`, because it outlives
  the call's token until the pool's pass; neither it nor `component.rs` keeps
  one handler per cell, for which no reason is recorded.
  4: beside its table, an owner may ask for every call's rendered body
  (`Component::keep_bodies`) and take each one (`Component::pop_body`), never
  an event, because a development deployment must read what left
  (`llm.status.md`).
