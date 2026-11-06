# The browser

Provisional, 2026-10-05, revised 2026-10-26. temper's browser client: a
second shell of temper's client (`README.md`), beside the terminal, on
skein's browser parts. What its pages do is the client's (`README.md`;
`forge.md`). What temper draws is the source of truth, recorded and
printed as journeys (`journeys.md`); the mockups (`mockups/`) are the
design of what is not built yet, reconciled to temper where they
differ (the user, 10-26). How a view runs in a browser is
skein's (skein's `ui-web.md`), and how a client of jig's does is jig's
(jig's `protocol/browser.md`). What is still open is section 5.

## 1. In one page

- **A second shell of the same client.** temper's client root, its view
  and the forge's family (`README.md`, sections 3 to 5), compiled to
  WebAssembly under a browser shell of temper's,
  `temper-client-shell-web`, on skein's browser parts (`skein-ui-web`;
  skein's `ui-web.md`) and jig's client adapter and sign-in part
  (`jig-client-shell-web`; jig's `protocol/browser.md`, 5.1), with
  neither jig's look nor its reference UI. The adapter carries jig's
  default HTTP mapping over fetch (`jig-client-protocol-http`). temper
  writes no web wire and no web protocol of its own: the client's
  records cross as jig's wire carries them, binary records from a schema
  per owner, never JSON (jig's `protocol/client.md`, section 3).
- **The pages temper has,** in both shells: the client's (`README.md`,
  3.5). A page added to the view is in the browser once it is in the
  view, with its look and, where it has one, its browser-only story
  (4.2).
- **No framework.** The client and its view are step code under the
  subset in a browser too (skein's `ui.md`, section 1).
- **The view tree is the fake DOM.** Below the browser, every world
  reads skein-ui's tree by role, name and text through skein's fake
  person (skein's `ui.md`, 17.1): no browser, no WebAssembly and no fake
  DOM library.
- **Served by temper's server, from one origin.** `temper-service` serves
  the page and the bundle on its clients' listener (skein's `ui-web.md`,
  4.4), beside jig's clients' component under `/jig` (`README.md`,
  section 8), from the bundle the binary holds or a directory its
  configuration names, with jig's example guards and the page's policy,
  which is temper's: a strict content security policy, no frames, no
  CORS (jig's `protocol/browser.md`, 12.1 and 12.2). A binary built
  without its browser serves the page for no bundle, and the terminal
  works as before (2.2).
- **The bundle is in the binary.** A plain `cargo build` (or `cargo
  build --release`, or `cargo install --git`) makes a `temper` that
  holds its bundle, so that running it is the whole install. The
  machine's builds, temper's gate and its agents, build without it
  (`TEMPER_BUNDLE=none`), as fast as before (2.6).
- **Signed in by the printed link,** opened in a browser on the engine's
  machine: jig's sign-in part spends it, the credential is the
  mapping's cookie, and a reload signs in again by it (2.4).
- **Browser tests only for what a browser alone shows.** temper's
  stories are the worlds' and the real loop's, through the terminal
  shell (`../domain/testing.md`, 2.1, 2.4 and 6.3). The browser tier proves
  the bundle under its policy, signing in by the link and the cookie,
  addresses, the composer's keys and jig's attack stories that need a
  browser, and nothing a world can show (section 4).
- **A real browser at the top, driven from Rust,** by `skein-browser`
  (section 4).
- **The gallery** prints the browser's pages in named states as static
  documents, beside the bundle's stylesheets, for screenshots held to
  the mockups: no server, no script, no network (section 7). It stays
  for the states no journey reaches.
- **Journeys** are walks recorded in a browser on a fresh fake box,
  replayed natively through the shipped host and printed as the
  client's pages, where the look is tuned and kept current by a test
  (`journeys.md`).

## 2. What temper brings to the browser

- **Its crates as they are.** The client root, the forge's side, the
  view and `temper-client-protocol-wire` are step crates under the
  subset, which build for the host and for wasm32 alike; the terminal
  and the agents' tools are not part of the bundle.
- **Its look:** temper's style tokens and a stylesheet over skein's,
  tuned on the journeys' prints (`journeys.md`, section 7; skein's `ui-web.md`, 3.2
  and section 12): 2.5.
- **Its pages,** the client's (`README.md`, 3.5), each at an address of
  its own (2.3); and the forge's parts, from `forge.md`, as the forge's
  side reads them.
- **Signing in:** by the link `temper serve` prints (2.4). The forge as
  people's sign-in provider is temper's own routes (`README.md`,
  section 11).
- **The toolchain:** the standard library for `wasm32-unknown-unknown`,
  skein's bundler and the glue it names (skein's `ui-web.md`, section 4),
  and Chromium for the browser tier.

### 2.1 The shell

- **`temper-client-shell-web`** is a `cdylib` for
  `wasm32-unknown-unknown`, and a library on every target for its host
  and its journeys (`journeys.md`, section 2), ordinary Rust as a shell is (skein's
  `ui.md`, section 2), copied from jig's reference browser shell,
  `jig-client-test-shell-web` (jig's `protocol/browser.md`, 3.1), with
  temper's client root, view and records in place of jig's test
  application's, and none of jig's look (`jig-client-shell-web`'s
  `look`, `jig.css`). Its modules:
  - `lib`: the start (skein's restart part; jig's sign-in part taking
    the link from the address before anything is sent; the small store;
    the page with temper's look; the DOM) and the turn of skein's
    `ui-web.md`, 5.2; and, as jig's reference shell has it, the test
    address `/test/panic`, at which the shell panics at start, for
    skein's `panic` story (4.1);
  - `host`: the client host: temper's client root and view, and the
    view's patch queue, reserved at start at the view's limit on
    patches and drained each turn (skein's `ui-web.md`, 5.2); the root's
    `Leave`, a forge's or an authorization's departure, as skein's
    transport's navigation away, in the turn that asked it (skein's
    `ui-web.md`, section 10); the root's `Tab`, a sign-in's tab
    (`README.md`, 3.5, `ConnectModel`, The sign-in's tab), as skein's
    platform's, the opening before the turn's transport, and a refusal
    given back as a report of that turn; jig's
    mapping under `/jig` and its stream; the root's requests to jig's
    adapter (`jig-client-shell-web`'s `adapter`) and skein's transport,
    and their reports back to the root; and the wake, skein's timer
    set at the earlier of the root's deadline and the instant the
    view's words next change (`README.md`, 5.12); its seam, a report
    given and the duties asked back, a turn's drawing and its deadline,
    the shell's turn and the journeys' replay call alike
    (`journeys.md`, section 2);
  - `journey`: a recording's records and file, its recorder and its
    replay (`journeys.md`, sections 3 and 4); the recorder's keys and
    save are in `lib`, under the `record` feature alone;
  - `records`: the small store's records, `jig-client`'s and a sign-in's
    address (`README.md`, section 7); from the
    adoption form, a pending adoption is among them, a kept
    request whose request `temper-client-protocol-wire` encodes
    (`README.md`, 4.6);
  - `limits`: the client's limits in a browser, derived as the
    terminal's are (`README.md`, section 2), and their sum.
- **The feature `playground`** builds the shell for the playground in
  the browser (`../playground.md`, 6): skein's worker platform where the
  shipped shell holds the browser's (skein's `ui-web.md`, 18), a type the
  feature chooses, the rest of the loop unchanged; the starter, which
  starts the world's worker and takes the printed link from what the
  world prints; the frame; and the failure page with the session's log.
  The shipped bundle holds none of it.
- **It decides nothing.** What a page shows and does is the domain's
  and the view's, step code under the subset; the shell orders the
  platform's work (jig's `protocol/browser.md`, 3.1).
- **The failure page:** past the restart part's limit, the page says
  "The application failed.", in temper's words, since it takes none of
  jig's look; it starts nothing and keeps the small store (jig's
  `protocol/browser.md`, 5.1).
- **The small store** is the browser's session storage, under the
  prefix `temper.`, holding `jig-client`'s records alone, as the
  terminal's file does (`README.md`, section 7); from the adoption form
  a pending adoption is among them, a kept request
  (`README.md`, 4.6). Each load of the page is a client of its own, with
  its own stream (jig's `protocol/browser.md`, section 10). The root can
  step the client several times in one turn: each write the client asks
  for (`Save`, `Erase`) is answered once, after the turn's whole storage
  has finished, and every one is refused if any write was refused.
- **What the terminal says in lines of its own** (`README.md`, 3.4 and
  6.6): another party's kept work and kept work dropped have no place
  in the browser, whose small store lives as long as its tab, of one
  person (section 5). A service's credential is never shown: temper's
  client makes no service (`README.md`, section 11).

The person's zone is one of the shell's inputs: `Date`'s offset, in minutes
east of universal time, read at start and recorded in a journey's facts, so a
replay says the same clocks (`README.md`, 5.12).

### 2.2 Serving

- **The routes of the clients' listener,** temper's, in this order,
  over jig's mapping (jig's `protocol/browser.md`, section 12) and
  skein's serving of a bundle (skein's `ui-web.md`, 4.4):
  1. what jig's default mapping claims, under `/jig`, to jig's clients'
     component: every request there but a `GET` of the printed link's
     route, which the mapping leaves to the page, answered with the
     page's policy and `Referrer-Policy: no-referrer` (jig's
     `protocol/browser.md`, 12.1 and 12.2);
  2. with no bundle, the configuration naming none (`null`) and the
     binary built without its own (2.6), a `GET` of any other path but
     under `/bundle/` and `/robots.txt`, the printed link's route among
     them, answers `200` temper's page for no bundle, every other
     request not found, as `README.md`, section 8 says. The page is
     fixed HTML in the binary, under the page's policy, with no script,
     style or picture: the level-1 heading "temper", then "This temper
     was built without its browser client.", "Open the link in a
     terminal with temper open DIRECTORY LINK." and "To serve the
     browser, build temper with the wasm32-unknown-unknown target, or
     name a bundle's directory in application.clients.bundle, and
     restart temper." (`../protocol/setup.md`, 8.2);
  3. a request other than `GET`: `405`;
  4. `/bundle/<digest>/…`: the bundle's files, immutable; any other path
     under `/bundle/`: not found;
  5. `/robots.txt`, the bundle's one root file (skein's `ui-web.md`,
     3.3, step 5; jig's `protocol/browser.md`, section 12), disallowing
     every path;
  6. any other `GET`, the printed link's route among them: the page,
     with its ETag and `no-cache`, and `304` on a matching
     `If-None-Match` (skein's `ui-web.md`, 4.4).
- **The page's policy,** on every answer temper gives: jig's reference
  policy as it stands (jig's `protocol/browser.md`, 12.1): `default-src 'none';
  script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src
  'self'; font-src 'self'; connect-src 'self'; base-uri 'none';
  form-action 'none'; frame-ancestors 'none'`, with `nosniff`,
  `Cross-Origin-Opener-Policy: same-origin`,
  `Cross-Origin-Resource-Policy: same-origin` and `Referrer-Policy:
  no-referrer`. skein-http's own refusals, `400`, `413` and `431`, given
  before temper sees the request, carry none: they hold no page to frame
  or sniff. temper adds no origin: no font, picture or script from
  elsewhere. The mockups' optional web font is not used; the look takes
  the system's monospaced font.
- **jig's example guards, as they are** (`README.md`, section 8): the
  Host allow-list holding the listener's own address; a cookie without
  jig's browser header refused.
- **Ordinary calls wait for jig's mapping's room** (jig's
  `protocol/client.md`, 17.4). Each waiting head stays under the listener's
  connection bound and stall deadline, with no body read until admitted;
  a closed connection drops its head. The small profile's six calls,
  four ordinary exchanges beside its two streams, therefore serve one
  page's concurrent requests in turn. Public reads retain their separate busy response (jig's
  `protocol/client.md`, 17.5); stopping refuses waiting ordinary calls.
- **The bundle** is skein's bundler's output (skein's `ui-web.md`,
  section 4): the shell's module, skein's stylesheet and temper's
  (2.5). temper's build embeds it in the binary (2.6);
  `scripts/bundle.sh` builds one apart, from the module built for
  wasm32, into a directory it prints. `application.clients.bundle` is
  `null` for the binary's own, or names a directory, whose bundle is
  served in place of the binary's (`../protocol/configuration.md`,
  section 3). `temper serve` makes it, at start, whole, into memory,
  within the limits `clients.bundle_files` and `clients.bundle_bytes`
  (`../protocol/configuration.md`, section 7), and refuses the start
  past them, or when it is no bundle (its page and its digest's
  directory), by one code for both: a directory's files read, or the
  binary's, already in memory, copied (2.6). A new bundle is served
  after a restart; a tab on the old one is answered not found for its
  files, and a reload loads the new (skein's `ui-web.md`, 4.4). A page
  from the binary's own bundle always speaks its engine's records, so a
  client out of date (`README.md`, 5.1) comes only from a binary
  replaced under an open tab, or from a directory configured.

### 2.3 Addresses

- **Each page's address is a path,** by temper's address codec, a
  module of `temper-client-domain`, step code, total and bounded:

  | Page | Path |
  |---|---|
  | `Inbox { filter }` | `/`, and its filter's query |
  | `Home { project }` | `/projects/<project>` |
  | `Changes { project }` | `/projects/<project>/changes` |
  | `Settings { project }` | `/projects/<project>/settings` |
  | `Repositories { project }` | `/projects/<project>/settings/repositories` |
  | `Builders { project }` | `/projects/<project>/settings/builders` |
  | `Configure { project }` | `/projects/<project>/settings/configure` |
  | `Goals { project, filter }` | `/projects/<project>/goals`, and its filter's query |
  | `Chats { filter }` | `/chats`, and its filter's query |
  | `NewChat { draft }` | `/chats/new`, and `?draft=1` with a draft |
  | `Accounts` | `/accounts` |
  | `Chat { task }` | `/chats/<task>` |
  | `Task { task }` | `/tasks/<task>` |
  | `Adopt { project }` | `/projects/<project>/adopt` |
  | `Welcome { step }` | `/welcome`, and its step's query |
  | `Forges` | `/forges` |
  | `ConnectForge { forge }` | `/forges/connect`, and its choice's query |
  | `ConnectModel { way }` | `/accounts/connect`, and its way's query |
  | `Authorize { query }` | `/oauth/authorize`, and the query as the address carried it, within jig's bound on an authorization's query (`../protocol/sign-in.md`, 9.3) |

  `<task>` and `<project>` are the task's and the project's numbers in
  decimal. A later page's path comes with it, in this shape
  (`/projects/<project>/board`). The
  public pages' paths, under `/public`, are section 8's.
- **The inbox's filter is its query:** the fields it sets, in one
  order, `?project=<project>&kind=<kind>&goal=<goal>`, and none with
  every field unset, so that `/` is the inbox unfiltered and
  `/?kind=question&goal=9` one filtered by kind and goal. `<project>`
  and `<goal>` are numbers in decimal, as a path's; `<kind>` is one of
  `proposal`, `question`, `held` and `person-task`: the kind's button
  word (`README.md`, 3.5's inbox), in lower case, its space a hyphen.
  The codec reads the fields between `&`, each `name=value`, and drops
  one it doesn't know, one whose value doesn't read as its field's
  (`kind=reply`, `project=07`, a number past its type), and one named
  again after a first that read; the rest set the filter. So a stale
  or edited link opens the inbox less filtered, never refused, and its
  address is replaced with the printed one. It reads numbers, not the
  person's projects: a project the person isn't in shows an empty
  inbox, which its unfiltered address undoes. The inbox's filter, the chats
  list's (below), the welcome's step,
  `ConnectForge`'s choice, `ConnectModel`'s way (below) and
  `Authorize`'s query, kept as it came and never printed again, alone
  are in an address, and another page's query is dropped, `/chats/7?x=1`
  replaced with `/chats/7`.
- **The chats list's filter is its query,**
  as the inbox's (the user's pick, 10-23): none for "All", so that
  `/chats` is the list whole; `?state=live` for "Live",
  `?state=drafts` for "Drafts" and `?state=closed` for "Closed". A `state` that
  doesn't read as one of the page's, or a field of another name, is dropped,
  and the address replaced with the printed one. `/tasks/9?state=live` reads as
  `/tasks/9`. A filter chosen is a move (below).
- **The goals' filter is its query,** as the chats list's: `/projects/<project>/goals`
  is the list whole; `?state=needs` for "Needs you", `?state=live` for
  "Under way" and `?state=ended` for "Ended"; any other value reads as the
  list whole and prints without it. Choosing one is a move.
- **The welcome's step is its query:** none for its first step,
  Welcome, so that `/welcome` is the welcome and decides by the store's
  state; `?at=welcome` for that step asked by its link, which stays on it
  unless the state is whole; `?at=forge`, `?at=model`
  and `?at=ready` for the others (`README.md`, 3.5, The welcome). As
  the inbox's filter, an `at` that doesn't read as a step, or a field
  of another name, is dropped, and the address replaced with the
  printed one: `/welcome?at=done` opens the welcome at `/welcome`.
  Its Forge step adds its choice, as `ConnectForge`'s (below):
  `/welcome?at=forge&forge=3`; its Model step its way, as
  `ConnectModel`'s: `/welcome?at=model&provider=1&way=key`.
- **`ConnectForge`'s choice is its query:** none at Choose, so that
  `/forges/connect` is the flow's first step; `?forge=github`,
  `?forge=owned` or `?forge=managed`, a kind to make, GitHub, Forgejo
  on this machine or Your Forgejo; or `?forge=N`, N a kept forge's
  number in decimal (`forge.md`, 5.4). A value that doesn't read as
  one, or a field of another name, is dropped, and the address
  replaced with the printed one. It reads a number, not the
  deployment's forges: a number that names no forge shows Choose.
- **`ConnectModel`'s way is its query:** none at Choose, so that
  `/accounts/connect` is the flow's first step; `?provider=N&way=W`,
  N an LLM provider's number in decimal and W `sign-in`, `program` or
  `key`, in that order, for the way of that kind its provider offers
  (`README.md`, 3.5, `ConnectModel`). The two go together: one without
  the other, a value that doesn't read as its field's, or a field of
  another name, is dropped, and the address replaced with the printed
  one. It reads a number and a kind, not the configured providers: a
  way they do not offer shows Choose. A sign-in's way may add its
  sign-in's source, `&source=S`, S a source's number in decimal, once the
  sign-in is answered, its history entry replaced (`README.md`, 3.5,
  `ConnectModel`, A reload while it waits):
  `/accounts/connect?provider=1&way=sign-in&source=7`. A source on
  another way, or one that doesn't read as a number, is dropped as
  above.
- **A filter chosen is a move** (below), with its history entry: a
  link's, at the page's top with the focus on its heading, and back
  returns to the filter before. Like any move it folds the open
  disclosures (2.5). Needs you has no filter menus; its filters remain
  address fields (`README.md`, 3.5). A move
  that stays on the chats list, between `Chats` and `NewChat`, or on one
  goal's page, changing its filter alone, keeps the page: its composers' words, the
  closed chats and ended delegates it read, which it reads not again;
  the inbox's filter, a query the engine answers, reads its page anew.
- **A path that is no page** opens the inbox, and the address is
  replaced with `/`. The codec never prints a path under `/jig`,
  `/bundle/`, `/test/` or `/forges/N/github/`, temper's routes for
  GitHub's steps (`forge.md`, 9.1), nor `/robots.txt`.
- **Moving** is skein's (skein's `ui-web.md`, section 9): a link pressed
  is a move within the client, with a new history entry; back and
  forward are moves too, each entry keeping its scroll; a reload, or an
  address typed or opened, is a new client at that address (jig's
  `protocol/browser.md`, section 10). That a client opens at the inbox
  (`README.md`, 3.4) is the terminal's: in a browser the address is the
  browser's, and the client keeps no record of its own.
- **A heading drawn after its page** (the answer of 2026-11-06 to the
  `ui-composer-focus` task `composer-focus`): a new entry whose level-1 heading
  isn't drawn yet, as a chat's isn't until its task is read, has the focus on the
  main (skein's `ui-web.md`, 7). When the heading is drawn, the shell moves the focus from the
  main to it, unless the focus has moved meanwhile: a focus placed since keeps its
  place, even back on the main, and a focus request ends the wait.

### 2.4 Signing in

- **The printed link,**
  `http://HOST:PORT/jig/v1/sign-in/link?secret=SECRET`, opened in a
  browser on the engine's machine (`README.md`, section 7): temper's
  server answers its `GET` with the page (2.2); jig's sign-in part takes
  the secret and replaces the address with `/welcome`, temper's home for
  a sign-in, before anything is sent; the client signs in with it
  (`SignIn { with: Link { secret, held: None } }`), and the mapping's
  answer sets its cookie, which the client never sees (jig's
  `protocol/browser.md`, section 11). The browser sends its cookie with
  the link's sign-in, as with any exchange: a person still signed in
  keeps their sign-in, its number and expiry, and the answer sets no
  cookie, so the browser's stays as it was (jig's `domain/people.md`,
  section 3); a cookie the engine does not know, or one ended or
  expired, signs in afresh, with a new cookie. The welcome follows,
  which moves on to the chats page once the first run is over, and to
  the inbox, drawing nothing of itself, on a deployment with no welcome
  (`README.md`, 3.5, The welcome).
- **A reload, or another tab,** signs in with `Transport`, by the cookie
  (jig's `client/domain/client.md`, section 12).
- **Signed out,** by "Sign out" (`README.md`, 6.6; jig's sign-out clears
  the cookie) or by a link spent or expired: the main is the view's
  signed-out paragraph (`README.md`, 5.1), in the browser's words:
  "Signed out. To sign in, open a sign-in link `temper serve` printed."
  The view takes its shell's words at construction; the terminal's stay
  "Signed out." with its own line (`README.md`, 6.6).
  With a way of signing in configured, the main is the sign-in page
  instead: temper's mark, "Sign in" and "Continue with Google", the ways
  read from the page's `temper-sign-in` meta at start
  (`../protocol/sign-in.md`, section 7); the link leaves for Google's
  sign-in and comes back to the page's address. Where the deployment
  has public projects, the signed-out main and the sign-in page add a
  link, "Read the public projects", to `/public` (section 8).
- **A link's sign-in not taken,** answered full, busy or not ready: the
  link is not spent, and the main says why and that the link stays
  usable until it expires (`README.md`, 5.1), full with the deployment's
  room, its earliest end and the link's own. The retry is the person's: opening the
  printed link again sends its secret again, as the first open did, and
  signs in once there is room; opened past its expiry, it is refused,
  and `temper serve` prints a fresh one. The page never retries by
  itself, and keeps no secret: the address already says `/welcome`.
- **One person, on loopback,** as `README.md`, section 7 says. A browser
  on another machine is open (`README.md`, section 11).

### 2.5 The look

- **temper's style tokens,** under the owner's prefix `temper-`, on the
  nodes the mockups style: the frame's banner, navigation and main; the
  cards; the conversation's turns; the composers. The view sets them
  (`README.md`, 5.1); the terminal prints none.
- **The bundle's stylesheets:** skein's `skein-ui.css`, then
  `temper.css`, temper's look. It takes no rules from the
  mockups' terminal sheets (`terminal-chat.css` and `terminal-pages.css`
  over `style.css`): it is tuned on the journeys' prints
  (`journeys.md`, section 7), and the mockups are reconciled to it
  (`mockups/README.md`, 1); they are not shipped. What follows here
  says what it draws, and is the gallery's for the states no journey reaches (section 7).
  These are the bundle's three
  stylesheets; `jig.css` is not in it. `temper.css` holds the mockups'
  palette as variables under the prefix, their monospaced stack, compact
  top navigation and quiet borders. DejaVu Sans Bold, for the wordmark
  alone, as the mockups draw it, is self-hosted in the bundle's files,
  subset to Latin, with its licence beside it; the page fetches nothing
  from another host. The mockups' icons are decorative background
  images drawn by `temper.css` from the bundle's SVG files, each in its
  own colour (the brand mark the accent's, the menu's ☰ its summary's,
  the send arrow white): never masks, which a browser fetches under
  CORS and so refuses to a page opened as a file, as the journeys'
  prints are (`journeys.md`, 5.4).
  Every control keeps its words as its name. Nothing a person must know
  is said by colour or icon alone (skein's `ui.md`, section 8): a status
  is its word, and its colour only repeats it.
- **The main's scale:** the page's main is drawn at 14/12 of the
  mockups' sheets' lengths and of this section's, as the mockups' own
  zoom draws theirs: 12 px text reads as 14 px, and its spacing and
  widths grow with it. `temper.css` and `task-pages.css` write each
  length of a rule for a part inside main at that scale, to two
  decimals, rather than zoom it (the user, 2026-10-26). What a zoom
  would leave stays: hairlines, the media queries' widths, and the
  banner with its wordmark, at the mockups' own. What a zoom scales
  but no length of temper's names is written out at the scale too:
  main's own text size, which its parts inherit with their `em`
  spacing; a native control's own size; skein's sheet's lengths inside
  main (its focus ring, its visually hidden boxes); and a viewport
  length, which a zoom grows like any other. Dialogs are drawn inside
  main, and scale with it. Confirmation sheets stand under `#app` at their
  own scale (`README.md`, 3.6).
- **Confirmation sheets:** `dialog.temper-confirm`, 460 px wide at most,
  with 24 px by 26 px padding, 14 px corners and a native `::backdrop`.
  The title is 19 px; the lead 14.5 px; impacts 14 px under a hairline.
  Keep precedes the named primary act in a separate group. An uncertain
  release's choices have a wrapping group of their own, with the same
  secondary buttons; those and the acts keep 44 px phone targets.
  Progress and refusal stay above the acts. A chat's end line, whichever
  its words (`README.md`, 3.5), is 12.5 px between two hairlines, and the
  bar has no clock.
- **Connection flows:** standalone connection cards use 5 px corners;
  the welcome's steps have a look of their own (The welcome's look,
  below). The
  current step is ink, its bar and its bold label, never the accent,
  which beside the done steps' green reads as a step gone wrong; the
  accent is for what to press. The view assigns `FLOW_PRIMARY` to
  filled terracotta acts, `FLOW_SECONDARY` to neutral small buttons and
  `FLOW_QUIET` to quiet terracotta links. Connect a forge's choices use
  three equal columns and model choices two, stacking below 760 px. Choice cards keep
  small headings, ink body text, muted metadata and bottom-aligned acts;
  details share their padding. `FLOW_PAIR` groups organization/repository
  and the half-width project field, stacking below 760 px. Setup field
  labels are small and muted, and setup notes are a muted bulleted list.
  Setup and Ready repository subtitles are muted and tight beneath their
  headings. Model key and program Details descriptions are muted. Model
  Ready uses its own definition list: 11 px labels, right-aligned 600-weight
  muted values, 6 px row padding and hairline separators. Its action row
  wraps with an 8 px gap, retaining Continue with ChatGPT (README, 5.5).
  Those list and action styles do not select Accounts or Forges rows.
  Progress draws ✓, ●, ✕ and ○ for done,
  current, failed and pending; status words remain accessible in the DOM,
  visually hidden.
  A managed Forgejo's admin Details' first-repository fields sit in an
  outlined fieldset with "Your first repository" as its legend. The view's
  named Region and heading map to fieldset and legend through `look::ELEMENTS`,
  shared by the browser and gallery. Claude's program Details uses the same
  small neutral act and its existing program-name input, labeled "Name".
  The welcome's Model ways sit directly under its lead, without the
  standalone page's holder choice or outer Choose card. Inside a way,
  the foot's Back and a quiet "All ways" return to Model Choose. On the
  welcome a way is one panel, its field the panel's width; standalone, the
  API key details keep the
  full-width card with contents limited to 420 px; failure reasons in flow details are red.
- **The welcome's title,** "Welcome · first run · temper" on every step,
  whatever its level-1 heading, as Home's is "Home · temper"; the shell's
  turn, the journeys' print and the gallery's take it from the same rule,
  `main` styled `temper-welcome`.
- **The welcome's states' parts,** under `temper.css`'s marker for the
  welcome, take the greenfield mock's rules at the welcome's tokens: its
  `.forge-status`, `.spin`, `.acts`, `.panel`, `.away-note`, `.numbered`,
  `.field-v`, `.cmd` and `.checklist .current .mark`.
- **The welcome's look** (the user's greenfield pass, 2026-11-04;
  `README.md`, 3.5 and 5.6), at its own scale, in a block of its own in
  `temper.css`, every rule scoped to the welcome's frame
  (`#app:has(> .temper-welcome)`), so that no other page changes: IBM
  Plex Sans, bundled under `files/fonts/` with its licence (SIL OFL
  1.1), 15 px on paper (#f7f7f5) in ink (#18181b), ✦ before temper's
  name in the signal colour (#c2410c). The banner is a top bar, 22 px
  by 32 px, the brand, temper box's line in 14 px muted words, its
  "temper box" ink and bold, and the steps at its right in 14.5 px
  words, each a link holding a numbered 24 px circle and its label, the
  current one filled ink, a done one ticked green, joined by short
  hairlines, hovered ink; below 760 px only the current label shows; at 1100 px and
  under, temper box's line takes its own row under them. The main is
  one 680 px column: the tagline (14 px) above the level-1 heading
  (32 px, 500),
  the lead (17 px), then the choices as cards side by side, white on
  paper, 12 px corners, their acts at their foot so that their
  buttons line up; the default card, the one holding the
  step's primary act, is outlined in ink. A primary act is a filled
  ink button, a secondary one a white bordered button, a quiet one
  underlined words. The muted line under the cards is 14.5 px; the
  foot, "‹ Back" at the left and the playground alternative at the
  right, is muted underlined words. Model's forge region follows the
  choices before the foot: from 1280 px, a 272 px pane to the right of
  the column, its top at the heading's, with a hairline on its left;
  narrower, in the column with a hairline above. Its line stacks a quiet
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
  2026-11-04). Ready's pane is Model's forge region, its lines 18 px apart:
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
  the terminal sheets (`README.md`, 5.9 and 5.10): a plan row's spend as a
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
  faint on no fill. A proposal of settings' card takes the conversation's
  `.proposal`, with `.proposal.superseded` at `opacity: .5` and `.proposal-state.ok`.
  The mock's `.proposal-split` is the review page's (`README.md`, 5.13).
  `Forges` and `Accounts` take the same values on their rows (`README.md`, 3.5,
  *The rows*): `.lines`' hairline and padding on a `details`, the chevron of
  `.switcher > summary::after` and, opened, `.setting`, `.btn` and its kinds and
  `.text`. A control a row shares with `ConnectForge` or the welcome takes them
  by a rule scoped under the row, so those pages keep their look. `Builders`
  adds `.empty`, and `Adopt` takes the `.col`, `.text` and `.btn`. No rule
  under the `--temper-*` tokens reaches these pages: the old rows' and pages'
  rules go, rather than being overridden. A flow's path takes `Configure`'s
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
  at the foot, four links, the page leaving it room. The link's state
  shows at the rail's foot, with its dot, while it is not live. The person's
  menu goes with the foot, so the `Accounts` tab's row with "Sign out", which
  `temper.css` hides over 760 px, stands at that width alone (`README.md`, 3.5
  and 5.3).
- **The page covers the display,** `viewport-fit=cover`, as the greenfield
  mock's does, wherever a page's head is printed: the bundler's page
  (skein's `ui-web.md`, 4.3), the playground's, the setup pages', and the
  gallery's and journeys' prints. So what sits at the foot keeps off a
  phone's home bar by `env(safe-area-inset-bottom)`, which is 0 on a display
  without one and so measures as the mock draws it, at 760 px and under: the
  tab bar's padding, a chat's foot, and the toast, 88 px over it, clear of the
  tabs.
- **What a phone presses,** at 760 px and under: a control scrolled into view
  stops clear of the tab bar, and the connection flows' primaries, Settings'
  and a dialog's stand at least 44 px tall. Every other control keeps what the
  mock draws.
- **The navigation** of a page that is no project's sits in the banner
  after the projects, as in the mockups. At phone width, 760 px and under as in the mockups, the
  banner keeps temper's name and the view's "Menu" alone (`README.md`,
  5.1): its summary a ☰, an image drawn over its word, and its
  content the project's switcher, its first row, the navigation and
  the corner (the link's state, the person, "Sign out") in a panel
  that opens under the banner's end, as `terminal-chat.css` draws the
  mockups' menu. Theirs
  holds the navigation alone, which leaves a phone no way to choose a
  project or sign out; temper's holds the corner and the projects too.
  Wider, `temper.css` draws the menu's content in the banner whether
  it is open or not, and its summary not at all, where the browser
  lets a closed disclosure's content be drawn (`::details-content`);
  elsewhere the ☰ is drawn at every width. Choosing an entry folds the
  menu: skein's page folds its open disclosures when its address moves
  (skein's `ui-web.md`, section 7).
- **The banner is one quiet line,** on the canvas with no fill of its
  own: its words on the wordmark's baseline, the navigation plain
  words, the inactive ones `--temper-faint`, the current one ink. The
  switcher draws the open project's name with a chevron; wider than
  phone width its list drops under it, the open one ticked. "Set up
  temper →" is the banner's one coloured control, outlined in the
  accent, its arrow drawn by the sheet (`README.md`, 5.1).
- **The Chats page and every chat are the greenfield mock's**
  (`README.md`, 3.5): each selector the document names is the mock's rule
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
- **A chat's document** (`README.md`, 3.5) is `configure-chat-draft`'s rules, in
  `temper.css` under the same marker: `temper-mode-chat`, `-split` and `-draft` on
  the session, `.temper-doc`, `.temper-doc-inner`, `.temper-doc-path`,
  `.temper-doc-sec`, `.temper-doc-changed`, `.temper-doc-note`, `.temper-seg`, and
  the rail's strip with `.temper-rail-unfold` and `files/menu.svg`. What those
  name is drawn, and the look of its flow's tokens isn't, since nothing draws a
  flow yet. The sheet to start a goal is the mock's `dialog.sheet`, `.option`,
  `.field` and `.text`, and `Ctrl+\` steps through the modes.
- **temper's own way** (`README.md`, 3.5) is `configure-temper-own-way`'s rules, in
  `temper.css` under `/* ==== ui: settings ==== */`: on the page
  `.temper-starter-head` and `.temper-starter-own` with its parts; in the pane, beside
  a chat's document's, `.temper-doc-flow`, which holds the
  picture's numbered list and no flow, with `.temper-doc-sub` and `.temper-doc-muted`
  under the chats' marker.
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
- **A page target draws no ring:** anything given `tabindex=-1` only to take
  focus (`main`, a heading, a document's section) draws no outline, by
  `skein-ui.css` (skein's `ui-web.md`, 12), and `temper.css` says so again of
  `main` and a level-1 heading; a control keeps its ring.
- **A scrolling pane holds its hidden parts:** a chat's log and its draft pane are
  positioned, so a visually hidden absolute part (a who-line) is clipped by the
  pane's overflow and does not lengthen the page: a long chat's document never
  exceeds its window.
- **Light only:** the mockups' palette is light, and `temper.css` sets
  `color-scheme: light`, where `skein-ui.css` follows the system's dark
  preference. A dark palette is open (section 5).
- **The composers** (`NewChat`'s and a chat's) are text areas with
  skein-ui's `SubmitsOnEnter`: Enter sends, Shift+Enter is a new line
  (skein's `ui-web.md`, section 7), which the view allows in a
  composer's words, counted within its bound (`README.md`, 5.1). A
  chat's shows its placeholder, "Reply to T6", while it is empty, as
  `placeholder`, its name staying "Message", and `NewChat`'s "What do you
  want done?", its name too (`README.md`, 3.5). They
  grow with their words and shrink after sending by `temper.css` alone
  (`field-sizing: content`, from one line to a bound), with no
  component and no script of temper's; a browser without it shows an
  area of fixed height that scrolls.
- **A run's live text** is a patch like any other: the chat's words
  being written (`README.md`, 3.5) change in place;
  skein's page keeps focus, caret and scroll across it, and the
  conversation, a log, keeps its end in view while the person has not
  scrolled away (skein's `ui-web.md`, section 7).

### 2.6 The bundle in the binary

The `temper` binary holds its bundle, so that `temper box` and `temper
serve` serve the browser with nothing beside the binary, and `cargo
build` in a checkout makes that binary. It is a person's convenience:
the machine's builds keep building the way that serves their work,
without it.

- **One file.** The binary holds the bundle: the shell's module, its
  glue, and the stylesheets and files skein's bundler puts beside them
  (2.2, 2.5). Running it is the whole install: no directory beside it,
  no `scripts/bundle.sh` step.
- **A plain `cargo build` makes it.** `cargo build`, then
  `./target/debug/temper box`, serves the browser; so do `cargo build
  --release` and `cargo install --git` of temper's repository.
- **A crate of its own embeds it,** `temper-client-bundle`, ordinary
  Rust on the `binary` lint profile (`tools/lints/design.md`, 2.3), which
  `temper` depends on. Its library is data: the bundle's files, each its path
  from the bundle's root (`index.html`, then `<digest>/…`) and its
  bytes by `include_bytes!`, in a directory read's order, and the
  digest, both written by its build script into `OUT_DIR`. A build
  without the embed has no files and no digest. The include sits in a
  small crate, not in `temper`'s, because a build script that runs
  makes its package build again, and `temper`'s is the slowest to
  build; `main.rs` gets no package of its own, since `temper` builds
  again after a client edit anyway (the terminal shell shares the
  client's crates), and its integration tests run the binary by
  `CARGO_BIN_EXE_temper`.
- **Its build script,** unless `TEMPER_BUNDLE` is `none`, takes
  `scripts/bundle.sh`'s steps with the cargo running it (`CARGO`), from
  the workspace's root, into the nested build's directory (below):
  1. `build --locked --target wasm32-unknown-unknown -p
     temper-client-shell-web`, with the module's profile (below);
  2. `build --locked --release -p skein-ui-bundle`: the bundler,
     always optimized, as skein's `ui-web.md`, 4.1 asks of a tool;
  3. the bundler, with `scripts/bundle.sh`'s inputs (the name, the
     prefix `/bundle/`, `temper.css`, `task-pages.css`, the shell's
     `files/` and the noscript words), into its `OUT_DIR`;
  4. the list of the bundle's files, written into `OUT_DIR` for the
     library's include.
  It has no build dependencies: the bundler is built by the nested
  cargo and run, never compiled into the script, so a build without the
  embed compiles a script of `std` alone, and nothing of the bundler is
  ever linked into `temper` (skein's `ui.md`: "never linked into a
  service or a module"). One `Cargo.lock` still locks the bundler's
  `wasm-bindgen-cli-support` with the module's macros (skein's
  `ui-web.md`, 4.1). The script's logic (the dep-info's read, the
  nested commands, the bundle's directory read into its list) is plain
  functions, which the crate's tests take on samples.
- **The nested build's directory** is `<target>/bundle/`, `<target>`
  the outer build's, found as the nearest ancestor of `OUT_DIR` holding
  cargo's `CACHEDIR.TAG`. The outer cargo holds its own directory's
  lock while build scripts run, so a nested build there would wait on
  it forever. It is one directory, not one under each `OUT_DIR`, so
  that a check, a clippy run, a build and a test build of `temper`
  reuse one module and one bundler, and a build after a change is
  incremental. Under `cargo clippy`, cargo builds a member's build
  script apart, so the first clippy run and the first build after a
  change each bundle once; the second finds the module built. A build
  without the embed never makes the directory.
- **None of the outer build's flags or wrappers:** the nested cargo
  runs without `CARGO_ENCODED_RUSTFLAGS` and `RUSTFLAGS` (the host's
  flags, not wasm32's), `RUSTC_WORKSPACE_WRAPPER` and `CLIPPY_ARGS`
  (which `cargo clippy` sets, and which would make a module of another
  fingerprint, built again at each switch between clippy and build),
  with `CARGO_TARGET_DIR` set anew. It keeps the jobserver
  (`CARGO_MAKEFLAGS`), so that its jobs count within the outer build's.
  Its stdout goes to the script's stderr, since cargo reads the
  script's stdout.
- **Rebuilt only when what it is made from changes,** each by
  `cargo:rerun-if-changed`: each source file the nested build's dep-info
  names (the `.d` cargo writes beside the module: every path package
  the module is built from, temper's client crates and, when a local
  jig or skein is patched in, theirs); each of those packages'
  `Cargo.toml`, the workspace's `Cargo.toml` and `Cargo.lock`, which
  pins the git dependencies; the shell's stylesheets and its `files/`;
  and `TEMPER_BUNDLE`, by `cargo:rerun-if-env-changed`. The first run
  has no dep-info yet, and runs regardless. An edit elsewhere never
  runs the script, and a build that does not include `temper` (a
  domain's tests, the client crates' own) never runs it.
- **The module's profile:** a dev build embeds a dev module, as
  `scripts/bundle.sh` makes; a release build embeds one built with
  `web`, the profile skein's `ui-web.md`, 4.1 names for an
  application's release module, which temper's workspace defines as
  skein's does. The script maps cargo's `PROFILE`, `debug` or
  `release`, to `dev` and `web`.
- **`TEMPER_BUNDLE`:** unset or `embed`, the embed; `none`, no nested
  build and no bundle; any other value fails the build, naming the two.
  `embed` is for a build whose environment says `none` and wants the
  browser this once. The machine's builds set `none`: the gate's and
  development builds, a machine without the
  target, an editor's own (rust-analyzer runs build scripts at its
  start). Such a binary serves the page for no bundle (2.2).
- **The embed needs the wasm32 target.** Before any nested build, the
  script asks rustc for the target's library directory
  (`--print target-libdir`); without it, the build fails with one
  message: `temper-client-bundle: the browser's bundle needs the
  wasm32-unknown-unknown target: add it (rustup target add
  wasm32-unknown-unknown), or build without the browser
  (TEMPER_BUNDLE=none)`. Never a binary without its browser in
  silence. temper ships no `rust-toolchain.toml` naming the target: it
  would set the channel for every rustup user, which temper does not
  set (`rust-version` alone).
- **Offline:** the nested build may download crates only wasm32 uses,
  as any first build of the target does; an offline build runs `cargo
  fetch` first, which fetches for every target.
- **Served as a directory's is.** At start, `application.clients.bundle`
  `null` takes the binary's bundle: its files, already in the binary,
  are made into `temper-service`'s `Bundle` (its page, its digest, its
  files by path with their types from the closed list, its `ETag`) by
  the same code as a directory's read, given files in memory in place
  of a directory, so the same checks hold: the closed list, each path's
  bound, one digest directory, and `clients.bundle_files` and
  `clients.bundle_bytes`, a start past them refused naming the limit
  (`application.clients.bundle: the binary's bundle: <path>: <why>`).
  The copy is held as a read one is, within what the profile's worst
  case already counts for the bundle, so the memory account does not
  change. `temper-service` adopts a bundle, wherever the binary got it.
  A directory configured is read as before, and the binary's is unused.
- **The fake-backend bundle is provisioned separately.** Build it with
  scripts/bundle.sh --loopback DIR, then start temper box --fake-backends
  --bundle DIR even with an empty configuration (../protocol/configuration.md,
  12.1). An explicit --bundle overrides source application.clients.bundle;
  otherwise that configured directory is used.
  The ordinary embedded bundle remains loopback-free and is not used for
  that box. Missing provision is a startup refusal explaining the bundle
  command and --bundle. Startup never builds or downloads the bundle.
  The second-embed prototype added 8,974,112 bytes in the measured dev build,
  exceeding its 5 MB limit; the provisioned path keeps that cost outside
  ordinary builds. TEMPER_BUNDLE=none runs no nested browser build.
- **`scripts/bundle.sh` stays,** for a bundle built apart: the browser
  tier's, which turns on `loopback-navigation` (4.3), and a
  developer's. The ordinary binary bundle never enables it. `scripts/install.sh PREFIX`
  installs the binary alone, built with `--release --locked`, and
  prints its path.
- **Tested in both builds.** Every suite passes with and without the
  embed, and the gate runs them under `none`. The build script's logic
  is tested on samples in its crate; the in-memory read on fixture
  files beside the directory's read, the same `Bundle` and the same
  refusals; and a start with `bundle` `null` expects what its build
  holds, which `temper`'s crate says (`temper::bundle::OWN`, the
  binary's digest or none): the page for no bundle under `none`,
  otherwise the bundle's page at `/` and its files under
  `/bundle/<digest>/` (`../domain/testing.md`, 2.6). The page for no
  bundle stays proved where it is built, in `temper-service`. The embed
  itself is a person's build, which fails loudly when its steps do; a
  change to it checks it with `TEMPER_BUNDLE=embed cargo nextest
  run -p temper`, which the gate does not run.

## 4. The browser tier

- **The driver is skein's.** `skein-browser` (skein's `browser.md`) runs
  headless Chromium as a child and drives it over the DevTools protocol
  on a pipe, from the loop beside the engine.
- **The person's browser face is skein's** (`skein-fake-person-browser`;
  skein's `ui-web.md`, 14.2): the same queries and acts as on the tree,
  made over Chromium's accessibility tree, and the browser's own: the
  current address, read from the navigation history, and keys pressed
  with Shift.
- **What it runs against:** `temper serve`, a child of the harness's
  loop, on a scratch directory, from a configuration the test writes
  with the bundle named: no Forgejo, no runner. 4.1's
  stories need no answer of the forge's or the LLM's, so its forge and
  its LLM endpoint are unreached, on closed loopback ports, as jig's
  tier serves (jig's `protocol/browser.md`, section 15). The fake
  forge, hosted on loopback in that loop by skein's forge peer
  (`../domain/testing.md`, 4.2), and skein's fake LLM peer, as the
  process tier has them (`../domain/testing.md`, 2.6), join with the
  first story that needs them (4.2). 4.4's test has the forge peer
  serve only the fake's GitHub `web`. skein's fake issuer joins for
  `ConnectModel`'s round trip alone (4.2).
- **Only what a browser alone shows.** temper's stories (what a page
  shows, what an act sends, a chat answered, a proposal accepted, a
  repository adopted, a run's live text) are the joint world's
  (`tests/client`; `../domain/testing.md`, 6.3) and the real loop's, through
  the terminal shell (`../domain/testing.md`, 2.4); they never run again in a
  browser. A story is in this tier only when no world can show it: the
  bundle, the browser's platform (the address, history, the cookie,
  keys) or an attack that needs a browser. Each names what only the
  browser shows. What skein's browser parts prove for every application
  (the mapping of roles, a patch keeping focus, caret and scroll, the
  store, the bundler) and what jig's prove for every client of jig's
  (jig's `protocol/browser.md`, section 15) are not proved again.
- **Every story checks the policy too:** skein's harness fails a test
  on a policy violation, an exception or a native dialog (skein's
  `ui-web.md`, 14.2).

### 4.1 The first milestone's stories

1. **Signing in by the printed link:** the welcome shows, at
   `/welcome`, or, where the first run is over, the chats, the
   address reading `/chats` once they do (2.4); a reload on a chat's
   address shows that chat, signed in by the cookie; a link spent shows
   the browser's signed-out paragraph (2.4). That the secret leaves the
   address before the first request is held by the shell's start order
   (2.1), and is not tested.
2. **Addresses:** skein's `back_and_forward` (skein's `ui-web.md`,
   14.2), from the chats to a chat by its link and back; an address
   that is no page opens the inbox, and the address reads `/` once it
   shows (2.3).
3. **The composer:** in the chats, Enter starts a chat with the words
   once, and Shift+Enter makes a new line in them (2.5). The chat opens
   with its composer focused, the focus stays there after Enter and after
   a press of "Send", and a chat opened from a list keeps its heading's
   (`README.md`, 3.5).
4. **A panic:** skein's `panic` (skein's `ui-web.md`, 14.2), which
   checks the shell's wiring of the restart part, at the test address
   `/test/panic` at which the shell panics at start, as jig's reference
   shell has it (2.1).
5. **Reading back:** scroll a chat's log up; Latest appears. Send new words
   while reading back and the log stays away from its end. Press Latest;
   the end returns and the pill goes (`README.md`, 3.5).
6. **jig's attack stories that need a browser,** from
   `jig-client-fake-person-browser`, against `temper serve`: a
   cross-site request from another port and from another host does
   nothing; a framed page shows nothing (jig's `protocol/browser.md`,
   section 15). The rebound Host, which needs no browser, stays in the
   process tier (`README.md`, section 10).

Not in the tier: skein's `typing_while_patched`, which takes a second
person, and `write_refused`, which takes a test build's bounded small
store; what they check is skein's page and store, which skein's tier
proves.

### 4.2 Later pages

A page gets a story here only for what of its own a browser alone
shows. The inbox, a task's page and the adoption form have none: a
first load at any address is the shell's one start (2.1), which 4.6's
tests 1 and 2 hold on a chat's address and a filter's. A procedure's
plan, its agent, the lineage bar and a fresh client at an agent's
address are the client world's (`suite::b2`, `suite::task`). The
change's steps to landed, its gates, its bounds and its ended tasks
hold nothing a browser alone shows: the client world's
`change_panel.rs` reads them. A later page that has such
behaviour brings its story with it: `ConnectModel`'s sign-in, its tab
sent to the provider's address and back to the page it left
(`README.md`, 3.5).

**`ConnectModel`'s sign-in, in a new tab.** What only a browser shows:
a press opening a tab within its gesture, the answer pointing it, the
provider's redirect followed to the engine's listener in that tab, and
the page, which stayed, drawing the end.
- **Against:** `temper serve` with its OpenAI provider's issuer, its
  authorization, token and key addresses, skein's fake issuer
  (`skein_fake_peers::oauth::Peer`), hosted on loopback in the
  harness's loop, its one grant a success for a synthetic subject, and
  the provider's redirect port the kernel's (the listener's port
  varies). The fake has no TLS, so the tier's bundle is built with
  skein-ui-web's `loopback-navigation` (4.3).
- **The story:** a person on `ConnectModel`'s Choose presses "Continue
  with ChatGPT"; one more tab opens (skein's `ui-web.md`, 14.2, Its tabs),
  which can't reach its opener by the time it's at the listener, its
  address the listener's; the page's own address reads
  `/accounts/connect?provider=1&way=sign-in&source=S`, in place of the
  entry it had; the page shows Ready ("Signed in to ChatGPT"); the fake
  has taken one authorization and one code exchange. A reload then shows
  Ready again and the issuer has taken no second authorization: the tab
  is pointed once.
- **The welcome's Model step:** the same press holds the welcome, and it
  moves on to Ready by itself, `/welcome?at=ready`.
- **Run on Chromium alone,** the tier's one browser. Firefox and Safari
  are read: a press reaches `window.open` within its event's listener
  (skein's `ui-web.md`, 14.3).
- **Not here:** the tab's order, the press's before the answer, a refusal,
  and the terminal's root are the client world's (`tests/client`) and the
  host's; a tab refused by the browser and the pop-up rule, skein's tier.

**`ConnectForge`'s App of your own, the pasted key,** has no story
here. Its field is a text field of the secret kind, which skein's page
maps to `<input type="password">` (skein's `ui-web.md`), its value read
as any text field's; HTML's value sanitization strips a password
input's line feeds, the one thing a browser alone would add, and the
client restores the PEM's armour from what is left (`forge.md`, 5.4).
The view's tests hold that the field is a text field of the secret
kind; the client domain's, that the stripped, the file's and the CRLF
forms reach one carried value with the armour's two line feeds, and
that the field is emptied; the service's, that `Paste` reads the fake's
real key in that restored form.

### 4.3 Its budget, outside the gate

- **A suite of its own,** outside the focused and fuzzy suites
  (`../domain/testing.md`, section 2), in its own crate
  (`tests/client-browser`, `temper-client-browser-world`), run by
  `scripts/browser.sh`, a slow tier outside temper's gate, with the nextest profile `browser-temper`,
  one test at a time, on the bundle it builds first, with the dev
  profile.
- **The tier's bundle** turns on skein-ui-web's
  `loopback-navigation` (`scripts/bundle.sh --loopback`), so that the
  round trip (4.2) can send its tab to the fake issuer on plaintext
  loopback; the ordinary binary bundle never does (skein's `ui-web.md`,
  section 10). The tier names that bundle in its configuration, so it
  runs alike over a binary built with or without its own (2.6).
- **The playground's test** (4.6, 12) runs on the built playground
  instead, `scripts/bundle.sh --playground` in its own profiles
  (`../playground.md`, 7), served as static files with their `headers`
  by the tier's own static server, which maps them as a site's host
  would; the tier reads the world's figures from the page's frame, and
  compresses the download's with the machine's `gzip` (`../playground.md`, 9).
- **A cap: at most 12 tests, and 120 seconds** on the slowest of the
  machines that build temper (the user, 10-27, temper#15). Past it,
  stories that share a setup become one test of several steps before
  any browser-only proof is dropped, and what must leave the browser
  goes to the lowest world that still shows it, named in its line of
  4.6; the cap stays. The tests are 4.6's.
- **Chromium** is the machine's, at the path its owner configures
  (skein's `browser.md`), which skein's, jig's and temper's browser
  tiers read from `SKEIN_TEST_CHROMIUM`. A missing Chromium or
  bundle fails the suite, and never skips it (skein's `ui-web.md`,
  14.1).
- **The client's crates for wasm32:** the gate runs clippy for
  `wasm32-unknown-unknown` over `temper-client-domain`,
  `temper-client-view`, `temper-client-protocol-wire` and
  `temper-client-shell-web`, so a change that stops them building for
  the browser fails the gate.

### 4.4 GitHub's manifest page

One test, the github plan's: 4.6's 10, counted in 4.3's cap.

- **What only a browser shows:** the manifest page's form posted under
  the page's own policy, `form-action WEB` (`forge.md`, 9.1). The
  redirect and the setup address need no browser; the real loop's
  GitHub stories follow all three steps as plain requests
  (`../domain/testing.md`, 2.4).
- **What it runs against:** `temper serve` with a GitHub forge whose
  `api` and `web` are the fake's GitHub layer, hosted on loopback in the
  harness's loop by skein's forge peer (skein's `fake-forge.md`, section
  10), which serves the tier for this test alone: the test needs only
  its `web`. A GitHub forge needs a fixed
  clients' listener (`forge.md`, 9.1), so the tier reserves a loopback
  port the kernel assigns before `temper serve` starts and names it in
  the configuration.
- **The test:** the signed-in browser opens the welcome's forge choice
  and presses Connect GitHub. The ordinary client supplies its return
  address. The manifest page posts its form on load to
  `WEB/settings/apps/new?state=S`, the fake's, with no policy violation.
  Its answer holds one link, Create GitHub App. Pressing that link runs
  the uninterrupted conversion, default installation and setup callback;
  setup answers with a 303 to the welcome's selected forge. The browser
  reaches that address and reads the connected state. One manifest post
  and one installation are observed. No URL is intercepted or rewritten,
  and split HTTP requests do not stand in for this click chain.
- **The advancing loop** keeps lending peers active during navigation and
  pending browser commands. A stalled browser operation is diagnosed at
  this tier before assigning it to the service or a fake. A separate
  process comparison uses the same ordinary peer configuration and
  lifecycle when needed. Evidence records route categories and statuses,
  never state/code values, token contents or a secret final address.
- **No github.com:** `web` is the fake's, and the page's policy lets
  its form post nowhere else. No test of the tier holds a real
  credential.

### 4.5 Forgejo's authorization

One test, the provider's (`../protocol/sign-in.md`, 9.8).

- **What only a browser shows:** jig's strict cookie, absent from a
  navigation that started on another site and present on the page's own
  exchanges, so that the `Authorize` place answers *go* for a person
  signed in.
- **What it runs against:** `temper serve` with jig's provider, its key
  and its client's secret made in the test's scratch directory, and one
  client whose redirect URI is a page of the harness's on the other
  loopback site (`localhost` against `127.0.0.1`), hosted in the
  harness's loop: a link to temper's `/oauth/authorize` with that
  client, and the callback, which records its query. No Forgejo: the
  process tier's story has the real one (`../domain/testing.md`, 2.6).
- **The test:** signed in by the printed link, the browser opens the
  harness's page and follows its link; temper's page sends `Authorize`,
  and the browser's address becomes the callback's, its `code` and the
  link's `state` there. Nothing of the token's is the browser's.

### 4.6 The twelve

The tier's tests, each on its own `temper serve` (the playground's on its
built playground; 2 and 11, 13 and 14 on `temper box`, whose first run is
over). Each is one test of the steps it names; 2 and 11 share one, and 4 and
5 another:

1. **Signing in** (4.1, 1): the printed link signs in, a reload on a
   chat signs in by the cookie, and the link spent shows the
   signed-out words; signing out clears the cookie on later loads. The same
   chat reconnects after a server restart and remains live after rapid reloads.
2. **History** (4.1, 2): from the chats to a chat by its link, Back and
   Forward; a new chat's composer focused, and its filter's address and
   pressed state kept across a reload and Back, and an address that is no page opens
   the inbox at `/` (4.1, 2).
4. **The composer** (4.1, 3).
5. **A panic** (4.1, 4).
6. **jig's attacks that need a browser** (4.1, 6): a cross-site request
   from another port and from another host does nothing, and a framed
   page shows nothing, in one harness with both attacker host names.
7. **A cross-site authorization** returns its code and the link's state
   (4.5).
8. **The accounts page after a key's send:** Back restores an emptied
   field, Forward the chats (4.2).
9. **`ConnectModel`'s round trip** (4.2).
10. **GitHub's manifest page** posts itself once, and Back shows it
    without posting again until Continue (4.4).
11. **The box's project across a reload:** the playground's project
    made, the Chats page Live, never Behind, with the story's chat,
    before and after a reload (`README.md`, 2).
12. **The playground's story A** on the built playground, its log
    downloaded and replayed natively (`../playground.md`, 9).
13. **Reading a public project** (8.4), with no sign-in, and
    losing it when it is made private.
14. **Reading back** (4.1, 5).
15. **A project's rename** closed by Escape, Enter and Save returns the
    focus to its summary, and never to the page's `main`.

The cap includes all browser tests, whatever their names.

Not in the tier, each in the lowest world that shows it:
- **Goals' link to a draft:** the view's
  `tests::goals::the_link_shows_with_a_tracked_template` holds its target, the
  Chats page; test 2 holds what a browser's history does with a link.
- **A procedure's plan, agent and lineage, and a fresh client at its address:**
  the client world's `suite::b2::roadmap_b2_through_the_task_pages` and
  `suite::task::a_delegates_page_shows_its_brief_run_and_report`.
- **A chat's change card to Landed, its closing and the Closed filter's row:**
  the client world's
  `suite::chat::a_new_chat_lands_its_delegated_change_and_closes_in_your_chats`.
  Test 2 keeps the filter's address and pressed state.
- **A key added, on the Accounts page:** the view's `tests::accounts` and the
  client world's `suite::accounts_page`. Test 8 keeps Back and Forward and the
  emptied field.
- **A landed goal's public pages:** the view's `tests::public`; an ended goal
  kept in the public index, jig's
  `suite::public::an_ended_public_goal_is_retained_by_the_public_referee`.
- **Address-codec edge rows:** `temper-client-domain`'s
  `tests::address::{a_path_that_is_no_page_is_missing,each_number_spans_its_types_range}`.
  Browser test 2 keeps the shell's replacement of a missing address with `/`.
- **Repeated loads at the watcher bound:** the client world's
  `suite::reloads::repeated_loads_release_the_previous_clients_watches`.
- **Composer bounds:** the client domain's
  `tests::pages::chats::new_chats_and_cues_keep_the_send_bound_when_receiving_grows`.
- **Provider routes and refusal pages:** the service's provider protocol test,
  with the view's `sign_in::tests::each_authorization_page_uses_the_mockup_words`.
- **A moving account's rendering:** the view's
  `tests::accounts::a_moving_account_shows_its_state_without_buttons`; the
  accounts world's `suite::moves::a_written_move_erases_the_source_and_its_account`
  holds removal.
- **A chat's day and turn clocks beside its speakers:** words and roles
  alone, which the view's tests (`temper-client-view`'s chat tests:
  order, day lines, midnight) and the client world's chat already hold.
- **GitHub's connect after the manifest** (conversion, installation,
  the forge chosen, adoption, the Model summary): plain requests and the
  client's events, in the real loop's GitHub world and the client
  world's welcome, forge and adoption stories; the manifest's post, the
  one thing a browser alone shows, stays in 10.

## 5. Open questions

- **Kept work in the browser** (`README.md`, 3.4 and 6.6): another
  party's kept work, and kept work dropped at start, which the terminal
  says in lines of its own. Default: not shown; the small store lives
  as long as its tab, of one person.
- **A bundle built apart, embedded:** `TEMPER_BUNDLE=DIR`, a bundle's
  directory embedded in place of building one, so that one wasm32
  build serves binaries for several hosts and a machine without the
  target can still make a whole binary. Default: none; it waits for
  the release pipeline (2.6).
- **Served from the binary's bytes,** with no copy at start: a `Bundle`
  that borrows. Default: copied (2.6), until a bundle whose copy
  matters.
- **The embed's steps, skein's for every application:** the nested
  build, its directory, its environment and what reruns it are nothing
  of temper's. Default: temper's crate, until a second application
  embeds its bundle (jig's reference application, or skein's example),
  when they move to a small crate of skein's that a build script
  depends on (skein's `ui-web.md`, section 15).
- **A binary to download:** where release binaries are built, for
  which targets and C library (a static musl build, or a floor of
  glibc), and where they are published. Default: none; temper's
  binary is whole (2.6), not shipped.
- **A switch to serve no browser** from a binary that has one, such as
  `clients.bundle` `false`. Default: none until a deployment asks: the
  browser is reached only by a printed link, as the terminal is.
- **A dark palette,** following the system's preference as
  `skein-ui.css` does. Default: light only (2.5).
- **Other browsers** than Chromium are `skein-browser`'s question
  (skein's `browser.md`).
- **One shared stream per public document,** fanned out by the server
  to every visitor reading it, in place of each page reading again
  (section 8). Default: none; pages read again at the `max-age`, and a
  visitor never holds a stream.
- **The kit's stylesheet without a bundle:** the gallery takes
  `skein-ui.css` by running skein's bundler with no module (7.1).
  Default: so, until skein's bundler offers the stylesheet alone.
- **A press a render can lose:** the diff matches unkeyed children by
  position among siblings of the same role (skein's `ui.md`, section 7),
  so a section arriving before a control's group gives the control a new
  element, and a press between its mouse down and up does nothing, with
  nothing shown. Default: a group that sections arrive beside takes a
  role of its own, as Accounts' Sign out does, page by page; skein's
  diff stays.
- **A phone's width in a test** (2.5): the sweep, the walk and the renders of
  the request `ui-phone` of 2026-11-05 are throwaways by its own words, and the
  browser tier cannot open a phone's width: `skein-browser` sets every page to
  1280 by 800. So nothing holds how the phone's rules draw when the sheet next
  moves: `temper-client-shell-web`'s suite reads one rule as text, the gallery's
  `the_accounts_tabs_sign_out_hides_over_a_phones_width_alone` another, the proof
  at 420 px of the request `ui-owner-menu` is a throwaway too, and the CDP walk of
  the request `ui-final-walk`, which sets the width, goes with its plan.
  Default: none, until a phone's rule breaks unseen.
- **The rail's foot on a phone** (2.5; `README.md`, 5.3): the top bar holds the
  switcher alone, as the mock's does, so a project's page on a phone shows no
  link's state, no "Behind" and, but on the `Accounts` tab, no "Sign out did
  not reach the engine.", and
  the way out of the playground is its banner on `Chats` alone. The mock has no
  link to say so of. Default: so; whether a link not live takes a place in the
  top bar is a look request's.
- **A covered page's other edges** (2.5): the mock and the sheet keep off the
  home bar alone, and at 760 px and under alone. A phone on its side, wider than
  that, draws the rail and a chat's foot under its notch and its home bar.
  Default: as the mock draws it, until a look request.
- **The try's states in the gallery** (7.2; `README.md`, 3.5, *The try*;
  `../draft/model-check.md`): Model's line past the host's check has no state of its
  own, since the gallery draws what the view draws and no fact reaches it. Default:
  the host's two rows, the view's tests holding the rest, until that draft gives
  the facts.
- **A tab refused in the gallery** (7.2; `README.md`, 3.5, *The sign-in under way*;
  `../draft/sign-in-new-tab.md`, 2): the gallery draws what the view draws, and the
  view has no fact of a tab refused, so the note's plain title and primary link have
  no state of their own. Default: the view's test holds them, until that draft gives
  the fact.
- **A chat's way to the Closed filter** (4.6): a chat closed on the box took 8.5 s
  to show there, and no measure splits that between the chat template's 5 s of
  grace (`temper-service`'s `stock.rs`) and the box's 5 s `agent_stop_ms`. A person
  waits it, and no test of the tier does. Default: as it stands, until the wait is
  split and a request asks it shorter.
- **The box's pace in the tier** (4.6): the stories on `temper box` keep story A's
  paces, a reply's 1200 ms and CI's 4000 ms. None waits for a landing, and unpaced
  fakes would land in the background for CPU nobody reads. Default: paced.
- **A wait's CPU in the tier** (4.3): a `See` asks Chromium's accessibility tree
  again at `skein-browser`'s `poll`, 50 ms, so a long wait costs browser and harness
  CPU: 519 queries in a story of 23 s. Default: the kit's poll stands, and a story
  doesn't wait long.
- **A public page's text in one query** (8.4): the public walk reads each page's text
  a node at a time, about 0.8 CPU s over its seven pages. Default: so, until that
  passes 1 CPU s, when the browser kit would read a page's text at once.
- **A compile cache across the tier's tests** (4.3): each test's fresh profile
  compiles the bundle's wasm again, where a second load in one profile hits
  Chromium's own cache. Default: none, since a shared profile would change what each
  test starts from.
- **A heading drawn after its page, in skein's reference** (2.3): the kit's rule
  stops at the main (skein's `ui-web.md`, 7), and any application whose page reads
  before it draws its heading meets what this shell repairs. Default: this shell's
  alone, until a second application writes the same.

## 7. The gallery

`temper-client-gallery` (`tools/gallery`), a program of temper's
workspace, prints the browser's pages in named states into a
directory, as static documents, for screenshots beside the mockups
and a person's eye. It is a tool, not a test of the pages: what
the pages do is the worlds' (section 4), and the gallery draws what the
view draws, never changing a view to suit a sample. It stays for the
states no journey reaches (`journeys.md`): a journey's print, replayed
from the engine's own answers, is where the look is tuned.

### 7.1 What it prints

- **One command:** `cargo run -p temper-client-gallery -- OUT`. It
  reads no configuration, environment, clock or network, and starts no
  server.
  `OUT` is replaced whole when it holds the gallery's `index.html`,
  made when it is missing, and refused otherwise.
- **Each state's page** as `OUT/NAME.html`: the view's tree for the
  state, made a page by skein's `Page` with the shell's look (`Look`
  with `look::NAMES`, primary input `Control`) and its prefix, its
  links given the hrefs the shell gives them (2.3), by the view's one
  href function, which the shell's host calls too. The page's root is
  the document's `#app`, as skein's `Dom` mounts it; dialogs carry
  `open`, since a static document has no top layer. The title is the
  shell's: the page's heading, then "temper".
- **The bundle's stylesheets and files,** under `OUT/bundle/`, made by
  skein's bundler from the stylesheets and files `scripts/bundle.sh`
  passes, in its order (2.5), with no module: each page links them as
  the bundle's page does, so `temper.css`'s files resolve as they do
  in the bundle. The bundle's own page and its empty module are unused.
- **An index,** `OUT/index.html`: each state's name, in the gallery's
  order, linking its page, with its mockup's address beside it where a
  mockup draws it.
- **Static:** no script runs. The opening focus is an `autofocus`
  attribute, and a named sample may hold one disclosure open; caret,
  scroll-following and live updates are not replayed.

### 7.2 Its states

- **A state** is a world at a page's address, then what is done there:
  the client's root stepped from start by the documents a world
  delivers, then by the person's acts or the engine's answers that the
  state names (a close pressed, a reject's reason box, a refusal), as a
  page would be. No page state is set by hand.
- **The worlds are the mockups':** reference, solo and the first run at
  its steps, their words copied from `mockups/world-*.js` where temper's
  model holds them, so that a picture differs from its mockup by look,
  not content. A value the model has no place for is left out, never
  invented. The gallery's own documents carry what the domain's
  fixtures don't: the providers, accounts and their holders by name.
  A spend is the contract's whole number (jig's owner numbers), so a
  mockup's fraction is rounded to the nearest (0.8 is 1) until the
  contract holds one.
- **Its time** is each world's, fixed: the mockup's now, its
  `world.now`, at which the view builds, and each record's time the
  mockup's offset from it, so that a time reads as its mockup's
  ("12 min ago"). The program still reads no clock (7.1).
- **Its bounds** are the browser's shipped client limits as they are
  (`temper-client-shell-terminal`'s `native()`, which the browser's
  shell takes), never widened, and the view's bounds are derived from
  them as the shell derives its own: a page shows what a person's
  browser would, a window's rows and a chat's lines included (`README.md`,
  5.4), so that its outline is the live page's. A world holds more than a window shows; the page shows the
  window, and the state's test row says what it shows.
- **Names,** binding: a state a mockup draws is named by
  its address, `<page>-<world>[-<step>][-<task>]` (`chat-solo-T10` for
  `chat.html?world=solo&task=T10`, `chats-first-run-1` for
  `chats.html?world=first-run&step=1`); one no mockup draws takes a
  plain name (`signed-out`, `chats-empty`, `adoption-refused`). A name
  is never reused for another state.

| Name | The state |
|---|---|
| `changes` | a hand-made `ChangesPanel` delivery: three queued and four landed |
| `changes-empty` | a hand-made `ChangesPanel` delivery: nothing queued and nothing landed |
| `changes-one` | a hand-made `ChangesPanel` delivery: one queued and four landed |
| `changes-landed-only` | a hand-made `ChangesPanel` delivery, the mock's `hello`: nothing queued, something landed |
| `home-away` | hand-made Home deliveries: `AwayRow` on, four `Decision`s and live tasks; the mock's `home.html` in *Back from away* |
| `home-away-one` | hand-made Home deliveries: one `Decision`, the switch off: "One thing was decided for you." and the kicker's two times |
| `home-away-overturned` | hand-made Home deliveries: a row overturned, its task live, and one ended, "landed 03:12" |
| `rail-away-off` | a project’s page after turning the switch off, with its toast open |
| `rail-away-on` | a project's page with the switch on, "Away until 08:00", and its toast open |
| `settings-away` | *How work runs* with the section `#away`, the defaults ticked |
| `home-three-left` | hand-made Home deliveries: all three steps left; first repository offered |
| `home-two-left` | hand-made Home deliveries: repository adopted; configuring and first chat left |
| `home-accounts-claude` | hand-made Home deliveries: Claude subscription alone; link offers ChatGPT or another plan |
| `home-accounts-both` | hand-made Home deliveries: ChatGPT plan and Claude subscription; link offers another account |
| `home-accounts-key` | hand-made Home deliveries: an API key; link offers another account |
| `home-one-left` | hand-made Home deliveries: repository adopted and a way of working accepted; first chat left |
| `home-with-goals` | `home-at-work`'s deliveries with a goal in the project, so that the rail shows Goals |
| `home-at-work` | hand-made Home deliveries: one question and three chats, the everyday request's words where the client holds its facts |
| `home-morning` | hand-made Home deliveries: `ProjectStatus` with everything: a brief above `seen`, a last look nine hours ago, three queued, needs and chats; the mock's morning |
| `home-computed` | hand-made Home deliveries: `ProjectStatus` with changes landed and no brief: the computed sentence, no lede or byline |
| `home-last-looked-none` | hand-made Home deliveries: `ProjectStatus` of a party's first look: no "last looked" and nothing new |
| `home-landing-none` | hand-made Home deliveries: `ProjectStatus` with nothing landed: "Landing: nothing has landed yet" |
| `home-landing-red` | hand-made Home deliveries: `ProjectStatus` with `red`: " · main red" in the signal's colour |
| `home-chats-sentence` | hand-made Home deliveries: no goal in the project: Chats' sentence after its rows |
| `home-needs-several` | hand-made Home deliveries: three needs, the first as a card and the others as rows |
| `home-nothing-needs` | hand-made Home deliveries: all steps done, no needs and one chat |
| `home-ready` | hand-made Home deliveries: all three steps done; plain Home |
| `home-adopting` | hand-made Home deliveries: first adoption in flight; no repository acts |
| `home-copying` | hand-made Home deliveries: acme/widget copying, acme/other waiting, in intent order |
| `home-copy-failed` | hand-made Home deliveries: first copy adopted, second failed with the forge’s words and Copy again |
| `home-copied` | hand-made Home deliveries: both copies adopted, with their source names and the Forgejo explanation |
| `home-copy-one` | hand-made Home deliveries: one copy adopted, with its singular explanation |
| `home-copy-offer` | hand-made Home deliveries: both addresses left unasked after a restart, with one filled Copy act |
| `home-add-repository` | hand-made Home deliveries: connected owned Forgejo with no first repository; Make, Copy and Adopt another |
| `home-add-repository-sheet` | those deliveries with Make an empty one pressed; the repository sheet open on Home |
| `home-add-repository-copies` | the same Home sheet opened at Copy a public one, with its addresses |
| `home-refused` | hand-made Home deliveries: first adoption refused; its words and retry acts |
| `home-no-repository` | hand-made Home deliveries: no repository or first offer |
| `home-after-skip` | hand-made Home deliveries: To begin skipped with steps left; plain Home with empty Needs you and Chats |
| `home-skip-refused` | hand-made Home deliveries: Skip refused; the alert under the note and Skip offered again |
| `home-needs-under-begin` | hand-made Home deliveries: To begin showing above one question and one chat |
| `home-begin-settings-left` | hand-made Home deliveries: the first and second steps left; their acts stay in Settings |
| `home-begin-one-chats` | hand-made Home deliveries: the third step alone left; its act stays in Chats |
| `goals-solo` | hand-made Goals deliveries: a small project's rows, one in each state, the mock's words where the client holds its facts |
| `goals-reference` | hand-made Goals deliveries: the reference's rows, a goal of a hundred tasks among them, and its ways in the lede |
| `goals-none` | hand-made Goals deliveries: "No goals yet."; the rail keeps Goals since the page is Goals |
| `goals-needs` | hand-made Goals deliveries: the filter Needs you, "needs funds" and "needs you" |
| `goals-under-way` | hand-made Goals deliveries: the filter Under way, running, planning and paused |
| `goals-ended` | hand-made Goals deliveries: the filter Ended, landed, closed and failed |
| `goals-filter-empty` | hand-made Goals deliveries: a filter no goal matches, "None." |
| `goals-more` | hand-made Goals deliveries: "More than N goals, M under way." and "More goals" |
| `home-github` | hand-made Home deliveries: GitHub connected with an accepted installation |
| `home-morning-card` | hand-made Home deliveries: three needs in the line form, the first as its card with its choices, the others as lines |
| `home-answered` | `home-morning-card` with the first answered: its line, its toast, the next as the card |
| `home-one-need` | one need: no lines and no "Answer in a row"; on a phone, "Answer" |
| `configure` | the configuring page before its first words, template 3 listed |
| `configure-reference` | "See how it works" pressed: the stock reference beside the page, in Split, the rail folded, no chat made |
| `configure-reference-draft` | `configure-reference` in Draft: the reference alone, the composer under it |
| `configure-typing` | words entered; Send enabled |
| `configure-sending` | the configuring start out; starters and Send disabled, words kept |
| `configure-refused` | the start refused; jig's words below the composer, the words kept |
| `configure-chat` | the solo world’s T10 as a configuring chat, conversation and proposal in the Settings layout |
| `configure-chat-proposal` | `configure-chat` with a pending proposal of settings: its title, lines, items and "Accept" |
| `configure-chat-accepted` | that proposal accepted: "✓ Accepted · new work follows it" |
| `configure-chat-replaced` | that proposal dimmed, "Replaced by the proposal below", the later one under it |
| `settings` | hand-made settings deliveries: How work runs with every part of kind 7 shown |
| `settings-all-hidden` | hand-made settings deliveries: kind 7's stock, the mock's `settings.html?project=hello` |
| `settings-goals` | hand-made settings deliveries: Goals shown, its ways of working with one the default |
| `settings-chat-open` | hand-made settings deliveries: a configuring chat open, "Go on" |
| `settings-drafts-off` | `settings` with "Chats can write drafts" off |
| `settings-goals-off` | `settings` with Goals hidden: no ways of working, and the rail without "Goals" |
| `settings-changes-off` | `settings` with Changes hidden: the rail without "Changes" |
| `settings-away-off` | `settings` with the Away switch hidden |
| `settings-refused` | a switch pressed and refused, jig's words under its row, the switch back at its value |
| `repositories` | the Repositories tab with rows |
| `repositories-none` | the Repositories tab's quiet band |
| `forges-in-settings` | `Forges` inside Settings' frame |
| `accounts-in-settings` | `Accounts` inside Settings' frame, "Sign out" at its foot at a phone's width alone |
| `forges-empty` | the Forges tab's quiet band, in a world with no forge |
| `forges-setting-up` | the first run's Forges at step 6: a row setting up |
| `forges-fetching` | the first run's Forges at step 24: a row fetching Forgejo, its percent in its summary |
| `forges-fetching-bytes` | the first run's Forges at step 25: a row fetching Forgejo, its bytes in its summary |
| `forges-failed` | the first run's Forges at step 9: a set up failed, with "Set up again" |
| `forges-replacing` | the solo world's Forges, a row's Replace open on its private key |
| `forges-removing` | the first run's Forges at step 7: a row's removal asking first |
| `accounts-empty` | the Accounts tab's quiet band, in a world with no account |
| `accounts-states` | the reference world's accounts, a row in each state of a source |
| `accounts-key` | the reference world's accounts, a row's key field open |
| `accounts-removing` | the reference world's accounts, an account's removal asking first |
| `accounts-signing-in` | the reference world's accounts, a row's sign-in under way with its field for the page's address |
| `accounts-program-needs-sign-in` | the solo world's accounts: a program's login not signed in on its host, with "Check again" |
| `builders` | the Builders tab, its hosts connected, draining and not connected |
| `builders-not-owner` | the Builders tab for a person with no role in the deployment |
| `builders-none` | the Builders tab with no host connected |
| `configure-chats-tag` | the solo world’s chats with T10 tagged Configuring beside ordinary chats |
| `chats-solo` | the solo world's chats |
| `chats-live` | `chats-solo` under "Live" |
| `chats-closed` | `chats-solo` under "Closed" |
| `chats-no-repository` | the Chats page of a project with no repository: its band |
| `chat-new` | `NewChat` with a repository: its `.meta-line` and focused composer |
| `chat-new-no-repository` | `NewChat` with no repository |
| `chat-new-first-run-1` | `NewChat` at the first run's first screen: the playground's start |
| `chat-work-done` | the `chat-work` request's conversation, calls settled and folded, answer committed |
| `chat-work-open` | the same conversation, the call run open to all thirteen rows |
| `chat-work-running` | its first four calls settled and template 2 being checked, with no supplied outcome |
| `chat-work-writing` | the same calls settled, the supplied opening paragraph being written with its end mark |
| `chat-work-failed` | a failed charter read, its line and outcome in the signal's colour |
| `chat-work-one` | one `read_docs` call alone, "Read the page hold" |
| `chat-work-unknown` | a tool outside the words table, "read_file ×2" |
| `chat-work-separated` | text between two calls ends one run and starts another |
| `chat-work-empty` | before the first word or call, the three dots |
| `chat-work-reading` | earlier messages loading and Latest over the foot while away from the end |
| `chat-working` | a live chat at work: its three dots |
| `agent-work` | an agent's page, its last run running with a call under way, and Latest over the foot while away from the end |
| `chat-change-running` | a chat with a delegated change running: its one line |
| `chat-change-landed` | a chat with a delegated change landed: its one line with its result's words |
| `chat-change-held` | the first run's chat T1 at step 2, its change held: its one line says so |
| `chat-goal` | a chat with a goal it started: its one line |
| `chat-byline-open` | a new chat with its byline pressed: the chooser's panel |
| `chat-draft-chat` | a chat with a document, in Chat: its group and tool line |
| `chat-draft-split` | the same chat in Split: the strip, the pane and a tinted section |
| `chat-draft-draft` | the same chat in Draft: the composer at the foot on white |
| `chat-draft-empty` | "+ Draft" pressed and nothing written: the empty pane |
| `chat-new-draft` | `NewChat { draft: true }`: the empty pane in Split |
| `chat-start-goal` | a chat's sheet to start a goal, open |
| `chat-start-goal-refused` | the same sheet, refused in jig's words |
| `chat-goal-linked` | a chat with its goal started: "Goal TITLE →", and its line |
| `chats-drafts` | the Chats page under Drafts: "writes away-mode.md" |
| `configure-chat-document` | a configuring chat in Chat, its document written |
| `configure-chat-document-split` | the same chat in Split, its pane's line and "Start over" |
| `rail-new-menu` | the rail's "+ New chat" with its menu open |
| `rail-person-menu` | the rail's person with its menu open |
| `chat-solo-T10` | the solo world's chat T10, its proposal in place |
| `inbox-solo` | the solo world's *Needs you* (`README.md`, 3.5): the first of a proposal and a question |
| `needs-one` | *Needs you*: a question in the line form, the first of three |
| `needs-saw-open` | `needs-one` with "What the agent saw" open |
| `needs-reply-open` | `needs-one` with "Reply…" pressed: the reply form |
| `needs-second-answered` | the first answered, its line and toast showing: the second of three |
| `needs-later` | the first marked later: the next shown, the toast's words |
| `needs-last` | the last card: "The last one" |
| `needs-done` | every card answered: "Nothing needs you." and the answered lines |
| `needs-proposal` | a proposal's card |
| `needs-held` | a held task's card |
| `needs-funds` | a held task out of budget: "S of L used." |
| `needs-plain-words` | a question in no line form: "Reply…" alone |
| `goal-running` | first-run-projects, running: six of nine tasks in, its report, Left and Risk, the bar, three waiting lines, "6 landed" folded |
| `goal-needs-funds` | first-run-projects held for funds: "needs funds", its band with Release and Leave held |
| `goal-needs-question` | welcome-stall with an open question: its band and its parsed choices, the recommended one primary |
| `goal-planning` | forge-cards, no report and no tasks: the planning words, the dots and "Proposing tasks from the draft…" |
| `goal-paused` | first-run-projects paused by the person: "paused by you", Resume |
| `goal-landed` | pane-chat-line landed: its result, "Landed at 01:12.", no acts and no composer |
| `goal-landed-open` | `goal-landed` with "N landed" pressed: its rows under it |
| `goal-closed` | welcome-stall closed: "Closed by you…", "Closed before it landed." |
| `goal-failed` | forge-cards failed: "Failed: " and its reason, "Failed before it landed." |
| `goal-no-report` | first-run-projects running with tasks and no report: "N of M tasks landed.", no byline |
| `goal-amending` | `goal-running` with Amend pressed: the editor, between the header and the report |
| `goal-closing` | `goal-running` with Close pressed: its confirmation |
| `goal-toast` | `goal-running` paused, its toast showing |
| `goal-sending` | `goal-running` with the person's words sent and one being sent, the composer holding its next words |
| `start-goal` | Goals with "Start one from a draft" pressed: the sheet with no draft, its Draft field and two ways |
| `start-goal-draft` | the sheet opened with first-run-projects.md, its name, its default way and "When it needs you", following the away switch |
| `start-goal-refused` | `start-goal-draft` after a refused start: "Always ask" chosen, `.sheet-error` in jig's words |
| `task-solo-T3` | the solo world's goal T3, on `goal.html`'s page |
| `accounts-solo` | the solo world's accounts |
| `chats-first-run-1` | the first run's Chats at step 1: an empty list |
| `welcome-first-run-0` | the first run's welcome: its two cards |
| `welcome-go-on` | Welcome asked for with a real forge under way: Go on setting up, no playground |
| `connect-forge-first-run-6` | the first run's `ConnectForge` at step 6: Forgejo here setting up, the box preparing |
| `forges-first-run-7` | the first run's Forges at step 7: Forgejo here connected |
| `welcome-forge-choose` | the welcome's Forge choices, before any forge is connected |
| `welcome-forge-details` | the welcome's Forgejo Details, with organization and repository paired |
| `welcome-sheet-copies` | Forgejo's sheet with public copy addresses, in the order written |
| `welcome-sheet-copies-refused` | Forgejo's sheet refuses an invalid copy address above its foot |
| `welcome-sheet-nothing` | Forgejo's sheet with Nothing for now selected |
| `welcome-forge-under-way` | the Forge step's status in the column, Each step, and Continue while Forgejo sets up |
| `welcome-forge-connected` | the Forge step's connected Forgejo and first repository, with Continue |
| `welcome-forge-github` | the Forge step's accepted GitHub installation, with Continue |
| `github-app-details` | GitHub's own-App form, with masked key input |
| `github-install` | GitHub's connected App, installed nowhere yet |
| `github-picker` | the installation's reached repositories, ready to adopt |
| `github-adopted` | the adopted GitHub repository and its primary Open act |
| `welcome-model-choose` | the welcome's Model choices, ChatGPT and Claude |
| `welcome-model-forge-under-way` | Model at first run step 6, Forgejo setting up, its Forge pane and Each step |
| `welcome-model-forge-fetching` | Model at first run step 6, Forgejo fetching at 42%, its bar filled and the figure at its end |
| `welcome-model-forge-fetching-bytes` | Model at first run step 6, Forgejo fetching with no size, its moving bar and 41 MB at its end |
| `welcome-model-forge-done` | Model at first run step 7, Forgejo connected, its marked Forge pane without Each step |
| `welcome-model-forge-failed` | Model at first run step 9, Forgejo failed, its Forge pane and Each step |
| `welcome-model-claude` | Claude’s welcome command and I’ve signed in, with no Name field |
| `welcome-model-key-details` | the welcome's API key form, without an unsupported display-name field |
| `welcome-model-connected` | a usable ChatGPT plan, its ticked status and Continue to Ready |
| `welcome-model-away` | the page waiting for ChatGPT’s sign-in, its note, "Open it again" and folded paste |
| `welcome-model-signing-in` | the same note while signing in to ChatGPT inside the welcome |
| `welcome-model-signin-ready` | the connected state after signing in to ChatGPT |
| `welcome-model-declined`, `welcome-model-renewed`, `welcome-model-refused` | the other sign-in endings inside the welcome |
| `welcome-paste-waiting`, `welcome-paste-written` | the waiting page’s folded paste, with an empty or supplied secret address |
| `welcome-paste-too-long`, `welcome-paste-address`, `welcome-paste-state` | the length and address refusals, ready for a fresh paste |
| `welcome-paste-sending`, `welcome-paste-finishing` | the finish request in flight, then accepted |
| `welcome-paste-too-many`, `welcome-paste-ended`, `welcome-paste-expired` | the restart endings, with no paste field |
| `welcome-paste-lost`, `welcome-paste-unsent` | the address cleared after a lost or unsent finish request |
| `welcome-ready-checking` | Ready from its first frame, the declaration or host check under way, Model's line at its first step, its bar and Each step |
| `welcome-ready-copies` | Ready offers Widget and names the two repositories its act will copy into the project |
| `welcome-ready-none` | Ready with neither forge nor model, the pane's two lines, their inline connect links and the offered playground |
| `welcome-ready-not-yet` | Almost ready, the settled check without a usable login, Model's line stopped at its first step, Try again, Sign in again and Each step |
| `welcome-ready-signing` | Ready while ChatGPT signs in, Model's line in the pane as it is |
| `welcome-ready-under-way` | Ready while Forgejo sets up, Forge before Model, Each step and the note below the form |
| `welcome-ready-adopted` | Ready with Forge and Model connected, the project form and Back to Model |
| `forge-setup-failed` | the failed setup's reason and neutral retry act |
| `forges-github-install` | GitHub's Forges row before installation, with its primary Install act |
| `forge-admin-details` | a managed Forgejo's admin Details, with its first-repository fieldset and legend |
| `connect-forge-choose` | `ConnectForge`'s Choose: GitHub, Forgejo on this machine and Your Forgejo |
| `adoption` | the adoption form, empty |
| `adoption-filled` | the adoption form, filled in |
| `adoption-refused` | the adoption form after a refusal, its alert |
| `chats-reference` | the reference world's chats |
| `chat-reference-T1` | the reference world's chat T1 |
| `chat-solo-T1` | the solo world's chat T1, closed: no composer and no chooser |
| `chat-solo-T6` | the solo world's chat T6 |
| `chat-held-task` | the solo world's chat T10, its proposal's reason saying a task is held |
| `chat-first-run-2-T1` | the first run's chat T1 at step 2, its change running |
| `chat-first-run-3-T1` | the first run's chat T1 at step 3, its change landed |
| `chat-first-run-4-T4` | the first run's chat T4 at step 4, waiting at rest for the person’s words |
| `playground-resetting` | the first run's Chats while the playground resets: "Leaving the playground…", and no playground's start |
| `chats-first-run-5` | Chats after the first plan connects |
| `inbox-reference` | the reference world's *Needs you* |
| `accounts-reference` | the reference world's accounts |
| `accounts-first-run-1` | the first run's accounts at step 1: the playground's account |
| `signed-out` | signed out |
| `out-of-date` | a bundle older than the engine |
| `chats-empty` | the chats with none, outside the playground |
| `inbox-empty` | nothing needs the person |
| `chat-closing-below` | a chat at rest with one task below it: a singular count and the hairline |
| `chat-closing-proposal` | a chat at rest with one proposal: a singular count and the hairline |
| `task-releasing-uncertain` | a release with an uncertain effect and its two choices |
| `chat-closing-work` | a run, two delegates with opposite fates, and two proposals: what closing changes |
| `chat-closing-failed` | the same close refused: the sheet keeps its choices and says why |
| `chat-closed` | the requester closed it: one end line at 21:05, with a quiet bar |
| `chat-paused` | a chat its person paused: "Paused", Resume and Close, and the composer's hint "Sending resumes this chat." |
| `chat-closing-start-over` | a configuring chat's Start over confirmation |
| `task-closing` | an agent's task Close confirmation, with a task below it |
| `task-paused` | an agent's task its person paused: "Paused", Resume and Close, and no own held card |
| `proposal-accepting` | a proposal's Accept confirmation |
| `proposal-rejecting` | a proposal's Reject confirmation with its reason |
| `chat-closing` | `chat-solo-T6` with Close pressed: its confirmation |
| `chat-rejecting` | `chat-solo-T10` with Reject pressed: the reason's box |
| `task-held` | an agent's task's plan with a held task, Release offered |
| `task-releasing` | `task-held` with Release pressed: its confirmation |
| `connect-model-choose` | the offered ways and the owner's holder choice |
| `connect-model-key-details` | a key's secret field and Add key |
| `connect-model-program-details` | a program's name, command and Declare |
| `connect-model-signing-in` | the waiting note, "Open it again" and folded paste |
| `connect-model-key-ready` | a key kept, Ready with Start a chat |
| `connect-model-signin-ready` | a sign-in made an account, Ready |
| `connect-model-declined` | plan usage declined, with another sign-in and a key way |
| `connect-model-renewed` | the same account again, with another sign-in |
| `connect-model-refused` | another holder's account refused, with another sign-in |
| `connect-model-key-lost` | a key request's answer lost, its alert asking for the value again |
| `task-reference-T28` | the reference world's change T28, queued 2nd after T26 |
| `task-reference-T24` | the reference world's change T24, landed, its ended task T25 |
| `task-reference-T12` | the reference world's goal T12, its tasks with its ended delegates T13, T14 and T24 folded |
| `task-solo-T9` | the solo world's agent T9, its run's committed calls and words within the shipped window |

### 7.3 Its tests

In `tools/gallery`'s own tests, in the view's world: each state's tree
read by skein's fake person (`../domain/testing.md`, 4.4), as the
gallery builds it.

- **The names:** the states' names, in order, are `tools/gallery/names.txt`,
  kept by hand, so a name gone, changed or added fails until the file
  says so.
- **Each state holds what its name says:** a table of rows, each a
  name, its page's heading and the one part the state is about, found
  by role and name (the proposal in `chat-solo-T10`, the three
  suggested requests in `chat-new-first-run-1`, the alert in
  `adoption-refused`).
- **The print:** the program run into a scratch directory writes every
  name's page, the index linking each, and the bundle's stylesheets in
  `scripts/bundle.sh`'s order, read from that script.
- **Their cost:** together at most 1 s (0.5 s before the user's doubling of
  10-22, provisional) of the default suite; no fuzzy test.

## 8. Reading without signing in

The web client's anonymous mode: a visitor reads a public project
(jig's `domain/people.md`, section 13) through its public documents,
jig's and the forge's (`forge.md`, section 11), never signing in and
never streaming (`../draft/first-release.md`, 5.3).

### 8.1 Its pages and addresses

| Page | Path | Reads |
|---|---|---|
| `PublicProjects` | `/public` | jig's *projects* |
| `PublicBoard { project }` | `/public/projects/<project>` | jig's *board*, and the forge's *landings* for its queues' lines |
| `PublicTask { project, task }` | `/public/projects/<project>/tasks/<task>` | jig's *card*, and the forge's *change* where the task makes one |
| `PublicLandings { project }` | `/public/projects/<project>/landings` | the forge's *landings* |
| `PublicLanding { project, task }` | `/public/projects/<project>/landings/<task>` | the forge's *landing*, through a member's change's task |

- **By temper's address codec,** as every page's (2.3); `<project>`
  and `<task>` are public identifiers, 16 lowercase hexadecimal digits
  (jig's `domain/people.md`, 13.8), never numbers. A path under
  `/public` that does not read is `/public`.
- **Served as every page is:** the server answers each with the page
  and the bundle (2.2); nothing on the server is new but the public
  kinds the forge declares to jig (`forge.md`, 11.1).

### 8.2 How it reads

- **No sign-in, no stream, no small store.** At an address under
  `/public`, the shell starts the public mode in place of `jig-client`'s
  sign-in: no `Transport`, no credential, no watch. It reads each
  document its page shows by jig's public `GET` through jig's HTTP
  client end (jig's `protocol/client.md`, 17.5), and decodes it with
  jig's and the forge's public records alone.
- **It reads again by itself,** when a document's freshness ends (the
  `max-age` its answer gave, less the `Age` a cache added), while its
  tab is visible; a hidden tab reads nothing, and reads at once when
  shown. The proxy's cache answers most of these reads; the engine sees
  one per document per `max-age` (`../protocol/sign-in.md`, 2.4).
- **Nothing shown past its freshness.** A document is shown only until
  its freshness ends: a read again that is answered busy or fails keeps
  it until then, marked as of when it was read, and after it the page
  drops it and says "Not available just now.", reading again after
  `Retry-After`, doubling to a bound of a minute. So a project made
  private leaves an open page within the `max-age`, as it leaves the
  proxy; a copy already shown or saved is not withdrawn.
- **Unknown** shows "Not public, or not here." with a link to
  `/public`, and drops what the page showed.
- **Anyone, signed in or not,** sees the same pages at these addresses:
  the answers are the same for every visitor. A signed-in person's
  cookie goes with the `GET` and is never read. The banner offers "Sign
  in" where the deployment has a way, and nothing else of a person's.

### 8.3 What it shows

- **Titles and the project's name, no other text:** the records carry
  no text but these (jig's `domain/people.md`, 13.3), so a project is
  its name, and a goal is "Goal" and a task "task", each with the first
  eight digits of its public identifier, then its title, with "…" where
  cut; an executor is its kind, "an agent", "a procedure", "a person".
- **The board:** the project's goals, each a line: its identifier and
  title, its phase in words, for how long, its spend; under it its
  tree's lines: "task 3f2a9c01 Write the parser, an agent, running for
  14 minutes", its phase, its
  times; the count of lines folded; and the queues' lines from the
  landings.
- **A task:** its line, its lineage up to its goal as links, its
  delegates, and where it makes a change, the change's step in words,
  CI's state, its gates' verdicts by number and kind, and "its change
  on the forge", the pull's address, with the head's commit linked by
  its short hash.
- **The landings:** the turn, each queue in order, without places, and
  the recent landings, each landed commit a link.
- **Never** a person's name, any words but a title, or anything the
  records do not carry: the view reads only the public records.

### 8.4 Its tests

- **Step tests:** the address codec's public paths, both ways, and a
  malformed one (a number, upper-case, short or long identifiers) read
  as `/public`; the public mode's reads, its timer from the answer's
  freshness and `Age`, a hidden tab reading nothing, busy keeping the
  page until its freshness ends and then dropping it, backing off,
  unknown's main.
- **Fuzzy tests:** the address codec's parser over seeded paths under
  `/public`, never panicking and printing back what it read.
- **The browser tier** (section 4): a public project on fakes read at
  `/public`, its board, a task and its landings, by a page with no
  sign-in and no stream, the view's text checked against the public
  records' fields alone; and a project made private, its pages unknown
  at the next read. The goal it reads is under way: a landed goal's pages
  hold nothing a browser alone shows (4.6).
