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

## 3. The column (`temper/crates/temper-client-view/src/outside.rs`)

A module of its own, `//! The frame and the column outside a project
(temper/docs/design/client/README.md, 5.1; temper/docs/design/client/browser.md, 2.5).`,
made by the task `outside-looks`. Its helpers, `pub(crate)`:

```rust
/// Opens the column's main, `look::OUTSIDE`, then `extra` styles in order, and writes its
/// level-1 heading, named and worded `heading`; the caller draws the rest and closes the main.
pub(crate) fn column(b: &mut Builder, heading: &[u8], extra: &[skein_ui::Style]);
```

```rust
/// One paragraph of the column's words.
pub(crate) fn words(b: &mut Builder, words: &crate::words::Words, word: crate::words::Word, holes: &[skein_ui::phrases::Hole]);
```

Later tasks use these rather than writing their own; an act takes its token in place
(`b.style(look::OUTSIDE_PRIMARY)`). Every column is this tree, in this order:

```
Main [OUTSIDE, extra…]
  Heading { level: 1 }, named its words
  Paragraph…                          its words (the approval's code [APPROVE_CODE], asked time and warning [APPROVE_ASKED])
  then its acts, one of:
    Link or Button [OUTSIDE_PRIMARY]                         a lone act, directly in the main
    Slot [OUTSIDE_ACTS] { [OUTSIDE_PRIMARY], [OUTSIDE_SECONDARY]… }   acts side by side, the first filled
    Slot [OUTSIDE_ACTS] { Status }                           a state standing in the acts' place ("Approving…")
    Form, named { TextField, named (its name drawn above it); Button [OUTSIDE_PRIMARY]; Alert? }
  Paragraph [OUTSIDE_QUIET] { Link }  a quiet link after the acts
```

No `MAIN`, `SIGN_IN`, `CARD` or `UNAVAILABLE` on a column, and no brand paragraph in it:
the top bar carries the mark. A form's alert is inside the form, after its button.

## 4. The top bar (`outside::top`, task `outside-frame`)

`root::build` draws it wherever it drew `frame::banner`. In both shells:

```
Banner [TOP]
  Link "temper" [BRAND] → PageAct::Go { address: Address::Inbox { filter: Filter::ALL } }   (printed `/`)
  on Welcome, the deployment read and its welcome on, the link usable:
    temper box's line and the welcome's steps, as `frame::banner` draws them today
  otherwise, a Slot [TOP_END]:
    Region link_region() { Status WORD [OFFLINE | LIVE | CONNECTING | none] }
        in both shells and every state, the link's state's word and token as `frame::banner`'s
        corner says them today ("Signed out", "Out of date", "Connecting", "Live" once `me`
        is read, "Offline"), so that the terminal's first line and the story worlds that read
        the link region (`temper/tests/client/src/app.rs`, `Temper::seen`) keep it; the
        browser hides the region but while it holds `OFFLINE`, by one rule under the
        outside marker:
        `.temper-top-end > [data-region=temper-link]:not(:has(> .temper-offline)) { display: none; }`
    Status "Behind" [BEHIND]          while a page's watch reopens: in the region in the terminal,
                                      after it in the browser, as `frame::rail`'s foot places it
    Status NAME                       once `me` is read
    Alert "Sign out did not reach the engine."   while `sign_out_lost`
    Button "Sign out" → PageAct::SignOut          once `me` is read, or while `sign_out_lost`
```

Nothing else: no `Disclosure` "Menu", no switcher, no navigation, no "Set up temper".
`frame::navigation`, `Place::Banner` and what only the banner used go with it.

The public pages' frame (`public.rs`, task `public-top`) is the same `Banner [TOP]`, its
brand link as today, and, where it offers one, `Slot [TOP_END] { Link "Sign in" }`.

## 5. Words (`temper/crates/temper-client-view/src/words.rs`)

| Word | English | Task |
|---|---|---|
| `SignedOutMain` | browser: "To sign in, open a sign-in link temper printed."; terminal: "Signed out." (unchanged: the TUI's) | `outside-looks` |
| `OutOfDateHow` (new) | "Run the temper the engine runs." | `outside-looks` |
| `OutOfDateMain` | unchanged (the TUI's) | |
| `NoProjectLine` | "Make your first one, or ask an owner to add you to theirs." | `outside-frame` |
| `ProjectName` (new) | "Project name" | `outside-frame` |
| `NoProjectSetUp` (new) | "Setting temper up makes your first project." | `outside-frame` |
| `ApproveSignedOutGoogle` (new) | "This browser isn’t signed in to temper. Sign in, and this page comes back with your terminal’s code." (U+2019 apostrophes, as its neighbours) | `approval-column` |

A new word sits beside its family's in the enum and the table. The heading of the
unavailable column is `Word::SignedOut` or `Word::OutOfDate`; its paragraph
`SignedOutMain` (browser), none (terminal), a link's `not_signed_in` paragraph as today,
or `OutOfDateHow`.
