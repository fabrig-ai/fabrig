+++
id = "skein-kernel-routes"
part = "service-accounts"
kind = "interface"
needs = []
files = ["skein/crates/skein-io/src/kernel/routes.rs", "skein/crates/skein-io/src/kernel.rs", "skein/crates/skein-io/src/tests/**", "skein/docs/design/kernel.status.md"]
effort = "medium"
minutes = 30
+++

# skein: `kernel::Routes`, the kernel hand-off's renaming

## Goal

`J/accounts.rs` and `T/accounts.rs` (`notes/service-parts.md` names the paths) hold the same `Kernel` type:
it renames each owner's operation tokens at a process's hand-off to the kernel, and has nothing to do with
accounts. A generic mechanism goes into skein before its jig consumer. Copy it to
`skein_io::kernel::Routes`; `jig-accounts-host` and `temper-accounts-host` delete the copies.

Acceptance:

- `skein_io::kernel::Routes` exists with the four methods below, and tests of its own.
- `kernel.status.md` no longer lists `kernel::Routes`.
- **Size cap:** at most 130 lines added, tests included. Measure it, and stop and ask if it's passed.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `AGENTS.md`, `skein/AGENTS.md`.
- `skein/docs/design/kernel.md`, 5, on the trunk.
- `jig/testing/jig-test-service/src/accounts.rs`, `struct Kernel` and its `impl`: the code you copy. The copy in
  `temper/crates/temper-service/src/accounts.rs` is the same but for `has_operations`.

## Owns and may change

- Owns: the new `skein/crates/skein-io/src/kernel/routes.rs`, its `mod` and `pub use` in `kernel.rs`, a test module
  beside it in the form `skein-io` uses.
- `skein/docs/design/kernel.status.md`: remove `5: `kernel::Routes`.` from **Not yet**, and update its date line.
  This departs from the implementer's text.
- Not yours: any jig or temper file. `Kernel` stays in both services until their tasks.

## Steps

1. The type, its body the copy's, with `Map` and `Queue` from `skein_lib`:

   ```rust
   /// A process's hand-off to the kernel: each owner's tokens renamed to one space (kernel.md, 5).
   #[derive(Debug)]
   pub struct Routes { .. }

   impl Routes {
       /// Routes for at most `room` operations.
       pub fn new(room: u32) -> Self;
       /// No operation is in flight and no cancel's target waits.
       pub fn is_empty(&self) -> bool;
       /// An operation is in flight.
       pub fn has_operations(&self) -> bool;
       /// Moves records from `source` to `target` under process-wide tokens, while both have room.
       pub fn publish(&mut self, source: &mut Queue<Submit>, target: &mut Queue<Submit>);
       /// The owner's completion for a process-wide one. `sockets` is the owner's queue, in which a cancel of
       /// this target may wait.
       pub fn complete(&mut self, complete: Complete, sockets: &Queue<Submit>) -> Complete;
   }
   ```

2. Tests, each on its own: a record leaves under a new token and completes under its own; a cancel leaves naming
   its target's process-wide token, and completes naming the owner's; a target that completed while its cancel waits
   in `source` keeps its route, and the cancel still names it; with no room in `target`, nothing leaves.
3. `cargo fmt`; `heavy cargo clippy -p skein-io --all-targets -- -D warnings`; `heavy cargo nextest run -p
   skein-io --lib`; the citations check. Commit and push.

## Report

The common note's items. Break: drop the `retired` insert in `complete`, and name the failure. Lines added.
