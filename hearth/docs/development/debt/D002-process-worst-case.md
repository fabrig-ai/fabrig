# D002: The builder sums a process-wide worst case nothing runs, in formulas under the old name

- **Kind:** complexity
- **Where:** `hearth-service-builder/src/lib.rs`, `hearth-service-builder/src/tests/memory.rs`, `hearth-domain-builder/src/lib.rs`, `hearth-domain-render/src/limits.rs`, `hearth-protocol-builder/src/limits.rs`, and each `worst_case` of the step crates
- **Seen:** 2026-11-04 at `66968a30ef`, in the plan hearth-workspace (its decision that `service::worst_case` stays under its old name)
- **Seen:** 2026-11-04 at `66968a30ef`, in `hearth/scripts/old-forms.txt` (`worst_case`)
- **Checked:** 2026-11-04 at `66968a30ef`
- **Value:** high (removes a figure the model retired; each formula says what it bounds)
- **Size:** M
- **Status:** open
- **Paid when:** `hearth-service-builder` has no `worst_case` and no test of a process-wide sum, the sums that fed only it are gone, and each formula left is named for what it bounds, `_bound` or `_price`.
- **Measure:** `git grep -E 'fn worst_case\b' -- 'hearth/crates/*.rs' | wc -l`: 25

## What

- `hearth_service_builder::worst_case` sums the domain, the protocol
  layer and nine queues. No production code calls it: the binary stopped
  when the port landed, and only
  `hearth-service-builder/src/tests/memory.rs` does, in
  `the_shipped_limits_have_a_worst_case_within_1_gib` and one test more.
- It no longer sums what a process holds: skein removed the io and file
  terms, so the 1 GiB it is held to is not a process's figure.
- 25 functions named `worst_case` remain across the step crates, with
  `held_worst_case` and `decoded_worst_case` beside them. Some are edges
  a test reads (a decoder's retained bound); others exist only to feed
  the sum above.

`skein/docs/foundation/programming-model.md`, 6.3, ends the whole-process
figure and names every formula for what it bounds. hearth's design still
states the sum (`domain/render.md`, 8.4); `hearth/docs/design/STATUS.md`
doesn't say so yet.

## Why it matters

A green test on a figure no process has is false comfort, and every limit
change keeps moving it. 132 uses of the old name make the tree read as if
the sum still held.

## What pays it back

Delete the sum and its two tests, then each formula with no other reader.
Rename what stays, by `skein/docs/foundation/programming-model.md`, 6.3:
`decoder_bound`, `document_bound`, `valid`, and so on. Formulas that a
memory test reads go with D003. The design's sentence changes first, or a
status line records the departure.
