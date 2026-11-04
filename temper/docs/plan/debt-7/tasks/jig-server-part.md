+++
id = "jig-server-part"
part = "service-clients"
kind = "migration"
needs = ["jig-workers-host"]
files = ["jig/crates/jig-protocol-engine-clients-http/Cargo.toml", "jig/crates/jig-protocol-engine-clients-http/src/lib.rs", "jig/crates/jig-protocol-engine-clients-http/src/server.rs", "jig/crates/jig-protocol-engine-clients-http/src/server/policy.rs", "jig/testing/jig-test-service/src/server.rs", "jig/testing/jig-test-service/src/server/bundle.rs", "jig/testing/jig-test-service/src/server/policy.rs", "jig/testing/jig-test-service/src/clients.rs", "jig/testing/jig-test-service/src/lib.rs", "jig/testing/jig-test-service/src/limits.rs", "jig/testing/jig-test-service/src/tests/server.rs", "jig/testing/jig-test-service/src/tests/fixtures/bundle.rs", "jig/tests/boundary/src/tests.rs", "jig/docs/design/protocol/service.status.md", "jig/docs/design/protocol/client.status.md", "Cargo.lock"]
effort = "high"
minutes = 120
+++

# jig: the server as product code

## Goal

`J/server.rs` and `J/server/policy.rs` (`J` and `T` are the paths `notes/service-parts.md` gives) move to
`jig_protocol_engine_clients_http::server`, in one branch. The shipped server is temper's form: it hands every call to
its owner as `Routed::Call` and answers nothing itself. jig's bundle stays in `jig-test-service`, as `Pages`, an adapter
that takes every call the way temper's `clients/bundle.rs` does. `temper-server-part` deletes temper's copy.

Acceptance:

- `jig_protocol_engine_clients_http::server` exists with the outer layer below, `policy` under it.
- `J/server.rs` holds `mod bundle`, its re-exports and nothing else; `J/server/policy.rs` is gone.
- `jig-test-service`'s tests and the worlds of step 9 pass, changed only as step 7 says. No test moves.
- jig's outside readers of `jig_test_service::server` (eight files, about 115 lines) are not edited.
- `jig/tests/boundary/src/tests.rs`, `engine_http_links_only_its_component_and_foundation`, names `skein-io`.
- `jig/docs/design/protocol/service.status.md` no longer lists the server. `client.status.md`'s 17.4 **Departs** is true
  of the new paths (step 8).
- **Size cap:** the branch's net lines are at most +212 (`## Answers`), tests and manifests included. It rises because the shipped
  server carries temper's `hold`, `finish` and `valid`, a hand-written `Debug` and two tests jig's copy never had;
  `temper-server-part` then falls by about 380. Measure it, and stop and ask if it's passed.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `temper/docs/plan/debt-7/notes/service-parts.md`, `AGENTS.md`, `jig/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11 and 11.4.
- `jig/docs/design/protocol/client.md`, 17.4; `jig/docs/design/protocol/browser.md`, 12 and 12.1.
- `temper/crates/temper-service/src/server.rs`: the code you ship. `temper/crates/temper-service/src/clients/bundle.rs`,
  `serve_call`, `serve_event`, `demand` and `chunk`: the form `Pages` takes.
- `jig/docs/development/debt/D005-accounts-host-copied.md`, for what this pays; this task doesn't touch it.

## Owns and may change

- Owns: `jig-protocol-engine-clients-http`'s new `src/server.rs` and `src/server/policy.rs`, its `pub mod server;`
  and its manifest's `skein-io.workspace = true`; `J/server.rs`, `J/server/bundle.rs`, the deletion of `J/server/policy.rs`.
