+++
id = "reason-shown"
part = "reason-shown"
kind = "feature"
needs = ["retry-reason"]
files = [
  "jig/crates/jig-client-protocol-wire/schema/client-v19.schema",
  "jig/crates/jig-client-protocol-wire/src/generated/v19.rs",
  "jig/crates/jig-client-protocol-wire/golden/**",
  "jig/crates/jig-protocol-engine-clients/src/translate/chips.rs",
  "jig/crates/jig-client-protocol-link/src/translate/facts/holds.rs",
  "jig/crates/jig-client-domain/src/contract/run_facts.rs",
  "jig/crates/jig-client-view/src/tree/fragments/chip.rs",
  "jig/crates/jig-client-view/src/tree/fragments/runtime.rs",
  "jig/crates/jig-client-view/src/words/words/status.rs",
  "jig/crates/jig-client-view/src/words/phrases/**",
  "temper/crates/temper-client-tools/src/jig/render/document/runs.rs",
]
effort = "high"
minutes = 120
+++

# The chip and `temper ctl task show` say the reason

## Goal

A task's retry notice carries its reason from `retry-reason` on, but the client's wire drops it. Carry it to where a person looks: the chip's retry notice, the running facts beside it, the chat's status, and `temper ctl task show`'s JSON. A held task's words already reach both through `ChipHold.words`.

Acceptance:

- `notes/contracts.md`, 4 is built. `client-v19` keeps its number; generated code and goldens come from their tools.
- The chip, `ActiveRun`'s facts and `ChatStatus::Retrying` each say the reason after the count, when there is one, and are as they were when there is none: a notice without a reason renders byte for byte as today.
- `temper ctl task show`'s retry object gains `"reason"` beside `"count"`, only when there is one.
- A test each, in the view's fragment tests and the tools' document tests, and a wire round trip. A held chip's reason is pinned by a test, adding the wiring only if that test fails.
- **This departs from the implementer's text,** for one entry alone: delete `jig/docs/design/client/domain/objects.status.md`'s "3: a retry notice's reason" **Not yet** entry.

## Inputs

- `temper/docs/plan/brief-failure/notes/common.md`; `temper/docs/plan/brief-failure/notes/contracts.md`, 2 and 4.
- `jig/docs/design/client/domain/objects.md`, 3 (the phase table); `jig/docs/design/domain/engine.md`, 9.1.
- `jig/AGENTS.md`.

## Owns and may change

Owns the files in `files`.

May also change, each only as the compiler forces it (a field, an arm, a clone where `Copy` went):

- `jig/crates/jig-client/src/facts.rs`, `jig/crates/jig-client/src/tests/fixtures.rs`, `jig/crates/jig-client-protocol-link/src/translate/facts/{chips,runs}.rs`, `jig/crates/jig-client-protocol-link/src/tests/documents.rs`, `jig/crates/jig-domain-core/src/documents/chip_phase.rs`, `jig/testing/jig-client-scripted-engine/src/codec/values/run_facts.rs`, `jig/testing/jig-client-scripted-engine/src/tests.rs`, `jig/testing/jig-test/tests/support/live_sign_in.rs`, `jig/tests/client-wire/tests/suite/fuzzy/client_wire.rs`.
- `jig/crates/jig-client-view/src/tests/tree/fragments/{chip,runtime_facts}.rs` and their goldens: a new row each; existing goldens must not change.
- `temper/crates/temper-client-tools/src/tests/fixtures/waits.rs`, and the tools' document tests.
- **`temper/crates/temper-client-view/src/tests/chats.rs`, the one `RetryNotice::OwnerFailed` literal: `reason: None`. Nothing else under `temper-client-view`:** fabrig/ui is changing it.

Options, pre-approved:

- A reason's phrase is a new `JigPhrase::RunRetryReason`, one `Hole::Name`, taking the next free phrase number; English: `Why: ` then the hole. If the book wants its own form, follow it and say so.
- `ChatStatus` or another contract type that was `Copy` through `RetryNotice` loses it too, with the clones it forces.
- Bytes that aren't UTF-8 on the wire are handled as `ChipHold.words` handles them.
- `temper-client-tools`'s `Cut` budget applies to the reason as `encode_document_text` applies it.

**The client's wire is moving on another draft** (`client-v19.schema`, `generated/v19.rs`, its goldens). Keep the schema edit to the three lines of the contract so a merge meets only those lines; a merge's generated file is regenerated, never hand-resolved.

## Steps

1. A failing test first: a chip fragment with `RetryNotice::OwnerFailed { reason: Some(..) }` says it. It can't compile before the field, so the first red is its assertion once the types are in.
2. Schema, `skein-codegen`, goldens; `translate/chips.rs::retry`, `translate/facts/holds.rs::retry_notice`, the contract.
3. The view's three places: `chip.rs`, `runtime.rs`, `status.rs`. The phrase.
4. `runs.rs`'s `encode_retry_notice`, with `encode_document_text`.
5. `cargo fmt`; old forms for jig and temper; `heavy cargo clippy -p jig-client-protocol-wire -p jig-protocol-engine-clients -p jig-client-protocol-link -p jig-client-domain -p jig-client -p jig-client-view -p temper-client-tools -p temper-client-view --all-targets -- -D warnings`; `heavy cargo nextest run` on each of those packages, then `-p jig-client-wire-world`. Name the world as its manifest has it. Check each count. `heavy cargo check -p temper-client-shell-web -p temper-client-shell-terminal --tests`.
6. Commit, push.

## Report

As the implementer's text has it, with the phrase's number, and any golden that moved and why.
