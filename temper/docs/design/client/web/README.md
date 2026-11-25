# The browser

Provisional, 2026-10-05, revised 2026-11-07. temper's browser client: a
second shell of temper's client (`../README.md`), beside the terminal, on
skein's browser parts. What its pages do is the client's (`../README.md`;
`../forge/README.md`). The greenfield mock (`greenfield/README.md`) is the look's
target, with later look requests, and takes precedence over `mockups/`
(the user, 2026-11-05). Journeys record and print what temper draws
(`journeys.md`). How a view runs in a browser is
skein's (skein's `ui/web/README.md`), and how a client of jig's does is jig's
(jig's `protocol/browser/README.md`). What is still open is section 3.

## 1. In one page

- **A second shell of the same client.** temper's client root, its view
  and the forge's family (`../root.md`, section 1 and `../forge/family.md` and `../view.md`), compiled to
  WebAssembly under a browser shell of temper's,
  `temper-client-shell-web`, on skein's browser parts (`skein-ui-web`;
  skein's `ui/web/README.md`) and jig's client adapter and sign-in part
  (`jig-client-shell-web`; jig's `protocol/browser/loop.md`, 1.1), with
  neither jig's look nor its reference UI. The adapter carries jig's
  default HTTP mapping over fetch (`jig-client-protocol-http`). temper
  writes no web wire and no web protocol of its own: the client's
  records cross as jig's wire carries them, binary records from a schema
  per owner, never JSON (jig's `protocol/client/wire.md`, section 1).
- **The pages temper has,** in both shells: the client's (`../pages/README.md`
  ). A page added to the view is in the browser once it is in the
  view, with its look and, where it has one, its browser-only story
  (`testing.md`, 2).
- **No framework.** The client and its view are step code under the
  subset in a browser too (skein's `ui/README.md`, section 1).
- **The view tree is the fake DOM.** Below the browser, every world
  reads skein-ui's tree by role, name and text through skein's fake
  person (skein's `ui/testing.md`, 1.1): no browser, no WebAssembly and no fake
  DOM library.
- **Served by temper's server, from one origin.** `temper-service` serves
  the page and the bundle on its clients' listener (skein's `ui/web/bundle.md`,
  4), beside jig's clients' component under `/jig` (`../engine.md`
  ), from the bundle the binary holds or a directory its
  configuration names, with jig's example guards and the page's policy,
  which is temper's: a strict content security policy, no frames, no
  CORS (jig's `protocol/browser/serving.md`, 1.1 and 1.2). A binary built
  without its browser serves the page for no bundle, and the terminal
  works as before (`serving.md`, 1).
- **The bundle is in the binary.** A plain `cargo build` (or `cargo
  build --release`, or `cargo install --git`) makes a `temper-box` that
  holds its bundle, so that running it is the whole install. The
  machine's builds, temper's gate and its agents, build without it
  (`TEMPER_INTERNAL_BUNDLE=none`), as fast as before (`bundle.md`, 1).
- **Signed in by the printed link,** opened in a browser on the engine's
  machine: jig's sign-in part spends it, the credential is the
  mapping's cookie, and a reload signs in again by it (`sign-in.md`, 1).
- **Browser tests only for what a browser alone shows.** temper's
  stories are the worlds' and the real loop's, through the terminal
  shell (`../../testing/tiers.md`, 1.1 and `../../testing/real-loop.md` and `../../testing/checks.md`, 3). The browser tier proves
  the bundle under its policy, signing in by the link and the cookie,
  addresses, the composer's keys and jig's attack stories that need a
  browser, and nothing a world can show (`testing.md`).
- **A real browser at the top, driven from Rust,** by `skein-browser`
  (`testing.md`).
- **The gallery** prints the browser's pages in named states as static
  documents, beside the bundle's stylesheets, for screenshots held to
  the mockups: no server, no script, no network (`gallery.md`). It stays
  for the states no journey reaches.
- **Journeys** are their walks, printed in one process in a world by
  `temper-client-walk` as the client's own pages, where the look is
  tuned (`journeys.md`; `../journey-walks.md`). Main's pages have one
  writer after each landing (`../journey-writer.md`). A person's
  recording prints outside the repository's journeys (`journeys.md`, 5.1).
- **Look requests** carry the person's changes from the prints into a
  landing, an account and an after-landing render check (`look-requests.md`).

The documents:

- [`shell.md`](shell.md): the browser shell and host.
- [`serving.md`](serving.md): serving the page and bundle.
- [`addresses.md`](addresses.md): addresses and navigation.
- [`sign-in.md`](sign-in.md): signing in.
- [`bundle.md`](bundle.md): the bundle in the binary.
- [`push.md`](push.md): push notifications.
- [`gallery.md`](gallery.md): the gallery and its tests.
- [`gallery-project-states.md`](gallery-project-states.md): project and settings states.
- [`gallery-journey-states.md`](gallery-journey-states.md): journey and interaction states.
- [`look.md`](look.md): the shared look.
- [`look-welcome.md`](look-welcome.md): the welcome's look.
- [`look-settings.md`](look-settings.md): settings and accounts' look.
- [`look-chats.md`](look-chats.md): chats and their documents' look.
- [`look-ways.md`](look-ways.md): ways of working's look.
- [`journeys.md`](journeys.md): journey walks and prints.
- [`journeys-recordings.md`](journeys-recordings.md): recording and replaying journeys.
- [`../journey-walks.md`](../journey-walks.md): a journey's walk.
- [`../journey-walks-world.md`](../journey-walks-world.md): executing a walk in a world.

## 2. What temper brings to the browser

- **Its crates as they are.** The client root, the forge's side, the
  view and `temper-client-protocol-wire` are step crates under the
  subset, which build for the host and for wasm32 alike; the terminal
  and the agents' tools are not part of the bundle.
- **Its look:** temper's style tokens and a stylesheet over skein's,
  tuned on the journeys' prints (`journeys.md`, section 7; skein's `ui/web/README.md`, 3.2
  and section 10): 2.5.
- **Its pages,** the client's (`../pages/README.md`), each at an address of
  its own (`addresses.md`, 1); and the forge's parts, from `../forge/README.md`, as the forge's
  side reads them.
- **Signing in:** by the link `temper-box serve` prints (`sign-in.md`, 1). The forge as
  people's sign-in provider is temper's own routes (`../README.md`,
  section 5).
- **The toolchain:** the standard library for `wasm32-unknown-unknown`,
  skein's bundler and the glue it names (skein's `ui/web/bundle.md`),
  and Chromium for the browser tier.

### 2.1 The shell

Moved to [`shell.md`](shell.md), section 1.

### 2.2 Serving

Moved to [`serving.md`](serving.md), section 1.

### 2.3 Addresses

Moved to [`addresses.md`](addresses.md), section 1.

### 2.4 Signing in

Moved to [`sign-in.md`](sign-in.md), section 1.

### 2.6 The bundle in the binary

Moved to [`bundle.md`](bundle.md), section 1.

### 2.7 Being pushed

Moved to [`push.md`](push.md), section 1.

## 3. Open questions

- **The two-step Home and night comparison** (`look.md`, `gallery.md`, 2): temper/ui
  compares `ask-temper-start`, `to-begin-two-steps` and `night-mode` after
  landing, including their named states, open menus and disclosures at
  1440, 1000 and 420 px (`look-requests.md`, section 4).
  Include `home-begin-*`, Home with a need and a chat left out with
  either step left, and their return after the last step or a skip.
  Check Ask temper's restricted rail, its three tabs at 420 with the
  composer and model picker wholly above them, the same rail and tabs in
  `ask-first-run`, `ask-typing`, `ask-sending` and `ask-refused`, and
  `night-home`, `night-ask` and `night-chat-split`,
  the switch in the
  folded rail open and closed and at a phone's chat bar, and the night's
  shadows against their day values (`look.md`, *Shadows keep their colour*). Check
  the faint top-right glow without rings and readability on every other
  page; refine the first night palette with the user. Run the history
  story's phone Ask row (`testing.md`, 6, story 11) and night reload row after landing.
  Default: tokens, focused tables and gallery prints establish no
  rendered equality or browser runtime result.

- **Ask temper's repaired bar and first-run gallery** (`look.md`, `gallery.md`, 2):
  temper/ui compares the `ask-temper-place` tag at 1440 and 1000 px as
  "T9 · Ask temper", and at 420 px as "Ask temper" alone, with the
  spend paragraph still hidden. Include the first-run states' rail row
  and phone tab, and reference chats without a false Ask temper tag or
  current row (`look-requests.md`, section 4). Default: the
  stylesheet and gallery fixtures; their checks establish no rendered
  equality. The Ask page's secondary heading, "How should temper run
  Hello?", stays until a later look round with the user.

- **The outside pages' comparison** (`sign-in.md`, 1, `look.md`, `gallery.md`, 2; `../pages/README.md` and `../view.md`, 1
  ; `../forge/served.md`, 1; `../../protocol/sign-in.md`, 4.3): temper/ui
  compares the look request `outside-pages`'s pages 01 to 10 under their
  overlay with main's printed pages after landing, at 1440, 1000 and 420 px
  (`look-requests.md`, section 4), with the states its message
  names in words: the top bar while `me` is read, offline and behind; the
  public pages' "Sign in"; every approval state, deciding and signed out
  with and without Google; each `Authorize` state; out of date; no
  project yet with and without the welcome; the GitHub App's setup pages;
  and night. It takes in the six repairs of `look-round-22`: the
  welcome's top bar with its steps and link region, no person or "Sign
  out", and the night switch last (`welcome-top` and `night-switch-top`)
  at each width; the top bar over an empty main
  while `me` is read at `/chats` and `/chats/new`, until "No project yet"
  settles, with the heading's focus once the page comes (`addresses.md`, 1); "Code" and
  "Project name" at 14 px on a 22.4 px line, the field and its act
  1.59 px lower; the night welcome at 420 by 900 with no day paper under
  its ground; "Sign in" on the public pages without Google; and the
  Google endings' typographic apostrophe. Default: the stylesheets, the
  focused tests and the gallery's prints establish no rendered equality.

- **The chat terminal's comparison and keys** (`look.md`, `gallery.md`, 2; `../pages/README.md`
  ): temper/ui compares the four `chat-terminal` requests' pages,
  rules and named states with the nine gallery states and live page
  after landing, at 1440, 1000 and 420 px, with menus and disclosures
  open (`look-requests.md`, sections 4 and 5). Include dark
  and light, both fonts, phone Split, a draft being written, tables,
  changed-section flash, the figure, questions, proposals, delegations
  and failures. Include the phone Split conversation and composer's dark
  ground, the draft's thin scrollbar without a reserved gutter at every
  width, the quiet model button's line height and the running line's
  placement inside the foot without a gap above the box. The ordinary
  bar's two-row title at 1000 px stays: `>_` and its status words share
  the bar, by the user's `look-round-17` request.
  Real keys check the full-screen behaviours named in
  `../README.md`, section 5. Default: the shared document rules and
  focused tables and prints; these establish no rendered equality or
  real-key result. Temper/ui retains the departures in
  `look.status.md`: Plex's phone size, adjacent-note spacing and
  options unchanged on hover.

