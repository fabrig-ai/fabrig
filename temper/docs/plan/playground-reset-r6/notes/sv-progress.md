# sv's progress (temper's service and skein's forge layers)

| Spec | State | Needs |
|---|---|---|
| `sv-skein-connection` | final | none |
| `sv-skein-setup` | final | none |

(the rest of the part's specs are being written; see the requests below, which they depend on)

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
2. **A record's API host and TLS trust need io (T1's `Api`).** skein's connection and setup
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
3. **One setup entry parsed alone (T3).** serve builds a record's setup-host `Given`, its
   local reconcile entry and its fake/owned facts from the record's `setup` text plus `forge`
   and `directory` (`forge-N`), as the box does from `box/groups/N`. Both need
   `temper-forge-setup`'s entry parser for one entry, outside `Setup::read`'s list.
   **Recommendation:** bx's task that adds the per-group constructors exposes
   `temper_forge_setup::setup::Entry::read_one(text: &[u8], forge: u16, limits:
   &skein_forge_setup::limits::Limits, directories: &config::Directories) -> Result<Entry,
   Problems>` (name bx's); sv's registry task names that task in `needs`.
4. **`forge.forges` (T4).** It bounds the store's `Forge`/`ForgeNumbers` rows (fe) and serve's
   forge layer, setup host and tokens (sv). **Recommendation:** fe adds
   `temper_engine_domain_forge::Limits::forges` (with `4` in temper-service's `FORGE`
   constant, compiler-forced); sv's `sv-startup-machine` adds the override name
   `forge.forges` (`limits/names.rs`) and checks `forge_component.forges` and the setup
   host's `forges` at least `forge.forges` at start.
