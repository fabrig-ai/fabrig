# ui-needs: the plan

From `inputs/request.md`: Needs you one card at a time, and answering on Home, as the greenfield mock.
The first of three for the area; `ui-home-status` and `ui-away` chain after it.

## Parts and tasks

| Part | Task | Needs | What |
|---|---|---|---|
| `question-words` | `question-parser` | none | `question::parse`, pure, with tests |
| `stock-words` | `stock-sentence` | none | the goal's sentence in `chat.txt`, the stock figures |
| `needs-state` | `needs-session` | none | `Pages`' session state with the held send and Undo, domain only |
| `needs-frame` | `needs-frame` | `needs-session` | the standing watch, the rail's and tab's count and link, the toast |
| `needs-frame` | `journey-again` | `needs-frame` | `welcome-playground` recorded again |
| `needs-page` | `needs-page` | `question-parser`, `needs-frame` | the browser's page for a question: head, card, form, next-up, answered, Done |
| `needs-page` | `needs-kinds` | `needs-page` | proposals, held tasks, funds, person tasks |
| `home-card` | `home-card` | `needs-kinds` | Home's card, lines, answered lines, headline words, phone block |
| `needs-proof` | `needs-proof` | `home-card`, `journey-again`, `stock-sentence` | rendered against the mock, one live walk, the status closed |

## Critical path

`needs-session`, `needs-frame`, `needs-page`, `needs-kinds`, `home-card`, `needs-proof`: about 510
implementer-minutes. The view's tasks chain, since `temper-client-view`, `temper.css` and the gallery's
`names.txt` and `tests.rs` take one writer at a time. `question-parser`, `stock-sentence` and
`needs-session` go out at once, each in its own files or crate; `journey-again` runs beside the chain.

## Decisions

- **The design went with the specs,** so the request's sixth task is the proof and the status alone: the
  design text of `client/README.md` 3.5, 5.3 and 11, `browser.md` 7.2, `mockups/STATUS.md`, `agent.md` 8.2
  and `forge.md` 8.11 was written by this cut.
- **The request's first task is two,** one per crate, so both go out at once.
- **The held send goes in with the session state,** ahead of the pages (the request's fifth task, moved
  up): the pages draw it from their first commit, and nothing sends at once and changes later.
- **The page is two tasks:** a question first, the other kinds after, each holding together with its
  gallery states.
- **The standing watch opens while a project's frame shows,** not at sign-in. The request says "once signed
  in". Only the rail reads it, and at sign-in it would stale all three journeys' recordings, whereas this
  stales `welcome-playground` alone, whose walk has no sign-in. Recording it again is `journey-again`,
  which may fail with no harm to the chain: `needs-frame` and later tasks leave the journeys' test red until
  it lands, and the trunk's end check holds it.
- **`/inbox` is `/`:** the request's "bare `/inbox`" is the Inbox's unfiltered address, which prints as `/`.
- **A choice past `CUE_BYTES`** (256, what the answer's words took) is no choice, since its button could
  send nothing. **A reply** takes a person's words, up to `domain.text`, rather than 256: past 256 it would
  have gone cut with no word (`needs-page`).
- **`.dots` is a figure,** not `role=img`: the kit's `Role::Image` is an `<img>`, with no children.
- **Money reads as the task page's,** plain numbers: "S of L used.", not "$S of $L". The contract's units
  aren't dollars, and the request says the task page's form.
- **Undo restores a reply's words** to the card, unseen until the form opens: more than the mock, at no cost.
- **Leave held takes its reason in a disclosure,** as Reject does: `Decision::LeftHeld` needs one, and the
  request names no textarea for it.
- **No stored format, protocol or public interface changes.**

## Overlaps with other drafts

None with `claude-always`, `ui-signin`, `hearth-memory-3` or `debt-17` by their listed files. `look-round-5`
was still being cut when this was written, and may touch Home and `temper.css`; this plan's CSS sits under its
marker, and its `home-card` removes rules where they stand, so a catch-up there is by hand. Its goldens and
gallery files regenerate on a catch-up.
