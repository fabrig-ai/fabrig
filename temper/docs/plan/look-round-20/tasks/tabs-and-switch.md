+++
id = "tabs-and-switch"
part = "frame"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-shell-web/temper.css",
]
effort = "high"
minutes = 75
+++

# Ask's three tabs on a phone, and the night switch at every width

## Goal

The request's tasks 2 and 3 (`temper/docs/plan/look-round-20/inputs/request.md`), one task since both turn on the chat layout's phone rule. On a phone (760 px and under) a chat's layout hides the rail and the tab bar whole (`#app:has(> .temper-session) > :is(.temper-rail, .temper-tabbar, .temper-accounts) { display: none; }`, about line 1793): so Ask temper's page as Start opens it loses the three tabs `to-begin-two-steps` gives it, and the night switch, the rail's last child, is a 0×0 box on every chat page. Beside a Split, the folded rail hides every child but the brand and the unfold (about line 1949), the switch among them, at every width. Give Ask's page its three tabs on a phone while To begin shows, and the switch a visible place in each layout, as `night-mode` places it: at the rail's foot, at the top bar's right on a phone. Stylesheet alone: the markup is as `look-round-19` drew it.

Acceptance, by `temper/docs/design/client/browser.md`, 2.5, *The project's frame* and *Day and night*, *The switch*:

1. **Ask's three tabs on a phone:** the chat layout's phone rule hides the tab bar only while it holds a fourth link (`.temper-tabbar:has(> a:nth-child(4))`, the idiom of the three-column rule at about line 2509), so the tab bar of Ask temper's page while To begin shows, "Home", "Ask" and "Settings", stands fixed at the foot as on Home, three equal columns by the existing rule. The page leaves it room: the composer and everything above it stand wholly over the tabs (the session's height, or `#app`'s bottom padding, less the tab bar's 64 px and `env(safe-area-inset-bottom)`), and nothing scrolls under them. Every other chat page on a phone draws no tab bar, as today. The `.temper-accounts` part of the rule is unchanged.
2. **The switch on a phone's chat layout:** the rail draws no box of its own there (`display: contents`) and none of its children but the switch; the switch keeps its phone rule's look (30 px, fixed, `z-index: 21`), placed at the right of the chat's bar (`.temper-session-top`), at its right padding and centred on its height; the bar's right padding grows by the switch's width and an 8 px gap, so that nothing of the bar (its "‹", its path, `.temper-session-right`'s buttons) sits under it. The rail's other children stay hidden on that layout whatever later rule would show them: the Split's unfold and brand (`#app:has(> .temper-mode-split) > .temper-rail > ...`, later in the file at equal or higher specificity) and the folded rail's open state among them.
3. **The switch in the folded rail beside a Split:** the rule hiding the folded rail's children (about line 1949) leaves the switch out of its `:not(..)`, and the switch stands centred in the 56 px column, 24 px over its foot; while the folded rail is open (about lines 2007 to 2009, the 240 px panel), at its foot's right as on any rail (16 px in, 24 px over), and the open rule's `display: flex` leaves the switch's own `display` alone.
4. **Everywhere else unchanged:** the switch on a wide rail and on a phone's top bar outside a chat's layout keeps today's rules (about lines 1408 to 1424); no rule's colour changes (`night-shadows` owns the night set).
5. **Where the rules go:** changed rules where they stand; new rules under `/* ==== ui: phone ==== */`'s `@media (max-width: 760px)` block (about line 3084), which comes after the Split's rules and so outranks them at equal specificity, never at the file's end; a one-line comment over them in the file's manner.
6. **Status:** delete the **Not yet** entries naming `tabs-and-switch` in `temper/docs/design/client/README.status.md` and `browser.status.md`. No other design line.
7. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-client-shell-web` (its `tests/suite` read `temper.css`) and `-p temper-client-gallery` (its print links it). No clippy: no Rust changes, unless a test's oracle is repaired.

One commit or more, each with both trailers, `Look-request: to-begin-two-steps` and `Look-request: night-mode`; the last carries the account (below).

## Inputs

- `temper/docs/plan/look-round-20/notes/common.md`.
- `temper/docs/plan/look-round-20/inputs/request.md`, tasks 2 and 3.
- `temper/docs/design/client/browser.md`, 2.5, *The project's frame* (the tab bar while To begin shows) and *Day and night*, *The switch*.
- `temper/docs/design/client/README.md`, 3.5, *One layout for every chat* and *Ask temper's page* (its composer on a phone); 5.3, the night switch and the tab bar on a phone.
- The requests: `git show 4332a26265` (the tab bar's three tabs, its overlay's rule) and `git show 3c5333485a` (the switch's place, its overlay's switch rules and `night-mode/02.html`, `03.html`), read as markup: no browser is started.
- `temper/crates/temper-client-shell-web/temper.css`: the project's frame block (about lines 1360 to 1438: the rail, the switch, the phone's top bar and tab bar), the chat layout (about lines 1777 to 1800, 1837 to 1840), the Split's folded rail (about lines 1946 to 1967, 2007 to 2009), `ui: phone` (about line 3084).

## Owns and may change

Owns the rules named above in `temper.css`. Not the night set at the file's top (`night-shadows`'s) and no other rule's colour. Reader files, an oracle only: a test of `temper-client-shell-web/tests/suite/*.rs` that reads a rule this task changes (`phone_session_hides_the_keyboard_hint` reads the phone blocks) keeps what it checks, its text adjusted to the rule's new form, and is named in the report.

Options, each decided:

- **The session's foot can't leave the tab bar room by one rule** (the chat layout sizes `#app` at `100dvh`, about line 1837): size `#app`'s grid row or the session's height for that one page, keyed on the three-tab bar (`#app:has(> .temper-session):has(> .temper-tabbar:not(:has(> a:nth-child(4))))`), and say which.
- **`display: contents` on the rail would place the switch or a hidden child in `#app`'s grid:** the switch is `position: fixed` and so takes no cell, and the rest are `display: none`; if a cell or gap still shows, give the rail a zero box instead (`position: fixed; width: 0; height: 0; padding: 0; border: 0; overflow: visible`), and say so.
- **The chat's bar has no room left at 320 px** once the switch's padding is taken: let the path's title truncate as it does (`text-overflow: ellipsis`); never hide the switch.

## Steps

1. Read the requests' overlay rules and pages, and the rules of Inputs; list each rule you change (they are the account's rows).
2. Acceptance 1, then 2 and 3, then a reading of 4: every page that draws the rail, wide, at 1000 px and at a phone's width, on paper (no browser): Home, Ask temper's page while To begin shows and after, a chat, a chat in Split, Split's open folded rail, `NewChat`.
3. The status entries (acceptance 6); the runs of acceptance 7; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`.
4. Commit with both trailers and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| Ask temper's page as Start opens it, 420 px: three equal tabs, Home, Ask and Settings | landed (the chat layout's phone rule; the three-column rule) |
| The composer over the tabs on that page | landed (the rule named) |
| The switch at 420 on Ask temper's page and every chat: at the chat bar's right | landed (`ui: phone`) |
| The switch in the folded rail beside a Split, every width; and while it is open | landed (the folded rail's rules) |
| The switch on a wide rail and a phone's top bar elsewhere | stands as `look-round-19` landed it |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4): compiling and focused tests establish no rendered equality |

## Report

The implementer's report: each rule changed or added with its selector and place, the room left for the tabs and the bar, the reading of acceptance 4 by page and width, the options taken, any oracle repaired, the runs' counts, and the account as committed.

## Rebase (2026-11-08, step merge, foreman look-round-20/tabs-and-switch/foreman-1)

resolve browser.status.md by preserving both tasks’ completed-gap deletions
The merge conflicts in `temper/docs/design/client/browser.status.md`.
Integration commit `cf6f1ddaf9` merged `54abccf18f` (“Keep each shadow's day colour at night”), deleting the `night-shadows` Not yet entry.
Task commit `9d01503711` deletes the adjacent `tabs-and-switch` Not yet entry.
Resolve the conflict with both entries absent; retain the following `begin-sections` entry and all other status text.
Preserve both branches’ CSS changes: the night-shadow override removals and this task’s tabs and switch rules. The log reports CSS auto-merged without conflict.
Keep this task’s `README.status.md` deletion of the `tabs-and-switch` entry.

## Rebase (2026-11-08, step merge, foreman look-round-20/tabs-and-switch/foreman-2)

resolve both status conflicts by preserving tabs-and-switch and begin-sections deletions
Rebase onto integration, where `75e10af8a5` merged `80f50a072c` (“Keep Home sections out while To begin shows”).
The conflicts are in `temper/docs/design/client/README.status.md` and `temper/docs/design/client/browser.status.md`.
In README.status.md, leave both tasks’ adjacent **Not yet** entries absent: `begin-sections` and `tabs-and-switch`.
In browser.status.md, leave the `tabs-and-switch`, `begin-sections` and already-completed `night-shadows` entries absent.
Preserve integration’s removal of “The nonempty Home sections’ suppression gap is `README.status.md`, 3.5’s.”
This supersedes the previous Rebase instruction to retain `begin-sections`: integration has now completed it.
Retain all other status text, this task’s CSS changes and integration’s completed changes.
