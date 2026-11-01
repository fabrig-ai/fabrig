+++
id = "rf-service"
part = "remove-forge"
kind = "feature"
needs = ["rf-root", "fr-svc-route", "rf-box"]
files = ["temper/crates/temper-service/src/**", "temper/crates/temper-init/template/temper.json", "temper/crates/temper-init/src/tests/**"]
effort = "high"
minutes = 110
+++

## Goal

The service's half of removing a forge (draft
`temper/docs/design/draft/playground-reset.md`, section 4;
`notes/remove-forge.md`, sections 2, 3 and 6): an ended forge is dropped
from every table and served no more; its secrets erased; its box group
removed through the pipe's `remove`; the client's `RemoveForge` reaches
the connector and is answered.

Acceptance:

- `root::Request::ForgeEnded { forge }`: `Engine::forget_forge(forge)`
  sends `skein_forge_connection::Request::Remove` and
  `skein_forge_setup::host::Request::Remove`, drops the forge's tokens,
  rows, workers' entry and setup lifecycle state (its live set up ends as a
  stopped one does, its first intent dropped), and adds it to the
  service's ended set. The service's ended predicate (`forge.rs`'s
  `ended`, today the demo mirror) reads that set besides the demo's: its
  calls answered `NotConnected` at no cost (`route.rs`), its hints dropped
  (`engine.rs`), no new connection, the setup host's requests and plans
  refused (`setup.rs`, `setup/lifecycle.rs`'s gates).
- `root::Request::EraseSecrets { forge }`: `Secrets::Erase { owner:
  Application, number: records::number(forge, item) }` for every
  `temper_protocol_records::Item`, then `root::Event::SecretsErased {
  forge }` once every erase is answered (an erase of an absent row counts
  as done). A failed erase is a note naming the forge; the next start
  sends `EraseSecrets` again.
- `root::Request::RemoveGroup { forge }`: `box_link::Link::remove(forge)`
  emits `remove N` (queued and held as `stop` and `start` are), its reply
  `removed N` parsed (`route/box_link.rs` `decoded`) and drained into
  `root::Event::GroupRemoved { forge }`; `failed N why` is a note, and
  the next start asks again. With no box link, `GroupRemoved` at once.
- `family::to_root`: wire `remove_forge { forge }` → `Ask::RemoveForge {
  forge }` (replacing `fr-wire`'s placeholder refusal); the answer →
  `removing { forge }` or `remove_refused { why }`.
- `temper init`'s template: `engine.acts`' connector 1 gains `cancel`
  (the removal's cancels; the playground part adds its own grants).
- Tests (library tests, a new `src/tests/removal.rs`, under a box pipe as
  `src/tests/forge_records.rs` drives one):
  `a_removed_forge_answers_its_calls_not_connected_and_drops_its_hints`,
  `a_removed_forges_secrets_are_erased_then_told`,
  `a_removed_owned_forge_asks_the_box_to_remove_its_group_then_tells_it_removed`,
  `remove_forge_from_the_client_answers_removing_and_the_forge_leaves_the_watch`,
  `a_restart_mid_removal_erases_and_removes_what_is_left`,
  `a_set_up_cut_short_is_removed_then_a_new_owned_forge_sets_up_afresh`;
  `src/tests/box_link.rs`
  `remove_lines_are_held_behind_an_active_ask_and_removed_replies_parse`.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (sections 3
  and 4), `temper/docs/plan/playground-reset-r5/notes/remove-forge.md`,
  `temper/docs/plan/playground-reset-r5/notes/forge-records.md`
  (section 5).
- `temper/docs/design/protocol/credentials.md`, 2.1;
  `temper/docs/design/protocol/setup.md`, 7.1.
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s3-playground-today.md`,
  section 2, "Service predicate is separately demo-specific" and the
  setup gates after it.

## Owns and may change

- Owns: `temper/crates/temper-service/src/**` (one writer for this
  wave); `temper-init/template/temper.json`'s `engine.acts` line and
  `temper-init/src/tests/` for that line.
- Options: if `Item` has no list of every item, add
  `temper_protocol_records::Item::ALL` (a const array) in
  `temper-protocol-records` and say so; if the service's carried-secret
  rows (`CARRIED` numbers) for the forge exist, leave them to their own
  erase path.

## Steps

1. `forget_forge` and the ended set.
2. Secrets erased; the box link's `remove`.
3. The wire's `remove_forge`; the template's grant.
4. Tests. Narrow: `heavy cargo nextest run -p temper-service --lib -E
   'test(/removal/) | test(/box_link/)'`.
5. `cargo fmt`; `heavy cargo clippy -p temper-service -p temper-init
   --all-targets -- -D warnings`; focused tests of `temper-service` and
   `temper-init`; `heavy cargo check -p temper -p temper-one-process-world
   --tests`. Old-forms from `temper/`. Commit and push.

## Report

As `notes/common.md` says.
