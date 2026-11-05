# report-in-finish: plan

All three of the request's changes, in one task in temper, as it asks. Nothing changes in smith or jig.

## Parts and tasks

| Part | Task | Request | Crates it writes |
|---|---|---|---|
| report-in-finish | `report-in-finish` | items 1, 2, 3 | `temper-engine-domain`, `temper-service`, `temper-engine-domain-world` |

## Critical path

The one task, about 60 minutes. It needs nothing.

## What was decided, and why

- **Item 3 is in.** `relay_payload` (`temper-engine-domain/src/route/tools.rs`) already holds the sender (`key.task`) and the target, and `goal::refusal` already refuses a call with words through `host_route::rendered` and `SettledAnswer::Host`. It is a few lines there and no protocol, schema, stored record, `tasks::Refusal` or `CallAnswer` changes.
- **Item 2 stays, with item 3's words.** A producer's plain words now stop at the call, and jig refuses its answer form for want of a credit, so the step's case is left for words already in an inbox across a restart. The request asks for it and it is three lines; its story restores the words into the inbox with a new world helper, `restore_delegate_words`.
- **"Its own producing task" is `asking_delegate`:** the producing task, a repair or a resolution. 8.11 calls these its own agents, a repair and a resolution run under the same charter, and each would pay the same turn. The stories run a producing task; `forge.status.md` already says a repair's and a resolution's relay is unobserved in stories, and that entry gains these words.
- **Under `none`, nothing changes.** Those agents hold no Message family, so authority refuses them first. The charter's sentence is shared with them and only narrows what they may do, so it misleads no one.
- **A delegate's question past the open-queue ceiling keeps the fallback feedback,** as the request's "its form isn't a question" and 8.11's ceiling paragraph say.
- **The charter's sentence:** `Summarize the change.` becomes `Summarize the change in finish; message your change only in the question form.`, 57 bytes more. Row `(2, false, 33_561, 366_574)` in `stock_brief.rs` should become `(2, false, 33_618, 366_517)`; the implementer's measure settles it and its report says so.
- **The reference coder and the judge's golden follow:** `reference_settings.rs` holds the reference's charter 2 to start with `coding.txt`, and `temper-stage-one` renders it into `fixtures/judge/brief.md`. So the task owns `settings.json`'s sentence and regenerates that golden. `forge.md` and its status are not in `.config/gate/read-by-tests.txt`; the golden is read by its own test.
- **The design:** a bullet in `temper/docs/design/domain/forge.md`, 8.11, *Its own agents' words*, and a **Not yet** in its status, which the task deletes: a stated departure from the implementer's text. 8.1 defers an enabled route's feedback to 8.11 and needs no edit. No draft covers this, so no draft's status changes.

## Overlaps with the drafts beside this one

None could be avoided, and each is small:

- `temper-service/src/tests/stock_brief.rs`: `note-parts`, `ui-away` and `ui-chat-draft` move these rows, row 2 among them. This is the likely catch-up; the spec says how to resolve it (add 57, take 57).
- `temper/docs/design/domain/forge.md`: `brief-failure`, `ui-away`, `ui-chat-draft` and `note-parts` change other sections; none touches 8.11.
- `temper/docs/design/domain/forge.status.md`: `note-parts` (merged) and `ui-away/away-proof` change other lines.
- `temper-engine-domain/src/route/tools.rs`: `note-parts` and `ui-away` add hunks in other arms.
- `temper-engine-domain/src/route/forge_route.rs`: one `use` line here, away from the hunks of `brief-failure` and `ui-away`. `repair-green`'s trunk has no change to any file here.
- `temper/tests/engine/domain/src/forge_routes.rs`: `brief-failure`, `ui-chat-draft` and `note-parts` add hunks elsewhere in the file.
