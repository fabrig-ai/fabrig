+++
id = "reading"
part = "reading"
kind = "feature"
needs = ["answer", "log-end"]
files = [
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/agent.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-view/src/tests/task.rs",
  "temper/crates/temper-client-view/src/tests/delegation.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/acts.rs",
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chat.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tests/client-browser/tests/browser_all/browser_pages.rs",
  "temper/tools/gallery/**",
  "temper/docs/design/client/journeys/**",
  "temper/docs/design/client/journeys.md",
  "temper/docs/design/client/browser.md",
  "temper/docs/design/client/README.md",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/mockups/STATUS.md",
  "temper/docs/design/draft/chat-reading.status.md",
]
effort = "high"
minutes = 110
+++

# Reading's pill and message, then the landing's render check and account

## Goal

"Latest ↓" over the pane's foot while a chat's end is out of view, "Loading earlier messages…" while they load, no "Earlier lines omitted N". Then the procedure's last steps for the whole request: reprint, render check, screenshots for the user, account. The last task of the look request `chat-work`, and its last commit carries the account.

Acceptance:

- A browser's chat page has a `Role::Button` named "Latest" with `LATEST` and `State::ToLogEnd`, the log's next sibling in `.temper-thread`; the terminal has none. It shows only while the log has `data-log-away`. In real Chromium: scrolled up it shows, new words don't move the view and it stays, pressed the end returns and it goes.
- Where the window has let the newest lines go (`!thread.lines.edges.reaches(End::Before)`), the button is bound to `Act::Latest`, which reads through `Thread::newer` by `ask_activity`.
- A `Role::Status` named "Loading earlier messages", with `THREAD_LOADING` and the words "Loading earlier messages…", is first in the conversation's head while `thread.reading` is `Some(Reading::Older)`, on a chat's page and an agent's, and "Older" isn't drawn while it is. Nothing is drawn once the chat's start is shown.
- `Line::Left` draws nothing and no empty region, and doesn't end a run. `Word::LinesLeft` is gone.
- "Older" stays, in one function (`older`, moved out of `conversation_head`), its rule kept: see *Decided*.
- Every rule of the overlay's *Reading* block is in `temper.css`.
- The render check, the screenshots and the account below are done, and design text agrees. Both **Not yet** entries this plan wrote that are still open in `temper/docs/design/client/README.status.md` are gone, what's deferred is named in `temper/docs/design/draft/chat-reading.status.md`, and that status's dates are today's.

## Inputs

- `temper/docs/design/draft/look-requests.md`, sections 3 to 7: the procedure, whole.
- `temper/docs/design/client/README.md`, 3.5: *Reading back*, *"Loading earlier messages…"*, *"Older"*; 5.4; 5.9.
- `temper/docs/design/draft/chat-reading.md`, section 5, for what's theirs.
- `skein/docs/design/ui-web.md`, 7, *A log*.
- `temper/docs/design/client/journeys.md`, 5, 6, 8 and 9.
- `temper/docs/plan/look-round-6/notes/contracts.md`, binding; `temper/docs/plan/look-round-6/notes/common.md`.
- The request: `git show 3f707babb9`, whole, its `overlay.css`, its pages and its `README.md`.

## Owns and may change

Owns the files in the header. **Departure from the implementer's text:** this task edits the design files in its header, for section 3, step 8, of the procedure, the account's deferred rows and the status entries; `temper/docs/design/client/README.md` only where a word of what was built differs from it.

Readers and neighbours:

- `temper/crates/temper-client-view/src/tests/delegation.rs` (near 738), `tests/chat.rs` (near 843, 2451), `tests/task.rs` (near 1129, 1603), and `temper/tests/client/tests/suite/task.rs` (near 187): "Older" stays, so these hold; only "Earlier lines omitted" goes. A test that read `LinesLeft`'s line now finds nothing.
- An exhaustive match on `Act` gains `Act::Latest`, in any file of the tree.

Decided:

