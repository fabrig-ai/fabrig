# Engine designer's progress

final: tasks/store-families.md
final: tasks/forge-asks.md

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

## Contract problems

- Contracts 2 says the `Forge` row holds the document's members "less what is a credential's (a file-backed `api_token`, `webhook_secret`, a `user` by file)". 27 files (process stories, real-loop, one-process, client-browser, service fixtures) use the file form, which D1 must keep seeding. Proposed fix, cut so: the row keeps the file form's file *names* (`record::Files`: `api_token`, `webhook_secret`, `user` by id or file, the identity's `token`), never their bytes; the service reads the files at start as today. A records-only forge has `files: None`. temper-15 removes the file form.
