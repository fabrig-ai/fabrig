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
| `fe-world-remove` | writing (waits for pd's ids) | fe-remove, fe-kept-root, pd's Cancel decision | engine domain world |
| `fe-demo-gone` | writing | fe-playground-forge, fe-playground-root, fe-world-remove | store, forge, root, worlds (+ forced readers) |
| `fe-world-reset` | writing (waits for pd's ids) | fe-demo-gone, pd's deletion orchestration | engine domain world |
| `fe-config-drop` | writing (waits for sv's refusal task id) | fe-records, sv's document refusal | forge, root, engine worlds (+ forced service sites) |

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