- A filter row while its project's ways arrive: `testing.md`, section 7.

- **Settings, Models comparison** (`look.md`; `../pages/README.md`): temper/ui owns
  page 07 of `model-picker` beside the shipped page, its named states and
  screenshots at 1440, 1280, 1000 and 420 px after landing
  (`look-requests.md`, section 4). Default: host view and client
  world checks; compilation alone establishes no rendered equality.
  Include Provider's select chevron, the add form's single row and the
  Provider and Add headings. The shared Settings H1 and open project
  menu row keep their frame's design, as the repair request leaves them.

- **A flow in a chat and its document** (`look.md`; `../pages/README.md`): temper/ui
  compares both against `skein/docs/design/draft/diagrams/preview.html`
  after landing, at a width where the flow is horizontal, one where it
  is vertical, and one where only the fallback fits. Default: the host's
  byte comparisons and the stylesheet's rules until that comparison.

- **The connect-flows proof** (`look.md`, `testing.md`, 6): temper/ui owns the comparison
  of request pages 01–23 with main, at 1440, 1000 and 420 px with
  disclosures and menus open, and screenshots also at 1280 px
  (`look-requests.md`, 4). The retained browser stories check
  Accounts return/history and sheet scroll, search, adoption and phone
  geometry; they do not compare all request pages. Default: findings
  after landing, with a passing repaired-story run and the unchanged
  whole-tier budget established by the after-landing tier. Compare the
  request-owned parts, including select chevrons, the source chooser,
  Copy public's caption and four-line box, list margins, empty search,
  More, control captions, connection actions and model-way paths. The
  shared Settings H1 and open project menu row keep their frame's design.

