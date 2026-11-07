+++
id = "terminal-look"
part = "terminal-look"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/pages/terminal.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/session.rs",
  "temper/crates/temper-client-view/src/tests/terminal.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 70
+++

# The terminal's ground on a phone, the draft's scrollbar, and two spacings

## Goal

Make three of the render check's misses match the requests (`temper/docs/plan/look-round-17/inputs/request.md`, 2 to 4): on a phone in Split the conversation and box keep the terminal's warm dark `#191816` beneath the document; the draft's pane scrolls with a thin scrollbar and no gutter; the quiet line's model button has no padding; and the running line is the first line of the box's form, with nothing between it and the box but its own padding.

Acceptance:

1. `temper-client-view`: the quiet line's model button (in `pages/terminal.rs`, `under`) is a plain button, with no `look::TERMINAL_BTN`, as the request's `.tt-model`; the top line's "Draft", "+ Draft" and "Exit" keep it. The chat's running line (`terminal::chat_running`) is drawn inside the box's form (`look::SESSION_FOOT`), before the composer (`look::SESSION_COMPOSER`), and no longer before the form; the browser's look and the terminal shell's print are unchanged.
2. `temper.css`, in the terminal's rules under `/* ==== ui: chats ==== */`:
   - the conversation's ground rule is scoped above the shell's Split and Draft rules: `#app > .temper-terminal > .temper-convo { ... background: var(--bg); }` becomes `#app > .temper-session.temper-terminal > .temper-convo`, so that the phone's `#app > .temper-session.temper-mode-split > .temper-convo { ... background: var(--gf-surface); }` and the desktop Draft rule no longer win;
   - `#app > .temper-session.temper-terminal > .temper-doc` gains `scrollbar-width: thin; scrollbar-gutter: auto;`, at every width;
   - `#app > .temper-terminal .temper-terminal-run { margin: 0 3ch; max-width: 100ch; ... }` loses `margin` and `max-width`, and the phone's `#app > .temper-terminal .temper-terminal-run { margin-inline: 2ch; }` goes: inside the foot, the foot's padding and its `> *` rule place it. Its padding `0 0 7px 2ch` stays.
3. Rows in `the_terminal_says_where_it_is_and_what_runs` (`temper/crates/temper-client-view/src/tests/terminal.rs`) hold 1, each made to fail once by hand (below).
4. The two `look-round-17` **Not yet** entries under 2.5 in `temper/docs/design/client/browser.status.md` are deleted, each in its request's commit.
5. Two commits at least, by request, each with its account: `Look-request: chat-terminal` (the ground, the button, the running line) and `Look-request: chat-terminal-draft` (the scrollbar).
6. The focused tests of `temper-client-view`, `temper-client-shell-web`, `temper-client-gallery` and `temper-client-journeys` pass.

## Inputs

- `temper/docs/plan/look-round-17/notes/common.md`.
- `temper/docs/plan/look-round-17/inputs/request.md`, 2, 3, 4 and *Leave alone*.
- `temper/docs/design/client/browser.md`, 2.5, *A chat's terminal*: *Its tokens*, *Its layout* and *The draft beside it*, whose lines on the model button, the running line, the ground and the scrollbar this cut wrote; `temper/docs/design/client/README.md`, 3.5, *A chat's terminal*, *The running line*, *The box* and the quiet line.
- The requests' rules and pages: `git show e8dd3197b5:temper/docs/design/client/journeys/chat-terminal/terminal.css` (`.tt-log, .tt-doc`, `.tt-foot`, `.tt-run`, `.tt-line`, the 760 px block) and `06.html` (the `footer.tt-foot` holding `.tt-run`, `.tt-box`, `.tt-list`, `.tt-line`, the line's plain `button.tt-model`).
- The code: `temper/crates/temper-client-view/src/pages/chat.rs` (`main`, where `chat_running` is called before `session_composer`; `session_composer`); `pages/session.rs` (`foot`, `foot_row`); `pages/terminal.rs` (`button`, `under`, `running`, `chat_running`); `pages/new_chat.rs` and `pages/configure.rs`, which call `session::foot` and stay as they are.

