# per-wake-temper: temper's settings, reference and template take charter parts and a later choice

The draft's temper half (`temper/docs/design/draft/per-wake.md`, section 9, item 2; its
sections 3, 5, 6, 7 and 8), on jig's half, landed by the plan `per-wake`. The design is the
cut's, pushed with the first batch: `temper/docs/design/client/tools.md`, 5.3 ("A charter's
parts and a later choice"); `temper/docs/design/protocol/configuration.md`, 13.9 (template
4's later choice, "Charter 5 in parts", "Each charter holds what its role acts on",
"Checked as it is read"); `temper/docs/design/domain/agent.md`, 4.3.1 (a change's `llm`
carries no `later`); each with its status's **Not yet**, one bullet per task that closes
it. The draft's status keeps lag 6's `reference-code-production.md` (tests and stage one's
judge read it) and lag 8 (a question). The shapes that cross tasks are `notes/contracts.md`;
the standing rules `notes/common.md`.

## Parts and tasks

Five tasks, within the request's seven; item 6 is dropped behind a question (below).

| Part | Task | Needs | What | Request item |
|---|---|---|---|---|
| form | `form` | | the settings file, the form's view, wire and client adapter carry `parts` and `later`; the decoder refuses a malformed part at its path; `FIELDS` and its page | 2 |
| focused | `focused` | | charters 2, 6, 7 and 8 focused; the settings test holds every charter to a stock worker's bounds and `root.journal.run_bytes` | 5, 7 |
| show | `show` | form | `settings show`'s text: a line per part with its `on` and bytes, a template's later choice; the charter and template help pages | 2 |
| reference | `reference` | form, focused | charter 5 as shared text and four parts, each rule once, focused, with the no-design rule; template 4's later choice; stage one's judge sees the parts | 3, 4, 5 |
| story | `story` | form, reference | the story in the engine's domain world (`temper/tests/engine/domain`): the cut at Opus xhigh, a question kept for the next run, which reads the questions part at Opus high | 1 |

One writer per file: `settings.json` and `reference_settings.rs` are `focused`'s, then
`reference`'s; the form crate is `form`'s, then `show`'s (`render.rs`). The design (each
task's status bullet) was written by the cut and pushed with the first batch.

## Dropped

- Item 6, a change of settings refused for a charter's stored size naming its size and the
  bound: jig's refusal detail names a charter's documents alone, in the client wire 19 and
  the people family's stored refusals, so it needs a protocol and stored-format change,
  which the request makes a question first: `plan.toml`'s `charter-size-refusal`. No task
  waits for it.

## Critical path

`form` (150 min), then `reference` (130), then `story` (150): about 430
implementer-minutes. `focused` (100) runs beside `form`, and `show` (70) beside
`reference`.

## Overlaps with drafts beside this one

- `live-change-call/run-words` (integration) lists
  `temper/docs/design/protocol/configuration.md` and `configuration.status.md`: this plan's
  design adds to 13.9 and two **Not yet** bullets, where that branch edits section 11's
  brief figure and its status's lines on it; `temper/crates/temper-service/src/tests/stock_brief.rs`,
  which `focused` and `reference` only read. Unavoidable, and textually apart.
- `tui-approval/temper-config` lists `temper/docs/help/reference/configuration.md`; `form`
  regenerates `settings-fields.md` alone, and `show` `ctl-settings.md` if it moves.
- No draft beside this one lists `temper/tests/engine/domain/**`, `temper/tools/stage-one/**`
  or `temper/docs/examples/reference/**`.
