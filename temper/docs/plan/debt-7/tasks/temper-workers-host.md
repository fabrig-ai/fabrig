+++
id = "temper-workers-host"
part = "service-worker"
kind = "migration"
needs = ["jig-workers-host", "temper-worker-agents"]
files = ["temper/crates/temper-service/src/workers.rs", "temper/crates/temper-service/src/system.rs", "temper/crates/temper-service/src/engine.rs", "temper/crates/temper-service/src/engine/diagnostics.rs", "temper/crates/temper-service/src/stop.rs", "temper/crates/temper-service/src/worker.rs", "temper/crates/temper-service/src/tests/workers.rs", "temper/crates/temper-service/src/tests/setup.rs", "temper/crates/temper-service/src/tests/setup_bound_webhook.rs", "temper/crates/temper-service/src/tests/fixtures/forge.rs", "temper/crates/temper-service/src/tests/system.rs", "temper/docs/design/protocol/README.status.md"]
effort = "high"
minutes = 50
+++

# temper: compose the workers' side, and delete its copy

## Goal

`jig-workers-host` shipped `jig_protocol_engine_workers::host::Workers`. Delete temper's copy and compose it. temper's
`Workers` stays the adapter: `host`, with `encoding`, `forges` and `bad_parts`, which temper reads across the service, and
its assignment's encoding, its parts' decoding, its limits and what a lost listener means. `T` is the path
`notes/service-parts.md` gives.

Acceptance:

- `T/workers.rs` holds no `drain`, `listeners_open`, `Closing`, `listener_closed`, `late_listener` or `closed`, and
  `Workers` has the four fields below.
- Every temper test passes, changed only as step 7 says. An unasked listener close still stops the engine with
  `Stop::Listener`, and a closing one doesn't.
