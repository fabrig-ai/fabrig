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
- **Not yet:** 8.4: `hearth_service_builder::worst_case` still sums the
  domain, the protocol layer and nine queues, and only its tests call it;
  the formulas keep the `worst_case` name; there is no `valid(limits)`, and
  `hearth_service_builder::Service::new` asserts only that the io limits are
  usable and that the queues hold every `MAX_OUT`; most worlds still hold
  peaks to formulas; and the large render isn't built
  (`hearth/docs/development/debt/D002-process-worst-case.md`).
