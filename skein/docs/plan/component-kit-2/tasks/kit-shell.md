+++
id = "kit-shell"
part = "world-surfaces"
kind = "interface"
needs = ["kit-lib"]
files = ["skein/crates/skein-shell/src/drive.rs", "skein/crates/skein-shell/src/lib.rs", "skein/crates/skein-shell/src/tests.rs", "skein/testing/skein-world/src/referee.rs", "skein/testing/skein-world/src/lib.rs"]
effort = "high"
minutes = 35
+++
# The loop's surfaces: a provided `Host::next_deadline`, `Delegate`, `Referee` defaults

## Goal

Draft, 3.5; the review's D6. Three additive surfaces, so that hosts stop
merging their deadlines by hand, test dispatchers stop forwarding `Host`
method by method, and referees stop restating empty methods:

- `Host::next_mechanism_deadline`, provided (`None`), and
  `Host::next_deadline`, now provided: the earlier of the mechanism and
  the policy deadline. `next_policy_deadline` stays required.
- `skein_shell::Delegate`: a type that states two accessors is a `Host`,
  every method forwarded (question 6.1, decided in
  `skein/docs/plan/component-kit-2/notes/decisions.md`, 6.1).
- `skein_world::Referee::{act, next_deadline, overdue}` get defaults, as
  `starved` and `last_word` have.

No host, dispatcher or referee outside these files moves here: the stage
and world tasks move them.

Acceptance:

- The items below, with their contracts.
- skein-shell's tests cover: a host that states only the policy
  deadline gets it as `next_deadline`; one that states both gets the
  earlier; a `Delegate` forwards `iterate`, `completions`, `exit`,
  `state_digest`, `drain` and `next_mechanism_deadline` to the host its
  accessor returns. `heavy cargo nextest run -p skein-shell -p
  skein-world` passes.
- Every existing `Host` impl in every tree compiles unchanged: `heavy
  cargo check --tests` on `-p skein-sim-world -p skein-world-tests -p
  skein-echo-world -p jig-one-process-world -p temper-one-process-world
  -p temper-fake-git -p temper-service -p smith-agent-shell -p
  jig-client-shell-native`, one command (the generic wrappers among them
  are the coherence cases for the blanket impl).
- `heavy cargo clippy -p skein-shell -p skein-world --all-targets -- -D warnings`.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`;
  `notes/decisions.md`, 6.1 (why the blanket impl is allowed here).
- `skein/docs/design/shell.md`, 12 (the loop: `Host` and `drive`);
  `skein/docs/foundation/testing-strategy.md`, 6 (the teardown invariant:
  why the policy deadline stays required) and 7 (the referee).
- `skein/crates/skein-shell/src/drive.rs:12-51` (`Host`);
  `skein/testing/skein-world/src/referee.rs:44-76` (`Referee`).
- `skein_lib::earliest` (from `kit-lib`, in `skein/crates/skein-lib/src/time.rs`).

## Owns and may change

- `skein/crates/skein-shell/src/drive.rs`, `src/lib.rs` (the export),
  `src/tests.rs` (or a test module it declares).
- `skein/testing/skein-world/src/referee.rs` and `src/lib.rs` (the
  re-export `pub use skein_shell::{Delegate, Host, drive};`).
- Nothing else.

Pre-approved:

- If the blanket impl is refused by coherence (E0119) against an
  existing impl anywhere, stop with a `Question:` naming the impl; do not
  change the other impl.
- If `#[cfg(unix)]` or another attribute guards `Host`'s export, the
  `Delegate` export takes the same guard.

## Steps

1. In `drive.rs`, `Host` gains, and `next_deadline` becomes provided:

   ```rust
   /// Its earliest mechanism deadline: io's close, retry and file-stall
   /// deadlines and any other mechanism layer's (testing-strategy.md, 6).
   /// `None` by default; a host that holds io and leaves `next_deadline`
   /// provided states it.
   fn next_mechanism_deadline(&self) -> Option<Time> {
       None
   }

   /// Its earliest deadline, over every layer: the earlier of its
   /// mechanism and policy deadlines.
   fn next_deadline(&self) -> Option<Time> {
       skein_lib::earliest(self.next_mechanism_deadline(), self.next_policy_deadline())
   }
   ```

   `next_policy_deadline`'s doc keeps "Every host states it explicitly".
2. `Delegate`, in `drive.rs`, exported from the crate root beside `Host`:

   ```rust
   /// A host that is one of several (a test world's process enum): it
   /// names the host it is, and every `Host` method is forwarded to it.
   /// The blanket impl below is the one generic item in the loop's
   /// surface, the exception to programming-model.md 4's concrete
   /// composition, so that the forwarding is written once.
   pub trait Delegate {
       fn host(&self) -> &dyn Host;
       fn host_mut(&mut self) -> &mut dyn Host;
   }

   impl<D: Delegate> Host for D { /* every method, provided ones included */ }
   ```

   Every method of `Host` is forwarded, `state_digest`, `drain`, `exit`,
   `next_mechanism_deadline` and `next_deadline` included.
3. In `referee.rs`, defaults, with the docs kept:

   ```rust
   fn act(&mut self, now: Time, procs: &mut [P]) {}            // injects nothing
   fn next_deadline(&self) -> Option<Time> { None }
   fn overdue(&self, now: Time) -> Option<String> { None }
   ```

   (Name unused parameters `_now`, `_procs` as the crate's lints need.)
   Each doc says what the default does in one clause.
4. The tests of the acceptance, in skein-shell's test module, on small
   test hosts.
5. Checks of `common.md` and the cross-tree check. Commit; push.

## Report

The final items, the tests (names, count), the cross-tree check's
result, and anything coherence or dyn-compatibility made you change.
