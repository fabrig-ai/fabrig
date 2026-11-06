# ui-walk-large: the plan

The first whole walk's two large findings, W3 and W6 (`inputs/request.md`; `temper/docs/plan/ui-final-walk/findings.md` on `origin/ui-final-walk/integration`).

## Parts and tasks

| Part | Task | Needs | Minutes |
|---|---|---|---|
| `w6-room` | `stock-room`: the stock's period is one chat's 24K beyond the pool | | 45 |
| `w3-engine` | `proposal-projects`: `projects` on a settings proposal's row and document, the filter, the wire, the recordings | | 120 |
| `w6-home` | `home-true`: a brief new since the last look shows, "The last change landed SPAN ago.", `forge.sitting_ms`, the plan's walk | `stock-room` | 75 |
| `w3-needs` | `needs-proposal`: the project's Needs you, the rail, Accept, the card's title, the fake model's cue, the browser walk, the reprint | `proposal-projects`, `home-true` | 90 |

Four tasks, as the request asks at most. Each finding's engine half goes first, with no wait for its client half. Shapes that cross tasks are in `notes/contracts.md`.

`needs-proposal` waits for `home-true` because of one writer per crate: both write `temper-client-view`, including `words.rs`, and its browser walk takes in W6's return.

## Decisions

- **W6's room is outside the pool.** The stock's period is 81 × 24K and the pools stay 80 × 24K. The request's other option, a smaller pool, is the same one line in `stock.rs`, but it makes "eighty chats funded at once" false in the design, `docs/help/budget.md` and two tests, and takes something from a person. `limits.rs` holds nothing on spend: the pool, period and K are all `stock.rs`'s, so nothing moves there for this. The room is 24K, not K, because a run's spend stays spent: at K a project would see one brief in its period, and at 24K at least twenty-four, one a sitting. It funds the forge's own repairs too, which were refused the same way.
- **A settings proposal keeps `project` 0 and its holder,** and gains `projects`. One number can't say "several", and Accept is decided in the deployment's scope either way.
- **`projects` comes from the items, in the core,** worked out when asked, not stored. A proposal's items never change once made, so that is the value set when it is made, with no stored format to move. A client never works it out.
- **A set of a project's items and the deployment's belongs to that project alone.** The request says a proposal belongs to the project its items touch; "deployment-wide" is for one that touches none.
- **W6's walk would have stayed silent with money alone.** The walk opened Home after the landing, so at its return `landed` equals `seen` and a brief covering it fails "`landed` above `seen`". The run would spend and show nothing. A brief written since the last look now counts as new, with no engine or wire change. When nothing new landed and no brief shows, the headline says "The last change landed SPAN ago.", so a landing never stands under "Nothing needs you." alone. That sentence is true on a refused, failed or unfunded run, and Home has no word for those.
- **The sitting becomes a limit,** `forge.sitting_ms`, still 30 minutes in both profiles. A real box has no fake clock, and the design's status held "land, come back past 30 minutes, see the words" as never walked. A short override in a test's document lets a real fake box do it, with the plan account. No test budget or timeout moves.
- **The status run's payer is the project.** A plan serves it while its holder is the project's only member, which holds on a person's fresh box. `home-true` asserts that; a shared project's plan doesn't serve it and its Home has the computed words.
- **A member's pool can take the room.** A member's first spend reserves 24K from the period, the same 24K. The brief then falls back to the sentence. Accepted: a shared project's owners can fund, and `forge.md`, 5.7, says so.
- **Who may decide is unchanged.** `jig/docs/design/domain/engine.status.md`, 17.6, has a settings proposal decided by a deployment's owner, not a project's. Rows go to those who can decide, so a project owner who doesn't own the deployment doesn't see one. The fresh box's person owns both. Closing that departure is a change of who may accept, outside this request.

## Formats and protocols

There is no supported deployment, so these change with no compatibility: the client wire, v19 in place (`InboxEntry`, `Proposal`), with the four journeys re-recorded in `proposal-projects`. The stock's defaults change in place, and `forge.sitting_ms` is a new public limit. No stored format moves.

## Overlaps

- `ui-final-walk` (integration): `fix-flow` owns `temper-client-domain/src/**`, `temper-client-shell-web/src/**` and `temper-fake/src/**`, and `walk-again` follows it. `needs-proposal` must write `temper-client-domain/src/pages/{inbox,needs}.rs` and a cue in `temper-fake/src/backends.rs`, so it goes last, after `home-true`. Its files can't be avoided: W3's reads live there. The other tasks stay off them but for a field's line in tests. Expect a catch-up.
- `ui-final-walk` may re-record journeys too: on a catch-up, re-record rather than hand-merge.
- `box-link-first` owns `temper/crates/temper/tests/process.rs` and support; `stock-room` and `home-true` only edit `fake_backends/walk/status.rs`.
- `debt-25` has `temper/tests/conformance/src/**` lines; this plan has none there. `proposal-projects`' `jig/testing/jig-client-scripted-engine/src/tests.rs` is `debt-25/numbers-client`'s too: one field's line, the overlap that can't be avoided, since the field is required.
- **Journeys' prints** are written once by `proposal-projects` and once by `needs-proposal`. `home-true` leaves them stale, so the two parallel branches never fight over them: a stated departure from the request's "yours to regenerate".

## Critical path

`proposal-projects` (120) then `needs-proposal` (90), 210 minutes, with `stock-room` (45) then `home-true` (75) reaching `needs-proposal` at 120, just in time. The whole plan: about 3.5 hours of slots' wall time.
