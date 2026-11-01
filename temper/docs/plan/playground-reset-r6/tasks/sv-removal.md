+++
id = "sv-removal"
part = "rf"
kind = "feature"
needs = ["sv-resolve", "bx-pipe"]
files = ["temper/crates/temper-service/**", "temper/crates/temper-protocol-records/src/lib.rs", "temper/crates/temper/src/host.rs"]
effort = "high"
minutes = 90
+++

## Goal

serve's side of the box's groups and of a removal's kept acts (`notes/contracts.md`, T3 and
T5): the group's settings file written before `start N`, the pipe's `remove N`/`removed N`,
and the forge's secrets erased. Each answers the root with the event fe's connector waits
for (`GroupRemoved`, `SecretsErased`), so a removal can end whole.

Acceptance:
- `root::Request::SetUpGroup { forge, request, entry }` with a non-empty `entry`: serve
  writes `<data>/box/groups/N` with the text `temper_forge_setup::setup::group_settings(
  &entry, forge)` (bx-steps) through its host, then asks `start N` as today
  (`box_link::Link::ask`). The write is whole: the host writes a temporary file in
  `<data>/box/groups/` (0700, made if absent; the file 0600) and renames it into place. A
  failed write (or `group_settings` refusing the entry) is the group's failure: the box
  link's `Out::Failed { forge, why }` with words naming the write, so the forge reads
  `SetUpFailed` as for `failed N WHY`. With an empty `entry` (a document forge's group, and
  every `SetUpGroup` until fe's records task), nothing is written and `start N` goes at once,
  as today.
- `root::Request::RemoveGroup { forge }`: the box link sends `remove N\n` when it holds no
  other ask (else holds it, as a held start is held: removes queue in order, at most
  `forge_component.forges`); on `removed N` serve sends the root
  `root::Event::Forge(Box::new(forge::Event::GroupRemoved { forge }))`. A `failed N WHY`
  while `remove N` is in flight is "not removed" (`notes/bx-progress.md`, Contract requests,
  1): named on serve's log (`Out::Named`), no event, the root's kept act stays and is sent
  again. With the box link ended (no box, or the pipe closed), a `RemoveGroup` is dropped
  and named once: the root sends it again at the next start.
- The box link reads `removed N` (and drops one not asked, as it drops other replies).
  `failed N no settings` after a `start N` is a start's failure as any `failed N WHY` is.
- `root::Request::EraseSecrets { forge }`: serve erases each of the forge's ten records
  (`temper_protocol_records::number(forge, item)` for each `Item`) from the secrets store
  (`Secrets::Erase { owner: SecretOwner::Application, number }`), and once every erase is
  answered durable sends `root::Event::Forge(Box::new(forge::Event::SecretsErased { forge
  }))`; an absent record's erase counts as erased. A failed store stops the service as
  every failed secrets write does today. Two `EraseSecrets` of one forge in flight erase
  once and answer each.
- `temper_protocol_records::ITEMS: [Item; 10]`, every item in code order, which serve
  walks.

## Inputs

- `temper/docs/design/protocol/setup.md`, 7.1 (the pipe), 8.4.
- Draft `temper/docs/design/draft/playground-reset.md`, section 4.
- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, T3, T5.
- `temper/docs/plan/playground-reset-r6/notes/fe-surface.md`, 3 (the requests and events).
- `temper/docs/plan/playground-reset-r6/notes/bx-progress.md`, Contract requests, 1 (a
  failed removal), and `temper/docs/plan/playground-reset-r6/tasks/bx-pipe.md` (the box's
  lines).

## Owns and may change

- Owns: `temper/crates/temper-service/` (`src/route.rs`, `src/route/box_link.rs`,
  `src/system.rs`, `src/lib.rs`, a new `src/erase.rs` or the setup layer's answer path for
  the erases' completions, and their tests); `temper/crates/temper-protocol-records/src/lib.rs`
  (`ITEMS`, with a unit test that it holds each item once, in code order).
- `temper/crates/temper/src/host.rs`: `drain`'s loop answering the group writes and its
  `write_group` function, alone (`sv-resolve` already edited `drain`).
- Pre-approved options:
  - The host interface, as the handover's: `temper_service::GroupWrite { forge: u16, text:
    Box<[u8]> }`, `Service::take_group_write(&mut self) -> Option<GroupWrite>`,
    `Service::group_written(&mut self, forge: u16, result: Result<(), i32>)`; the host
    makes the path from its data directory (`box/groups/N`), the service names no path.
    Default as said.
  - The erase's owner tokens: the setup layer's `next_run` tokens, as `setup/public.rs::
    erase` takes one, with a map from token to forge. Default as said.

## Steps

1. `src/route.rs`: the three arms (fe-surface left `RemoveGroup` and `EraseSecrets` as
   no-ops and ignored `SetUpGroup`'s `entry`).
2. `src/route/box_link.rs`: `Ask::Remove(u16)`, the removes' queue, `Link::remove(forge,
   out)`, `Reply::Removed(u16)`, `Out::Removed { forge }` (routed to the root's event in
   `box_link::drain`); the start held until its group file is written (`Link::ask` is
   called from `group_written`'s `Ok`).
3. `src/system.rs`, `src/lib.rs`: `GroupWrite`, `take_group_write`, `group_written`.
4. The erases (`src/erase.rs`): `pub(crate) fn erase(engine, forge)`, its completions.
5. The binary: `host.rs::drain` and `fn write_group(data: &Path, write:
   &temper_service::GroupWrite) -> Result<(), i32>` (create `box/groups` 0700, write
   `box/groups/.N.tmp` 0600, `fsync`, rename to `box/groups/N`).
6. Tests:
   - temper-service `src/tests/box_link.rs`: `a_set_up_group_with_an_entry_writes_its_file_then_asks_start`,
     `a_failed_group_write_fails_the_group_without_asking_start`,
     `an_empty_entry_asks_start_at_once`, `remove_is_sent_answered_and_routed_as_group_removed`,
     `removes_wait_behind_a_live_ask_in_order`, `a_failed_remove_names_it_and_sends_no_event`,
     `a_remove_with_the_link_ended_is_dropped_and_named`,
     `a_start_failed_no_settings_fails_the_group`.
   - temper-service `src/tests/erase.rs`: `erase_secrets_erases_each_item_and_answers_once_durable`,
     `erase_secrets_of_a_forge_with_no_records_answers_erased`,
     `two_erases_of_one_forge_answer_each`.
   - temper-protocol-records: `items_hold_each_item_once_in_code_order`.
   - the binary: `write_group_writes_whole_with_private_modes` (a scratch data directory:
     the file's text, 0600, the directory 0700, no temporary file left).
7. Narrow runs, one at a time:
   `heavy cargo nextest run -p temper-service --lib -E 'test(/tests::(box_link|erase)::/)'`;
   `heavy cargo nextest run -p temper-protocol-records --lib -E 'test(=tests::items_hold_each_item_once_in_code_order)'`
   (or its path where the test lands); `heavy cargo nextest run -p temper --lib -E 'test(/write_group/)'`.
   Then `heavy cargo nextest run -p temper-service`; `heavy cargo nextest run -p
   temper-protocol-records`; `heavy cargo check --workspace --tests`.
8. `cargo fmt`; `heavy cargo clippy -p temper-service -p temper-protocol-records -p temper
   --all-targets`; old forms. Commit and push.

## Report

Nothing beyond the implementer's text.