- May change, in the way listed:

  | File | The edit |
  | --- | --- |
  | `J/clients.rs` | the `pages` field and its `new` line, the two arms of `server_requests` (step 6), one `.field` in its `Debug` |
  | `J/lib.rs` | `Service::serve`: `clients.pages.serve(bundle)` |
  | `J/limits.rs` | `peer_end: crate::server::PeerEnd::Call` in `for_profile`'s literal |
  | `J/tests/server.rs` | `with_no_bundle_a_page_get_is_404`'s constructor; `the_bundle_is_installed_before_the_service_loop`'s `clients.pages`; two new tests (step 7) |
  | `J/tests/fixtures/bundle.rs` | `serving`, `answer`, `wire_response` (step 7) |
  | `jig/tests/boundary/src/tests.rs` | `"skein-io"` in the one list named above, sorted |
  | `jig/docs/design/protocol/service.status.md`, `client.status.md` | as step 8; this departs from the implementer's text for those two files alone |
  | `Cargo.lock` | what the manifest line brings |

- Checked by reading, needing no edit: `J/tests/fixtures/server.rs` and the lifecycle tests of `J/tests/server.rs` (only
  public methods); `J/clients.rs`'s `crate::server::{Server, Routed}`, 7 sites; `J/limits.rs`'s `crate::server::Limits`;
  `jig/testing/jig-test/src/bundle.rs` (`Bundle`, `File`, `kind`); `jig/tests/engine-clients/{src/fixtures.rs, src/peer.rs,
  src/peer/public_http.rs, tests/suite/public_http.rs}` (`policy::FIELDS` among them); `jig/tests/client-protocol/src/{engine,
  fixtures}.rs`; `jig/tests/ops/src/joint/bytes.rs`. All read `jig_test_service::server`, which re-exports.
- **A world that sends an unclaimed call.** Those three worlds drive the server with no bundle, and hand every
  `Routed::Call` to the mount. By reading, each sends only targets under `/jig/`, and `POST` the link. If a world's test
  goes red because it expected the server's own 404 or 405, route that world's unclaimed calls through a `Pages` with
  no bundle, as `J/clients.rs` does, and say so in the report.
- Not yours: `T`. Temper's `clients/policy.rs` keeps its own `add` until `temper-server-part`.

## What differs, and which holds

From `notes/service-parts.md`, as it falls here. All of it is temper's form but the first line.

- **A peer's end during a call:** the `peer_end` value. jig's is `PeerEnd::Call`, its behaviour today.
- **The queues:** `Queue::uncapped()` for every connection's queue, as jig has them; temper's `with_capacity(8)` doesn't ship.
- **`held`, `hold`, `release`, `finished`, `finish`, `valid`:** ship; jig never calls them.
- **`Closed` on the listener** clears `listening`, as temper's does. No jig test reaches it.
- **The bundle:** outside the server. `Pages` keeps jig's answers and jig's one difference from temper's: a reply that
  fails aborts its connection, with `abandon`, where temper's ends it gracefully with `done`.
- **Policy:** `FIELDS`, jig's five, the same bytes as temper's `Page::Service`.
- **Limits:** values, as they are.

`Clients::up`'s listener failure, the listener's `Closed` and `kit::Request`'s `unreachable!` are the service's: unchanged.

## Credentials

Both copies derive `Debug` over `http::Call`, which prints every header, cookies among them, and over `Stream`'s bytes.
There is nothing to preserve, and the shipped type holds credentials in the sense `service-parts.md` means, so it gets
temper's hidden form: this is an addition. `Server` and `Routed` get the hand-written `Debug` below; `Connection`, private,
gets none. `Limits` and `PeerEnd` derive. `J/clients.rs`'s hand-written `Debug` keeps showing `server` and `server_out`,
now by counts.

## Steps

