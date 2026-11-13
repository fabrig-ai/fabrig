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
  and the forge's family (`../README.md`, section 3 and `../forge/family.md` and `../view.md`), compiled to
  WebAssembly under a browser shell of temper's,
  `temper-client-shell-web`, on skein's browser parts (`skein-ui-web`;
  skein's `ui/web/README.md`) and jig's client adapter and sign-in part
  (`jig-client-shell-web`; jig's `protocol/browser/README.md`, 5.1), with
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
  person (skein's `ui/README.md`, 14.1): no browser, no WebAssembly and no fake
  DOM library.
- **Served by temper's server, from one origin.** `temper-service` serves
  the page and the bundle on its clients' listener (skein's `ui/web/bundle.md`,
  4), beside jig's clients' component under `/jig` (`../engine.md`
  ), from the bundle the binary holds or a directory its
  configuration names, with jig's example guards and the page's policy,
  which is temper's: a strict content security policy, no frames, no
  CORS (jig's `protocol/browser/README.md`, 12.1 and 12.2). A binary built
  without its browser serves the page for no bundle, and the terminal
  works as before (2.2).
- **The bundle is in the binary.** A plain `cargo build` (or `cargo
  build --release`, or `cargo install --git`) makes a `temper-box` that
  holds its bundle, so that running it is the whole install. The
  machine's builds, temper's gate and its agents, build without it
  (`TEMPER_INTERNAL_BUNDLE=none`), as fast as before (2.6).
- **Signed in by the printed link,** opened in a browser on the engine's
  machine: jig's sign-in part spends it, the credential is the
  mapping's cookie, and a reload signs in again by it (2.4).
- **Browser tests only for what a browser alone shows.** temper's
  stories are the worlds' and the real loop's, through the terminal
  shell (`../../testing/README.md`, 2.1 and `../../testing/real-loop.md` and `../../testing/checks.md`, 3). The browser tier proves
  the bundle under its policy, signing in by the link and the cookie,
  addresses, the composer's keys and jig's attack stories that need a
  browser, and nothing a world can show (`testing.md`).
- **A real browser at the top, driven from Rust,** by `skein-browser`
  (`testing.md`).
- **The gallery** prints the browser's pages in named states as static
  documents, beside the bundle's stylesheets, for screenshots held to
  the mockups: no server, no script, no network (`gallery.md`). It stays
  for the states no journey reaches.
- **Journeys** are walks recorded in a browser on a fresh fake box,
  replayed natively through the shipped host and printed as the
  client's pages, where the look is tuned and kept current by a test
  (`journeys.md`).
- **Look requests** carry the person's changes from the prints into a
  landing, an account and an after-landing render check (`look-requests.md`).

## 2. What temper brings to the browser

- **Its crates as they are.** The client root, the forge's side, the
  view and `temper-client-protocol-wire` are step crates under the
  subset, which build for the host and for wasm32 alike; the terminal
  and the agents' tools are not part of the bundle.
- **Its look:** temper's style tokens and a stylesheet over skein's,
  tuned on the journeys' prints (`journeys.md`, section 7; skein's `ui/web/README.md`, 3.2
  and section 10): 2.5.
- **Its pages,** the client's (`../pages/README.md`), each at an address of
  its own (2.3); and the forge's parts, from `../forge/README.md`, as the forge's
  side reads them.
- **Signing in:** by the link `temper-box serve` prints (2.4). The forge as
  people's sign-in provider is temper's own routes (`../README.md`,
  section 5).
- **The toolchain:** the standard library for `wasm32-unknown-unknown`,
  skein's bundler and the glue it names (skein's `ui/web/bundle.md`),
  and Chromium for the browser tier.

### 2.1 The shell

