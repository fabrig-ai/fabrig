+++
id = "jig-workers-host"
part = "service-worker"
kind = "migration"
needs = ["jig-worker-agents"]
files = ["jig/crates/jig-protocol-engine-workers/src/host.rs", "jig/crates/jig-protocol-engine-workers/src/lib.rs", "jig/crates/jig-protocol-engine-workers/Cargo.toml", "jig/testing/jig-test-service/src/workers.rs", "jig/testing/jig-test-service/src/stop.rs", "jig/testing/jig-test-service/src/engine.rs", "jig/testing/jig-test-service/src/lib.rs", "jig/testing/jig-test-service/src/tests/stop.rs", "jig/testing/jig-test-service/src/tests/engine.rs", "jig/tests/boundary/src/tests/graph.rs", "jig/docs/design/protocol/service.status.md"]
effort = "high"
minutes = 60
+++

# jig: the workers' side as product code

## Goal

Move `J/workers.rs`, and what of `J/stop.rs` belongs to it, to `jig_protocol_engine_workers::host`, in one branch. The
root's events become a pull, and `J` keeps a thin adapter: its assignment's encoding, its grants' values, its root's
events. `temper-workers-host` deletes temper's copy. `T` and `J` are the paths `notes/service-parts.md` gives.

Acceptance:

