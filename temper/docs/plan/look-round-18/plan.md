# look-round-18: Ask temper's place, the render check's two misses

The request: `inputs/request.md`. A look repair after `ui-ask-m1` landed the look request `ask-temper-place` (`180f76e480` on `origin/look-requests`), by `temper/docs/design/draft/look-requests.md`, 3 and 5: the render check passed all but two misses. `notes/common.md` holds the rules every task follows.

## Parts and tasks

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `bar-tag` | `tasks/bar-tag.md` | | an Ask temper chat's bar reads "T9 · Ask temper" at 1440 and 1000, "Ask temper" at 420: `temper.css` alone | 40 |
| `first-run-gallery` | `tasks/gallery-stock.md` | | the gallery's first-run worlds list templates 1, 2 and 5, so `home-two-left` and the others draw the rail's row and the tab; the reference world's own ways leave 5 to Ask temper | 90 |
| `first-run-journeys` | `tasks/journeys-again.md` | | `project-landing`, `chat-draft` and `welcome-playground` recorded again on a fresh fake box, their prints drawing the row and the tab | 90 |

**Critical path:** `gallery-stock` and `journeys-again`, about 90 implementer-minutes each, side by side. All three start at once and need nothing of each other.

## What was decided

- **The tag is hidden by the stylesheet alone.** `temper.css`'s `.temper-session-right > p:not(.temper-byline)` hides it at every width, and the bar's right holds no other paragraph (`session::right`'s three callers: a chat, a new chat, Ask temper's page), so the rule goes rather than gaining an exception. The view already draws the tag, so no view change, which also keeps this plan off `temper-client-view`, whose names `ui-ask-drop` is changing.
- **The tag's look:** the request's page draws no chat bar; its words say the bar's " · Configuring" reads " · Ask temper". This cut wrote it as the number's continuation, `.temper-byline`'s 12 px and faint, the bar's 10 px gap taken back so that "T9 · Ask temper" reads as the Chats row's top line does; at 760 px and under the number goes, so the tag stays without its " · " (`browser.md`, 2.5, *An Ask temper chat's tag*). `ui-ask-drop`'s README text, on its trunk, says "in the look of its tag in Chats": one line with its number, small and faint, is that look in a bar whose number isn't mono.
- **The gallery's first-run worlds list what a fresh box offers:** the stock's chat (1), goal (2) and Ask temper (5, charter 11), as `temper-service`'s `stock.rs` gives them. Template 3 leaves the stock in `ui-ask-drop` and no client code reads it, so the gallery doesn't model it.
- **The reference world's collision:** its own way "Plan, then wait for me" was template 5 by charter 1, so the client took every charter 1 chat of that world for an Ask temper chat, tagged it, and made "Ask temper" current on it, which the bar's tag would now show too. Its ways move to 6 and 7, numbers the stock leaves free, and it lists the stock's Ask temper at 5, keeping the rail's row it showed. The solo world stays as it is: its Ask temper states list template 5 through `configure_frame`.
- **`project-landing`'s print can only change by recording it again:** its recording (last refreshed 2026-11-07 15:24) predates the stock's template 5 (`b41c7267b1`, 21:40), so the `me` it replays lists none. `chat-draft` runs `project-landing`'s setup and `welcome-playground`'s Demo project is offered template 5 too; both were recorded before it, and their rails lack the row the same way, so the same task records them again. `again chat-draft` last completed on 2026-11-06 (`6760272d2b`), through `project-landing`'s setup and its sign-in tab.
- **Design written here:** `browser.md`, 2.5, *An Ask temper chat's tag*, and 7.2, *The worlds are the mockups'*; `journeys.md`, 9, `project-landing`'s row; `journeys/project-landing/README.md` and `journeys/welcome-playground/README.md`; one **Not yet** entry per task in `browser.status.md` and `journeys.status.md`, each naming `look-round-18`, each deleted by its task. The client's `README.md`, 3.5, is left to `ui-ask-drop`, whose design batch rewrites that bullet ("its number and \" · Ask temper\""); `mockups/STATUS.md` needs no change.
- **Tests stay cheap:** no new test function. The gallery gains rows of `each_state_holds_what_its_name_says` and assertions in `reference_lede_names_the_worlds_templates_in_number_order`; the stylesheet takes no test; the journeys' focused replay checks the new recordings.

No stored format, protocol or public interface changes.

## Overlaps with drafts beside this one

- **`ui-ask-drop`'s `gallery`** owns `temper/tools/gallery/**`, renaming the `configure*` samples, `configure_frame` and `configuring_chip`; `gallery-stock` writes `worlds.rs`'s `frame` and the reference loop, and `tests.rs`'s `home-two-left`, `chats-first-run-1` and lede rows, and leaves the names and those helpers alone. The two meet in `worlds.rs` and `tests.rs`; unavoidable, as both change the gallery's worlds. Whichever merges second regenerates rather than hand-merges, as both plans' notes say.
- **`ui-ask-drop`'s `journeys`** records `project-landing`, `chat-draft` and `welcome-playground` again (with `welcome-set-up`) after its stock drops template 3. This plan records them first; that task starts from its trunk's tip and records them again over these, with no merge of recordings.
- **`ui-ask-drop`'s design batch** (on its trunk) edits `browser.md`'s 2.3, 2.5 (the project's frame) and 7.2's table, `journeys.md`'s section 9 prose and the status files' dates and items. This cut's lines sit in other paragraphs: a new 2.5 bullet after *The Chats page and every chat*, a sentence at the end of 7.2's *The worlds are the mockups'*, `project-landing`'s table row, and its status items away from theirs (journeys' last in the **Not yet** list, theirs first). Both date the status files 2026-11-08.
- **`ui-ask-drop`'s `client-names`** owns `temper-client-view` and `temper-client-domain`: no task here changes either.
- No other draft beside this one writes these files.