- **"Older" stays until `chat-reading` (c).** The request removes it, and so do the user's words, but it's the only way to earlier lines until a log's head says it is near, which is section 5's and skein's. Removing it now would strand a long chat's history. The account's row says so, deferred to `chat-reading`, with its function named; the user is told in this plan's report.
- **Sending doesn't press the pill's way back:** Enter isn't a press of a button. Deferred to `chat-reading` (c), and named.
- **An agent's page has no pill.** Its log and page scroll otherwise, and the overlay draws none. Named in the account.
- **The pages can't be reprinted to equal the request's:** its `README.md` says so. The render check is the request's pages under its overlay against the same pages under the landed stylesheets with an empty overlay, then against this task's gallery states for the markup the view truly draws.

Options, each decided:

- If `chat-reading` (c) has landed by catch-up, take its head and window: "Older" and its rule then go, and the account's row is landed.
- The gallery's new states take a world of their own, or `Bare`'s, so that no other sample's print moves; if none can, regenerate those and say so.

Not this task: skein, jig, the line, the answer's drawing but for a fault the render check finds, which is fixed here under the trailer and named.

## Steps

1. `Word::Latest` ("Latest"), `Word::LoadingEarlier` ("Loading earlier messages…") and its name; `LATEST`, `THREAD_LOADING` and their `NAMES`; the removal of `LinesLeft`.
2. `pages/chat.rs`: `older`, the status, and `latest(b, chat, words, bindings)`, a browser's alone, after the `conversation_with_times` call inside the chat's `.temper-thread`. `pages/agent.rs`: the status beside its "Older". `temper_client_domain`: `Act::Latest` on the chat page, as `Act::Older`'s route at `pages.rs:617` and `pages/chat.rs:923`.
3. `temper.css`, under `/* ==== ui: chats ==== */`: the overlay's *Reading* rules; `.temper-latest` hidden unless `#app > .temper-session .temper-thread:has(> [role=log][data-log-away]) > .temper-latest`. If `ui-rename-focus` has set `position: relative` on the scroller, keep one rule.
4. Tests: `latest_is_a_browsers_alone_and_goes_to_the_log_end`, `latest_is_bound_only_where_the_window_let_the_end_go`, `the_message_stands_for_older_while_a_read_is_out`, `no_line_is_drawn_for_lines_left`; in `temper/tests/client-browser/tests/browser_all/browser_pages.rs`, one story on real Chromium: scroll up, pill, a new word, the view unmoved, press, the end, pill gone. Each new rule bites.
5. The gallery, `temper/tools/gallery`: states on the request's conversation, its words and calls as in `chat-work/README.md`: `chat-work-done` (the line closed), `chat-work-open` (`03`), `chat-work-running` (`01`), `chat-work-writing` (`02`), `chat-work-failed`, and a run of one call. `names.txt` sorted, `tests.rs` holding each state's parts, `browser.md`'s table of states.
6. Reprint: `heavy cargo run -p temper-client-journeys`, gallery, goldens. Recording stale: `common.md`.
7. **The terminal,** procedure 3.5: read the prints of a chat with the line, the answer and the end; nothing the request moved has lost its meaning, the report says what was read.
8. **Render check,** procedure section 4, by a throwaway script: headless Chromium at 1440, 1000 and 420, each disclosure open; each element's computed style and box, then screenshots. Into `~/src/fabrig/look-round-6/` on the builder, outside any worktree, named `NN-WIDTH-{request,landed}[-open].png`, plus the gallery's states at the same widths. The states in words are rendered through the gallery.
9. **Design text,** procedure 3, step 8: `journeys.md`, 9; journeys' `README.md` files whose words changed; `browser.md`; `mockups/STATUS.md`; `README.status.md`; `chat-reading.status.md`.
10. **The account,** in the last commit's message, one row per item below, each result true of what's on the branch by then. Check each against the code and the render, rather than copying its expected result.

    | Item | Expected |
    |---|---|
    | each of the overlay's 17 rules and its `@keyframes`, by selector | landed; `.temper-writing-end`: differs, drawn by `::after`, skein's reader closes its blocks; `.temper-thread { position: relative }`: as `ui-rename-focus` left it |
    | the line, 01, 03; its rows' markup | landed |
    | a call under way's row | differs: no outcome, where the hand-made page has one |
    | "Content in", "2164 bytes" gone | landed |
    | "Earlier text omitted", "Earlier lines omitted N" gone | landed |
    | the small block, 02 | landed |
    | a bold being written, a marker that waits, the whole live text | deferred, `chat-reading-a`, `chat-reading.status.md`, 3 |
    | the small block on a closed fenced code block | deferred, `chat-reading-a`'s `chat-live`, `client/README.status.md`, 3.5: it falls on the line after the last |
    | a numbered list's numbers | rules landed; numbers deferred, `chat-reading-d`, status, 6 |
    | "Latest ↓", 04 | landed |
    | "Loading earlier messages…", 04 | landed, as "Older" reads; as the head nears, deferred, `chat-reading` (c), status, 5 |
    | no "Older" | deferred, `chat-reading` (c), status, 5: the one way to earlier lines until then |
    | PgUp, PgDn, the position kept, sending returns, one press reads to the end | deferred, `chat-reading` (c), status, 5 |
    | state: a failed call | landed; differs: the time shares the signal's colour |
    | state: one call alone; a tool with no words; text ends a line; cards and a write apart | landed |
    | state: the three dots; a phone's width; new words while reading back | landed |
    | state: the terminal's line and rows; no pill | landed |
    | a run's time of one turn; the table's further tools; an agent's page's running state and pill | deferred, `chat-reading` (b), status, 2 and 7 |
    | render check | identical, or each difference with its reason |

    Done is every row landed or explained, by `look-requests.md`, section 5.
11. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`; `heavy cargo clippy -p temper-client-view -p temper-client-domain -p temper-client-shell-web -p temper-client-gallery --all-targets -- -D warnings`; the focused tests of those, `temper-client-journeys`, `temper/tests/client` and one browser run of `browser_pages`. Commit with the trailer, and push.

## Report

As the implementer's text asks, and: the account whole; the builder and directory holding the screenshots; which journeys were reprinted, and any re-recorded; what the user is to read on a real chat (the line's verbs, `chat-reading.md`, section 7).

## Rebase (2026-11-06, step merge, foreman look-round-6/reading/foreman-1)

merge both vocabularies and style tables, resolving overlapping identifiers and regenerating declarations.
The merge conflicts are in `temper/crates/temper-client-view/src/look.rs` and `temper/crates/temper-client-view/src/words.rs`.
Main’s conflicting changes are `8566a5434b` (sign-in waiting page) and `b0f391ffc0` (model-check status box), reconciled by `df530aacf6`.
In `look.rs`, preserve main’s welcome changes, including reserved token 367 and removal of `WELCOME_NUMBERED`.
Keep main’s welcome tokens 447–452; append the branch’s eight style names and assign `WORK_RUN` through `THREAD_LOADING` tokens 453–460 in their existing order.
Ensure each constant indexes its matching entry in `NAMES.styles`; concatenating constants without renumbering leaves collisions.
In `words.rs`, preserve main’s welcome vocabulary changes and this branch’s work phrases, Latest/loading phrases, and removal of `ToolCalls`, `ToolRepeated`, and `LinesLeft`.
Main’s `WelcomeAwayOwn` and `WelcomeAwayAgain` use IDs 3687–3688, colliding with the branch’s work phrases.
Preserve main’s IDs; assign the branch’s nineteen work phrases and two loading phrases unused IDs above main’s occupied range, checking the complete composed vocabulary. Keep `Latest` at 152.
Regenerate `ALL`, `PHRASES`, and `SEGMENTS` using `tests::accounts::english_phrase_declarations_match_the_vocabulary` with `SKEIN_CODEGEN_WRITE=1` through heavy; report regenerated declarations.
Check the merged style-name tests, English vocabulary declarations, reading controls, and welcome/model-check tests through heavy.
Regenerate and report any prints or goldens made stale by the combined code using their existing tools.
Preserve both sides’ automatically merged CSS, design text, and gallery assertions; keep this task’s reading behavior and account intact.
