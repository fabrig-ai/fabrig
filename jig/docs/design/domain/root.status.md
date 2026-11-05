# root: status

2026-11-04. Where jig's code lags or departs from [root.md](root.md).

- **Not yet:** 2, 5, 6: `smith-inline-agent` composed in the engine's
  root.
- **Departs:** 11, 12: `ops`'s root reads a value it carries: it decodes a
  triage's delegate batch (`jig_protocol_tools::batch`, `translate.rs`)
  itself, rather than handing it out; a code task moves it to its protocol
  layer.
