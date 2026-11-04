# root: status

2026-11-04. Where jig's code lags or departs from [root.md](root.md).

- **Not yet:** 2, 5, 6: `smith-inline-agent` composed in the engine's
  root. 3: the core does not yet take an agent's preparation alone: it
  asks each root with `Now::PrepareAgent`, which each root answers with
  `Event::PreparedAgent`, and `Event::Activate` has no `busy`
  (`jig-domain-core/src/routing/run/preparation.rs`).
- **Departs:** 11, 12: `ops`'s root reads values it carries: it decodes a
  triage's delegate batch (`jig_protocol_tools::batch`, `translate.rs`)
  and renders start messages' words (`jig_protocol_tools::words`,
  `assemble.rs`) itself, rather than handing them out; a code task moves
  them to its protocol layer.
