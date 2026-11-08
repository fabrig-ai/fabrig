+++
id = "night-tokens"
part = "night"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-web/task-pages.css",
]
effort = "high"
minutes = 240
+++

# The night look: tokens first, a night set, the glow and the switch's rules

## Goal

The request's task 3 (`temper/docs/plan/look-round-19/inputs/request.md`), its stylesheet: the look request `night-mode` (`3c5333485a`). Tokens first: every colour of `temper.css` and `task-pages.css` comes from a custom property, the day's values exactly as the literals they replace, so that a day page draws as before; then one night set on `:root[data-night]`, after fabrig.ai's colours as the request's overlay draws them, the glow, the switch's rules, and what tokens can't carry (images, filters) under a night marker. The rest of the client takes the tokens as they fall, with no page left with words in the day's ink on the night's ground. Stylesheets alone: `night-keep` sets `html[data-night]`, `night-switch` draws the button.

Acceptance:

1. **One day set.** The `--gf-*` copies in the five `:root:has(..)` scopes (`.temper-welcome`, `#app > .temper-rail`, `#app > :is(.temper-session, .temper-chats-page)`, `#app > .temper-needs`, `#app > .temper-public-page`; their values are identical, checked by this cut) merge into the `:root` block at the file's top, beside the `--temper-*` set; the scoped blocks keep only what isn't a copy (the `--gf-sans`, `--gf-mono` fonts may move too). The aliases on `#app:has(..)` (`--temper-* : var(--gf-*)` on the welcome and the public page) and the chat terminal's `--doc-*` and palette stay where they stand. Before merging, `grep` every `var(--gf-` outside those five scopes' pages: a rule that read an undefined `--gf-*` before (and so fell back) and would now take a value is named in the report and given its old look back (its fallback written in), never left changed.
2. **Tokens first.** Every literal colour outside a token's definition, in both stylesheets (about 360 in `temper.css` and 35 in `task-pages.css` when cut: hex, `rgb`/`rgba`, `white`, `black`), becomes `var(--TOKEN)`:
   - the token of its part whose day value is the literal (a ground, an ink, a line, the hover, the signal, good, amber, red and their pales), by role first: `white` as a ground is `--gf-surface` or `--temper-paper` as its scope reads, `white` as words on a filled button is the fill's words token;
   - otherwise a new token named by its part, `--temper-` or `--gf-` as its neighbours are (`--gf-primary` and `--gf-primary-ink` for a primary's ground and words, `--gf-shadow` for the 24,24,27 shadows, `--temper-on-accent` for words on the accent, and so on), defined once in the `:root` day set with the literal's value, a near duplicate kept as its own token rather than rounded to a neighbour, so that the day is unchanged.
   Left literal, and named in the report: colours inside `url("data:image/svg+xml,..")` images; `transparent`, `currentColor`, `inherit`; the chat terminal's `--doc-*` and `#app > .temper-terminal` palettes, which keep their own dark and light at night (`browser.md`, 2.5, *What stays*); `loading.css`, which draws before the shell starts.
3. **The night set,** a block `:root[data-night] { color-scheme: dark; .. }` right after the `:root` day set at the file's top, outranking every day set (with acceptance 1 done, the scoped copies are gone; if the option below keeps them, the overlay's selector `html:root[data-night]:has(#app)` instead), holding:
   - the overlay's values for every `--gf-*` and `--temper-*` token it names (`git show 3c5333485a:temper/docs/design/client/journeys/overlay.css`, its first block), and `--gf-hover`, `--gf-signal-*`, `--gf-hint` as it has them;
   - a night value for each new token of acceptance 2, by its part, from the same palette: a ground the night's paper or surface, an ink the night's ink, ink-2, muted or faint, a line the night's lines, a shadow `rgba(0, 0, 0, .4)` or none; `--gf-primary` `#fd7a2a` and `--gf-primary-ink` `#0a0f22` (the overlay's "a primary is fabrig.ai's orange with dark words"), so that every selector the overlay lists under that comment takes them through the tokens rather than through rules of its own.
4. **The night's rules,** under a new marker `/* ==== ui: night ==== */` placed right after the night set (not at the file's end), each as the overlay has it, its selector prefixed `:root[data-night]`:
   - the glow: the overlay's `body` background (its grain image and radial gradients, as written) and `#app, #app > main { background: transparent; }`;
   - the send buttons' arrow at night (the overlay's `background-image` with `stroke='%230a0f22'`) and their disabled ground (`var(--gf-line-2)` with `files/send.svg`), where the arrow's image can't be a token;
   - the overlay's "dark line icons, turned light" (`filter: invert(.85)`) and its tick of a done step (`.temper-home-steps > li.temper-home-done::before`), where they are images or need the night's own pale.
   A rule of the overlay whose effect acceptance 3's tokens already give (a primary's ground and words, a literal grey become a line or the hover) is not repeated; the account says which token carries it.
