# The browser's look

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
- **Confirmation sheets:** `dialog.temper-confirm`, 460 px wide at most,
  with 24 px by 26 px padding, 14 px corners and a native `::backdrop`.
  The title is 19 px; the lead 14.5 px; impacts 14 px under a hairline.
  Keep precedes the named primary act in a separate group. An uncertain
  release's choices have a wrapping group of their own, with the same
  secondary buttons; those and the acts keep 44 px phone targets.
  Progress and refusal stay above the acts. A chat's end line, whichever
  its words (`../pages/README.md`), is 12.5 px between two hairlines, and the
  bar has no clock.
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
- **The welcome's title,** "Welcome · first run · temper" on every step,
  whatever its level-1 heading, as Home's is "Home · temper"; the shell's
  turn, the journeys' print and the gallery's take it from the same rule,
  `main` styled `temper-welcome`.
- **The welcome's states' parts,** under `temper.css`'s marker for the
  welcome, take the greenfield mock's rules at the welcome's tokens: its
  `.forge-status`, `.spin`, `.acts`, `.panel`, `.away-note`, `.numbered`,
  `.field-v`, `.cmd` and `.checklist .current .mark`.
- **The welcome's look** (the user's greenfield pass, 2026-11-04;
  `../pages/README.md` and `../pages/welcome.md`, 2), at its own scale, in a block of its own in
  `temper.css`, every rule scoped to the welcome's frame
  (`#app:has(> .temper-welcome)`), so that no other page changes: IBM
  Plex Sans, bundled under `files/fonts/` with its licence (SIL OFL
  1.1), 15 px on paper (#f7f7f5) in ink (#18181b), ✦ before temper's
  name in the signal colour (#c2410c). Its frame is the top bar
  (*Outside a project*, below), 22 px by 32 px, the brand, temper box's line in 14 px muted words, its
  "temper box" ink and bold, and the steps at its right in 14.5 px
  words, each a link holding a numbered 24 px circle and its label, the
  current one filled ink, a done one ticked green, joined by short
  hairlines, hovered ink; below 760 px only the current label shows; at 1100 px and
  under, temper box's line takes its own row under them. After the
  steps, the top bar's end as on every page outside a project, signed
  in the person's name and "Sign out". The bar packs its rows to their
  end (`justify-content: flex-end`) and the steps take the free space at
  their left (`margin-left: auto`, where temper box's line took it at its
  right), so that the steps and the end keep to the right, temper box's
  line beside the brand, and an end the row cannot hold wraps to the
  right of the next; an end straight after the brand, before the
  deployment's read answers, takes the free space at its own left. The main is
  one 680 px column: the tagline (14 px) above the level-1 heading
  (32 px, 500),
  the lead (17 px), then the choices as cards side by side, white on
  paper, 12 px corners, their acts at their foot so that their
  buttons line up; the default card, the one holding the
  step's primary act, is outlined in ink. A primary act is a filled
  ink button, a secondary one a white bordered button, a quiet one
  underlined words. The muted line under the cards is 14.5 px; the
  foot, "‹ Back" at the left and the playground alternative at the
  right, is muted underlined words. Model's pane, "Forge and model",
  follows the choices before the foot: from 1280 px, a 272 px pane to the right of
  the column, its top at the heading's, with a hairline on its left;
  narrower, in the column with a hairline above. Forge's line stacks a quiet
  ring, the forge's strong name in ink and each detail muted on its own
  line; once connected the ring is a green tick, drawn by style alone,
  and "Each step" is gone. Under way, the repository precedes the status,
  with a 3 px hairline track under it, a 35% ink segment sliding across
  every 1.4 s; reduced motion makes it a faint full track with no animation.
  While the fetch says how far it is, `temper-flow-meter` draws a 4 px
  bar in its place, ink over the hairline colour, with the figure at its
  end, muted, in tabular figures: filled to `--skein-ui-progress` with a
  size, and with none, its `[role="progressbar"]` having no
  `data-progress`, the same sliding segment and the bytes at its end.
  Done, failed and GitHub's installation waits have no bar. Forgejo's sheet is a native modal dialog, 620 px
  wide, white, 14 px corners, over a dimmed page: its heading 19 px,
  its line 14 px muted; "Start it with" its fieldset's legend; each
  choice a bordered card, 10 px corners, that picks its choice
  wherever pressed, the chosen one outlined in ink; the chosen one's
  fields under it, indented, small bold labels over 38 px inputs, the
  addresses a monospace text area; its foot, under a hairline, the
  quiet "Cancel" and the primary Run at its right (the user's mockup,
  2026-11-04). Ready's pane is Model's, its lines 18 px apart:
  each stacks its mark, a quiet ring, a green tick or the signal's "✕", its
  name in ink at 500 and what it says muted, "Open Forgejo ↗" under the
  forge line's words; a stopped line's "WHO didn’t answer" is in the signal's
  colour, its why 13.5 px under it, its "Try again" and quiet act a row under
  that (the request `model-check-pane`). After the lead,
  the first project's form, its
  small bold label over a 300 px field, 40 px high, and "Create your
  project" at the field's right, "Rename it whenever you like." muted
  under them. At 760 px and under, the cards stack, the steps but the
  current one keep their circles alone, Ready's field and act stack, the act 48 px high and
  full width, and the gutters are 16 px in the top bar and 20 px in
  the main. Each act keeps a focus ring.
- **The task pages' stylesheet,** `task-pages.css`, comes third, after
  `temper.css`, whose variables and rules it builds on: what an agent's
  task's and a change's pages add to the look, as `task.html` draws it in
  the terminal sheets (`../pages/task.md`, 2 and 3): a plan row's spend as a
  fourth, right-aligned cell, a row's dependency line in small muted words
  under its name, the list's name shown before its items. A goal's page takes
  nothing from it. `scripts/bundle.sh` passes the three to `skein-ui-bundle` in
  that order.
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
  chat's bar does, the night switch keeping its place. The link's state
  shows at the rail's foot, with its dot, while it is not live. The person's
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
  at the foot, a row as the link's state's, above the person's row, its
  dots after its word. The rules go under `temper.css`'s marker `/* ==== ui:
  rail ==== */`, before the switcher's; the rules they replace go where they
  stand.
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
- **The Chats page and every chat are the greenfield mock's**
  (`../pages/README.md`): each selector the document names is the mock's rule
  copied onto the client's own classes, pixel for pixel, under the sheet's
  `/* ==== ui: chats ==== */` marker, in the `--gf-*` tokens, with
  `--gf-hint: #8a8a93`, `--gf-signal-ink: #9a3412` and
  `--gf-signal-line: #f1d5bd`; the mock's phone rules for those selectors
  come from its last `@media (max-width: 760px)` block. At that width the
  session's keyboard hint is hidden, as the configure-chat look request
  draws it. The tokens:
  `CHATS_PAGE` on the Chats page's main and `SESSION` on every chat's and
  `NewChat`'s. An agent's task's page keeps its own look, `AGENT_PAGE` and
  `TASKBAR`, with `ENTRY`, `COMPOSER_AREA`, `COMPOSER` and `HINT` for its
  composer.
- **An Ask temper chat's tag** (`../pages/README.md`), `CHATS_TAG`'s " · Ask
  temper" after the chat's number in its bar, takes the number's look,
  `.temper-byline`'s 12 px and faint, on one line, the bar's gap before it
  taken back so that its own " · " is the only space and the two read
  "T9 · Ask temper". At 760 px and under, where the number goes, the tag
  stays without its " · " (the look request `ask-temper-place`). The bar's
  spend paragraph stays hidden: the hiding rule excepts only the number's
  `.temper-byline` and the tag's `.temper-chats-tag`.
- **A chat's document** (`../pages/README.md`) is `configure-chat-draft`'s rules, in
  `temper.css` under the same marker: `temper-mode-chat`, `-split` and `-draft` on
  the session, `.temper-doc`, `.temper-doc-inner`, `.temper-doc-path`,
  `.temper-doc-sec`, `.temper-doc-changed`, `.temper-doc-note`, `.temper-seg`, and
  the rail's strip with `.temper-rail-unfold` and `files/menu.svg`. What those
  name is drawn; a flow takes the tokens' look below. The sheet to start a goal
  is the mock's `dialog.sheet`, `.option`, `.field` and `.text`, and `Ctrl+\` steps
  through the modes.
- **A chat's terminal** (`../pages/README.md`) is the requests `chat-terminal`,
  `chat-terminal-font` and `chat-terminal-draft`'s `terminal.css`, in `temper.css`
  under `/* ==== ui: chats ==== */`, its selectors on temper's tokens:
  - *Its tokens:* `temper-terminal` on the page's main, for the request's `.tt`,
    with `temper-terminal-light` for `[data-theme="light"]` and
    `temper-terminal-plex` for `[data-font="plex"]`; `temper-terminal-top`,
    `-where`, `-star`, `-place`, `-sep`, `-task`, `-acts` and `-btn` for the top
    line; `temper-terminal-run` for the running line; `temper-terminal-line` and
    `-hint` for the quiet line, whose model button takes no token, as the
    request's `.tt-model`, and so none of `-btn`'s padding or ground;
    `temper-terminal-head` for a list's head;
    `temper-terminal-note` for a note; `temper-to-terminal` for `>_` in the bar.
    The conversation, the box, the list and the pane keep the page's own tokens
    (`SESSION_COMPOSER`, `TURN_TOOL`, `DOC` and the rest), which the terminal's
    rules reach under `.temper-terminal`; the list is skein's options menu, whose
    `listbox` and `option` roles the rules select.
  - *Its ground and ink:* dark first, the request's `--bg` `#191816`, `--ink`
    `#e8e3da`, `--dim`, `--faint`, `--line`, `--sel`, `--accent` `#e08a68` and
    `--good`, as variables on `.temper-terminal`, and its light ones on
    `.temper-terminal-light`. Both themes are the terminal's: the rest of the
    client keeps its light palette.
  - *Its type:* DejaVu Sans Mono 400 and 700 at 11pt, line height 1.6, ligatures
    off, a single size, 13px at 760 px or less; with `temper-terminal-plex`, IBM
    Plex Mono at 14px, bold 500. The two DejaVu Sans Mono weights are the
    request's subsets, in the bundle's `files/fonts/` as
    `dejavu-sans-mono-400.woff2` and `-700.woff2`, beside the DejaVu licence
    already there, `files/fonts/README.txt` saying where they come from and how
    they were cut; their `@font-face` rules stand with the others, and a browser
    fetches them only when the terminal first draws. Italic is the regular face
    slanted; text past the subset falls back to the system's monospace.
  - *Its layout:* fixed over the whole window, a grid of the top line and the
    panes; the conversation's column 100 characters at most, left; Split two
    halves behind a rule, the phone's one pane with the box at the foot; the
    request's 760 px rules. The running line is the first line of the box's
    form, the request's `.tt-foot`, so that only its own padding stands between
    it and the box. The conversation's ground is the terminal's `--bg` in every
    mode and at every width, scoped above the shell's own Split and Draft rules,
    whose `--gf-surface` is the browser look's.
  - *The draft beside it:* the pane keeps `.temper-doc`'s rules from *A chat's
    document*, above; the terminal sets their variables on its pane, its ground
    `--doc-bg` a shade lighter than the terminal's, its ink softer, a box a shade
    above the ground, the strong box in the ink, a changed section's pale block
    and note in the terminal's accent (the request's `--doc-*` values, light and
    dark), and adds only what the shell's rules don't say: the pane's ground, its
    padding, its rule and its scrollbar, the request's `scrollbar-width: thin`
    with `scrollbar-gutter: auto` at every width, where the shell's pane keeps a
    stable gutter on both edges. It restates none of the shell's document rules,
    which `terminal.css` restates as a page outside `#app` must.
- **Ask temper's ways** (`../pages/README.md` and `../pages/chat/conversation.md`, 3) are `ask-temper-ways`'s rules, in
  `temper.css`: the opening's, `.temper-ways` with its tabs, axis, panel, card, legend
  and act, under `/* ==== ui: settings ==== */`, where `.temper-starter`'s stand; a
  path's, `.temper-ways-path` and its marks, under `/* ==== ui: chats ==== */`, which a
  reply's views share. Where skein draws an element otherwise than the request's page (a
  tab an `<a role="tab">`, a scene's stop a `<g>` with no ARIA, an open stop a token, one
  SVG laid out by width where the page has two), the selector names what is drawn and
  keeps the request's declarations.
  The drawing's rules (the path's and the legend's) and the tabs', line's,
  card's and act's hold under How work runs' frame as under Ask temper's
  page (`../pages/settings.md`, 2), each selector naming both. With a fourth tab,
  "Yours", at 760 px and under the tabs stand two by two, as the whole
  story's `whole-story.css` has them, so that "Agents review" keeps its
  words whole on a phone.
  A path's marks take no stroke but their own rules', as the request's
  page has none: skein's default stroke for a scene's marks (skein's
  `ui/web/README.md`, 10) gives way under `.temper-ways-path` by one rule of the
  path's, scoped to its drawn figure, `stroke: none` on the marks skein's
  rule names, outweighing skein's rule and outweighed by every mark's own
  (the line's, the boundary's, a person's ring, the halo, the caret, a
  focused stop's area); skein's rule stays as it is.
  Who acts is the whole story's marks (`web/whole-story/whole-story.css`,
  `.ways-*`; `../../draft/surfaces.md`, 13), orange the person's alone: a
  person's ring `--gf-paper` stroked `--gf-signal` at 2; an agent's star
  `--gf-ink`, and outlined when needed, `--gf-paper` stroked `--gf-ink`
  at 1.3 with round joins; code's square `--gf-muted`, and outlined,
  `--gf-paper` stroked `--gf-muted` at 1.3; a stop's small words 10 px
  in `--gf-faint`; the legend's agent "✦" in `--gf-ink` and its agent
  when needed the outlined star. The line's base is `--gf-line-2` at
  1.5; each rise's fade (`../pages/chat/conversation.md`, 3) is the look request's, its
  stops coloured by their tokens: a person's `--gf-signal` at .8 at its
  mark, .28 at the dotted line, `--gf-line-2` at the base; an agent's
  `--gf-ink` at .3 and .1, the whole story's lighter tone for an agent
  (`.ways-rise.a`), then `--gf-line-2`. Night takes the same tokens'
  night values.
- **How work runs' look** (`../pages/settings.md`, 2, *The page*) is the whole
  story's `web/whole-story/how.html`, its `whole-story.css` rules taken
  exactly, values and all, in `temper.css` under `/* ==== ui: how ==== */`,
  which stands after the ways' rules under Settings' marker, scoped under
  the project's frame on tokens of its own, `temper-how-*`:
  - *its column,* `temper-how-wide` on Settings' column while How work runs
    is its tab: `.col.wide`'s 1020 px, the frame's other tabs keeping 720;
  - *its head,* Settings' own (`.intro`, `.kicker`, `.headline`, `.lede`),
    "Ask temper to change this" a `.btn.dark`, Settings' primary;
  - *a section's head,* `.shead`: its title and, at its right,
    `temper-how-way`, "the way: NAME", as `.shead span`;
  - *the path,* Ask temper's drawing and legend under their own rules
    (*Ask temper's ways,* above), who acts by the path's marks alone: no
    lanes, no pills, no colour of its own;
  - *the ways' tabs, line, card and act* take Ask temper's ways' rules
    (above), the same declarations under the frame's scope: the card under
    the drawing, its caret across;
  - *Roles,* `whole-story.css`' `.rows` and `.role` rules taken exactly:
    a role's row `.role > summary`'s grid, 200 px for the name, its line
    small and muted, the model at its right with a "›" turning down while
    open; opened, `.role > dl`'s terms and `.role-more`'s link under its
    hairline; at 760 px and under the line under the name, the terms
    stacked; on tokens of its own, `temper-how-roles`, `temper-how-role`,
    `temper-how-role-line`, `temper-how-role-model` and
    `temper-how-role-more`;
  - *Budgets' row,* `temper-how-row`, `.row`'s grid with the first column
    200 px, its small words muted and its figure at its right;
  - *What reaches you,* `whole-story.css`' `.grid3`, `.box` and `.who`
    rules taken exactly, on tokens of its own: `temper-how-reaches`, the
    columns' grid, `repeat(auto-fit, minmax(240px, 1fr))` with a gap of
    16, so two boxes share the width and a phone stacks them;
    `temper-how-reach`, a box, its lines `.small` with no bullets' margin
    of their own; `temper-how-who-agent` and `temper-how-who-person`, the
    chip and its mark, `.who.a`'s star (`--star`, `--gf-ink`) and
    `.who.p`'s ring (`--gf-signal`); `temper-how-reach-note`, the box's
    last line, `.small.muted`; its rows under the grid Settings'
    `.setting` rows;
  - *What follows a landing,* `whole-story.css`' `.grid2`, `.box`, `.box
    h3`, `.chips` and `.who.c` rules taken exactly, on tokens of its own:
    `temper-how-follows`, the boxes' grid, `repeat(auto-fit, minmax(300px,
    1fr))` with a gap of 16; `temper-how-follow`, a box, its title `.box
    h3`'s 17 px at 600, its words and its lag's line `.small`, the lag's
    `.small.muted`; `temper-how-chips`, its chips' row, wrapping with a gap
    of 6; `temper-how-who-code`, code's chip, `.who.c`'s 7 px square with
    corners of 1.5 on `--gf-muted`, beside What reaches you's
    `temper-how-who-agent`;
  - *Budgets' head,* the period's words at the section head's right, as
    `temper-how-way` is;
  - *on a phone,* at 760 px and under, the column takes the frame's phone
    width and the path runs down the page, as on Ask temper's page.
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
- **A page target draws no ring:** anything given `tabindex=-1` only to take
  focus (`main`, a heading, a document's section) draws no outline, by
  `skein-ui.css` (skein's `ui/web/README.md`, 10), and `temper.css` says so again of
  `main` and a level-1 heading; a control keeps its ring.
- **A scrolling pane holds its hidden parts:** a chat's log and its draft pane are
  positioned, so a visually hidden absolute part (a who-line) is clipped by the
  pane's overflow and does not lengthen the page: a long chat's document never
  exceeds its window.
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
    growing by the switch's width and an 8 px gap.
  - *What stays:* a chat's terminal keeps its own theme, dark or light, at
    night (`look.md`, *A chat's terminal*); the placeholder's first paint is the
    day's, since it draws before the shell reads the kept look. The rest of the
    client takes the tokens as they fall, and no page is left with words in
    the day's ink on the night's ground.
- **The composers** (`NewChat`'s and a chat's) are text areas with
  skein-ui's `SubmitsOnEnter`: Enter sends, Shift+Enter is a new line
  (skein's `ui/web/README.md`, section 5), which the view allows in a
  composer's words, counted within its bound (`../view.md`, 1). A
  chat's shows its placeholder, "Reply to T6", while it is empty, as
  `placeholder`, its name staying "Message", and `NewChat`'s "What do you
  want done?", its name too (`../pages/README.md`). They
  grow with their words and shrink after sending by `temper.css` alone
  (`field-sizing: content`, from one line to a bound), with no
  component and no script of temper's; a browser without it shows an
  area of fixed height that scrolls.
- **A run's live text** is a patch like any other: the chat's words
  being written (`../pages/README.md`) change in place;
  skein's page keeps focus, caret and scroll across it, and the
  conversation, a log, keeps its end in view while the person has not
  scrolled away (skein's `ui/web/README.md`, section 5). Where the window has
  let its newest lines go, live text waits until the end is read again
  (`../pages/chat/conversation.md`, 2).
- **Reading the conversation** (`../pages/chat/conversation.md`, 2): the bound log's
  head and foot ask for earlier and later lines as they near the view,
  with no Older control. PgUp and PgDn scroll it from the composer too,
  except while a menu or dialog is open. skein's page part measures and
  restores the words' position when lines are read above, including in
  a browser without `overflow-anchor`; Latest or sending on a chat
  returns to its end, reading it again where the window let it go.
- **Outside a project** (`../view.md`, 1; the look request
  `outside-pages`, its pages `journeys/outside-pages/01.html` to
  `08.html`): the welcome's look, on the `--gf-*` tokens, under
  `temper.css`'s marker `/* ==== ui: outside ==== */`, wherever the
  rail isn't.
  - *The top bar,* `temper-top`, the frame's banner: temper's mark at
    the left, `temper-brand`, "✦" in the signal before "temper", 15 px
    semibold; at the right, `temper-top-end`, the muted 14 px statuses
    and the person's name, and "Sign out" an outlined 32 px button.
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
