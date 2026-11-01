+++
id = "fe-remove"
part = "rf"
kind = "feature"
needs = ["fe-make"]
files = ["temper/crates/temper-engine-domain-forge/**"]
effort = "high"
minutes = 110
+++

## Goal

The forge connector removes a forge whole (draft `temper/docs/design/draft/playground-reset.md`,
section 4; `notes/contracts.md` T5): in one decision the record reads `Removing`, the forge
is ended, its writer and flights are erased, serve is told (`Unregister`), and its kept acts
are written and sent; each is sent again at every start until answered; when the last is
answered the `Forge` row goes. The same removal, without the request's refusals, is the
playground reset's `Forge` step (`fe-playground-forge` calls it).

Acceptance:
- `RemoveForge { forge }`: refused `NoSuchForge` with no `Kept` record `forge`; refused
  `Playground` for the playground's record (`playground: true`); otherwise `Removing`,
  kept in memory as `connect::State::answers` keeps answers.
- `pub(crate) fn remove(d: &mut Domain, env: &Env<Limits>, forge: u16, out: &mut
  Queue<Request>)`, the decision, for a `Kept` record:
  - the record saved `state: Removing`; `Unregister { forge }` sent; `MarkedForges` sent;
  - the forge is ended at once (`forge_ended` is true since `fe-records`): the setup
    intent, connection, ended reason, App way and installations of the forge are dropped;
    `retire_ended`'s drops and the client's watches are rebuilt without it; a running
    connect or setup of the forge answers as the box's close answers it today;
  - the `Writer(forge)` row erased (and the writer dropped from `writers` and the client
    child, through the child's own path if it has one); every `Flight` row of an entry on
    the forge erased;
  - the kept acts, each saved as `Stored::KeptAct` and then sent as `SendKeptAct`: one
    `Cancel { project, task }` per live change task (`changes`) whose repository is on the
    forge, `project` the repository row's; `RemoveGroup` when the record has a setup entry
    and the connector is under the box; `EraseSecrets` always. Keys:
    `kept_key(forge, number, purpose)`, the first 16 bytes of SHA-256 over `b"temper forge
    removal\0"`, the forge's two big-endian bytes, the number's eight big-endian bytes (the
    task, or 0) and the purpose (`b"cancel"`, `b"group"`, `b"secrets"`).
- Answers erase their row: `KeptActAnswered { key, keep: false }`; `GroupRemoved { forge }`
  the forge's `RemoveGroup` row; `SecretsErased { forge }` its `EraseSecrets` row (an
  answer for a forge or key with no row does nothing). When a `Removing` record's last
  kept act is erased, in that decision the `Forge` row is erased and the record dropped;
  `fn removed(&self, forge) -> bool` (or an event the reset reads) tells the playground's
  reset that the row is gone.
- `SendKeptActs` sends every kept act again, in key order; a restored `Removing` record and
  its restored `KeptAct` rows resume the same way; a restored `Removing` record with no
  kept act left erases its `Forge` row at `SendKeptActs`.
- Repository rows of the forge stay (draft section 9's default), in no list, readable by
  their tag; adoptions onto the forge are refused as onto an ended forge.
- Restore fits: `KeptAct` rows at most `l.tasks + 2 * l.forges`.
- Unit tests below pass; every existing test of the crate passes.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T2, T5;
  `notes/fe-surface.md` sections 3 to 5.
- `temper/docs/design/draft/playground-reset.md`, section 4.
- `temper/docs/design/domain/forge.md`, 4.1 (the ended forge) and 6 (entries, flights).
- The demo's kept acts, the model: `temper/crates/temper-engine-domain-forge/src/demo.rs`
  (`acts`, `send_acts`, `act_answered`).

## Owns and may change

- Owns: `temper/crates/temper-engine-domain-forge/`.
- May change: nothing else.
- Pre-approved options:
  - Module: default, a new `src/removal.rs` with `remove`, the kept act map (`kept:
    Map<[u8; 16], KeptAct>`, sized `l.tasks + 2 * l.forges`), `send_kept`, `answered`.
  - Entries on the removed forge: default, as the ended demo forge's are today: they stay
    until their task's release settles them, and a release skips writes on an ended forge
    (find how `continue_release` treats `forge_ended`). If an entry of the forge would then
    never settle (a sent one waiting for an outcome the client can no longer read),
    settle it not made (`not_made`) in the removal's decision and say so.
  - A `Cancel` for a change whose repository row has no project (none is expected): skip
    it and say so.

## Steps

1. `src/removal.rs` as above; the step's `Ask::RemoveForge`, `GroupRemoved`,
   `SecretsErased`, `KeptActAnswered` and `SendKeptActs` arms call it (each a no-op until
   now); restore of `Stored::KeptAct` into the map; `restore.rs`'s room check;
   `reconcile.rs`: a `KeptAct` needs its forge's record (`Need::Forge`).
2. Tests in a new `src/tests/removal.rs`:
   `removing_a_forge_ends_it_and_keeps_its_acts_in_one_decision` (the saves, erases and
   sends of the decision, in order: record, writer, flights, kept acts; `Unregister`);
   `a_removed_forge_is_in_no_list_and_its_repositories_stay_readable_by_tag`;
   `the_playground_and_an_unknown_forge_are_refused`;
   `each_answer_erases_its_act_and_the_last_erases_the_record`;
   `kept_acts_are_sent_again_at_each_start_until_answered` (restore a `Removing` record and
   its rows, `SendKeptActs`, answer them one by one);
   `a_forge_without_a_setup_entry_or_outside_the_box_keeps_no_group_act`;
   `kept_acts_past_their_bound_do_not_fit`.
3. Run, one at a time: `heavy cargo nextest run -p temper-engine-domain-forge --lib`;
   `heavy cargo nextest run -p temper-engine-domain --lib`; `heavy cargo check --workspace
   --all-targets`; `cargo fmt --all`; `heavy cargo clippy -p temper-engine-domain-forge
   --all-targets -- -D warnings`; `python3 -I -B scripts/old_forms.py` from `temper/`.
4. Commit, push your branch.

## Report

- How entries and flights of the removed forge end, and every connector row family that
  can still name the forge after its record is erased.
