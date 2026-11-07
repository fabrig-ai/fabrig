+++
id = "tui-world"
part = "tui"
kind = "feature"
needs = ["tui-host"]
files = [
  "temper/tests/client/src/component.rs",
  "temper/tests/client/src/front.rs",
  "temper/tests/client/src/fixtures.rs",
  "temper/tests/client/src/lib.rs",
  "temper/tests/client/src/tui.rs",
  "temper/tests/client/tests/suite/main.rs",
  "temper/tests/client/tests/suite/tui.rs",
  "temper/tests/client/Cargo.toml",
  "Cargo.lock",
]
effort = "high"
minutes = 190
+++

# The TUI's host in the client's joint world

## Goal

The meeting point of the TUI and the engine: the TUI's shipped host, its
root linked as a native shell holds it, run against temper's engine root
in `tests/client`. The world's front learns to carry a linked client's
exchanges and stream to the engine's clients' component as a transport
would, and the component makes a printed link. Three stories: a TUI signed
in by the pasted link drawing `me`; the engine restarted under it, Offline
then Live with nothing lost; signed out elsewhere, back at signing in with
its token erased.

Acceptance:

1. **The component** (`temper/tests/client/src/component.rs`):
   - `Composition::with_printed_link(configuration, limits, identity,
     secret) -> Composition`: as `new`, its clients' settings given
     `link: Some(identity)` and `link_secret: Some(secret)`; `new` is
     unchanged for its two callers (`front.rs` and
     `tests/suite/forges.rs`), both through one private constructor.
   - `link_client(&mut self, client: ClientId, limits: jig_client::DomainLimits)`:
     names `client` a linked client, its receiving limits set as
     `receiving` sets them.
   - `carry(&mut self, client: ClientId, request: jig_client::Request)`:
     an `Exchange` translated and sent as `link_request` sends a link's
     (`FromTransport::Exchange`, its local exchange kept in `exchanges`);
     `OpenStream` and `CloseStream` as `link_request` does them; `Save` and
     `Erase` answered `Stored` at once; `Told`, `Authorized` and
     `ServiceCredential` dropped.
   - `heard(&mut self, client: ClientId) -> Vec<jig_client::Event>`: what
     reached the linked client since last taken, in order. Every place the
     composition gives a client's link `Reply`, `Lost`, `Streamed` or
     `StreamGone` (`transport_reply`, `request`'s streamed and ended arms,
     `drop_watches`) gives a linked client the same `jig_client::Event`
     instead; any other link event for a linked client panics ("a linked
     client's own link speaks for it").
   - Unlinked clients behave exactly as before.
2. **The front** (`temper/tests/client/src/front.rs`), `Joint` gains:
   - `with_printed_link(seed, limits) -> Joint`: as `new`, its component
     made by `with_printed_link` with the identity `bind` installs (`key:
     user_key(1, 7)`, login "person", name "Person") and the secret
     `PRINTED`;
   - `pub const PRINTED: [u8; 32] = [9; 32]` and `printed_link() ->
     Box<[u8]>`, `http://127.0.0.1:8367/jig/v1/sign-in/link?secret=` and
     `PRINTED` in unpadded base64url;
   - `link_client(&mut self, who, limits)`, `carry(&mut self, who, now,
     request)` and `heard(&mut self, who, now) -> Vec<jig_client::Event>`,
     each advancing to `now` first, `carry` and `heard` pumping the
     component as `send` does. While the world is offline
     (`offline_until` ahead of `now`), `carry` answers an `Exchange` with
     `Lost { exchange }` and an `OpenStream` with `StreamGone` in the
     client's heard events, and sends nothing, as a transport that reaches
     no engine does.
3. `fixtures::tui_world(seed) -> World<Temper, Joint>`: `world_with(seed,
   Joint::with_printed_link(seed, domain_limits()))`.
4. **The world's TUI** (`temper/tests/client/src/tui.rs`, `pub mod tui` in
   `lib.rs`):

   ```rust
   //! The TUI's shipped host in the joint world (temper/docs/design/client/README.md, 6.13;
   //! temper/docs/design/domain/testing.md, 6.3).
   pub const ADDRESS: &[u8] = b"127.0.0.1:8367";
   pub struct Tui {
       pub host: temper_client_shell_tui::host::Host,
       pub who: ClientId,
       pub keeps: Vec<Option<skein_lib::Secret>>,   // every Duties::keep, in order
       pub address: Option<Box<[u8]>>,              // Duties::address
       pub closed: bool,
   }
   impl Tui {
       /// Links `who`, makes the host with `limits::tui()`, reports Start (80 by 24, Dark, ADDRESS, no records).
       pub fn start(world: &mut World<Temper, Joint>, who: ClientId, credential: Option<skein_lib::Secret>) -> Tui;
       /// Each key a report, the frame laid before it, as the person saw it.
       pub fn press(&mut self, world: &mut World<Temper, Joint>, keys: Vec<Key>);
       /// The world run for `span` in its own steps; after each, the host's clocks, every event heard
       /// as a report, Due when the deadline passed; every duty carried (asked to `carry`, keep and
       /// address recorded, close noted).
       pub fn run(&mut self, world: &mut World<Temper, Joint>, span: Duration);
       pub fn text(&mut self) -> String;     // the frame's text
       pub fn status(&mut self) -> String;   // the frame's last row, trimmed
       pub fn credential(&self) -> Option<&skein_lib::Secret>;   // the last keep's
   }
   ```

