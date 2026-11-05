+++
id = "changes-wire"
part = "changes-wire"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema",
  "temper/crates/temper-client-protocol-wire/src/generated/v1.rs",
  "temper/crates/temper-client-protocol-wire/golden/**",
  "temper/crates/temper-client-protocol-wire/src/lib.rs",
  "temper/crates/temper-client-protocol-wire/src/tests.rs",
]
effort = "medium"
minutes = 35
+++

# The subject `changes` on the wire

## Goal

The forge's family gains the watch `changes` and its document `ChangesPanel`, so that the engine's and
the client's tasks each build their half to it. The engine does not serve it yet: a `changes` watch
is refused as unknown.

Acceptance:

- The schema holds `notes/contracts.md`'s records, `Watch` gaining `changes` last.
- `changes_panel` and `decode_changes_panel` round trip the fullest and the smallest panel, and a
  `Watching` of `changes` round trips, in `temper-client-protocol-wire`'s tests.
- `test(drift::)` passes in `temper-codecs-world`.
- A `Watch::Changes` reaching `temper-service`'s `family.rs` is refused as `WatchRefused::Unknown`.

## Inputs

- `temper/docs/plan/ui-changes/notes/common.md`; `temper/docs/plan/ui-changes/notes/contracts.md`, *The wire*.
- `temper/docs/design/client/README.md`, 4.6, *A project's changes*.
- `LandingsSubject`, `LandingsPanel` and `landings_panel` are the model.

## Owns and may change

Owns `files`. May also change, only for the arm the compiler asks of `Watch`:

- `temper/crates/temper-service/src/family.rs`: `w::Watch::Changes(_)` beside `w::Watch::Change(_) |
  w::Watch::Landing(_)`, the refusal's arm; `changes-view` serves it.
- `temper/tools/gallery/src/worlds.rs` and `samples.rs`: beside `w::Watch::Landings(_)`, `continue`.
- Any test crate's exhaustive `Watch` match or a codec world's list of records
  (`temper/tests/codecs/tests/suite/client_wire.rs`, its `fuzzy/`): the new records' lines.

If a generated worst case or a price test names the wire's largest record, regenerate or lift it and
say so in the report.

## Steps

1. The schema; regenerate by `notes/common.md`, *Codecs*.
2. The two functions in `lib.rs`, beside `landings_panel`.
3. Tests: `changes_panel_round_trips` and `watching_changes_round_trips`, in the file's existing form
   (`tests.rs`, around `v1::Watch::Landings`). They only pin a generated form: no bite step.
4. Check: `heavy cargo nextest run -p temper-client-protocol-wire`, `-p temper-codecs-world --test suite
   -E 'test(drift::)'`; `heavy cargo check -p temper-service -p temper-client-gallery -p
   temper-client-tools -p temper-client-domain --tests`; fmt, old forms, clippy on what you changed.
5. Commit and push.

## Report

As the implementer's text has it, with the arms you added.
