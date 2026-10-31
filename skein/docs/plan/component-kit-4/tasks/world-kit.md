+++
id = "world-kit"
part = "world-surfaces"
kind = "interface"
needs = []
files = ["skein/crates/skein-shell/src/drive.rs", "skein/crates/skein-shell/src/lib.rs", "skein/testing/skein-world/**", "skein/tests/world/**"]
effort = "high"
minutes = 75
+++

## Goal

The loop's and the referee's surfaces stop costing every world its
boilerplate (`skein/docs/design/draft/component-kit.md, 3.5`): `Referee`'s
`act`, `next_deadline` and `overdue` get defaults; `Host::next_deadline`
becomes provided, the earlier of a new mechanism deadline and the policy
deadline; and a `Forward` trait lets a world's `enum Process` dispatcher
shrink to its two accessors. tests/world's three dispatchers move onto it,
as the measure of question 6.1.

The run's decision on question 6.1 (the forwarding `Host`): a forwarding
trait in skein-shell with one blanket impl, the review's "Delegate trait"
(option 2 of the draft's 6.1 in its trait form). skein-shell is a shell
crate, not step code: it already defines `Host` and takes `impl Host`, so
programming-model 10's ban on traits and generic items does not reach it.
The step crates see none of it.

Acceptance:

- The items below exist as sketched, documented.
- Every existing `Host` and `Referee` impl in the workspace compiles
  unchanged: the change is additive (step 5 checks the generic impls in
  other trees).
- tests/world's three dispatchers (`tests/suite/end_to_end.rs`,
  `writer_pipe.rs`, `real_conformance.rs`) implement `Forward` instead of
  `Host`; the `forwarding_host` count falls by three.
- Trivial `act`, `next_deadline` and `overdue` overrides (an empty body,
  or `None`) in skein-world and tests/world are removed.
- skein-shell's, skein-world's and tests/world's tests pass.

## Inputs

- `skein/docs/plan/component-kit-4/notes/common.md`.
- `skein/docs/design/draft/component-kit.md, 3.5 and 6.1`.
- `skein/docs/design/shell.md, 12` (`Host` and `drive`); `testing.md`
  where it names `Referee` (around line 255).
- `crates/skein-shell/src/drive.rs` (the `Host` trait, 1-51; `drive`'s use
  of the deadlines); `testing/skein-world/src/referee.rs:43-80`.
- A dispatcher as it is: `tests/echo/tests/end_to_end.rs:14-67` (do not
  change it: another task owns tests/echo).

## Owns and may change

- Owns `crates/skein-shell/src/drive.rs` and `lib.rs` (the export),
  `testing/skein-world/` and `tests/world/`.
- tests/world's io stage site (`src/bin/terminal_fixture.rs`) is not
  yours: a later task moves it.
- Pre-approved: if the blanket impl fails coherence anywhere in the
  workspace, stop and report `Question:` with the error and the impls it
  names; do not work around it.

## Steps

1. `Host` (drive.rs), additive:

   ```rust
   /// Its earliest mechanism deadline: io's close, retry and file stalls,
   /// and any other layer's that is not policy. A host with io states it;
   /// the default, `None`, is for a host with no mechanism of its own.
   fn next_mechanism_deadline(&self) -> Option<Time> {
       None
   }

   /// Its earliest deadline, over every layer: the earlier of the
   /// mechanism and the policy deadlines.
   fn next_deadline(&self) -> Option<Time> { /* the earlier of the two */ }
   ```

   `next_policy_deadline` stays required, as its doc says. A host that
   overrides `next_deadline` today keeps working unchanged; write the
   provided body without a private helper (skein-lib's `earliest` comes
   with task lib-kit, which this task does not wait for: an inline
   `match` or `[a, b].into_iter().flatten().min()` is right here).
2. `Forward` (drive.rs, exported from skein-shell's root and re-exported
   by skein-world beside `Host`):

   ```rust
   /// A world's process that is one of several hosts, named by its two
   /// accessors: every `Host` method forwards to the host it is, the
   /// provided ones included, so none drifts.
   pub trait Forward {
       fn host(&self) -> &dyn Host;
       fn host_mut(&mut self) -> &mut dyn Host;
   }

   impl<F: Forward> Host for F { /* each method, forwarded */ }
   ```

   Forward every method, `state_digest`, `drain`, `exit`, `next_deadline`
   and `next_mechanism_deadline` included.
3. `Referee` (referee.rs): defaults `act` (does nothing), `next_deadline`
   (`None`), `overdue` (`None`), each doc saying what the default means
   ("a referee that acts on nothing", "keeps no deadline of its own",
   "never overdue"). Remove the trivial overrides in skein-world and
   tests/world.
4. tests/world's three dispatchers: replace each `impl Host for Process`
   with `impl Forward for Process` (the enum and its accessors stay). Where
   a dispatcher forwarded `exit` or `state_digest` and another did not,
   forwarding now covers both: if a test changes outcome, that test
   relied on the drift: report it and keep the forwarding.
   Measure for question 6.1, for `end_to_end.rs` and `writer_pipe.rs`: the
   dispatcher's lines before, after (option 2), and with the defaults alone
   (option 3: count only, do not write it).
5. `cargo fmt`; `heavy cargo clippy -p skein-shell -p skein-world -p
   skein-world-tests --all-targets -- -D warnings`; `heavy cargo nextest
   run -p skein-shell -p skein-world -p skein-world-tests`. The generic
   `Host` impls elsewhere must still compile: `heavy cargo check --tests`
   with `-p skein-sim-world -p temper-fake-git` and the packages holding
   `jig/tests/one-process/src/world_signals.rs` and
   `temper/tests/one-process/src/machine.rs` (find their names in their
   `Cargo.toml`).
6. `old_forms.py --write` (common.md); commit and push.

## Report

As common.md says, plus the 6.1 measurements (lines per dispatcher for
each option), and any `Host` impl in skein whose `next_deadline` is a hand
merge of a mechanism deadline and its policy deadline that you saw (the
later tasks move them).
