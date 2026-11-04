# root: status

2026-11-04. Where jig's code lags or departs from [root.md](root.md).

- **Not yet:** 2, 5, 6: `smith-inline-agent` composed in the engine's
  root. 3: a task that comes due waits in the core: the core still asks
  each root with `Now::Activate` and `Now::PrepareAgent`
  (`jig-domain-core/src/routing/task/lifecycle.rs`,
  `jig-domain-core/src/routing/run/preparation.rs`), and each root
  answers with `Event::Activate` and `Event::PreparedAgent`. The user's
  request of 11-04 asked for this with behaviour unchanged, but
  building 3 runs an activation in a later decision than the one that
  made its task due, so it waits on section 14 (the code's side is
  `jig/docs/development/debt/D001-roots-sequence-core-steps.md`). Until
  it is built, `temper/docs/design/domain/root.md`, 4.1 and
  `temper/docs/design/domain/forge.md`, 8.3 keep the activation in that
  decision, where the code has it.
- **Departs:** 11, 12: `ops`'s root reads values it carries: it decodes a
  triage's delegate batch (`jig_protocol_tools::batch`, `translate.rs`)
  and renders start messages' words (`jig_protocol_tools::words`,
  `assemble.rs`) itself, rather than handing them out; a code task moves
  them to its protocol layer.
