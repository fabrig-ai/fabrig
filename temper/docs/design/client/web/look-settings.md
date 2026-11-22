# Settings and accounts' look

Settings and accounts' look is split from [the browser's look](look.md).

## 1. Settings and accounts

- **Settings' look** is the greenfield mock's `settings.html`, its
  `mock.css` rules taken exactly, values and all (`.col`, `.intro`, `.kicker`,
  `.headline`, `.lede`, `.tabs`, `.settings`, `.setting`, `.checks`, `.switch`,
  `.toggle-row`, `.section`, `.section-head`, `.lines`, `.when`, `.band`,
  `.quiet-band`, `.facts`, `.btn.small`, and their rules at 760 px and under),
  in `temper.css`, scoped under the project's frame with its `--gf-*` variables,
  as Home's are, on tokens of its own, `temper-settings*`. A disabled select is
  faint on no fill. Every select under Settings draws `files/chevron.svg` 14 px
  in from its border, with 36 px of padding at its right, by one rule no other
  scope in the stylesheet outweighs. A proposal of settings' card takes the conversation's
  `.proposal`, with `.proposal.superseded` at `opacity: .5` and `.proposal-state.ok`.
  The mock's `.proposal-split` is the review page's (`../pages/settings.md`, 2).
  `Forges` and `Accounts` take the same values on their rows (`../pages/README.md`,
  *The rows*): `.lines`' hairline and padding on a `details`, the chevron of
  `.switcher > summary::after` and, opened, `.setting`, `.btn` and its kinds and
  `.text`. A control a row shares with `ConnectForge` or the welcome takes them
  by a rule scoped under the row, so those pages keep their look. `Builders`
  adds `.empty`, and `Adopt` takes the `.col`, `.text` and `.btn`. No rule
  under the `--temper-*` tokens reaches these pages: the old rows' and pages'
  rules go, rather than being overridden. A flow's path takes `Ask`'s
  crumbs' values, its "‹" alone at 760 px and under.
- **An account's row** (`../pages/accounts-words.md`, 1) is the look request
  `accounts-usage`'s (2026-11-18), its overlay's rules in `temper.css`
  under Settings' rows, the rules they replace removed:
  - *Usage:* the head's `temper-usage` group, 10 px under the line, its
    windows 4 px apart; each `temper-usage-window` a grid of its name
    (`temper-usage-name`), 40 px, its bar, 120 px, its share
    (`temper-usage-used`), 36 px, right-aligned in ink, and its pace
    (`temper-usage-pace`), faint, 12 px apart, in 13 px on 1.6, muted, with
    tabular figures. The bar, `temper-usage-bar`, is skein's meter with its
    words hidden from sight: 4 px high, 2 px corners, `--gf-ink-2` to
    `--skein-ui-progress` on `--gf-line`; `--temper-amber` under
    `temper-usage-warned`, 80% or more; `--temper-red`, whole, under
    `temper-usage-limited`. "Full in", `temper-usage-fills`, is amber. At
    760 px and under, the grid is the name, the bar and the share, and the
    pace takes the row under the bar. The state is a token on the window,
    where the request's overlay reads a `data-state` no element of skein's
    carries.
  - *The check line and the uses:* `temper-account-check` and
    `temper-account-pays`, 13.5 px, muted.
  - *Quiet acts,* as `.proposal`'s and Remembered's secondary acts: every
    button and link in a row, the section head's "+ Connect a model" and
    "Sign out", no box and no border, 6 px corners, no fill, muted 13.5 px
    at weight 400, 32 px high on 5 px 10 px of padding; on hover the
    `--gf-hover` ground and ink; 44 px high at 760 px and under. A row's
    acts, `temper-account-acts`, are one wrapping line 10 px under its
    facts, 2 px apart, hung 10 px into the margin so that the first act's
    words align with the row's, as a source's first act is. "Remove
    account", `temper-account-remove`, is pushed to the line's end and red
    on hover. A move, `temper-account-move`, is 32 px wide, its arrow 15 px,
    "↑" for `temper-account-up` and "↓" for `temper-account-down`, drawn by
    the stylesheet over its name's words, which a named button shows and
    a reader reads. Forges' rows, sharing the rows' values, take the quiet
    acts too, by the overlay's selectors as they are.

## 2. Open questions

None.
