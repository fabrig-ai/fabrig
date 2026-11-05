# note-parts: the plan

jig's notes as an LLM's working area: `note` takes a line range, so that a body past what one call holds, about 10.6 KB on a box, is built
in parts up to `notes.body_bytes`. As the request says: `jig/docs/plan/note-parts/inputs/request.md`.

## The parts

Design first, in the cut itself: jig's `domain/engine.md`, 10 and 16, `client/domain/contract.md`, 6, `draft/notes-memory.md`, 5, and
temper's `protocol/configuration.md`, 13.8, `domain/agent.md`, 8.3, `client/tools.md`, 5.2. The shapes that cross tasks are in
`notes/contracts.md`.

| Part | Task | Needs | Minutes | Says |
|---|---|---|---|---|
| `note-range` | `note-range` | | 85 | jig: machine, core, store, jig's tool and words; pure tests and world refusals |
| `task-notes` | `task-notes` | | 60 | temper ctl: `task notes`, the pane's read, which the proof needs and ctl lacks |
| `note-tool` | `note-tool` | `note-range` | 60 | temper: schema, decoder, words, route and the stock instructions |
| `note-walk` | `note-walk` | `note-tool`, `task-notes` | 60 | the proof, through ctl on a fake box |

## The critical path

`note-range` (85) → `note-tool` (60) → `note-walk` (60): 205 implementer-minutes, `task-notes` (60) beside the first. Then the trunk's end
check, which waits on `ui-chat-draft`'s landing: `after = ["ui-chat-draft"]`, as the request has it.

## What the cut decided

Public interfaces and stored formats change in place, with no question, as the request allows.

- **The revision read stays `recalled`.** The request names it `revision`, and also keeps a call without `first` as it is today. `recalled`
  already means the revision read, and a range requires it. A second name for one fact, or a rename of the whole form across a template and its
  scripts, buys nothing. A rename stays cheap later, since there's no deployment.
- **`first` and `remove` come together.** `first` alone would silently insert where a model meant to replace; it's refused, naming `remove`.
- **`scope` and `description` stay required in both forms.** One schema; the description is written as named. `references` left out keeps the
  entry's, since a range changes lines.
- **A newline is added only where two lines would otherwise join:** `notes/contracts.md` has the table, a body with no final newline and an empty
  one among its rows.
- **New refusals, stored in place:** `PastEnd { lines }` and `PastBound { bytes, bound }` in `core-v6`'s `NoteRefusal`, with no version bump. A body
  past the bound says so in the whole form too, since the bound is one rule.
- **A later write's words close with its lines,** `· +18 −6 · line 31 · 64 lines`. The request keeps the answer's values and how they're computed,
  and they don't change. But an append's `first` is the body's lines + 1, and a later write printed no total, so a model had to add up: a slip there
  inserts in the wrong place with no refusal.
- **`task notes`, a new ctl verb,** which the request doesn't name. Its proof asks for the pane's read through ctl on a fake box, ctl has no read of a
  note, and the pane's page isn't on this trunk yet. The verb makes that same read (`Read::Notes` by the task's scope). fabrig/qa's walks can use it too.
- **The walk's script is the walk's own,** not `temper-init`'s template, which would carry 30 KB of filler in every box's `temper.json`. The default
  is two runs of a `Walk`, the first making the retained sandbox; `note-walk` lists its fallbacks in order, and the template comes last.
- **No client wire change:** a body is stored whole, and a run's answer reaches a client as its words.
- **`recall` keeps no line range,** as the request says, until a walk shows it's needed: `jig/docs/design/domain/engine.md`, section 16.
- **The draft's own text changed.** `notes-memory.md`, 5's *Written by the run's note tool* now says what the design does, because the request
  names that draft as saying the same. Its status records who decided. The rest of the draft is untouched; kinds and the working scope stay held.
- **hearth uses no jig notes:** it names skein alone, and `rg` finds no `jig-domain-notes` in hearth, smith or skein. fabrig/hearth isn't told.

## The overlaps that couldn't be avoided

- **`ui-chat-draft/chat-state`'s `note_write`** (`temper-client-domain/src/pages/chat/document.rs`, running) parses a later write's words, and
  `note-tool` adds to their end. If it holds the end strictly, it needs the closing ` · L lines` at this trunk's catch-up; its `chat_draft` world
  breaks there if so. That's the one known red ahead, and the end check would show it. `chat-draft-journey`'s prints may show the new words too:
  regenerate them, as the request says.
- **`temper/crates/temper/tests/fake_backends/walk.rs`:** `ui-welcome/first-run-walks` is running there; `note-walk` adds one `mod` line.
- **`temper/docs/help/reference/*`** are generated: regenerate at a catch-up rather than hand-merge.
- **`jig-domain-core`:** `ui-chat-draft/notes-list` is in `reads*`, and `note-range` is in `routing`, `vocabulary.rs`: no shared file.
- **Forced lines across trees:** `note-range` carries one temper arm, `temper-engine-smith/src/answers.rs`' `NoteRefused`, so that the workspace builds;
  its words are final there, and `note-tool` pins them.

## What isn't proved on a real model

The proof runs a script. Unobserved until a real model builds a document: whether it counts lines right, appends rather than rewrites, and recovers from
a refusal. `temper/docs/design/domain/agent.status.md`, 8.3 says so once `note-walk` lands. The pane's own drawing of a long document is `ui-chat-draft`'s;
`note-walk` reads what the pane reads and draws nothing.

## Not this plan

`propose_settings` naming a note: fabrig/qa's, under finding-330, which needs `first` and `remove` from here. Note kinds and the working scope of
`notes-memory.md` stay held.