- **Whole questions and starting from Goals** (`look.md`, `gallery.md`, 2; `../pages/README.md`, `../pages/goal.md`, 2, 3
  ): the long question's context, evidence and choices, the
  typed Draft sheet, zero-funding explanations and change-only landings
  need their browser, walk and render checks after landing. Default:
  shared view and domain tables, focused client stories from a chat and
  Goals, and gallery prints; no new browser test for `ui-goal-asks-2`.

- **The fresh project's draft flow** (`../pages/README.md`, `../view-navigation.md`, 1, `../pages/settings.md`, 2):
  the real-box check of an Ask temper chat's draft offer and the draft-to-goal
  flow, including Goals in the phone tab bar, is after-landing work
  (`../README.md`, section 5). Default: the view tables and fake-model
  `chat-draft` journey, with no browser or render check during the plan.

- **A box still starting:** a link opened before serve is ready gets no
  bytes until it is, so no placeholder either, and that wait stays the
  browser's own (`../../protocol/setup/box.md`, 1). Default: as it is.
- **Night following the system,** a first visit taking the system's dark
  preference as `skein-ui.css` does. Default: a browser with no kept look
  is day, and night is the switch's alone (`look.md`, *Day and night*).
- **Night before the first paint,** the placeholder drawn at night when the kept
  look says so, before the shell starts. Default: the placeholder is the day's
  (`look.md`, *Day and night*).