1. Manifest, and `pub mod server;` in `lib.rs`. `src/server.rs`'s module doc cites `jig/docs/design/protocol/service.md, 11.4`.
2. `src/server.rs` is `T/server.rs` without its one test, with: `Queue::with_capacity(8)` as `Queue::uncapped()` at every
   site, `end` included; `pub` for `hold` and `release`; `policy::add` for `crate::clients::policy::add`; no `derive(Debug)`
   on `Routed`, `Connection` or `Server`; and the `Stream` arm's `ended` read as below. The outer layer:

   ```rust
   pub mod policy;

   #[derive(Clone, Copy, Debug)]
   pub struct Limits {
       pub connections: u32,
       pub http: http::Limits,
       pub stall: Duration,
       /// What a peer's end does while a call is unfinished (protocol/client.md, 17.4).
       pub peer_end: PeerEnd,
   }
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub enum PeerEnd {
       /// Ends the connection, and the owner hears it closed.
       Connection,
       /// Leaves the connection; the owner hears the call's `Ended`.
       Call,
   }
   pub enum Routed {
       Io(io::Request),
       Call { call: Token, head: http::Call },
       Http { call: Token, event: http::Event },
   }
   pub struct Server { .. }
   impl Server {
       pub fn new(listener: io::Addr, limits: &Limits) -> Self;
       pub fn listening(&self) -> Option<SocketAddr>;
       pub fn opened(&self) -> bool;
       pub fn closed(&self) -> bool;
       pub fn up(&mut self, event: io::Event, out: &mut Queue<Routed>);
       pub fn hold(&mut self, call: Token, deadline: Time);
       pub fn release(&mut self, call: Token);
       pub fn answer(&mut self, call: Token, request: http::Request, out: &mut Queue<Routed>);
       pub fn done(&mut self, call: Token, out: &mut Queue<Routed>);
       /// Ends the call's connection at once, as a stall does.
       pub fn abandon(&mut self, call: Token, out: &mut Queue<Routed>);
       pub fn stop(&mut self, out: &mut Queue<Routed>);
       pub fn close(&mut self, out: &mut Queue<Routed>);
       pub fn finish(&mut self);
       pub fn abort(&mut self, out: &mut Queue<Routed>);
       pub fn next_deadline(&self) -> Option<Time>;
       pub fn clock(&mut self, now: Time, wall: Wall);
       pub fn fire(&mut self, out: &mut Queue<Routed>);
       pub fn advance(&mut self, out: &mut Queue<Routed>) -> bool;
   }
   pub fn valid(limits: &Limits) -> bool;
   ```

   ```rust
   let ended = match &up {
       Up::End => self.env.limits.peer_end == PeerEnd::Connection && connection.call.is_some(),
       Up::Bytes(_) | Up::Room | Up::Failed(_) => false,
   };
   ```

   `abandon` finds the owner as `hold` does and calls `end(connection, false, out)`.

3. `Debug`, each with a comment saying a head holds cookies and a response `Set-Cookie`:

   ```rust
   impl core::fmt::Debug for Server { /* address, listener, phase, connections as `self.connections.len()` */ }
   impl core::fmt::Debug for Routed { /* `Io` alone; `Call` and `Http` with `call` alone */ }
   ```

4. `src/server/policy.rs` is `J/server/policy.rs`, with `add` `pub`: `temper-server-part`'s test reads it.
5. `J/server.rs` becomes:

   ```rust
   //! The test application's server: jig's server part, with its bundle (protocol/service.md, 7 and 11.4).
   mod bundle;
   pub use bundle::{Answer, Bundle, File, Pages, kind};
   pub use jig_protocol_engine_clients_http::server::{Limits, PeerEnd, Routed, Server, policy};
   ```

   `J/server/bundle.rs` keeps `File`, `Bundle`, `kind`, `Answer` (it moves here from `J/server.rs`), `target`, `route` and
   `Bundle`'s `file` and `content`. `start`, `demand` and `send` become `Pages`'s, over `Server::answer`, as temper's
   `serve_call`, `demand` and `chunk` are:

   ```rust
   /// The test application's answers to calls the mapping doesn't claim (protocol/service.md, 7).
   #[derive(Debug)]
   pub struct Pages { bundle: Option<Bundle>, replies: Map<Token, Reply>, send: u32 }
   impl Pages {
       pub fn new(limits: &Limits) -> Self; // `replies` bounded by `limits.connections`, `send` from `limits.http`
       /// The bundle the startup read; none serves no browser.
       pub fn serve(&mut self, bundle: Bundle);
       pub fn route(&self, head: &http::Call) -> Answer;
       /// Starts the answer to a call `route` gave no `Mount`.
       pub fn call(&mut self, server: &mut Server, call: Token, head: &http::Call, answer: Answer, out: &mut Queue<Routed>);
       pub fn owns(&self, call: Token) -> bool;
       /// One event of a call being answered.
       pub fn event(&mut self, server: &mut Server, call: Token, event: http::Event, out: &mut Queue<Routed>);
   }
   ```

   `call` sends `Discard`, then `Respond`, then the first demand, through `server.answer`, and adds no policy: `answer`
   does. `event`'s arms are `J/server.rs`'s, in `event`, with the reply taken out of `replies`:

   ```rust
   http::Event::Reply(Up::Room) => /* the next chunk */,
   http::Event::Done(_) | http::Event::Ended => { self.replies.remove(&call); server.done(call, out); }
   http::Event::Refused(_) | http::Event::Failed(_) | http::Event::Closed
   | http::Event::Reply(Up::Failed(_)) | http::Event::Body(Up::Failed(_)) => {
       self.replies.remove(&call);
       server.abandon(call, out);
   }
   http::Event::Body(_) | http::Event::Reply(_) => {}
   http::Event::Call(_) => unreachable!("a head is routed, not an event"),
   ```

