# root: status

2026-11-04. Where jig's code lags or departs from [root.md](root.md).

- **Not yet:** 2, 5, 6: `smith-inline-agent` composed in the engine's
  root. 3: `jig-domain-link`'s `Class` and `LinkUp::new_work`: each root
  keeps its own `Class`, and the test root lists `LinkUp` by hand. 5:
  `jig-domain-hub-link`'s shared translations, which each root spells:
  the job's assignment, the relay of a settled call, the brief's plan
  and completion, the `Now`s that are the link's.
- **Departs:** 11, 12: `ops`'s root reads values it carries: it decodes a
  triage's delegate batch (`jig_protocol_tools::batch`, `translate.rs`)
  and renders start messages' words (`jig_protocol_tools::words`,
  `assemble.rs`) itself, rather than handing them out; a code task moves
  them to its protocol layer.