- `Workers::new`, `configured`, `limits`, `valid`, `parts`, `received` and `listener_address`'s callers behave as today.
- **Size cap:** the branch's net lines are at most -200, and `T/workers.rs` loses about 250. Measure it, and stop and ask
  if it's passed.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `temper/docs/plan/debt-7/notes/service-parts.md`, `AGENTS.md`,
  `temper/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11 and 11.3.
- `jig/crates/jig-protocol-engine-workers/src/host.rs`, on the trunk, and `jig/testing/jig-test-service/src/workers.rs`:
  the host and an adapter of the same form.

## Owns and may change

- Owns: `T/workers.rs`.
- Readers, path edits only. `engine.workers.forges` and `engine.workers.encoding` stay on the adapter: they're read
  across temper, and no site of theirs changes. `Deref` is out, so a delegating method would cost more lines than the
  edit:

  | File | Lines at this plan's base | Edit |
  |---|---|---|
  | `T/system.rs` | 77 | `engine.workers.host.closed()`, `engine.workers.host.io.len()` |
  | `T/system.rs` | 472 | `engine.workers.host.opened()` |
  | `T/system.rs` | 584 | `engine.workers.host.closed()` |
  | `T/system.rs` | 631, 635 | `engine.workers.host.io` |
  | `T/system.rs` | 928 | `engine.workers.host.listener_address(owner)` |
  | `T/system.rs` | 934 | `engine.workers.host.listening.take()` |
  | `T/engine.rs` | 255, 280 | `self.workers.host.next_deadline()`, `work_pending(now)` |
  | `T/engine/diagnostics.rs` | 25, 53 | `self.workers.host.work_pending(now)`, `opened()` |
  | `T/stop.rs` | 77 | `engine.workers.host.closed()` |
  | `T/worker.rs` | 260, 331 | `event.owner()`: `workers::owner` goes, one call around it |
  | `T/tests/workers.rs` | 398, 410, 420, 432 | `workers.host.io` |
  | `T/tests/workers.rs` | 404 | `workers.host.opened()` |
  | `T/tests/workers.rs` | 415 | `workers.host.socket(connection)` |
  | `T/tests/workers.rs` | 434, 440 | `workers.host.closed()` |
  | `T/tests/setup.rs` | 94, 96 | `.workers.host.io` |
  | `T/tests/setup_bound_webhook.rs` | 21, 22 | `engine.workers.host.io` |
  | `T/tests/fixtures/forge.rs` | 108, 109 | `engine.workers.host.io` |
  | `T/tests/system.rs` | 27 | `.workers.host.opened()` |

  These read the adapter and don't change: `T/route.rs`'s `down`, `assignment` and `send`, `T/stop.rs`'s `down` and
  `close`, `T/engine.rs`'s `abort` and `iterate`, `T/system.rs:925`'s `up`, `T/system.rs:221`'s `configured`,
  `T/tests/fixtures.rs`'s `Workers::new`. Nothing outside `temper-service` reads `Workers`.
- `temper/docs/design/protocol/README.status.md`, **Departs**, and its date line: from the first "6, 7" sentence's list
  of drifted copies, remove `workers.rs`. This departs from the implementer's text for this file.
- `Cargo.toml`: no change. `jig-protocol-engine-workers` is already a dependency.
- Not yours: any jig file. If the host can't meet a need here, stop and ask: the answer is a jig task.

Options, with their defaults:

- If a site not in the table reads a moved item, make the same path edit and name it in the report.
- `received` stays `pub`: `T/tests/workers.rs` reads it.

## Steps

What this branch changes from temper's copy, each settled by `jig-workers-host`:

| Point | What holds |
|---|---|
| A listener closing | `Seen::ListenerClosed { ending }` answers `ending`: temper's `self.ending.is_some()`, unasked a stop, as today. |
| Hand-off upward | `Output`, pulled in the same pass, so the root sees the same order. `Up` goes through `received`, as today. |
| `bad_parts`, `forges`, `encoding` | Stay on the adapter. |
| Assignment | `encoded_assignment` and `crate::assignment::down` don't change. |
| Payload version | `PAYLOAD_VERSION`, at `new`. |
| Bound address | The host's `listening` and `listener_address`, read as before. |
| Tokens | `tokens::prefix` of the two constants. `the_workers_listener_and_accepted_socket_have_their_own_prefixes` pins that they don't change. |
| `limits()`, `valid`, `configured` | Stay. `configured`'s `invalid` is `kit::host::invalid`. |

1. The struct and its methods:

   ```rust
   #[derive(Debug)]
   pub struct Workers {
       pub(crate) host: kit::host::Workers,
       pub(crate) encoding: Encoding,
       pub(crate) forges: crate::forges::Forges,
       pub(crate) bad_parts: u64,
   }
   impl Workers {
       // `new`'s signature doesn't change.
       pub fn new(..) -> Result<Self, Problems> {
           let tokens = kit::host::Tokens {
               listeners: crate::tokens::prefix(Token::new(crate::tokens::WORKER_LISTENERS)),
               connections: crate::tokens::prefix(Token::new(crate::tokens::WORKER_CONNECTIONS)),
           };
           let host =
               kit::host::Workers::new(config, files, tokens, temper_protocol_worker_wire::PAYLOAD_VERSION, limits)?;
           let mut table = crate::forges::Forges::with_capacity(forge_limit);
           for forge in forges {
               assert!(table.insert(forge), "unique fixture forges");
           }
           Ok(Self { host, encoding, forges: table, bad_parts: 0 })
       }
       pub fn down(&mut self, now: Time, event: kit::Event, events: &mut Queue<root::Event>) {
           self.host.down(now, event);
           self.pull(events);
       }
       pub fn up(&mut self, now: Time, event: io::Event, events: &mut Queue<root::Event>) -> bool {
           let seen = self.host.up(now, event);
           self.pull(events);
           match seen {
               kit::host::Seen::Taken => true,
               kit::host::Seen::Unknown => false,
               kit::host::Seen::ListenerClosed { ending } => ending,
           }
       }
       // `iterate`, `abort` and `close` as `down`, over the host's method of the same name.
       fn pull(&mut self, events: &mut Queue<root::Event>) {
           for _ in 0..self.host.pending() {
               if let Some(output) = self.host.output() {
                   self.route(output, events);
               }
           }
       }
       fn route(&mut self, output: kit::host::Output, events: &mut Queue<root::Event>) {
           match output {
               kit::host::Output::Hello { channel, worker, hello } => events.push(root::Event::Worker(
                   root::WorkerInput::Link { channel, link: root::Link::Hello { worker, hello } },
               )),
               kit::host::Output::Up { channel, up, parts } => self.received(channel, up, &parts, events),
               kit::host::Output::Lost { channel } => {
                   events.push(root::Event::Worker(root::WorkerInput::Link { channel, link: root::Link::Lost }));
               }
           }
       }
   }

   fn delivery(now: Time, channel: Token, down: host::LinkDown, values: &Values) -> kit::Event {
       let values = values.grants(kit::host::grants(&down), now);
       kit::Event::Down { channel, down, parts: Box::default(), values }
   }
   ```

2. `assignment`, `encoded_assignment` and `send` keep their bodies. `received`, `parts`, `bytes`, `left`, `saved`,
   `preparation_part`, `limits` and `valid` don't change. `configured` reads `kit::host::invalid` for `invalid`.
3. Delete `listeners_open`, `owner`, `Closing`, `invalid`, `listener_address`, `listener_closed`, `late_listener`,
   `closed`, `drain`, `next_deadline`, `opened`, `work_pending`, and the `LISTENERS` and `CONNECTIONS` constants.
4. The readers in the table. `T/worker.rs`'s two sites read `event.owner()`.
5. The status file.
6. Don't change `T/tests/fixtures.rs`: `workers()` calls `new`.
7. Tests. No test moves: `T/tests/workers.rs`' 26 are temper's composition tests, and moving them to give jig coverage
   would raise jig's focused budget (`notes/service-parts.md`). They stay, with their assertions as they are, and:

   - the path edits in the table;
   - `T/workers.rs`'s `a_hello_forwards_a_nonzero_configured_worker_position`: the `out.push(kit::Request::Hello { .. })`
     and `drain(&mut events)` lines become one, `.workers.route(kit::host::Output::Hello { channel, worker: 1, hello },
     &mut events)`. `out` is the host's. Its assertions stay.

   `T/tests/engine.rs`' `an_io_listener_failure_stops_the_engine` pins the stop on an unasked close, and
   `T/tests/workers.rs`' `a_listener_that_opens_during_closing_is_closed_immediately` the asked one. Neither changes but
   for the path edits; read them first. None of the tests `notes/service-parts.md` lists as cited is touched.
8. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`, with `--write` only if a count fell. With
   `TEMPER_BUNDLE=none`: `heavy cargo clippy -p temper-service --all-targets -- -D warnings`; then `heavy cargo nextest
   run -p temper-service -p temper -p temper-github-app -p temper-playground -p temper-fake -p
   temper-engine-protocol-forge-world -p temper-engine-domain-world -p temper-playground-world -p temper-system-world -p
   temper-real-loop-world -p temper-client-world -p temper-one-process-world -p temper-client-browser-world`. Check each
   count; read a failure's log before anything reruns. The citations check, then commit and push, one commit per coherent
   step.

## Report

The common note's items. D001's **Measure**, with the command as the item gives it, per file, against 1,388. No test is
added or moved, so none needs a break. Net lines against the cap.
