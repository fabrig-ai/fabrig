# look-round-17: the chat terminal's render-check misses

The request: `inputs/request.md`. A look repair after `look-round-16` landed the chat's terminal, by `temper/docs/design/draft/look-requests.md`, 4: four misses temper/ui's render check found against the requests `chat-terminal` and `chat-terminal-draft`, which the user said to keep exactly so. `notes/common.md` holds the rules both tasks follow.

## Parts and tasks

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `terminal-redraw` | `tasks/terminal-redraw.md` | | ">_" draws full screen and the terminal in its press's step; "Exit" and a list's Esc redraw too | 50 |
| `terminal-look` | `tasks/terminal-look.md` | | the terminal's ground on a phone in Split; the draft's thin scrollbar; the quiet line's plain model button; the running line in the foot | 70 |

Which request each repairs: `terminal-redraw` is `chat-terminal`'s (item 1), and corrects a claim of `chat-terminal-esc`'s account; `terminal-look` is `chat-terminal`'s (items 2 and 4) and `chat-terminal-draft`'s (item 3), in a commit each.

**Critical path:** `terminal-look`, about 70 implementer-minutes. Both tasks start at once and need nothing of each other.

## What was decided

- **Item 1's cause is the shown count,** not the shell or full screen. The view builds only when the domain's shown count moves (`temper/docs/design/client/README.md`, 3 and 5.1), and `terminal::Act::Enter` sets the look and asks for full screen without moving it; `/` then redraws, since clearing or typing in the box moves it. The same gap leaves "Exit" as a press, `Theme` and `Font` as acts, and a list closed by Esc or by leaving full screen with nothing typed on the screen until the next change. The fix is the domain's: every terminal act and a list's closing move the count. The view's tests missed it because each builds a fresh view after the act; the failing test is a row of `the_look_moves_by_its_acts` on the count, the contract the view builds from. This cut wrote that line in 3.5, *Entered*.
- **The Escape landing's account was wrong** where it says the press draws the terminal (`2a62e07db5`, its page 01 and secure-context rows): the press set the look and asked for full screen, and the page drew at the next change. `terminal-redraw`'s account says so.
- **The ground on a phone** is a specificity loss: the shell's phone Split rule, `#app > .temper-session.temper-mode-split > .temper-convo`, sets `--gf-surface` over the terminal's `#app > .temper-terminal > .temper-convo`. The terminal's rule takes `.temper-session.temper-terminal`, which also beats the desktop Draft rule.
- **The draft's scrollbar:** the pane keeps the shell's `.temper-doc` rules, and with them `scrollbar-gutter: stable both-edges`; the terminal's pane rule now says `thin` and `auto`, as the request's `.tt-doc`. This cut wrote it in `browser.md`, 2.5, *The draft beside it*.
- **The quiet line's model button** loses `temper-terminal-btn`, which belongs to the top line (`browser.md`, 2.5, *Its tokens*); the request's `.tt-model` is a plain button. A view change, not a padding override, so the button also loses `-btn`'s hover ground, which the request doesn't have.
- **The running line moves into the box's form,** as the request's `.tt-run` in `.tt-foot`, rather than a negative margin: `session.rs` opens the form and its composer apart, so the chat's terminal draws the running line between them; `new_chat.rs` and `configure.rs` keep `foot`. Its margins go, since the foot's padding and `> *` rule place it.
- **The title's two rows at 1000 px** stay, by the user's word: ">_" and the status words leave the path less than its 320 px, and the design already says the row wraps there (`README.md`, 3.5, *A chat's heading is short*). `terminal-look`'s `chat-terminal` account says so.
- **Two tasks, one writer per crate:** `terminal-redraw` owns `temper-client-domain` and `README.status.md`; `terminal-look` owns `temper-client-view`, the terminal's rules in `temper.css` and `browser.status.md`. Each deletes the **Not yet** entries this cut wrote for it.
- **Tests stay cheap:** no new test function; rows of `the_look_moves_by_its_acts` and `the_terminal_says_where_it_is_and_what_runs`. The stylesheet changes take no test: the render check compares them after landing. No recorded journey reaches the terminal and prints link `temper.css` by path, so no reprint is expected.

No stored format, protocol or public interface changes. `session.rs` gains two crate-private functions.

## For the next round

`look-requests.md`, 7, asks that each miss's cause become a line of that draft; a draft is the user's, so this cut leaves them here:

- A landing checks what a press **draws**, through one view kept across the press, not a fresh view built after it, and not only what the press asks for.
- A terminal rule is checked against the shell's rules for the same element at each width and mode (the phone's Split, the pane's gutters): a scoped look loses to a more specific shell rule silently.
- A reused token brings all its declarations: a request's plain element takes no token borrowed from another part.

## Overlaps

None with the drafts beside this one: `per-wake` is jig's, `forge-read-answer` temper's engine. Within this plan the two tasks share no file.
