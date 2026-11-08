# look-outside-pages: the account's rows

The landing's account (`temper/docs/design/draft/look-requests.md`, 5): one row per
overlay rule, page change and state the request names, with the result the plan expects
and the task that lands it. The task `account` checks each against the trunk and writes
what stands.

## The overlay's rules (into `temper.css`, under `/* ==== ui: outside ==== */`)

| Item | Expected | Task |
|---|---|---|
| `:root:has(#app > .temper-top)` tokens, and its `body` | landed | outside-looks |
| `.temper-top` | landed | outside-looks |
| `.temper-top > .temper-brand`, `::before`, `::after` | landed | outside-looks |
| `.temper-top-end`, `> [role=status]`, `> button`, `:hover` | landed | outside-looks |
| `.temper-outside`, its `h1`, its `p` | landed | outside-looks |
| `.temper-outside-primary`, `-secondary`, their hovers, `.temper-outside > .temper-outside-primary` | landed | outside-looks |
| `.temper-outside-acts`, `> *` | landed | outside-looks |
| `.temper-outside .temper-outside-quiet`, its link and hover | landed | outside-looks |
| `.temper-outside :is(a, button):focus-visible` | landed | outside-looks |
| `.temper-outside form`, its label, input, focus, button, alert | landed | outside-looks |
| `.temper-outside form > .temper-approve-field` | landed | outside-looks |
| `.temper-outside .temper-approve-code`, `.temper-approve-asked` | landed | outside-looks |
| `@media (max-width: 760px)`: `.temper-top`, `.temper-outside` | landed | outside-looks |
| the welcome's banner rules select `.temper-top` | landed | outside-frame |
| `.temper-top-end > [data-region=temper-link]:not(:has(> .temper-offline))` | differs: a rule the overlay lacks; the top bar's tree keeps the link's state for the terminal's first line and the story worlds, the browser showing it while offline alone | outside-frame |
| `outside-pages/pages.css` | landed as `files/pages.css`, its fonts relative | server-pages |

## The pages

| Item | Expected | Task |
|---|---|---|
| 01.html: Sign in, Continue with Google filled, Read public projects quiet | landed | sign-in-column |
| 02.html: asking for a code, the field and Check | landed | approval-column |
| 03.html: the ask, the code large, Approve filled, Refuse outlined; the top bar signed in | landed | approval-column, outside-frame |
| 04.html: the terminal signed in | landed | approval-column |
| 05.html: signed out with Google, its `then` the page's address with its code | landed | approval-column |
| 06.html: Continuing to Forgejo… | landed | sign-in-column |
| 07.html: Signed out, its words under it | landed | outside-looks |
| 08.html: No project yet, the form "New project" | landed | outside-frame |
| 09.html: a Google sign-in's ending on `pages.css` | landed | server-pages |
| 10.html: the boot's mark | landed | boot-mark |

## The states and parts the message names in words

| Item | Expected | Task |
|---|---|---|
| The top bar wherever the rail isn't: mark, person, Sign out, Offline, Behind; no menu, switcher, navigation, Signed out, Connecting, Live | landed | outside-frame |
| The top bar while `me` is read: the mark alone | landed | outside-frame |
| The welcome's top bar, its line and steps as today | landed; differs: its brand goes to `/` as the request's frame has it | outside-frame |
| The public pages' top bar, "Sign in" at its right | landed | public-top |
| The column: heading, words, acts, the quiet link; a field and its act on one line | landed | outside-looks |
| The card's own "temper" gone | landed | sign-in-column, approval-column |
| A forge's sign-in: "Forgejo at HOST", "to continue to" | landed | sign-in-column |
| Every approval state of `client/README.md`'s table (typing, unread, reading, asking, deciding, approved, signed in, refused, expired, used, unknown, signed out, busy) | landed | approval-column |
| Deciding: "Approving…" or "Refusing…" in the acts' place | landed | approval-column |
| Signed out at the approval with no Google: today's words | landed | approval-column |
| Each `Authorize` state in the column | landed | sign-in-column |
| Temper unavailable: Signed out, Out of date, each link reason under "Signed out" | landed; differs: out of date says "Run the temper the engine runs." under its heading, and the terminal's signed-out column is its heading alone | outside-looks |
| No project yet, with the welcome: Setting temper up…, Set up temper, Try the playground | landed | outside-frame |
| No project yet replaces Chats' and NewChat's bodies and any other address's page | landed; differs: no accounts' banner above it | outside-frame |
| The GitHub App's setup pages: the same rules inline, hashes moved | landed; differs: no `@font-face` (their policy fetches nothing) | server-pages |
| `no-bundle.html` plain | landed | server-pages |
| Night follows the client's look; the server's pages and the boot have none | landed | outside-looks |
| `BANNER`, `MENU`, `CORNER`, `SIGN_IN`, `UNAVAILABLE`, `APPROVE_ACTS`, `APPROVE_PRIMARY` leave `look.rs`, their rules `temper.css` | landed | old-look-out |
| `files/brand.svg` goes | landed | old-look-out |
| New looks `TOP`, `OUTSIDE`, `OUTSIDE_PRIMARY`, `OUTSIDE_SECONDARY`, `OUTSIDE_ACTS`, `OUTSIDE_QUIET` | landed; differs: `TOP_END` and `APPROVE_FIELD` are new too (the overlay selects them), and all take the reserved slots 500 to 507 | outside-looks |
| Every class `temper.css` selects is one `look.rs` gives, and a focused test | landed | old-look-out |
| No source of the bundle or the server's pages names `#b8664e`, and a focused test | landed; differs: `--temper-accent` and the light terminal's `--accent` take the signal too | old-look-out |
| The view's tests: each place outside a project draws `TOP` and the column | landed | outside-frame, outside-looks, approval-column, sign-in-column, public-top |
| The approval's signed-out state with Google gives the Google link with its `then` | landed | approval-column |
| The Google endings link `pages.css` | landed | server-pages |
| The approval's codes, acts and `then` unchanged (the TUI's owner was told) | landed | approval-column |
| The terminal's prints keep their meaning: headings stay headings, the first line keeps the link's state | landed | outside-looks, outside-frame |
| The render check | follows landing, on main, outside the plan; owner temper/ui, through fab (`look-requests.md`, 4) | |
