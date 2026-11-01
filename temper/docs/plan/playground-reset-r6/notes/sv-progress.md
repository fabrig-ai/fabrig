# sv's progress (temper's service and skein's forge layers)

| Spec | State | Needs | Crates |
|---|---|---|---|
| `sv-skein-connection` | final | none | skein-forge-connection, skein-forge-machine-world |
| `sv-skein-setup` | final | none | skein-forge-setup, skein-forge-setup-world |
| `sv-family` | final | cl-wire, fe-surface | temper-service (family) |
| `sv-startup-machine` | final | sv-family, fe-surface, bx-steps | temper-service (startup) |
| `sv-registry` | final | sv-startup-machine, sv-skein-connection, sv-skein-setup | temper-service, temper-protocol-records |
| `sv-registry-setup` | final | sv-registry, bx-steps | temper-service (setup) |
| `sv-resolve` | final | sv-registry-setup | temper-service; temper's `src/host.rs` |
| `sv-removal` | final | sv-resolve, bx-pipe | temper-service, temper-protocol-records; temper's `src/host.rs` |
| `sv-playground` | final | sv-removal, fe-kept-root, fe-records, cl-wire | temper-service |
| `sv-seed-kit` | final | fe-records | temper-github-app (testing) |
| `sv-world-one-process` | final | sv-seed-kit, sv-registry-setup | temper-one-process-world |
| `sv-world-real-loop` | final | sv-seed-kit, sv-registry-setup | temper-real-loop-world |
| `sv-world-client-browser` | final | sv-seed-kit, sv-registry-setup, bx-init | temper-client-browser-world |
| `sv-service-fixtures` | final | sv-playground | temper-service (tests) |
| `sv-startup-refuse` | final | sv-service-fixtures, sv-world-one-process, sv-world-real-loop, sv-world-client-browser, bx-init, bx-fixtures, bx-walks | temper-service; temper's `lib.rs`/`setup.rs` call sites |

`fe-records` and `fe-kept-root` are named from their task files (not yet marked final in
`notes/fe-progress.md` when written): release `sv-playground` and `sv-seed-kit` after them.
temper-system-world builds a `settings::Forge` literal only (no document): no task of its
own; `sv-startup-refuse` runs it.

## Task graph and critical path

- Parallel from the start: `sv-skein-connection`, `sv-skein-setup` (skein only).
- temper-service has one writer at a time, so its tasks are a chain:
  `sv-family` → `sv-startup-machine` → `sv-registry` → `sv-registry-setup` → `sv-resolve` →
  `sv-removal` → `sv-playground` → `sv-service-fixtures` → `sv-startup-refuse`.
  `sv-family` and `sv-startup-machine` go first because cl's world and bx's chain
  (`bx-doc` → `bx-init`, `bx-fixtures`) wait for them; `sv-registry` also waits for the two
  skein tasks.
- Off the chain, in parallel once `sv-registry-setup` and `fe-records` land: `sv-seed-kit`,
  then the three worlds (one crate each).
- Critical path (implementer minutes, after `cl-wire` and `fe-surface`): the service chain,
  about 50 + 110 + 120 + 110 + 80 + 90 + 70 + 120 + 120 = 870, gated near its end by
  `fe-kept-root` (for `sv-playground`) and by bx's `bx-fixtures`/`bx-walks` and the worlds
  (for `sv-startup-refuse`).

## Contract requests

