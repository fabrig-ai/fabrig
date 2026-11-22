# The browser's look

## 1. The style's foundations

- **temper's style tokens,** under the owner's prefix `temper-`, on the
  nodes the mockups style: the frame's top bar or rail, navigation and main; the
  cards; the conversation's turns; the composers. The view sets them
  (`../view.md`, 1); the terminal prints none.
- **The first paint, the placeholder's:** the shell's `loading.html` and
  `loading.css`, given to skein's bundler (skein's `ui/web/bundle.md`, 3), so
  that a person on a slow connection has something to read in a few
  kilobytes. The mark as inline paths, so that no font is fetched and
  nothing swaps: the welcome's, the four-point star in the signal,
  `#c2410c`, with no dot, then the wordmark in the ink (the look request
  `outside-pages`), and no night,
  centred at about 38% of the height; under it a 240 px
  `progress`, as `temper-flow-meter` draws one, 4 px, ink over the
  hairline, and one muted line; a phone keeps the 16 px gutter. The lines,
  each in `loading.html`'s data attributes:
  - for the first 250 ms, the paper alone, so a cached load never flashes;
  - downloading: "2.8 of 11.5 MB" (what travels, so a gzipped module's
    total is its compressed size), and "· about 35 s left" once 3 s of data
    gives an estimate over 5 s;
  - after 8 s, under it, "Your browser keeps this, so next time it opens
    at once.";
  - no byte for 15 s: "Waiting for the server…" in the figure's place;
  - every byte in: "Starting…";
  - failed: no bar, "temper didn't load", why ("The connection dropped at
    4.8 of 11.5 MB.", "The server answered 404.", "The server didn't
    answer.", "This browser has no WebAssembly.", "It didn't start."), and
    "Try again", which reloads;
  - scripts off: no bar, and `--noscript`'s words.

  Handing over, a 200 ms fade once the client's first node and the
  stylesheets are in; reduced motion, no fade and no moving bar. The tab's
  title is "temper" throughout. The drawing is the look request
  `bundle-loading`'s; a terminal has no bundle and no placeholder.
- **The bundle's stylesheets:** skein's `skein-ui.css`, then
  `temper.css`, temper's look. It takes no rules from the
  mockups' terminal sheets (`terminal-chat.css` and `terminal-pages.css`
  over `style.css`): it is tuned on the journeys' prints
  (`journeys.md`, section 7), and the mockups are reconciled to it
  (`mockups/README.md`, 1); they are not shipped. What follows here
  says what it draws, and is the gallery's for the states no journey reaches (`gallery.md`).
  These are the bundle's three
  stylesheets; `jig.css` is not in it. `temper.css` holds the mockups'
  palette as variables under the prefix, their monospaced stack, compact
  top navigation and quiet borders. DejaVu Sans Bold, for the wordmark
  alone, as the mockups draw it, is self-hosted in the bundle's files,
  subset to Latin, with its licence beside it; the page fetches nothing
  from another host. The mockups' icons are decorative background
  images drawn by `temper.css` from the bundle's SVG files, each in its
  own colour (the rail's unfold ☰ its summary's, the send arrow white;
  temper's mark is the "✦" in the signal, a character, not an image):
  never masks, which a browser fetches under
  CORS and so refuses to a page opened as a file, as the journeys'
  prints are (`journeys.md`, 5.4).
  Every control keeps its words as its name. Nothing a person must know
  is said by colour or icon alone (skein's `ui/README.md`, section 8): a status
  is its word, and its colour only repeats it.
- **The main's scale:** the page's main is drawn at 14/12 of the
  mockups' sheets' lengths and of this section's, as the mockups' own
  zoom draws theirs: 12 px text reads as 14 px, and its spacing and
  widths grow with it. `temper.css` and `task-pages.css` write each
  length of a rule for a part inside main at that scale, to two
  decimals, rather than zoom it (the user, 2026-10-26). What a zoom
  would leave stays: hairlines, the media queries' widths, and the
  top bar with its mark, at the mockups' own. What a zoom scales
  but no length of temper's names is written out at the scale too:
  main's own text size, which its parts inherit with their `em`
  spacing; a native control's own size; skein's sheet's lengths inside
  main (its focus ring, its visually hidden boxes); and a viewport
  length, which a zoom grows like any other. Dialogs are drawn inside
  main, and scale with it. Confirmation sheets stand under `#app` at their
  own scale (`../README.md`, 3.6).
## 2. Confirmation sheets

- **Confirmation sheets:** `dialog.temper-confirm`, 460 px wide at most,
  with 24 px by 26 px padding, 14 px corners and a native `::backdrop`.
  The title is 19 px; the lead 14.5 px; impacts 14 px under a hairline.
  Keep precedes the named primary act in a separate group. An uncertain
  release's choices have a wrapping group of their own, with the same
  secondary buttons; those and the acts keep 44 px phone targets.
  Progress and refusal stay above the acts. A chat's end line, whichever
  its words (`../pages/README.md`), is 12.5 px between two hairlines, and the
  bar has no clock.
## 3. Connection flows

- **Connection flows** (`../pages/README.md` and `../view.md`, 2): Settings and the
  welcome share cards, panels and controls. Settings has no steps bar,
  outer Choose card or Ready card; the welcome retains its frame’s Steps.
  `CONNECT` adds the `temper-connect` class to Settings’ column, with zero
  gap and 28 px below its path (18 px on a phone). `CONNECT_CARDS` is a
  two-column grid with 16 px gaps; `CONNECT_CARD` has 22 px padding, 12 px
  corners and the surface shadow. `CONNECT_DEFAULT` outlines the first
  unconnected way in ink; `CONNECT_YES` marks connected ways in green.
  `CONNECT_ACTS` wraps with 8 px by 16 px gaps; `CONNECT_PRIMARY` is ink
  with white words, `CONNECT_SECONDARY` neutral, `CONNECT_QUIET` a muted
  underlined link. `CONNECT_MUTED` is supporting text. `CONNECT_MORE`,
  `CONNECT_ROWS` and `CONNECT_ROW` hold the API key and other-server rows.
  `CONNECT_PANEL` keeps the same padding and corners with an 18 px gap;
  `CONNECT_PAIR` groups half-width fields, and `CONNECT_ONE` makes one
  column. Below 760 px pairs and cards stack. `CONNECT_TITLE` marks a
  panel’s small heading; `CONNECT_CMD` holds a wrapping command with Copy;
  `CONNECT_WAITING` is its two-column waiting note. `CONNECT_HOW` holds
  labelled radio choices and `CONNECT_STEPS` numbered configuration steps.
  `CONNECT_REPOS` holds the welcome’s repository choices. `CONNECT_LIVE`
  draws the compact Forges progress list, with `CONNECT_LATER` for pending
  lines; state words remain in the terminal and accessible DOM. Failed
  lines retain their refusal. `CONNECT_EMPTY` draws an empty account group,
  and `CONNECT_CONFIRM` the success line above Accounts or Forges rows.
- **The repository sheet:** `CONNECT_SHEET` is at most 640 px wide, with
  14 px corners, a native backdrop and a fixed top at 6 vh. Its
  `CONNECT_SHEET_HEAD`, `CONNECT_SHEET_BODY` and `CONNECT_SHEET_FOOT` keep
  the source switch at the top while the foot follows the content; the
  body scrolls within its height. `CONNECT_SOURCES` holds the source radios,
  `CONNECT_SOURCE` the chosen source. `CONNECT_LIST` is the bordered list;
  `CONNECT_BOUNDED` gives long lists their own height, clamped from 150 to
  288 px against the viewport. `CONNECT_NONE` collapses an empty list;
  `CONNECT_COUNT` reports read and picked counts. `CONNECT_SAYS` mutes the
  More summary’s current choices. `CONNECT_ADDRESSES` holds one copy address
  per line; `CONNECT_ALERTS` its per-line refusals; `CONNECT_FAILED` the
  failed-copy row and its acts. Disabled primary acts fade and cannot send.
  At 760 px and below the sheet meets the bottom, full width, at most
  92 vh high, with 16 px side padding and a shorter bounded list.
## 4. The welcome

Moved to [`look-welcome.md`](look-welcome.md), section 1.

## 5. Task pages

- **The task pages' stylesheet,** `task-pages.css`, comes third, after
  `temper.css`, whose variables and rules it builds on: what an agent's
  task's and a change's pages add to the look, as `task.html` draws it in
  the terminal sheets (`../pages/task.md`, 2 and 3): a plan row's spend as a
  fourth, right-aligned cell, a row's dependency line in small muted words
  under its name, the list's name shown before its items. A goal's page takes
  nothing from it. `scripts/bundle.sh` passes the three to `skein-ui-bundle` in
  that order.
## 6. Settings and accounts

Moved to [`look-settings.md`](look-settings.md), section 1.

## 7. The project's frame

- **The project's frame** is the greenfield mock's `home.html`, in
  the welcome's paper, ink and IBM Plex from temper's files
  (`temper.css`, The project's frame): the rail a 232 px column at the
  left, sticky at the viewport's height, ruled from the page, whose
  rules take every page but the welcome and those of the banner; the
  page's parts, the main and any banner of a page's, in the other
  column. On a phone, at 760 px and under, the column is one, the rail
  a top bar of the switcher alone, the brand's star before the
  project's name, its menu at least 220 px wide, and the tab bar fixed
  at the foot, four links in four columns as wide as their words, Home,
  Needs, Work and Ask, each on one line at any phone's width, the Needs
  count included, and three while `me` offers no Ask temper (the look
  request `ask-temper-place`),
  the page leaving it room; while To begin shows, three equal tabs. Ask
  temper's page and an Ask temper chat, the Ask tab's, keep the tabs, where
  every other chat layout draws none, its composer and model picker standing
  wholly over them, the page's foot raised by the tabs' 64 px and the safe
  area (`../pages/README.md` and `../view.md`, 3). Needs you's open card takes the top bar
  with its own, `temper-needs-bar` (`../pages/README.md`, *Needs you*), as a
  chat's bar does, the night switch keeping its place. Over 760 px the
  link's state is one word with its dot at the right of the rail's
  first row, temper's name's, out of the foot's flow: "Live"; "Behind", its
  dot amber, in place of "Live" while a page's watch reopens; or
  "Connecting" or "Offline" alone, so that the foot's height and place
  never move with it (`../view.md`, 3; the look request `rail-word`). Its
  rules replace, where it stood, the one that hid the live link's region.
  The word is the foot's 13.5 px on temper's own 24 px line, so that the
  two baselines meet, and every state's dot, Live's too, is 7 px, as tall
  as the word's lowercase letters, sitting on the baseline (the look
  request `rail-word-line`, 2026-11-16). Its rule outweighs the foot's
  rule for the foot's items, wherever the two stand in the file, by one
  more attribute in its selector (`[role=status].temper-behind`), so that
  the foot's 6 by 10 px padding never reaches it; a phone's top bar keeps
  its dots as they are.
  The person's
  menu goes with the foot, so the `Accounts` tab's row with "Sign out", which
  `temper.css` hides over 760 px, stands at that width alone (`../pages/README.md` and `../view.md`, 3
  ).
- **The rail as the whole story draws it** (`../view.md`, 3; the mockups'
  `whole-story.css`, `.rail`, `.nav`, `.nav .sep`, `.new` and `.foot`, in the
  greenfield mock's paper and ink): the hairline after "Ask temper",
  `div.temper-rail-sep`, a rule of the rail's line colour 8 px over and
  under, inset as the entries are; the new box, "+ New draft or change" or
  "+ New change", the dashed box `div.temper-rail-new` keeps, one button or
  one link filling it, its menu the panel the box opened before; "Deployment"
  at the foot, a row as the foot's others, above the person's row, its
  dots after its word. The switcher's menu ends at "Rename PROJECT" where
  offered and "New project" on a wide page; at 760 px and under it adds
  "Team for PROJECT" where Team shows, "How work runs for PROJECT" and
  "Deployment", in that order. Each of those three links carries
  `temper-projects-phone`, hidden from 761 px under the switcher's acts;
  the terminal still prints them (`../view.md`, 3; the look request
  `switcher-links`). The rules go under `temper.css`'s marker `/* ==== ui:
  rail ==== */`, before the switcher's; the rules they replace go where they
  stand.
## 8. Work and deployment

- **Work's look** (`../pages/README.md`, *Work*; the mockup's `work.html` and
  `whole-story.css`): the head as Home's intro, its kicker, headline and lede,
  the landing's line in `small` under the lede; the filters, the mockup's
  `.chips`, a chip each, the current one `.chip.sig`; each section's head
  `.shead`, its title at the left and its quiet line at the right; each card
  the mockup's `.card`, its from line `.from` led by its tag, `.who.a` for a
  goal, `.who.c` for a change, `.who.p` for a chat and a draft, its
  headline `.q`, a goal's bar 6 px high on the hairline's grey, its done part
  ink, its rows `.rows`, its byline `.byline`; a landed row `.row`, its time
  mono. Its rules go under `temper.css`'s marker `/* ==== ui: work ==== */`,
  before Goals'. On a phone the column is one, the cards full width.
- **Deployment's frame** is How work runs' tabs' frame, `.col`, its head and
  its tab bar the same rules, its kicker "Deployment"; Members' and Limits'
  lines are Repositories' and Workers' `.setting` lines. Its rules go under
  `/* ==== ui: settings ==== */`.
## 9. Browser layout

- **A selector a browser accepts:** no rule of the shell's stylesheets
  nests `:has()` inside another `:has()`, which Chromium refuses, and the
  whole rule with it, without a word. What a rule asks of a child's own
  children it says through the relative selector's combinators, `#app:has(>
  .temper-tabbar):not(:has(> .temper-tabbar > a:nth-child(4)))`, as
  `:not(:has())` is accepted.
- **The page covers the display,** `viewport-fit=cover`, as the greenfield
  mock's does, wherever a page's head is printed: the bundler's page
  (skein's `ui/web/bundle.md`, 3), the playground's, the setup pages', and the
  gallery's and journeys' prints. So what sits at the foot keeps off a
  phone's home bar by `env(safe-area-inset-bottom)`, which is 0 on a display
  without one and so measures as the mock draws it, at 760 px and under: the
  tab bar's padding, a chat's foot, and the toast, 88 px over it, clear of the
  tabs.
- **What a phone presses,** at 760 px and under: a control scrolled into view
  stops clear of the tab bar, and every target stands at least 44 px tall to
  a press (`../../draft/surfaces.md`, section 10): the connection flows'
  primaries, Settings' and a dialog's, and every control of the frame, Home,
  Needs you, Work, Ask temper's page and an Ask temper chat; and any chat's
  bar and composer, which one layout gives every chat (`../pages/README.md`,
  *One layout for every chat*). A control the mockups draw smaller,
  the Away chip's 36 px pill, the night switch's 30 px circle, a way's tab,
  keeps its drawing and takes a 44 px target around it, by its padding or a
  transparent `::after` over it. A link inside a line of words is no target
  of its own. Every other control keeps what the mock draws.
- **No frame but the top bar outside a project** (`../view.md`, 1; the
  look request `outside-pages`): a page that is no project's has no
  navigation, switcher or menu, at any width; the top bar is drawn as
  *Outside a project* (below) says.
## 10. Chats and their documents

Moved to [`look-chats.md`](look-chats.md), section 1.

## 11. Ways of working

Moved to [`look-ways.md`](look-ways.md), section 1.

## 12. Flows and prints

- **A flow** (`skein/docs/design/ui/scenes.md`, 5) has the look of
  `skein/docs/design/draft/diagrams/preview.html`, in `temper.css` under
  `/* ==== ui: chats ==== */`: `--gf-paper` boxes on a `--gf-line-2` line, a strong
  box `--gf-ink` with a `--gf-surface` name, names `--gf-sans` 500 at 13 px and
  captions 400 at 12 px in `--gf-muted`, arrows `--gf-faint` at 1.5 with round ends. A
  flow's figure, which has no token, takes no margin, so that its first measure is
  its width in the page.
- **What the mock lacks and a print needs** stays in the view, hidden by the
  sheet in the browser. A word the browser alone shows, or says otherwise,
  the view chooses by its shell.
## 13. Goals

- **A goal's look** is the greenfield mock's `goal.html` and its `startSheet`, its
  `mock.css`'s rules taken exactly: `.col.wide`, `.byline`, `.btn` and its
  `.primary`, `.quiet`, `.small`, `.rec`, `:disabled`; `.acts`, `.band`, `.goal-head`,
  `.two`, `.report`, `.side`, `.bar`, `.task-line`, `.sent`, `.composer` and
  `.composer-hint`, `.typing`, `dialog.sheet` and all under it, `.option`, `.field`,
  `.text`, `.sheet-error`; at 760 px and under, `.choices` and its `.btn`,
  `.goal-head h1` and `.two`. They live in `temper.css` under `#app:has(>
  .temper-rail)`, on the `--gf-*` variables, under the marker `/* ==== ui: goals ====
  */`, on tokens of their own, `temper-goal*` and `temper-sheet*`. `.acts` keep the
  `.btn.small` height at phone width, wrapping as drawn; the bands' buttons stack at
  760 px (`.choices`); the side follows the report at 1000 px and under by `.two`'s
  wrap, not by a media rule.
- **Goals' look** is the greenfield mock's `goals.html`, its `mock.css`
  rules taken exactly, values and all (`.col`, `.kicker`, `.headline`,
  `.lede`, `.intro`, `.section`, `.section-head`, `.empty`, `.goal-rows`,
  `.goal-row`, its `.state-*` and `.filters`, and their rules at 760 px and
  under), in `temper.css`, scoped under the project's frame with its
  `--gf-*` variables, as Home's are, on tokens of its own, `temper-goals*`.
  A row's top line is `--gf-mono`, 12 px, faint: IBM Plex Mono 400 and 500,
  bundled as the sans is, then JetBrains Mono, `ui-monospace`, `monospace`.
## 14. Team

- **Team's look** (`../pages/team.md`, 1; the whole story's `team.html`
  and `team/team.html`, and `whole-story.css`): its intro as Work's and
  Home's, kicker, headline and lede, in a column 1020 px wide at most
  (`.col.wide`); People's head the mockup's `.shead`, its title at the
  left and the people's shared role, where it's drawn, at its right on
  the title's baseline, as `.shead span`: 13 px, muted, wrapping under
  the title on a narrow width; the people a grid of boxes, `.grid3`, columns at least 240 px
  wide filling the row, 16 px apart, each box `.box`, on the surface's
  ground with the hairline's border, 12 px round, 18 px by 20 px inside,
  8 px between its lines: the name in bold, the role in `small` and
  `muted`, the line in `small`. On a phone the grid is one column, each
  box full width. Its rules go under `temper.css`'s marker `/* ==== ui:
  team ==== */`, before Settings', on tokens of its own, `temper-team*`.
## 15. Focus and scrolling

- **A page target draws no ring:** anything given `tabindex=-1` only to take
  focus (`main`, a heading, a document's section) draws no outline, by
  `skein-ui.css` (skein's `ui/web/README.md`, 10), and `temper.css` says so again of
  `main` and a level-1 heading; a control keeps its ring.
- **A scrolling pane holds its hidden parts:** a chat's log and its draft pane are
  positioned, so a visually hidden absolute part (a who-line) is clipped by the
  pane's overflow and does not lengthen the page: a long chat's document never
  exceeds its window.
## 16. Day and night

- **Day and night** (the look request `night-mode`, 2026-11-07, a first pass the
  user keeps to refine): the mockups' palette is the day's, and `temper.css` sets
  `color-scheme: light`, where `skein-ui.css` follows the system's dark
  preference. *Night* is the same pages on `html[data-night]`, which the shell
  sets from the kept look (`../pages/README.md`, *Its look, kept per browser*):
  - *Tokens first:* every colour of `temper.css` and `task-pages.css` comes from
    a custom property, the day's values exactly as the literals they replace, so
    that a day page is drawn as before. The day `:root` holds the
    `--temper-*` tokens and the tokens the literals become, named by their
    part (a ground, an ink, a line, a primary's ground and words). The five
    greenfield day sets stay scoped to welcome, rail, session or Chats,
    Needs you and public pages: Accounts and connection pages outside
    those scopes keep their undefined-property fallbacks. One night set,
    `html:root[data-night]:has(#app)`, outranks every day set.
    `color-scheme` is `dark` at night. Embedded SVG images keep literal
    colours, and the chat terminal's local palettes keep their own dark
    and light; `transparent`, `currentColor` and `inherit` keep their
    meanings. The placeholder's `loading.css` is separate.
  - *The night's colours* are fabrig.ai's, as the request's overlay draws them:
    paper `#01061b`, surface `#070d2b`, top bar `#050a24`, ink `#fafafa`,
    second ink `#d2def6`, muted `#a9b6d8`, faint and hint `#8493b8`, lines
    `rgba(140, 170, 230, .17)` and `.3`, hover `.1`; the signal `#fb7f2e`, its pale
    `rgba(251, 127, 46, .12)`, line `.38` and words `#ffb27a`; good `#0be2ff`;
    amber `#f5b45a` and red `#ff7a7a` with pales at `.12`. A primary, ink with
    white words by day, is the orange `#fd7a2a` with words `#0a0f22` at night,
    its send arrow drawn in those words; a disabled send is the second line's
    ground. The day's literal greys become the night's lines and hover, and the
    dark line icons drawn as images are turned light, `filter: invert(.85)`.
  - *Shadows keep their colour:* the night set names no shadow's colour, so
    each drawn shadow takes the day's token at night, as the request's overlay
    leaves them: the project picker's and the person menu's `0 12px 32px
    rgba(24, 24, 27, .12)`, the model menu's `.14`, the starter and scale
    cards' `0 1px 2px rgba(24, 24, 27, .04)`, the selected Split's `0 1px 2px
    rgba(0, 0, 0, .08)`, Latest's `0 2px 8px rgba(24, 24, 27, .1)`, and the
    dialogs', the entry's, the cards' and the other menus' alike. The night
    draws its own rings alone, as the overlay does: a card's edge and To
    begin's card in the night's line, the selected effort with none, and the
    composer's focus ring in the hover.
  - *The glow:* small and faint, at the window's top right, over the whole
    window, as the body's ground under a transparent `#app` and main: the
    overlay's radial gradients over its fractal-noise grain, so that it draws
    no rings. The window's ground is the body's however short the page (the
    welcome at 420 by 900 among them): at night the root's own ground, the
    placeholder's day paper (`loading.css`), gives way, so that the body's
    ground fills the canvas.
  - *The switch* (`../view.md`, 3): `temper-night-switch`, 30 px square, no
    border, at the rail's right, 16 px in and 24 px over its foot, the rail
    `position: sticky`; its icon a mask of 16 px line art in the muted colour,
    a moon, and a sun while pressed; the hover's ground and ink on hover. On a
    phone, at 760 px and under, fixed 18 px from the top and 14 px from the
    right, over the top bar. It stays where the rail's other parts go: in the
    rail folded beside a Split, 56 px wide, centred at its foot, 24 px over
    it, and at the open folded rail's foot as on any rail; on a phone's chat
    layout (`#app:has(> .temper-session)`), where the rail draws no box of its
    own (`display: contents`) and none of its other parts, fixed at the right
    of the chat's bar and centred on its height, the bar's right padding
    growing by the switch's width and an 8 px gap. On the top bar
    (*Outside a project*, below; the look request `night-switch-top`),
    `#app > .temper-top > .temper-night-switch`, the bar's last child in its
    row: `flex: none`, the same 30 px square, icon, colours and hover, its
    right margin −6 px, at the bar's right edge; the bar stays one row. At 760 px and under it stays in the row, its press
    target widened to 44 px by a centred `::after`, as the rail's. In every
    state the mark stays at the bar's left and the switch at its right edge,
    whatever comes between: the welcome's steps, or before they show the
    mark alone; outside the welcome the end, packed beside the switch rather
    than centred by the bar's `space-between`, and the mark alone while `me`
    is read or the link is signed out or out of date. The request's pages
    show the welcome's steps alone, so a rule of the landing's own holds the
    other states, leaving the welcome with its steps drawn as the pages draw
    it.
  - *What stays:* a chat's terminal keeps its own theme, dark or light, at
    night (`look.md`, *A chat's terminal*); the placeholder's first paint is the
    day's, since it draws before the shell reads the kept look. The rest of the
    client takes the tokens as they fall, and no page is left with words in
    the day's ink on the night's ground.
## 17. Chat interaction

Moved to [`look-chats.md`](look-chats.md), section 2.

## 18. Outside a project

- **Outside a project** (`../view.md`, 1; the look request
  `outside-pages`, its pages `journeys/outside-pages/01.html` to
  `08.html`): the welcome's look, on the `--gf-*` tokens, under
  `temper.css`'s marker `/* ==== ui: outside ==== */`, wherever the
  rail isn't.
  - *The top bar,* `temper-top`, the frame's banner: temper's mark at
    the left, `temper-brand`, "✦" in the signal before "temper", 15 px
    semibold; at the right, `temper-top-end`, the muted 14 px statuses
    and the person's name, and "Sign out" an outlined 32 px button; last,
    after it, the night switch (*Day and night*, *The switch*).
    22 px by 32 px padding, 14 px by 16 px at 760 px and under, no rule
    under it. While it shows, the page's body takes the paper, the ink
    and IBM Plex Sans at 15 px.
  - *The column,* `temper-outside`, the main: 400 px wide at most,
    centred, from `max(40px, 12vh)` down (32 px at 760 px and under); its
    level-1 heading 28 px semibold; its words muted. The first act,
    `temper-outside-primary`, is filled, 44 px high, in `--gf-primary`
    (ink by day, orange by night); the others, `temper-outside-secondary`,
    outlined on the surface; acts side by side share a row,
    `temper-outside-acts`, equally; a quiet link, `temper-outside-quiet`,
    follows them, centred, 14 px, muted and underlined. A form puts its
    field's name above, 14 px medium in `--gf-ink-2` on the page's
    22.4 px line, and the field and its act on one line; its alert is the
    forms' red. The name's rule outweighs skein's small label
    (`skein-ui.css`'s `label[for^="skein-ui-label-"]:has(+ input…)`,
    `font-size: small`), which a selector of the column's label alone
    does not.
  - *The approval's code,* `temper-approve-code`: 30 px in the mono
    face, letter-spaced .14em, centred on the surface in a hairline box,
    so that a person compares it at a glance with the terminal's; the
    asked time and the warning, `temper-approve-asked`, 14 px; the
    code's field, `temper-approve-field`, 18 px mono at the code's
    spacing, in capitals.
  - *Night* follows the client's look, as on the rail's pages, through
    `--gf-*`; the server's pages and the boot have none.
  - *The old look is gone for good:* the banner, its menu and corner,
    the sign-in card and the unavailable card have no token and no rule
    (`../view.md`, 1); `files/brand.svg` is not in the bundle; the
    stylesheets' accent, `--temper-accent` and the light terminal's
    `--accent`, is the signal, `#c2410c`. Two focused tests keep it so:
    every class `temper.css` selects is one `look.rs` names, and no
    source of the bundle or of the server's pages names the old mark's
    `#b8664e`.

## 19. Open questions

None.