- **`temper-client-shell-web`** is a library, for
  `wasm32-unknown-unknown` and on every target for its host
  and its journeys (`journeys.md`, section 2), ordinary Rust as a shell is (skein's
  `ui/README.md`, section 2), copied from jig's reference browser shell,
  `jig-client-test-shell-web` (jig's `protocol/browser/README.md`, 3.1), with
  temper's client root, view and records in place of jig's test
  application's, and none of jig's look (`jig-client-shell-web`'s
  `look`, `jig.css`). Its modules:
  - `lib`: the start (the page's internal settings, below; skein's
    restart part; jig's sign-in part taking
    the link from the address before anything is sent; the small store;
    the page with temper's look; the DOM) and the turn of skein's
    `ui/web/README.md`, 4.2; and, as jig's reference shell has it, the test
    address `/test/panic`, at which the shell panics at start, for
    skein's `panic` story (`testing.md`, 1);
  - `host`: the client host: temper's client root and view, and the
    view's patch queue, reserved at start at the view's limit on
    patches and drained each turn (skein's `ui/web/README.md`, 4.2); the root's
    `Leave`, a forge's or an authorization's departure, as skein's
    transport's navigation away, in the turn that asked it (skein's
    `ui/web/README.md`, section 8); the root's `Tab`, a sign-in's tab
    (`../pages/README.md`, `ConnectModel`, The sign-in's tab), as skein's
    platform's, the opening before the turn's transport, and a refusal
    given back as a report of that turn; jig's
    mapping under `/jig` and its stream; the root's requests to jig's
    adapter (`jig-client-shell-web`'s `adapter`) and skein's transport,
    and their reports back to the root; and the wake, skein's timer
    set at the earlier of the root's deadline and the instant the
    view's words next change (`../view.md`, 6); its seam, a report
    given and the duties asked back, a turn's drawing and its deadline,
    the shell's turn and the journeys' replay call alike
    (`journeys.md`, section 2);
  - `journey`: a recording's records and file, its recorder and its
    replay (`journeys.md`, sections 3 and 4), on every target; the
    recorder's keys and save are in `lib`, installed only when the page
    records, by its `internal-record` or its `temper-record` meta (below);
  - `records`: the small store's records, `jig-client`'s and a sign-in's
    address (`../README.md`, section 4); from the
    adoption form, a pending adoption is among them, a kept
    request whose request `temper-client-protocol-wire` encodes
    (`../forge/records.md`);
  - `limits`: the client's limits in a browser, derived as the
    terminal's are (`../README.md`, section 2), and their sum; and
    `RECORDING_BYTES`, 67,108,864, a recording's bound in memory
    (`journeys.md`, 3.1): two of `BODY_BYTES`, so that a recording
    holds a whole body's answer beside what came before it.
- **A page is an entry crate on the shell:** the library holds no wasm
  start, since a module has one and a page's crate links the library.
  `temper-client-web-page`, a `cdylib`, is the shipped page: its start
  calls the shell's, and the bundle is built from it (2.6).
  `temper-playground-page` is the playground's (`../../playground.md`,
  3 and 6): skein's worker platform where the shipped page holds the
  browser's (skein's `ui/web/README.md`, 15), driving the shell's loop
  through its public seam (the shell, its platform's type and its
  report), the starter, the frame and the failure page with the
  session's log. The shipped bundle holds none of the playground's.
- **The page's internal settings** (`../../draft/cli.md`, section 6) are
  query parameters of the address the page loads at, read once at its
  start, before anything else reads the address, and taken out of it
  then, with the history's replace the sign-in part uses, so that no
  route, recording or link sees them. Each is on when its value is `1`,
  and off when absent or anything else:
  - `internal-record`: the browser's recorder runs (`journeys.md`, 3.1),
    on any origin, `http` or `https`;
  - `internal-small-window`: the client's limits name a conversation's
    window of 32,768 bytes, with `long_text` and `turn_text` 32,768
    (`testing.md`, 3, The tier's window), where they name none and
    131,072;
  - `internal-loopback-navigation`: skein's platform follows a
    navigation to plaintext loopback (skein's `ui/web/README.md`, section 8).

  `internal-small-window` and `internal-loopback-navigation` are
  honoured only on a loopback origin, the page's host `127.0.0.1` or
  `[::1]`, as the browser tier and the journeys tool serve it, and
  ignored elsewhere: loopback navigation elsewhere would let a page
  send a person's browser to plaintext services on their own machine,
  and the small window is the browser tier's. `internal-record` is
  honoured everywhere: the recorder adds no node, no style and no
  request, and a recording leaves the browser only by the person's own
  Alt+Shift+S, or the save before a departure, into their own downloads
  (`journeys.md`, 3.1), so a link that sets it on someone else's page
  takes nothing from them. Each of the three is taken out of the
  address wherever it is given, honoured or not. A reload is a load
  without them. The shell's limits are given at its host's start, which
  the replay gives the ordinary ones.
- **The page's `temper-record` meta:** a page made with `<meta
  name="temper-record" content="on">` (2.2) records as if its address
  had `internal-record=1`, on any origin, at every load: a reload, or a
  return from a departure, starts a new recording (`journeys.md`, 3.1).
  The shell reads it at start, beside `temper-sign-in`; a meta of any
  other content, or none, turns nothing on. The address's setting and
  the meta are read once each and joined: either turns the recorder on,
  and neither turns it off.
- **It decides nothing.** What a page shows and does is the domain's
  and the view's, step code under the subset; the shell orders the
  platform's work (jig's `protocol/browser/README.md`, 3.1).
- **The failure page:** past the restart part's limit, the page says
  "The application failed.", in temper's words, since it takes none of
  jig's look; it starts nothing and keeps the small store (jig's
  `protocol/browser/README.md`, 5.1).
- **The small store** is the browser's session storage, under the
  prefix `temper.`, holding `jig-client`'s records alone, as the
  terminal's file does (`../README.md`, section 4); from the adoption form
  a pending adoption is among them, a kept request
  (`../forge/records.md`). Each load of the page is a client of its own, with
  its own stream (jig's `protocol/browser/README.md`, section 10). The root can
  step the client several times in one turn: each write the client asks
  for (`Save`, `Erase`) is answered once, after the turn's whole storage
  has finished, and every one is refused if any write was refused.
- **What the terminal says in lines of its own** (`../README.md`, 3.4, and `../line.md`, 6
  ): another party's kept work and kept work dropped have no place
  in the browser, whose small store lives as long as its tab, of one
  person (`README.md`, section 3). A service's credential is never shown: temper's
  client makes no service (`../README.md`, section 5).

The person's zone is one of the shell's inputs: `Date`'s offset, in minutes
east of universal time, read at start and recorded in a journey's facts, so a
replay says the same clocks (`../view.md`, 6).

### 2.2 Serving

- **The routes of the clients' listener,** temper's, in this order,
  over jig's mapping (jig's `protocol/browser/README.md`, section 12) and
  skein's serving of a bundle (skein's `ui/web/bundle.md`, 4):
  1. what jig's default mapping claims, under `/jig`, to jig's clients'
     component: every request there but a `GET` of the printed link's
     route, which the mapping leaves to the page, answered with the
     page's policy and `Referrer-Policy: no-referrer` (jig's
     `protocol/browser/README.md`, 12.1 and 12.2);
  2. with no bundle, the configuration naming none (`null`) and the
     binary built without its own (2.6), a `GET` of any other path but
     under `/bundle/` and `/robots.txt`, the printed link's route among
     them, answers `200` temper's page for no bundle, every other
     request not found, as `../engine.md` says. The page is
     fixed HTML in the binary, under the page's policy, with no script,
     style or picture: the level-1 heading "temper", then "This temper
     was built without its browser client.", "Open the link in a
     terminal with temper-box open DIRECTORY LINK." and "To serve the
     browser, build temper with the wasm32-unknown-unknown target, or
     name a bundle's directory in application.clients.bundle, and
     restart temper." (`../../protocol/setup/init.md`, 2);
  3. a request other than `GET`: `405`;
  4. `/bundle/<digest>/…`: the bundle's files, immutable, each in the
     form the request takes (skein's `ui/web/bundle.md`, 4); any other path
     under `/bundle/`: not found;
  5. `/robots.txt`, the bundle's one root file (skein's `ui/web/README.md`,
     3.3, step 5; jig's `protocol/browser/README.md`, section 12), disallowing
     every path;
  6. `/sw.js` and `/manifest.webmanifest`, temper's two root files,
     fixed in its binary, `no-cache` (2.7): the service worker,
     `Content-Type: text/javascript`, and the web app's manifest,
     `application/manifest+json`;
  7. any other `GET`, the printed link's route among them: the page,
     with its ETag and `no-cache`, and `304` on a matching
     `If-None-Match` (skein's `ui/web/bundle.md`, 4).
- **The page's metas,** made into the bundle's page at start, before
  its `</head>` and before its ETag is taken: `temper-sign-in`
  (`../../protocol/sign-in.md`, 7); then, when the command that serves
  was given `--record` (`../../protocol/binaries.md`), `<meta
  name="temper-record" content="on">`, so that every page the
  deployment serves records from its load (2.1), in each person's own
  browser, signed in or not; and the forges' `temper-forge-sign-in`
  metas, made once their records load
  (`../../protocol/openid-provider.md`, 7). Without `--record` the page
  has no `temper-record` meta. Nothing of a recording reaches the
  deployment.
- **The page's policy,** on every answer temper gives: jig's reference
  policy as it stands (jig's `protocol/browser/README.md`, 12.1): `default-src 'none';
  script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src
  'self'; font-src 'self'; connect-src 'self'; base-uri 'none';
  form-action 'none'; frame-ancestors 'none'`, and temper's two
  sources beside it, `worker-src 'self'; manifest-src 'self'`, for its
  root files (2.7), with `nosniff`,
  `Cross-Origin-Opener-Policy: same-origin`,
  `Cross-Origin-Resource-Policy: same-origin` and `Referrer-Policy:
  no-referrer`. skein-http's own refusals, `400`, `413` and `431`, given
  before temper sees the request, carry none: they hold no page to frame
  or sniff. temper adds no origin: no font, picture or script from
  elsewhere. The mockups' optional web font is not used; the look takes
  the system's monospaced font.
- **jig's example guards, as they are** (`../engine.md`): the
  Host allow-list holding the listener's own address; a cookie without
  jig's browser header refused.
- **Ordinary calls wait for jig's mapping's room** (jig's
  `protocol/client/default-http.md`, 4). Each waiting head stays under the listener's
  connection bound and stall deadline, with no body read until admitted;
  a closed connection drops its head. The small profile's six calls,
  four ordinary exchanges beside its two streams, therefore serve one
  page's concurrent requests in turn. Public reads retain their separate busy response (jig's
  `protocol/client/default-http.md`, 5); stopping refuses waiting ordinary calls.
- **The bundle** is skein's bundler's output (skein's `ui/web/bundle.md`
  ): the shell's module, skein's stylesheet and temper's
  (`look.md`). temper's build embeds it in the binary (2.6);
  `scripts/bundle.sh` builds one apart, from the module built for
  wasm32, into a directory it prints. `application.clients.bundle` is
  `null` for the binary's own, or names a directory, whose bundle is
  served in place of the binary's (`../../protocol/configuration/engine.md`
  ). `temper-box serve` makes it, at start, whole, into memory,
  within the limits `clients.bundle_files` and `clients.bundle_bytes`
  (`../../protocol/configuration/limits.md`), and refuses the start
  past them, or when it is no bundle (its page and its digest's
  directory), by one code for both: a directory's files read, or the
  binary's, already in memory, copied (2.6). **Both forms stay whole in
  memory,** a file and its `NAME.gz`: the service holds no inflater, and
  a client not taking gzip is rare but is answered without one, a
  `curl` among them. A `NAME.gz` with no `NAME` beside it refuses the
  start, as no bundle does; a directory without its forms (built with
  `--gzip off`) serves `NAME` alone. A new bundle is served
  after a restart; a tab on the old one is answered not found for its
  files, and a reload loads the new (skein's `ui/web/bundle.md`, 4). A page
  from the binary's own bundle always speaks its engine's records, so a
  client out of date (`../view.md`, 1) comes only from a binary
  replaced under an open tab, or from a directory configured, and its
  page asks the person to reload, which loads the engine's bundle.

### 2.3 Addresses

- **Each page's address is a path,** by temper's address codec, a
  module of `temper-client-domain`, step code, total and bounded:

  | Page | Path |
  |---|---|
  | `Inbox { filter }` | `/`, and its filter's query |
  | `Home { project }` | `/projects/<project>` |
  | `AllHome` | `/home` (`../pages/README.md`, *Home on All projects*) |
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
  | `Team { project, role }` | `/projects/<project>/team`, and `?role=N` for `Some(N)` |
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
  | `Authorize { query }` | `/oauth/authorize`, and the query as the address carried it, within jig's bound on an authorization's query (`../../protocol/openid-provider.md`, 3) |
  | `Approve { code }` | `/terminal`, and `?code=` the code as shown, `WDJB-MJHT` (`../pages/README.md`, *The approval page*) |

  `<task>` and `<project>` are the task's and the project's numbers in
  decimal. A later page's path comes with it, in this shape
  (`/projects/<project>/board`). The
  public pages' paths, under `/public`, are `public.md`'s.
- **The inbox's filter is its query:** the fields it sets, in one
  order, `?project=<project>&kind=<kind>&goal=<goal>`, and none with
  every field unset, so that `/` is the inbox unfiltered and
  `/?kind=question&goal=9` one filtered by kind and goal. `<project>`
  and `<goal>` are numbers in decimal, as a path's; `<kind>` is one of
  `proposal`, `question`, `held` and `person-task`: the kind's button
  word (`../pages/README.md`'s inbox), in lower case, its space a hyphen.
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
  filters are their pages' addresses, Goals' and Changes' (`../pages/README.md`,
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
  and `?at=ready` for the others (`../pages/README.md`, The welcome). As
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
  number in decimal (`../forge/deployment.md`). A value that doesn't read as
  one, or a field of another name, is dropped, and the address
  replaced with the printed one. It reads a number, not the
  deployment's forges: a number that names no forge shows Choose.
- **`ConnectModel`'s way is its query:** none at Choose, so that
  `/accounts/connect` is the flow's first step; `?provider=N&way=W`,
  N an LLM provider's number in decimal and W `sign-in`, `program` or
  `key`, in that order, for the way of that kind its provider offers
  (`../pages/README.md`, `ConnectModel`). The two go together: one without
  the other, a value that doesn't read as its field's, or a field of
  another name, is dropped, and the address replaced with the printed
  one. It reads a number and a kind, not the configured providers: a
  way they do not offer shows Choose. A sign-in's way may add its
  sign-in's source, `&source=S`, S a source's number in decimal, once the
  sign-in is answered, its history entry replaced (`../pages/README.md`,
  `ConnectModel`, A reload while it waits):
  `/accounts/connect?provider=1&way=sign-in&source=7`. A source on
  another way, or one that doesn't read as a number, is dropped as
  above.
- **The approval page's code is its query:** `/terminal` is the page
  asking for a code, and `?code=WDJB-MJHT` carries one, the code as
  shown, four letters, a hyphen and four (`../pages/README.md`, *The
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
  address and this path alone (`../README.md`, 4.1). The place is no
  page's address, as `Authorize` is none: the pages' links never lead
  to it, and the person reaches it by the address alone. The code is
  no secret (jig's `protocol/client/sign-in.md`, *Its risk*), and
  the server's `Referrer-Policy: no-referrer` (2.2) keeps it from any
  other site the page might link to. temper's server answers `GET
  /terminal` with the page, as it answers every path that is no route
  of its own (2.2).
- **A filter chosen is a move** (below), with its history entry: a
  link's, at the page's top with the focus on its heading, and back
  returns to the filter before. Like any move it folds the open
  disclosures (`look.md`). Needs you has no filter menus; its filters remain
  address fields (`../pages/README.md`). A move
  that stays on the chats list, between `Chats` and `NewChat`, or on one
  goal's page, changing its filter alone, keeps the page: its composers' words, the
  closed chats and ended delegates it read, which it reads not again;
  the inbox's filter, a query the engine answers, reads its page anew.
- **A path that is no page** opens the inbox, and the address is
  replaced with `/`. The codec never prints a path under `/jig`,
  `/bundle/`, `/test/` or `/forges/N/github/`, temper's routes for
  GitHub's steps (`../forge/served.md`, 1), nor `/robots.txt`.
- **Moving** is skein's (skein's `ui/web/README.md`, section 7): a link pressed
  is a move within the client, with a new history entry; back and
  forward are moves too, each entry keeping its scroll; a reload, or an
  address typed or opened, is a new client at that address (jig's
  `protocol/browser/README.md`, section 10). That a client opens at the inbox
  (`../README.md`, 3.4) is the terminal's: in a browser the address is the
  browser's, and the client keeps no record of its own.
- **A heading drawn after its page** (the answer of 2026-11-06 to the
  `ui-composer-focus` task `composer-focus`): a new entry whose level-1 heading
  isn't drawn yet, as a chat's isn't until its task is read, has the focus on the
  main (skein's `ui/web/README.md`, 5). When the heading is drawn, the shell moves the focus from the
  main to it, unless the focus has moved meanwhile: a focus placed since keeps its
  place, even back on the main, and a focus request ends the wait. While
  `me` is read the main is empty and marked busy (`../view.md`, 1), so a
  new entry then has the focus on it, and the page's heading drawn once
  `me` is read takes it alike, the page's main being that node (skein's
  `ui/README.md`, 7). A new entry with no main yet waits for its
  main alike: when the main is drawn, its heading takes the focus, or the
  main until the heading comes, unless the focus has moved or a focus
  request came meanwhile.

### 2.4 Signing in

- **The printed link,**
  `http://HOST:PORT/jig/v1/sign-in/link?secret=SECRET`, opened in a
  browser on the engine's machine (`../README.md`, section 4): temper's
  server answers its `GET` with the page (2.2); jig's sign-in part takes
  the secret and replaces the address with `/welcome`, temper's home for
  a sign-in, before anything is sent; the client signs in with it
  (`SignIn { with: Link { secret, held: None } }`), and the mapping's
  answer sets its cookie, which the client never sees (jig's
  `protocol/browser/README.md`, section 11). The browser sends its cookie with
  the link's sign-in, as with any exchange: with a live link, a person
  still signed in keeps their sign-in, its number and expiry, and the answer sets no
  cookie, so the browser's stays as it was (jig's `domain/people/README.md`,
  section 3); a cookie the engine does not know, or one ended or
  expired, signs in afresh, with a new cookie. The welcome follows,
  which moves on to the chats page once the first run is over, and to
  the inbox, drawing nothing of itself, on a deployment with no welcome
  (`../pages/README.md`, The welcome).
- **A reload, or another tab,** signs in with `Transport`, by the cookie
  (jig's `client/domain/client/README.md`, section 9).
- **Signed out,** by "Sign out" (`../line.md`, 6; jig's sign-out clears
  the cookie) or with no link: the main is the view's
  signed-out column (`../view.md`, 1), its heading "Signed out" and,
  in the browser's words, "To sign in, open a sign-in link temper
  printed." The view takes its shell's words at construction; the
  terminal's heading stands alone, with its own line (`../line.md`, 6).
  With a way of signing in configured, the main is the sign-in page
  instead: "Sign in" and "Continue with Google", the ways
  read from the page's `temper-sign-in` meta at start
  (`../../protocol/sign-in.md`, section 7); the link leaves for Google's
  sign-in and comes back to the page's address. Where the deployment
  has public projects, the signed-out main and the sign-in page add a
  link, "Read the public projects", to `/public` (`public.md`).
- **A link's sign-in not taken,** answered full, busy or not ready: the
  link is not spent, and the main says why and that the link stays
  usable until it expires (`../view.md`, 1), full with the deployment's
  room, its earliest end and the link's own. The retry is the person's: opening the
  printed link again sends its secret again, as the first open did, and
  signs in once there is room; opened past its expiry, it answers why
  (expired while remembered), and temper prints a fresh one if no link is
  live or reserved. The page never retries by itself, and keeps no secret:
  the address already says `/welcome`.
- **A used, expired or unknown link** draws the reason's paragraph in
  place of the sign-in page (`../view.md`, 1), with no automatic retry.
  A link forgotten by the engine, after a restart included, is unknown.
  A cookie still live does not override that paragraph; a reload uses
  the cookie instead (`README.md`, section 3).
- **A terminal's approval** (`../pages/README.md`, *The approval page*):
  the page at `/terminal` signs in with `Transport`, by the cookie, as a
  reload does. jig's cookie is `SameSite=Strict` under jig's prefix, so
  the tab's navigation, started by the terminal, carries none; the
  page's own exchanges, same-origin and started by temper's page,
  carry it, whatever opened the tab, as the `Authorize` place's do
  (`../../protocol/openid-provider.md`, 3). Its read and its decision go under
  jig's browser guards (jig's `protocol/browser/README.md`, 12.2), and the
  mapping tells the engine they carried the cookie, so the engine
  takes them as a browser's (jig's `protocol/client/default-http.md`, 3); `temper
  ctl`'s bearer is refused (`../ctl/README.md`, section 7). Signed out,
  the page says "Sign in first". With Google's way given, as the
  sign-in page reads it, it offers "Continue with Google", whose `then`
  is the page's address with its code, so the sign-in comes back to the
  approval (the look request `outside-pages`). Without it, the printed
  link is the way in: the page says to open it first, then the page's
  address again, since the link's sign-in lands on `/welcome`, not back
  on the page (`../pages/README.md`, *The approval page*).
- **One person, on loopback,** as `../README.md`, section 4 says. A browser
  on another machine is open (`../README.md`, section 5).

### 2.6 The bundle in the binary

The `temper-box` binary holds its bundle, so that `temper box` and `temper-box
serve` serve the browser with nothing beside the binary, and `cargo
build` in a checkout makes that binary. The engine's binary,
`temper-engine`, holds the same bundle through the same crate, the only
one of a deployment's binaries that does (`../../protocol/binaries.md`,
2). It is a person's convenience:
the machine's builds keep building the way that serves their work,
without it.

- **One file.** The binary holds the bundle: the shell's module, its
  glue, and the stylesheets and files skein's bundler puts beside them
  (`README.md`, 2.2, `look.md`), and their compressed forms (skein's `ui/web/bundle.md`, 2).
  Running it is the whole install: no directory beside it,
  no `scripts/bundle.sh` step.
- **A plain `cargo build` makes it.** `cargo build`, then
  `./target/debug/temper box`, serves the browser; so do `cargo build
  --release` and `cargo install --git` of temper's repository.
- **A crate of its own embeds it,** `temper-client-bundle`, ordinary
  Rust on the `binary` lint profile (`tools/lints/design.md`, 2.3), which
  `temper-box-shell` depends on. Its library is data: the bundle's files, each its path
  from the bundle's root (`index.html`, then `<digest>/…`) and its
  bytes by `include_bytes!`, in a directory read's order, and the
  digest, both written by its build script into `OUT_DIR`. A build
  without the embed has no files and no digest. The include sits in a
  small crate, not in `temper-box-shell`'s, because a build script that runs
  makes its package build again, and `temper-box-shell`'s is the slowest to
  build. `main.rs` and the integration tests live in the binary's
  package, `bin/temper-box` (`../../cli/README.md`, 2).
- **Its build script,** unless `TEMPER_INTERNAL_BUNDLE` is `none`, takes
  `scripts/bundle.sh`'s steps with the cargo running it (`CARGO`), from
  the workspace's root, into the nested build's directory (below):
  1. `build --locked --target wasm32-unknown-unknown -p
     temper-client-web-page`, the shipped page's entry (2.1), with the
     module's profile (below);
  2. `build --locked --release -p skein-ui-bundle`: the bundler,
     always optimized, as skein's `ui/web/bundle.md`, 1 asks of a tool;
  3. the bundler, with `scripts/bundle.sh`'s inputs (the name, the
     title, the prefix `/bundle/`, `temper.css`, `task-pages.css`, the
     placeholder's `loading.html` and `loading.css`, the shell's `files/`
     and the noscript words), into its `OUT_DIR`;
  4. the list of the bundle's files, written into `OUT_DIR` for the
     library's include.
  It has no build dependencies: the bundler is built by the nested
  cargo and run, never compiled into the script, so a build without the
  embed compiles a script of `std` alone, and nothing of the bundler is
  ever linked into `temper-box` (skein's `ui/README.md`: "never linked into a
  service or a module"). One `Cargo.lock` still locks the bundler's
  `wasm-bindgen-cli-support` with the module's macros (skein's
  `ui/web/bundle.md`, 1). The script's logic (the dep-info's read, the
  nested commands, the bundle's directory read into its list) is plain
  functions, which the crate's tests take on samples.
- **The nested build's directory** is `<target>/bundle/`, `<target>`
  the outer build's, found as the nearest ancestor of `OUT_DIR` holding
  cargo's `CACHEDIR.TAG`. The outer cargo holds its own directory's
  lock while build scripts run, so a nested build there would wait on
  it forever. It is one directory, not one under each `OUT_DIR`, so
  that a check, a clippy run, a build and a test build of `temper-box`
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
- **Its own selection:** both nested cargo commands run with
  `CARGO_RESOLVER_FEATURE_UNIFICATION=selected`, against the workspace's
  unification of every member (`skein/docs/foundation/cli.md`, 3). The
  module's features come from the packages it builds alone, so a union
  resolved for `wasm32-unknown-unknown` over members that never build
  there doesn't reach it.
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
  and `TEMPER_INTERNAL_BUNDLE`, by `cargo:rerun-if-env-changed`. The first run
  has no dep-info yet, and runs regardless. An edit elsewhere never
  runs the script, and a build that does not include `temper-box` (a
  domain's tests, the client crates' own) never runs it.
- **The module's profile:** a dev build embeds a dev module, as
  `scripts/bundle.sh` makes; a release build embeds one built with
  `web`, the profile skein's `ui/web/bundle.md`, 1 names for an
  application's release module, which temper's workspace defines as
  skein's does. The script maps cargo's `PROFILE`, `debug` or
  `release`, to `dev` and `web`.
- **`TEMPER_INTERNAL_BUNDLE`:** unset or `embed`, the embed; `none`, no nested
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
  (TEMPER_INTERNAL_BUNDLE=none)`. Never a binary without its browser in
  silence. The repository's `rust-toolchain.toml` pins the channel and
  names the target (`skein/docs/foundation/programming-model.md`,
  10.1), so rustup
  installs it with the toolchain; the message is for a toolchain
  installed otherwise, or rustup told not to install.
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
  scripts/bundle.sh DIR, then start temper-box --fake-backends
  --bundle DIR even with an empty configuration (../../protocol/configuration/directories.md,
  1). An explicit --bundle overrides source application.clients.bundle;
  otherwise that configured directory is used.
  The embedded bundle is not used for that box. Missing provision is a startup refusal explaining the bundle
  command and --bundle. Startup never builds or downloads the bundle.
  The second-embed prototype added 8,974,112 bytes in the measured dev build,
  exceeding its 5 MB limit; the provisioned path keeps that cost outside
  ordinary builds. TEMPER_INTERNAL_BUNDLE=none runs no nested browser build.
- **`scripts/bundle.sh` stays,** for a bundle built apart: the browser
  tier's and the journeys tool's, whose pages load with their internal
  settings (2.1; `testing.md`, 3), and a developer's. It builds the
  shipped page as the binary's build does, with no switch of its own
  but `--release` and `--playground`. `scripts/install.sh PREFIX`
  installs the binary alone, built with `--release --locked`, and
  prints its path.
- **Tested in both builds.** Every suite passes with and without the
  embed, and the gate runs them under `none`. The build script's logic
  is tested on samples in its crate; the in-memory read on fixture
  files beside the directory's read, the same `Bundle` and the same
  refusals; and a start with `bundle` `null` expects what its build
  holds, which `temper-box-shell` says (`temper_box_shell::bundle::OWN`, the
  binary's digest or none): the page for no bundle under `none`,
  otherwise the bundle's page at `/` and its files under
  `/bundle/<digest>/` (`../../testing/process-tier.md`). The page for no
  bundle stays proved where it is built, in `temper-service`. The embed
  itself is a person's build, which fails loudly when its steps do; a
  change to it checks it with `TEMPER_INTERNAL_BUNDLE=embed cargo nextest
  run -p temper-box`, which the gate does not run.

### 2.7 Being pushed

What the browser does for Telling you (`../pages/told.md`;
`../../domain/forge/attention.md`, section 7). Push is the browser's,
through its push service (Web Push); the page and the module stay as
they are, and two small root files carry the rest.

- **The service worker,** `/sw.js` (2.2, step 6): a fixed script of
  temper's, hand-written, its scope the origin's root. It does two
  things and nothing else, with no fetch handler and no cache, so the
  page loads as it does without it:
  - *a push* shows a notification: its payload's JSON (`../../domain/forge/attention.md`,
    7.2), its `title` the notification's title and its `body` its body,
    its tag `project-N` so that a newer count replaces an older one, its
    data the project; a payload it can't read shows "temper" and
    "Something needs you.", since a browser shows one for every push;
  - *a press on a notification* closes it and opens Needs you filtered
    by its project, `Inbox { project }`'s path (`../view.md`, 3): an open
    window of the origin, focused and sent there, or a new one.
- **The manifest,** `/manifest.webmanifest`: `name` and `short_name`
  "temper", `start_url` `/`, `display` `standalone`, and the paper's
  colour of the day look as `background_color` and `theme_color`
  (`look.md`), so that a phone may add temper to its Home Screen, which
  iOS asks before it pushes to a web app. The page links it, `<link
  rel="manifest">`, in its head: from skein's bundler's page if it takes
  one, otherwise added by the shell at its start.
- **The shell's part,** in `temper-client-shell-web`, through web-sys's
  service worker, push and notification bindings: it reports this
  browser's state to the domain at start and after each act (whether a
  service worker and a push manager exist, the permission, the current
  subscription's endpoint); and on "Turn on" it asks the permission,
  registers `/sw.js`, subscribes with `userVisibleOnly` and the
  deployment's key, and gives the domain the subscription's endpoint and
  keys, which the domain sends as `AddDevice`; on "Turn off", once
  `RemoveDevice` is answered, it drops the subscription. A browser asks
  permission only within a person's gesture: the shell asks it inside
  the press's own event, before anything waits.
- **Tested** below the browser where it can be: the page and its acts
  in the domain's and the view's tests, this browser's states given as
  the shell would report them; the two root files' routes and headers
  in `temper-service`'s; the shell's push calls and the service worker
  in a browser-tier story, written and compiled, and run after landing
  (`testing.md`).

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

- **The terminal approval page after landing** (`README.md`, 2.3 to 2.4 and `look.md` and `gallery.md`, 2):
  check `/terminal` in a signed-in browser on a fresh box, its typed-code
  entry and the twelve `approve-*` gallery states at narrow and wide
  widths. Domain, view and shell tables and gallery prints establish no
  rendered browser result. The TUI's own flow and walk follow
  `../../draft/tui.execution.md`, M2.
- **Terminal approval at a public name** (2.4): Google's browser sign-in,
  the code typed rather than carried in the address, the request's origin
  and age, and a person's list of sign-ins with each ended from there
  (`../../draft/tui.md`, section 7). The loopback page signs in by the
  printed link, or by Google's way where it is given.

- **The outside pages' comparison** (`README.md`, 2.4, `look.md`, `gallery.md`, 2; `../pages/README.md` and `../view.md`, 1
  ; `../forge/served.md`, 1; `../../protocol/sign-in.md`, 4.3): temper/ui
  compares the look request `outside-pages`'s pages 01 to 10 under their
  overlay with the reprint after landing, at 1440, 1000 and 420 px
  (`look-requests.md`, section 4), with the states its message
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

- A failed link with a live cookie: `../README.md`, section 5.8.

- **The fresh project's draft flow** (`../pages/README.md`, `../view.md`, 3, `../pages/settings.md`, 2):
  the real-box check of an Ask temper chat's draft offer and the draft-to-goal
  flow, including Goals in the phone tab bar, is after-landing work
  (`../README.md`, section 5). Default: the view tables and fake-model
  `chat-draft` journey, with no browser or render check during the plan.

- **Kept work in the browser** (`../README.md`, 3.4, and `../line.md`, 6): another
  party's kept work, and kept work dropped at start, which the terminal
  says in lines of its own. Default: not shown; the small store lives
  as long as its tab, of one person.
- **A bundle built apart, embedded:** `TEMPER_INTERNAL_BUNDLE=DIR`, a bundle's
  directory embedded in place of building one, so that one wasm32
  build serves binaries for several hosts and a machine without the
  target can still make a whole binary. Default: none; it waits for
  the release pipeline (2.6).
- **Served from the binary's bytes,** with no copy at start: a `Bundle`
  that borrows. Default: copied (2.6), until a bundle whose copy
  matters.
- **A box still starting:** a link opened before serve is ready gets no
  bytes until it is, so no placeholder either, and that wait stays the
  browser's own (`../../protocol/setup/box.md`, 1). Default: as it is.
- **The embed's steps, skein's for every application:** the nested
  build, its directory, its environment and what reruns it are nothing
  of temper's. Default: temper's crate, until a second application
  embeds its bundle (jig's reference application, or skein's example),
  when they move to a small crate of skein's that a build script
  depends on (skein's `ui/web/README.md`, section 12).
- **A binary to download:** where release binaries are built, for
  which targets and C library (a static musl build, or a floor of
  glibc), and where they are published. Default: none; temper's
  binary is whole (2.6), not shipped.
- **A switch to serve no browser** from a binary that has one, such as
  `clients.bundle` `false`. Default: none until a deployment asks: the
  browser is reached only by a printed link, as the terminal is.
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
- **The rail's foot on a phone** (`look.md`; `../view.md`, 3): the top bar holds the
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
- **A heading drawn after its page, in skein's reference** (2.3): the kit's rule
  stops at the main (skein's `ui/web/README.md`, 5), and any application whose page reads
  before it draws its heading meets what this shell repairs. Default: this shell's
  alone, until a second application writes the same.
- **Reading a conversation in every browser** (`../pages/chat/conversation.md`, 2; `../../draft/chat-reading.md`, 5):
  the position staying put when lines load above is for the user to read on a
  real chat. Default: the browser tier runs Chromium, including with
  `overflow-anchor: none`; Firefox and Safari themselves are read, not run.
