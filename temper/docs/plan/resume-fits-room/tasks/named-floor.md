+++
id = "named-floor"
part = "named-floor"
kind = "feature"
needs = ["fit"]
files = ["temper/crates/temper-service/src/startup/checks.rs", "temper/crates/temper-service/src/tests/checks.rs"]
effort = "high"
minutes = 40
+++

## Goal

temper refuses, at start, limits under which even the smallest resumed
start cannot fit the engine's link room, naming the bound:
`engine.limits.io.output: N bytes, short of the smallest resumed start,
S bytes` (`temper/docs/design/protocol/configuration.md`, 7, the
channel's checks; `temper/docs/design/domain/worker.md`, 4.3). This is
the request's item 2, *refused when it's set, naming the bound*. No
stock document reaches it: `io` takes no override, so its rows set the
engine's `io.output` on the profile's limits directly.

Acceptance:

- `checks::resumed_start` exists as
  `temper/docs/plan/resume-fits-room/notes/contracts.md`, section 4 has
  it, and runs among the engine's start checks.
- The table of Steps, 2 passes, bitten (Steps, 3).
- Every existing test of `temper-service` passes.
- The **Not yet** entry `7, the channel's: no check holds the smallest
  resumed start within the engine's io.output` in
  `temper/docs/design/protocol/configuration.status.md` is removed, its
  lines alone (`notes/common.md`, *Status lines*).

## Inputs

- `temper/docs/design/protocol/configuration.md`, 7 (*Limits checked
  against each other at start*, the channel's bullet, its last clause).
- `jig/docs/design/protocol/channel.md`, 11 (the sizes' bullet: the
  smallest resumed start).
- `temper/docs/plan/resume-fits-room/notes/contracts.md`, sections 2
  (`smallest_resumed`) and 4.
- `temper/docs/plan/resume-fits-room/notes/common.md`.

## Owns and may change

- Owns: `resumed_start` in `startup/checks.rs`, beside `hub`; its test
  in `src/tests/checks.rs`.
- Neighbour files, as named:
  - `temper/crates/temper-service/src/startup.rs`: one line,
    `checks::resumed_start(engine.as_ref().expect("required engine
    checked"), &mut problems);`, after the `checks::hub(...)` line of
    the engine's checks, so every shape with an engine runs it.
- Pre-approved:
  - **The fields.** As contracts section 4: the engine's
    `limits.core.call_records`, `limits.core.fleet.call_name_bytes`
    (through `u32::try_from`; past it, `short`'s `None`),
    `jig_protocol_worker_wire::CEILINGS.call_tool` and
    `limits.core.call_answer_bytes`, the same fields
    `crate::workers::limits` maps into the engine's `Sizes`; the room
    `limits.io.largest_room()`. Never read them through
    `crate::workers::limits`, which refuses such limits whole.
  - **The words** go through the existing `short` with `of` `b"the
    smallest resumed start"`; if the renderer's form differs from
    `PATH: N bytes, short of OF, NEEDS bytes`, keep the renderer's and
    say so.
  - **The rows** take `fixtures::engine().jig`, as the file's
    neighbouring tests do, give it `config::profile(Small)`'s or
    `(Standard)`'s limits and set `limits.io.output` on that copy; no
    fixture or figure changes. If the fixture's limits are not a
    profile's whole, take the profile's `core` and `io` and say so.
- Not yours: `workers.rs` and `worker_settings.rs`, jig, any `docs/`
  but the status entry, `temper/docs/help/**`.

## Steps

1. `resumed_start` in `startup/checks.rs`, its doc citing
   `configuration.md`, 7, and the call line in `startup.rs`.
2. **The table,** in `src/tests/checks.rs`:
   `the_smallest_resumed_start_fits_the_engines_room_or_names_it`, its
   rows run on each profile, `Small` and `Standard`:

   | row | the engine's `io.output` | expected |
   |---|---|---|
   | stock | as the profile has it | no problem |
   | met | the smallest resumed start's bytes (6,817 small, 27,201 standard) | no problem |
   | one short | that less one | `engine.limits.io.output: 6816 bytes, short of the smallest resumed start, 6817 bytes` (standard: `27200`, `27201`) |

   Assert the rendered words whole, as the file's neighbouring tables
   do.
3. **Bite.** Break the rule once by hand (compare with `<` for `<=`, or
   drop the check's call) and see the rows fail for that reason;
   restore; name it in the report.
4. **Runs.** `heavy cargo nextest run -p temper-service --lib -E
   'test(/smallest_resumed/)'`, then `heavy cargo nextest run -p
   temper-service --lib`. Check each run's count.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-service --all-targets -- -D warnings`; remove the
   status entry; commit saying why (limits under which no resumed run
   can cross the link are refused at start, naming the bound); the
   citations check; push.

## Report

As the implementer's text has it, and: each row's words per profile;
the break of Steps, 3 and the failure seen; the neighbour line edited.
