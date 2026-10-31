+++
id = "skein-setup-forges"
part = "runtime-forges"
kind = "interface"
needs = []
files = ["skein/crates/skein-forge-setup/**"]
effort = "high"
minutes = 60
+++

# skein-forge-setup's host: a forge added and removed at runtime

## Goal

temper's forges become records made at runtime
(`temper/docs/design/draft/playground-reset.md`, sections 3 and 4). The
setup host component takes its forges once, in `host::Settings::forges`,
at construction. Give it `Add` and `Remove` so that a forge made after
start can be set up and connected, and a removed one can run no plan.

Acceptance:

- `host::Request::Add { forge: host::Forge }` adds a forge as one in
  `Settings` is, with the same checks construction makes of each forge
  (`skein/docs/design/forge-setup.md`, 6.1 and 6.3: `reachable` within
  `inner`, the given values' checks). Refused, nothing changes and
  `host::Event::Refused { forge, why: AddRefused }` says why: its number is
  already a forge of the host (`Served`), the host holds its limit of forges
  (`Full`, a new limit `forges` in `host::Limits`, checked at `new` as the
  settings' count is today if it is not already), or a check failed
  (`Invalid`).
- `host::Request::Remove { forge }`: each run of that forge ends at once
  with its one `Event::Stop`, a new `Failure` reason `Removed` (or the
  closest existing shape; name it in the report), its connections are
  aborted, its one-time states (5.7) dropped, and then `Event::Removed {
  forge }`, once. A run asked for a removed or unknown forge stops at once
  as one for an unknown forge does today. Removing a forge not held answers
  `Removed` at once.
- Room is sized at `new` for `limits.forges`; `Add` reuses a removed
  forge's room; nothing grows at runtime.
- Tests in the crate (`src/host/tests.rs` or a new file beside it, on its
  fixtures): add then reconcile on the added forge; add refused for each
  reason; remove with a run waiting on a `Need`, which stops `Removed` and
  whose late `Answer` is ignored; remove then add the same number.

## Inputs

- `temper/docs/plan/playground-reset-r1/notes/common.md`.
- `skein/docs/design/forge-setup.md`: 6.1, 6.3, 6.5, and 5.7 for the
  states.
- The draft, sections 3 and 4 (why).

## Owns and may change

- Owns `skein/crates/skein-forge-setup/`.
- Readers outside: `temper/crates/temper-service/src/setup/` matches
  `host::Event` and builds `host::Limits`. Add the new arms and the new
  limit there, call sites only: the limit is the settings' forge count as
  today (or the existing bound, if one already exists), `Refused` and
  `Removed` are logged as an unexpected stop is today. temper's use of them
  is a later task's. Check with `heavy cargo check -p temper-service
  --tests` and `heavy cargo check -p temper --tests`.
- `skein/docs/design/` is not yours: report what 6.3 and 6.5 should say.

## Steps

1. Read `host.rs`, `host/run.rs`, `host/settings.rs`, `host/limits.rs` for
   how forges are looked up by number and which tables are per forge.
2. Sketch, in `host/boundary.rs`:

   ```rust
   pub enum Request {
       // ... as now
       /// One more forge, as one in the settings (draft playground-reset.md, 3).
       Add { forge: Forge },
       /// Ends a forge's runs and forgets it; `Event::Removed` answers.
       Remove { forge: u16 },
   }
   pub enum Event {
       // ... as now
       Refused { forge: u16, why: AddRefused },
       Removed { forge: u16 },
   }
   pub enum AddRefused { Served, Full, Invalid }
   ```

3. Tests as in Acceptance; `cargo fmt --all`; `heavy cargo clippy -p
   skein-forge-setup --all-targets -- -D warnings`; `heavy cargo nextest
   run -p skein-forge-setup`; the two `check`s above.
4. Commit and push your branch.

## Report

As the implementer text says, and what the design should now say.
