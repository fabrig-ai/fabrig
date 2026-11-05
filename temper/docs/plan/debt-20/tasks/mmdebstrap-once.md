+++
id = "mmdebstrap-once"
part = "hearth-fakes"
kind = "feature"
needs = []
files = ["hearth/testing/hearth-fake-mmdebstrap/src/**", "hearth/tests/builder/src/hosted/mmdebstrap.rs", "hearth/docs/development/debt/D007-two-process-fakes.md"]
effort = "medium"
minutes = 20
+++

# hearth: mmdebstrap's fake resolves once

## Goal

Pay `hearth/docs/development/debt/D007-two-process-fakes.md`. `process.rs` and `hosted/mmdebstrap.rs` each hold the same private `resolve`, `insts` and `Lists`. They move once into `hearth-fake-mmdebstrap`, as a function over already-read bytes; each file keeps only its way of reading and writing files.

Acceptance:

- `rg -n 'fn resolve|fn insts|type Lists' hearth/testing/hearth-fake-mmdebstrap/src/process.rs hearth/tests/builder/src/hosted/mmdebstrap.rs` finds nothing.
- `heavy cargo nextest run -p hearth-fake-mmdebstrap -p hearth-builder-world` passes, and `heavy cargo check -p hearth-conformance-world --tests` compiles.
- By hand: change one thing in the moved function (the `NotFound` line, or drop `InRelease` from the lists), run the builder world, and see a test fail through each way of running the fake (the fake machine's process and the hosted one). If one way has no test that reaches it, say so. Restore it.
- The branch's net lines fall (estimate: about -60). The item is deleted.

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `hearth/docs/development/debt/D007-two-process-fakes.md`
- `hearth/docs/design/domain/testing.md`, 4.1

## Owns and may change

- **Owns:** the files in `files`.
- **Reader:** `hearth/tests/builder/src/hosted/mmdebstrap.rs`'s imports and its one call; `process.rs`'s likewise. `hearth/tests/conformance/src/fake_mmdebstrap.rs` reads `process` and `resolve` and shouldn't need a change: a call site only, if the compiler asks.
- **If more than `resolve` turns out to be the same** in the two (the directory list `prepare` and `start` make differ in form, so they probably stay), move it only if it's over bytes and doesn't reach files.

## Steps

1. A new `hearth-fake-mmdebstrap/src/apt.rs`, with `pub mod apt;` in `lib.rs`:

   ```rust
   //! What apt's pass answers from a mirror, over bytes already read (hearth/docs/design/domain/testing.md, 4.1).
   pub type Lists = Vec<(Box<[u8]>, Box<[u8]>)>;
   /// The lists apt fetched and its outcome; no mirror within reach answers `Outcome::Unreachable`.
   pub fn pass(mirror: Option<&Mirror>, simulate: &Simulate, sources: &[Source]) -> (Lists, Outcome);
   ```

   It takes `resolve` and `insts` unchanged, `insts` private, and the `None` arm both callers write.
2. `Mmdebstrap::start` calls `apt::pass(machine.reach(simulate.proxy), &simulate, &sources)`; the hosted `prepare` calls the same on `fake.behind.machine()`.
3. Delete the item. `cargo fmt`; `python3 -I -B hearth/scripts/old_forms.py`; `heavy cargo clippy -p hearth-fake-mmdebstrap -p hearth-builder-world --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; what you broke and what failed in each way of running the fake.
