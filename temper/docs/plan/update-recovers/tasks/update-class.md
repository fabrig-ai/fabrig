+++
id = "update-class"
part = "class"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-client/src/boundary.rs", "skein/crates/skein-forge-client/src/tests/recovery.rs", "skein/crates/skein-forge-client/src/tests/outbox.rs", "skein/tests/forge/client/src/world/recovery.rs", "skein/tests/forge/client/tests/fixtures/mod.rs", "skein/tests/forge/client/tests/suite/recovery.rs", "skein/tests/forge/client/tests/suite/fuzzy/simulation.rs", "skein/docs/design/forge-client.status.md"]
effort = "high"
minutes = 90
+++

# A lost update is made again, not held

## Goal

Reproduce the request's reading, then move the class. Today an update whose answer is lost and which isn't found after its lifetime is `Outcome::Held`, and the client never sends it again. It becomes `Recovery::Idempotent`: checked and made again, up to `write_attempts`, then `Failed(Timeout)`. The world shows the class holds for both styles, with a copy past its lifetime.

Acceptance, each test seen failing first, for its rule's reason, unless it says otherwise:

1. Step tests in `skein/crates/skein-forge-client/src/tests/recovery.rs`, each over `api::UpdateStyle::Merge` and `Rebase`:
   - `a_lost_update_not_found_is_checked_and_made_again`: a send answered `Timeout`, a find whose comparison isn't empty, the wait to the deadline, a second find the same; then a `Read::Pull` check and a second send, under a new attempt whose deadline is from the new send. `h.writes` is 2. No `Outcome::Held`.
   - `a_lost_update_fails_timeout_after_write_attempts`: every send lost and not found. `h.writes == LIMITS.write_attempts`; the last outcome `Failed(Error::Timeout)`; the entry gone from the store.
   - `an_update_made_again_is_found_by_a_later_look`: the second send lost, its find's comparison empty: `Made { made: Made::Updated(head), found: true }`, `h.writes` 2.
   - `an_update_checked_again_fails_stale_when_its_head_or_base_moved`: the check after a not-found reads another head, then in another case another `base_commit`: `Failed(Error::Stale)`, `h.writes` 1.
   - `a_held_update_remade_is_looked_for_first`: an entry with a started attempt handed in by `Event::Remake`, as a store holding a held update would: found if its comparison is empty; otherwise made again once its deadline has passed. This one is expected to pass today, and is the plan's item 2.
2. `skein/crates/skein-forge-client/src/tests/outbox.rs`, the class assertion on `Write::Update` says `Recovery::Idempotent`. Every test of an unrecoverable write stays as it is, `Outcome::Held` and all.
3. `recovery` in `skein/crates/skein-forge-client/src/boundary.rs` gives `Write::Update { .. }` `Recovery::Idempotent`, its comment saying why in the design's words (`forge-client.md`, 7.3).
4. Stories in `skein/tests/forge/client/tests/suite/recovery.rs`, over both styles, on a pull whose head is behind its base:
   - `a_lost_update_is_found_if_its_copy_lands_within_its_lifetime`: `land_writes_late` within the lifetime: `Made { found: true }`, `stats().writes` 1, the head holding the base. Expected to pass today.
   - `a_lost_update_is_made_again_if_no_copy_lands_within_its_lifetime`: the first send's copy lands past the lifetime, the second is answered: `stats().writes` 2, `Made { found: false }`. Run past the copy's landing: `late_landings` 1 and the head unchanged, still holding the base.
   - `a_copy_past_its_lifetime_after_the_base_went_on_moves_the_head_and_keeps_the_base`: as above, the base pushed by an outside hand before the copy lands. The head moves and holds both tips. By a merge the old head is its ancestor; by a rebase it is not. This names the cost in `forge-client.md`, 7.3.
   - `a_lost_update_fails_timeout_when_no_copy_lands_in_its_attempts`: `stats().writes` is `write_attempts`, the outcome `Failed(Timeout)`, no `Held`.
