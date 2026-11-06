# chat-reading-a: a chat's whole live text, and a committed turn that keeps its calls' names

Sections 3 and 4 of `temper/docs/design/draft/chat-reading.md`, in jig and temper, with a reader in skein. Smith doesn't change: a turn's display budget lives in `jig-protocol-transcript`, not in smith. Hearth's manifests name no jig crate.

## Parts and tasks

| Part | Task | Needs | What it makes true |
|---|---|---|---|
| `reader` | `growing-reader` | | `read_growing` reads a text still being written |
| `committed-turn` | `turn-order` | | names priced first, an outcome its first line, a cut ends in `…` |
| `live-text` | `live-whole` | | the link, the domain and the thread carry a turn being written whole |
| `conversation` | `chat-committed` | | no "Earlier text omitted", no reasoning, a result's first line |
| `live-page` (next batch) | `turn-prices` | `live-whole`, `chat-committed`, `turn-order` | temper's 131,072-byte turn, the view repriced, and the world stories |
| `live-page` (next batch) | `chat-live` | `growing-reader`, `turn-prices` | the turn being written is drawn as its committed turn, with its end |

**Critical path:** `live-whole` (or `chat-committed`) → `turn-prices` → `chat-live`. The first four start at once.

This file is brought up to date with each push.

## What didn't hold in the draft

Each is in `temper/docs/design/draft/chat-reading.status.md`, *Design departs*.

- **The order (section 4).** The draft prices inputs before outcomes. A note's input can fill the budget, and the chat's document line and "Show" read that call's outcome (`note_write`). So an outcome's first line comes before inputs, bounded by `text`.
- **A delegate's summary (section 4).** The draft keeps whole only the summaries the view reads a task's number from. The transcript knows no tool's words, so every result's summary stays whole, priced last. The drawn line is the first line, and the summaries can't take a name, a text or an outcome's line. A delegate's card reads `Delegated tasks: ...` and a note's `Note N revision...`: both are one line, so a first line breaks neither.
- **Drawn once (section 3).** The page reads the whole text again at each delivery, and a closed block's nodes come out the same, so it stays on the page unchanged. Nothing keeps a block read.
- **Whole, past the engine's bound (section 3).** `views.text_bytes` is 65,536, and so is one delivery's wire field. An answer past that arrives whole only to a page following from its start. A page opened later shows the retained end until the turn commits.

Section 7's verbs and a turn's time belong to section 2, a later plan: nothing here draws them.

## The turn budget

The turn's display budget is the client's preset, not a profile's, so `small` and `standard` give the same figures.

| Under `standard` | Today | With this plan |
|---|---|---|
| A committed turn's text (`turn_text`) | 2,048 bytes | 131,072 bytes |
| A turn being written, in a thread (`live`) | 512 bytes, its tail | 131,072 bytes, its start |
| One delivery of a run's text, received | 512 bytes | 65,536 bytes (the wire's ceiling) |
| A window's turns' text, 16 lines | 32,768 bytes | 2,097,152 bytes |
| The domain's held text (`turn_bytes`) | 1,048,576 bytes | unchanged |
| The engine's retained live text (`views.text_bytes`, 32 runs) | 65,536 bytes a run | unchanged |
| The engine's largest turn (`hub.turn_bytes`, T) | as measured, up to 33,554,432 bytes | unchanged |

**Limits raised:** `turn_text`, because an answer must commit whole. A thread's `live` is new, at `long_text`, because the turn being written must be seen whole. The link's receiving `run_text_text`, because one delivery can carry what the engine retained. No wire or store format changes.

**Heap ceilings:** the engine's `temper/tests` ceilings shouldn't move, as nothing the engine holds changes. `turn-prices` measures a full window against the shells' ceilings, and names any that moves.

## Overlaps

Each is kept to a small hunk, and each task's spec says so.

- **`temper-client-view/src/pages/chat.rs`** and `cards/words.rs`: `ui-final-walk/fix-look` has changed this crate (`temper-client-view/src/**`). No other draft names these files.
- **`temper/docs/design/client/README.status.md`:** `ui-rename-focus/rename-row` closes a line there, this plan adds and closes its own.
- **`temper/crates/temper/tests/support/person.rs`:** `box-link-first/box-readers-early` holds it, this plan removes one step.
- **`temper-client-domain/src/pages/chat.rs`:** `ui-final-walk/fix-flow`, one field here.
- **`jig-client-protocol-link/src/limits.rs`:** `turn-order` and `live-whole` both touch it, in different lines.
- **`temper-client-shell-web/temper.css`** (`chat-live`, next batch): `ui-rename-focus/rename-row`, `ui-final-walk/fix-look` and perhaps `chat-work`, one rule beside `.temper-typing`.

`temper/crates/temper-client-view/src/words.rs` isn't touched here, leaving it to `ui-walk-large`.
