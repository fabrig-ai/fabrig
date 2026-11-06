+++
id = "uncap"
part = "queues"
kind = "migration"
needs = ["stage", "store-step", "stuck-client"]
files = ["hearth/crates/hearth-service-builder/src/lib.rs", "hearth/crates/hearth-service-builder/src/limits.rs", "hearth/crates/hearth-service-builder/src/tests.rs", "hearth/crates/hearth-service-builder/src/tests/valid.rs", "hearth/crates/hearth/src/lib.rs", "hearth/tests/builder/src/simulated/client.rs", "hearth/tests/builder/tests/suite/bounds.rs", "hearth/tests/builder/tests/suite/orders.rs", "hearth/tests/builder/tests/suite/memory.rs", "hearth/scripts/old-forms.txt"]
effort = "high"
minutes = 60
+++

# The queues between the layers are uncapped, and a watermark holds render

## Goal

The heart of the round. The queues between the builder's layers are uncapped, `iterate` asks for room only at the kernel's edge, and the watermark takes over from `answers.room()`. A pass's loops take new bounds, and `limits.queue` sizes only the two queues shared with the kernel. What hearth does for a user doesn't change.

Acceptance:

- `git grep -nE 'room\(\)|capacity\(\)|MAX_OUT|MaxOut|\bwithin\b|queue_limit|protocol_room' -- hearth/crates/hearth-service-builder/src/lib.rs` prints one line: `submissions.room()`, at the end of `iterate`.
- `limits.queue` is read only for `completions` and `submissions`. `calls`, `asked` and `answers` are `Queue::uncapped()`.
- `Limits`, `Service::backlog`, `admits`, `protocol_up`, `domain_stage`, `protocol_down` and `hearth::Client::service` have the forms in `notes/contracts.md`.
- The `work_pending` guards in `protocol_up` stay, the return and the check in both loops.
- `a_client_that_stops_reading_holds_render_under_its_ceiling` passes on the watermark, and bites: with `!admits(service)` taken out of `domain_stage`, the world fails on the ceiling. Its ceiling moves only as `notes/common.md` says.
- `a_pass_stops_short_of_a_whole_render` passes and bites (below).
- `startup_refuses_a_watermark_of_nothing` passes.
- `orders::what_a_call_prints_and_stores_does_not_depend_on_its_queues` passes over the pairs below. `a_build_prints_its_layers_then_its_failure_then_its_reply`, `a_client_s_output_has_one_demand_out_whatever_the_chunk` and `fixtures::CUTS` stand as they are.
- A change of meaning, so its readers' tests run: every test of `hearth-service-builder`, `hearth-protocol-builder`, `hearth-domain-builder`, `hearth` and `hearth-builder-world`, at `default` and at `--profile fuzzy` for `hearth-builder-world`, and `hearth-conformance-world` and `hearth-test-repository` build their tests.
- `hearth/scripts/old-forms.txt`'s `max_out` falls, with `--write`.

## Inputs

- `hearth/docs/plan/hearth-memory-6/notes/common.md`
- `hearth/docs/plan/hearth-memory-6/notes/contracts.md`, whole
- `hearth/docs/plan/hearth-memory-6/inputs/request.md`, 3
- `hearth/docs/design/domain/render.md`, 8.4 and 8.5
- `skein/docs/foundation/programming-model.md`, 6.3, 7.1 and 7.2
- `hearth/docs/development/debt/D001-room-checks-and-max-out.md`, "The service" and "What the service round must finish". Every row is a check that goes here, unless `stage` took it, or it is the kernel's edge.
- `skein/crates/skein-io/src/stage.rs` and `skein/examples/echo/service/src/lib.rs`, `iterate` and `protocol_up`: how a pass is paced elsewhere

## Owns and may change

