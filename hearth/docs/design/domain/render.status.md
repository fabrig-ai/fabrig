# render: status

2026-11-05. Where hearth's code lags or departs from [render.md](render.md).

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
- **Not yet:** 8.5: the tests of mmdebstrap's pipe reader and list reader,
  in `hearth/tests/mmdebstrap`, and the queues beside them uncapped.
- **Departs:** 8.5: the store step's order has no named predicate, because
  the service round comes later (`hearth/docs/development/debt/D001-room-checks-and-max-out.md`).
  `hearth-protocol-builder`'s `up` reads `!above.is_empty()`, of the whole
  queue, as that step's answer; the queue's order and the indices' `waiting`
  keep it true.
