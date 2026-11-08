+++
id = "domain-settings"
part = "domain-runs"
kind = "migration"
needs = []
files = ["temper/tests/engine/domain/tests/suite/settings.rs", "temper/tests/engine/domain/tests/suite/settings/**", "temper/tests/engine/domain/tests/suite/forge_routes/**", "temper/tests/engine/domain/src/**"]
effort = "high"
minutes = 180
+++

# domain-settings: the engine's settings and approval stories leave charter 3

## Goal

The engine's domain world's settings stories (`suite::settings`) and the box's approval
stories (`suite::forge_routes::approval_identity`) stop running stock charter 3: what
they check of Ask temper runs stock charter 11; what they check of the settings
family's own tools, which no stock charter will be declared, runs a charter of the
world's own; the world's stock-sized limits stop counting charter 3. Runs beside
`domain-configuring`, which owns `suite::configuring`.

Acceptance:

- No `charter: 3`, `Charter(3)`, `number == 3` or `template 3` naming the configuring
  chat or charter, and no "configuring" in a name or a comment for it, in your files;
  a number 3 that is a negative input or an arbitrary fixture (`settings/kinds.rs`'s
  unoffered producer, ~388) stays.
- `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(/^settings::/) | test(/^forge_routes::/)'`
  passes with the same count as the base; its CPU time no more than 10% above the
  base's (report both).

## Inputs

- `temper/docs/plan/ui-ask-drop/notes/common.md` and `notes/contracts.md` (binding:
  *What stays*, *What moves to charter 11*).
- `temper/docs/design/domain/testing.md`, 6.4: the row "configuring by Ask temper" (the
  settings family's own tools on a world's charter; a rule refused at acceptance) and
  the row of a box's approval identities.
- `temper/docs/design/domain/agent.md`, 4.3 (which tools a charter is declared) and
  8.4.2 (approvals judged at an asking call).

## Owns and may change

- Owns `temper/tests/engine/domain/tests/suite/settings.rs`, `suite/settings/**`,
  `suite/forge_routes/**` and `temper/tests/engine/domain/src/**`.
- Not yours: `suite/configuring.rs` and `suite/configuring/**` (`domain-configuring`).
  A `src/` helper `suite::configuring` calls keeps its signature; if you would change
  one, add a new one beside it instead.
- Never `temper/crates/**` (charter 3 stays in the stock until `stock`).
- Options, pre-approved:
  - **The world's settings charter** is numbered 12, template 12 named `settings`,
    `tools` 1,920, instructions `b"Read and propose settings."`, offered by the rules'
    and policy 1's delegation (`notes/contracts.md`). `settings.rs`'s
    `settings_agent_config` builds its own defaults: its charter and template 3 become
    12, named `SETTINGS_CHARTER`/`SETTINGS_TEMPLATE` there, its mask (1,536) kept.
  - **`settings/b2.rs`:** the asking run (~385) delegates to stock charter 11 and drops
    the conversion of charter 3's tools to 3,072 (~370); the settings run (~441), which
    checks a non-asking charter holding the settings family is declared `settings` and
    `propose_settings` while the chat is not, delegates to the world's settings charter
    12, added to that story's config as above.
  - **`forge_routes/approval_identity.rs`:** `configuring_proposal` becomes
    `asking_proposal`, its chat run by charter 11. A story that observes a rule refused
    at **acceptance** (`a_box_refuses_an_unfulfillable_person_approval_rule`, and the
    early cases of `stock_adoption_seeds_safe_human_approval_roles` that expect a
    recorded proposal before eligibility) keeps that observation: default, its proposal
    is made by the world's settings charter 12, added to the forge-routes world's
    defaults in `src/forge_routes.rs`; if a party's own `settings propose` (`pN/K`)
    reaches acceptance in that world with less code, use it and say so. Where an asking
    call now refuses earlier, add that as one row of the same story, not a new test.
  - **`src/forge_routes.rs`'s `STOCK_CHARTERS` (5) and `STOCK_EXECUTORS` (8)** lag the
    stock today; size them from the supplied defaults (`config.core.defaults.charters.len()`
    and the ceiling's kinds) plus the world's own, or set them to hold today's stock and
    the world's charters, whichever is one line; the comment says what they count, with
    no "configuring".
  - **`settings.rs` ~285's stock-sized limits** (8 charters, 13 executor kinds) keep room
    for today's stock and the world's charters; only their comment changes.

## Steps

1. Write the world's settings charter where the options put it, then move each story
   by the options, running it alone after each move
   (`heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(=settings::NAME)'`).
2. Names: a test or helper named for the configuring chat is named for what it now runs
   (`asking_proposal`, `settings_charter_...`); a test name a design or status cites
   (`rg` it in `temper/docs/design/`) keeps its name, and the report lists it.
3. Once: `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D warnings`;
   the acceptance's run; `python3 -I -B temper/scripts/old_forms.py`. Moved stories need
   no bite check; a row you add does (break the refusal it checks by hand once, see it
   fail, restore).
4. Commit ("Run the engine's settings stories without charter 3") and push.

## Report

Each story moved, with what it runs now and its name; each option taken; the test
count and CPU seconds before and after; anything `suite::configuring` will see changed
in `src/`.
