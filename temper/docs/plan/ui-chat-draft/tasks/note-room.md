+++
id = "note-room"
part = "note-room"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-protocol-config/src/profile/standard.rs",
  "jig/crates/jig-protocol-config/src/profile/small.rs",
  "jig/crates/jig-protocol-config/src/clients.rs",
  "jig/crates/jig-protocol-config/src/tests/profile.rs",
  "jig/crates/jig-domain-core/src/documents/project_note.rs",
  "temper/crates/temper-service/src/limits.rs",
  "temper/docs/help/reference/limits-engine.md",
]
effort = "high"
minutes = 80
+++

# A document's room

## Goal

A note's body holds a document a person reads, and everything that carries a body whole on the engine's side holds it. The client's
side is the task `person-words`.

Acceptance, jig's `domain/engine.md`, section 10, *A body's bound*:

- `notes.body_bytes`: 65,536 in `standard`, 32,768 in `small`. A box runs `standard` and overrides nothing of `notes.*` (`temper-init`'s
  `temper.json`, `bare.json`), so it follows.
- **A client's read holds a body whole:** `documents/project_note.rs`, `Note`'s `body` is cut at `limits.notes.body_bytes` and `description` at
  `limits.notes.description_bytes`, not `documents.text_bytes` (8,192 in `standard`, 128 on a box). `document_bound` prices them as they now are, if it
  prices them. The wire's check, `clients.rs`' list beside `documents.text_bytes`, gains `notes.body_bytes` against `wire.document_text_bytes`, ceiling
  65,536, with a test as `setting_names_fit_the_wire` is.
- **A call and an event hold a body:** `core.call_answer_bytes` and `hub.event_bytes` are `2 * body + 4_096`, for the JSON's escapes: 135,168 in `standard`,
  69,632 in `small`. A box's `hub.event_bytes`, `tool_payload` and `tasks.message_bytes` stay: 10,660, because a call's payload is bounded by its model's
  output (temper's `protocol/configuration.md`, 13.8). Its body's room waits for a larger model; nothing is refused for the room's sake.
- **A load holds a row:** `core.load_bytes` and temper's `loads.read_bytes` (`temper-service/src/limits.rs`) are 131,072 in `standard`; `small`'s 65,536 holds its
  32,768. Today a row past `load_bytes` is an empty page with no next and no error (`jig-domain-core/src/reads`): pin that a 65,536-byte body comes back in
  `jig/tests/core`, on the shape of `a_note_written_corrected_by_a_person_and_recalled_keeps_the_corrected_body`:
  `a_documents_whole_body_reads_back`, at `standard`'s figures, by `Read::ScopedNotes` and by a recall by name.
- Derived limits follow, by raising each at its own definition: `jig-protocol-config`'s tests hold, its pinned `store.value` by its new number, and temper's start
  checks (`startup/checks.rs`) hold.
- `jig/testing/jig-test-profile`'s and `jig/tests/notes`' own figures stay as they are.
- `jig/docs/design/domain/engine.status.md`'s line on a body's bound goes, that line alone.

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`.
- `temper/docs/plan/ui-chat-draft/inputs/request.md`: *The engine*, *jig*, *A document's room*.
- `jig/docs/design/domain/engine.md`, section 10 and 13; `jig/docs/design/protocol/configuration.md`, section 6.
- `temper/docs/design/protocol/configuration.md`, 13.8, and its status's line on 13.4, 13.8.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `jig/crates/jig-protocol-config/src/{domain,names,overrides,sizes}.rs` and its tests, for what follows.
- `jig/crates/jig-domain-core/src/reads/**` and `tests/**`, where a load's bound is read; `jig/tests/core/**`, the new story; `jig/testing/jig-test-service/src/tools.rs`.
- `temper/crates/temper-service/src/{workers,worker_settings}.rs`, `startup/checks.rs` and its tests, `temper/crates/temper-worker-domain/src/limits.rs` and its tests, each for a
  figure or a relation that follows.
- `temper/crates/temper-init/template/*.json`, `temper/docs/examples/**`, by the tests' writer, only if a check demands it, never to raise the box's three named above.
- `temper/docs/help/reference/*`: by the renderer, `temper/crates/temper/src/tests/limits.rs`, its drift test's own way.

Options, each pre-approved:

- **A derived bound past another:** where a figure makes a page, frame, row or memory check fail, raise that bound at its definition by the least that passes, and name it with its arithmetic in
  your report. It's derived, no budget. If a stored row would pass the store's own bound, or a check on a box fails that only a raised `tool_payload` would satisfy, stop there, land
  `standard`'s nearest passing body, name it in `## Left`, and say what it would take.
- **`hub.event_bytes` in `standard`:** if raising it moves more than the derived figures in `temper-worker-domain` and the turn queue, keep the event at 65,536 and say so: a call's input is then
  what bounds a body's write.
- **`note-scope` and `note-answer`** run beside you in `jig-domain-core`'s routing and in `stored.rs`: stay out of them.

## Steps

1. The figures; `heavy cargo nextest run -p jig-protocol-config`.
2. Whatever follows, by the options.
3. `project_note.rs`, the wire's check; the story, having bitten: with the old `documents.text_bytes` cut, see it cut the body; with the old `load_bytes`, see the empty page.
4. `heavy cargo nextest run -p jig-domain-core -p jig-domain-notes -p jig-protocol-config -p temper-init`; `-p temper-service --lib`; `-p temper-worker-domain`; `-p temper-client-tools`'s `limits`, if it reads them.
5. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report: every bound that moved, with its arithmetic.

## Rebase (2026-11-05, step merge, foreman ui-chat-draft/note-room/foreman-1)

resolve the engine status conflict by preserving both tasks’ gap removals
The merge failed in `jig/docs/design/domain/engine.status.md`.
Commit `238c482809` removes the body-bound gap; integration commit `e082e02833` removes the adjacent two-line gap for a written note’s answer.
Rebase onto the landing’s current integration tip and retain both removals. Do not restore any gap already removed upstream.
At `51986788f0`, the chat-scope gap remains; preserve it unless the new base has removed it.
Keep all unrelated status text unchanged.
`jig/tests/core/tests/suite/stories.rs` auto-merged without a reported conflict; preserve the whole-body readback story and upstream note-answer changes.
