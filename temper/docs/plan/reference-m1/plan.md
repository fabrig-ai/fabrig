# reference-m1: plan

Milestone M1 of `temper/docs/design/draft/reference-case.execution.md`: the goal's own
branch g (N3), `change`'s landing branch, dependencies and batch (N22 in part), the light
rules (N4), a result carrying its producer's report (N15), a charter's default wake policy
(N23), the landing rules as settings, and the reference's first editions. Written by the
cut as it releases; each batch brings it up to date.

## Families and parts

| Family | Parts | Design | Written by |
|---|---|---|---|
| bound | `bound` | jig's `domain/tasks.md`, 11 (already designed) | the lead |
| goal | `goal-jig`, `goal-forge` | jig's `protocol/configuration.md` 3.1, `domain/connectors.md` 3.3 and 4.6, `domain/tasks.md` 6.1 to 6.5, `domain/engine.md` 7.1; temper's `domain/forge.md` 2, 11, 11.1 | its designer |
| change | `change` | temper's `domain/agent.md` 4.3.1; `domain/forge.md` 8.1, 8.2, 8.7, 10 | its designer |
| settings | `settings-landing`, `settings-wake`, `settings-reference` | temper's `domain/root.md` 5.5, 5.6, 6, `domain/forge.md` 8.9, `protocol/configuration.md` 13.2, 13.3 (designed); jig's `domain/tasks.md` 8.3 and `protocol/configuration.md` 3; the reference's configuration | its designer |

The cross-family shapes are `notes/contracts.md` (C1 to C8); the standing rules,
`notes/common.md`.

## Tasks so far

Released (needs in brackets):

- bound: `bound-verdict-fields` [].
- goal-jig: `goal-jig-writers` [] (stories only); `goal-jig-surface` [] (the interface:
  `people::Symbol`, `Member.symbols`, `Template.resources`, the holdings asks' `tracked` and
  symbols, `HoldingsRefusal`, a taking's outbox entry); `goal-jig-holdings` [surface];
  `goal-jig-store` [surface, bound-verdict-fields]; `goal-jig-config` [surface];
  `goal-jig-client` [surface, config, store].
- change: `change-requires` [] (C5's `Requires`, `Domain::requires`); `change-held-release`
  [] (the held-then-released change left with no producer, still on main); `change-call` []
  (the call's parsed shape); `change-by-rule` [requires]; `change-batch` [call,
  held-release]; `change-words` [batch, by-rule, goal-forge-symbol]; `change-help`
  [words]; `change-goal-story` [words, goal-forge-symbol, goal-forge-branch,
  goal-forge-story].
- goal-forge: `goal-forge-branch` [goal-jig-surface, change-requires, change-held-release,
  change-by-rule, change-batch]; `goal-forge-symbol` [goal-jig-surface, goal-forge-branch];
  `goal-forge-story` [goal-jig-holdings, goal-forge-branch, goal-forge-symbol].

- settings-landing: `settings-landing-rules` [change-requires] (the forge's pure surface for
  kinds 1 and 2, `requires` over the held rules; ported from temper's old
  `configuring-stock/forge-kinds`).
- settings-wake: `settings-wake-pace` [goal-jig-store, goal-jig-client];
  `settings-wake-charter` [settings-wake-pace, goal-jig-holdings, goal-jig-config];
  `settings-wake-clients` [settings-wake-charter].
- settings-reference: `settings-pre-pr` [] (`.temper/pre-pr`: format and the old-form counts);
  `settings-reference-files` [settings-wake-clients, goal-jig-client, change-call,
  change-requires] (`temper/docs/examples/reference/settings.json` and `landing.json`, from
  `notes/settings.md`).

## Slices

1. bound; 2. goal-jig and goal-forge; 3. change; 4. settings-landing; 5. settings-wake and
settings-reference, **the walk** (`walk = true`): reference-case.md steps 1, 3, 7 and 19, then
the goal's end with g kept, unlanded.

## Left for later, and the user's questions

- **The rest of the landing rules as settings** (`[[question]] landing-settings-rest`): it
  needs jig's application settings (jig's `domain/engine.md`, 17.5 to 17.7), unbuilt, about
  ten tasks. M1 keeps the rules in the document's `application.landing`; the reference's
  `landing.json` carries main's rule and the goal branches' rule.
- **What `.temper/pre-pr` runs** (`[[question]] pre-pr-checks`): the first edition runs
  format and the old-form counts; lints and focused tests wait for the user's budget.
- **For the walk's document:** applying the reference's settings on the box needs
  `authority.executors` and `tasks.executor_kinds` at 8 (temper's `protocol/configuration.md`,
  13.4); temper-init's template and `docs/examples/owned.json` still give 6
  (`configuration.status.md`, Departs 13.4). The walk's document sets them.

## Order decided by the lead

One task writes a file at a time. The forge crate's `domain.rs` and `boundary.rs`, and the
root's `forge_route.rs`, are written by both the goal and the change families, so they go in
one line: `change-requires` and `change-held-release`, then `change-by-rule` and
`change-batch`, then `goal-forge-branch`, `goal-forge-symbol`, then `change-words`, then the
stories. `goal-jig-surface`'s compiler-forced field lines in the root's `asks.rs` and
`forge_route.rs` run beside `change-held-release`: field-only hunks, accepted.
`goal-jig-store` follows `bound-verdict-fields` (both in `jig-protocol-store`).

## Critical path, as it stands

`change-call` (75) -> `change-batch` (100) -> `goal-forge-branch` (110) ->
`goal-forge-symbol` (75) -> `change-words` (100) -> `change-goal-story` (90): about 550
implementer-minutes. The jig side (`goal-jig-surface` (75) -> `goal-jig-store` (60) -> `goal-jig-client` (90) ->
`settings-wake-pace` (120) -> `settings-wake-charter` (110) -> `settings-wake-clients` (100) ->
`settings-reference-files` (60)), about 615 implementer-minutes, is as long: the walk waits
for both lines.

## Overlaps with other plans

- reference-m7a changes `jig-domain-core`, `jig-domain-tasks` and `jig-protocol-store` on its
  trunk; `bound-verdict-fields` touches each in a few lines (`limits.rs`, `verdict.rs`,
  `activity.rs`, `codec/settings.rs`): unavoidable, since the bound lives there.
- reference-m7a: `goal-jig-surface` and `goal-jig-holdings` in `jig-domain-core`'s routing
  (field-only hunks in `run/delegation.rs`, `task/terminals.rs`); `goal-jig-client` in
  `jig-client-protocol-wire`.
- reference-m8: the forge crate (`domain.rs`, `boundary.rs`, `gates.rs`), the root's
  `forge_route.rs`, the forge-change crate, `store-v11.schema` and its goldens
  (`change-words`, a field on `ChangeRow`, in place), and `temper/docs/help/change.md`
  (`change-help`, against reference-m8's `help-pages`). Each spec keeps its hunks narrow.
- reference-m7a: the settings-wake tasks in `jig-domain-tasks`, `jig-domain-core`,
  `jig-protocol-store` and jig's client crates; reference-m8: `settings-wake-pace` in
  `temper-client-view` (`pages/chats.rs`), `settings-landing-rules` in the forge crate.
