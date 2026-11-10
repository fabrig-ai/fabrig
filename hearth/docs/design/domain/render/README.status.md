# render: status

2026-11-06. Where hearth's code lags or departs from [README.md](README.md).

- **Not yet:** 4.2: the diff's rebuild time from each layer's last
  recorded cost. The domain's and printer's typed changes carry the
  rebuild layer, with no time or lookup of those costs.
- **Not yet:** 5.5: resumed work's order, for a file event.
  `hearth-service-builder/src/tests.rs`,
  `resumed_cache_work_goes_before_a_child_s_exit`, holds `protocol_up`'s guard
  on `told`, and pushes no file event; no test of that crate pushes one to
  `file_told`. Whether any test fails with the guard out of the `file_told`
  loop is unknown: no run on record tried it. The guard stays.
- **Departs:** 5.2: unsettled: this section's `Resolved` reply carries
  a closure, while `hearth/docs/design/domain/render/spec.md`, 2.3, puts
  matching `Inst` triples to `Packages` stanzas in the domain. The code
  follows `spec.md`, 2.3: the protocol layer sends `Inst` and `Stanza` events, then
  `Resolved` with the indices, tool versions and costs; the domain
  assembles the closure. `README.md`, section 7 keeps the question.
- **Departs:** 5.5: the request of 2026-11-05 for the config machine has
  the queues beside the JSON decoder uncapped, and one test driver keeps
  both at capacity 1:
  `hearth/testing/hearth-test-render/src/golden/repository.rs`, `decoded`,
  with `JSON_QUEUE`. No task has taken it up, and no order rests on it: a
  second output panics there, and loses nothing, as in hearth's other test
  drivers with a queue of 1 (`hearth/tests/tree/src/helper.rs`).
- **Departs:** 5: `skein/docs/foundation/programming-model.md`, 5.4 asks for one explicit
  state enum per lifecycle, each state holding exactly its data, no lifecycle
  in flags or `Option`s beside it; `hearth-protocol-builder`'s
  `store.rs` keeps `pending`, `cursor`, `closing`, `deferred` and `walk`, and
  `keep.rs` `removing` and `failed` with its `Option`s, because the request of
  2026-11-05 that gave the rest of that crate its states paid a debt item that
  did not name these two.
