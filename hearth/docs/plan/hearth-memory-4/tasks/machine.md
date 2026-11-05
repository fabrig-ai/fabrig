+++
id = "machine"
part = "machine"
kind = "migration"
needs = ["config-orders"]
files = ["hearth/crates/hearth-protocol-config/**", "hearth/tests/config/**", "hearth/scripts/old-forms.txt"]
effort = "high"
minutes = 30
+++

# The config machine, uncapped

## Goal

`hearth-protocol-config` is the third machine off the old form. Its `MaxOut`, `UP_MAX_OUT` and `DOWN_MAX_OUT` go, the queues beside its decoder are uncapped, its lists grown by hand go to
`List::uncapped`, and only the edges refuse. What hearth does doesn't change.

Acceptance:

- `git grep -nE 'MaxOut|MAX_OUT' -- hearth/crates/hearth-protocol-config hearth/tests/config` prints nothing, and `git grep -n '\.room()' -- hearth/crates/hearth-protocol-config` prints nothing.
- `git grep -n 'Queue::with_capacity' -- hearth/crates/hearth-protocol-config hearth/tests/config hearth/crates/hearth-protocol-builder/src/read.rs hearth/tests/builder/src/domain/decode.rs hearth/tests/render/src/declarations.rs`
  prints nothing.
- `hearth/tests/config/tests/suite/orders.rs` passes unchanged.
- `hearth/scripts/old-forms.txt`'s `max_out` falls by 4, from 57 to 53, with `--write`: three names in `json.rs` and the world's message.
- Every test of `hearth-protocol-config`, `hearth-config-world`, `hearth-protocol-builder`, `hearth-render-world` and `hearth-builder-world`'s focused suite passes, and `hearth-conformance-world`,
  `hearth-service-builder` and `hearth` build their tests.

## Inputs

- `hearth/docs/plan/hearth-memory-4/notes/common.md`
- `skein/docs/foundation/programming-model.md`, 6.1, 6.3, 7.1 and 7.2
- `hearth/docs/design/domain/render.md`, 8.4 and 8.5
- `skein/crates/skein-lib/src/queue.rs`, `list.rs` and `growth.rs`: `uncapped`, whose first push takes four items' room
- `skein/docs/foundation/testing-strategy.md`, 6, for a ceiling

## Owns and may change

- **Owns:** the files under `files`.
- **`hearth/crates/hearth-protocol-builder/src/read.rs`:** `Decode::new`'s two queues, and nothing else. The `if let`s in `Decode::resume` stay: an entry point gives one of each, and `suite/orders.rs`
  holds that. `client.rs`'s queues stay: those are the records machine's, a later round's.
- **`hearth/tests/builder/src/domain/decode.rs` and `hearth/tests/render/src/declarations.rs`:** the lines that make the decoder's two queues, and nothing else.
- **`hearth/tests/render/src/golden/repository.rs` and `JSON_QUEUE` don't change:** another draft (`debt-21`) moves them into a test kit. A capped queue beside the decoder still holds what it gives.
- **Nothing else reads the constants.** A survey of the trunk found no reader of `json::UP_MAX_OUT`, `json::DOWN_MAX_OUT` or `json::MaxOut` outside `json.rs`, so `hearth-service-builder` should need
  no change. If it does, make the smallest change that keeps it compiling and behaving as it does, and name in the commit and the report what the service round must finish.
- **The bites stand.** `suite/orders.rs`'s queues were uncapped from the start, and `config-orders` broke each rule, so this task doesn't break them again.
- **Ceilings, pre-approved:** the uncapped queues' and lists' first push takes more. If `each_step_holds_no_more_than_its_ceiling_under_tiny_limits`,
  `declarations_hold_no_more_than_their_ceiling_at_the_limits` or a ceiling in `hearth/tests/builder/tests/suite/memory.rs` is passed, measure the peak, set the ceiling a quarter above it rounded up to
  two significant digits, and give the old peak, the new one and the queues or lists that moved it in the commit. A ceiling that isn't passed stays. Any other test that fails is a finding first: read
  it before changing anything.
- **If the plan's order proves wrong,** so that the service must move first, stop and ask: a `Question:`, with what forced it.

## Steps

1. **`hearth/crates/hearth-protocol-config/src/json.rs`.**
   - Remove `MaxOut`, `UP_MAX_OUT` and `DOWN_MAX_OUT`.
   - `Decoder::new`'s `events` to `Queue::uncapped()`.
   - `Frame::Array`'s `values` to `List::uncapped()`, and in `place`, the `values.room() == 0` block goes: `values.push(value).expect("an uncapped list")`. `use core::mem` goes with it.
   - `decoder_bound` and `decoded_bound` price `events` at what it holds once grown. The formula bounds a retained heap, and `Queue::price(1)` no longer does:

     ```rust
     /// What `events` holds once it has grown: an uncapped queue's first push takes four items' room, and `drive` takes each event before it asks the next.
     const EVENTS: u32 = 4;
     ```

     `decoder_bound`'s `storage` stays, and its comment: an open array's four slots cost at least three bytes of document (`[0,`), 4/3 of a record a byte under the 3/2 it counts.

2. **`hearth/crates/hearth-protocol-config/src/document.rs`,** `refuse_list_null`: `pending` to `List::uncapped()`, the `pending.room() == 0` block goes, and each push takes
   `.expect("an uncapped list")`.

3. **The crate's tests.** `src/tests/fixtures.rs`: both queues in `decode_under` to `Queue::uncapped()`. `src/tests/json.rs`, `the_json_bounds_cover_flat_values_and_refuse_unrepresentable_limits`, and
   `src/tests/declaration.rs`, `the_declaration_bound_covers_flat_conversion_and_refuses_unrepresentable_limits`: each `event` in `cases` to `4 * event`. No other value moves.

4. **The world.** `hearth/tests/config/src/world.rs`: `run`'s two queues to `Queue::uncapped()`, and its `"MAX_OUT"` assertion stays, its message now `"one thing an entry point"`.
   `hearth/tests/config/tests/suite/memory/measured.rs`: `measure`'s two queues to `Queue::uncapped()`.

5. **The queues beside the machine,** each to `Queue::uncapped()`: `above` and `below` in `Decode::new`, in `read.rs`; the two queues in `hearth/tests/builder/src/domain/decode.rs`'s `decode`; the two
   in `hearth/tests/render/src/declarations.rs`.

6. **Run the readers' tests:** this changes what a full queue does, so run the packages of the acceptance, not only the crate. `cargo fmt`; clippy on `hearth-protocol-config`,
   `hearth-config-world`, `hearth-protocol-builder`, `hearth-builder-world` and `hearth-render-world`.

7. `notes/common.md`, "The end of every task", and `heavy cargo nextest run -p fabrig-boundary --test outward`. Commit, with any ceiling's figures and anything the service round must finish, and push.

## Report

The first line; the count before and after; each ceiling moved, with its peaks and why, or that none moved; the tests run and how many; anything the service round must finish.