6. `J/clients.rs`: `pub(super) pages: crate::server::Pages`, made in `new` from `transport`. In `server_requests`:

   ```rust
   crate::server::Routed::Call { call, head } => {
       let answer = self.pages.route(&head);
       if answer != crate::server::Answer::Mount {
           self.pages.call(&mut self.server, call, &head, answer, &mut self.server_out);
       } else if self.mount.room_for(&head) > 0 {
           /* as it stands */
       } else {
           /* as it stands */
       }
   }
   crate::server::Routed::Http { call, event } => {
       if self.pages.owns(call) {
           self.pages.event(&mut self.server, call, event, &mut self.server_out);
       } else if self.refused.get(&call).is_some() {
           /* as it stands */
       } else {
           /* as it stands */
       }
   }
   ```

7. Tests. No test moves: `J/tests/server.rs`'s lifecycle tests read only public methods, so they stay and now pin the part.
   - `J/tests/fixtures/bundle.rs`: `serving` returns a fixture type holding a `Server` and a `Pages`, with a `route` that
     asks the `Pages`, so `each_path_is_answered_by_its_part_of_the_server` doesn't change. `answer` takes that type. It
     and `wire_response` share a pump in which `Routed::Call` goes to `pages.call` and `Routed::Http` to `pages.event`
     while `pages.owns(call)`; `wire_response` on a bare `Server` keeps its behaviour, and its `panic!` on a
     `Routed::Call` stays for that case.
   - `with_no_bundle_a_page_get_is_404` takes that type with no bundle: a constructor, its assertions unchanged.
   - Add `a_peers_end_during_a_call_follows_its_value`: over `f::server_call`, `Up::End` with `PeerEnd::Call` gives no
     `Abort` and hands on `Ended`; with `PeerEnd::Connection` it gives the `Abort`.
   - Add `debug_of_the_server_shows_no_cookie`, in the form of `debug_of_the_clients_shows_the_links_count_never_a_link`:
     a request with a `Cookie` field holding a canary, then neither `{server:?}` nor `{out:?}` holds the canary as text
     or as bytes. Each new test bites: a `derive(Debug)` on `Routed` for the second, and `ended` ignoring `peer_end` for
     the first.
8. Status. `service.status.md`: remove `the server` from **Not yet**, 11; date line. `client.status.md`, **Departs**, 17.4:
   the reference server, by its `PeerEnd::Call` (`jig/testing/jig-test-service/src/limits.rs`), hands a peer's end to the
   engine end as the call's `Ended` and does not end the connection itself
   (`jig/crates/jig-protocol-engine-clients-http/src/server.rs`), and the rest of the sentence as it stands; date line.
9. Checks, one at a time: `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-protocol-engine-clients-http -p jig-test-service --all-targets -- -D warnings`; `heavy cargo nextest run -p
   jig-protocol-engine-clients-http -p jig-test-service -p jig-test -p jig-boundary -p jig-one-process-world -p
   jig-channel-world -p jig-engine-clients-world -p jig-client-protocol-world -p jig-ops-world`; `heavy cargo check -p
   temper-service --tests`; the citations check.
