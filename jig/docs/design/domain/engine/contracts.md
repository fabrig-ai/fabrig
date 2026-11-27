# Contracts: connectors' declarations

The shapes the changes of the goal that carries connectors' declarations share
(`temper/docs/design/draft/design-economy.md`, 4.3), beside what `../connectors.md`, 2 and
`vocabulary.md`, 2.1 keep. They bind while the goal runs; its last change deletes this document.

## 1. The core's constructor

- **`Core::new(config, connectors, limits)`,** `connectors` a `&[jig_domain_contract::Declaration]`,
  the same as `Engine::new` takes, which `Engine::new` passes through; it checks and refuses as
  `vocabulary.md`, 2.1 says, with the same `jig_domain_link::Problem`.
- **The core keeps every declaration given,** for the life of the core, in the order given: its
  resource kinds installed in the tasks child (`../tasks/holds.md`, 2), its effect kinds for the
  descriptions it judges (3).
- **A test that builds a core** gives a declaration for each connector its configuration names,
  with the kinds the test relies on; one with none gives `Declaration::empty(connector)`,
  `pub fn empty(connector: u16) -> Declaration`, its two lists empty.

## 2. The tasks child's kinds

- **`jig_domain_tasks::Event::Kinds { connector: u16, kinds: Box<[ResourceKind]> }`,** the
  contract's `resources::ResourceKind`; `jig_domain_tasks::Kind` goes, since it repeated the
  connector the event names. The child's checks of the event stay as they are.
- **Old paths kept:** `jig_domain_tasks::HoldKind` names `jig_domain_contract::resources::Rule`, and
  `jig_domain_tasks::Taken` names `jig_domain_contract::resources::Taken`, by re-export.

## 3. Descriptions

- **`EffectDescription { connector, purpose, effect }`** once the description gives up its kind's
  fields: `form`, `recovery` and `waits_for_dependencies` go.
- **Its kind** is `effect.kind` of `effect.connector`, read by
  `Core::effect_kind(&self, connector: u16, kind: u16) -> Option<&EffectKind>`, `pub(crate)`: the
  declared kind of that number of that connector, or none.
- **Each root's declared effect kinds** are what its descriptions say of each kind before they give
  up the fields, but temper's forge's kind 4, a merge, which waits for dependencies as the forge's
  own table says (`temper-engine-domain-forge-vocabulary`, `waits_for_dependencies`), its landing's
  merges among them.
