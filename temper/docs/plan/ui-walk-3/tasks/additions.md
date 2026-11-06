+++
id = "additions"
part = "additions"
kind = "docs"
needs = []
files = [
  "temper/docs/plan/ui-walk-3/additions.md",
  "temper/docs/plan/ui-walk-3/driver/additions.py",
  "temper/docs/plan/ui-walk-3/driver/focus.py",
]
effort = "high"
minutes = 120
+++

# What the second walk never held: A4 ten times, the rename form, the scroll bars, the plans landed since

## Goal

Walk the request's additions other than W3, W6 and R1, which `walk` and `real-login` hold. Each on fresh fake boxes, at 1440 and at 420 where it has a width, beside the mock or the design's words. This task fixes nothing in the client: its product is `additions.md`.

Acceptance: `additions.md` has the table (`notes/walk.md`, The table; `step` is the name below), then the findings table (`notes/common.md`), then `owed` lines. A state left out is a failure of this task. `driver/additions.py` runs each section by name, and its head says how.

**A4.** On a fresh box: enable the features, send "Start a local away goal.", open Needs you, and press exactly Later; Reply…; "Your reply" filled with "Use the small implementation"; Send reply; Undo; then the offered choice "Use the small implementation" at once, with no wait, while the goal runs. Then, past the five-second hold, the card reads answered, the question and its count have gone, and `temper ctl inbox --json` no longer holds it unanswered. **At least ten repeats, at least three of them at 420.** A table of each repeat: width, seconds from Undo to the press, what it showed. Every answer sends; any that doesn't is a `large` finding with both a failing and a passing pair kept, and the ctl JSON. A toast "That answer wasn't sent. Press it again." counts as not sent, and its row says what caused it.

**The rename form, focus and the Owner menu** (`notes/decisions.md`, its last section; `temper/docs/design/client/greenfield/app.js`, 229-240). The open project's row becomes the form, a name field filling the row, min-height 34px, and a primary small Save; "Rename NAME" leaves the menu's acts; no visible Cancel. Enter saves. Escape cancels. An empty name keeps the old one. After Save, Enter and Escape: the focused element, by name and tag, is the summary or the opener, never `main`; `main` and `h1` draw no outline; the next Tab shows a real control's ring. Numbers in `focus-metrics.json`. A2 and A3 get their fates. Then Sign out from the person's menu at 1440; the signed-out page; a reload; a typed `/accounts`; still out; back in by a fresh printed link. **At 420:** Sign out at the foot of the Accounts tab (`temper/docs/design/client/README.md`, 3.5, the `Accounts` row), the same reload and staying out, and the Accounts tab itself against the mock. Sign out goes last on its box.

**The two scroll bars.** A long chat at 1280x720 and at 1000x600. Press "Older" until two loads have landed, as far as that chat has them. At each, with the chat's end, then scrolled to the top: `documentElement.scrollHeight` and `scrollWidth` against the window's, and `body`'s likewise, and exactly one scrolling pane in view, by `overflow` and `scrollHeight` over `clientHeight`. A screenshot of each. Also with the draft pane open beside it (Split), if the width offers it. The document never exceeds its window.

**`chat-work`** (`temper/docs/design/client/README.md`, 3.5, *Reading back*, the call line): on a landing's chat ("Land a local hello status change."), the run of calls on one line; open it, shut it; the same on an agent's page. The growing answer: poll while a turn is being written, and capture it if the fake holds it long enough, else the gallery's state, named. "Latest ↓": scroll up in a long chat: it shows; new words don't move the view; press it, the end returns, it goes. "Loading earlier messages…" while Older reads, if you can catch it; a row says if not. Compared with the design's words. What `chat-reading` hasn't landed is `owed` (`notes/common.md`).

**`close-chat`** (`temper/docs/design/client/README.md`, 3.6, *The sheet*): each row of its table, at both widths: title, what it says, the keep act first, the act second and primary, Escape and the keep act changing nothing, the act doing its work. Closing a chat; Start over; a goal's Stop; a task's Cancel; an agent's task's Stop; accepting a proposal; rejecting one; releasing a task, with an uncertain effect's two choices. A box's real state where one is reachable (proposal: the W3 cue; a hold: `temper ctl` stop), and the gallery otherwise, named on the row.

**`sign-in-new-tab` and `model-check`:** the looks as landed (`temper/docs/design/client/README.md`, 3.5, *Signing in*, *The try*). The Model step's states on a fresh box, as far as the fake reaches them, at both widths; the gallery's for the rest (`grep -E 'sign|check|try' temper/tools/gallery/names.txt`). The behaviour still owed is `owed`, each naming its status file's section.

**Temper's own way** (`temper/docs/design/client/README.md`, 3.5): Configure temper's card and "Not sure yet?"; See how it works, the reference beside the page, and its modes; Start from it, what it sends, what the fake answers (a fake's answer is not a finding); then the same on a fresh project made through the switcher.

Every box and Chromium you started is stopped; the screenshots and `index.html` are on the builder.

## Inputs

- `temper/docs/plan/ui-walk-3/inputs/request.md`, all of it.
- `temper/docs/plan/ui-walk-3/notes/common.md`, `notes/walk.md`, `notes/decisions.md`.
- `temper/docs/plan/ui-walk-3/driver/README.md`, `walk.py` (`needs`, near "Use the small implementation", holds A4's steps), `focus.py`, `cdp.py`.
- `temper/docs/plan/ui-walk-3/walk-2/findings.md`: A2, A3, A4.
- The design's sections named above, and only those. `temper/docs/design/draft/chat-reading.status.md`, `sign-in-new-tab.status.md`, `model-check.status.md`: what's owed.
- `temper/tests/client-browser/tests/browser_all/browser_pages.rs`, `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back`: how a chat is made long enough to read back.

## Owns and may change

Owns, as `files` lists them. **`walk.py`, `cdp.py`, `README.md` and `temper-fake` are `walk`'s, running beside you:** import from `walk.py` as your base has it, and copy into `additions.py` what you'd have to change. No fake script here: a state that needs one is `unreachable`, shown by the gallery, and a row.

Options, each pre-approved, the first named being the default:

- **A4's repeats:** a fresh box for each, since the script's note holds on a box's first goal. Share the build; run one box at a time.
- **A long chat:** the composer, as a person would, with messages long enough to fill the pane. If "Older" needs more than a walk can send in reasonable time, send the history by `temper ctl` and press the pane's controls in the browser; the row says so. If no chat this box can make has two loads of Older, say how many it had, and name the fixture that would.
- **A fake box's Sign out differs from a real one's:** it's the fake's sign-in links you use; a difference that's the fake's is named, not a finding.
- **A stop on a fault:** as `walk`'s: a finding with its reproduction, and on to the next section. No section's failure leaves the others unwalked.

## Steps

1. Build and copy, by `driver/README.md`, into `~/src/fabrig/ui-walk-3/additions/build/`.
2. `additions.py`, section by section, A4 first since it's the one a person loses most by.
3. The tables, `index.html`.
4. `python3 -I -B tools/citations/citations.py --base origin/main`. Commit and push.

## Report

As `notes/common.md`, Your report, and: A4's score, the scroll numbers at both sizes, and the findings' ids.
