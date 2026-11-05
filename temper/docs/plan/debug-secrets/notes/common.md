# debug-secrets: what every task holds to

The request is `temper/docs/plan/debug-secrets/inputs/debug-secrets.md`. No user-facing behaviour changes: a trace shows a length where it showed bytes.

## The printed form

- A hand-written `impl core::fmt::Debug` using `f.debug_struct(..)`, beside its type, in place of `Debug` in the derive; every other derive stays. A one-line comment cites the design section, as `Control` in `skein/crates/skein-channel/src/frame.rs` and `TokenRequest` in `skein/crates/skein-oauth-issuer/src/lib.rs` do.
- Fields keep their order. A field whose bytes go prints as `<field>_len`; every other field prints as it derived.
- No reshaping: no type, field, signature or behaviour changes but what your spec names.

## The test

- A made-up canary, never a real credential. For `{:?}` and `{:#?}`, assert the canary's text is absent and its rendering as a byte list is absent (the pretty form compared with spaces and line ends removed), as `debug_of_an_opening_shows_the_credentials_length_never_its_bytes` in `skein/crates/skein-channel/src/tests.rs` does. Compare `{:?}` exactly with the form your spec gives.
- The test fails on the old code. The implementer's text already has you break the rule by hand: put the derive back, see the test fail on the canary, restore it, and name that in your report.
- A step crate's test formats with `alloc::fmt::format(format_args!(..))`, as `skein/crates/skein-oauth-issuer/src/tests/token.rs` does, or `alloc::format!` under a scoped `#[expect(clippy::disallowed_macros, reason = "...")]`.
- A test that compares Debug output with a value changes with it. Anything else asserting on printed bytes is a question.

## Readers

A search of test strings and of non-Rust files for each printed form (`Header { name`, `Frame { kind`, `Body { kind`, `Spawn { program`, `Variable { name`, `Authorization { client`) finds one reader outside the types themselves: `debug_of_records_without_a_credential_is_the_derived_form` in `skein/crates/skein-channel/src/tests.rs`, which `channel-frames` changes. `jig/crates/jig-client-protocol-http/src/tests.rs` compares jig's own `Call`, whose wrappers already print `Header { name: .., value_len: .. }`: it stays, and those wrappers stay.

**Departure from the implementer's text, for a task that changes only `Debug` (every task here but `link-secret`):** no public signature changes and no test outside your crate reads the printed form, so you run your package and its world and not its direct dependents. The trunk's end check runs the wider set.

## Design files

**Departure from the implementer's text:** your spec names one entry in one status file. Remove that entry, and bring the file's date to the day you do it. A status file with nothing left says `No gap.` under its date line. Touch no other `docs/design/` file.

## What not to touch

- The carriers `skein/docs/design/lib.status.md` names under 6 stay as they are, even where a test of yours formats a machine that holds one.
- `debt-20`'s `widen-*` tasks are changing the bodies of `skein-http`, `skein-channel` and `skein-oauth-issuer`. Keep your edits to the named types and your tests, and reformat nothing beside them.
