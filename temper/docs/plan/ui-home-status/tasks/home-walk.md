+++
id = "home-walk"
part = "home-walk"
kind = "feature"
needs = ["status-read", "home-domain"]
files = [
  "temper/tests/client/tests/suite/status.rs",
  "temper/tests/client/tests/suite/main.rs",
  "temper/tests/client/src/front/status.rs",
  "temper/tests/client/src/front.rs",
]
effort = "high"
minutes = 60
+++

# The meeting point: a look crosses the real roots

## Goal

The engine's half and the client's meet: the client's Home, the real roots between, a fake clock past 30 minutes. `temper/docs/design/domain/testing.md`, 2.1; shaped as `temper/tests/client/tests/suite/dismissed.rs`.

Acceptance, one story each, in `suite/status.rs`:

- `entering_home_looks_once_and_shows_status`: a first entry has no `previous`, `Status::Shown` with nothing new.
- `a_second_entry_within_the_sitting_keeps_its_last_looked`: at 29 minutes, `previous` holds and `landed_since` doesn't move.
- `coming_back_after_a_landing_says_what_landed`: a change lands while away; past 30 minutes, the look; `Facts::landed_since` is 1 and `previous` is the first sitting's last look. **The rule of `notes/common.md`:** the test that the request's literal words would fail.
- `no_look_goes_while_to_begin_shows`, then `a_skip_sends_it`.
- `a_look_reaches_another_device_of_the_party`, and another party's watch keeps its own `previous`.
- `a_restart_keeps_the_rows`: `OFFLINE`, as `dismissed.rs`.

## Inputs

- `temper/docs/plan/ui-home-status/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/domain/forge.md`, 5.6; `temper/docs/design/client/README.md`, 3.5, 4.6.
- `temper/tests/client/tests/suite/dismissed.rs`, `to_begin.rs`; `temper/tests/client/src/front.rs`, `advance` at `:992`, and `front/dismissed.rs`.

## Owns and may change

Owns, as `files` lists them.

Options, each pre-approved:

- **How a change lands in this world:** the cheapest thing that drives `landing_log::landed` for real, or a forge event the harness can hand the engine's root as `dismissed.rs`'s probe does. If only a restored row can do it, a `ProjectFacts` row restored at `OFFLINE`'s restart will do for the count; say so, and keep the landing's own count to `project-facts`' forge tests.
- **`advance` is private:** make it `pub(crate)`, or call it by the `now` `wire` and `poll` already take; either is a one-line change.

## Steps

1. **`front/status.rs`,** beside `front/dismissed.rs`, answering what `Joint` needs.
2. **The stories,** one by one.
3. **Check,** the implementer's text: `heavy cargo nextest run -p temper-client-world --test suite -E 'test(status::)'`, with the tier's budget (`temper/docs/design/domain/testing.md`), then push.

## Report

As `notes/common.md`, Your report, with each story's CPU time.
