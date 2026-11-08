# look-outside-pages: the shapes that cross tasks

The run's binding calls. A later batch adds sections; none changes.

## 1. The look's tokens

New tokens take the reserved slots 500 to 507 of `temper/crates/temper-client-view/src/look.rs`'s
`NAMES.styles` (each `temper-reserved-N` there becomes the name below), so that no
token number meets another plan's appended ones. The constants sit together, under a
doc comment citing `temper/docs/design/client/README.md, 5.1` and `browser.md, 2.5`.

| Constant | Token | Name | What it styles |
|---|---|---|---|
| `TOP` | 500 | `temper-top` | the top bar, the frame's `Role::Banner` outside a project |
| `TOP_END` | 501 | `temper-top-end` | the top bar's right: statuses, the person, "Sign out", "Sign in" |
| `OUTSIDE` | 502 | `temper-outside` | the column, the `Role::Main` outside a project |
| `OUTSIDE_PRIMARY` | 503 | `temper-outside-primary` | the first act, filled: a link or a button |
| `OUTSIDE_SECONDARY` | 504 | `temper-outside-secondary` | a further act, outlined |
| `OUTSIDE_ACTS` | 505 | `temper-outside-acts` | a slot of acts side by side |
| `OUTSIDE_QUIET` | 506 | `temper-outside-quiet` | the paragraph holding a quiet link after the acts |
| `APPROVE_FIELD` | 507 | `temper-approve-field` | the approval's code field |

The looks that leave (task `old-look-out`): `BANNER` 0, `CORNER` 21, `UNAVAILABLE` 68,
`MENU` 85, `SIGN_IN` 123, `APPROVE_ACTS` 606, `APPROVE_PRIMARY` 607; each slot's name
becomes `temper-reserved-N`, N its token. `CARD`, `MAIN`, `BRAND`, `APPROVE`,
`APPROVE_CODE` and `APPROVE_ASKED` stay.

## 2. The server's pages

- **The stylesheet:** `temper/crates/temper-client-shell-web/files/pages.css`, the
  request's `outside-pages/pages.css` with each font's `url(...)` made relative to the
  file, `url(fonts/ibm-plex-sans-latin-400-normal.woff2)` and so on. The bundle carries
  it under its `files/`; a page links it as `/bundle/DIGEST/files/pages.css`.
- **A Google sign-in's ending** (`temper/crates/temper-service/src/clients/google/pages.rs`),
  the bytes in this order, `SHEET` the stylesheet's address, and the `<link>` absent with
  no bundle served:

  ```html
  <!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover"><title>temper</title><link rel="stylesheet" href="SHEET"></head><body><header><a href="/">temper</a></header><main><h1>HEADING</h1><p>PARAGRAPH</p><a class="primary" href="START">Continue with Google</a><p class="quiet"><a href="THEN">Back to sign in</a></p></main></body></html>
  ```

  "Use another Google account" in place of "Continue with Google" where the ending
  chooses; the `quiet` paragraph absent where the ending has no "Back to sign in".
- **The GitHub App's setup pages** (`temper/crates/temper-service/src/setup/routes.rs`):
  the same `header` and column, `HEAD`'s inline `<style>` holding `pages.css`'s rules but
  its `@font-face` ones (and the manifest page's `.js body{display:none}` after them), and
  a filled `button.primary` drawn as the filled link is. The plain page:
  `<header><a href="/">temper</a></header><main><h1>HEADING</h1><p>WHY</p><a
  class="primary" href="BACK">Back to temper</a></main>`. The manifest page: `<h1>temper’s
  App, on GitHub</h1>`, its form's `<button type="submit" class="primary">Continue to
  GitHub</button>`, then `<p class="quiet"><a href="BACK">Back to temper</a></p>`.
  `PAGE_STYLE_HASH` and `MANIFEST_STYLE_HASH` are the new styles' hashes.
- **The boot's mark** (`temper/crates/temper-client-shell-web/loading.html`): the request's
  `outside-pages/10.html` mark, `<g fill="#c2410c" transform="scale(11.4)">` holding the
  star's path alone, no `<circle>`; the wordmark's path unchanged.
