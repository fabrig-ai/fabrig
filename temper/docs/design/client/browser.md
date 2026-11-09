# The browser

Provisional, 2026-10-05, revised 2026-11-07. temper's browser client: a
second shell of temper's client (`README.md`), beside the terminal, on
skein's browser parts. What its pages do is the client's (`README.md`;
`forge.md`). The greenfield mock (`greenfield/README.md`) is the look's
target, with later look requests, and takes precedence over `mockups/`
(the user, 2026-11-05). Journeys record and print what temper draws
(`journeys.md`). How a view runs in a browser is
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
  4. `/bundle/<digest>/…`: the bundle's files, immutable, each in the
     form the request takes (skein's `ui-web.md`, 4.4); any other path
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
  binary's, already in memory, copied (2.6). **Both forms stay whole in
  memory,** a file and its `NAME.gz`: the service holds no inflater, and
  a client not taking gzip is rare but is answered without one, a
  `curl` among them. A `NAME.gz` with no `NAME` beside it refuses the
  start, as no bundle does; a directory without its forms (built with
  `--gzip off`) serves `NAME` alone. A new bundle is served
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
  | `AllHome` | `/home` (`README.md`, 3.5, *Home on All projects*) |
  | `Changes { project }` | `/projects/<project>/changes` |
  | `Work { project, filter }` | `/projects/<project>/work`, and its filter's query |
  | `Settings { project }` | `/projects/<project>/settings` |
  | `Repositories { project }` | `/projects/<project>/settings/repositories` |
  | `Models { project }` | `/projects/<project>/settings/models` |
  | `Builders { project }` | `/projects/<project>/settings/builders` |
  | `Role { project, charter }` | `/projects/<project>/settings/roles/<charter>` |
  | `Rule { project, kind, number }` | `/projects/<project>/settings/rules/<kind>/<number>` |
  | `Ask { project }` | `/projects/<project>/ask` |
  | `Goals { project, filter }` | `/projects/<project>/goals`, and its filter's query |
  | `Team { project }` | `/projects/<project>/team` |
  | `Chats { filter }` | `/chats`, and its filter's query |
  | `NewChat { draft }` | `/chats/new`, and `?draft=1` with a draft |
  | `Accounts` | `/accounts` |
  | `Members` | `/deployment/members` |
  | `Limits` | `/deployment/limits` |
  | `Chat { task }` | `/chats/<task>` |
  | `Task { task }` | `/tasks/<task>` |
  | `Adopt { project }` | `/projects/<project>/adopt` |
  | `Welcome { step }` | `/welcome`, and its step's query |
  | `Forges` | `/forges` |
  | `ConnectForge { forge }` | `/forges/connect`, and its choice's query |
  | `ConnectModel { way }` | `/accounts/connect`, and its way's query |
  | `Authorize { query }` | `/oauth/authorize`, and the query as the address carried it, within jig's bound on an authorization's query (`../protocol/sign-in.md`, 9.3) |
  | `Approve { code }` | `/terminal`, and `?code=` the code as shown, `WDJB-MJHT` (`README.md`, 3.5, *The approval page*) |

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
  `ConnectForge`'s choice, `ConnectModel`'s way (below),
  `Approve`'s code (below) and
  `Authorize`'s query, kept as it came and never printed again, alone
  are in an address, and another page's query is dropped, `/chats/7?x=1`
  replaced with `/chats/7`.
- **Work's filter is its query,** as the chats list's: none for All, so
  that `/projects/<project>/work` is Work whole, and `?show=drafts` for
  Drafts. A `show` that doesn't read as Drafts, or a field of another name,
  is dropped, and the address replaced with the printed one. Work's other
  filters are their pages' addresses, Goals' and Changes' (`README.md`, 3.5,
  *Work*).
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
- **The approval page's code is its query:** `/terminal` is the page
  asking for a code, and `?code=WDJB-MJHT` carries one, the code as
  shown, four letters, a hyphen and four (`README.md`, 3.5, *The
  approval page*). The codec reads the first `code` field whose value
  reads by jig's `ApprovalCode::parse`: eight letters of the code's
  alphabet in either case, hyphens and spaces ignored, within 32 bytes,
  with no percent-decoding; it drops one that doesn't read, one named
  again after it and a field of another name, and the address is
  replaced with the printed one: `/terminal?code=wdjb-mjht` with
  `/terminal?code=WDJB-MJHT`, `/terminal?code=AEIO` and
  `/terminal?x=1` with `/terminal`. Its printing,
  `address::print_approve(code: Option<ApprovalCode>)`, `/terminal`
  for none, is the shown form, upper case, and the only one: the TUI makes its tab's address from the engine's
  address and this path alone (`README.md`, 7.1). The place is no
  page's address, as `Authorize` is none: the pages' links never lead
  to it, and the person reaches it by the address alone. The code is
  no secret (jig's `protocol/client.md`, section 8, *Its risk*), and
  the server's `Referrer-Policy: no-referrer` (2.2) keeps it from any
  other site the page might link to. temper's server answers `GET
  /terminal` with the page, as it answers every path that is no route
  of its own (2.2).
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
  place, even back on the main, and a focus request ends the wait. While
  `me` is read the main is empty and marked busy (`README.md`, 5.1), so a
  new entry then has the focus on it, and the page's heading drawn once
  `me` is read takes it alike, the page's main being that node (skein's
  `ui.md`, 7). A new entry with no main yet waits for its
  main alike: when the main is drawn, its heading takes the focus, or the
  main until the heading comes, unless the focus has moved or a focus
  request came meanwhile.

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
  the link's sign-in, as with any exchange: with a live link, a person
  still signed in keeps their sign-in, its number and expiry, and the answer sets no
  cookie, so the browser's stays as it was (jig's `domain/people.md`,
  section 3); a cookie the engine does not know, or one ended or
  expired, signs in afresh, with a new cookie. The welcome follows,
  which moves on to the chats page once the first run is over, and to
  the inbox, drawing nothing of itself, on a deployment with no welcome
  (`README.md`, 3.5, The welcome).
- **A reload, or another tab,** signs in with `Transport`, by the cookie
  (jig's `client/domain/client.md`, section 12).
- **Signed out,** by "Sign out" (`README.md`, 6.6; jig's sign-out clears
  the cookie) or with no link: the main is the view's
  signed-out column (`README.md`, 5.1), its heading "Signed out" and,
  in the browser's words, "To sign in, open a sign-in link temper
  printed." The view takes its shell's words at construction; the
  terminal's heading stands alone, with its own line (`README.md`, 6.6).
  With a way of signing in configured, the main is the sign-in page
  instead: "Sign in" and "Continue with Google", the ways
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
  signs in once there is room; opened past its expiry, it answers why
  (expired while remembered), and temper prints a fresh one if no link is
  live or reserved. The page never retries by itself, and keeps no secret:
  the address already says `/welcome`.
- **A used, expired or unknown link** draws the reason's paragraph in
  place of the sign-in page (`README.md`, 5.1), with no automatic retry.
  A link forgotten by the engine, after a restart included, is unknown.
  A cookie still live does not override that paragraph; a reload uses
  the cookie instead (section 5).
- **A terminal's approval** (`README.md`, 3.5, *The approval page*):
  the page at `/terminal` signs in with `Transport`, by the cookie, as a
  reload does. jig's cookie is `SameSite=Strict` under jig's prefix, so
  the tab's navigation, started by the terminal, carries none; the
  page's own exchanges, same-origin and started by temper's page,
  carry it, whatever opened the tab, as the `Authorize` place's do
  (`../protocol/sign-in.md`, 9.3). Its read and its decision go under
  jig's browser guards (jig's `protocol/browser.md`, 12.2), and the
  mapping tells the engine they carried the cookie, so the engine
  takes them as a browser's (jig's `protocol/client.md`, 17.3); `temper
  ctl`'s bearer is refused (`tools.md`, section 9). Signed out,
  the page says "Sign in first". With Google's way given, as the
  sign-in page reads it, it offers "Continue with Google", whose `then`
  is the page's address with its code, so the sign-in comes back to the
  approval (the look request `outside-pages`). Without it, the printed
  link is the way in: the page says to open it first, then the page's
  address again, since the link's sign-in lands on `/welcome`, not back
  on the page (`README.md`, 3.5, *The approval page*).
- **One person, on loopback,** as `README.md`, section 7 says. A browser
  on another machine is open (`README.md`, section 11).

### 2.5 The look

- **temper's style tokens,** under the owner's prefix `temper-`, on the
  nodes the mockups style: the frame's top bar or rail, navigation and main; the
  cards; the conversation's turns; the composers. The view sets them
  (`README.md`, 5.1); the terminal prints none.
- **The first paint, the placeholder's:** the shell's `loading.html` and
  `loading.css`, given to skein's bundler (skein's `ui-web.md`, 4.3), so
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
  says what it draws, and is the gallery's for the states no journey reaches (section 7).
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
  is said by colour or icon alone (skein's `ui.md`, section 8): a status
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
- **Connection flows** (`README.md`, 3.5 and 5.2): Settings and the
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
  `README.md`, 3.5 and 5.6), at its own scale, in a block of its own in
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
  faint on no fill. Every select under Settings draws `files/chevron.svg` 14 px
  in from its border, with 36 px of padding at its right, by one rule no other
  scope in the stylesheet outweighs. A proposal of settings' card takes the conversation's
  `.proposal`, with `.proposal.superseded` at `opacity: .5` and `.proposal-state.ok`.
  The mock's `.proposal-split` is the review page's (`README.md`, 5.13).
  `Forges` and `Accounts` take the same values on their rows (`README.md`, 3.5,
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
  area (`README.md`, 3.5 and 5.3). Needs you's open card takes the top bar
  with its own, `temper-needs-bar` (`README.md`, 3.5, *Needs you*), as a
  chat's bar does, the night switch keeping its place. The link's state
  shows at the rail's foot, with its dot, while it is not live. The person's
  menu goes with the foot, so the `Accounts` tab's row with "Sign out", which
  `temper.css` hides over 760 px, stands at that width alone (`README.md`, 3.5
  and 5.3).
- **The rail as the whole story draws it** (`README.md`, 5.3; the mockups'
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
- **Work's look** (`README.md`, 3.5, *Work*; the mockup's `work.html` and
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
  (skein's `ui-web.md`, 4.3), the playground's, the setup pages', and the
  gallery's and journeys' prints. So what sits at the foot keeps off a
  phone's home bar by `env(safe-area-inset-bottom)`, which is 0 on a display
  without one and so measures as the mock draws it, at 760 px and under: the
  tab bar's padding, a chat's foot, and the toast, 88 px over it, clear of the
  tabs.
- **What a phone presses,** at 760 px and under: a control scrolled into view
  stops clear of the tab bar, and every target stands at least 44 px tall to
  a press (`../draft/surfaces.md`, section 10): the connection flows'
  primaries, Settings' and a dialog's, and every control of the frame, Home,
  Needs you, Work, Ask temper's page and an Ask temper chat. A control the mockups draw smaller,
  the Away chip's 36 px pill, the night switch's 30 px circle, a way's tab,
  keeps its drawing and takes a 44 px target around it, by its padding or a
  transparent `::after` over it. A link inside a line of words is no target
  of its own. Every other page keeps what the mock draws.
- **No frame but the top bar outside a project** (`README.md`, 5.1; the
  look request `outside-pages`): a page that is no project's has no
  navigation, switcher or menu, at any width; the top bar is drawn as
  *Outside a project* (below) says.
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
- **An Ask temper chat's tag** (`README.md`, 3.5), `CHATS_TAG`'s " · Ask
  temper" after the chat's number in its bar, takes the number's look,
  `.temper-byline`'s 12 px and faint, on one line, the bar's gap before it
  taken back so that its own " · " is the only space and the two read
  "T9 · Ask temper". At 760 px and under, where the number goes, the tag
  stays without its " · " (the look request `ask-temper-place`). The bar's
  spend paragraph stays hidden: the hiding rule excepts only the number's
  `.temper-byline` and the tag's `.temper-chats-tag`.
- **A chat's document** (`README.md`, 3.5) is `configure-chat-draft`'s rules, in
  `temper.css` under the same marker: `temper-mode-chat`, `-split` and `-draft` on
  the session, `.temper-doc`, `.temper-doc-inner`, `.temper-doc-path`,
  `.temper-doc-sec`, `.temper-doc-changed`, `.temper-doc-note`, `.temper-seg`, and
  the rail's strip with `.temper-rail-unfold` and `files/menu.svg`. What those
  name is drawn; a flow takes the tokens' look below. The sheet to start a goal
  is the mock's `dialog.sheet`, `.option`, `.field` and `.text`, and `Ctrl+\` steps
  through the modes.
- **A chat's terminal** (`README.md`, 3.5) is the requests `chat-terminal`,
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
- **Ask temper's ways** (`README.md`, 3.5 and 5.16) are `ask-temper-ways`'s rules, in
  `temper.css`: the opening's, `.temper-ways` with its tabs, axis, panel, card, legend
  and act, under `/* ==== ui: settings ==== */`, where `.temper-starter`'s stand; a
  path's, `.temper-ways-path` and its marks, under `/* ==== ui: chats ==== */`, which a
  reply's views share. Where skein draws an element otherwise than the request's page (a
  tab an `<a role="tab">`, a scene's stop a `<g>` with no ARIA, an open stop a token, one
  SVG laid out by width where the page has two), the selector names what is drawn and
  keeps the request's declarations.
  The drawing's rules (the path's and the legend's) and the tabs', line's,
  card's and act's hold under How work runs' frame as under Ask temper's
  page (`README.md`, 5.13), each selector naming both. With a fourth tab,
  "Yours", at 760 px and under the tabs stand two by two, as the whole
  story's `whole-story.css` has them, so that "Agents review" keeps its
  words whole on a phone.
  A path's marks take no stroke but their own rules', as the request's
  page has none: skein's default stroke for a scene's marks (skein's
  `ui-web.md`, 12) gives way under `.temper-ways-path` by one rule of the
  path's, scoped to its drawn figure, `stroke: none` on the marks skein's
  rule names, outweighing skein's rule and outweighed by every mark's own
  (the line's, the boundary's, a person's ring, the halo, the caret, a
  focused stop's area); skein's rule stays as it is.
  Who acts is the whole story's marks (`web/whole-story/whole-story.css`,
  `.ways-*`; `../draft/surfaces.md`, 13), orange the person's alone: a
  person's ring `--gf-paper` stroked `--gf-signal` at 2; an agent's star
  `--gf-ink`, and outlined when needed, `--gf-paper` stroked `--gf-ink`
  at 1.3 with round joins; code's square `--gf-muted`, and outlined,
  `--gf-paper` stroked `--gf-muted` at 1.3; a stop's small words 10 px
  in `--gf-faint`; the legend's agent "✦" in `--gf-ink` and its agent
  when needed the outlined star. The line's base is `--gf-line-2` at
  1.5; each rise's fade (`README.md`, 5.16) is the look request's, its
  stops coloured by their tokens: a person's `--gf-signal` at .8 at its
  mark, .28 at the dotted line, `--gf-line-2` at the base; an agent's
  `--gf-ink` at .3 and .1, the whole story's lighter tone for an agent
  (`.ways-rise.a`), then `--gf-line-2`. Night takes the same tokens'
  night values.
- **How work runs' look** (`README.md`, 5.13, *The page*) is the whole
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
  - *on a phone,* at 760 px and under, the column takes the frame's phone
    width and the path runs down the page, as on Ask temper's page.
- **A flow** (`skein/docs/design/ui.md`, 11.5) has the look of
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
- **Team's look** (`README.md`, 3.5, *Team*; the whole story's `team.html`
  and `team/team.html`, and `whole-story.css`): its intro as Work's and
  Home's, kicker, headline and lede, in a column 1020 px wide at most
  (`.col.wide`); People's head the mockup's `.shead`, its title at the
  left; the people a grid of boxes, `.grid3`, columns at least 240 px
  wide filling the row, 16 px apart, each box `.box`, on the surface's
  ground with the hairline's border, 12 px round, 18 px by 20 px inside,
  8 px between its lines: the name in bold, the role in `small` and
  `muted`, the line in `small`. On a phone the grid is one column, each
  box full width. Its rules go under `temper.css`'s marker `/* ==== ui:
  team ==== */`, before Settings', on tokens of its own, `temper-team*`.
- **A page target draws no ring:** anything given `tabindex=-1` only to take
  focus (`main`, a heading, a document's section) draws no outline, by
  `skein-ui.css` (skein's `ui-web.md`, 12), and `temper.css` says so again of
  `main` and a level-1 heading; a control keeps its ring.
- **A scrolling pane holds its hidden parts:** a chat's log and its draft pane are
  positioned, so a visually hidden absolute part (a who-line) is clipped by the
  pane's overflow and does not lengthen the page: a long chat's document never
  exceeds its window.
- **Day and night** (the look request `night-mode`, 2026-11-07, a first pass the
  user keeps to refine): the mockups' palette is the day's, and `temper.css` sets
  `color-scheme: light`, where `skein-ui.css` follows the system's dark
  preference. *Night* is the same pages on `html[data-night]`, which the shell
  sets from the kept look (`README.md`, 3.5, *Its look, kept per browser*):
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
  - *The switch* (`README.md`, 5.3): `temper-night-switch`, 30 px square, no
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
    night (2.5, *A chat's terminal*); the placeholder's first paint is the
    day's, since it draws before the shell reads the kept look. The rest of the
    client takes the tokens as they fall, and no page is left with words in
    the day's ink on the night's ground.
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
  scrolled away (skein's `ui-web.md`, section 7). Where the window has
  let its newest lines go, live text waits until the end is read again
  (`README.md`, 5.15).
- **Reading the conversation** (`README.md`, 5.15): the bound log's
  head and foot ask for earlier and later lines as they near the view,
  with no Older control. PgUp and PgDn scroll it from the composer too,
  except while a menu or dialog is open. skein's page part measures and
  restores the words' position when lines are read above, including in
  a browser without `overflow-anchor`; Latest or sending on a chat
  returns to its end, reading it again where the window let it go.
- **Outside a project** (`README.md`, 5.1; the look request
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
    (`README.md`, 5.1); `files/brand.svg` is not in the bundle; the
    stylesheets' accent, `--temper-accent` and the light terminal's
    `--accent`, is the signal, `#c2410c`. Two focused tests keep it so:
    every class `temper.css` selects is one `look.rs` names, and no
    source of the bundle or of the server's pages names the old mark's
    `#b8664e`.

### 2.6 The bundle in the binary

The `temper` binary holds its bundle, so that `temper box` and `temper
serve` serve the browser with nothing beside the binary, and `cargo
build` in a checkout makes that binary. The engine's binary,
`temper-engine`, holds the same bundle through the same crate, the only
one of a deployment's binaries that does (`../protocol/README.md`,
4.2). It is a person's convenience:
the machine's builds keep building the way that serves their work,
without it.

