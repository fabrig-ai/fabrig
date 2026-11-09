+++
id = "tabs-surface"
part = "tabs"
kind = "interface"
needs = []
files = ["temper/crates/temper-client-domain/src/pages/rail.rs", "temper/crates/temper-client-domain/src/tests/pages/rail.rs", "temper/tests/client/tests/suite/fuzzy/rail_and_work.rs"]
effort = "high"
minutes = 60
+++

# tabs-surface: the phone's tabs, one pure function

## Goal

The phone's tab bar gets its rule in the domain, as the rail's parts have
theirs: `rail::Tabs` and `rail::tabs(&Parts) -> Tabs`, which the browser's
tab bar will draw from (the task `phone-tabs`), held by a table test and by
rows of the existing fuzz test of the rail's parts and their settings moves.
Nothing draws from it yet.

Acceptance:

- `temper_client_domain::pages::rail::{Tabs, tabs}` exist exactly as
  `temper/docs/plan/ui-p5a-phone/notes/contracts.md`, *The phone's tabs*,
  has them, with their rules.
- The domain's rail table test holds `tabs` on the rows it already has and
  on the To begin rows: four tabs with Ask, three without, To begin's Home,
  Ask and Deployment, and Home and Deployment alone without Ask.
- The fuzz test `parts_and_live_work_survive_random_settings_moves`
  (`temper/tests/client/tests/suite/fuzzy/rail_and_work.rs`) checks, on
  every seed and both settings revisions, `rail::tabs` of the parts against
  its own oracle written from the design's table (not from `tabs`), and that
  a settings move keeps the tabs but for nothing (the tabs read no setting
  but To begin's cut and Ask temper's template, which the sweep's moves
  don't change: assert they are equal across the move).
- Each new check bites: break `tabs` once (say `deployment = parts.work`)
  and see both tests fail for that reason; restore.

## Inputs

- `temper/docs/design/client/README.md`, 5.3, *The rail's parts* and *On a
  phone* (its table of tabs).
- `temper/docs/design/draft/surfaces.md`, 10 and 12.
- `temper/docs/plan/ui-p5a-phone/notes/contracts.md`, *The phone's tabs*.
- `temper/docs/plan/ui-p5a-phone/notes/common.md`.

## Owns and may change

- `temper/crates/temper-client-domain/src/pages/rail.rs`: `Tabs` and `tabs`
  after `parts`, the module comment citing 5.3's *On a phone* as well.
- `temper/crates/temper-client-domain/src/tests/pages/rail.rs`: its table of
  `parts` gains the expected `Tabs` per row, or a sibling table beside it;
  no new test function where a column does.
- `temper/tests/client/tests/suite/fuzzy/rail_and_work.rs`: `check_rail`
  also checks `tabs` against a `tabs_oracle(&rail::Facts)`; the move
  assertion in `sweep` covers the tabs. No new `#[test]`.
- Nothing else. If `tabs` would need a fact `Parts` lacks, stop and ask:
  the contract says it doesn't.

## Steps

1. Add `Tabs` and `tabs` to `rail.rs`, its doc comments in the design's
   words.
2. Extend the domain table:
   `heavy cargo nextest run -p temper-client-domain --lib -E 'test(/rail/)'`.
3. Extend the fuzz:
   `heavy cargo nextest run -p temper-client-world --test suite -E 'test(=fuzzy::rail_and_work::parts_and_live_work_survive_random_settings_moves)'`
   in the default profile, then once with `--profile fuzzy`. Report its CPU
   seconds before and after (`/usr/bin/time` around the run, or nextest's
   own times); it must stay a pure sweep with no root built.
4. Break each check once, as the acceptance says.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-client-domain -p temper-client-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-client-domain`; and
   `heavy cargo check -p temper-client-view -p temper-client-tui -p temper-client-shell-tui --tests`
   for the new public items' readers.
6. Commit ("ui-p5a-phone: the phone's tabs, one pure function over the
   rail's parts") and push.

## Report

As the implementer's text says, with the rule broken for each check and the
failure seen, and the fuzz test's CPU seconds before and after.
