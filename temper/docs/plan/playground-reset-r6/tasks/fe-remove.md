+++
id = "fe-remove"
part = "rf"
kind = "feature"
needs = ["fe-make", "sv-skein-retire"]
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

## Answers

- 2026-11-01. Q: may this task add a forge-retirement API in skein-forge-client, or does a
  prerequisite provide it? A: a prerequisite provides it, and this task now needs it:
  `sv-skein-retire` adds `skein_forge_client::Event::Retire { forge }` (read its spec: the
  writer dropped; every client entry on the forge settled `Failed(NotConnected)` at once,
  or at its call's one terminal when a call is out; nothing named afterwards). Change no
  skein file here. Use it so:
  - In `remove`'s decision, after the `Writer(forge)` row's erase and the writer's drop from
    `d.writers`, step `child(d, env, skein_forge_client::Event::Retire { forge }, out)`.
    Its outcomes come back through `drain` as any `Outcome` does, erasing each settled
    entry's `Entry` and `Flight` rows and telling its task. An entry of the forge not yet
    handed to the child (`handoff != Handed`) is settled the same way by the connector
    itself, as `withdraw` settles one today (an `Outcome::Failed(Error::NotConnected)`
    pushed to `client_out`, then `drain`). Held entries (`held: true`, not the child's)
    stay until their task's close settles them not made, as the pre-approved default says.
    Do not call `not_made` for an entry the child holds.
  - So "every `Flight` row of an entry on the forge erased" in the decision reads: erased as
    each entry settles; an entry whose call is out settles, with its flight, at that call's
    answer (serve's `Unregister` ends every open call of the forge at once,
    `sv-skein-connection`'s `Remove`, and serve answers an ended forge's later calls
    `NotConnected`). The first test's order is then: the record, `Unregister`, the writer,
    each settled entry's erases and outcome, the kept acts.
  - At a start, for each restored `Removing` record, `SendKeptActs` steps `Retire` again
    before it sends the record's kept acts (after restore has ended; a forge with nothing
    held answers nothing), and settles its not-handed entries as above.
  - Add the test `a_removed_forges_entries_settle_and_none_is_named_afterwards` (an unsent,
    a sent and a held entry on the forge, one on another forge untouched).

### 2026-11-01: Should fab restart fe-remove with sv-skein-retire included (recommended), or revise the retirement contract? The required skein_forge_client::Event::Retire API is absent from this checkout. (playground-reset-r6/fe-remove-2, foreman playground-reset-r6/fe-remove/foreman-2)

Restart fe-remove with its declared prerequisite sv-skein-retire included; retain the retirement contract.
The spec declares `needs = ["fe-make", "sv-skein-retire"]`.
Its Answers section states: “a prerequisite provides it, and this task now needs it”.
It also states: “Change no skein file here.”
The prerequisite spec exists at `temper/docs/plan/playground-reset-r6/tasks/sv-skein-retire.md` and requires `Event::Retire { forge: u16 }`.
The API remains absent from this checkout.
The next run uses the completed prerequisite and implements removal and restored Removing records as the existing answer specifies.
The task cannot be accepted as done: implementation and acceptance checks remain pending.

Report file was not written because this session permits filesystem reads only.

### 2026-11-01: Restart fe-remove with sv-skein-retire included (recommended), or revise the retirement contract? The required skein_forge_client::Event::Retire API is still absent from this checkout. (playground-reset-r6/fe-remove-3, foreman playground-reset-r6/fe-remove/foreman-3)

Restart fe-remove with completed sv-skein-retire included; retain the retirement contract.
The spec declares `needs = ["fe-make", "sv-skein-retire"]`.
Its Answers state: “a prerequisite provides it, and this task now needs it”.
The previous answer explicitly says: “Restart fe-remove with its declared prerequisite sv-skein-retire included; retain the retirement contract.”
The prerequisite spec exists in this checkout and requires `Event::Retire { forge: u16 }`.
The client’s `Event` enum still lacks `Retire`.
The next run uses the completed prerequisite and implements removal as the existing Answers specify, changing no skein file.
Acceptance remains pending; this task cannot be accepted as done.
Report file was not written because this session permits filesystem reads only.

### 2026-11-01: Restart fe-remove with completed sv-skein-retire included (recommended), or revise the retirement contract? The required skein_forge_client::Event::Retire API is still absent from this checkout. (playground-reset-r6/fe-remove-4, foreman playground-reset-r6/fe-remove/foreman-4)

Restart fe-remove with completed sv-skein-retire included; retain the retirement contract.
The spec declares `needs = ["fe-make", "sv-skein-retire"]`.
Its Answers state: “a prerequisite provides it, and this task now needs it” and “Change no skein file here.”
The previous answer explicitly requires restarting with the completed prerequisite.
The prerequisite spec exists in this checkout and requires `Event::Retire { forge: u16 }`.
The client’s `Event` enum still lacks `Retire`.
The next run uses the completed prerequisite and implements removal as the existing Answers specify.
Acceptance remains pending; this task cannot be accepted as done.
Report file was not written because this session permits filesystem reads only.

### 2026-11-01: four runs found no `Event::Retire`; restart with the prerequisite, or revise the contract? (playground-reset-r6/fe-remove-4)

Keep the contract; the prerequisite is done, and the branch is stale. `sv-skein-retire`
landed on the trunk as b2d06716dc ("Retire forge writers and settle their held writes"),
merged into `playground-reset-r6/integration` at b3550a5d3b. The branch
`playground-reset-r6/fe-remove` stands at fbfbef1289, a merge of `fe-make` made before
that, so each run that reused it found neither the variant nor the spec. First step of the
next run: `git fetch origin` and merge `origin/playground-reset-r6/integration` into the
branch (a merge, no rebase, no force-push); then check that
`skein/crates/skein-forge-client/src/boundary.rs` has `Retire { forge: u16 }`. If it is
still missing after that merge, that is a new question; do not ask this one again. After the
merge, build the task as the Answers above say, changing no skein file.
