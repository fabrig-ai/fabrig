# fe: temper's engine (store, forge connector, root): progress

Designer fe of playground-reset-r6. Crates: temper-protocol-store,
temper-engine-domain-forge (forge), temper-engine-domain (root), their worlds
(`temper/tests/engine/*`), `temper/tests/codecs` (store parts), `temper/tests/conformance`
(temper side). Shared shapes: `notes/fe-surface.md`.

## Specs

| id | state | needs | crates |
|---|---|---|---|
| `fe-surface` | final | none | forge, root, store, codecs world, engine worlds (+ forced arms in service, fake, client/system worlds) |
| `fe-records` | final | fe-surface | forge, root, engine worlds |
| `fe-make` | final | fe-records | forge |
| `fe-kept-root` | final | fe-records, pd-vocabulary | root |
| `fe-remove` | final | fe-make | forge |
| `fe-playground-forge` | final | fe-remove | forge |
| `fe-playground-root` | final | fe-kept-root, pd-vocabulary | root |
| `fe-world-remove` | final; the lead adds pd's task that decides `Act::Cancel` in the core to `needs` | fe-remove, fe-kept-root, pd-vocabulary (+ pd's Cancel decision) | engine domain world |
| `fe-demo-gone` | final | fe-playground-forge, fe-playground-root, fe-world-remove | store, forge, root, codecs and engine worlds (+ forced readers in service, fake, worlds) |
| `fe-world-reset` | final; the lead adds pd's deletion tasks (core decision and stages, connector told, `pd-accounts-uses`) to `needs` | fe-demo-gone (+ pd's deletion) | engine domain world |
| `fe-config-drop` | final; the lead adds sv's `sv-startup-refuse` to `needs` once sv names it | fe-world-reset (+ sv-startup-refuse) | forge, root, engine worlds (+ mechanical seeding in service, client/system/conformance worlds) |
| `fe-make-git` | final | fe-config-drop (the forge connector's last writer before it; move it earlier only by putting it between two forge-crate tasks) | forge |

## What lands where (for other designers)

- `fe-surface`: every type and variant of `notes/fe-surface.md`; requests decided as
  refusals or no-ops (its section 7).
- `fe-records`: `Register` for every restored record at `Restored` (before the root's
  `ReadAfresh`, so before serve's credential load if serve waits for it), records listed in
  `forges()` beside the document's forges; `ForgeRecord::configured` (pub).
- `fe-make`: `MakeForge` answers `Made { forge }` (refused `NotOffered` while the document
  still configures forges); `Register` on make and on every saved change (first
  repository, GitHub App pasted); the record's `first` kept across a restart, so serve's
  setup host can rebuild its intent from `Register(record).first`.
- `fe-kept-root`: `root::Request::{RemoveGroup, EraseSecrets}` sent for a removal's kept
  acts; `root::Domain::playground() -> Option<forge::Playground>` (serve's playground read).
- `fe-remove`: `RemoveForge` answers `Removing`; `Unregister` in its decision; `RemoveGroup`
  and `EraseSecrets` sent until `GroupRemoved`/`SecretsErased` (resent at each start).
- `fe-playground-forge` + `fe-playground-root`: `StartPlayground` and `EndPlayground` on
  records (both are needed for the playground's process stories: `bx-story-playground`
  names `fe-playground-forge` and `fe-kept-root`; it also needs `fe-playground-root`).

## Contract requests

1. **`Configured.owned.password` (T1, "derived as today").** Today it is an absolute path
   built from the setup entry's directory; the connector (no_std, no JSON, no data
   directory) cannot build it from a record. **Default taken (fe-records):** the projection
   gives `Owned { forgejo: pin::VERSION, runner: Some(pin::RUNNER_VERSION), password:
   b"forge-N/person_password" }`, the path relative to the data directory; serve's FORGE
   document (`temper-service/src/family.rs`, `owned`) joins it with the data directory, and
   may correct `runner` from the registered record's setup entry. sv's family task should
   do the join.
2. **The playground start's act keys (T6, "keyed from the start's key").** A start's key is
   the client's and is lost by a restart; a restart between jig's commit of `MakeProject`
   and the row's save would leave an orphan project the reset never deletes. **Default
   taken (fe-playground-root):** the start's acts are keyed from the playground forge's
   number (never reused), so a resent act meets jig's retained answer. Inside temper; no
   other crate sees these keys.
3. **The playground's adoption as the deployment's `Act::Adopt` (T6).** jig routes an
   admitted `Act::Adopt` to the connector as `Ask::Adopt { request }` with the core's own
   request token, which temper's root matches against the forge's located application
   (`route/asks.rs`, `domain.forge.applying(request)`); a deployment act has no located
   application, so it is refused `NotOffered`. Making it work needs a new two-phase path in
   the forge and root. **Default taken (fe-playground-root):** the adoption goes through the
   application route as today's demo adopts, under the starting person's sign-in (that
   person made the project with `MakeProject { maker }` and owns it); the project and the
   account are the deployment's acts as T6 says.
4. **The transition while forges leave the document.** fe follows sv's contract request 1
   (step (a)): `fe-records` is additive (records beside the document's `Config.forges`,
   identities by number for records only), `fe-make` refuses `MakeForge` while the document
   configures forges, and `fe-config-drop` removes `Config.forges`/`identities` after sv's
   refusal task (to be named when sv marks it). If the lead settles otherwise, only
   `fe-config-drop`'s place changes.
5. **The Forgejo-elsewhere bot.** T4 now gives its setup entry; its bot's display name is
   `Temper` and email `temper@localhost`, as sv-startup-machine names the machine's bot.

## Task graph and critical path

```
fe-surface ─► fe-records ─┬─► fe-make ─► fe-remove ─┬─► fe-playground-forge ─┐
                          │                         └─► fe-world-remove ──────┤
                          └─► fe-kept-root ─┬───────────(also ► world-remove) │
                               (pd-vocab)   └─► fe-playground-root ───────────┤
                                                                              ▼
                                         fe-demo-gone ─► fe-world-reset ─► fe-config-drop ─► fe-make-git
```

- Crates are written one task at a time: the forge connector by surface → records → make →
  remove → playground-forge → demo-gone → config-drop; the root by surface → records →
  kept-root → playground-root → demo-gone → config-drop; the engine domain world by surface
  → records → world-remove → demo-gone → world-reset → config-drop; the store by surface →
  demo-gone. `fe-kept-root` and `fe-playground-root` (root) run beside `fe-make`,
  `fe-remove` and `fe-playground-forge` (forge); `fe-world-remove` beside
  `fe-playground-forge`.
- Critical path (implementer minutes): fe-surface 110 → fe-records 110 → fe-make 100 →
  fe-remove 110 → fe-playground-forge 110 → fe-demo-gone 80 → fe-world-reset 100 (→
  fe-config-drop 70, which also waits for sv's refusal): about 12 hours of builds end to
  end, plus waits on pd's deletion tasks for the two world tasks. Other designers need
  only `fe-surface` early; the process tier's playground stories need
  `fe-playground-forge`, `fe-playground-root` and `fe-kept-root`.
- Note: pd's `pd-vocabulary` writes temper-engine-domain files (compiler-forced arms,
  `route/asks.rs`'s `ReleaseProject` answer) while `fe-surface`/`fe-records` may also
  write that crate: small, separate lines, but two writers; `fe-kept-root` replaces
  pd's `ReleaseProject` arm.
