+++
id = "long-pass"
part = "long-pass"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-service/src/long_pass.rs",
  "temper/crates/temper-service/src/system.rs",
  "temper/crates/temper-service/src/lib.rs",
  "temper/crates/temper-service/src/tests.rs",
  "temper/crates/temper-service/src/tests/long_pass.rs",
  "temper/crates/temper/src/host.rs",
]
effort = "high"
minutes = 45
+++

# A long pass of the service's loop writes one line

## Goal

So the next walk's `serve.log` can say whether the engine stalled: when a pass of the loop takes longer than 1 s, one line names
its longest stage and how long, at most one line in 10 s, with no payload: `temper/docs/design/protocol/README.md`, 8.1. Nothing
else about the loop changes.

Acceptance:

- **The stages:** `iterate` runs the same statements in the same order as before, now as five stages; its signature and every
  caller stay. The service's existing tests pass unedited.
- **The bound, the rate and the words** are held on supplied times: exactly 1 s writes nothing, past it writes; a second long pass
  within 10 s is held, as the longest and a count; the first pass past the 10 s writes what is held, long or not; a held line is
  readable at the end.
- **The words,** byte for byte: `temper serve: long pass: 1250 ms, engine 1200 ms of it, past the 1000 ms bound\n`, and with a
  count `temper serve: long pass: 1250 ms, engine 1200 ms of it, past the 1000 ms bound; 3 more since the last line\n`.
- **Nothing slow costs no write and no allocation:** the watch holds plain values and returns `None`.

## Inputs

- `temper/docs/plan/ctl-waits/notes/common.md`.
- `temper/docs/design/protocol/README.md`, 8.1; and 14, *The kernel's share of a pass*, what is not timed.
- `skein/docs/foundation/programming-model.md`, 2 and 10: a step takes its times as arguments; no closures, function pointers or
  traits in a step crate. That's why the adapter loops over the stages, rather than passing `iterate` a clock.

## Owns and may change

**Owns:** the `files` above.

**The survey, on the trunk this spec was cut from:** `temper_service::iterate` is called by `temper/crates/temper/src/host.rs`,
`temper/crates/temper-playground/src/service.rs`, `temper/tests/real-loop/src/service.rs` and the service's own tests, and none of
them changes, so a world has no watch. `Host::new` is called once, `temper/crates/temper/src/lib.rs`, and keeps its signature.
`skein-shell`'s `drive` does not change. This adds only public items to `temper-service`, so its dependents need no check beyond
`temper` and `temper-playground`.

**Pre-approved:** if `Note` proves a better home for the line than the adapter's own `eprint!`, it isn't: the watch belongs to the
adapter, because a world has none, so keep `eprint!` and `Note` untouched.

## Steps

**1. The stages,** in `system.rs`, failing nothing: split `iterate`'s body at its existing lines.

```rust
/// One stage of a pass, in the loop's order (protocol/README.md, 8.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage { Io, Route, Engine, Worker, Out }
pub const STAGES: [Stage; 5] = [Stage::Io, Stage::Route, Stage::Engine, Stage::Worker, Stage::Out];
pub fn iterate_stage(service: &mut Service, stage: Stage, now: Time, wall: Wall);
pub fn iterate(service: &mut Service, now: Time, wall: Wall) { /* each of STAGES, in turn */ }
```

- `Io`: the first statements, through the completions' and timers' block.
- `Route`: `route_events`.
- `Engine`: `engine::iterate`.
- `Worker`: `worker_runs`, and the whole `if let Some(worker)` block.
- `Out`: `close_sources` to `service.io.reclaim()`.

`lib.rs` exports `Stage`, `STAGES` and `iterate_stage` beside `iterate`, and `pub mod long_pass;`.

**2. The watch,** `src/long_pass.rs`, under the strict subset, with no clock:

```rust
//! A long pass of the service's loop (protocol/README.md, 8.1).
pub const LONG_PASS: Duration = Duration::from_secs(1);
pub const LINE_EVERY: Duration = Duration::from_secs(10);
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Part { Service(Stage), Names, Drain }
impl Part { pub const fn name(self) -> &'static str; } // io, route, engine, worker, out, names, drain
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LongPass { pub took: Duration, pub part: Part, pub part_took: Duration, pub more: u32 }
pub struct Watch { /* plain values */ }
impl Watch {
    pub const fn new() -> Self;
    pub fn begin(&mut self, at: Time);
    pub fn ended(&mut self, part: Part, at: Time);
    pub fn pass_ended(&mut self) -> Option<LongPass>; // the pass ends at the last `ended`
    pub const fn held(&self) -> Option<LongPass>;     // what is held, for the exit
}
```

The rule: a pass past `LONG_PASS` is long (the bound is on the pass: no part can pass it without its pass passing it). With no line
due yet, it's held: the longest pass held stays, `more` counting the others. `pass_ended` returns what's held once `LINE_EVERY` has
passed since the last line it returned, or at once if it has returned none: then the 10 s start again and nothing is held. A
later `begin` keeps what's held. Use `saturating_since` and `saturating_add`; `more` saturates.

**3. The adapter,** `temper/crates/temper/src/host.rs`: `Host` gains `watch: Watch` and `clock: skein_shell::Clock`.

```rust
fn iterate(&mut self, now: Time, wall: Wall) {
    self.now = now;
    self.watch.begin(now);
    for stage in temper_service::STAGES {
        temper_service::iterate_stage(&mut self.service, stage, now, wall);
        self.watch.ended(Part::Service(stage), self.clock.now().now);
    }
    // the names' loop, as it is
    self.watch.ended(Part::Names, self.clock.now().now);
}
```

`drain` keeps its statements; after them, `ended(Part::Drain, ..)`, then `pass_ended`, and `eprint!` of `long_pass_line` when it
returns one. `exit` writes `held()`'s line before `ending`'s lines. `long_pass_line(long: LongPass) -> String` sits beside
`note_line`, with `LONG_PASS.as_millis()` for the bound and whole milliseconds throughout.

**4. Tests:**

- `temper-service`'s `src/tests/long_pass.rs` (`mod long_pass;` in `tests.rs`), failing first, on supplied `Time`s, in the shape:
  `a_pass_of_the_bound_writes_nothing`, `a_pass_past_the_bound_names_its_longest_part`,
  `long_passes_in_the_window_are_held_as_the_longest_and_a_count`,
  `the_first_pass_past_the_window_writes_what_is_held_long_or_not`, `what_is_held_can_be_read_at_the_end`,
  `a_short_pass_holds_nothing`.
- `temper`'s `host.rs` tests, beside the other line tests: both forms of the words.
- `iterate_stage` is held by the service's existing tests through `iterate`: no table repeating `STAGES`.

**5. Checks:** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-service -p temper
--all-targets -- -D warnings`; `heavy cargo nextest run -p temper-service --lib`; the line's tests, `heavy cargo nextest run -p
temper --lib -E 'test(long_pass)'`; `heavy cargo check -p temper-playground --tests`. Commit and push.

## Report

The rule each new test broke and the failure seen. **The cost,** measured once: a throwaway untracked timing of `Clock::now()` on
this builder, gone before the push, and so what seven reads cost a pass. The design says seven clock reads, a few comparisons, no
allocation and no write: if that's untrue, it's a finding.
