+++
id = "away-walk"
part = "away-walk"
kind = "feature"
needs = ["away-route", "decided-tool", "home-decided"]
files = [
  "temper/tests/client/tests/suite/away.rs",
  "temper/tests/client/tests/suite/main.rs",
  "temper/tests/client/src/front/away.rs",
  "temper/tests/client/src/front.rs",
]
effort = "high"
minutes = 60
+++

# The meeting point: the switch and a decision cross the real roots

## Goal

One story through both real sides: the client's act, the engine's root and forge, the client's facts. `temper/docs/design/client/README.md`, 3.5 (*Away*), 4.6; `temper/docs/design/domain/forge.md`, 5.8 and 5.9. Shaped as `temper/tests/client/tests/suite/status.rs`.

Acceptance, each a story in `tests/suite/away.rs`, on the world's fake clock:

- `on_crosses_the_roots_and_reaches_every_device`: `Act::Away(Toggle)` on one client; a second device's `Domain::away` reads the row; a store's row holds it, restored after a restart.
- `off_keeps_the_choices`: off, `kinds` and `until` stand, and on again is a new spell.
- `a_goal_is_answered_by_the_switch`: a goal's `decided`, through the root's host call, answers "Decided: ..." with the switch on, and Home's `Facts.away` shows the row; off, it answers "Ask: ...".
- `past_until_the_client_draws_off_and_the_root_says_ask`: the clock past `until`; no write, `away_on` false, and the call answers ask.
- `another_partys_home_shows_nothing`: a second person's Home has no form, and their `Overturn` is refused unknown.

Each story must bite, as the implementer's text says.

## Inputs

- `temper/docs/plan/ui-away/notes/common.md`, `notes/contracts.md`.
- `temper/tests/client/tests/suite/status.rs`, `dismissed.rs`; `temper/tests/client/src/front/status.rs`; `temper/tests/engine/domain/tests/suite/forge_routes/decided.rs`, from `decided-tool`.

## Owns and may change

Owns, as `files` lists them. May also change: `temper/tests/client/src/component.rs`, `app.rs` and `fixtures.rs`, for what the world lacks.

Options, each pre-approved:

- **The host call:** if this world can't run an agent's host call, deliver it as `temper/tests/engine/domain/src/forge_routes.rs` does, to the same root. If that can't be reached from here either, the story saves the row by `forge::Event::Decided` on the root, and your report says the tool's path stays with `decided-tool`'s tests and `away-proof`'s walk.
- **A fault this story finds in another task's code:** fix it in the lowest tier that shows it, with its test there, and say so: that crate is yours for that fix.

## Steps

1. `src/front/away.rs`, what the stories drive.
2. The stories, one at a time, `heavy cargo nextest run -p temper-client-world --test suite -E 'test(away::)'`.
3. The check of the implementer's text. Push.

## Report

As `notes/common.md`, Your report: each story's bite.
