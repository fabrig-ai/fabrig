+++
id = "io-stage"
part = "kit-contract"
kind = "interface"
needs = []
files = ["skein/crates/skein-io/src/stage.rs", "skein/crates/skein-io/src/lib.rs", "skein/crates/skein-io/src/limits.rs", "skein/tests/io/tests/suite/stage.rs", "skein/tests/io/tests/suite/main.rs"]
effort = "high"
minutes = 75
+++

## Goal

io's stage, the loop's resume, up and fire passes and its down pass, is
written once in skein-io as `skein_io::stage::{up, down}`, each entry
point taken only while the kernel's submission room holds its `MAX_OUT`
(`skein/docs/design/draft/component-kit.md, 3.1`; testing-strategy.md, 6).
It is modelled on the echo example's private stage. No host moves onto it
here; later tasks do.

Acceptance:

- `skein_io::stage::up` and `skein_io::stage::down` exist with the
  signatures below, documented, in a public module `stage`.
- One ordering test in `tests/io` (`tests/suite/stage.rs`) shows what the
  two `#[path]`-included copies show today
  (`tests/echo/tests/suite/scenarios.rs:249`,
  `tests/fake-peers/tests/suite/stages.rs`): ready results are told before
  completions, completions before deadlines; and a full submission queue
  stops each pass before the entry point whose `MAX_OUT` no longer fits.
  Those two copies stay as they are here; the tasks that move echo and
  fake-peers delete them.
- skein-io's and tests/io's tests and clippy pass.

## Inputs

- `skein/docs/plan/component-kit-4/notes/common.md`.
- `skein/docs/design/draft/component-kit.md, 3.1`.
- `skein/docs/design/io.md, 2` (the loop's order); `testing-strategy.md, 6`
  (submission room at the kernel's edge).
- The model: `skein/examples/echo/service/src/io_stage.rs` (53 lines) and
  `io_down` in `skein/examples/echo/service/src/lib.rs:400-417`.
- The second copy and its test: `skein/crates/skein-fake-peers/src/face.rs`
  (`Face::up`, `Face::down`) and
  `skein/tests/fake-peers/tests/suite/stages.rs`.
- `crates/skein-io/src/lib.rs` "Driving it", `layer.rs` (`resume`, `up`,
  `fire`, `down`, `Io::is_ready`, `is_due`, `takes`), `limits.rs`.

## Owns and may change

- Owns `crates/skein-io/src/stage.rs` (new), `crates/skein-io/src/lib.rs`
  (the module, its export, a line in "Driving it" naming the stage), and
  `tests/io/tests/suite/stage.rs` (new) with its `mod` line in
  `tests/io/tests/suite/main.rs`.
- May add one `pub(crate)` or `pub const fn` to `limits.rs` for the stage's
  bound (step 2), and nothing else there.
- Nothing else: no host, no other test crate.

## Steps

1. The surface (signatures bind):

   ```rust
   //! io's stage of a service's loop (io.md, 2): what every host's loop
   //! does with io in each pass, the kernel's submission room checked
   //! before each entry point (testing-strategy.md, 6).

   /// The up pass's stage: `resume` while io is ready, then, once its ready
   /// list is empty, `up` for each completion, then, once every completion
   /// is taken, `fire` while a deadline is due at `env.now`; each only
   /// while `submissions` has room for its `MAX_OUT`. What is not taken
   /// stays for the next iteration.
   pub fn up(
       io: &mut Io,
       env: &Env<Limits>,
       completions: &mut Queue<Complete>,
       events: &mut Queue<Event>,
       submissions: &mut Queue<Submit>,
   );

   /// The down pass's stage: `down` for each request in order, while io
   /// takes one (`Io::takes`) and `submissions` has room for
   /// `MAX_OUT_DOWN`. What is not taken stays queued.
   pub fn down(
       io: &mut Io,
       env: &Env<Limits>,
       requests: &mut Queue<Request>,
       submissions: &mut Queue<Submit>,
   );
   ```

   The time is `env.now` (echo's stage takes `now` beside an `env` whose
   `now` its loop has just set; the shared one reads the env).
2. Every loop is a bounded `for` (programming-model.md, 10). Echo bounds
   `resume` by `sockets + refusals` and `fire` by `2 * sockets`, which are
   right for echo's entities; the shared stage must bound by every entity
   kind io holds (pipes, children, adopted appends and signals). Derive the
   bound from `Limits` and io's own counts, say why it holds in a comment,
   and put a helper in `limits.rs` if one makes it clearer. The `completions`
   loop is bounded by the queue's length on entry.
3. The test, in tests/io's style (its `src/` helpers as they are), with
   named constants for every size: one world that holds a ready result, a
   completion and a due deadline at once and shows the order told; one
   per entry point that a submission queue one short of its `MAX_OUT`
   stops it, and that what stayed is taken in the next call once room is
   back. Port the assertions of the two copies, rather than inventing new
   ones; name in the report which assertion of each copy maps to which of
   yours.
4. `cargo fmt`; `heavy cargo clippy -p skein-io -p skein-io-world
   --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-io`;
   `heavy cargo nextest run -p skein-io-world --test suite -E
   'test(/stage/)'`, then the whole of `-p skein-io-world` once.
5. Commit and push.

## Report

As common.md says, plus: the bounds and why they hold; the map from the
two copies' assertions to yours; any site shape you foresee the stage not
fitting (a host whose requests are not a `Queue<Request>`, one that needs
`now` apart from `env.now`).
