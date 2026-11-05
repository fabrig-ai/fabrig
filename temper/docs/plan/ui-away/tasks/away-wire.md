+++
id = "away-wire"
part = "away-wire"
kind = "interface"
needs = ["home-base"]
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

# The client's wire gains `SetAway`, `Overturn`, and the subjects `away` and `decided`

## Goal

`notes/contracts.md`, *The wire*, exactly, with its helpers. Nothing reads them yet but what the compiler forces.

Acceptance:

- The schema, the generated code and the goldens agree (`temper/tests/codecs/tests/suite/drift.rs`).
- `kinds` above 15 fails the request's validation, in `temper-client-protocol-wire/src/lib.rs`' arm list.
- Round trips of both documents and both requests, and each list's bound, in `src/tests.rs`, shaped as its `project_status`
  round trip and its `DISMISSED` bound.

## Inputs

- `temper/docs/plan/ui-away/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 4.6 (`SetAway`, `Overturn`, the subjects `away` and `decided`).

## Owns and may change

Owns, as `files` lists them. May also change, compiler-forced arms only, each saying nothing new:

- `Request`: `temper-client-domain/src/tests/form.rs`, `temper-service/src/family.rs`, `temper-client-tools/src/forges.rs`,
  `src/forges/connection.rs`, `src/forges/lifecycle.rs`, shell-cli and terminal fixtures.
- `Watch`: `temper/tests/client/src/component.rs`, `temper/tools/gallery/src/samples.rs`, `src/worlds.rs`, `temper-service/src/family.rs`.
- `Outcome`: `temper-client-domain/src/forge/family.rs`, `github.rs`, `playground.rs`, `dismissed.rs`, `status.rs`, and the
  tools crate.

In `temper-service/src/family.rs` the new `Request` and `Watch` arms answer as a request or subject the service doesn't
serve, with a one-line comment naming `away-route`, which fills them.

Options, each pre-approved:

- **The family's version:** if the schema tool or `temper/tests/codecs/tests/suite/client_wire.rs` asks for a new version,
  move it and that test's name, as `to-begin.md` allows: nothing keeps an older side. Otherwise leave it.
- **A journey recording made stale by the wire:** re-record it here (`notes/common.md`).

## Steps

1. The schema. `SKEIN_CODEGEN_WRITE=1 TEMPER_BUNDLE=none heavy cargo nextest run -p temper-codecs-world --test suite -E
   'test(drift::)'`, then again unset (`temper/tests/codecs/tests/suite/drift.rs` says so at its head, and
   `temper-client-protocol-wire`'s own run may be needed for its goldens).
2. The helpers and the validation, then `src/tests.rs`.
3. The forced arms: `heavy cargo check -p temper-client-domain -p temper-service -p temper-client-tools --tests`.
4. The check of the implementer's text, `temper-client-domain`, `temper-service` and `temper-client-tools` as direct
   dependents. Commit, push.

## Report

As `notes/common.md`, Your report.