10. **The browser tier.** jig's browser tier runs this server and never runs in the gate. Run `TIERS_WRAPPER=heavy
    tools/tiers/run jig browser`, the pair `browser-jig`, once, after step 9, and report its counts. It's a check you
    run and report: red is yours to read and fix, as any test of yours; if the tier can't start in your slot, say so in the
    report, and it doesn't stop your acceptance.
11. Commit and push.

## Report

The common note's items, with the two tests' breaks, the browser tier's counts, and net lines against the cap.

## Answers

2026-11-04. Q: over `f::server_call`, `PeerEnd::Call` gives no `Ended`, where step 7 and service.md, 11.4 say it does:
keep the behaviour, or emit a new `Ended`? And compact to +170, or raise the cap?

**Keep the behaviour; no new `Ended`.** The request says behaviour stays unchanged, `notes/service-parts.md` says each
service keeps its own, and `skein/crates/skein-http/src/server.rs`, `exchange_up`'s `Up::End` arm is what
`jig/docs/design/protocol/client.md`, 17.4 and `skein/docs/design/http.md`, 5.5 describe: the response goes on and
nothing is said above. Nothing in jig's copy ever made an `Ended` of it: after a call, `Ended` meets `connection.call`
empty. So step 7's assertion, step 2's doc comment and step 8's status sentence were wrong, not the code. This answer
corrects `jig/docs/design/protocol/service.md`, 11.4 on the trunk. Change nothing in skein.

- **Step 2:** `PeerEnd::Call`'s doc comment is `/// Leaves the connection: the response goes on, and the owner hears the
  call end with it.`
- **Step 7:** `a_peers_end_during_a_call_follows_its_value` loses `ended` and its match, and ends:

  ```rust
  server.up(skein_io::Event::Stream { owner: OWNER, up: skein_lib::stream::Up::End }, &mut out);
  f::server_drain(&mut server, &mut out);
  let abort = |routed: &Routed| matches!(routed, Routed::Io(skein_io::Request::Abort { entity }) if *entity == SOCKET);
  assert_eq!(out.iter().any(abort), peer_end == PeerEnd::Connection, "{peer_end:?}: connection end");
  assert!(!out.iter().any(|routed| matches!(routed, Routed::Http { .. })), "{peer_end:?}: nothing said above");
  ```

  Its break stays: `ended` ignoring `peer_end` makes the first assertion red under `PeerEnd::Call`.
- **Step 8:** `client.status.md`, **Departs**, 17.4 reads, whole: "17.4: the reference server, by its `PeerEnd::Call`
  (`jig/testing/jig-test-service/src/limits.rs`), leaves a peer's end during a mounted call to skein's server and does
  not end the connection itself (`jig/crates/jig-protocol-engine-clients-http/src/server.rs`): a request all read goes
  on to its response, the engine end hears nothing of that end, and a stream's end is seen at a failed write or the
  stall." The clause on `call.rs` goes: it described an `Ended` that doesn't come.

**The cap becomes +212** (42 lines, 25%, the most the plan owner's answer of 2026-11-04 lets an answer call raise).
Measured: +219 in `d1850d74ed`. Where it goes: the shipped server +407 against -365 in `J/server.rs`, so +42, nearly all
what the spec names; `Pages` +69, the fixture +37, the two tests +56, `J/clients.rs` +8, the manifests, the lock, the
boundary list and `limits.rs` +7. +170 can't be met: the rest of that is temper's form, `Pages` and the fixture as steps
2, 5 and 7 fix them, and the spec's estimate left out the last two. The test above takes about 13 lines, which reaches
the cap; take any other line that goes without losing behaviour, and don't touch `T`'s form to do it.

Measure, excluding what merges of `jig-workers-host` bring: `git diff a5bd40fd66 HEAD --shortstat -- .
':!jig/crates/jig-protocol-engine-workers/src/host.rs'`. It reads +219 today. Past +212, ask.

Keep `d1850d74ed` and add commits; merge `origin/debt-7/jig-workers-host` again once it has moved. Then finish what the
stop left: steps 9 and 10 whole, and both tests' breaks.