- **Other browsers** than Chromium are `skein-browser`'s question
  (skein's `testing/browser.md`).
- **One shared stream per public document,** fanned out by the server
  to every visitor reading it, in place of each page reading again
  (`public.md`). Default: none; pages read again at the `max-age`, and a
  visitor never holds a stream.
- **Public history** (`public.md`, 3): ended goals from before publication or across
  a restart need jig's source index (`jig/docs/design/domain/people/README.md`,
  section 11). Default: goals ended while the public index kept the
  project, and the board's line says what it leaves out. A merge from
  before publication can show while its change is handed from a live
  goal's tree; publication is not a commit-time cutoff. A merge past the
  forge's log is `../forge/README.md`, section 8's.
- **The kit's stylesheet without a bundle:** the gallery takes
  `skein-ui.css` by running skein's bundler with no module (`gallery.md`, 1).
  Default: so, until skein's bundler offers the stylesheet alone.
- **A press a render can lose:** the diff matches unkeyed children by
  position among siblings of the same role (skein's `ui/README.md`, section 7),
  so a section arriving before a control's group gives the control a new
  element, and a press between its mouse down and up does nothing, with
  nothing shown. Default: a group that sections arrive beside takes a
  role of its own, as Accounts' Sign out does, page by page; skein's
  diff stays.
- A phone's width in a test: `testing.md`, section 7.
- **The rail's foot on a phone** (`look.md`; `../view-navigation.md`, 1): the top bar holds the
  switcher alone, as the mock's does, so a project's page on a phone shows no
  link's state, no "Behind" and, but on the `Accounts` tab, no "Sign out did
  not reach the engine.", and
  the way out of the playground is its banner on `Chats` alone. The mock has no
  link to say so of. Default: so; whether a link not live takes a place in the
  top bar is a look request's.
- **A covered page's other edges** (`look.md`): the mock and the sheet keep off the
  home bar alone, and at 760 px and under alone. A phone on its side, wider than
  that, draws the rail and a chat's foot under its notch and its home bar.
  Default: as the mock draws it, until a look request.
- **A tab refused in the gallery** (`gallery.md`, 2; `../pages/README.md`, *The sign-in under way*;
  `../../draft/sign-in-new-tab.md`, 2): the gallery draws what the view draws, and the
  view has no fact of a tab refused, so the note's plain title and primary link have
  no state of their own. Default: the view's test holds them, until that draft gives
  the fact.
- A chat's way to the Closed filter: `testing.md`, section 7.
- The box's pace in the tier: `testing.md`, section 7.
- A wait's CPU in the tier: `testing.md`, section 7.
- The remaining browser cost: `testing.md`, section 7.
- Merged stories after landing: `testing.md`, section 7.
- Reading's test past its run's forty seconds: `testing.md`, section 7.
- A long chat at the person's whole bound: `testing.md`, section 7.
- A public page's plain text in one query: `testing.md`, section 7.
- Every wrapped text fragment: `testing.md`, section 7.
- A public page changing during measurement: `testing.md`, section 7.
- Expected browser errors scoped to an act: `testing.md`, section 7.
- A compile cache across the tier's tests: `testing.md`, section 7.
- **Reading a conversation in every browser** (`../pages/chat/conversation.md`, 2; `../../draft/chat-reading.md`, 5):
  the position staying put when lines load above is for the user to read on a
  real chat. Default: the browser tier runs Chromium, including with
  `overflow-anchor: none`; Firefox and Safari themselves are read, not run.
