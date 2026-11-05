# ui-phone: the plan

From `inputs/request.md`. It builds the little nobody owns and proves the whole at a phone's width.

## Parts and tasks

One order, each task needing the last, as the request says: `temper.css` conflicts across plans, so it comes last. There's no fan-out for that reason.

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `viewport-bundle` | | medium, 15 | skein: the bundler's page, which is temper's served page, prints `viewport-fit=cover` |
| 2 | `phone-bits` | 1 | medium, 45 | temper's other heads; the toast and a chat's foot keep off the home bar; the closed menu answered |
| 3 | `phone-chat` | 2 | medium, 45 | the request's section 1 measured against `chatPage()`; a miss fixed |
| 4 | `phone-sweep-mock` | 3 | high, 75 | section 3's table, on the client and the mock at 390 and 420; a miss fixed in its page's place |
| 5 | `phone-sweep-gallery` | 4 | high, 90 | the four checks, every gallery state at 360, 390 and 420 |
| 6 | `phone-walk` | 5 | high, 60 | one box walk at 390 by 844 |
| 7 | `phone-proof` | 6 | high, 120 | merges main; every page beside the mock at 1440, 1000, 420 and 390 by 844 |

Design: `temper/docs/design/client/browser.md`, 2.5 (*The page covers the display*) and `skein/docs/design/ui-web.md`, 4.3. The cut wrote both; the section numbers stand.

## What the cut decided

- **The meta starts in skein.** temper's served page is the bundler's (`temper-client-bundle/build/steps.rs` passes no `--page`), so `viewport-fit=cover` goes in `skein-ui-bundle`'s page for every application, not behind a new option: nothing else consumes that page in production, and an application's own page is copied as bytes as before. Hence a first task in skein.
- **A covered page owes the home bar more than the tab bar.** Without `cover` the browser kept a chat's composer off the home bar; with it, nothing does. So a chat's foot and the toast add `env(safe-area-inset-bottom)`, beyond the request's letter. The inset is 0 wherever the mock has been measured, so they still measure as the mock draws them.
- **A chat's foot stays at `8px 16px 14px`.** The request's check says the mock's `8px 12px 14px`, but the look request `configure-temper-name` (`70e66541af`) moved it to 16 for every chat, and the request's first line says "modulo look requests already made". `phone-chat` reports it rather than undoing it.
- **The closed menu is measured, not assumed.** `frame.rs`, `banner`, puts the corner inside the closed `MENU` disclosure, so the 36 to 44px are probably a hidden box's. `phone-bits` settles it with `checkVisibility` and a screenshot.
- **The sweep's script stays off the tree,** as the request says, in `~/src/fabrig/ui-phone/sweep/` on the builder. Builders share no disk, so `notes/sweep.md` binds what it checks and a task on another builder writes it again from the note.
- **Two sweeps and a walk, not one task:** each is a stopping point whose report a later task reads. The walk is apart from the proof, since a bundle and a box are a different kind of work from renders.
- **The proof merges `origin/main`** as its first step: that's the request's catch-up (*Beside it*). A task's branch merging main is a stated departure in its spec.

## Critical path

All of it, in order: about 450 implementer-minutes.

## Overlaps

Nothing could avoid `temper.css`; the order puts this plan's use of it as late as it can be.

- **`ui-chat-draft`** (`draft-proof`, `rail-menu`, `goal-sheet`) and **`ui-away`** (`away-proof`, `home-away`, `settings-away`) change `temper.css`, `look.rs`, `words.rs` and `pages/*`. This plan's tasks stay off those plans' pages and add few lines, each where its rule stands.
- **`ui-chat-draft/chat-draft-journey`** adds journey prints whose heads predate `phone-bits`' `print.rs`. Whichever lands second reprints; the prints are generated.
- **`ui-chat-draft`'s and `ui-away`'s gallery names** conflict with nothing here: no task adds a state.
- **`temper/docs/design/client/browser.status.md`** is beside both those plans: entries are added and closed by line.

Nothing changes a stored format, a protocol or a public interface.
