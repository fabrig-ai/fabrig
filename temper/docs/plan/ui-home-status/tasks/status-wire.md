+++
id = "status-wire"
part = "status-wire"
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
minutes = 40
+++

# The client's wire gains `Looked` and the subject `status`

## Goal

The records of `temper/docs/design/client/README.md`, 4.6, on the wire, so the engine's and the client's halves can each build to them. No behaviour: an engine sent `Looked` or `Watch::Status` refuses it until `status-read`.

Acceptance:

- The schema holds exactly `notes/contracts.md`, *The wire*; the generated code and goldens are regenerated, and the drift test passes with `SKEIN_CODEGEN_WRITE` unset.
- `project_status` and `decode_project_status` exist, and a test round-trips a full and a smallest `ProjectStatus` and refuses trailing bytes.
- `heavy cargo check -p temper-service -p temper-client-domain -p temper-client-tools -p temper-client-gallery --tests` and `-p temper-client-world --tests` pass.

## Inputs

- `temper/docs/plan/ui-home-status/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 4.6: `Looked`, the subject `status`.
- The model: commit `5de7e4b6ec`, which added `ChangesSubject` and `ChangesPanel` the same way. Read its `--stat`.

## Owns and may change

Owns, as `files` lists them.

May also change, for the compiler-forced arms alone:

- `temper/crates/temper-service/src/family.rs`: `w::Watch::Status(_)` joins the `w::Watch::Change(_) | w::Watch::Landing(_)` arm, refused unknown. `w::Request::Looked(_)` joins the `unreachable!("connect or install")` list; in the main match it refuses as the record's undecodable case does. Its `Outcome`'s arm on the answer's side, if any.
- `temper/crates/temper-client-tools/src/forges/connection.rs`, `forges/lifecycle.rs`; `temper/crates/temper-client-domain/src/forge/family.rs`, `forge/dismissed.rs`; `temper/crates/temper-client-protocol-wire/src/lib.rs`'s own request match.
- `temper/tests/client/src/component.rs`, `temper/tools/gallery/src/samples.rs`, `temper/tools/gallery/src/worlds.rs`: `w::Watch::Status(_)` beside `w::Watch::Changes(_)`, skipped.

Options, each pre-approved:

- If the generator wants a limit for `branch`'s or `sentence`'s bound in `Limits`, name it `project_status_branch`, and so on, as the generator asks, and fill it in `lib.rs`'s `limits` from the bounds beside it.
- If another plan's arm has joined an enum first, this plan's still goes last.

## Steps

1. **The schema,** from `notes/contracts.md`; regenerate: `SKEIN_CODEGEN_WRITE=1 TEMPER_BUNDLE=none heavy cargo nextest run -p temper-client-protocol-wire`, then again unset.
2. **`lib.rs`:** the two functions, beside `changes_panel`.
3. **Tests,** `src/tests.rs`: `project_status_round_trips_and_refuses_trailing_bytes`, beside the changes panel's.
4. **Arms:** let `heavy cargo check` name them; each only as above.
5. **Check,** the implementer's text; `cargo fmt`, clippy on the packages you changed, then commit and push.

## Report

As `notes/common.md`, Your report.
