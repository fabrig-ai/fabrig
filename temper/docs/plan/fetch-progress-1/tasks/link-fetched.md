+++
id = "link-fetched"
part = "row"
kind = "feature"
needs = ["engine-fetched", "wire-fetched"]
files = ["temper/crates/temper-service/src/route/box_link.rs", "temper/crates/temper-service/src/route.rs", "temper/crates/temper-service/src/family.rs", "temper/crates/temper-service/src/tests/box_link.rs", "temper/crates/temper-service/src/tests/family.rs", "temper/crates/temper-service/src/tests/system.rs"]
effort = "high"
minutes = 40
+++

# serve reads the `fetched` line, and the row reaches the wire

## Goal

The box's link reads `fetched FORGE BINARY DONE TOTAL` and tells the root, and the client family writes the row's `fetched` onto the wire's `SettingUp`. This joins the pipe to the forge's row: `box-relay` writes the line to the same form, and neither needs the other.

Acceptance:

- each form of the line, for the asked forge, reaches the root as `temper_engine_domain_forge::Event::Fetched`;
- a line for a forge not asked, or with none asked, is named and dropped, as a `step` line is;
- a zero `TOTAL`, a `DONE` past it, another binary's word and a malformed number are "an unknown line", named and dropped;
- the longest line reads at `box.line_bytes` 64;
- a row with `fetched` writes it on the wire's `SettingUp`, with a total and without;
- through the scripted box of `system.rs`: the row's `fetched` set from a line, then gone at the next step, at `started` and at `failed`.

## Inputs

- `temper/docs/design/protocol/forge.md`, 2.8: the box's lines, and "Named and dropped".
- `temper/docs/design/protocol/setup.md`, 7.1, the table's `fetched` row.
- `temper/docs/plan/fetch-progress-1/notes/contracts.md`, sections 3, 4 and 5, and `notes/common.md`.

## Owns and may change

Owns the files in the header, and in `route.rs` the arm alone.

Pre-approved:

- `repository-copies-1/making-run` is changing `family.rs` and `src/tests/family.rs`; keep your hunks to `connection` and its call.
- `Out::Fetched` carries the engine's `SetUpFetched`: `box_link.rs` already names `temper_engine_domain_forge`.

## Steps

1. `box_link.rs`:

   ```rust
   pub(crate) enum Out { /* its variants */ Fetched { forge: u16, fetched: SetUpFetched } }
   enum Reply { /* its variants */ Fetched(u16, SetUpFetched) }
   ```

   `decoded` reads the line, with `number`'s style for `FORGE` and a `u64` reader for the two sizes. `line`'s `matches` takes `Reply::Fetched` where it takes `Reply::Step`. `drain` pushes the event as its `Out::Removed` arm does, so `setup/lifecycle.rs` does not change.
2. `route.rs`: `Event::Fetched { .. }` joins the list of events the root routes back.
3. `family.rs`: `connection` takes `row.fetched`, and writes `w::SetUpFetched` beside `step`.
4. Tests. `tests/box_link.rs`: rows in `each_line_for_the_asked_forge_is_told`, in `a_line_for_another_forge_or_with_none_asked_is_named_and_dropped`, and a new `a_fetched_line_it_cannot_read_is_named`, `the_longest_fetched_line_fits_the_least_line_bound`. `tests/family.rs`: `a_row_s_fetched_reaches_the_wire`. `tests/system.rs`, by `a_scripted_box_answers_keyed_setup_requests_through_the_clients_family`'s way: `the_row_carries_the_latest_fetched_line_until_the_next_step`, its three endings.
5. `cargo fmt`; the old-forms check; `heavy cargo clippy -p temper-service --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-service`. Commit and push.

## Report

As the implementer's text has it, with each rule broken for a new test and the failure seen.
