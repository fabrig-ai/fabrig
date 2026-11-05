+++
id = "rail-menu"
part = "rail-menu"
kind = "feature"
needs = ["chats-drafts"]
files = [
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/rail.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 70
+++

# The rail's "+ New chat" gets its menu

## Goal

The look request `new-menu`, landed whole: every step of `temper/docs/design/draft/look-requests.md`, sections 3 to 5, and its account. Every commit carries `Look-request: new-menu` directly above `Co-Authored-By:`, and the last holds the account's table.

Acceptance, temper's `client/README.md`, 5.3, *The rail*:

- `a.temper-rail-new` becomes `div.temper-rail-new`, one dashed box split in two: the link "+ New chat", named "New chat", the box's padding moved onto it; `Role::Menu`, named "More ways to start work", its name hidden by style and its chevron `content: '⌄'`, 36 pixels, behind a dashed line.
- **Its items,** `Role::MenuItem`, a name and a description each (`skein-keys`): "Chat", "Ask a question or ask for a change.", to `NewChat`; "Goal", "Agents plan a larger change and land it.", to `NewChat { draft: true }`, while `drafts_offered`; after a rule, "All ways to start work", muted, `RAIL_NEW_ALL`, to `Settings { project }`, or `HowWorkRuns` if your base gives it an address.
- **The rules,** `new-menu`'s overlay, selectors and declarations, under the marker; the replaced ones removed. Each half takes the hover's grey; open, the button keeps it and its chevron turns ink; a focused item takes the hover's grey.
- **The phone** doesn't show the entry, as today.
- **Gallery:** `rail-new-menu`, opened by the sample's way.
- **The terminal:** the menu's button and its items print as a menu's do; `first_run` and any golden with the rail follow; each keeps its print's meaning.
- **Reprint** every journey whose pages change, and render as section 4 says: each page's request form from `git show 4194189e15` under its overlay against the reprint under the landed sheets, at 1440, 1000 and 420, the menu forced open; identical, or each difference with its reason. Screenshots under `~/src/fabrig/ui-chat-draft/new-menu/`.
- **The account:** a table in the last commit's message, one row per overlay rule, page change and state the message names: landed, or differing and why, or deferred and to whom. Deferred is `template-places`' choice of items, already its draft's status.
- `README.status.md`: 5.3's words on the rail's menu go. `temper/docs/design/client/mockups/STATUS.md` and `client/journeys.md`, 9, where they name this, per section 3's step 8; the request's own `new-menu/` pages and README aren't landed (`journeys.md`, 7).

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chat-draft/inputs/request.md`: *The client*, 9.
- `git show 4194189e15`, whole: its message, `overlay.css`, `new-menu/01.html`.
- `temper/docs/design/draft/look-requests.md`, sections 3, 4 and 5.
- `skein/docs/design/ui-web.md`, section 6's menu and section 7.

## Owns and may change

Owns, as `files` lists them. May also change `temper/crates/temper-client-domain/src/**` for nothing but a `drafts_offered` read; `temper/crates/temper-client-shell-terminal/src/tests/**`, regenerated; `temper/docs/design/client/journeys/**`, prints only, the request's own pages never; `temper/tests/client-browser/**`, what names `a.temper-rail-new` or the link "New chat"; `temper/tests/client/**` the same.

Options, each pre-approved:

- **The menu's button:** skein's `Role::Menu` makes its button; `#app > .temper-rail > .temper-rail-new > [aria-haspopup]` selects it.
- **`ui-settings`' `HowWorkRuns`:** if your base gives it an address, the item goes there, "at its Ways to start work" if a fragment exists, and otherwise its page; else `Settings { project }`.
- **A menu item's focus and Escape:** skein's, as they are.
- **A difference in `[role=menu]`'s children from the hand-made page,** where skein's mapping prints what it prints: the report's account says why.

## Steps

1. Tokens, the rules, `frame.rs`.
2. **Tests,** `tests/rail.rs`: `the_rails_new_chat_is_a_link_and_a_menu`, `its_items_go_where_they_say`, `goal_shows_while_drafts_are_offered`, `all_ways_goes_to_settings`, each having bitten; update those naming `RAIL_NEW` as a link.
3. `rg` the browser tier's and the world's tests for "New chat"; change what you make stale; run what's cheap.
4. `heavy cargo nextest run` of the client's packages; goldens; reprint; the render check.
5. **Check,** the implementer's text; the account; push.

## Report

As `notes/common.md`, Your report, with the account's table and the builder's name.
