+++
id = "io-file-debug"
part = "skein-debug"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-io/src/file.rs",
  "skein/crates/skein-io/src/file_layer.rs",
  "skein/crates/skein-io/src/file_layer/store.rs",
  "skein/crates/skein-io/src/tests/file.rs",
  "skein/crates/skein-io/src/tests/store.rs",
  "skein/docs/design/io.status.md",
]
effort = "high"
minutes = 90
+++

# skein-io's file records and file machine print a file's contents as their length

## Goal

`skein/crates/skein-io/src/file.rs`'s `Request` and `Event` derive `Debug`, so a trace of
`Store { bytes }` prints a stored record (a token record among them) in full, and so do
`WriteAt`, `Loaded` and `Read`. `FileIo` (`file_layer.rs` ~242) derives `Debug` too and prints the
same contents while they are in flight: a store's bytes (`file_layer/store.rs`, `Store.bytes`), a
read's and a load's buffers (`Pending::Read`, `LoadReading`, `LoadClosing`, each `bytes:
Vec<u8>`). Make all of them print those contents as their length, and every other field as
derived (`skein/docs/design/io.md`, 5, "A file's contents print as their length").

Acceptance:

- `Request::Store` and `WriteAt`, `Event::Loaded` and `Read` print `len: N` in their `bytes`'
  place, `N` the bytes' length, and no byte of them: `Store { owner: .., root: .., path: .., len:
  13, expected: .., no_follow: .. }`, `Loaded { owner: .., len: 13 }`.
- Every other variant of both enums prints exactly as the derived impl does (the same names,
  fields, order and nesting).
- A `FileIo` holding a store in flight, a read in flight and a load in flight, each of the
  canary, formats without the canary in either form (`notes/debug.md`); what replaces each is
  its length.
- The new tests fail on the old code and pass on the new; the variant-without-contents test
  passes on both.
- `PartialEq` and `Eq` stay derived; `Expect`, `Residue`, `Entry` and `Unsafe` keep their derives.
  No public signature, field or variant changes.
- `io.status.md` drops the clause this closes. clippy clean; skein-io's lib tests pass.

## Inputs

- `temper/docs/plan/bugs-security/notes/common.md` and `notes/debug.md`.
- `skein/AGENTS.md`.
- `temper/docs/plan/bugs-security/inputs/request.md`, item 2 ("and the same for any other
  variant carrying file contents").
- `skein/docs/design/io.md`, 5: the request list and the bullet "A file's contents print as
  their length", written by this cut; 5.3, *What io does not do*.
- For the shape: `skein/crates/skein-io/src/kernel.rs`'s `impl core::fmt::Debug for Spawn`
  (~551, with its private `EnvNames`) and its test `spawn_debug_prints_env_names_without_values`
  in `src/tests/process.rs` (~312).
- The readers, surveyed: no test in any tree asserts these types' `Debug` text, and no trace or
  golden file holds it (the skein-sim trace prints its own `Summary`;
  `smith-protocol-program`'s `"File { payload_bytes: 64, .. }"` is its own wrapper's). They
  appear in panic messages only. Types deriving `Debug` that hold them (smith-protocol-machine's
  `Below`, `BelowEvent`, `Next`; smith-local-protocol's `MarkerAction`, `Action`; smith-local-
  service's `HostIo`, which holds a `FileIo`) need nothing more.

## Owns and may change

- `skein/crates/skein-io/src/file.rs`: `Request`'s and `Event`'s derives (drop `Debug`), and two
  hand-written `impl core::fmt::Debug`, each one exhaustive `match`.
- `skein/crates/skein-io/src/file_layer.rs` and `src/file_layer/store.rs`: what `FileIo`'s
  `Debug` prints of the contents in flight, and nothing of their behaviour. The shape is yours,
  among the pre-approved options below.
- `skein/crates/skein-io/src/tests/file.rs` and `src/tests/store.rs`: the tests below, appended;
  the helpers they need from those files' rigs, reused.
- `skein/docs/design/io.status.md`: delete the clause "; file requests, events and `FileIo`
  printing their contents as a length (their `Debug` is derived and prints the bytes)" from the
  **Not yet** line of 5, keeping the rest of the line as it stands; the date line today's.

Nothing else; `io.md` stays as it is.

Pre-approved options:

- For `FileIo`: (a) a private type for contents in flight (`struct Contents(..)` in
  `file_layer.rs`, its `Debug` printing `len`), used for the four fields, their uses edited to
  match; or (b) hand-written `Debug` for `store::Store` and for the three `Pending` variants'
  holders. Take (a) unless it changes a step's behaviour or its allocation (a `Vec`'s growth, a
  box's length): then (b). Say which.
- If clippy's `missing_fields_in_debug` fires, a scoped `#[expect(clippy::missing_fields_in_debug,
  reason = "io.md, 5: a file's contents print as their length")]` on the impl.
- A private helper (a `fn` or a small struct with its own `Debug`) to keep the arms short is
  yours; the printed form is the acceptance's.
- If a test outside skein-io fails because it reads the old text of one of the four variants or
  of `FileIo`, stop and ask, naming the test.

## Steps

1. Tests first, with a made-up canary in both forms (`notes/debug.md`), formatting by
   `#[expect(clippy::disallowed_macros, reason = "...")]` as `process.rs`'s test does:

   ```rust
   // appended to src/tests/file.rs (the crate's lib tests, tests::file)
   const CANARY: &[u8] = b"canary-7f3a9c";

   #[test]
   fn debug_of_a_stored_record_shows_its_length_never_its_bytes() {
       // Request::Store { owner: OWNER, root: <a token>, path: b"tokens/a", bytes: CANARY,
       //                  expected: Expect::Any, no_follow: true }
       // and Request::WriteAt { .., bytes: CANARY }: each contains "len: 13", and neither the
       // canary's text nor format!("{:?}", CANARY)
   }

   #[test]
   fn debug_of_a_loaded_or_read_file_shows_its_length_never_its_bytes() {
       // Event::Loaded { owner: OWNER, bytes: CANARY } is exactly "Loaded { owner: <OWNER's
       // Debug>, len: 13 }"; Event::Read likewise
   }

   #[test]
   fn debug_of_records_without_contents_is_the_derived_form() {
       // Request::Load { .. } and Event::Failed { .., residue: Some(..) } each equal the
       // literal the old derived impl prints, written from a run on the old code
   }

   #[test]
   fn debug_of_file_io_hides_a_load_and_a_read_in_flight() {
       // a whole Load whose first read completed with the canary's bytes, and an open file's
       // ReadAt likewise, both still in flight: format the FileIo, the canary absent in both
       // forms, "len: 13" present
   }

   // appended to src/tests/store.rs (tests::store)
   #[test]
   fn debug_of_file_io_hides_a_store_in_flight() {
       // a Store of the canary submitted and not yet answered: format the FileIo, the canary
       // absent in both forms
   }
   ```

   Run `heavy cargo nextest run -p skein-io --lib -E 'test(/debug_of_/)'` on the old code: all
   but `debug_of_records_without_contents_is_the_derived_form` fail. Keep the output.
2. Write the two impls in `file.rs`, a line above each citing io.md, 5; then `FileIo`'s contents.
3. Narrow tests, one at a time:
   - `heavy cargo nextest run -p skein-io --lib -E 'test(/debug_of_/)'`: 5 passed.
   - `heavy cargo nextest run -p skein-io --lib`.
   - `heavy cargo check -p smith-local-service --tests` (the one holder of a `FileIo` outside
     skein-io), only if option (a) or (b) changed anything visible outside `file_layer`.
4. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `skein/`;
   `heavy cargo clippy -p skein-io --all-targets -- -D warnings`;
   `python3 -I -B tools/citations/citations.py --base origin/main` from the root.
5. One commit for the impls and their tests, one for `io.status.md`, each saying why; push.

## Report

As the implementer's text says, and: the failing run on the old code, the passing runs and their
counts, the `FileIo` option taken and why.
