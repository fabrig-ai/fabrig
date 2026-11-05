+++
id = "machine"
part = "machine"
kind = "migration"
needs = ["git-orders", "service-orders"]
files = ["hearth/crates/hearth-protocol-git/**", "hearth/tests/git/**", "hearth/scripts/old-forms.txt"]
effort = "high"
minutes = 45
+++

# The git machine, uncapped

## Goal

`hearth-protocol-git` is the first machine off the old form. Its `MaxOut`, `UP_MAX_OUT` and `DOWN_MAX_OUT` go, the queues beside its machines are
uncapped, and its `.room()` calls go. Only the edges refuse: the machine's own limits stay as they are. What hearth does doesn't change.

Acceptance:

- `git grep -nE 'MaxOut|MAX_OUT|\.room\(\)' -- hearth/crates/hearth-protocol-git hearth/tests/git` prints nothing.
- `git grep -n 'with_capacity(1)' -- hearth/tests/git hearth/crates/hearth-protocol-git` prints nothing, and neither do the lines `## Steps`, 3, names
  in `hearth-protocol-builder`.
- The tests of the orders, `hearth/docs/design/domain/render.md`, 8.5, pass unchanged: those in `hearth/tests/git`, and `suite/orders.rs` in
  `hearth/tests/builder`. Each of git's still bites on the uncapped queues: break one rule of your choice per machine, see its test fail on the order
  and not on a panic, and restore it.
- `hearth/scripts/old-forms.txt`'s `max_out` falls by 6, with `--write`.
- Every test of `hearth-protocol-git`, `hearth-git-world`, `hearth-protocol-builder` and `hearth-builder-world`'s focused suite passes, and
  `hearth-conformance-world`, `hearth-render-world`, `hearth-service-builder` and `hearth` build their tests.

## Inputs

- `hearth/docs/plan/hearth-memory-2/notes/common.md`
- `skein/docs/foundation/programming-model.md`, 6.1, 6.3, 7.1 and 7.2
- `hearth/docs/design/domain/render.md`, 2.2, 2.3 and 8.5
- `skein/crates/skein-lib/src/queue.rs`: `Queue::uncapped`, whose first push takes four items' room where a capacity of 1 took one
- `skein/docs/foundation/testing-strategy.md`, 6, for a ceiling

## Owns and may change

- **Owns:** the files under `files`.
- **`hearth/crates/hearth-protocol-builder/src/lib.rs` and `read.rs`:** the lines of `## Steps`, 3, and nothing else. Every other queue there stays,
  `Decode`'s and `client.rs`'s and `resolve.rs`'s among them: those are the config, records and mmdebstrap machines', and later rounds'.
- **Nothing reads the constants.** A survey of the trunk found no reader of git's `MaxOut`, `UP_MAX_OUT` or `DOWN_MAX_OUT` outside the crate, and
  `indices.rs`'s `packages::UP_MAX_OUT` is mmdebstrap's, so neither `hearth-service-builder` nor `hearth-protocol-builder` should need a change for
  them. If one does, make the smallest change that keeps it compiling and behaving as it does, and name in the report what the service round must
  finish.
- **Ceilings, pre-approved:** the uncapped queues' first push takes more. If `each_decoder_at_its_limits_holds_no_more_than_its_ceiling` or a ceiling in
  `hearth/tests/builder/tests/suite/memory.rs` is passed, measure the peak, set the ceiling a quarter above it rounded up to two significant digits,
  and give the old peak, the new one and the queues that moved it in the commit. A ceiling that isn't passed stays. Any other test that fails
  is a finding first: read it before changing anything.
- **`Tail`'s cap stays.** Its queue is an edge's, git's standard error (`skein/docs/foundation/programming-model.md`, 6.1): it stays
  `Queue::with_capacity(limits.tail_lines)`, and so does `Decoder::Tail`'s `decoder_bound`. Only the `.room()` calls go.
- **If the plan's order proves wrong,** so that the service must move first, stop and ask: a `Question:`, with what forced it.

## Steps

1. **The constants.** Remove `MaxOut` from `lib.rs`, and `UP_MAX_OUT`, `DOWN_MAX_OUT` and the `MaxOut` import from `batch.rs`, `hunks.rs` and
   `listing.rs`.

2. **`tail.rs`.** Each check becomes the limit it stands for:

   ```rust
   if self.lines.capacity() > 0 {
       if self.lines.len() == self.lines.capacity() {
           drop(self.lines.pop());
       }
       self.lines.push(line);
   }
   // ...
   if self.line.len() < self.max {
       self.line.push(byte).expect("line within its limit");
   }
   // Tail::lines
   if self.pending && lines.len() < self.lines.capacity() {
   ```

3. **The queues beside the machines,** each `Queue::with_capacity(1)` to `Queue::uncapped()`:
   - `hearth-protocol-builder/src/lib.rs`: `events` and `requests`, where `down` makes `hunks::Diff` and where `finish` makes `listing::Listing`;
   - `hearth-protocol-builder/src/read.rs`: `above` and `below` in `Read::new`, the batch's, and not those in `Decode::new`;
   - `hearth/tests/git/src/world.rs` and `hunks.rs`, and `hearth-protocol-git/src/tests/fixtures.rs` and `tests/batch.rs`: all of them.

4. **Run the readers' tests:** this changes what a full queue does, so run the packages of the acceptance, not only the crate. `cargo fmt`; clippy on
   `hearth-protocol-git`, `hearth-git-world` and `hearth-protocol-builder`.

5. `notes/common.md`, "The end of every task", and `heavy cargo nextest run -p fabrig-boundary --test outward`. Commit, with any ceiling's figures, and
   push.

## Report

The first line; the count before and after; what bit on the uncapped queues; each ceiling moved, with its peaks and why, or that none moved; the
tests run and how many; anything the service round must finish.