## Owns and may change

Owns the header's files. In `temper.css`, only the terminal's rules named above. Readers: `pages/new_chat.rs` and `pages/configure.rs` call `session::foot`; keep its signature, so they need no edit.

Departs from the implementer's text: the task deletes its own two **Not yet** entries in `browser.status.md`, a design status file.

Options, each decided:

- **Opening the form before the composer:** default, `session.rs` gains `pub(crate) fn form(b: &mut Builder, name: &[u8])`, which opens the `Role::Form` with its name and `SESSION_FOOT`, and `pub(crate) fn composer(b: &mut Builder)`, which opens the `SESSION_COMPOSER` slot; `foot` becomes the two in order. In `session_composer`, the terminal opens `form`, draws `super::terminal::chat_running`, then `composer`; the browser's look calls `foot` as today. If another shape keeps `foot`'s callers unchanged and reads better, use it and say so.
- **The ground rule:** default, change that rule's selector as acceptance 2 says. If raising the whole rule changes another of its declarations' winners (it shouldn't: the shell's Split and Draft rules set only `grid-row`, `background` and `min-height` there), split `background: var(--bg)` into a rule of its own under that selector instead, and say so.
- **The model button's hover:** with no `-btn`, it takes `.temper-terminal-line button:hover { color: var(--ink); }` and no ground, as `.tt-line button:hover`; add nothing.
- **A gallery or view test that pinned the running line's place or the button's token** in a way this change makes wrong: repair its expectation, keeping what it checks otherwise, and name it in the report.

## Steps

1. Rows in `the_terminal_says_where_it_is_and_what_runs`, after the existing `run` check: where `running`, the `TERMINAL_RUN` node's parent (`super::connect_forge::parent`, or the tree's own walk) is the `SESSION_FOOT` form, and it precedes that form's `SESSION_COMPOSER` slot; where `!ended`, the button inside the `TERMINAL_LINE` slot carries no `TERMINAL_BTN`, while "Exit" carries it. Run it red on the trunk (`heavy cargo nextest run -p temper-client-view --lib -E 'test(=tests::terminal::the_terminal_says_where_it_is_and_what_runs)'`).
2. The view's changes (acceptance 1); the test green. Break each rule once by hand (put `chat_running` back before the form; give the line's button `TERMINAL_BTN`), see the row fail for it, restore.
3. The stylesheet's changes (acceptance 2).
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p temper-client-shell-web --all-targets -- -D warnings`; the focused tests acceptance 6 names, one `heavy` command at a time; `python3 -I -B tools/citations/citations.py --base origin/main`.
5. Commit by request, the status entry deleted in its own request's commit, each with its account and trailer; push with `git push origin HEAD:refs/heads/<your branch>`.

The `chat-terminal` account's rows, at least:

| Item | Result |
|---|---|
| 420, pages 03 to 05: the conversation and box on `#191816` under the draft | landed: the rule and its selector |
| Pages 02, 03, 06: the quiet line's model button with no padding (29.47 px line, 26.8 on a phone) | landed: the view's plain button |
| Page 06: the running line inside the foot, 0 px above the box | landed: the form's first line, and the run's margins gone |
| A chat's title on a 1000-wide bar wraps to two rows | differs, and stays: the bar holds ">_" and its status words, which leave the path less than its 320 px (`temper/docs/design/client/README.md`, 3.5, *A chat's heading is short*), so the row wraps; the requests meeting, kept by the user's word (`look-round-17`) |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4); compiling and focused tests establish no rendered equality |

The `chat-terminal-draft` account's rows: page 03 at 1440 and 1000, the pane's thin scrollbar with no gutter, the text 621 and 401 px wide from 765 and 545; at 420, thin; and the render check's.

## Report

The implementer's report: each row broken by hand and the failure seen, the shape chosen for the form, any test whose expectation changed, and both accounts as committed.
