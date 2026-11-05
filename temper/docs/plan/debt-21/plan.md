# debt-21: the twenty-first debt round

Six independent parts (`inputs/request.md`): smith's shell-cwd grader; jig's D004 record; skein D009, D020; jig D017; hearth D009. No temper client view, no `temper.css`, no `jig-client-view`. This file grows with each push.

## Parts and tasks

| Source | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| request | `shell-cwd-grader` | `grader-worst-case` | | high | 60 | `worst_case` back; a bundle's proof, or "not validated" in the status |
| request | `d004-record` | `d004-record` | | medium | 5 | one line of a debt record |
| skein D009 | `oauth-secrets` | `oauth-tokens` | | high | 90 | records and token answers on `Secret`, readers in every tree; **Measure** 19 |
| skein D009 | `oauth-secrets` | `oauth-exchange` | `oauth-tokens` | high | 60 | a sign-in's values on `Secret`; carriers print lengths; **Measure** 13 |
| skein D009 | `oauth-secrets` | `accounts-secrets` | `oauth-exchange` | high | 60 | accounts' credentials and its `Debug`s |
| skein D009 | `oauth-secrets` | `fake-oauth-secrets` | `oauth-exchange` | medium | 45 | the fake's credentials and its `Debug`s |
| skein D009 | `oauth-secrets` | `oauth-canary` | `accounts-secrets`, `fake-oauth-secrets` | medium | 30 | the canary's table; **Measure** 0; the item goes |
| skein D020 | `forge-states` | `hooked-states` | | high | 90 | `Hooked`'s four states and its `follow`; pins first |
| skein D020 | `forge-states` | `connection-states` | | high | 100 | `Connection`'s data-carrying `State`; `Phase` a computed tag |
| skein D020 | `forge-states` | `connection-deadlines` | `connection-states` | high | 60 | `until`, `follow`, `hold`, `rest`; fifteen sites become one; pins first |
| skein D020 | `forge-states` | `forge-states-close` | the two above | medium | 15 | status; the item goes; **Measure** 4 |
| hearth D009 | `hearth-kits` | `hearth-kits-lay` | | medium | 20 | three empty kits in the root manifest |
| hearth D009 | `hearth-kits` | `hearth-hostile` | | medium | 30 | `mutate` to `hearth-test-fixtures`; **Measure** 3 |
| hearth D009 | `hearth-kits` | `hearth-kit-render` | `hearth-kits-lay`, `hearth-hostile` | medium | 75 | goldens' readers, repositories, parent |
| hearth D009 | `hearth-kits` | `hearth-kit-repository` | `hearth-kits-lay` | high | 75 | a scratch repository and the binary's run; `hearth` off the builder world |
| hearth D009 | `hearth-kits` | `hearth-kit-git` | `hearth-kits-lay` | medium | 60 | git's answers and replay |
| hearth D009 | `hearth-kits` | `hearth-builder-switch` | `hearth-kit-render`, `hearth-kit-repository` | medium | 30 | the builder's edge goes; the gate reads goldens by path |
| hearth D009 | `hearth-kits` | `hearth-kits-close` | `hearth-hostile`, `hearth-kit-git`, `hearth-builder-switch` | medium | 10 | status; the item goes; **Measure** 0 |
| jig D017 | `narrowing` | `narrow-settings` | | high | 90 | 28 sites: a `Value` accessor, the offered lists, the small ones |
| jig D017 | `narrowing` | `narrow-application` | | high | 75 | 13 sites: `translate`, `named`, `Answered` |
| jig D017 | `narrowing` | `narrow-effects` | | high | 60 | 11 sites: a private `Making` |
| jig D017 | `narrowing` | `narrow-core-local` | | medium | 45 | 10 sites, each matched once |
| jig D017 | `narrowing` | `narrow-authority` | | medium | 20 | 4 sites: an array of pairs |
| jig D017 | `narrowing` | `narrow-tasks` | | high | 90 | 25 sites in `jig-domain-tasks` |
| jig D017 | `narrowing` | `narrow-engine-clients` | | high | 75 | 12 sites: `GetOwner`, `CarriedRequest` |
| jig D017 | `narrowing` | `narrow-notes-reads` | | high | 60 | 5 sites: `ReadPending` pairs its kind and stage |
| jig D017 | `narrowing` | `d017-record` | the eight above | medium | 15 | the item stays open and names what's left |

