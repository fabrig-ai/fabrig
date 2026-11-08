# look-outside-pages: the plan

The look request `outside-pages` (`43a18a5042` on `origin/look-requests`), landed as
`temper/docs/design/draft/look-requests.md`, 3 and 5 say: every page outside a project
in the welcome's look, and the old look gone for good. Its rules for every task:
`inputs/request.md` and `notes/common.md`; the shapes that cross tasks:
`notes/contracts.md`; the account's rows: `notes/account.md`.

## Parts and tasks

| Part | Task | Needs | What |
|---|---|---|---|
| server-pages | `server-pages` | | Google endings on the bundle's `files/pages.css`; GitHub App setup pages with its rules inline |
| boot | `boot-mark` | | `loading.html`'s mark: the star in `#c2410c`, no dot |
| outside-looks | `outside-looks` | | the eight tokens, the overlay's rules, `outside.rs`'s column; the pages not about the work in it |
| outside-frame | `outside-frame` | outside-looks | the top bar wherever the rail isn't; no project yet in the column; the welcome's frame; journeys reprinted |
| approval-column | `approval-column` | outside-looks | every approval state in the column; Google's way back when signed out |
| sign-in-column | `sign-in-column` | outside-looks | the sign-in page and every `Authorize` state in the column |
| public-top | `public-top` | outside-looks | the public pages' top bar, Sign in at its right |
| old-look-out | `old-look-out` | all of the above | the old looks, rules, `brand.svg` and `#b8664e` leave; the class test and the old mark's test |
| account | `account` | old-look-out | status entries closed, mockups' and journeys' lines, the account |

## Decisions

- **New look tokens take the reserved slots 500 to 507** (`notes/contracts.md`, 1), not
  appended ones: `ui-ask-m3a`'s branches append at 608 and up, so appended numbers would
  meet theirs. `TOP_END` and `APPROVE_FIELD` are new beside the request's six: its
  overlay selects them.
- **The top bar keeps the link region in its tree** in both shells, every state: the
  terminal's first line prints the link's state from it (`client/README.md`, 6.4), and the
  client world's `Temper::seen` and the slow tiers' drivers read "Live" there. The browser
  shows it only while offline, by one rule the overlay lacks (account: differs).
- **The brand goes to `/` everywhere outside a project,** the welcome included, as the
  request's frame says.
- **No project yet** is `me` read with no project open (`pages.rs`'s `project_changed`
  opens the first project once `me` names one), and no accounts' banner shows over it.
- **The unavailable column's words:** the browser's "To sign in, open a sign-in link temper
  printed." and "Run the temper the engine runs." under their headings; the terminal's
  signed-out column is its heading alone. `SignedOutMain`'s terminal words and
  `OutOfDateMain` stay as they are: the TUI reads them.
- **The old accent:** `--temper-accent` and the light terminal's `--accent` take the
  signal, `#c2410c`, so the focused test can hold that no source of the bundle names
  `#b8664e`.
- **`files/pages.css`** is the bundle's stylesheet for the server's pages, served under
  the bundle's `files/` like its fonts, so no bundling step changes; the setup pages carry
  its rules inline without fonts (their policy fetches nothing).
- **A stored format, protocol or public interface:** none changes. The approval page's
  codes, acts and `then` stay (the TUI's owner shares them).

## Critical path

`outside-looks` (70 min) → `outside-frame` (180 min) → `old-look-out` (90 min) →
`account` (45 min). `approval-column`, `sign-in-column` and `public-top` run beside
`outside-frame` once `outside-looks` lands; `server-pages` and `boot-mark` need nothing
(`boot-mark` has landed).

## Overlaps with other drafts

- `temper/crates/temper-client-view/src/look.rs`, `words.rs`, `temper.css` and the gallery
  are also `ui-ask-m3a/views-draw`'s: this plan's tokens sit in reserved slots, its rules
  under its own marker, its words beside their families, so the merges meet only where
  both append to a table.
- `temper/tests/client/**` is `walks-lower`'s and `ui-ask-m3a/ways-stories`'; only
  `outside-frame` and `outside-looks` may touch a story there, and only an assertion that
  reads the old frame.
- `temper/crates/temper-client-shell-web/`: this plan's `files/pages.css` and
  `loading.html` are untouched by the others; `temper.css` is shared as above.
