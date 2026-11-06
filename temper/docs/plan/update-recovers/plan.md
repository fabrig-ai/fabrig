# update-recovers: a lost answer to an update doesn't hold a change for good

Request: `inputs/request.md`. Design: `skein/docs/design/forge-client.md`, 7.3 and 7.5; `jig/docs/design/domain/connectors.md`, 4.3; `temper/docs/design/domain/forge.md`, 6 and 10; `temper/docs/design/domain/landing.md`, 5.2. Shapes that cross tasks: `notes/contracts.md`.

## What was found, from reading

The cut runs nothing. `update-class` reproduces the request's reading first, in `skein-forge-client`, and says if it's wrong.

- **The failing story is a rebase.** `a_turn_landing_is_found_after_a_runner_cancellation_and_restarts` lands by turn, so its update is a prepare, `Write::Update { style: Rebase }`. Moving only the merge style would not cure it.
- **Two more ways a slow machine holds a change, past the one the request names:**
  - A retried update's check reads its pull again. By then a base gone on, or a late copy's own move, is `Failed(Stale)`, and that reached `Hold::Failed` too. On a busy repository a base moving in 11 s is ordinary.
  - `release_change` keeps `row.effect`, so a person's release of `Hold::Failed` from `Updating` or `Preparing` holds again at once. Released, it never updated anything.

## What was decided

- **The class is idempotent, for both styles.** Not keyed: no forge keeps a key. Not conditional: Forgejo's update checks no head, and another hand can erase what it leads to. An update has no value a copy could write over a newer one, so every copy leaves the head holding its base, or changes nothing.
- **`connectors.md`, 4.3 widened, a change in jig's design.** It said every class rests on the lifetime and an unbounded system has only unrecoverable, and the request's own reading is that an update can outlast its lifetime. An idempotent transition is now named as the one class that rests on no lifetime, its world showing a copy past one.
- **A late copy can do worse than a CI run, and this is the nearest thing.**
  - By a merge: forward only, refused at its push if the head moved. A CI run, and a hold if the change has moved on (below).
  - By a rebase: the push is forced and names no head, so a copy can drop a push made since it read the head. It takes a copy outliving its own retry and a later push. The change holds `BranchMoved`, and a merge names its exact head, so nothing unchecked lands. A repair so dropped finds its failure again when the head is checked.
  - Holding rebases instead would leave the failing story failing.
- **A late copy that moves the head once the change has moved on holds it** (`forge.md`, 8.5): a person, for what the change can't tell from another's move. Kept as an open question, `forge.md`, 20, for measuring after landing.
- **Item 1: a change let go decides again.** `Failed(Timeout | Unavailable | Stale)` on an update reports no result; the change reads afresh and updates again, counted in `updates`, stock 3. At the bound, `Hold::Updates`, in the last write's account. So neither a spin nor a silent stop. Other refusals hold `Failed` as now. A release now decides again.
- **Item 2:** nothing to clear. A held update stored keeps its hold and its `again` question; `Remake` reads no class. `a_held_update_remade_is_looked_for_first` pins it.
- **Item 3:** `forge.client.lifetime_ms` stays; see the report for what a short one costs.
- **No stored format or protocol change.** `EffectResult` could not gain a variant without one, which is why a let-go update is `None` with its account kept.

## Parts and tasks

| Part | Task | Needs |
|---|---|---|
| class | `update-class` | |
| machine | `change-decides-again` | |
| machine | `update-let-go` | `change-decides-again` |
| machine | `update-hold-words` | `update-class`, `update-let-go` |
| worlds | `update-worlds` | `update-class`, `update-let-go` |

## Critical path

`change-decides-again`, `update-let-go`, `update-worlds` (about 195 minutes). `update-class` runs beside the first two, and `update-hold-words` beside the last.

## After landing

temper/ui re-runs its three-thread trial, which is the proof. Not recorded here, and to record then: `style=merge` of a head already holding its base, and what a rebase's push does to a push made while it works (`skein/docs/design/forge.md`, 9). The plan relies on the first only for a copy landing between a check and a send, where a refusal is typed. The second is taken at its worst.

## Overlaps not avoided

- `skein/tests/forge/client/src/world.rs` with `debt-29/fake-config`: new helpers go in `world/recovery.rs`.
- `temper-engine-domain-forge/src/domain.rs` and `temper-engine-domain/src/route/forge_route.rs` with `landing-fixes-2/saved-close` and `repair-cause`: kept to named functions.
- `temper/docs/design/domain/forge.status.md` with `landing-fixes-2`, and `skein/docs/design/forge.md` with `debt-29/first-demand`: separate lines.