5. **The stories**, `temper/tests/client/tests/suite/tui.rs` (`mod tui;` in
   `main.rs`), focused, each in the report with its time:
   - `a_tui_signed_in_by_its_pasted_link_draws_me`: from `tui_world`, the
     TUI started with no credential shows "Sign in this terminal to
     127.0.0.1:8367" and "● Signed out"; the printed link pasted and
     Enter; within `fixtures::WITHIN` the frame says "Connected to
     127.0.0.1:8367 as Person", the projects' line `me` gives party 1
     (below), "● Live"; `keeps` is one credential; `address` is
     `ADDRESS`.
   - `the_engine_restarted_under_a_tui_goes_offline_then_live_and_keeps_me`:
     signed in as above; `FrontFault::Restart { offline: 300 ms }`; run in
     steps of 50 ms for two seconds, each frame still "Connected to
     127.0.0.1:8367 as Person": some frame's status "● Offline", a later
     one "● Live", the last "● Live"; `keeps` still one, nothing erased.
   - `a_tui_signed_out_elsewhere_asks_for_a_link_with_its_token_erased`:
     signed in as above; a second linked client carries `Exchange {
     credential: Some(the TUI's), message: Message::SignOut, .. }`, as
     another client of the same sign-in would; within `WITHIN` the TUI's
     frame says "Signed out." and "Sign in this terminal to
     127.0.0.1:8367", its status "● Signed out", and `keeps`' last is
     `None`.
