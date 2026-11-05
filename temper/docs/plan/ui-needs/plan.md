# ui-needs: the plan

From `inputs/request.md`. In progress: this file grows with each release.

## Parts and tasks so far

| Part | Task | Needs | What |
|---|---|---|---|
| `question-words` | `question-parser` | none | `temper-client-domain`'s `question::parse`, pure, with tests |
| `stock-words` | `stock-sentence` | none | the goal's sentence in `chat.txt`, the stock figures |
| `needs-state` | `needs-session` | none | `Pages`' session state with the held send and Undo, domain only |

## Critical path, as it stands

`question-parser`, then the session state, the page, Home, the held send, the proof.

## Overlaps with other drafts

None for this release: neither task's files are in `claude-always`, `ui-signin`, `hearth-memory-3` or
`debt-17`. `look-round-5` is still being cut, and may touch Home and `temper.css`.

## Decisions

- The design goes with the specs, so the words form is written first (`temper/docs/design/domain/agent.md`,
  8.2) rather than in the request's sixth task, which keeps the proof and the status.
- The request's first task is split in two, one per crate, so both go out at once.
- A choice past `CUE_BYTES` (256, what `Act::CardWords` takes) is no choice: its button could send
  nothing.
- The held send goes in with the session state, ahead of the pages (the request's fifth task, moved up):
  the pages then draw it from their first commit, and nothing sends at once and changes later.
- No stored format, protocol or public interface changes so far.
