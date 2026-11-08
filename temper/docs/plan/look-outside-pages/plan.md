# look-outside-pages: the plan

The look request `outside-pages` (`43a18a5042` on `origin/look-requests`), landed as
`temper/docs/design/draft/look-requests.md`, 3 and 5 say: every page outside a project
in the welcome's look, and the old look gone for good. Its rules for every task:
`inputs/request.md` and `notes/common.md`; the shapes that cross tasks:
`notes/contracts.md`.

## Parts and tasks

| Part | Task | Needs | What |
|---|---|---|---|
| server-pages | `server-pages` | | Google endings on the bundle's `files/pages.css`; GitHub App setup pages with its rules inline |
| boot | `boot-mark` | | `loading.html`'s mark: the star in `#c2410c`, no dot |

Further batches (being written): the client's top bar and column (`outside-frame`), the
approval page, the sign-in and authorization pages, no project yet, the public pages' top
bar, the old look's removal with its two focused tests, and the account.

## Decisions

- **New look tokens take the reserved slots 500 to 507** (`notes/contracts.md`, 1), not
  appended ones: `ui-ask-m3a`'s branches append at 608 and up, so appended numbers would
  meet theirs.
- **`files/pages.css`** is the bundle's stylesheet for the server's pages, served under
  the bundle's `files/` like its fonts, so no bundling step changes.
- **A stored format, protocol or public interface:** none changes so far.

## Critical path (so far)

`server-pages` and `boot-mark` start at once and need nothing.

## Overlaps with other drafts

- `temper/crates/temper-client-shell-web/` is also `ui-ask-m3a/path-draw`'s and
  `views-draw`'s (`temper.css`); this batch's files there (`files/pages.css`,
  `loading.html`) are new or untouched by them.