1,445 implementer-minutes in all.

## What was decided

- **`shell-cwd` stays at version 3.** `smith/docs/design/benchmarks.md`, 5.2 changes a task's version with its grader, so that results of different versions are never compared. No result exists at 3: version 3 came on 2026-11-04 and no bundle has run it. A bump would say something changed since a run that never happened.
- **The rebuild is timed.** The frozen source is in neither this repository nor the old mirror, so a digest match is a search among rewritten commits and may not exist. `grader-worst-case` has 40 minutes, then records "not validated".

- **skein D009's order is forced.** Public fields change type in one commit with all their readers, so `skein-oauth` goes first and alone. `state` and `nonce` are `Secret` too: `skein/docs/design/oauth.md`, 4 names them, and it lets `Redirect` and `Event` derive. Carriers stay byte boxes with a hand `Debug` (`skein/docs/design/lib.md`, 6). `Secret` gains no `From`: 54 test sites spell `Secret::new`, and lib's interface stays.
- **D020 keeps `installations.rs`'s `GitHub`.** The item's **Where** and the status name it, and its **Paid when** doesn't, so the status keeps that clause without the item. D016 has closed, so the item's "with D016" is stale. Its `Measure` reads 4 when paid: each `follow`'s line and two locals that use the word.
- **D020's pins come before its refactors.** Nothing pins several behaviours a state enum could silently change. `Hooked`'s deadline running from accept rather than from each state is the worst, since its test runs everything from t=0.
- **hearth D009's kits go in `hearth/testing/`, named as `hearth/docs/design/domain/testing.md`, 2.9 now has them.** This cut wrote that section's rows and its status entry. `properties::run` makes the render kit take the scripted parent, so it's larger than the item guessed.
- **jig D017 stays open.** Its **Paid when** can't hold in this plan: the request keeps `jig-client-view` off limits, and four of its seven sites re-narrow. Hub-link's payload map, notes' `Rows`, people's `Update` and others need a public type's change, which the request makes a question, and `Held { why, source }` is stored. The surveys found 218 re-narrows of 382 sites, and the plan takes the 108 that pay best and need no question, from 382 to about 274. `d017-record` writes what's left, so the next round starts from it. The cut chose by sites per changed line, kept off other drafts' files, and took `narrow-notes-reads` despite its ratio, since it's the item's named example. Left for size: the store, link, fleet, host and client clusters.
- **`jig-domain-core` has four writers at once, on disjoint files.** None changes an interface another reads, since each stays `pub(crate)` or private.

## Overlaps

- D009 edits lines in `temper/crates/temper-service/src/clients/google.rs`, `temper/crates/temper-service/src/tests/accounts_host.rs` and `temper/crates/temper-fake/src/oauth.rs`; `ui-welcome/setup-intent` holds `temper/crates/temper-service/src/tests/**`. No other draft names the first or the last. They are one-line edits, and fab's merges carry them.
- `hearth-kit-git` moves `hearth/tests/git/src/world.rs`, which hearth's memory-model work changed today. The task leaves every queue form as it stands, and that work stays off the rest: `hearth-service-builder/src/lib.rs` and the machines' `MAX_OUT` forms are in no spec.
- jig D017 keeps off `reads.rs`, `reads/rows.rs`, `reads/settings.rs`, `routing/effect/note.rs` and notes' `domain.rs` and `boundary.rs`, which `ui-settings`, `ui-settings-tabs`, `note-parts` and `ui-chat-draft` change. `narrow-notes-reads` holds `read.rs`, which none of them does.

## Critical path

`oauth-tokens`, `oauth-exchange`, `accounts-secrets`, `oauth-canary`, 240 minutes. `connection-states`, `connection-deadlines`, `forge-states-close`, 175. `hearth-hostile` or `hearth-kits-lay`, `hearth-kit-render`, `hearth-builder-switch`, `hearth-kits-close`, 145. `narrow-settings` or `narrow-tasks`, `d017-record`, 105.