6. `heavy cargo nextest run -p temper-client-world` passes whole (its
   default profile: the focused suite and `live_text`), and `heavy cargo
   check -p temper --tests` (temper's binary depends on this world).

## Inputs

- `temper/docs/plan/tui-connected/notes/common.md`.
- `temper/docs/plan/tui-connected/notes/contracts.md`, section 6 (the
  seam): it binds.
- `temper/docs/plan/tui-connected/notes/tui.md`, sections 2 and 5.
- `temper/docs/design/domain/testing.md`, 6.3 (the joint world, and its
  bullet *The TUI's host*); `temper/docs/design/client/README.md`, 6.13,
  6.14 and section 10's *The TUI in the joint world*; section 7 (the
  printed link).
- The world: `temper/tests/client/src/component.rs` (`link`,
  `link_request`, `transport_reply`, `request`, `drop_watches`, `detach`),
  `temper/tests/client/src/front.rs` (`with_engine_with_refused_hello`,
  `bind`, `send`, `pump_component`, `request`'s offline parking, `poll`,
  `fault`'s `Restart`), `temper/tests/client/src/fixtures.rs`
  (`world_with`), `temper/tests/client/tests/suite/to_begin.rs`
  (`skip_holds_across_a_restart`: a restart under live clients).
- jig's harness: `jig/testing/jig-client-kit/src/harness/world.rs`
  (`World::run`, `Until`, `now`, `front`, `fault`, `ClientId`).
- The engine's clients' component's printed link:
  `jig/crates/jig-protocol-engine-clients/src/component.rs` (`new`'s
  `link_secret`, `end_streams` at a sign-out) and
  `jig/testing/jig-client-protocol-kit/src/fixtures.rs` (`link:
  Some(identity(FIRST_OWNER))`).

## Owns and may change

**Owned:** the `files` above.

**Readers and neighbours, edits allowed:**

- `temper/tests/client/tests/suite/forges.rs`, line 123,
  `Composition::new`: unchanged, since `new` keeps its signature.
- `temper/tests/client/Cargo.toml`: the dependency lines
  `temper-client-shell-tui.workspace = true`,
  `temper-client-tui.workspace = true`, `skein-ui-screen.workspace =
  true` and `skein-ui-screen-terminal.workspace = true`, and
  `jig-client-protocol-http` if you parse the printed link there.

**Options, each with its default:**

- The TUI's `ClientId`: `ClientId(1_000)`, and `ClientId(1_001)` for the
  second client of the sign-out story; no harness client reaches them.
  If `World::run` with no harness client returns without moving time,
  open one harness client first (`world.client(fixtures::PARTY,
  fixtures::inbox())`) and say so.
- The host's wall: the front's engine's wall (`world.front().engine.env.wall`)
  at each step; if that is not public, `Wall::EPOCH` plus the world's
  `now`.
- The projects' line: the exact line the world's engine gives party 1,
  read from the configuration `Joint::new` runs
  (`temper_engine_domain_world::walking::config` and the fixtures'
  projects); with none, "No project yet". Name it in the report.
- The sign-out elsewhere: the default is the second client's `SignOut`
  exchange, which the clients' component answers by ending the sign-in's
  streams (`end_streams`), so the TUI hears `Streamed { SignedOut }`. If
  the component ends no stream there, give the TUI's stream the component's
  own `StreamEnded { why: Ended::SignedOut }` path for that sign-in, and say
  so; the assertions don't change.
- A linked client's `CloseStream` with no stream: nothing, as an
  unlinked link's.

## Steps

1. The component's linked clients (acceptance 1): a set of linked clients
   and their heard queues; one dispatcher in place of each `self.link(client,
   event)` that a linked client can reach.
2. The front (acceptance 2) and the fixture (acceptance 3).
3. The world's TUI (acceptance 4): `run` steps the world
   (`world.run(Until::Steps(1))` until `now` passes the span's end), then
   gives the host its clocks, each `heard` event as `Report::Heard`, and
   `Report::Due` when `host.deadline()` has passed, carrying each report's
   duties at once.
4. The stories (acceptance 5). Bite: make `drop_watches` skip linked
   clients once (the restart story then sees no "● Offline"), and make the
   host's credential duty skip `None` once, in `temper-client-shell-tui`'s
   `host.rs` (the sign-out story's last keep then is not `None`); see each
   fail for its reason, and restore both before you commit.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-world --all-targets -- -D warnings`; the three
   stories alone first (`heavy cargo nextest run -p temper-client-world
   --test suite -E 'test(/^tui::/)'`), naming their CPU time; then
   acceptance 6. Commit ("temper-client-world: the TUI's host on the
   engine, signed in by its pasted link"), push.

## Report

As the implementer's text says, and: the projects' line, how the sign-out
elsewhere reached the TUI, whether `World::run` needed a harness client,
the three stories' CPU time, and anything the seam lacked for a story
(named, not added).
