+++
id = "ws-shell"
part = "world-surfaces"
kind = "interface"
needs = []
files = ["skein/crates/skein-shell/src/drive.rs", "skein/crates/skein-shell/src/lib.rs", "skein/crates/skein-shell/src/tests.rs", "skein/crates/skein-shell/src/tests/**", "skein/testing/skein-world/**"]
effort = "high"
minutes = 50
+++

## Goal

The loop and referee surfaces stop costing every world boilerplate
(`skein/docs/design/draft/component-kit.md`, 3.5; the review's D6): the
`Referee` methods every referee writes the same way get defaults; `Host`'s
`next_deadline` becomes provided, the earlier of a mechanism deadline and
the policy deadline; and a forwarding `Host` lets a world's `enum Process`
dispatcher write two accessors instead of forwarding each method. This
task adds the surface and its tests only: no world outside skein-world
moves onto it here.

Acceptance:
- the surface below in skein-shell and skein-world;
- every existing `Host` and `Referee` impl in all four trees still builds
  unchanged (each new item is additive);
- the tests below pass; skein-shell's and skein-world's tests and clippy
  pass.

The forwarding form is the cut's call on the draft's question 6.1,
recorded here: a delegate trait with one blanket impl (a generic item, as
the draft's option 1, but over an accessor trait rather than `Box<H>`).
Measured on two dispatchers (`skein/tests/echo/tests/end_to_end.rs` and
`skein/tests/world/tests/suite/end_to_end.rs`): their referees match on
the enum's variants, so `Box<dyn Host>` (option 1 as written) removes none
of their forwarding; a delegate trait removes all of it (36 of 52 lines in
echo's); option 3 removes only `next_deadline`. A concrete delegate type
(option 2) cannot name each world's variants.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`.
- `skein/docs/design/shell.md`, 12 (the loop: `Host` and `drive`);
  `skein/docs/foundation/testing-strategy.md`, 6 (the teardown invariant;
  why `next_policy_deadline` stays required) and 7 (scenarios and the referee).
- `skein/crates/skein-shell/src/drive.rs` (`Host`), `skein/testing/skein-world/src/referee.rs`
  (`Referee`), and the two dispatchers named above.

## Owns and may change

- Owns `skein/crates/skein-shell/src/drive.rs`, `src/lib.rs`'s re-exports,
  skein-shell's tests, and `skein/testing/skein-world/**`.
- skein-world's own `Host` impls and referees may move onto the new surface.
- No other crate: other worlds move in their own tasks.
- Pre-approved:
  - If the blanket impl is refused by coherence against an existing `Host`
    impl in any tree (`heavy cargo check --workspace --tests` shows it), keep
    the trait and replace the blanket impl by a `macro_rules!` in skein-shell
    that writes the forwarding impl for a named type
    (`skein_shell::forward_host!(Process);`), and say so.
  - A scoped `#[expect]` for a lint the generic impl or `dyn` trips, with its
    reason (the documented exception above).

## Steps

1. The surface, exactly:

   ```rust
   // skein-shell, drive.rs
   pub trait Host {
       // ...as today, but:
       /// Its earliest deadline that is not policy: io's close, retry and file
       /// stalls, and each layer's own (testing-strategy.md, 6). A host whose
       /// layers hold deadlines states them here.
       fn next_mechanism_deadline(&self) -> Option<Time> {
           None
       }
       /// Its earliest deadline, over every layer: the earlier of its
       /// mechanism and policy deadlines.
       fn next_deadline(&self) -> Option<Time> {
           skein_lib::earliest(self.next_mechanism_deadline(), self.next_policy_deadline())
       }
       /// (unchanged, still required)
       fn next_policy_deadline(&self) -> Option<Time>;
   }

   /// A host that is one of several, by its variant: a world's process enum.
   /// Its `Host` forwards every method to the one it holds.
   pub trait Dispatch {
       fn host(&self) -> &dyn Host;
       fn host_mut(&mut self) -> &mut dyn Host;
   }
   impl<D: Dispatch> Host for D { /* every method of Host forwarded, the
       provided ones included: state_digest, drain, exit, next_mechanism_deadline,
       next_deadline, next_policy_deadline */ }
   // lib.rs: pub use drive::{Dispatch, Host};

   // skein-world, referee.rs: defaults, as starved and last_word have
   fn act(&mut self, now: Time, procs: &mut [P]) {}
   fn next_deadline(&self) -> Option<Time> { None }
   fn overdue(&self, now: Time) -> Option<String> { None }
   // skein-world re-exports Dispatch beside Host, wherever it re-exports Host.
   ```

   `skein_lib::earliest` lands in task `lib-contract`. If it has not merged
   when you start, write the merge inline in `next_deadline`'s body (a
   `match` over the pair), and leave a line in the report: a later task
   replaces it.
2. Tests, one behaviour each, in skein-shell's tests (a two-variant enum
   over two small hosts written in the test):
   - `a_dispatcher_forwards_every_host_method_to_the_host_it_holds`
     (including `exit` and `state_digest`, the two the review found dropped);
   - `next_deadline_is_the_earlier_of_mechanism_and_policy`;
   - `a_host_that_states_only_a_policy_deadline_has_it_as_its_next`.
   And in skein-world's tests: `a_referee_that_writes_only_observe_and_passed_runs`
   (a referee with neither `act`, `next_deadline` nor `overdue` in a world
   that settles).
3. `cargo fmt`; `heavy cargo clippy -p skein-shell -p skein-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-shell -p skein-world`;
   `heavy cargo check --workspace --tests` once. This departs from the
   implementer text's "never the whole workspace": a blanket impl's coherence
   is checked against every `Host` impl in every tree, and only a workspace
   check sees them all.
   Commit and push.

## Report

The surface as it landed, whether the blanket impl held (or the macro), the
check's result, and the line counts of the two dispatchers named above as
they would be under the delegate trait (count, do not change them).
