# root: status

2026-11-04. Where jig's code lags or departs from [root.md](root.md).

- **Not yet:** 2, 5, 6: `smith-inline-agent` composed in the engine's
  root.
- **Not yet:** 3: a task that comes due waits in the core: the core still asks each root with `Now::Activate` and `Now::PrepareAgent`, and each root pushes the answer back.
- **Departs:** 11, 12: `ops`'s root reads values it carries: it decodes a
  triage's delegate batch (`jig_protocol_tools::batch`, `translate.rs`)
  and renders start messages' words (`jig_protocol_tools::words`,
  `assemble.rs`) itself, rather than handing them out; a code task moves
  them to its protocol layer.
