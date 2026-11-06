# render: status

2026-11-06. Where hearth's code lags or departs from [render.md](render.md).

- **Not yet:** 5.4: recording apt's and mmdebstrap's versions from
  `--version` in M2. `hearth-protocol-builder`'s resolution reply gives
  both as empty byte strings.
- **Not yet:** 6.2: the diff's rebuild time from each layer's last
  recorded cost. The domain's and printer's typed changes carry the
  rebuild layer, with no time or lookup of those costs.
- **Departs:** 8.2: unsettled: this section's `Resolved` reply carries
  a closure, while `hearth/docs/design/domain/render.md`, 5.3, puts
  matching `Inst` triples to `Packages` stanzas in the domain. The code
  follows 5.3: the protocol layer sends `Inst` and `Stanza` events, then
  `Resolved` with the indices, tool versions and costs; the domain
  assembles the closure. Section 10 keeps the question.
- **Departs:** 8.5: the request of 2026-11-05 for the config machine has
  the queues beside the JSON decoder uncapped, and one test driver keeps
  both at capacity 1:
  `hearth/testing/hearth-test-render/src/golden/repository.rs`, `decoded`,
  with `JSON_QUEUE`. That round left the file alone, as another plan was
  moving it into that kit, and no order rests on it: a second output
  panics there, and loses nothing.
- **Departs:** 8: `skein/docs/foundation/programming-model.md`, 5.4 asks for one explicit
  state enum per lifecycle, each state holding exactly its data, no lifecycle
  in flags or `Option`s beside it; `hearth-protocol-builder`'s
  `store.rs` keeps `pending`, `cursor`, `closing`, `deferred` and `walk`, and
  `keep.rs` `removing` and `failed` with its `Option`s, because the request of
  2026-11-05 that gave the rest of that crate its states paid a debt item that
  did not name these two.
