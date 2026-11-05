# D001: The service and its machines pace work by room checks and `MAX_OUT`

- **Kind:** complexity
- **Where:** `hearth-service-builder/src/lib.rs`, `hearth-protocol-builder/src/lib.rs`, `hearth-protocol-git`, `hearth-protocol-mmdebstrap`, `hearth-protocol-config/src/json.rs`, `hearth-client-protocol-records/src/printer.rs`, `hearth-domain-builder/src/lib.rs`, `hearth-domain-render/src/lib.rs`, `hearth-domain-store/src/lib.rs`, `hearth-domain-tree/src/lib.rs`, `hearth-domain-build/src/lib.rs`, `tests/builder/src/fixtures/bounds.rs`
- **Seen:** 2026-11-04 at `66968a30ef`, in the request of plan hearth-workspace (its port table, and its rule that adopting the newer model in full is out of scope)
- **Seen:** 2026-11-04 at `66968a30ef`, in `hearth/scripts/old-forms.txt` (`max_out`)
- **Checked:** 2026-11-04 at `66968a30ef`
- **Value:** high (hearth paces as the other trees do; `iterate` shrinks; new code stops copying the old form)
- **Size:** L
- **Status:** open
- **Paid when:** no hearth step crate defines a `MaxOut` or a `MAX_OUT` constant, `iterate` asks no queue for room but the kernel's edge, the queues between a service's own layers are uncapped, and the orders the room checks kept are named predicates with world tests.
- **Measure:** `grep '^max_out ' hearth/scripts/old-forms.txt`: `max_out 85`

## What

The port took the renames and stopped there, as its request said.
Under `skein/docs/foundation/programming-model.md`, 6.3 and 7.1, nothing
caps the entries between a root's own children; only the edges refuse.
hearth still holds the old form:

- every step crate of the builder declares what one entry point may emit:
  `MAX_OUT` in five domains, `MaxOut` and `MAX_OUT_*` in
  `hearth-protocol-builder`, and `MaxOut`, `UP_MAX_OUT` and `DOWN_MAX_OUT`
  in the git, mmdebstrap, config and records machines;
- `hearth-service-builder/src/lib.rs` keeps eight capped queues, and
  `told` is the only uncapped one. Before each entry point it asks
  `io_room` or `protocol_room`; after it, `within` asserts the constant.
  23 lines of that file hold the form;
- `Service::new` refuses `limits.queue` under 3, "queues holding every
  MAX_OUT", and `tests/builder/src/fixtures/bounds.rs` sizes a world on
  that figure;
- 26 `.room()` calls remain in non-test step code.

Eight of the 85 are `io::MAX_OUT_*`, the kernel's edge, which skein keeps
and the other trees exempt (D011).

## Why it matters

Each new machine must declare its constants and each pass must loop on
them. Reading `iterate` means following the room checks, not the flow.
skein, smith, jig and temper count zero of this form, so hearth reads
unlike them, and an order a room check keeps by accident is written down
nowhere (`skein/docs/foundation/programming-model.md`, 7.2).

## What pays it back

First list what each room check keeps, a capacity or an order, as 7.2
asks, and write each order's test. Then make the inner queues uncapped,
from the machines up, and take the constants and `within` out. Do it with
D004, which rewrites the same `io_up` and `io_down`.
