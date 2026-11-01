# Where the code stands (playground-reset-r6)

From the tree at `6e3f383612` (main `c30cf6a03` plus this run's inputs), 2026-11-01, by six
read-only surveys. Against the draft's section 2, "today".

## Forges named by `temper.json`

- `application.forges` (number, kind, api, transport, git, web), `application.identities`
  (bot login and author per forge) and `application.setup` (one entry per set-up forge:
  directory, bot, mode fake/owned/managed, `start: asked`) are parsed by temper-service's
  `startup/application.rs` (`Members`, `Forge`, `Identity`) and temper-forge-setup's
  `setup.rs` (`Setup`, `Entry`, `Mode`). `temper init`'s template names forge 1 (fake), 2
  (GitHub) and 3 (owned, asked). A document with no forge is refused today.
- The forge connector is built from `Configured` (temper-engine-domain-forge, boundary.rs)
  in `Domain::new`; numbers are plain `u16`, fixed by the document; no record of a forge
  exists in the store, only its secrets (temper-protocol-records, per-forge items) and its
  writer (family 16).
- skein's forge connection component and setup host take their forge list once, at start:
  forges made at runtime need a skein mechanism first.
- Limits: no `forge.forges`; counts borrow `forge.client.repositories` or the configured
  forge count.

## The playground

- `StartDemo` exists (no `EndDemo`); the `Demo` row (family 17) is `{forge, state:
  Running|Ended}`; the end is automatic on another configured forge's connect, closing the
  project's chats ("Ended with the playground") and removing the scripted account through
  kept `DemoAct` rows (family 19), resent at each start. The playground's project is the
  seeded project 1; the start adds the account and adopts `playground/widget` with the
  person's sign-in; the scripted account has no uses.
- The box runs the playground's group (fake LLM, prepare, fake forge) unrecorded, from the
  document's setup entry 1.
- The client keeps "halves" (model, forge), a retired demo, a setup strip and Connect links
  on a demo banner; it removes the scripted account itself when another model is usable.
- `playground.md` (the browser playground) is a separate design; unbuilt but its addresses.

## The set up held in memory

- The first repository's intent lives in the forge domain's `first` map and the setup host's
  `Local.first` / `Run.first` (temper-service `setup/lifecycle.rs`), both in memory: a
  restart before the first repository loses it (setup.md, 6.7). The first project and
  adoption are the client's.

## The box

- Groups are fixed at the box's construction from the document; the pipe has `start N`,
  `stop N` (`started`, `stopped`, `failed`); `box/started/N` markers; no `box/groups/N`, no
  `remove N`; forge directories are configured paths (`forge-3`, `demo/forge`); Forgejo's
  binaries are in the cache (`forgejo/<VERSION>/forgejo`).

## jig and deleting a project

- No `RemoveProject`, no deletion state, no act that cancels any task in the deployment's
  name (only `Close` of a person's conversation). Retention's whole-task erasure (store.md,
  11.1) is unbuilt; the store's `DropTranscript` exists but no core write reaches it.
  Forgotten-once-read erasure exists (`jig-domain-core/src/forget.rs`). Application settings
  of a project's scope do not exist in code. jig's client has project settings pages
  (`/projects/N/settings`); temper's client has none.

## Earlier work

- `playground-apart.md` is not in the history; the draft says what it held. No earlier
  branch at or before the replayed run's start builds any of the draft. The replayed run's
  own output (and every replay) is not read, as the inputs' README says.
