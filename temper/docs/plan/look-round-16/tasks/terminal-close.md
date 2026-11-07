+++
id = "terminal-close"
part = "terminal-close"
kind = "docs"
needs = ["terminal-browser", "terminal-draft", "terminal-cards"]
files = [
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
  "temper/docs/design/client/mockups/STATUS.md",
  "temper/docs/design/client/journeys/chat-draft/README.md",
  "temper/docs/design/STATUS.md",
]
effort = "medium"
minutes = 60
+++

# look-round-16's account, and the design in line

## Goal

Close the landing of the four look requests by `temper/docs/design/draft/look-requests.md`, 3, step 7, and 5: one commit per request, in the requests' order, each carrying its trailer and its account, a table of every rule, page change and state its message names; and the design's status in line with what landed. The design text itself was written by the cut (`temper/docs/design/client/README.md`, 3.5, *A chat's terminal*; `browser.md`, 2.5 and 7.2; `journeys.md`, 9); this task records where the code departs from it, if anywhere.

Acceptance:

1. Four commits, in this order, each with one table `| Item | Result |` in its message (`look-requests.md`, 5) and its own trailer: `chat-terminal`, `chat-terminal-font`, `chat-terminal-draft`, `chat-terminal-esc`. A row per rule of the request's `terminal.css` or `terminal.js` change, per page change and per state its message names: landed (where), differs and why, or deferred to whom and where it is tracked. Each table's last row is the render check's: it follows landing, owned by temper/ui, and compiling and print checks establish no rendered equality.
2. The `look-round-16` **Not yet** entries in `temper/docs/design/client/README.status.md` and `browser.status.md` are deleted where every part they name has landed; a part that departs becomes a **Departs** entry there, with its reason; a part left undone stays as a **Not yet** naming what is left. `temper/docs/design/STATUS.md`'s summary lines follow, if they name these files' gaps.
3. `mockups/STATUS.md`'s chat paragraph names the terminal as a browser's other look of a chat (one or two sentences, citing `../README.md`, 3.5, *A chat's terminal*); `journeys/chat-draft/README.md` names ">_" in the bar where it lists the bar's controls, if it does.
4. `python3 -I -B tools/citations/citations.py --base origin/main` reports none broken.

## Inputs

- `temper/docs/plan/look-round-16/notes/common.md`: the standing rules, the requests and how to read them.
- `temper/docs/design/draft/look-requests.md`, 3 (step 7 and the trailers) and 5.
- `temper/docs/design/client/README.md`, 3.5, *A chat's terminal*; `browser.md`, 2.5, *A chat's terminal*, and 7.2.
- The four requests, each by `git show` (its message, its `terminal.css` and `terminal.js` diffs, its pages' diffs), and what landed: `git log --grep='Look-request: chat-terminal' origin/look-round-16/integration` or the trunk your branch starts from, each commit's message, and the code it names.

## Owns and may change

Owns the header's files, and the four commits' messages. No code: a row whose rule did not land is written as it stands, deferred or departed, never repaired here.

Departs from the implementer's text: `docs/design/` is otherwise closed, but this task edits the header's status and STATUS files, `mockups/STATUS.md` and the journey's `README.md`, as a docs task.

Options, each decided:

- **The commits:** the status edits ride in the first and last of the four commits; the middle two may hold their account alone (`git commit --allow-empty` where nothing else changes).
- **A rule landed on another selector:** "landed, as `SELECTOR` on temper's token (`browser.md`, 2.5)"; the request's `.tt-*` selectors name the page's own markup, which the view's tokens replace.
- **The states in words:** each a row: the phone's one line, the draft in the conversation's place, `/start-over` and `/goal`, "waiting on 1 delegate" on the running line, a failed tool's error, a closed chat's box, italic as a slanted regular face, text past the subset, a draft being written, a table in a draft, a flash on a changed section, the figure on a phone, Chromium's short and long Esc, Exit, Firefox's and Safari's one Esc, a page that isn't a secure context.
- **Not the look's**, as the requests name them: whether `/commands` reach the server (they don't: the page's own acts), what keeps a person in this mode between chats and devices (per browser, the kept store), which theme a person gets first (dark): each a row saying what was decided and where (`temper/docs/plan/look-round-16/plan.md`).

## Steps

1. Read each request whole, then what landed; draw each table.
2. Edit the status files and the two docs.
3. `python3 -I -B tools/citations/citations.py --base origin/main`.
4. Commit the four, in order, each with its trailer one line above the other trailers; push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report, and each row that is not "landed", with where it is tracked.