- `jig_protocol_engine_workers::host` exists with the surface below. It names no root type, and no `Values`.
- `J/workers.rs` holds the adapter below and nothing else. `J/stop.rs` holds no `Closing` and no `impl Workers`.
- `jig-test-service`'s tests and the worlds in step 7 pass, changed only as step 6 says.
- `jig/docs/design/protocol/service.status.md` no longer lists the workers' side.
- **Size cap:** the branch's net lines are at most +120, tests and manifests included. It takes temper's address and
  closing form, and `J/workers.rs` keeps about 120 lines; temper's task then falls by about 250. Measure it, and stop and
  ask if it's passed.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `temper/docs/plan/debt-7/notes/service-parts.md`, `AGENTS.md`,
  `jig/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11 and 11.3, on the trunk: the contract. 11.1 is the pull's pattern.
- `jig/docs/design/protocol/channel.md`, 10.2; `jig/docs/design/protocol/service.md`, 5.
- `J/workers.rs`, `J/stop.rs`. In `T/workers.rs`, read-only: `listening`, `listener_address`, the listeners' address.
- `jig_protocol_config::Files`, on the trunk from `jig-accounts-host`, with `contents` public.

## Owns and may change

- Owns: the new `jig/crates/jig-protocol-engine-workers/src/host.rs` and its `pub mod host;` in `lib.rs`; `J/workers.rs`;
  in `J/stop.rs`, `Closing` and `impl crate::workers::Workers`, which go.
- `jig/crates/jig-protocol-engine-workers/Cargo.toml`: `jig-protocol-config.workspace = true`. It links nothing back: of
  `jig-protocol-config`'s closure, none depends on this crate. `Cargo.lock` follows.
- `jig/tests/boundary/src/tests/graph.rs`: `engine_worker_channels_link_only_their_boundary_and_foundation` is an exact
  list. It gains `jig-protocol-config`, in sorted place, and is renamed
  `engine_worker_channels_link_only_their_boundary_configuration_and_foundation`: no document cites it. Nothing else in
  that file changes.
- Readers, path edits only. The adapter's field is `host`, and `Deref` is out, so a delegating method would cost more
  lines than the edit:

  | File | Lines at this plan's base | Edit |
  |---|---|---|
  | `J/engine.rs` | 129, 160, 175, 253 | `workers.host.next_deadline()`, `work_pending(now)`, `opened()` |
  | `J/engine.rs` | 281, 282 | `workers.host.io` |
  | `J/lib.rs` | 374, 378 | `workers.host.io` |
  | `J/lib.rs` | 429, 491 | `workers.host.opened()` |
  | `J/stop.rs` | 72 | `workers.host.closed()` |
  | `J/tests/engine.rs` | 528, 530 | `.host.opened()` |
  | `J/tests/stop.rs` | 33, 45 | `.host.ending()` |
  | `J/tests/stop.rs` | 48 | `.host.ending()`, `Some(jig_protocol_engine_workers::host::Closing::Drain)` |
  | `J/tests/stop.rs` | 61, 73 | `.host.closed()` |
  | `J/tests/stop.rs` | 111 | `.host.component.dropped_down()` |

  These sites read the adapter and don't change: `J/route.rs`'s seven, `J/stop.rs`'s `down` and `close`, `J/engine.rs`'s
  `abort` and `iterate`, `J/lib.rs`'s `up`, `J/startup.rs`'s `new`, `J/tests/clients.rs`, `J/tests/fixtures.rs`,
  `J/tests/engine.rs`'s `encoded_assignment`. Nothing outside `jig-test-service` reads `Workers`.
- `jig/docs/design/protocol/service.status.md`: from **Not yet**, 11, remove "the workers' side", and update the date
  line.
- Not yours: any temper file. `kit::Component`, `Limits`, `Event` and `Request` don't change.

Options, with their defaults:

- If a test not in the table reads a moved field, make the same path edit and name it in the report. A field no read
  above needs stays private.
- `J/tests/clients.rs`'s `Files` is `jig_test_service::Files`, the re-export of `jig_protocol_config::Files`: `new` takes
  it as it stands.

## Steps

The rules the two copies disagree on, which this branch settles:

| Point | What holds |
|---|---|
| A listener closing | A report: `up` returns `Seen::ListenerClosed { ending }`. jig ignores it, as today. |
| Hand-off upward | `Output`, pulled. jig drops `parts`, as today. |
| `bad_parts` | temper's; not in the host. |
| Assignment | The owner's. It hands `down` the finished `Event::Down`. |
| Payload version | A value at `new`: jig's stays 1. |
| Bound address | temper's: `listening`, and the address in the listeners' list. jig never reads either. |
| Tokens | A value: `Tokens`. `Token::tagged(tag, value)` gives both services' tokens: temper's `(1 << 60) + index` is `tagged(1, index)`. |
| `limits()`, `valid`, `configured` | Each service's. |

1. `src/host.rs`, module doc citing `jig/docs/design/protocol/channel.md, 10.2` and
   `jig/docs/design/protocol/service.md, 11.3`. `hub` is `jig_domain_host`, and `kit`'s types are the crate's own:

   ```rust
   /// The owner's namespaces for this part's io (skein/docs/design/lib.md, 4).
   #[derive(Clone, Copy, PartialEq, Eq, Debug)]
   pub struct Tokens {
       pub listeners: u8,
       pub connections: u8,
   }

   #[derive(Clone, Copy, PartialEq, Eq, Debug)]
   pub enum Closing {
       Drain,
       Abort,
   }

   /// What the channel hands its root, in the component's order; the owner routes each before the next.
   #[derive(Debug)]
   pub enum Output {
       Hello { channel: Token, worker: u32, hello: hub::Hello },
       Up { channel: Token, up: hub::LinkUp, parts: Box<[Part]> },
       Lost { channel: Token },
   }

   /// What became of an event `up` was given.
   #[must_use]
   #[derive(Clone, Copy, PartialEq, Eq, Debug)]
   pub enum Seen {
       Taken,
       /// An event this part has no use for.
       Unknown,
       /// A listener closed: `ending` says whether the part was already closing.
       ListenerClosed { ending: bool },
   }

   #[derive(Debug)]
   pub struct Workers {
       /// Bound address awaiting the owner's report.
       pub listening: Option<core::net::SocketAddr>,
       pub component: Component,
       pub io: Queue<io::Request>,
       out: Queue<Request>,
       listeners: List<(Token, io::Addr, Option<Token>)>,
       sockets: Map<Token, Token>,
       tokens: Tokens,
       next: u64,
       opened: bool,
       ending: Option<Closing>,
       component_closed: bool,
   }

   impl Workers {
       pub fn new(
           config: &config::Engine,
           files: &config::Files,
           tokens: Tokens,
           payload: u32,
           limits: &Limits,
       ) -> Result<Self, Problems>;
       #[must_use]
       pub fn next_deadline(&self) -> Option<Time>;
       #[must_use]
       pub fn opened(&self) -> bool;
       #[must_use]
       pub fn ending(&self) -> Option<Closing>;
       #[must_use]
       pub fn socket(&self, connection: Token) -> Option<&Token>;
       #[must_use]
       pub fn work_pending(&self, now: Time) -> bool;
       /// Every listener, accepted socket and queued request has closed.
       #[must_use]
       pub fn closed(&self) -> bool;
       #[must_use]
       pub fn listener_address(&self, owner: Option<Token>) -> Option<io::Addr>;
       pub fn down(&mut self, now: Time, event: Event);
       pub fn up(&mut self, now: Time, event: io::Event) -> Seen;
       pub fn iterate(&mut self, now: Time);
       pub fn abort(&mut self, now: Time);
       /// Close listeners and drain the channel component.
       pub fn close(&mut self, now: Time);
       /// The component's requests waiting for `output`.
       #[must_use]
       pub fn pending(&self) -> u32;
       /// Takes one request, doing itself what needs no owner.
       pub fn output(&mut self) -> Option<Output>;
   }

   /// The grants a down carries, whose values the owner holds, in order.
   #[must_use]
   pub fn grants(down: &hub::LinkDown) -> &[hub::Grant];

   /// A problem at `engine.hosts`.
   #[must_use]
   pub fn invalid(what: What) -> Problems;
   ```

   - `new` is `J/workers.rs`'s body, in its order: digests, the component, the count, the listeners. A listener's owner is
     `Token::tagged(tokens.listeners, index)`, with `.expect("service token fits its namespace")` as `crate::tokens::token`
     has it; the list keeps `*address`. The endpoints stay in the adapter.
   - `up` is `J/workers.rs`'s, with `self.listening = addr.socket()` first in `Listening`, each `true` a `Seen::Taken`,
     each `false` a `Seen::Unknown`, and in `Closed`, `if self.listener_closed(owner) { return Seen::ListenerClosed
     { ending: self.ending.is_some() }; }`. A connection's token is `Token::tagged(tokens.connections, self.next)`.
   - `down` is `self.component.down(now, event, &mut self.out)`, and nothing else; `iterate` is the same with `fire`.
   - `output` is one `drain` arm: `self.out.pop()?`, then the same arms, with `Hello`, `Up` and `Lost` returning `Some`
     and the others `None`.
   - `abort` and `close` are today's, with `self.down(now, ..)`. `listener_closed`, `late_listener` and `listeners_open`
     stay private.
   - `grants` is `delivery`'s match, returning the slice, and `&[]` for the other arms.

2. `J/workers.rs`:

   ```rust
   #[derive(Debug)]
   pub(super) struct Workers {
       pub(super) host: kit::host::Workers,
       deployment: [u8; 16],
       endpoints: List<jig_protocol_charter::EndpointName>,
   }
   impl Workers {
       pub fn new(config: &config::Engine, files: &Files, deployment: [u8; 16], limits: &kit::Limits)
           -> Result<Self, Problems> {
           let tokens = kit::host::Tokens {
               listeners: crate::tokens::WORKER_LISTENERS,
               connections: crate::tokens::WORKER_CONNECTIONS,
           };
           let host = kit::host::Workers::new(config, files, tokens, 1, limits)?;
           // The endpoints, as today.
           Ok(Self { host, deployment, endpoints })
       }
       pub fn down(&mut self, now: Time, event: kit::Event, events: &mut Queue<root::Event>) {
           self.host.down(now, event);
           self.pull(events);
       }
       pub fn up(&mut self, now: Time, event: io::Event, events: &mut Queue<root::Event>) -> bool {
           let seen = self.host.up(now, event);
           self.pull(events);
           match seen {
               kit::host::Seen::Taken | kit::host::Seen::ListenerClosed { .. } => true,
               kit::host::Seen::Unknown => false,
           }
       }
       // `iterate`, `abort` and `close` as `down`, over the host's method of the same name.
       // `assignment`, `encoded_assignment` and `send`: their bodies don't change.
       fn pull(&mut self, events: &mut Queue<root::Event>) {
           for _ in 0..self.host.pending() {
               match self.host.output() {
                   Some(kit::host::Output::Hello { channel, worker, hello }) => {
                       events.push(root::Event::Link { channel, link: root::link::Event::Hello { worker, hello } });
                   }
                   Some(kit::host::Output::Up { channel, up, parts }) => {
                       drop(parts);
                       events.push(root::Event::Link { channel, link: root::link::Event::Up(up) });
                   }
                   Some(kit::host::Output::Lost { channel }) => {
                       events.push(root::Event::Link { channel, link: root::link::Event::Lost });
                   }
                   None => {}
               }
           }
       }
   }

   fn delivery(now: Time, channel: Token, down: host::LinkDown, values: &Values) -> kit::Event {
       let values = values.grants(kit::host::grants(&down), now);
       kit::Event::Down { channel, down, parts: Box::default(), values }
   }
   ```

   `next_deadline`, `opened`, `work_pending` and `closed` leave the adapter: their readers take `host`, as the table says.
3. `J/stop.rs`: delete `Closing` and `impl crate::workers::Workers`.
4. The manifest, `lib.rs` and `graph.rs`.
5. The readers in the table.
6. Tests. `T/tests/workers.rs` and `T/workers.rs`'s inline tests are temper's composition tests: none moves, and the
   host gets no test crate of its own, since moving them would raise jig's focused budget
   (`notes/service-parts.md`). Every jig test stays where it is, with the path edits in the table. One is added, in
   `J/tests/stop.rs`, since no jig test reaches `ListenerClosed` and this branch writes the arm that ignores it:

   ```rust
   #[test]
   fn a_lost_listener_changes_nothing_for_the_reference() {
       let mut service = components();
       let engine = fixtures::engine(&mut service);
       let owner = crate::tokens::token(crate::tokens::WORKER_LISTENERS, 0);
       let workers = engine.workers.as_mut().expect("workers");
       assert!(workers.up(skein_lib::Time::ZERO, skein_io::Event::Closed { owner }, &mut engine.events));
       assert_eq!(workers.host.ending(), None);
       assert_eq!(service.end(), None);
   }
   ```

   Adjust the borrows as the compiler asks; the three assertions stand. `service.md`, 5's
   `tests::engine::the_store_root_closes_after_delayed_driver_completions` keeps its name and module.
7. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-engine-workers -p
   jig-test-service -p jig-boundary --all-targets -- -D warnings`; then `heavy cargo nextest run -p
   jig-protocol-engine-workers -p jig-test-service -p jig-test -p jig-boundary -p jig-engine-workers-world -p
   jig-worker-link-world -p jig-channel-world -p jig-one-process-world -p jig-engine-clients-world -p
   jig-client-protocol-world`. This branch changes what no temper code reads yet, so temper needs no check. Check each
   count; read a failure's log before anything reruns. The citations check, then commit and push, one commit per coherent
   step.

## Report

The common note's items. Two breaks: make the adapter's `up` return `false` on `ListenerClosed`, and name what
`a_lost_listener_changes_nothing_for_the_reference` reads; add a stray dependency to the crate's manifest, and name what
the renamed boundary test reads. Net lines against the cap.
