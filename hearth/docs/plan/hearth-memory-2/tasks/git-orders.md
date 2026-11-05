+++
id = "git-orders"
part = "orders"
kind = "feature"
needs = []
files = ["hearth/tests/git/**"]
effort = "high"
minutes = 35
+++

# Tests of the orders git's machines keep

## Goal

`hearth-protocol-git`'s room checks and capacity-1 queues go in a later task. This one writes, in git's world, the tests that hold what they kept,
all passing on today's code. No production code changes.

Acceptance:

- The tests of `## Steps`, 1 to 4, stand in `hearth/tests/git/tests/suite/`, and pass.
- Each bites, by the rule named beside it.
- Every test of `hearth-git-world`'s `suite` passes: check the count, and that it rose by the tests you added.
- Nothing outside `hearth/tests/git/` changes.

## Inputs

- `hearth/docs/plan/hearth-memory-2/notes/common.md`
- `hearth/docs/design/domain/render.md`, 2.2's tool error, and 8.5
- `hearth/docs/design/domain/testing.md`, 2.4
- `skein/docs/foundation/programming-model.md`, 7.2

## Owns and may change

- **Owns:** the files under `files`.
- **Reader files:** none.
- **Option, pre-approved:** the sketches use `Held` where a machine must stand between deliveries, and `Source::Cut` where the cuts are the load. If
  a test reads better on the other, use it and say so.
- **Option, pre-approved:** `generate::batch` draws its blobs. If no seed gives a blob of more than one piece at `fixtures::SMALL`, build that batch by
  hand in the test.

## Steps

1. **The tail, a new `hearth/tests/git/tests/suite/tail.rs`** (`mod tail;` in `main.rs`), driving `hearth_protocol_git::tail::Tail` directly:

   ```rust
   // hearth/tests/git/src/generate.rs: encoded independently of the machine.
   pub struct Stderr { pub bytes: Vec<u8>, pub lines: Box<[Box<[u8]>]> }  // `lines`: what the tail must answer
   pub fn stderr(rng: &mut Rng, limits: &Limits) -> Stderr
   // hearth/tests/git/src/world.rs
   pub fn tail(stderr: &[u8], limits: &Limits, seed: u64) -> Box<[Box<[u8]>]>  // cuts by the seed, calling `admit` per cut
   ```

   - `the_tail_keeps_the_newest_lines_oldest_first_whatever_the_cuts`: for 32 seeds at `SMALL`, lines of distinct text, some past `tail_line`, as many
     as twice `tail_lines`. **Break:** in `Tail::admit`, drop the new line instead of popping the oldest.
   - `an_unended_last_line_is_the_newest`: rows of input and answer at `SMALL`: `one\ntwo\nthree` gives `two`, `three`; `one\nthree` gives `one`,
     `three`; `three` gives `three`; `one\ntwo\n` gives `one`, `two`. **Break:** in `Tail::lines`, put the unended line first.
   - `a_tail_of_no_lines_holds_nothing`: `Limits { tail_lines: 0, ..SMALL }`, with and without an unended line, answers no line. **Break:** in
     `Tail::lines`, push the unended line whatever the room.

2. **A piece holds the next read, in `suite/batch.rs`.** `world::read` asserts, each time `batch::up` leaves a `Piece` above, that `below` holds no
   demand. The world's drivers keep their capacity-1 queues for now: the task that uncaps the machines changes them.

   - `the_next_read_waits_until_the_piece_before_it_is_taken`: `world::read` over generated batches at cuts, asserting that some blob took more
     than one piece. **Break:** make `batch::up`'s `Body` arm demand at once, as if `Paused` were `Body`.

3. **An outcome behind a piece, in `suite/batch.rs`.**

   - `an_end_or_a_fault_behind_a_piece_answers_the_next_request`: rows of `Up::End` giving `Error::Truncated` and `Up::Failed(Fault::Reset)` giving
     `Error::Stream(Fault::Reset)`, each in the three places a batch waits: `Paused` with bytes left, `Paused` with none, and `Idle` after a `Whole`.
     The delivery puts nothing above and nothing below; the next request (`Next`, or a `Read` at `Idle`) answers `Failed`, and demands nothing.
     **Break:** in `batch::up`, `fail` at once instead of `State::Stopped`.

4. **An outcome before its request, in `suite/listing.rs` and `suite/hunks.rs`.**

   - `an_end_or_a_fault_before_the_list_answers_the_list`: on a new `Listing`, `Up::End` then `Request::List` gives `Listed` of nothing, and
     `Up::Failed(Fault::Reset)` then `Request::List` gives `Failed(Error::Stream(Fault::Reset))`. Nothing goes above before the request. **Break:**
     in `listing::up`, `finish` at `State::Idle`.
   - `an_end_or_a_fault_before_the_read_answers_the_read`: the same on a new `Diff` and `hunks::Request::Read`, where the `End` gives
     `Failed(Error::Malformed)`, git having printed nothing. **Break:** in `hunks::up`, `fail` at `State::Idle`.

5. `cargo fmt`, clippy on `hearth-git-world`, `heavy cargo nextest run -p hearth-git-world`.
6. `notes/common.md`, "The end of every task". Commit and push.

## Report

The first line; each test with what bit it; the count of `suite` before and after; the tests' CPU seconds.