1. **How the trunk stays green while forges leave the document (T4).** Refusing
   `application.forges`/`identities`/`setup` breaks, at once, every world and fixture that
   names a forge (tests/one-process, tests/real-loop, tests/system, tests/client-browser,
   temper's process fixtures, temper-init's template and `docs/examples/owned.json`, the box's
   steps read from `application.setup`), and the engine's forges then exist only as records
   made by `MakeForge` and restored by `Register`. No single task can carry all of it.
   **Recommendation:** additive first, refusal last:
   - (a) the records path works beside the document's forges: fe's connector projects
     `Configured` from `Config.forges` (the document's) and from `Kept` records, sends
     `Register` for each record at restore and on make, and refuses `MakeForge` `NotOffered`
     while `Config.forges` is not empty (so numbers never collide); serve's registry is
     seeded by the document's forges at start and fed by `Register`/`Unregister` (sv);
   - (b) `sv-startup-machine`: startup accepts `application.machine`, `forge.forges`, the new
     `application.demo` shape and a document with no forge (beside the old members), and
     hands the root `Machine`;
   - (c) the worlds, bx's init template, `owned.json` and process fixtures move to the new
     document (forges made by `make_forge` or `MakeForge`), in parallel tasks;
   - (d) `sv-startup-refuse`: the three members refused naming the member, the document's
     forge readers deleted, the transitional seeding gone; fe drops `Config.forges` in the
     same step (compiler-forced) or after.
   Until answered, sv's specs follow this order and name fe's tasks by the role they play
   (filled in from `notes/fe-progress.md` when it has ids).
2. **A record's API host and TLS trust need io (T1's `Api`).** *Taken by `sv-resolve` (see 7: `lib.rs` is not edited).* skein's connection and setup
   host take a resolved socket address and a built `skein_tls::Config`; today the binary
   resolves each document forge's host (`to_socket_addrs`) and reads its trust file before
   the loop (`temper/crates/temper/src/lib.rs`, `forges()` and `trust()`), and an unresolved
   host is held (`temper-service/src/forge.rs`, `HELD_ADDRESS`). A record made at runtime
   (`MakeForge` of a Forgejo elsewhere, GitHub's `api.github.com`) has neither.
   **Recommendation:** serve asks its host, as it asks for handover reads: the service gains
   `take_resolve(&mut self) -> Option<crate::Resolve>` (`Resolve { forge: u16, host:
   Box<[u8]>, port: u16, tls: Option<(Box<[u8]>, Option<Box<[u8]>>)> }`, server name and trust
   path) and `resolved(&mut self, forge: u16, Result<crate::ForgeRuntime, Box<[u8]>>)`; the
   binary's host loop (`temper/crates/temper/src/host.rs`, where `read_handover` is) performs
   it with today's `address()` and `trust()`. An IP literal host with plaintext needs no host
   and resolves in serve. Until resolved, or on a refusal, the forge is held, as an
   unresolved host is today (`Note::ForgeNotConnected { Unreachable }`). sv's task edits
   `host.rs` and `lib.rs` for this alone, ordered after bx's tasks that touch those files.
   Until answered, sv's spec follows this recommendation.
3. **One setup entry parsed alone (T3).** *Answered by `bx-steps` (`Entry::body_text`).* serve builds a record's setup-host `Given`, its
   local reconcile entry and its fake/owned facts from the record's `setup` text plus `forge`
   and `directory` (`forge-N`), as the box does from `box/groups/N`. Both need
   `temper-forge-setup`'s entry parser for one entry, outside `Setup::read`'s list.
   **Recommendation:** bx's task that adds the per-group constructors exposes
   `temper_forge_setup::setup::Entry::read_one(text: &[u8], forge: u16, limits:
   &skein_forge_setup::limits::Limits, directories: &config::Directories) -> Result<Entry,
   Problems>` (name bx's); sv's registry task names that task in `needs`.
4. **`forge.forges` (T4).** *Answered by `fe-surface` (`Limits::forges`); `sv-startup-machine` adds the override.* It bounds the store's `Forge`/`ForgeNumbers` rows (fe) and serve's
   forge layer, setup host and tokens (sv). **Recommendation:** fe adds
   `temper_engine_domain_forge::Limits::forges` (with `4` in temper-service's `FORGE`
   constant, compiler-forced); sv's `sv-startup-machine` adds the override name
   `forge.forges` (`limits/names.rs`) and checks `forge_component.forges` and the setup
   host's `forges` at least `forge.forges` at start.
5. **`fe-records` would turn the trunk red (request 1, made concrete).** `fe-records` drops
   `root::Config.forges` and `identities` and expects temper-service's and the one-process
   world's tests that read configured forges to fail after it ("they are sv's to move to
   records", its Report). Every document-driven deployment and world (temper-service's
   tests, tests/one-process, tests/real-loop, the binary's process fixtures, a fresh `temper
   init`) configures its forges in the document until the worlds move (request 1 (c)), so the
   engine would know none of them. Two facts make it worse: the engine's git identity becomes
   `credential: forge as u32`, while serve keys each document identity by its own number
   (`temper init` writes forge 1 with identity 2 and forge 2 with identity 1), so a worker's
   credential lookup would miss. **Recommendation:** `fe-records` keeps the trunk green by
   itself, two small additions in its task:
   - (a) a transitional seed: `root::Config` gains `seed: Box<[forge::ForgeRecord]>` (doc:
     "the document's forges, until sv-startup-refuse"); temper-service's `system.rs` fills it
     from today's `startup::application::configured(&settings)` through a pub
     `ForgeRecord::from_configured(&Configured)` in the forge crate (fe-records' world helper,
     moved there); at restoration's end the connector saves each seed record whose number
     has no restored `Forge` row, and moves `ForgeNumbers` past the highest, before it sends
     the `Register`s. So a document's forges become records at their first start.
   - (b) serve keys a document identity's values by its forge's number (`values.rs::fill`,
     `translate::identity`'s number, `setup/settings.rs::local_starts`' identity number,
     `forge/tokens.rs`), the four sites the compiler will not name, so that the worker's
     credential `forge as u32` is found.
   serve's registry (`sv-registry`) treats a `Register` of a number the document configures as
   the document's forge (only its mode taken), so the seeded records change nothing in
   serve. `sv-startup-refuse` removes the seed and the document path together. If the lead
   prefers, (b) can be an sv task that `fe-records` names in `needs`, landing in the same
   merge window; it cannot land alone (before `fe-records` the root still uses the identity
   numbers).
6. **`bx-doc` edits a temper-service test file** (`src/tests/startup.rs`, `check::checked`'s
   new `Option`), concurrently with `sv-registry` (both after `sv-startup-machine`): two
   writers of temper-service at once. **Recommendation:** `bx-doc` keeps
   `check::checked(.., at: u32)` and adds `check::checked_at(.., at: Option<u32>)`, so it
   writes no temper-service file; `sv-startup-refuse` moves the call. Until answered, sv's
   specs assume this.

7. **Answered by the lead (recorded):** `failed N ...` during `remove N` is "not removed"
   (`sv-removal`); `application.machine.git_program` goes to `Machine.github.git_program`
   (`sv-startup-machine`); `sv-startup-refuse` needs `bx-init`, `bx-fixtures`, `bx-walks`;
   sv's tasks may edit `temper/crates/temper/src/host.rs` (so `sv-resolve` no longer needs
   `bx-init`: it calls `lib.rs`'s private `address`/`trust`/`server_name` from `host.rs`
   without editing `lib.rs`); a Forgejo elsewhere has a managed setup entry (T4), which
   `sv-registry` and `sv-registry-setup` follow.