5. **The switch's rules,** day and night, beside the rail's foot rules in the project's frame block (about line 1076 on), as the overlay has them: `#app > .temper-rail > .temper-night-switch` 30 px, absolute at `right: 16px; bottom: 24px`, its `::before` mask, the moon, and the sun on `[aria-pressed="true"]`, its hover; and in the frame's phone `@media (max-width: 760px)` block, `display: block; position: fixed; top: 18px; right: 14px; bottom: auto; z-index: 21`. The overlay's `#app > .temper-rail { position: sticky; }` already stands (line ~1076): no new rule, an account row.
6. **Unreadable at night:** read each page group's rules once with the night set in mind (the welcome and its sheets, the frame and Home, Chats and a chat, its document and Split, Needs you, Goals and a goal, Changes, Settings and its tabs, Accounts and the connection flows, the approval and sign-in pages, the public board, `task-pages.css`'s task pages, dialogs, toasts, menus): no rule may leave a literal or token that is dark by night on a night ground. The report lists any rule left so, with why.
7. **Status:** the **Not yet** entry under 2.5, *Day and night*, in `temper/docs/design/client/browser.status.md` naming `night-tokens` is deleted whole; no other design line.
8. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-client-shell-web` (its `tests/suite` read `temper.css`), `-p temper-client-view` (`tests/delegation.rs` reads it), `-p temper-client-gallery` and `-p temper-client-journeys` (their prints link it); `cargo check -p temper-client-browser-world --tests` (its reading fixture appends to the bundled sheet). No clippy: no Rust changes, unless a test's oracle is repaired.

One commit or more, each `Look-request: night-mode`; the last carries the account (below).

## Inputs

- `temper/docs/plan/look-round-19/notes/common.md`.
- `temper/docs/plan/look-round-19/inputs/request.md`, task 3.
- `temper/docs/design/client/browser.md`, 2.5, *Day and night* (this cut's) and *The project's frame*; section 5's *Night before the first paint*.
- The request: `git show 3c5333485a`, its message, `overlay.css` and the pages `night-mode/01.html` to `03.html` there (their `<html>` gains `data-night` by `night.js`), read as markup: no browser is started.
- `temper/crates/temper-client-shell-web/temper.css`: the `:root` set (lines 1 to 20), the five `:root:has(..)` scopes (about 912, 1067, 1473, 1981, 2645), the project's frame block (about 1066 to 1160).

## Owns and may change

Owns `temper.css` and `task-pages.css`: their colours, token sets, the night set, the night marker's rules and the switch's rules. Not the rule `two-steps` adds beside the tab bar's grid (about line 2213), and no rule's layout, size or font: colours and the night alone. Reader files, an oracle only: a test of `temper-client-shell-web/tests/suite/*.rs` or `temper-client-view/src/tests/delegation.rs` that compared a literal colour now a `var(..)` takes the token, keeping what else it checks, and is named in the report.

Options, each decided:

- **Merging the scoped copies would change a page** (acceptance 1's grep finds a rule outside the five scopes reading `--gf-*` with a fallback or none, on a page none of them covers): keep that page's look by writing its fallback in; if more than three such rules, keep the scoped copies instead, and write the night set with the overlay's selector `html:root[data-night]:has(#app)`, saying so.
- **A literal used both as words and as a ground** (`white`, `#fff`): one token per role, even with one day value.
- **The overlay's per-selector primary rules** name selectors that the tokens don't reach (a primary drawn with another token than the ones acceptance 2 gave it): add that rule under the night marker as the overlay has it, and say which.
- **A colour in `task-pages.css` with no night meaning** (a task page's own palette): token it the same; its night value from the palette by role.

## Steps

1. Read the overlay whole and the request's message; list the overlay's rules (they are the account's rows).
2. Acceptance 1: merge the day set, with its grep.
3. Acceptance 2, section by section (the base and welcome, the frame, each marker's section, `task-pages.css`), keeping a list of the new tokens.
4. Acceptances 3 to 5: the night set, the night marker's rules, the switch's rules.
5. Acceptance 6's reading; the status entry; the runs of acceptance 8; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`.
6. Commit with `Look-request: night-mode` and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least, one per overlay rule or group:

| Item | Result |
|---|---|
| `html:root[data-night] { color-scheme: dark; }` and the token block | landed as the night set (`:root[data-night]`), its values the overlay's |
| The glow: `body`'s grain and gradients, `#app, #app > main` transparent | landed under `ui: night` |
| `#app > .temper-rail { position: sticky; }` | stands already |
| The switch: size, place, moon, sun, hover, phone | landed in the frame block |
| A primary: the orange with dark words (each selector the overlay lists) | landed through `--gf-primary` and `--gf-primary-ink`, or a rule under `ui: night` where named |
| The send arrows and their disabled ground | landed under `ui: night` |
| The day's literal greys, as night tokens (each rule) | landed through the tokens named |
| The dark line icons, turned light; the done step's tick | landed under `ui: night` |
| Tokens first: N literals become tokens, M new tokens, the day unchanged | landed; the literals left (images, the terminal's palettes) named |
| The rest of the client at night | readable by acceptance 6's reading; what isn't, named |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4); compiling, prints and focused tests establish no rendered equality |

## Report

The implementer's report: the day set's merge and its grep, the new tokens with their day and night values, the literals left and why, acceptance 6's reading by page group, the options taken, any oracle repaired, the runs' counts, and the account as committed.
