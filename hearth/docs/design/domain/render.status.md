# render: status

2026-11-04. Where hearth's code lags or departs from [render.md](render.md).

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
  assembles the closure.
- **Departs:** 8.4: unsettled: this section requires layer peak formulas
  and a builder-wide sum, while `skein/docs/foundation/programming-model.md`,
  6.3, requires edge bounds and measured heap ceilings, with no
  process-wide worst case. `hearth_service_builder::worst_case` sums
  hearth's domain, protocol layer and nine queues, omitting skein's io
  and file terms; only its tests call it, and the 1 GiB they hold it to
  is no process's figure. The formulas retain the `worst_case` name
  (`hearth/docs/development/debt/D002-process-worst-case.md`).
