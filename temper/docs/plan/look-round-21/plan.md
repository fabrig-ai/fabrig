# look-round-21: Ask's composer above its phone tabs, and the gallery's first-run Ask

The request: `inputs/request.md`. The render check after `look-round-20` found two misses of the look request `to-begin-two-steps` (`4332a26265` on `origin/look-requests`), landed by `temper/docs/design/draft/look-requests.md`, 3 and 5. `notes/common.md` holds the rules every task follows.

## Parts and tasks

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `room` | `tasks/tab-room.md` | | the tab bar's room on Ask temper's page at 420 restated without `:has()` in `:has()`; a story of the history test, compiled, measuring the composer and picker against the tabs | 50 |
| `begin` | `tasks/ask-begin-states.md` | | the five first-run Ask states given an unfinished To begin; `night-ask` based on `ask-first-run`; five `HOLDS` rows gain the rail's and tab bar's links | 70 |

**Critical path:** `ask-begin-states`, about 70 implementer-minutes. Both start at once; neither needs the other.

## What was decided

- **The room is restated in CSS alone (task 1),** `#app:has(> .temper-session):has(> .temper-tabbar):not(:has(> .temper-tabbar > a:nth-child(4)))`, rather than by a class or attribute the shell sets, which the request offers as one form among others ("such as"). The relative selector's own combinators say what the nested `:has()` said, and `:not(:has())` is accepted (the file's `scroll-padding-bottom` rule already relies on it). A marker would change the view's markup, so every journey print through To begin, and would reach `temper-client-view`, which `ui-ask-m3a` and `tui-launchpad` are changing; the selector keeps the task to one line of `temper.css`. The declaration stays: `#app` is border-box at `100dvh` on a chat's layout, so its 64 px bottom padding lifts the session's row, composer and picker with it, over the tab bar's 63 px. A scan of both stylesheets found no other `:has()` nested in `:has()`.
- **The row is a story of `history_keeps_the_chat_focus_and_filter_across_reload_and_back`** (a survey's finding, checked against the code): its `Tier::demo_asked` box chooses the playground and reaches the project ready before its first chat, the live state the render check reached, where To begin shows with the way of working left. The story goes between the playground's readiness and the chat, so `start_playground_chat`, whose one caller this is, splits into `playground_ready` and `playground_chat`. It keeps the composer's, the picker's and the tab bar's boxes; the test's body compares their edges. Written and compiled, not run: after landing runs it. No test function is added: the tier keeps twelve.
- **No static guard against nested `:has()`** in the focused suite: the request names the browser row as the test, and a new focused test of the stylesheets' grammar is a new behaviour the request didn't ask for. *A selector a browser accepts*, new in `browser.md`, 2.5, states the rule.
- **The five states take Home's unfinished To begin (task 2),** the way of working left, as `home-begin-ask-left` delivers it: what Start opens live. `ask-reference` and `ask-reference-draft`, the same world at Ask's address, keep theirs: the render check passed them and the request names five. `night-ask`'s base, `"configure"`, names no state and falls to the generic world; it becomes `ask-first-run`, which it draws at night.
- **Prints:** neither task changes the view's markup or a recording, so no journey is reprinted; the gallery's print is never committed, so the request's "regenerate their prints" is the gallery's tests passing on the new states, and `ask-begin-states` compares its print before and after (the five pages alone differ).
- **Design written here:** `browser.md`, 2.5 (*The project's frame*: composer and picker wholly over the tabs; *A selector a browser accepts*, new), 4.6, story 11 (its phone row), 7.2 (the five rows); a **Not yet** entry per task at the top of `browser.status.md`, each deleted by its task, and the status's prose no longer says the composer stands above the tabs. `README.md` already says so (3.5, 5.3) and its status takes no entry: `browser.md`, 2.5 is the stylesheet's contract, and `README.status.md` is busy with `tui-launchpad` and `ui-ask-m3a`. No draft status changes: the design doesn't lag `look-requests.md`.

No stored format, protocol or public interface changes.

## Overlaps with drafts beside this one

- **`ui-ask-m3a`'s `path-draw` (merged on its trunk) and `views-draw`** change `temper.css`; `views-draw` changes `temper/tools/gallery/**`. Unavoidable: the request's repairs are in those files. `tab-room` changes one line in `ui: phone`; `ask-begin-states` changes `ask_domain`, one night base and five `HOLDS` rows; a gallery conflict is regenerated on catch-up (the request).
- No other draft beside this one writes `browser.md`, `browser.status.md`, or `temper/tests/client-browser`.
