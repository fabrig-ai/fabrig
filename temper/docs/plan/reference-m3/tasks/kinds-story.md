+++
id = "kinds-story"
part = "kinds-root"
kind = "feature"
needs = ["kinds-document", "change-story", "apps-proposals"]
files = ["temper/tests/engine/domain/tests/suite/settings/kinds.rs", "temper/tests/engine/domain/tests/suite/settings.rs", "temper/tests/engine/domain/src/kinds.rs", "temper/tests/engine/domain/src/lib.rs", "temper/docs/design/domain/forge.status.md", "temper/docs/design/domain/root.status.md"]
effort = "high"
minutes = 120
+++

# kinds-story: an engine world story of temper's kinds 1 to 3

## Goal

The domain world tells, end to end through the root, jig's core and the forge, what M3 built
(`configuration.md`, 13.2 and 13.3; `forge.md`, 8.7, 8.9 and section 16): a change of landing
rules judged, kept, and read at the next merge; a refused rule; a proposal kept and accepted;
a restart that holds the rules and finds the stock unchanged; a project deleted with its
rules. The forge world's part of section 18 and the last of the status entries close.

Acceptance:

- A world helper module `temper/tests/engine/domain/src/kinds.rs` (`pub mod kinds;` in the
  world's `lib.rs`): a driver built as the binary builds its configuration for kinds
  (`core.kinds` from `temper_service::stock::kinds`, `kinds_stock` from
  `temper_service::stock::forge`), and a party's change or proposal of settings naming items
  of kinds 1 to 3, its `KindValue`s decoded as the entrance decodes them
  (`temper_engine_smith::kinds::decode`), so a story never builds the root's typed values by
  hand.
- Stories in `suite/settings/kinds.rs` (`mod kinds;` in `suite/settings.rs`), each one thread,
  one loop, a seed that replays:
  - `a_landing_rule_changed_reaches_the_next_merge`: an adopted repository whose goal branch
    a change lands into; the deployment's owners set kind 2 rule 1 for the project (the goal
    branches, open `temper`, enforced, asking neither CI nor up to date); the change's next
    merge goes ahead without CI (requirements 5 and 6 met), where before it waited for CI.
  - `a_rule_out_of_bounds_is_refused_naming_what_fails`: a rule with a gate twice: the change
    of settings refused whole, the refusal carrying the forge's words for `Gates` and naming
    kind 1, scope 0, number 2; nothing kept.
  - `a_proposal_of_rules_is_kept_then_accepted`: a proposal naming kind 1 rule 2 is kept by
    the forge (its row under the proposal's key), accepted by an owner, and its value becomes
    the change; a second proposal withdrawn leaves no row.
  - `a_restart_holds_the_rules_and_finds_the_stock_unchanged`: after a kept change, a restart:
    the forge holds the rule before any change steps, the core is told no default differs,
    and no revision is made.
  - `the_forges_charter_change_reaches_the_next_change`: kind 3 number 1 set to another
    offered charter: the next change's producer runs under it; a value naming an unoffered
    charter is refused by jig's reference check, naming kind 3.
  - `a_deleted_project_loses_its_rules`: a project with kind 2 rules deleted: its rows erased,
    the deployment's kept.
- Status: `forge.status.md` loses the entry "8.7 to 8.9, 16, 18: the landing rules and the
  forge's charter as settings of temper's kinds; ..." whole (8.8's own entry stays);
  `root.status.md` loses what is left of "5.5, 5.6, 6, 9.1: settings of temper's kinds ..."
  (9.1's `temper upgrade --check` and the other sections' entries stay).
- The stories pass: `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
  'test(/settings::kinds::/)'` (check the count: six), then the world's whole suite.

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.2, 13.3, 13.4 and 13.6 step 2;
  `temper/docs/design/domain/forge.md`, 8.7, 8.9, section 16 and section 18;
  `temper/docs/design/domain/root.md`, 5.6 and 9.1; `temper/docs/design/domain/testing.md`
  (the domain world's rules).
- `temper/docs/plan/reference-m3/notes/common.md`; `notes/contracts.md`, C1 to C3.

## Owns and may change

- Owns the files in the header. Uses the world's drivers and the forge routes' helpers
  (`src/forge_routes.rs`, `change-story`'s) without changing them; a helper they lack goes in
  `src/kinds.rs`.
- Options: if a story finds the code departs from the design, keep the story as the design
  says, mark it `#[ignore = "..."]` naming the departure, and report it (never change the
  design to fit). If a story needs more than the world's limits, raise them in its own
  driver, never the world's defaults.

## Steps

1. The helper module.
2. The stories, one at a time, each narrow run's count checked.
3. The status lines.
4. fmt, old forms, clippy on the world; its suite.
5. Commit ("Tell temper's kinds end to end in the domain world") and push.

## Report

The implementer's text's report, plus: any story ignored, and why.
