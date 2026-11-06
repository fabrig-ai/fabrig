# ui-pgup: the plan

PgUp must scroll a chat while its agent writes and after its turn ends, with the focus in the composer, and the view must not snap back. The request asks for a reproduction before any change, so the first task changes no product code, and every fix waits for its report.

## Parts and tasks

| Part | Task | Tree | Needs | What it does |
|---|---|---|---|---|
| `reproduce` | `reproduce` | plan only | | A driver in this plan's `driver/` runs each step on a fake box at 1440 and 420, on the code before any fix; `reproduced.md` reports offsets, holds or fails, and causes |
| `log-page` | `page-follow` | skein | `reproduce` | A key's scroll is measured in its own apply, so a delivery can't restore the foot first; a pointer's press on "Latest ↓" leaves the focus |
| `live-kept` | `live-kept` | jig | `reproduce` | A run state with no attempt no longer clears the live words before their committed turn has been read |
| `ending-keeps` | `ending-keeps` | temper | `live-kept` | Tests of what a committed redraw keeps, in the view and a step at a time in a world; a fix only if they show a cause |
| `proof` | `proof` | plan, and what a failure needs | all three | The same driver on the fixed code, beside `reproduced.md`, with two screenshots; a step still failing is fixed here |

## Critical path

`reproduce`, `live-kept`, `ending-keeps`, `proof`. `page-follow` runs beside `live-kept`.

## What reading found

Each of these is read from the code, and `reproduce` shows or refutes it on a real page.

- **A key's scroll isn't a person's scroll until a frame later.** `Page::key` asks for the scroll, but `follow_log` changes only in `scrolled`, when the browser's scroll event comes, on its next frame. A delivery before it finds the log still following and restores the foot. A PgDn reaching the foot has the same gap the other way. A wheel's position and its event come together in one frame, so the two paths differ, as the request guessed.
- **A run's end can take the words away.** `Thread::run` clears the live turns at a run state with no attempt, while `Thread::end_run` waits for the committed activity. If that delivery comes first, the log collapses until the read answers, the scroll clamps to the new foot, the measure finds the view at its end, and it follows from then on. The request looked for this in skein and temper's view; the cause is in jig's thread.
- **The committed redraw's keys look right.** A provisional line and its committed first block share a key, and a test holds it. `ending-keeps` holds what the whole redraw keeps.
- **"Latest ↓" takes the focus.** A pointer's press focuses the button, which then hides, and the focus falls to the page's body.

## Decided

- **The driver is tracked in the plan's directory,** as `temper/docs/plan/bundle-loading/driver/` is, so that the proof repeats the reproduction's steps and the two reports compare. It leaves with the plan's directory. It is no tier and no test.
- **No browser story,** as the request says: the driver's report is the proof.
- **Two fixes go in whatever a sampled run saw:** the key's measure and the run's end. The request says a step that holds gets only its test, and the rest of this plan keeps to that, but each of these is a race, or an order the engine decides at that moment, so a run that doesn't see it doesn't show it can't happen. Each is a few lines, and a report says if it saw nothing.
- **The fix is in the key path.** The wheel path stays as it is, and the two then match, as the request says.
- **jig's design changes** in one line of `client/domain/client.md`, section 10: a run's end takes no line away by itself. skein's changes in section 7 of `ui-web.md`. No stored format, protocol or budget changes; skein's page part gains one public function.

## Overlaps we could not avoid

Each is a small hunk in a file another draft has changed on its trunk, not yet on main. `common.md` tells each task to keep their change.

- `page-follow`: `skein/crates/skein-ui-web/src/page.rs`, `page/web.rs`, `tests/page/events.rs` and `skein/docs/design/ui-web.status.md`, with `ui-proposal-card/focus-reveal`.
- `live-kept`: `jig/crates/jig-client/src/thread/model.rs`, `tests/thread.rs` and `jig/docs/design/client/domain/client.status.md`, with `long-words/pending-words`.
- `ending-keeps`: `temper/crates/temper-client-view/src/tests/chat.rs`, with `long-words` and `ui-proposal-card`, and `pages/chat.rs` too if a test shows a cause there. `temper/tests/client/tests/suite/live_text.rs` meets no other draft.

The design lines this cut wrote, in `skein/docs/design/ui-web.md` and `jig/docs/design/client/domain/client.md`, meet no other draft's listed files. Their status bullets stand on lines of their own, so they merge cleanly.

## Left out

- **A "Latest ↓" reached by Tab,** then pressed, hides and sends the focus to the page's body. The request has the focus in the composer throughout, so this is outside it, and `proof` only reports it.
- **A turn whose first drawn block isn't its text,** such as a call before its words, shifts that text's key, so the redraw replaces its nodes. That is rare, and `ending-keeps` fixes it only if the reproduction saw it.
