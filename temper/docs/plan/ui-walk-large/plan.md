# ui-walk-large: the plan

The first whole walk's two large findings, W3 and W6 (`inputs/request.md`; `temper/docs/plan/ui-final-walk/findings.md` on `origin/ui-final-walk/integration`).

## Parts and tasks

| Part | Task | Needs | Minutes |
|---|---|---|---|
| `w6-room` | `stock-room`: the stock's period is one chat's 24K beyond the pool | | 45 |
| `w3-engine` | `proposal-projects`: `projects` on a settings proposal's row and document, the filter, the wire, the recordings | | 120 |

More to come in this cut: `home-true` (W6's words and its personal-account check) and `needs-proposal` (W3's client and browser walk).

## Decisions

- **W6's room is outside the pool.** The stock's period is 81 × 24K and the pools stay 80 × 24K. The request's other option, a smaller pool, is the same one line in `stock.rs`, but it makes "eighty chats funded at once" false in the design, `docs/help/budget.md` and two tests, and takes something from a person. `limits.rs` holds nothing on spend: the pool, period and K are all `stock.rs`'s, so nothing moves there for this. The room is 24K, not K, because a run's spend stays spent: at K a project would see one brief in its period, and at 24K at least twenty-four, one a sitting. It funds the forge's own repairs too, which were refused the same way.
- **A settings proposal keeps `project` 0 and its holder,** and gains `projects`. One number can't say "several", and Accept is decided in the deployment's scope either way.
- **`projects` comes from the items, in the core,** worked out when asked, not stored. A proposal's items never change once made, so that is the value set when it is made, with no stored format to move. A client never works it out.
- **A set of a project's items and the deployment's belongs to that project alone.** The request says a proposal belongs to the project its items touch; "deployment-wide" is for one that touches none.
- **Who may decide is unchanged.** `jig/docs/design/domain/engine.status.md`, 17.6, has a settings proposal decided by a deployment's owner, not a project's. Rows go to those who can decide, so a project owner who doesn't own the deployment doesn't see one. The fresh box's person owns both. Closing that departure is a change of who may accept, outside this request.

## Formats and protocols

There is no supported deployment, so these change with no compatibility: the client wire, v19 in place (`InboxEntry`, `Proposal`). The stock's defaults change in place too.

## Overlaps

- `ui-final-walk` (integration): `fix-flow` owns `temper-client-domain/src/**`, `temper-client-shell-web/src/**` and `temper-fake/src/**`, and `walk-again` follows it. The two tasks so far stay off those but for a field's line in `temper-client-domain`'s tests. Expect a catch-up.
- `ui-final-walk` may re-record journeys too: on a catch-up, re-record rather than hand-merge.
- `box-link-first` owns `temper/crates/temper/tests/process.rs` and support; `stock-room` only edits `fake_backends/walk/status.rs`.

## Critical path

So far: `proposal-projects`, 120 minutes.