5. One fuzzy story in `skein/tests/forge/client/tests/suite/fuzzy/simulation.rs`, shaped as `unrecoverable_branch_creation_survives_cuts_before_and_after_its_late_copy`: `an_update_survives_cuts_before_and_after_its_late_copy`, both styles, a restart at each cut. Whichever copy lands the head holds the base at the end, or the entry ended `Failed(Timeout)`, and it is never `Held`.
6. `temper_engine_domain_forge::effect_shape(11)` and `temper/tests/conformance/src/observations.rs`'s row for kind 11 say idempotent, and `temper-engine-domain-forge`'s test comparing the two classes passes.
7. `skein/docs/design/forge-client.status.md` says `No gap.`, and the bullet naming `update-class` is gone from `temper/docs/design/domain/forge.status.md`.

## Inputs

- `temper/docs/plan/update-recovers/notes/common.md`, `temper/docs/plan/update-recovers/notes/contracts.md`, *The class*, *What the client settles an update with*
- `skein/docs/design/forge-client.md`, 7.3, 7.4, 7.5, 10
- `jig/docs/design/domain/connectors.md`, 4.3, 15
- `skein/crates/skein-forge-client/src/outbox/retry.rs`, `not_found`, which reads the class

## Owns and may change

- Owns the files in the header. `skein/tests/forge/client/src/world/recovery.rs` gains what the stories need: `land_writes_past_lifetime(after)`, as `land_writes_late` without its assertion, saying why an update's class allows it; a head and ancestry read of the fake; an outside push to a branch. `fixtures/mod.rs` gains `update_entry(pull, head, base, style)`.
- **The other tree's lines, each a value:** `temper/crates/temper-engine-domain-forge/src/lib.rs`, `effect_shape`: `11` leaves the `2 | 11 | 17` arm for an arm of its own, `(EffectForm::Transition, Recovery::Idempotent)`; `temper/tests/conformance/src/observations.rs`: `(11, r::Recovery::Idempotent)`. Nothing else in temper.

**Options:**

- **If a test reading the request's way fails otherwise** (the update isn't held today, or the find isn't by comparison), the request's reading is wrong: pin what's true, and say so first in your report.
- **If a skein or temper test outside your files asserts `Held` for an update,** change that assertion to what the table in `contracts.md` says, and name it in the report. None is known.
- **If the fake can't answer a story** (a copy's timing, a read the world lacks), add to the world in `world/recovery.rs`. Leave `skein-fake-forge-domain` alone: `debt-29/fake-config` writes it, and its copy reads at landing on purpose (`notes/common.md`).
- **If temper's engine worlds go red because a lost update now ends `Failed(Timeout)` and holds `Failed`,** where they relied on `Held`: that's the machine's tasks' to settle. Report it under `## Left`, with the test, and don't change the machine.

## Steps

1. Write the step tests. Run them: `heavy cargo nextest run -p skein-forge-client --lib -E 'test(/update/)'`. Check the count. Note each failure's reason for the report.
2. Change `recovery`, its comment and the class assertion, then the temper value lines. Run the step tests again.
3. Write the world's helpers, the stories and the fuzzy story, seeing a story that should fail today do so by putting the class back once. `heavy cargo nextest run -p skein-forge-client-world`, and `--profile fuzzy` for the fuzzy story.
4. **A change of meaning runs its readers' tests,** as `recovery` now means something else for kind 11. `heavy cargo nextest run -p skein-forge-client -p skein-forge-client-world`, `-p temper-engine-domain-forge --lib`, `-p temper-engine-forge-world`, `-p temper-engine-domain-world`, `-p temper-conformance-world`, one command at a time.
5. Close the status entries; `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py` and temper's; `python3 -I -B tools/citations/citations.py --base origin/main`; `heavy cargo clippy -p skein-forge-client -p skein-forge-client-world -p temper-engine-domain-forge -p temper-conformance-world --all-targets -- -D warnings`. Commit, small and saying why; push.

## Report

As your text says, and: what each test showed before the class moved, which is the plan's reproduction; each rule broken by hand for a new test and the failure seen; what the merge-style and rebase-style late copies did in the world.

## Rebase (2026-11-06, step merge, foreman update-recovers/update-class/foreman-1)

resolve forge.status.md by preserving both tasks’ status-bullet deletions
The merge conflicts in `temper/docs/design/domain/forge.status.md`.
Task commit `f42cdfd415` removes the `update-class` bullet.
Integration commit `58d6dcebca` (merged by `9771c5aa70`) removes the adjacent `change-decides-again` bullet.
Resolve the conflict with both bullets absent, retaining all other status entries and integration changes.
This satisfies acceptance item 7; no code change is needed for this refusal.