- **Owns:** the files under `files`.
- **`MAX_OUT` in the domains and in `hearth-protocol-builder` stays:** `constants` takes it, once nothing here reads it. Don't touch those crates.
- **What was decided, and why:** the backlog leaves `calls` out because an event there waits behind the domain's own next resume, so counting it could hold the watermark shut for good. Only `domain::resume` waits on the watermark. `domain::step`, `domain::fire`, `protocol::up`, `protocol::down` and `io_down` never do. That is `render.md`, 8.4.
- **`hearth::Client`:** the `service()` accessor, and no other line.
- **`hearth/tests/builder/src/simulated/client.rs`:** the count below. `completions` and `submissions` stay at `limits.queue`. If a queue of 1 overflows the wrapper's own `submissions` on its startup closes, raise that wrapper's capacity by what the wrapper itself adds, and say so.
- **`domain_stage`'s step loop** keeps taking event after event without asking `work_pending`: `render.md`, 10 keeps that question for the first build step that streams.
- **`hearth/tests/builder/src/protocol.rs`:** it reads `builder::MAX_OUT` and drives the machines itself, not through `iterate`. `constants` takes it. Leave it.
- **A deadlock or a hang is a finding.** If a world stops settling at some watermark, read why before anything changes: a `Question:`, with what you found, if the answer isn't in `render.md`, 8.4.

## Steps

1. `limits.rs`: `watermark`, 30 in `SHIPPED`. `lib.rs`: `Service`, `new`, `backlog`, `admits`, and the three stages to `notes/contracts.md`; `within`, `protocol_within`, `protocol_room` and `queue_limit` go.
2. `hearth::Client::service`.
3. `bounds.rs`: `QUEUE` and `TARGETS = QUEUE + 1` go, and nothing sizes a world on the old assertion.

   ```rust
   // A watermark this small, with one more target than it holds, forces a later
   // pass while printer output is still moving through the layers.
   pub const WATERMARK: u32 = 2;
   pub const TARGETS: u32 = 4;
   ```

   `limits()` sets `limits.watermark = WATERMARK`, and `limits.domain.render.targets = TARGETS`. `stuck_limits()` sets `limits.watermark = WATERMARK` in place of `limits.queue = 3`.
4. `orders.rs`: `QUEUES` becomes `(queue, watermark)` pairs, `[(1, 1), (3, 2), (32, 30), (4_096, 4_096)]`, each setting both, the baseline being the first. `a_store_answer_precedes_the_next_index` runs at the first pair: check that it still bites. The test's name stays: other documents cite it, and `limits.queue` still varies in it.
5. `memory.rs`: the bite of `a_client_that_stops_reading_holds_render_under_its_ceiling` is now `admits`. Take `!admits(service)` out, see the world fail on the ceiling, restore it, and give both peaks. `queued_specs_hold_no_more_than_their_ceiling` keeps its assertion.
6. **The pass.** One pass mustn't run a whole render. `simulated/client.rs` counts, after each `inner.iterate`, the passes that ended with the domain's work pending and the backlog under the watermark: only the pass's own bound can have stopped those.

   ```rust
   #[test] // orders.rs
   fn a_pass_stops_short_of_a_whole_render() {
       // A small watermark, and a call whose render steps make nothing for the queues:
       // `bounds::queued()` if its render does, else a diff's first side, which keeps
       // its specs (`hearth-domain-render/src/call.rs`, `retain_spec`).
       // The count is above zero, and every target prints.
   }
   ```

   **Bite:** make `domain_stage`'s bound `0..u32::MAX`, see the count fall to zero and the test fail, and restore it. The test's comment says what stops a pass, so that the statement in the code and in `render.md`, 8.4 can be held against it.
7. `src/tests/valid.rs`:

   ```rust
   #[test]
   #[should_panic(expected = "a watermark that admits work")]
   fn startup_refuses_a_watermark_of_nothing()
   ```

8. `python3 -I -B scripts/old_forms.py --write` from `hearth/`.
9. `heavy cargo clippy -p hearth-service-builder -p hearth -p hearth-builder-world --all-targets -- -D warnings`, then the tests above. Commit and push, one commit if it holds together. A ceiling moved names its peak and why.

## Report

Each rule broken and the failure seen; every ceiling, moved or not, with its peak and why; `fixtures::CUTS`; the peaks of the stuck client with and without `admits`; what ran, how many tests, the CPU seconds of the new tests, and how the suite's time compares if nextest's total shows it. Anything `resumed-order` should know about the guard goes here.
