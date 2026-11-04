+++
id = "fetched-line"
part = "fetch"
kind = "interface"
needs = []
files = ["temper/crates/temper-forge-setup/src/fetch.rs", "temper/crates/temper-forge-setup/src/tests/fetch.rs"]
effort = "high"
minutes = 20
+++

# The fetch's line, as a type

## Goal

The fetch and the box meet at one plain line. This task gives both sides that line's type, its writer and its reader, so `fetch-count` and `box-relay` can run side by side. It changes no behaviour: nothing calls them yet.

Acceptance:

- `temper_forge_setup::fetch::Fetched`, `line`, `word` and `read`, exactly as `temper/docs/plan/fetch-progress-1/notes/contracts.md`, section 2, has them;
- `read` takes back what `line` wrote, with its line feed and without, for both binaries, with a size and without, at `u64::MAX`;
- `read` refuses every other line.

## Inputs

- `temper/docs/design/protocol/setup.md`, 6.6 ("It says how far it has gone on stdout").
- `temper/docs/plan/fetch-progress-1/notes/contracts.md`, section 2, and `notes/common.md`.

## Owns and may change

Owns the two files in the header, for this type and its tests alone. `Pin`, `Fetch` and `Request` are `fetch-count`'s.

Pre-approved: if `temper-forge-setup`'s `lib.rs` must export something for `temper-box` to name `fetch::Fetched`, add that line.

## Steps

1. In `fetch.rs`, the struct and its three functions. `line` joins as `Failure::words` does, with `Decimal`. `read` is strict: lowercase word, ASCII digits only, no leading sign, one space each, `checked_mul` and `checked_add` on the numbers (overflow is `None`), a total of zero or `done` past `total` is `None`. This is a step crate: no `as`, no indexing, no `unwrap`.
2. Tests in `src/tests/fetch.rs`: `a_fetched_line_reads_back` (the round trip above) and `only_a_fetched_line_reads` (a table: empty, another word, `forgejo-runner`, a sign, a missing `bytes`, a zero total, done past total, a number past `u64`).
3. `cargo fmt`; the old-forms check; `heavy cargo clippy -p temper-forge-setup --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-forge-setup --lib`; `heavy cargo check --tests -p temper-box`. Commit and push.

## Report

As the implementer's text has it.
