+++
id = "skein-connection-forges"
part = "runtime-forges"
kind = "interface"
needs = []
files = ["skein/crates/skein-forge-connection/**", "skein/tests/forge/**"]
effort = "high"
minutes = 90
+++

# skein-forge-connection: a forge added and removed at runtime

## Goal

temper's forges become records made at runtime
(`temper/docs/design/draft/playground-reset.md`, sections 3 and 4), but
skein's forge component takes its forges once, in `Settings::forges` at
`Component::new`, and holds them until close. Give the component two
requests, `Add` and `Remove`, so that its owner can serve a forge made
after start and end one that is removed, within `Limits::forges`.

Acceptance:

- `Request::Add { forge: Forge }` adds a forge as one handed in `Settings`
  is: with access, it starts as a configured forge with access starts
  (`skein/docs/design/forge-connection.md`, 3.3); without, its calls are
  answered `NotConnected` until a `Connect`. Refused, nothing changes and
  one `Event::Refused { forge, why: AddRefused }` says why: its number is
  already served (`Served`), the forges served are at `Limits::forges`
  (`Full`), or the forge would not pass start's checks (`Invalid`, the
  same checks as 3.2 and 7.4 make of a configured forge: address, transport,
  a GitHub forge's log origins within `log_origins`).
- `Request::Remove { forge }` ends a forge: each of its calls still open
  ends with its one `Event::Answered`, `NotConnected`, as at a replacement
  (3.5), its start and renewals stop, its installations and git tokens
  are dropped, its webhook path answers as an unknown forge's does (6.3),
  and once nothing of it is in flight `Event::Removed { forge }` says so,
  once. Its slot is free for a later `Add`, of the same number or another.
  A `Remove` of a forge not served is answered `Event::Removed` at once.
  No event of the removed forge follows its `Removed`.
- `Request::Connect`, `Installations` and `Call` for a removed forge are as
  for one never configured (ignored, and a call answered `NotConnected`).
- Memory stays bounded by `Limits::forges`: the per-forge tables are
  sized once at `new` for `limits.forges` (as `component.rs` sizes
  `github`, `starts`, `own_tokens`, `own_plans` today), and an `Add`
  reuses a removed forge's room.
- Tests in the crate's own tests (`src/tests/`), on its existing fixtures:
  add with and without access, then calls; add refused for each reason;
  remove with a call in flight, an installation token and a webhook path,
  then `Removed` once and the call answered; remove then add the same
  number again; add at the limit after a remove.

## Inputs

- `temper/docs/plan/playground-reset-r1/notes/common.md`.
- `skein/docs/design/forge-connection.md`: 2.1, 3.1 to 3.5, 6.1, 6.3, 7.2,
  7.4, 8.1 and 8.5.
- The draft, sections 3 and 4 (why; nothing in temper changes here).

## Owns and may change

- Owns `skein/crates/skein-forge-connection/` and its tests, and the forge
  world `skein/tests/forge/` if a case there pins `Settings` being fixed.
- Readers outside: `temper/crates/temper-service` matches
  `skein_forge_connection::Event` (and may match `Request`). Add the new
  variants' arms there, call sites only: `Refused` and `Removed` are logged
  as a `Fact` is today, nothing more; temper's use of them is a later
  task's. Check with `heavy cargo check -p temper-service --tests`.
- If a new variant must be named `Event::Refused` differently because the
  name is taken, choose the name and say so in the report.
- `skein/docs/design/` is not yours: report what the design should say.

## Steps

1. Read `component.rs` for how `Settings::forges` is indexed (`at`
   positions) and which tables are sized per forge; make each forge's slot
   reusable (a free slot, or `Option` per slot) without growing at runtime.
2. Sketch, in `boundary.rs`:

   ```rust
   pub enum Request {
       // ... as now
       /// Serves one more forge, as one handed at construction (draft playground-reset.md, 3).
       Add { forge: crate::settings::Forge },
       /// Ends a forge whole; `Event::Removed` answers once nothing of it is in flight.
       Remove { forge: u16 },
   }
   pub enum Event {
       // ... as now
       Refused { forge: u16, why: AddRefused },
       Removed { forge: u16 },
   }
   pub enum AddRefused { Served, Full, Invalid }
   ```

3. Tests as in Acceptance, then `cargo fmt --all`, `heavy cargo clippy -p
   skein-forge-connection --all-targets -- -D warnings`, `heavy cargo
   nextest run -p skein-forge-connection`, the forge world's tests if you
   touched it, and `heavy cargo check -p temper-service --tests`.
4. Commit and push your branch.

## Report

As the implementer text says; name the per-forge tables you made reusable
and anything the design should now say (3.1, 7.4, 8.1, 8.5).
