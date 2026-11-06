+++
id = "machine"
part = "machine"
kind = "migration"
needs = ["printer-orders"]
files = ["hearth/crates/hearth-client-protocol-records/src/printer.rs", "hearth/crates/hearth-client-protocol-records/src/tests/printer.rs", "hearth/crates/hearth-protocol-builder/src/client.rs", "hearth/scripts/old-forms.txt"]
effort = "high"
minutes = 20
+++

# The records machine, uncapped

## Goal

`hearth-client-protocol-records` is the fourth machine off the old form. Its `MaxOut`, `UP_MAX_OUT` and `DOWN_MAX_OUT` go, the queues beside the printer are uncapped, and only the edges refuse. What hearth does doesn't change.

Acceptance:

- `git grep -nE 'MaxOut|MAX_OUT' -- hearth/crates/hearth-client-protocol-records` prints nothing.
- `git grep -n 'Queue::with_capacity' -- hearth/crates/hearth-client-protocol-records hearth/crates/hearth-protocol-builder/src/client.rs` prints nothing.
- `printer-orders`' three tests pass unchanged, and so do `orders::what_a_call_prints_and_stores_does_not_depend_on_its_queues` and `a_refused_spec_is_left_out_and_the_call_ends_failed`.
- `hearth/scripts/old-forms.txt`'s `max_out` falls by 14, with `--write`, from 53 to 39 on the trunk as cut: two names in `printer.rs` and twelve in `src/tests/printer.rs`.
- Every test of `hearth-client-protocol-records`, `hearth-protocol-builder`, `hearth-service-builder`, `hearth-render-world` and `hearth-builder-world`'s focused suite passes, and `hearth-conformance-world`, `hearth-test-render` and `hearth` build their tests.

## Inputs

- `hearth/docs/plan/hearth-memory-5/notes/common.md`
- `skein/docs/foundation/programming-model.md`, 6.3, 7.1 and 7.2
- `hearth/docs/design/domain/render.md`, 8.4 and 8.5
- `skein/crates/skein-lib/src/queue.rs` and `growth.rs`: `uncapped`, whose first push takes four items' room
- `skein/docs/foundation/testing-strategy.md`, 6, for a ceiling

## Owns and may change

- **Owns:** the files under `files`.
- **`hearth-protocol-builder/src/client.rs`:** `Client::new`'s two queues, and no other line. Another draft (`debt-25`) rewrites the lines beside them, so a wider edit costs a wider merge.
- **Nothing else reads the constants.** A survey of the trunk found no reader of `printer::UP_MAX_OUT`, `printer::DOWN_MAX_OUT` or `printer::MaxOut` outside `printer.rs` and `src/tests/printer.rs`, so `hearth-service-builder` and the rest of `hearth-protocol-builder` should need no change. If one does, make the smallest change that keeps it compiling and behaving as it does, and name in the commit and the report what the service round must finish.
- **The bites stand.** `printer-orders` broke each rule, on queues the machine tests own and within `requests`' two in the world, so this task doesn't break them again.
- **Ceilings, pre-approved:** two `Client`s take a little more heap, `events` four items' room where it took one and `requests` four where it took two. If a ceiling in `hearth/tests/builder/tests/suite/memory.rs` is passed, measure the peak, set the ceiling a quarter above it rounded up to two significant digits, and give the old peak, the new one and the queues that moved it in the commit. A ceiling that isn't passed stays. Any other test that fails is a finding first: read it before changing anything.
- **If the plan's order proves wrong,** so that the service must move first, stop and ask: a `Question:`, with what forced it.

## Steps

1. **`hearth/crates/hearth-client-protocol-records/src/printer.rs`.** Remove `MaxOut`, `UP_MAX_OUT` and `DOWN_MAX_OUT`, and nothing else: `valid`, `Limits`, `Printer::new`'s assertion and `Event::Refused` stay.

2. **`hearth/crates/hearth-protocol-builder/src/client.rs`,** `Client::new`: `events` and `requests` to `Queue::uncapped()`.

3. **`hearth/crates/hearth-client-protocol-records/src/tests/printer.rs`,** `Machine`:
   - `Machine::new`: both queues to `Queue::uncapped()`.
   - `Machine::down` keeps what the constants said: `self.above.len() <= 1`, `"one event a down"`, and `self.below.len() <= 1`, `"one request a down"`.
   - `Machine::up` too: `self.above.len() <= 1`, `"one event an up"`, and `self.below.len() <= 2`, `"a piece, then the next demand"`.

4. **Run the readers' tests:** this changes what a full queue does, so run the packages of the acceptance, not only the crate. `cargo fmt`; clippy on `hearth-client-protocol-records` and `hearth-protocol-builder`.

5. From `hearth/`, `python3 -I -B scripts/old_forms.py --write`.

6. `notes/common.md`, "The end of every task", and `heavy cargo nextest run -p fabrig-boundary --test outward`. Commit, with any ceiling's figures and anything the service round must finish, and push.

## Report

The first line; the count before and after; each ceiling moved, with its peaks and why, or that none moved; the tests run and how many; anything the service round must finish.
