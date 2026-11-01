# Engine designer's progress

final: tasks/store-families.md
final: tasks/forge-asks.md
final: tasks/sk-conn-add.md
final: tasks/sk-setup-forges.md
final: tasks/sk-client-remove.md
final: tasks/sk-conn-remove.md
final: tasks/fr-limits.md
final: tasks/fr-surface.md
final: tasks/fr-seed.md
final: tasks/fr-forge-tables.md
final: tasks/fr-forge-added.md
final: tasks/fr-group-file.md
final: tasks/fr-make-forge.md
final: tasks/fr-connect.md
final: tasks/fr-setup.md
final: tasks/fr-first-intent.md
final: tasks/kept-acts.md

## Parts

- `skein-forges`: skein's forge crates add and remove a forge at runtime (draft 3, 4; mechanisms).
- `forge-records`: forges as records of the forge connector, numbered, seeded (D1), made by `SetUp`/`Connect`, limits as counts, the `forges` subject (draft 3; contracts 1, 2, 5, 6).
- `forge-removal`: `RemoveForge`, kept acts, records erased, the connector's `ProjectReleased`, a set up started over (draft 4, 6.4; contracts 1 D3/D4, 4).

## Critical path

(filled as the tasks are written)

## Decisions

- `Forge` row holds the document's three members typed (`temper_engine_domain_forge::record`), mirroring `settings::Forge`, `settings::Identity` and `temper_forge_setup::setup::Entry` field for field, so the service and the domain read the same typed record; the setup entry is typed, not kept as JSON bytes (the group file's JSON is rendered from it).
- `Forges` row is the inline variant `Stored::Forges { next }`: `boundary::Forges` is already the subject's type.
- The new families' tables are sized by `l.client.repositories`, the bound sizing `configured` today, until the limits task moves every per-forge table to `forge.forges`.
- `Configured::from_record` and `Forge::fits` land with the types in `store-families`, so the seed and the make path build on them at once.

- `forge-asks` needs `store-families`: both edit `temper-engine-domain-forge`'s `boundary.rs` and `domain.rs` (different enums and regions; the lead may release them together if fab's merge is trusted there, at the risk of a conflict).
- `Connect`'s forge is `ConnectForge::{Existing(u16), GitHub, Forgejo { address }}`: the person's input; the make path (`fr-make-forge`) turns it into a record. The client wire stays at `CLIENT_VERSION` 11, changed in place (client/README.md, 4.6).
- `SetUp`'s stub sets up the lowest owned asked forge (the seeded document's); `fr-setup` makes the record from `application.owned`.
- New answers: `RemoveAnswer::{Removing, Refused(NotOffered | NoSuchForge | Role)}`, `EndAnswer::{Reset, NotOffered}` (answered once the playground is gone); `StartPlayground` answers as `StartDemo` (`StartAnswer`).

- skein's runtime forges are methods (`Component::add`/`remove`, `Host::add`/`remove`, `skein_forge_client::remove_forge`), no new `Request`/`Event`/`Why` variant, so the skein branches force no temper arm; skein's `forge-setup.md` 6.5 (settings fixed for the host's life) is departed from, named in the commit for its designer.

- `fr-surface` lands the vocabulary of every later forge-records/forge-removal body (connector `Request::{ForgeKept, ForgeRemoved, SendKept, ProjectReleased}`, `Event::{SendKept, KeptAnswered, BoxRemoved, ReleaseProject}`, root `Request::{ForgeKept, ForgeRemoved, BoxRemove}`) routed to stubs in new module files (`kept.rs`, `release.rs`, `removal.rs`, `make.rs`, root `route/kept.rs`, service `forge/table.rs`), so the bodies own disjoint files and run in parallel instead of queueing on `domain.rs`.
- A kept `BoxRemove` answered `failed` stays kept (the box designer's contract problem, accepted by the lead): `Event::BoxRemoved { removed: false }`.

- The deployment-level `SetUp` offer the playground designer asked for is the client wire's `Forges.owned: Offer`, filled by the service (`fr-setup`).
- `fr-setup` edits the client wire after `pg-wire`; `pg-demo-out-wire` also owns the wire: the lead orders the two (suggested: `pg-demo-out-wire` needs `fr-setup`).
- `kept-acts` owns the root's `domain.rs` after `fr-seed`; `fr-setup` (one call there) needs `kept-acts` so the two never overlap.

## Contract problems

- Contracts 2 says the `Forge` row holds the document's members "less what is a credential's (a file-backed `api_token`, `webhook_secret`, a `user` by file)". 27 files (process stories, real-loop, one-process, client-browser, service fixtures) use the file form, which D1 must keep seeding. Proposed fix, cut so: the row keeps the file form's file *names* (`record::Files`: `api_token`, `webhook_secret`, `user` by id or file, the identity's `token`), never their bytes; the service reads the files at start as today. A records-only forge has `files: None`. temper-15 removes the file form.
