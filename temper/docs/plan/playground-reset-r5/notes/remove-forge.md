# Part note: `remove-forge` (playground-reset-r5)

Removal's shapes, beside `notes/forge-records.md` (the record, its
codec, the wire). Draft: `temper/docs/design/draft/playground-reset.md`,
sections 4 and 6.4. It adds to `notes/contracts.md` and never changes it.

## 1. `Removal` (in the record, codec by `fr-family`)

```rust
/// A forge's removal under way (draft, 4): what is left to answer.
pub struct Removal {
    /// The box answered `removed N`, or the forge has no box group (or no box).
    pub group: bool,
    /// The service erased the forge's secret records (credentials.md, 2.1).
    pub secrets: bool,
}
```

Schema: `record Removal { group: bool secrets: bool }`, the record's
`removal: option Removal`.

## 2. The decision (`Domain::remove_forge`, task `rf-domain`)

`remove_forge(&mut self, forge: u16, out: &mut Queue<Request>) ->
Result<(), RemoveRefusal>`, `RemoveRefusal::{NoSuchForge}` (no record, or
its removal already set; the owners' role is the ask's admission, as
`Connect`'s). In the caller's decision:

1. The record saved with `removal: Some(Removal { group, secrets: false })`,
   `group` true when its mode is neither owned nor fake, or the
   connector is not under the box.
2. **Ended at once:** `Domain::forge_ended(forge)` is true for a record
   with a removal (as for today's ended demo forge): every gate that
   reads it today (lists, connections, offers, adoption, context,
   observations, `retire_ended`, restore and reconcile gates; survey S3,
   section 2) reads it for any forge. `Request::ForgeEnded { forge }` is
   emitted: the service drops it from every table (skein's `Remove` on
   both components), answers its calls `NotConnected`, drops its hints,
   and sets it up no more.
3. **Its writer and its flights** erased in this decision.
4. **Its live work:** for each live change task (a `Change` row) on one
   of its repositories, a `Stored::Kept(Kept { key, act:
   KeptAct::Cancel { project, task } })` row saved, the key
   `sha256("temper remove\0" || forge as 2 bytes BE || task as 8 bytes
   BE)`'s first 16 bytes; sent as jig's `Act::Cancel { project, task,
   grace: None, reason: "The forge was removed." }`.
5. `Request::RemoveGroup { forge }` unless `group`, and
   `Request::EraseSecrets { forge }`.

## 3. Kept until whole

- `Event::GroupRemoved { forge }` (box `removed N`) and
  `Event::SecretsErased { forge }` (the service's erases committed) save
  the record's flags.
- A `Kept` row is erased on its act's saved answer (compound, ended,
  unknown, or a refusal naming an unknown project or task), kept on a
  refusal to retry (`Again`, no grant), as `DemoAct` rows are today
  (`temper-engine-domain/src/route/demo.rs`, `end_acted`).
- **Its repositories' rows** are erased once no live row names them: no
  `Names`, `Hold`, `Change`, `Release`, `Subscription` or `Landing` row of
  a task names the repository (draft, section 9's default: kept while a
  task names them). The check runs at each decision that erases such a
  row, and at restart.
- **The record** is erased when `group && secrets`, no `Kept` row is its
  own and no `Repository` row of the forge remains. `ForgeNext` stays.
- **At each start:** a restored record with a removal is ended, not
  configured (no `ForgeKept`); its `RemoveGroup` / `EraseSecrets` sent
  again while its flag is false; its `Kept` rows sent again with every
  kept row (restart's `Open` step, as `SendDemoActs` today).

## 4. `Kept` (family 19, renamed from `DemoAct`, task `rf-family`)

```rust
pub struct Kept { pub key: [u8; 16], pub act: KeptAct }
pub enum KeptAct {
    /// Today's demo end's close, until the playground's tasks remove it.
    Close { project: u32, task: u64 },
    Cancel { project: u32, task: u64 },
    RemoveProject { project: u32 },
    RemoveAccount { account: u32 },
}
```

`Stored::Kept(Kept)`, `Key::Kept([u8; 16])`; `keys::KEPT = 19`; schema
`KeptClose`, `KeptCancel`, `KeptRemoveProject`, `KeptRemoveAccount`, `enum
KeptAct`, `record Kept`. The root sends every kept row's act and routes
its answer back by key, whatever made it (the demo's end, a removal, the
playground's reset): `Request::SendKept(Kept)` and `Event::KeptAnswered {
key, answer }`, replacing `SendDemoAct` / `DemoActAnswered` (its
`ListDemoActs` / `DemoActs` selection stays the demo's until the
playground's tasks). The kept-row bound is `tasks + 2` (a removal's
cancels, at most one per task, and the playground's two).

## 5. A project removed (jig's `Request::ProjectRemoved`, task `rf-domain`)

The root routes jig's `core::Request::ProjectRemoved { project }` to the
connector as `Event::ProjectRemoved { project }` (in the closing
decision). The connector marks the project's repositories gone from it:
their `Repository` rows stay readable to its tasks' names but leave every
list, adoption choice and context (a project read as absent), and are
erased by section 3's rule. temper keeps no settings rows of its own
(store family 20 is design-only), so nothing else is erased; a later
`Setting` family erases its rows of that scope then.

## 6. The box's `remove` (task `rf-box`)

As contracts, 4: `remove N` → stop the group if running (as `stop N`),
then delete `<data>/box/groups/N`, `<data>/box/started/N` and the
group's directory (the group file's entry's `directory`, `forge-N`),
keeping the cache; answer `removed N`; a number with no group file
answers `removed N` at once; refusals `busy` and `stopping` as `stop`'s.
Serve's link: `Link::remove(forge)` emits `remove N`, its reply
`removed N` drained into `Event::GroupRemoved { forge }`.
