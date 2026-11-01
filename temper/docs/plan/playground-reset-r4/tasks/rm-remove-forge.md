+++
id = "rm-remove-forge"
part = "forge-removal"
kind = "feature"
needs = ["kept-acts", "fr-setup", "fr-connect", "sk-client-remove"]
files = [
  "temper/crates/temper-engine-domain-forge/src/removal.rs",
  "temper/crates/temper-engine-domain-forge/src/asks.rs",
  "temper/crates/temper-engine-domain-forge/src/kept.rs",
  "temper/crates/temper-engine-domain-forge/src/seed.rs",
  "temper/crates/temper-engine-domain-forge/src/demo.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/removal.rs",
]
effort = "high"
minutes = 70
+++

## Goal

`RemoveForge`'s body (draft 4; contracts D3, D4): in one decision the
forge is ended, as `forge_ended` ends the demo's forge today, for any
forge; its row reads `removing`; its writer, flights and outbox entries
are erased at once and the client forgets it; each live change on its
repositories is kept as a `Cancel`, and an owned or fake forge's group
as a `BoxRemove`; the service is told (`Request::ForgeRemoved`), which
drops it and erases its secrets (`rm-forge-secrets`). The row is erased
once its kept acts are all answered, and that answers the reset's
`KeptAct::RemoveForge` when one names it. A restart resumes it.

Acceptance:

- `removal::remove(d, env, forge, out) -> Result<(), crate::RemoveRefusal>`
  is the decision (the ask's and the playground's reset's, which calls it
  directly): `NoSuchForge` for a number with no record or one already
  removing; otherwise, in the decision: the row saved with `removing`
  true; `Request::ForgeRemoved { forge }`; the client's
  `skein_forge_client::remove_forge` (`sk-client-remove`), its requests
  drained as the connector drains the client's (its
  `Erase { Key::Writer(n) }` translated to the connector's own
  `Key::Writer(n)`: the client's writer is kept as `Stored::Writer`,
  `temper-protocol-store/src/keys.rs`'s `unreachable!` stays true);
  the connector's `writers` entry, `flights` and outbox `entries` of the
  forge's repositories erased (`Request::Erase` each); `connections`,
  `first`, `apps`, `installations`, the forge's pending reads (as
  `retire_ended` drops them) dropped; then `kept::keep` of one `Cancel {
  project, task }` per change row on the forge's repositories whose task
  is live, and of `BoxRemove { forge }` under the box for a record whose
  setup is `Owned` or `Fake`; `Request::MarkedForges`.
- `RemoveForge` the ask (`forge-asks`' stub in `asks.rs`): the
  deployment's owners' (`Role::Owner`, else `RemoveRefusal::Role`); the
  playground's forge while a `Playground` row is running answers
  `NotOffered` (only the reset removes it); otherwise `remove`, answered
  `RemoveAnswer::Removing` or its refusal, through the connect answer
  table as today's stubs answer.
- "Ended" holds for any removing forge: `forge_ended(n)` (in `demo.rs`
  today) is also true for a removing record; `retire_ended` runs for any
  ended forge, not only when the demo has ended; `forges()` leaves it
  out (`fr-seed` already skips removing rows); a removing record is not
  told as `ForgeKept` at a start (`seed.rs`'s `kept`).
- The finish: after each kept answer (`kept::answered`,
  `kept::box_answered`) and at each start's `Restored`, every removing
  forge whose rows are gone (`kept::erased_for`) has its `Forge` row
  erased (`Key::Forge`), its tables entries dropped, and the reset's
  `KeptAct::RemoveForge { forge }` row erased if kept (`kept::erase`).
  Its `Repository` rows stay, ended and readable (D3).
- The tests below pass; the demo's end tests pass unchanged.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (D3, D4,
  section 2's `KeptAct`).
- The draft, section 4 whole.
- `temper/docs/design/domain/forge.md`, 4.1 (the playground's forge
  ended: the model).
- The code it generalises: `demo.rs` (`forge_ended`, the end's
  `end`), `domain.rs` (`retire_ended`, the client's output drain near
  `Request::Erase { key: Key::Client(..) }`).
- The merged `kept-acts`, `fr-make-forge`, `sk-client-remove` and
  `forge-asks` reports.

## Owns and may change

New test files are declared from the module they test, at its end
(`#[cfg(test)] #[path = "tests/NAME.rs"] mod tests;`, the path relative to
that module, as `temper-protocol-store/src/records.rs` declares `holds`),
never in a crate's `src/tests.rs`: tasks running at once would all edit it.
Where this spec says "its `mod` line in `tests.rs`", read that.

Owned: `removal.rs` (its stub), the new `tests/removal.rs` and its
`mod` line. Edits elsewhere, each only for this: `asks.rs` (the
`RemoveForge` stub's body), `kept.rs` (calling the finish after an
answer), `seed.rs` (`kept` skips removing records), `demo.rs`
(`forge_ended`'s one condition; if the playground part has moved
`forge_ended` by then, edit it where it is), `domain.rs` (`retire_ended`'s
gate, the client drain's `Key::Writer` translation, `restored`'s call to
the finish).

Neighbours: none.

Pre-approved options:

- If a change row's task is not directly visible to the connector
  (only the root sees the core's tasks), keep `Cancel` for every change
  row of the forge whose `ChangeRow` is not terminal, and let the core's
  answer (*ended* or *unknown*, both done: contracts 4) clear the rest.
- If the client's outbox entries are erased by its own requests
  (`remove_forge`'s), do not erase them twice.

## Steps

1. `removal.rs` (module comment cites draft 4, D3, D4): `remove`,
   `pub(crate) fn removing(d: &Domain, forge: u16) -> bool`,
   `pub(crate) fn finish(d, env, out)`.
2. The edits above.
3. **Tests**, `tests/removal.rs`, with `tests/fixtures.rs`' builders:
   `removing_a_forge_ends_it_at_once` (no row, `forge_ended`, a hint for
   it dropped, `ForgeRemoved` out),
   `its_writer_flights_and_entries_are_erased_in_the_decision`,
   `each_live_change_on_it_is_kept_as_a_cancel`,
   `an_owned_forge_under_the_box_keeps_its_box_remove`,
   `the_row_is_erased_once_its_kept_acts_are_answered`,
   `a_removal_with_nothing_to_keep_erases_its_row_at_once`,
   `the_resets_remove_forge_row_is_answered_by_the_finish`,
   `a_restart_resumes_a_removal` (restore a removing row and its kept
   rows: ended, not told kept, finished once answered),
   `a_non_owner_and_an_unknown_forge_are_refused`,
   `the_running_playgrounds_forge_is_not_offered`,
   `its_repository_rows_stay_readable`.
   Narrow: `-p temper-engine-domain-forge --lib -E 'test(removal::) |
   test(demo::) | test(kept::)'`.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge
   --all-targets -- -D warnings`; the crate's focused tests; this
   changes what `forge_ended` means, so the focused tests of
   `temper-engine-domain`, `temper-engine-domain-world` and
   `temper-engine-forge-world` too. Commit and push.

## Report

fab's sections, and `## Interfaces`: `removal::{remove, removing,
finish}`, and each existing function whose meaning changed.