- **One file.** The binary holds the bundle: the shell's module, its
  glue, and the stylesheets and files skein's bundler puts beside them
  (2.2, 2.5), and their compressed forms (skein's `ui-web.md`, 4.2).
  Running it is the whole install: no directory beside it,
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
     title, the prefix `/bundle/`, `temper.css`, `task-pages.css`, the
     placeholder's `loading.html` and `loading.css`, the shell's `files/`
     and the noscript words), into its `OUT_DIR`;
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
  pins the git dependencies; the shell's stylesheets, its placeholder and
  its `files/`;
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
  a plan's sign-in alone (4.2).
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
   the reason-specific signed-out paragraph (`README.md`, 5.1). After a
   server restart, reopening the first printed link shows "This link
   isn't from this temper. Use the one its terminal printed." because a
   restart forgets retired link digests (jig's `protocol/client.md`, section 8).
   That the secret leaves the address before the first request is held by
   the shell's start order (2.1), and is not tested.
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
5. **Reading back:** scroll a chat's log up; Latest appears. New words
   arriving while reading back leave the view where it is. Press Latest
   or send from the chat; the end returns, read again if the window let
   it go, and the pill goes (`README.md`, 5.15).
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
behaviour brings its story with it: `ConnectModel`'s sign-in, a tab of
its own sent to the provider's address while the page stays
(`README.md`, 3.5).

**`ConnectModel`'s sign-in, in a new tab.** What only a browser shows:
a press opening a tab within its gesture, the answer pointing it, the
provider's redirect followed to the engine's listener in that tab, and
the page, which stayed, drawing the end.
- **Against:** `temper serve` with two synthetic providers sharing an issuer,
  its authorization, token and key addresses, skein's fake issuer
  (`skein_fake_peers::oauth::Peer`), hosted on loopback in the
  harness's loop, each grant a success for one synthetic subject, and
  the provider's redirect port the kernel's (the listener's port
  varies). The fake has no TLS, so the tier's bundle is built with
  skein-ui-web's `loopback-navigation` (4.3).
- **The story:** a person on `ConnectModel` presses "Sign in to Test provider";
  one more tab opens (skein's `ui-web.md`, 14.2, Its tabs),
  which can't reach its opener by the time it's at the listener, its
  address the listener's; the page's own address reads
  `/accounts/connect?provider=17&way=sign-in&source=S`, in place of the
  entry it had; the page shows Ready ("Signed in to Test provider"); the fake
  has taken one authorization and one code exchange. A reload then shows
  Ready again and the issuer has taken no second authorization: the tab
  is pointed once.
- **The welcome's Model step,** the first thing a person meets: the same
  press holds the welcome at `/welcome?at=model`, and it moves on to Ready
  by itself, `/welcome?at=ready`, with one authorization and one code
  exchange of its own. It runs first in this story's test (4.6, 9), on a
  second provider of the same issuer, Welcome provider (18), with a model of
  its own: the issuer's one subject signing in twice on one provider renews
  its account, in other words than Ready's
  (`README.md`, 5.5), and a usable model leaves the welcome no sign-in to
  press. Each sign-in keeps its tab, so the story ends with two.
  The client world's `suite::second_sign_in` holds that the second provider's
  sign-in reaches Ready after the welcome's.
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
  `loopback-navigation` (`scripts/bundle.sh --loopback`), so that a
  sign-in's tab (4.2) can go to the fake issuer on plaintext
  loopback; the ordinary binary bundle never does (skein's `ui-web.md`,
  section 10). The tier names that bundle in its configuration, so it
  runs alike over a binary built with or without its own (2.6). It holds
  the compressed forms and Chromium takes gzip on loopback, so every
  test loads the compressed module under temper's policy.
- **The tier's window:** the tier's bundle also turns on
  `temper-client-shell-web`'s `small-window`, which names a conversation's
  window of 32,768 bytes, a thirty-second of a person's `document_bytes`,
  through the same window mechanism (`README.md`, 5.15). A window holds no
  less than `long_text`, so the feature sets the bundle's `long_text` and
  `turn_text` to the same 32,768: the largest page `read_docs` answers
  (`temper-docs`' `PAGE_BYTES`), so that a tier story's line holding one
  stays whole. The long reading
  story (4.6, 14) passes that bound by at least two windows' worth and
  evicts and recovers lines at both ends. The unanchored position story
  stays within one window; it checks the words holding across a read above,
  and a window dropping lines would change what that proves, so it never
  reads the long story's chat.
  Reading's replies are sized to the window: a reply's bytes ride on one
  horizontally scrollable code line, its height on its paragraphs. So the
  code lines scale with the window, and each history's bytes keep their
  share of it, while the replies' counts and paragraphs stay: the keys
  each step presses and the reply each step reads rest on them.
  The ordinary binary bundle and the journeys'
  never turn it on, and a long chat at the whole bound is read by hand.
- **A chat's history:** what a reading story reads is made through
  `temper ctl` before a page opens on it: `playground start`, `chat start`
  and `task message`, each answer waited for with
  `task show --until replied`. A face of its own signs ctl in with the printed
  link and opens the spent link, so that the server prints the reader's, as
  8.4's walk does. A page types only what its story is about: in reading's,
  the message sent from far back. Reading's test (4.6, 14) holds three chats
  on one `temper box`, as few as its stories allow: T1, the short chat, which
  six messages sent through ctl then make Latest's; T2, the position story's,
  inside one window; T3, the long story's, past three. Their requests cue
  disjoint scripts, and their replies together stay within the fake's joint
  script allowance (`temper-fake`'s `SCRIPT_BYTES`), both held by a focused test
  of the tier's crate.
- **Anchoring within a step** (4.6, 14): reading's test serves a private
  bundle whose stylesheet, only at a page one pixel taller than the kit's
  1280 by 800, turns the browser's scroll anchoring off on the log
  (`overflow-anchor: none`) and sets the log 7 px lower. The position story
  goes to that height on its live page and back at its end, so its words stand
  on the page's own anchoring while the other stories stay on the browser's.
  That the log moved by those 7 px says the rule holds, and the next story's log
  standing where the position story's first did says it has gone.
- **The playground's test** (4.6, 12) runs on the built playground
  instead, `scripts/bundle.sh --playground` in its own profiles
  (`../playground.md`, 7), served as static files with their `headers`
  by the tier's own static server, which maps them as a site's host
  would; the tier reads the world's figures from the page's frame, and
  compresses the download's with the machine's `gzip` (`../playground.md`, 9).
  That bundle has no compressed forms (`../playground.md`, 7).
- **A cap: at most 12 tests, and 120 seconds** on the slowest of the
  machines that build temper (the user, 10-27, temper#15). Past it,
  stories that share a setup become one test of several steps before
  any browser-only proof is dropped, and what must leave the browser
  goes to the lowest world that still shows it, named in its line of
  4.6; the cap stays. The tests are 4.6's.
- **A test of several stories** says which failed: it marks where each
  story's steps begin (the tier's `Stories`), and a failed step's report, and
  each measure read after the run, name its story. Each story's assertions
  stay as its steps, in its order. It reruns whole on a flake, and a failure
  hides the steps after it: taken, for the cap. It stands on the same forty
  seconds as any run of the harness (skein's `ui-web.md`, 14.2): a test that
  can't fit them is two tests, never a longer run.
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

### 4.6 The stories

The tier's tests, each on its own `temper serve` (the playground's on its
built playground; 2 and 11, 13 and 14 on `temper box`, whose first run is
over). Each is one test of the steps it names; 2 and 11 share one, and 4 and
5 another; 8 and 9 share their Accounts setup, and 16 shares its sheet's
setup across desktop and phone widths. The twelve test functions in
`tests/client-browser/tests/browser_all` are:

| Stories below | Module | Test |
|---|---|---|
| 1 | `browser_sign_in` | `signing_in_reloading_reconnecting_and_signing_out_keep_the_cookie_contract` |
| 2 and 11 | `browser_pages` | `history_keeps_the_chat_focus_and_filter_across_reload_and_back` |
| 4 and 5 | `browser_pages` | `enter_starts_one_chat_and_the_composer_keeps_the_focus` |
| 6 | `browser_attacks` | `cross_origin_requests_and_framing_cannot_act_as_the_person` |
| 7 | `browser_authorize` | `a_cross_site_authorization_returns_a_code_and_the_links_state` |
| 8 and 9 | `browser_accounts` | `the_sign_in_opens_a_tab_and_lands_on_accounts` |
| 10 | `browser_github` | `the_manifest_posts_once_and_history_waits_for_continue` |
| 12 | `browser_playground` | `story_a_walks_on_the_built_playground` |
| 13 | `browser_public` | `public_board_and_task_lines_are_readable_at_desktop_and_phone_widths` |
| 14 | `browser_pages` | `a_chat_opens_at_its_end_and_reads_back_keeping_its_place` |
| 15 | `browser_pages` | `closing_project_rename_returns_focus_to_the_summary` |
| 16 | `browser_connect_flows` | `scrolling_the_sheet_reads_more_then_search_picks_and_adopts` |

1. **Signing in** (4.1, 1): the printed link signs in, a reload on a
   chat signs in by the cookie, and the link spent shows the
   signed-out words; signing out clears the cookie on later loads. The same
   chat reconnects after a server restart and remains live after rapid reloads.
2. **History** (4.1, 2): from the chats to a chat by its link, Back and
   Forward; a new chat's composer focused, and its filter's address and
   pressed state kept across a reload and Back. The story awaits Closed
   pressed after reloading `/chats?state=closed`, then All pressed after
   Back to `/chats`, each by its exact button name within `WITHIN`
   (skein's `ui-web.md`, 14.2), independently of a Drafts button arriving
   before Closed. An address that is no page opens
   the inbox at `/` (4.1, 2). Its chat's model scripts a run of calls: the folded
   line hides its rows until a press, and stays open on its node across a reply,
   `details[open]` being the browser's, which the model doesn't hold
   (`README.md`, 3.5).
4. **The composer** (4.1, 3), and its model picker (`README.md`, 3.5): one press
   applies a model and an effort, at 420 px and 1440 px, and a press away or
   Escape closes it.
5. **A panic** (4.1, 4).
6. **jig's attacks that need a browser** (4.1, 6): a cross-site request
   from another port and from another host does nothing, and a framed
   page shows nothing, in one harness with both attacker host names.
7. **A cross-site authorization** returns its code and the link's state
   (4.5).
8. **The accounts page after a key's send:** the completion returns to Accounts
   with the key's row; Back restores an emptied field, Forward the chats (4.2).
9. **`ConnectModel`'s sign-in, in a new tab** (4.2), after the welcome's Model
   step's, each with one authorization and one code exchange. The replaced
   history entry returns to Accounts, and its counted heading remains through
   Back, Forward and reload while the completion message disappears.
10. **GitHub's manifest page** posts itself once, and Back shows it
    without posting again until Continue (4.4).
11. **The box's project across a reload:** the playground's project
    made, the Chats page Live, never Behind, with the story's chat,
    before and after a reload (`README.md`, 2). Before that chat, at
    420 px, Home's To begin and its Start open Ask temper's page, whose
    composer and model picker end above the three tabs' top (2.5, *The
    project's frame*), and whose ways show their tabs, Agents review
    current; "Draft first" pressed shows its line, its path and "Use this"
    (`README.md`, 3.5).
12. **The playground's story A** on the built playground, its log
    downloaded and replayed natively (`../playground.md`, 9).
13. **Reading a public project** (8.4), with no sign-in, its board and task
    lines readable at 1440 px and 420 px, and losing it when it is made private.
14. **Reading back** (4.1, 5), one test of four stories, each on its own page
    load (4.3): a short chat opens whole, with no reading status, both its ends
    in the log; Latest returns a chat to its end, after new words and after a
    send from far back, the view holding while it reads back; earlier words stay
    put without the browser's scroll anchoring; and a long chat opens at its end
    and reads back from the composer, past its window at both ends.
15. **A project's rename** closed by Escape, Enter and Save returns the
    focus to its summary, and never to the page's `main`.

16. **The repository sheet** (`README.md`, 4.4; `forge.md`, 5.1): scrolling
    the last loaded row into view reads the second page without changing the
    list's bounds; search finds its repository, selection counts it, and adoption
    returns to its row. On the same deployment at phone width, a reopened
    sheet meets the viewport's bottom and keeps its primary button visible.

The cap includes all browser tests, whatever their names.

Not in the tier, each in the lowest world that shows it:
- **Models switched off and back, and added by name, across reloads:** the client world's `models::a_switch_turned_off_and_back_holds_across_a_reload` and `models::a_model_added_by_name_is_on_the_picker`.
- **A run of calls' rows and their words:** the client world's
  `suite::call_line::opened_it_holds_one_row_a_call` holds one row a call, its
  first line alone. Test 2 keeps the closed line hiding them and the open line
  across a reply.
- **Goals' link to a draft:** the view's
  `tests::goals::the_link_shows_with_a_tracked_template` holds its button
  opening the Start goal sheet with no draft. The client world's
  `suite::chat_draft::a_goal_from_goals_asks_its_person_and_the_answer_wakes_it`
  starts by a typed Draft path and answers its person task. Test 2 holds
  what a browser's history does with a link.
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

- **The two-step Home and night comparison** (2.5, 7.2): temper/ui
  compares `ask-temper-start`, `to-begin-two-steps` and `night-mode` after
  landing, including their named states, open menus and disclosures at
  1440, 1000 and 420 px (`../draft/look-requests.md`, section 4).
  Include `home-begin-*`, Home with a need and a chat left out with
  either step left, and their return after the last step or a skip.
  Check Ask temper's restricted rail, its three tabs at 420 with the
  composer and model picker wholly above them, the same rail and tabs in
  `ask-first-run`, `ask-typing`, `ask-sending` and `ask-refused`, and
  `night-home`, `night-ask` and `night-chat-split`,
  the switch in the
  folded rail open and closed and at a phone's chat bar, and the night's
  shadows against their day values (2.5, *Shadows keep their colour*). Check
  the faint top-right glow without rings and readability on every other
  page; refine the first night palette with the user. Run the history
  story's phone Ask row (4.6, 11) and night reload row after landing.
  Default: tokens, focused tables and gallery prints establish no
  rendered equality or browser runtime result.

- **Ask temper's repaired bar and first-run gallery** (2.5, 7.2):
  temper/ui compares the `ask-temper-place` tag at 1440 and 1000 px as
  "T9 · Ask temper", and at 420 px as "Ask temper" alone, with the
  spend paragraph still hidden. Include the first-run states' rail row
  and phone tab, and reference chats without a false Ask temper tag or
  current row (`../draft/look-requests.md`, section 4). Default: the
  stylesheet and gallery fixtures; their checks establish no rendered
  equality. The Ask page's secondary heading, "How should temper run
  Hello?", stays until a later look round with the user.

- **The terminal approval page after landing** (2.3 to 2.5, 7.2):
  check `/terminal` in a signed-in browser on a fresh box, its typed-code
  entry and the twelve `approve-*` gallery states at narrow and wide
  widths. Domain, view and shell tables and gallery prints establish no
  rendered browser result. The TUI's own flow and walk follow
  `../draft/tui.execution.md`, M2.
- **Terminal approval at a public name** (2.4): Google's browser sign-in,
  the code typed rather than carried in the address, the request's origin
  and age, and a person's list of sign-ins with each ended from there
  (`../draft/tui.md`, section 7). The loopback page signs in by the
  printed link, or by Google's way where it is given.

- **The outside pages' comparison** (2.4, 2.5, 7.2; `README.md`, 3.5
  and 5.1; `forge.md`, 9.1; `../protocol/sign-in.md`, 4.3): temper/ui
  compares the look request `outside-pages`'s pages 01 to 10 under their
  overlay with the reprint after landing, at 1440, 1000 and 420 px
  (`../draft/look-requests.md`, section 4), with the states its message
  names in words: the top bar while `me` is read, offline and behind; the
  public pages' "Sign in"; every approval state, deciding and signed out
  with and without Google; each `Authorize` state; out of date; no
  project yet with and without the welcome; the GitHub App's setup pages;
  and night. It takes in the six repairs of `look-round-22`: the
  welcome's top bar, signed in, ending with the person's name and "Sign
  out" after its steps at each width; the top bar over an empty main
  while `me` is read at `/chats` and `/chats/new`, until "No project yet"
  settles, with the heading's focus once the page comes (2.3); "Code" and
  "Project name" at 14 px on a 22.4 px line, the field and its act
  1.59 px lower; the night welcome at 420 by 900 with no day paper under
  its ground; "Sign in" on the public pages without Google; and the
  Google endings' typographic apostrophe. Default: the stylesheets, the
  focused tests and the gallery's prints establish no rendered equality.

- **The chat terminal's comparison and keys** (2.5, 7.2; `README.md`,
  3.5): temper/ui compares the four `chat-terminal` requests' pages,
  rules and named states with the nine gallery states and live page
  after landing, at 1440, 1000 and 420 px, with menus and disclosures
  open (`../draft/look-requests.md`, sections 4 and 5). Include dark
  and light, both fonts, phone Split, a draft being written, tables,
  changed-section flash, the figure, questions, proposals, delegations
  and failures. Include the phone Split conversation and composer's dark
  ground, the draft's thin scrollbar without a reserved gutter at every
  width, the quiet model button's line height and the running line's
  placement inside the foot without a gap above the box. The ordinary
  bar's two-row title at 1000 px stays: `>_` and its status words share
  the bar, by the user's `look-round-17` request.
  Real keys check the full-screen behaviours named in
  `README.md`, section 11. Default: the shared document rules and
  focused tables and prints; these establish no rendered equality or
  real-key result. Temper/ui retains the departures in
  `browser.status.md`, 2.5: Plex's phone size, adjacent-note spacing and
  options unchanged on hover.

- **A filter row while its project's ways arrive** (2.3, 4.6): Drafts
  appears before Closed once the ways offer drafts, moving Closed under
  a person's pointer. Default: keep the conditional Drafts button; a
  separate look decision settles whether the row should reserve its
  place. The history story reads the filter by name and pressed state,
  and makes no claim that the row stays still.

- **Settings, Models comparison** (2.5; `README.md`, 3.5): temper/ui owns
  page 07 of `model-picker` beside the shipped page, its named states and
  screenshots at 1440, 1280, 1000 and 420 px after landing
  (`../draft/look-requests.md`, section 4). Default: host view and client
  world checks; compilation alone establishes no rendered equality.
  Include Provider's select chevron, the add form's single row and the
  Provider and Add headings. The shared Settings H1 and open project
  menu row keep their frame's design, as the repair request leaves them.

- **A flow in a chat and its document** (2.5; `README.md`, 3.5): temper/ui
  compares both against `skein/docs/design/draft/diagrams/preview.html`
  after landing, at a width where the flow is horizontal, one where it
  is vertical, and one where only the fallback fits. Default: the host's
  byte comparisons and the stylesheet's rules until that comparison.

- **The connect-flows proof** (2.5, 4.6): temper/ui owns the comparison
  of request pages 01–23 with main, at 1440, 1000 and 420 px with
  disclosures and menus open, and screenshots also at 1280 px
  (`../draft/look-requests.md`, 4). The retained browser stories check
  Accounts return/history and sheet scroll, search, adoption and phone
  geometry; they do not compare all request pages. Default: findings
  after landing, with a passing repaired-story run and the unchanged
  whole-tier budget established by the after-landing tier. Compare the
  request-owned parts, including select chevrons, the source chooser,
  Copy public's caption and four-line box, list margins, empty search,
  More, control captions, connection actions and model-way paths. The
  shared Settings H1 and open project menu row keep their frame's design.

- **Whole questions and starting from Goals** (2.5, 7.2; `README.md`,
  3.5, 5.8, 5.14): the long question's context, evidence and choices, the
  typed Draft sheet, zero-funding explanations and change-only landings
  need their browser, walk and render checks after landing. Default:
  shared view and domain tables, focused client stories from a chat and
  Goals, and gallery prints; no new browser test for `ui-goal-asks-2`.

- **A failed link with a live cookie** (2.4): falling back to the cookie
  instead of keeping the link's reason is jig's client question
  (`jig/docs/design/client/domain/client.md`, section 18). Default: the
  reason remains until another sign-in; reload to use the cookie.

- **The fresh project's draft flow** (`README.md`, 3.5, 5.3, 5.13):
  the real-box check of an Ask temper chat's draft offer and the draft-to-goal
  flow, including Goals in the phone tab bar, is after-landing work
  (`README.md`, section 11). Default: the view tables and fake-model
  `chat-draft` journey, with no browser or render check during the plan.

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
- **A box still starting:** a link opened before serve is ready gets no
  bytes until it is, so no placeholder either, and that wait stays the
  browser's own (`../protocol/setup.md`, 7.1). Default: as it is.
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
- **Night following the system,** a first visit taking the system's dark
  preference as `skein-ui.css` does. Default: a browser with no kept look
  is day, and night is the switch's alone (2.5, *Day and night*).
- **Night before the first paint,** the placeholder drawn at night when the kept
  look says so, before the shell starts. Default: the placeholder is the day's
  (2.5, *Day and night*).
- **Other browsers** than Chromium are `skein-browser`'s question
  (skein's `browser.md`).
- **One shared stream per public document,** fanned out by the server
  to every visitor reading it, in place of each page reading again
  (section 8). Default: none; pages read again at the `max-age`, and a
  visitor never holds a stream.
- **Public history** (8.3): ended goals from before publication or across
  a restart need jig's source index (`jig/docs/design/domain/people.md`,
  section 12). Default: goals ended while the public index kept the
  project, and the board's line says what it leaves out. A merge from
  before publication can show while its change is handed from a live
  goal's tree; publication is not a commit-time cutoff. A merge past the
  forge's log is `forge.md`, section 10's.
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
- **A phone's width in a test** (2.5): skein's browser kit opens pages at
  1280 by 800 and offers `Viewport` to change their CSS pixel dimensions,
  with desktop input behaviour. The public board and task story (8.4)
  checks 1440 px and 420 px. Other phone rules remain held by their sheet
  checks and throwaway walks alone: the accounts tabs, owner menu, rail,
  chat and document panes have no retained viewport story.
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
  again at `skein-browser`'s `poll`, 50 ms. Default: that poll stays. Repeated-miss
  backoff needs a measured saving without delaying presses or the notice of a
  change; the repaired long story's query count alone gives no reason to add it.
- **The remaining browser cost** (4.3, 4.6): another repair may prepare the
  other stories' histories outside the page, and combine stories sharing a setup
  before dropping a browser-only proof. Default: keep the CPU budget, timeouts,
  membership and cap; the four reading stories retain their layout, focus and
  scroll proofs. A whole-tier run after landing decides its remaining cost.
- **Merged stories after landing** (4.2, 4.3, 4.6): check the call line's
  open node across a reply, the composer's model picker, both sign-in tabs
  with the welcome first, and reading's four page loads with anchoring off
  only for the position story. Default: compile during the plan; run in
  Chromium after landing, preserving every assertion, the forty-second
  run deadline and the whole tier's 127.5 CPU-second budget, 85 raised by
  half by the user's word of 11-08, for now, to be revisited. A whole-tier
  run supplies the cost proof; separate story measurements do not.
- **Reading's test past its run's forty seconds** (4.3; 4.6, 14). Default: it
  becomes two tests, the long chat's apart, and test 15, which shares its setup
  with 4 and 5, becomes a step of theirs, so that the tier stays at twelve.
- **A long chat at the person's whole bound** (4.3; `README.md`, 5.15): the
  small-window stories prove eviction and recovery through the same mechanism,
  but leave the 1,048,576-byte window's manual reading check separate. Default:
  keep the person's bound and check opening, position, both ends, Latest and
  sending there by hand.
- **A public page's plain text in one query** (8.4): the page-text walk,
  apart from geometry measurement, reads a node at a time, about 0.8 CPU s
  over its seven pages. Default: read a node at a time, accepting 8.4's
  queued and unqueued landings forms, until that passes 1 CPU s, when the
  browser kit would read a page's text at once.
- **Every wrapped text fragment** (8.4): checking all of a title's wrapped
  fragments needs a browser testing interface decision (skein's
  `browser.md`, 11). Default: keep `Seen` and its first content quad;
  missing rectangles remain failures, with the text and measurement stage.
- **A public page changing during measurement** (8.2, 8.4): one terminal
  boxed query avoids repeating every rectangle request for each text node,
  but is no atomic snapshot of the page. Default: keep the field, gutter,
  wrapping, navigation and overlap assertions and read a missing box's text
  and stage from its failure. A node gone between requests refuses the
  query, which fails the page assertions; that alone does not explain an
  answered query with a missing rectangle. A timer-driven read remains an
  unproved explanation.
- **Expected browser errors scoped to an act** (8.4): the public walk's
  sign-out permits an expired stream's 401 console message. Expectations
  match text for the whole walk, so the same words from a later public
  read are accepted too. Default: keep that expectation; distinguishing
  the act or resource needs a harness interface decision.
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
  shell's: the page's heading, then "temper". A state whose root's kept
  look is night, its switch pressed, prints `data-night` on its `html`, as
  the shell sets it (2.5, *Day and night*).
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
- **A path printed for both widths:** a static page measures no box, so
  each path's drawing (`README.md`, 5.16) is printed twice in its figure:
  across, drawn in a box 720 wide, and down the page, drawn in a box 388
  wide, the box at 420 pixels with a phone's 16 pixels each side
  (`README.md`, 3.5). The page's own style, in its head, shows the second
  at 760 pixels and under, the shell's phone width, and the first above
  it, so that the print at 420 runs the path down the page as the shell
  does.

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
  contract holds one. A project lists the templates its box would offer:
  the first run's worlds the stock's chat (1), goal (2) and Ask temper
  (5, charter 11), as a fresh box does, so that their pages draw Ask
  temper's rail row and tab (the look request `ask-temper-place`); the
  reference world Ask temper at 5 too, and its own ways at numbers the
  stock leaves free; the solo world Ask temper in its Ask temper states
  alone.
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
| `all-home` | the at-scale world's four projects, vsr, vsr-rs, cowsql and fabrig: three new briefs, two needs across two projects, four decisions, fabrig's main red; the whole story's `home.html`, and `phone-home.html` at a phone's width |
| `all-home-nothing-new` | the four projects with nothing landed since the last look, no brief new and no need: the kicker, "The last change landed SPAN ago.", "Nothing needs you." and Projects |
| `all-home-one-project` | one project read: no Projects, the headline "N changes landed in PROJECT." |
| `all-home-away` | `all-home` with Away on for vsr-rs and cowsql: the switch and the chip "Away on 2 of 4", not pressed (`README.md`, 3.5, *Away*, *On All projects*) |
| `all-home-away-on` | `all-home-away` after a press, each answer Away: the switch "Away until 08:00", pressed, and the toast "Away turned on for 2 projects: agents decide within their limits." |
| `night-all-home` | `all-home` by night |
| `home-step-pair` | a project's Home under You approve: a brief, what landed, an approval's card leading with "Review on GitHub ↗" and a question (`../draft/surfaces.md`, 12, Pair) |
| `home-step-team` | a project's Home with Decided for you and a question (12, Team) |
| `home-step-stock` | a project's Home with a brief and Decided for you, no need (12, Stock) |
| `home-step-scale` | a project's Home with a question and a funds card (12, At scale) |
| `needs-list` | Needs you across two projects, three open: the first open, the others closed rows; Decided for you under them; the whole story's `needs.html`, and `phone-need.html` at a phone's width |
| `needs-list-second` | `needs-list` with its second row opened |
| `needs-tell` | `needs-list` with words written in "Or tell it something else" and Send shown |
| `needs-approval-first` | Needs you with an approval's card open, "Review on GitHub ↗" first |
| `night-needs` | `needs-list` by night |
| `home-try-stopped` | a failed trial below Home's intro: "WHO didn’t answer when tried" and Try again; no change to its counts |
| `home-away` | hand-made Home deliveries: `AwayRow` on, four `Decision`s and live tasks; the mock's `home.html` in *Back from away* |
| `home-away-one` | hand-made Home deliveries: one `Decision`, the switch off: "One thing was decided for you." and the kicker's two times |
| `home-away-overturned` | hand-made Home deliveries: a row overturned, its task live, and one ended, "landed 03:12" |
| `rail-away-off` | a project’s page after turning the switch off, with its toast open |
| `rail-away-on` | a project's page with the switch on, "Away until 08:00", and its toast open |
| `settings-away` | *How work runs* with the section `#away`, the defaults ticked |
| `home-begin-both-left` | hand-made Home deliveries: both steps left, the repository first; first repository offered; the rail's Home and Ask temper alone, the phone's three tabs |
| `home-begin-ask-left` | hand-made Home deliveries: repository adopted, "temper can change its code now."; "Tell temper how you work" left, its act staying in Ask temper (the look request `to-begin-two-steps`'s page) |
| `home-accounts-claude` | hand-made Home deliveries: Claude subscription alone; link offers ChatGPT or another plan |
| `home-accounts-both` | hand-made Home deliveries: ChatGPT plan and Claude subscription; link offers another account |
| `home-accounts-key` | hand-made Home deliveries: an API key; link offers another account |
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
| `home-ready` | hand-made Home deliveries: both steps done; plain Home, its rail and tab bar whole |
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
| `home-begin-with-needs` | hand-made Home deliveries: To begin showing, with one question and one chat delivered and neither drawn: no Needs you and no Chats under the card |
| `home-begin-settings-left` | hand-made Home deliveries: a way of working accepted, the repository alone left; its act stays in Settings; its Ask temper chat delivered and not drawn: no Chats under the card |
| `night-home` | `home-begin-ask-left` with the night switch pressed: the night look, the switch a sun (the look request `night-mode`'s page 01) |
| `night-ask` | `ask-first-run` with the night switch pressed: Ask temper as "Start" opens it on the first run's project, template 5 listed, at night (its page 02) |
| `night-chat-split` | `chat-draft-split` at night: a chat beside its draft (its page 03) |
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
| `ask` | the solo world's Ask temper page before its first words: its heading, the form "Ask temper" and its empty text area |
| `ask-first-run` | the same page in the first run's world at its step 7, template 5 listed, To begin unfinished, its way of working left: the rail's Home and Ask temper alone, the phone's three tabs; the lede and the ways, Agents review current and shown with "Keep it", Send disabled while the words are empty |
| `ask-ways-approve` | `ask-first-run` with "You approve" pressed: its line, its path with "you approve" and no review, and "Use this" |
| `ask-ways-draft` | hand-made `ways` delivery over a repository with CI and tiers on goal branches and on main, "Draft first" shown: its two landings, the goal branch's bar dimmer, "verify" before main |
| `ask-ways-card` | `ask-first-run` with the review's stop pressed: its card under the path, its caret at the stop, its model, what it can do and what it is asked to |
| `ask-ways-card-person` | `ask-ways-approve` with "you approve" pressed: a person's card, the role and what they decide |
| `ask-ways-yours` | hand-made `ways` delivery whose project runs no stock way: a fourth tab, "Yours", current and shown, its path, no act |
| `ask-ways-refused` | `ask-ways-approve` with "Use this" refused for too few approvers: the forge, the role, how many it asks and how many can, under the foot |
| `ask-ways-member` | a member's page: the tabs and the paths, no act |
| `ask-ways-kept` | after "Keep it": its toast naming the revision, Agents review current with no act, To begin's first step done |
| `ask-typing` | words entered; Send enabled; the rail and tabs as `ask-first-run`'s |
| `ask-sending` | the Ask temper start out; Send disabled, words kept; the rail and tabs as `ask-first-run`'s |
| `ask-refused` | the start refused; jig's words below the composer, the words kept; the rail and tabs as `ask-first-run`'s |
| `ask-chat` | the solo world’s T10 as an Ask temper chat, conversation and proposal in its layout, its path through Ask temper |
| `ask-chat-proposal` | `ask-chat` with a pending proposal of settings: its title, lines, items and "Accept" |
| `ask-chat-proposal-deployment` | that proposal of the deployment's settings: "How temper will run your projects" and "Accept" |
| `ask-chat-proposal-two-projects` | that proposal of two projects' settings: "How temper will run tally and 2" and "Accept" |
| `ask-chat-accepted` | that proposal accepted: "✓ Accepted · new work follows it" |
| `ask-chat-rejected` | that proposal rejected: "Rejected · nothing changes", its items kept, no "Accept" and no "Change something" |
| `ask-chat-replaced` | that proposal dimmed, "Replaced by the proposal below", the later one under it |
| `ask-chat-withdrawn` | that proposal withdrawn as stale: "Withdrawn · nothing changes", its items kept, no "Accept" and no "Change something" |
| `settings` | hand-made settings deliveries: How work runs with every part of kind 7 shown |
| `settings-all-hidden` | hand-made settings deliveries: every switch of kind 7 off, the mock's `settings.html?project=hello` |
| `settings-goals` | hand-made settings deliveries: Goals shown, its ways of working with one the default |
| `settings-chat-open` | hand-made settings deliveries: an Ask temper chat open; How work runs offers no "Start" and no "Go on", and the rail's "Ask temper" opens that chat |
| `settings-drafts-off` | `settings` with "Chats can write drafts" off |
| `settings-goals-off` | `settings` with Goals hidden: no ways of working, and the rail without "Goals" |
| `settings-changes-off` | `settings` with Changes hidden: the rail without "Changes" |
| `settings-away-off` | `settings` with the Away switch hidden |
| `settings-refused` | a switch pressed and refused, jig's words under its row, the switch back at its value |
| `how` | hand-made `ways` delivery over a repository with CI and tiers, Agents review current and shown: the heading "Agents review", its line, "Ask temper to change this", its path drawn with "verify" before main's bar and the legend under it, no act but "Keep it" while To begin's first step is left; the foot's switches (`README.md`, 5.13, *The page*) |
| `how-approve` | hand-made `ways` delivery over a repository with neither CI nor tiers, You approve current: its path with no "check" and no tier, "you approve" above the line |
| `how-draft` | `how` with Draft first current, Goals shown and its template giving 80: two landings, the goal branch's bar, Roles from a hand-made `roles` answer with the coordinator and the foreman, Budgets' row "Goals", "80" |
| `how-yours` | hand-made `ways` delivery whose project runs no stock way: "A way of its own", the way "its own", a fourth tab, "Yours", current and shown, no act |
| `how-card` | `how` with the review's stop pressed: its ring and words, its card under the drawing with its caret, its model, what it can do and what it is asked to, "Open the reviewer ›" |
| `night-how` | `how` with the night switch pressed: the path's marks by night |
| `repositories` | the Repositories tab with rows |
| `repositories-none` | the Repositories tab's quiet band |
| `forges-in-settings` | `Forges` inside Deployment's frame |
| `accounts-in-settings` | `Accounts` inside Deployment's frame, "Sign out" at its foot at a phone's width alone |
| `forges-empty` | the Forges tab's quiet band, in a world with no forge |
| `forges-setting-up` | the first run's Forges at step 6: a row setting up |
| `forges-fetching` | the first run's Forges at step 24: a row fetching Forgejo, its percent in its summary |
| `forges-fetching-bytes` | the first run's Forges at step 25: a row fetching Forgejo, its bytes in its summary |
| `forges-failed` | the first run's Forges at step 9: a set up failed, with "Set up again" |
| `forges-replacing` | the solo world's Forges, a row's Replace open on its private key |
| `forges-removing` | the first run's Forges at step 7: a row's removal asking first |
| `models` | Settings, Models: offered models by provider, what pays, default efforts, picker switches and the last line's link to Ask temper |
| `models-add` | an owner adding a model by provider and name |
| `models-off` | a model switched off, still in the list |
| `models-added` | a person's model: "added by you" and "estimated prices" |
| `models-member` | a member reads On and Off, with no switch or add form |
| `models-one-account` | Claude alone: OpenAI has its Connect line and no stock rows |
| `accounts-empty` | the Accounts tab's quiet band, in a world with no account |
| `accounts-states` | the reference world's accounts, a row in each state of a source |
| `accounts-key` | the reference world's accounts, a row's key field open |
| `accounts-removing` | the reference world's accounts, an account's removal asking first |
| `accounts-signing-in` | the reference world's accounts, a row's sign-in under way with its field for the page's address |
| `accounts-try-stopped` | a failed trial under an account's head and before Sources, its why and Try again; Sign in again goes to Connect a model |
| `accounts-program-needs-sign-in` | the solo world's accounts: a program's login not signed in on its host, with "Check again" |
| `builders` | the Builders tab, its hosts connected, draining and not connected |
| `builders-not-owner` | the Builders tab for a person with no role in the deployment |
| `builders-none` | the Builders tab with no host connected |
| `ask-chats-tag` | the solo world’s chats with T10 tagged Ask temper beside ordinary chats |
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
| `chat-model-open` | a new chat, Claude alone, its picker open: the other provider's line, the efforts |
| `chat-model-both` | a new chat with Claude and a ChatGPT plan, its picker open |
| `chat-model-all` | a new chat, its picker on "All models", with a model `picker` doesn't show |
| `chat-model-two-accounts` | a new chat with two accounts of one provider, its picker open: a heading for each |
| `chat-model-sign-in-again` | a new chat whose account has stopped being usable, its picker open: the provider's own fix and no model |
| `chat-model-fixed` | a chat whose choice can't be changed: the same words, not a button |
| `chat-started-model-open` | a started chat, its picker open: "A change applies from the next turn." |
| `chat-started-model-changing` | a started chat with a change sent, its picker open: "Changing…" |
| `chat-started-model-refused` | a started chat with a change refused, its picker open: why |
| `ask-model-open` | Ask temper's page, its picker open |
| `chat-draft-chat` | a chat with a document, in Chat: its group and tool line |
| `chat-draft-split` | the same chat in Split: the strip, the pane and a tinted section |
| `chat-draft-draft` | the same chat in Draft: the composer at the foot on white |
| `chat-draft-empty` | "+ Draft" pressed and nothing written: the empty pane |
| `chat-new-draft` | `NewChat { draft: true }`: the empty pane in Split |
| `chat-start-goal` | a chat's sheet to start a goal, open |
| `chat-terminal` | the `chat-draft` chat with ">_" pressed: the terminal, dark, DejaVu Sans Mono (the request's page 02) |
| `chat-terminal-draft` | the same with the draft beside it (page 03) |
| `chat-terminal-commands` | the same with "/" in the box: the commands' list (page 04) |
| `chat-terminal-model` | the same with `/model`'s list (page 05) |
| `chat-terminal-working` | a turn running: the running line above the box (page 06) |
| `chat-terminal-light` | the terminal in its light theme and IBM Plex Mono, its draft beside it |
| `chat-terminal-closed` | a closed chat in the terminal: the box for commands, "/new starts another chat" |
| `chat-terminal-question` | a chat whose goal asked the person: the question's list under the box |
| `chat-terminal-proposal` | an Ask temper chat's proposal in the terminal: its one line and its acts |
| `chat-start-goal-refused` | the same sheet, refused in jig's words |
| `chat-goal-linked` | a chat with its goal started: "Goal TITLE →", and its line |
| `chats-drafts` | the Chats page under Drafts: "writes away-mode.md" |
| `ask-chat-document` | an Ask temper chat in Chat, its document written |
| `ask-chat-document-split` | the same chat in Split, its pane's line and "Start over" |
| `rail-new-menu` | the rail's "+ New draft or change" with its menu open: Draft, Change and Ask temper |
| `rail-person-menu` | the rail's person with its menu open |
| `rail-new-change` | a project offering no drafts, as at Pair: the rail's "+ New change", a link, no menu |
| `rail-all-projects` | a person in two projects, the switcher open: "All projects", "2 projects", first |
| `rail-across` | *Needs you* across projects: the switcher names "All projects", the rail's Work and How work runs the project last open |
| `work` | hand-made deliveries after the whole story's `work.html`: two live goals, one with a change checking and one landed on its card; a change of its own, queued; a chat with its change; two drafts; four changes landed this week |
| `work-pair` | hand-made deliveries after `pair/work.html`: chats and their changes alone, drafts not offered; no Drafts or Goals filter, the rail's "+ New change" |
| `work-drafts` | `work` under Drafts: the drafts' cards alone |
| `work-goals` | `work` under Goals: Work's head and filters, the goals' list under them, its heading at level 2 |
| `work-changes` | `work` under Changes: Work's head and filters, the queue and what landed under them |
| `work-empty` | hand-made deliveries with nothing listed: "Nothing in flight yet.", All alone among the filters |
| `members` | Deployment's Members: the deployment's line, two projects' members with their roles |
| `members-refused` | Members with one project's read refused: its title and "Read by the project’s members." |
| `limits` | Deployment's Limits: the capacities, two waiting attempts, and two projects' tasks |
| `limits-not-owner` | Limits for a person with no role in the deployment: its one line |
| `night-work` | `work` with the night look on |
| `night-limits` | `limits` with the night look on |
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
| `goal-no-report` | first-run-projects running with tasks and no report: "N of M tasks done.", no byline |
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
| `welcome-ready-starting` | a usable subscription's trial at Starts: its first step ticked, Claude Code starts on this machine under way |
| `welcome-ready-answering` | the trial past Starting: its first two steps ticked, Claude answers under way |
| `welcome-ready-answered` | an answered trial: ticked, answered in N s, no Each step |
| `welcome-ready-stopped-starting` | a trial whose run ended before starting: stopped at its second step, the run's why, Try again; Ready and Create stay usable |
| `welcome-ready-stopped-answering` | a trial whose run ended before answering: stopped at its third step, the run's why and Try again |
| `welcome-ready-waiting` | a running trial with a wait that won't settle in a moment: stopped at its step, the wait's words, no Try again until it ends |
| `welcome-ready-waited` | a trial ended after a minute: the last wait's words, or no answer within a minute; Try again |
| `welcome-ready-plan` | a plan's trial: Signed in to your PLAN plan ticked, then PLAN answers under way |
| `welcome-ready-key` | a key's trial: Key added ticked, then PLAN answers under way |
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
| `connect-forge-choose` | the shared Forgejo and GitHub cards, with Your Forgejo below |
| `connect-forge-connected` | shared cards with the connected Forgejo marked and GitHub primary |
| `connect-forge-yours-admin` | Your Forgejo’s admin form, with the other form absent |
| `connect-forge-yours-tokens` | Your Forgejo’s bot tokens, with the admin form absent |
| `adoption` | the adoption form, empty |
| `adoption-filled` | the adoption form, filled in |
| `adoption-refused` | the adoption form after a refusal, its alert |
| `sheet-adopt` | Add repositories with a long list, search, count and disabled project rows |
| `sheet-searching` | the sheet searching with a pick retained |
| `sheet-none` | the completed search with no match and Not listed opened |
| `sheet-few` | a small completed list, no search or count, More open |
| `sheet-empty` | the sheet with no reachable repository |
| `sheet-copy` | Copy public, one address per line |
| `sheet-copy-lines` | invalid copy lines and their repairs |
| `sheet-empty-source` | Make empty, with organization, name and the destination forge |
| `repositories-making` | copies under way and failed, the forge’s reason, Copy again and Dismiss |
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
| `signed-out` | signed out: the top bar's mark alone, and the column "Signed out" with the printed link's words (2.5, *Outside a project*) |
| `out-of-date` | a bundle older than the engine: the column "Out of date" |
| `no-project` | signed in, `me` naming no project, on a deployment with no welcome: the top bar with the person and "Sign out", and the column "No project yet" with the form "New project" (`README.md`, 3.5, *With no project yet*) |
| `approve-typing` | the approval page at `/terminal`, no code: "Sign in a terminal?", the field holding a code typed in lower case with a space, `wdjb mjht`, and Check (`README.md`, 3.5, *The approval page*) |
| `approve-unread` | the same field after Check with a code that doesn't read, `WDJB-MJH`: its alert |
| `approve-asking` | `/terminal?code=WDJB-MJHT` read waiting, asked two minutes before: the code, "Asked 2 min ago", the warning, Approve and Refuse |
| `approve-approved` | the same approved, waiting for the terminal's take: "Approved" and the code |
| `approve-signed-in` | the same taken: "Signed in. You can close this tab." |
| `approve-refused` | the same refused |
| `approve-expired` | the same read expired |
| `approve-used` | the same decided twice: Approve answered used |
| `approve-unknown` | a code the engine doesn't know: "Unknown code" and "Type a code" |
| `approve-signed-out` | the approval page with the browser signed out: "Sign in first", the printed link first, then the page's address again |
| `approve-signed-out-google` | the same with Google's way given: "Sign in, and this page comes back with your terminal's code." and "Continue with Google", its `then` `/terminal?code=WDJB-MJHT` |
| `approve-busy` | a read answered busy: "temper couldn't answer just now" and Try again |
| `chats-empty` | the chats with none, outside the playground |
| `inbox-empty` | nothing needs the person |
| `team` | hand-made deliveries after the whole story's `team/team.html`: two people, the person reading the project's owner with two questions and a review waiting, and Sam, a member, with a question waiting on one of Sam's chats; "Two of you."; a service member left out |
| `team-alone` | one person, nothing waiting: "Just you." and one box, "Nothing waits on you." |
| `night-team` | `team` with the night look on |
| `chat-closing-below` | a chat at rest with one task below it: a singular count and the hairline |
| `chat-closing-proposal` | a chat at rest with one proposal: a singular count and the hairline |
| `task-releasing-uncertain` | a release with an uncertain effect and its two choices |
| `chat-closing-work` | a run, two delegates with opposite fates, and two proposals: what closing changes |
| `chat-closing-failed` | the same close refused: the sheet keeps its choices and says why |
| `chat-closed` | the requester closed it: one end line at 21:05, with a quiet bar |
| `chat-paused` | a chat its person paused: "Paused", Resume and Close, and the composer's hint "Sending resumes this chat." |
| `chat-closing-start-over` | an Ask temper chat's Start over confirmation |
| `task-closing` | an agent's task Close confirmation, with a task below it |
| `task-paused` | an agent's task its person paused: "Paused", Resume and Close, and no own held card |
| `proposal-accepting` | a proposal's Accept confirmation |
| `proposal-rejecting` | a proposal's Reject confirmation with its reason |
| `chat-closing` | `chat-solo-T6` with Close pressed: its confirmation |
| `chat-rejecting` | `chat-solo-T10` with Reject pressed: the reason's box |
| `task-held` | an agent's task's plan with a held task, Release offered |
| `task-releasing` | `task-held` with Release pressed: its confirmation |
| `connect-model-choose` | shared plan cards and key rows, with the owner’s quiet holder switch |
| `connect-model-connected` | a connected plan marked on its card, with the first unconnected plan primary |
| `connect-model-key-details` | a key's secret field and Add key |
| `connect-model-program-details` | a program’s panel, command, Copy, login name and I’ve signed in |
| `connect-model-signing-in` | the waiting note, "Open it again" and folded paste |
| `connect-model-declined` | plan usage declined, with another sign-in and a key way |
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
  cookie goes with the `GET` and is never read. The top bar (2.5,
  *Outside a project*) offers "Sign in" at its right, with or without
  Google's way, and nothing else of a person's: a link to the page's
  origin, `/`, whose page signed out is the sign-in page where Google's
  way is given, and otherwise the column "Signed out" with the printed
  link's words (`README.md`, 5.1, *The pages not about the work*). It
  needs the page's origin alone, as the view's `Ways` give it.

### 8.3 What it shows

- **Titles and the project's name, no other text:** the records carry
  no text but these (jig's `domain/people.md`, 13.3), so a project is
  its name, and a goal is "Goal" and a task "task", each with the first
  eight digits of its public identifier, then its title, with "…" where
  cut; an executor is its kind, "an agent", "a procedure", "a person".
  A title is the first line of a spec an agent wrote from what a person
  said, so a chat's words can show here; ctl's publication help and completed
  answer say so (`tools.md`, 5.3). A preceding acknowledgement is Later
  (`tools.md`, section 14), and no page makes a project public.
- **The look:** one plain column, the same at every width from 320 to
  1440 px, in the client's type and colours (`temper.css`'s tokens), its
  rows modelled on the Chats page's: no table, no monospace, and nothing
  overlapping at any width. A long title wraps inside its row.
- **A row:** a goal's is its identifier and title, then, muted, on one
  line that wraps cleanly, its phase in words, for how long, and its
  spend: "running · for 14 minutes · 126 spent". A task's is the same
  form without spend, indented below its goal: "task 3f2a9c01 Write the
  parser", then "running · for 14 minutes". No row shows a budget or a
  priority; a goal's priority orders the board.
- **An ended row** says how it ended and how long ago: "landed · 2 hours
  ago · 126 spent". A goal reads "landed", "failed", or "ended" for one
  closed with neither, by the same endings as the Goals page; a task
  reads "done", "failed" or "ended".
- **The board:** the project's name as its title and, under it, "What
  temper is doing on this project, as it happens. Read-only." and, in the
  same muted form, "Work that ended before this project went public, or
  before temper last restarted, isn't shown."; each goal's row, with its
  tree's rows below it and the count of lines folded; "No goals to show
  yet." when it has none. Below the goals, only when the forge's landings
  hold a queue or a landing, "Landings", a link to them, with the queues'
  lines, and "No landings to show yet." until one has. The link looks like
  one whatever stands under it, which may be nothing: the mock's `.link`,
  in the ink at a row's size and underlined, not a section title's small
  muted capitals.
- **A task:** its row, its executor first among the muted words ("an
  agent · running · for 14 minutes"), its lineage up to its goal as
  links, its delegates' rows, and where it makes a change, the change's
  step in words, CI's state, its gates' verdicts by number and kind, and
  "Its change: 789ada1", the merge's short commit once it has landed and
  the head's until then.
- **Public lines wrap at desktop and phone widths:** the identity link and
  title form one group; the row's executor, phase, duration and spend form
  a second group with gaps between facts. Titles wrap without truncation
  beyond the public record's cut. Public content is a block reading column
  with at least 16 CSS pixels of horizontal gutter, including navigation.
- **The landings:** the turn, each queue in order, without places, and
  the recent landings, each landed commit a link to its public landing;
  "No landings to show yet." when there are none. A landing's member
  reads "Its change:" and its landed commit, or its candidate until
  then. What a stock queue merged is a recent landing in this same form
  (`forge.md`, 11.2): its line says landed, its change and its commit,
  and its public landing holds its change, with no gates. A landing's
  stale proofs show only when it has any: a zero draws nothing, on a queue
  merge's landing as on a landing by turn.
- **The projects:** each public project's name, a link to its board; "No
  public projects yet." when there are none.
- **A short commit** is the first seven digits, plain text on a change
  or a landing's member. In the recent landings list it links to the
  public landing, as above. A commit is held in 32 bytes, a 40-digit one
  zero-padded, so a page never prints one whole.
- **No link to the forge:** a commit, a pull request and a CI run are
  never links to the forge, for the reason in `forge.md`, 11.3.
- **Words never touch:** where a line holds several, a visible separator
  or space stands between them in the page's text, not only in its
  layout.
- **Public data comes only from the public records:** their text is titles
  and the project's name, never a person's name. The view adds the page's
  own words specified above.

### 8.4 Its tests

- **Step tests:** the address codec's public paths, both ways, and a
  malformed one (a number, upper-case, short or long identifiers) read
  as `/public`; the public mode's reads, its timer from the answer's
  freshness and `Age`, a hidden tab reading nothing, busy keeping the
  page until its freshness ends and then dropping it, backing off,
  unknown's main; a goal's row and a task's, an ended row's words, each
  page's empty words, the board's line of what it doesn't hold, Landings
  on the board only with a queue or a landing, and as a link, whose look
  the sheet's rule holds, a queue merge's line and its landing, a landing's
  zero stale proofs drawing nothing, a 40-digit commit's seven digits as
  plain text, and no link to the forge on any page.
- **Fuzzy tests:** the address codec's parser over seeded paths under
  `/public`, never panicking and printing back what it read.
- **The browser tier** (section 4): a public project on fakes read at
  `/public`, its board, a task and its landings, by a page with no
  sign-in and no stream, the view's text checked against the public
  records' fields alone. Its board and task lines, goals and delegates,
  are also read at 1440 px and 420 px with a long title and populated facts:
  one geometry query reads all text matches at each page and width, with the
  count of any matches beyond the browser's limit checked. Text boxes stay
  separate, titles remain readable, navigation stays inset and content stays
  within the viewport. A missing text box reports its text and measurement stage. A project made private has its pages
  unknown at the next read. The goal it reads is under way: a landed goal's pages
  hold nothing a browser alone shows (4.6). Every page includes the "temper" and
  "Sign in" top bar (8.2), including unknown and empty pages. Its landings page
  then reads "Landings", optionally "Queue" and its one member, then "No landings to show yet."
  or the stock queue's merge. The member reads "task " and eight lowercase hexadecimal
  digits, " · Goal " and eight such digits, then " · Ready " and "just now",
  "a minute ago" or decimal digits and " minutes ago". The merge reads
  "landed · task ", eight lowercase hexadecimal digits, " · " and seven such
  commit digits. These are the whole text joined without separators; nothing may
  precede or follow it. One run reads only one form, so the tier's library holds
  the queued and unqueued forms as a table in the focused suite, and refuses
  any other line. The story's other page and geometry checks stay the same.
