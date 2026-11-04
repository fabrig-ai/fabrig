+++
id = "writer-width"
part = "forge-client"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-client/src/identity.rs", "skein/crates/skein-forge-client/src/measure.rs", "skein/crates/skein-forge-client/src/tests/writer.rs", "skein/docs/design/forge-client.status.md"]
effort = "medium"
minutes = 25
+++

# A `Config`'s writers count at a fixed width

## Goal

`skein-forge-client`'s `identity::valid` admits a `Config` by `size_of::<Writer>()`, so what is
admitted depends on the platform. Count each writer at a published width instead, beside the
widths `measure.rs` already publishes. (Bug 9.)

Acceptance:

- `measure.rs` has `pub const WRITER_BYTES: usize = 16;`, documented as its neighbours are, and its
  `const _` block asserts `WRITER_BYTES >= size_of::<Writer>()`.
- `valid` multiplies by `WRITER_BYTES`; no `size_of` is left in `identity.rs`.
- A test pins the boundary: at `op_bytes` exactly a `Config` is admitted, one namespace byte more
  is refused.

**What stands in for a failure on the old code** (`notes/common.md`): `size_of::<Writer>()` is 16
on every target this repository builds, so behaviour cannot differ here. The test names
`crate::measure::WRITER_BYTES`, which the old code lacks; it fails there by not compiling. Say so
in the report.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/forge-client.md`, 7.1 ("`Config { namespace, writers }`").

## Owns and may change

- Owns: `src/identity.rs` (`valid`), `src/measure.rs`, `src/tests/writer.rs`.
- `skein/docs/design/forge-client.status.md`: remove this plan's **Not yet** entry (`notes/common.md`).
- `Writer` stays as it is. Nothing outside the crate reads the new constant. If the `const _`
  block needs `Writer` imported, that one `use` line.

## Steps

1. In `src/tests/writer.rs`, write `a_config_counts_each_writer_at_writer_bytes`: with two writers
   of two forges and `LIMITS` (`op_bytes` 256, `repositories` 2), a namespace of
   `256 - 2 * WRITER_BYTES` bytes is admitted by `Domain::configured`; one byte more is refused.
   Use the file's helpers.
2. The constant, its assert, and `valid`.
3. `heavy cargo nextest run -p skein-forge-client --lib -E 'test(a_config_counts_each_writer)'`,
   then the focused tests of `skein-forge-client` and `skein-forge-client-world`. `cargo fmt`,
   `heavy cargo clippy -p skein-forge-client --all-targets -- -D warnings`.
4. Commit, push.

## Report

As the implementer's text says.
