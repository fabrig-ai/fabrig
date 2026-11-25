# temper's client

Provisional, 2026-10-14, revised 2026-11-07. temper's client: temper's own
presentation on jig's client, `jig-client`. Its crates; its client root,
which holds `jig-client` and temper's pages, the LLM accounts' and the
forges' among them; the forge's client family, its words, its one form
and its requests; its view, on
skein's UI kit; the terminal that `temper-box open` runs, on skein's text
shell; signing in and the small store; what temper's server does for it;
and how it is tested. jig's client is jig's `client/domain/client/README.md`, its
API jig's `client/domain/contract/README.md` and its protocol jig's
`protocol/client/README.md`: this document takes them as given and cites them.
The forge's parts of the client are `forge/README.md`, the browser `web/README.md`,
and the agents' tools, `temper ctl`, `ctl/README.md`. temper is the source
of truth for what it draws; the mockups (`web/mockups/`) design what it
does not draw yet: their look, their parts and their order, their
words and where each act leads. This document holds what each page
reads, watches and does, and cites them for the rest
(`web/mockups/README.md`, section 1). What is still open is listed in
section 5.

- **Client:** `root.md`, the client root; `sign-in.md`, signing in and the small store.
- **View:** `view.md`, the view; `view-navigation.md`, its navigation.
- **Browser:** `web/README.md`, the browser's documents.
- **TUI:** `tui/README.md`, the terminal's documents.
- **Agents:** `ctl/README.md`, the command-line client's documents.

## 1. In one page

- **temper's presentation, on jig's client.** temper's client root
  holds `jig-client` as a component (jig's `client/domain/root.md`):
  the link, watches by handle, keyed requests until durable, the work
  kept across a restart, and the forge's family carried as bytes. Its
  pages, words, confirmations and navigation are temper's own. jig's
  reference UI (jig's `client/domain/pages.md` and jig's
  `client/domain/view.md`) is not temper's: temper arranges no page of
  jig's, and copies no client root, view root, native client or card of
  jig's.
- **Its pages,** on jig's (jig's `client/domain/pages.md`): the inbox
  (home), the chats, a chat, a project's goals, a task's page, the
  changes, the forge's adoption form, the forges, and the LLM accounts
  (`pages/README.md`); "How work runs" and a proposal of settings' review (`pages/settings.md`, 2).
  The mockups design them: their look, parts, words and acts. What no page
  shows is open (section 5).
- **Its cards are its own.** A proposal, a question, a held task, a
  person task, a result, an entry of activity, an outcome and each
  confirmation are temper's nodes, drawn from the object's document as
  `jig-client` delivers it and from `jig-client`'s `facts` (jig's
  `client/domain/client/helpers.md`), as jig's
  `client/domain/objects.md`, section 2 allows an application. temper
  calls no renderer of jig's reference UI, and copies none of its
  modules, functions or phrase tables.
- **On jig's API alone.** What a page shows, it watches through jig's
  subjects (`me`, `inbox`, `tasks{query}`, `task`, `tree`, `run`,
  `accounts`) or
  reads (`activity`, `proposal`; jig's `client/domain/contract/reads.md`, section 1, and `client/domain/contract/watches.md`, section 1
  ); what a person does is jig's operations (jig's
  `client/domain/contract/requests.md`), through jig's helpers where
  they fit (`root.md`, 1.6).
- **The confirmation is temper's.** An act that cannot be undone says
  what it will do before it is sent, from the facts jig's documents
  give, and every deciding act names the revision the person was shown
  (`root.md`, 1.6).
- **LLM accounts, made from the client.** On the accounts pages,
  `Accounts` and its flow `ConnectModel`, a person signs in with an LLM
  provider, declares a program's login, or adds a key, pasted once and
  never kept (`pages/README.md`, `root.md`, 1.6); while no account their work may use is usable,
  a banner on every page says so and offers the three (`pages/accounts-words.md`, 1). A chat's
  composer and a task's page name the account and model the work runs
  on. A browser opens a sign-in's address in a new tab while the page
  waits; the terminal prints the address and opens no browser (`line.md`, 5).
  The accounts are jig's, kept by its engine
  (`../protocol/credentials-llm-accounts.md`, section 1).
- **A first run's playground.** When the person chooses it, the
  engine makes the playground, a fake forge, the scripted model's
  account and the playground's project (`../protocol/setup/playground.md`),
  and the client reads it from the engine (`forge/family.md`, 7): a banner of one line
  on every page says it is the playground and offers "set up temper",
  which resets it and opens the welcome's Forge step, and the
  playground's `NewChat` suggests the requests its model
  plays (`pages/welcome.md`, 2). The printed link's sign-in in a browser lands on the
  welcome where the document shows it, and on the inbox otherwise
  (`web/sign-in.md`, 1).
- **The forge's family.** Its words for the connector, a repository, a
  branch, a pull request and each kind of write need no document (`forge/family.md`, 1).
  Its other parts are documents its client side reads, with requests of
  their own (`forge/README.md`); a part it does not read says "Shown on the
  forge." (`forge/family.md`, 2). Its form is the adoption, with a picker of the
  repositories temper can reach on the chosen forge and "Not listed"
  for the others, sent through `jig-client`'s request table as the
  forge's bytes, like any other keyed request (`forge/family.md`, 3 to 5 and `forge/records.md` and `forge/family.md`, 6).
- **Forges connected from the client.** On the forges' pages, `Forges`
  and its flow `ConnectForge`, a person whose role holds the
  deployment's connectors (in standalone, the owner) sets up or
  connects a forge, which the set up or the connect makes as a record
  the store keeps (`../protocol/forge/engine.md`, 1 and 2), installs
  GitHub's App on another account, replaces a forge's key or tokens,
  forgets a managed forge's admin's token and removes a forge: the
  FORGE requests (`engine.md`; `forge/deployment.md`). A pasted value rides its request as jig's carried
  value, sent once and kept by neither the client nor the core (`root.md`, 1.6).
  A connect's or an install's link is printed, never opened (`line.md`, 5).
- **One crate of records.** `temper-client-protocol-wire` carries the
  forge family's records: the adoption, its outcome and the small
  store's record of a pending one (`forge/records.md`), the FORGE requests and their
  outcomes (`engine.md`), and those of `forge/served.md`. temper's
  server decodes them before the engine's root, which locates the
  repository and adopts it on the person's behalf, or runs the connect
  (`engine.md`).
- **One view, every shell.** `temper-client-view` builds temper's pages
  as skein-ui's tree (skein's `ui/README.md`); the terminal prints that tree
  with skein's text shell (skein's `ui/text.md`); the browser, its
  second shell, maps it with skein's browser parts (`web/README.md`).
- **A terminal first.** `temper-box open` runs temper's client, its view and
  skein's text shell over `jig-client-shell-native`, jig's native
  transport (`line.md`).
- **Signed in by jig's printed link,** on loopback, and kept signed in
  across restarts by its bearer credential, which the terminal keeps in
  the client's private directory (`sign-in.md`, section 1). A native client, the
  TUI, may instead be approved by the person's browser, signed in
  there, on the approval page (`pages/README.md`, `sign-in.md`, 1.1). The forge as people's
  sign-in provider is open (`README.md`, section 5).
- **temper's server owns the clients' listener,** and mounts jig's
  clients' component on it with jig's default HTTP mapping, under
  `/jig`, with its example guards, and routes `/forges/...` for a
  GitHub connect's browser steps (`engine.md`; `forge/served.md`, 1). The
  mapping's engine end
  owns its calls in flight: the server asks for its room before it
  hands it a call. An ordinary call under `/jig` that finds none waits
  for it, its body unread (`web/serving.md`, 1); once the server is
  stopping it answers that call itself, as it answers a path no one
  claims: `503`, with no body, closing the connection (jig's
  `protocol/client/default-http.md`, 4). A public read that finds none takes its
  own busy answer (17.5 there). The server's connections are its own
  bound, never the engine end's.
- **Joined to the engine from the start.** `tests/client`
  (`temper-client-world`) runs temper's client and view, with skein's
  fake person, against temper's engine root (`testing.md`;
  `../testing/checks.md`, 3).

## 2. The client's crates

```
temper's client
temper-client-domain            the client root: jig-client held, temper's pages, the forge's side (sections 3 and 4); its routes and every piece's items under today's paths (domain-crates.md)
├── temper-client-domain-pages  temper's pages: their state, facts and acts, reaching the root through pages::Root and pages::Routing
├── temper-client-domain-forge  the forge's side and the client's vocabulary: addresses, limits, owners, words, the public reader
├── jig-client                  jig's client: the link, watches by handle, the request table, kept work, families; jig's helpers
└── temper-client-protocol-wire the forge family's records, for both ends (4.6)
temper-client-view              the view: temper's pages as skein-ui's tree, temper's words (section 5); its state, build and act, the frame, and every piece's items under today's paths (view-crates.md)
├── temper-client-view-work     the pages of a project's work: home, goals, tasks, needs, the inbox, asks and chats
├── temper-client-view-setup    the pages that set temper up: accounts, models, welcome, settings, team, how work runs
├── temper-client-view-parts    what pages draw with: bindings, cards, choices, confirmations, the forge's and the accounts' parts, the shared surface, the public pages, sign-in
├── temper-client-view-words    the view's vocabulary: its phrasebook, its look's tokens, its paths' drawing, its limits
└── skein-ui                    skein's UI kit: the tree, the builder, bindings, the diff, phrasebooks

below the client
jig-client-protocol-wire        jig's records, for both ends, under jig-client
jig-protocol-transcript         turns decoded with smith's transcript crate, under jig-client

the shell
temper-client-shell-terminal    the terminal: temper's host, its settings and lines (section 6)
├── skein-ui-text               skein's text shell: the renderer, the command reader, the terminal
└── jig-client-shell-native     jig's native transport: skein's HTTP client with jig-client-protocol-http; the small store's file and the credential's
temper-client-tui               the TUI: its state, its events, its pages as skein-ui's tree, its words, its look's names and its theme (6.11 to 6.16)
└── skein-ui-screen             skein's full-screen shell's renderer: the theme, the frame, the focus (skein's ui-screen.md)
temper-client-shell-tui         the TUI's host, its seam, and its loop over the native transport and skein's full-screen terminal (6.13, 6.16); its start (6.11, 6.12)
├── skein-ui-screen-terminal    skein's full-screen terminal: the keys, the drawing, the terminal on io
└── temper-client-redaction     the journeys' shared originals, redaction and scan, which the TUI's and the browser's recorders share (tui/journeys.md, 2)
temper-shell                    what every binary of temper's starts from: its table's rows and help, the three directories, the engine's link at rest (6.12)
temper-box-shell                the box's binary's crate; `temper-box open` runs the line client (../protocol/binaries.md)
temper                         the deployment's client binary, bin/temper: its `main` hands temper-version's line to its crate (../cli/README.md, 2)
temper-client-shell             `temper`'s crate: `temper [ADDRESS]` runs the TUI, `temper box` the TUI in front of a box, `temper ctl` the agents' tools (../cli/README.md, 7.4)
└── temper-fetch-shell          the fetch of `temper-box` for `temper box` (box-binary.md)
temper-client-shell-web         the browser: temper's shell, a library for wasm32 and on every target, its host ordinary Rust (web/shell.md, 1)
├── skein-ui-web                skein's browser parts: the page, the DOM, history, fetch, session storage
├── jig-client-shell-web        jig's client adapter over skein's fetch, and the printed link's sign-in part
└── temper-client-redaction     the journeys' shared originals, redaction and scan (web/journeys.md, 2)
temper-client-web-page          the shipped page's entry, built for wasm32 into the bundle (web/shell.md, 1)

for agents
temper-client-tools             the agents' tools on jig-client, beside the view (ctl/README.md, section 4): temper's part, the tools' root, and jig's part under its path (ctl/tools-crates.md)
└── temper-client-tools-jig     jig's part of the tools: the table, a call, the runner, digests and conditions, the rendering
temper-client-shell-cli         `temper ctl`, over jig-client-shell-native, and its start (ctl/command-line.md, section 1)

for tests
skein-fake-person               the person, on skein-ui's tree (../testing/fakes.md, 4)
jig-client-test-kit             jig's check, and the faults of jig's client worlds (jig's `client/domain/testing.md`, 6.2 and section 7)
jig-client-fake-person-browser  jig's attack stories (jig's `protocol/browser/testing.md`)
temper-client-world             tests/client: temper's client joined to temper's engine root (section 10)
temper-client-view-kit          temper/testing/temper-client-view-kit: the view's tests' shared fixtures (view-crates.md, 4.2)
skein-fake-person-browser       the person in Chromium, through skein-browser (web/testing.md)
temper-client-browser-world     tests/client-browser: the browser tier (web/testing.md)
temper-client-gallery-tool      crates/temper-client-gallery-tool: the browser's pages in named states, as static documents (web/gallery.md)
temper-client-journeys-tool     crates/temper-client-journeys-tool, run by bin/temper-client-journeys: recorded walks replayed through the shipped hosts and printed (web/journeys.md; tui/journeys.md); QA's drive session and recipes (drive.md; recipe.md)
```

- **The crate graph.** `temper-client-domain` depends on skein-lib,
  `jig-client`, jig's records (`jig-client-protocol-wire`,
  `jig-client-protocol-link`) and `temper-client-protocol-wire`; so do
  its two pieces, below it (`domain-crates.md`, 2);
  `temper-client-view` on skein-lib, `skein-ui`, `jig-client` (for jig's
  facts and words) and `temper-client-domain`, and on no crate of jig's
  reference UI; so do its four pieces, below it (`view-crates.md`, 2). Neither links io, HTTP, smith, or an engine crate of
  temper's: `jig-client` emits jig's messages, never HTTP, and turns
  reach it decoded (jig's `client/domain/client/README.md`).
- **`temper-client-protocol-wire`** is a codec crate generated by
  skein's generator from a schema, depending on skein-lib and
  skein-codec alone, no crate of jig's (jig's `protocol/client/wire.md`, section 1, and `protocol/client/README.md`, section 8
  ). temper's server and the client both use it.
- **`temper-client-shell-terminal`** holds, as modules:
  - `client`: the native client, `NativeClient`: temper's client root,
    its view and `jig-client-shell-native`'s transport, and when
    `jig-client-shell-native` keeps the credential's file (`sign-in.md`, section 1).
    What its tests and the process tier's person wait on it shows read-only:
    `pending`, its readiness counts and the view's revision, and
    `address`, the address it last applied, by its own `go` or a root's
    `Show` (`view-navigation.md`, 1, *A new project*);
  - `route`: each of the root's requests to the native shell, the host
    or one of temper's lines;
  - `terminal`: skein's text shell's terminal with temper's settings,
    its own lines and the sign-in's retries (`line.md`, 2, 6, 7); and
    `Terminal`, the one host of skein's loop, composing the native
    client with stdin, stdout and the termination signals (`line.md`, 2). The
    worlds put skein's fake person where `temper-box open` puts the
    terminal (`testing.md`);
  - `limits`: the client's limits and their sum.
- **`temper-client-tui`** holds, as modules: `state`, the TUI's own
  state beside the client root (`tui/pages/README.md`); `event`, what its view makes of a
  person's acts; `words`, its own phrases; `view`, its pages as
  skein-ui's tree, with a module of its own for each part of a screen
  (`view::launchpad`, `view::foot`, `view::chat`); `look`, its owner and
  the names of its regions and tokens; `theme`, its two themes (`tui/theme.md`);
  and `limits`, its tree's. It depends on `skein-ui`, `skein-ui-screen`,
  skein-lib, `jig-client`, `temper-client-domain` and the view's words
  and parts (`temper-client-view-words`, `temper-client-view-parts`;
  `view-crates.md`, 6), whose words it says where it says what the view
  says, through the view's shared surface, `temper_client_view::shared`
  (`tui/pages/README.md`), and links no io.
- **`temper_client_view::shared`** is what the browser's terminal says
  that the TUI says too, as words and data with no tree: functions of
  the root and the view's words, each one the browser's view itself
  builds from, so that the two say the same thing alike (`tui/pages/README.md`, *Its
  words*). The view's builders and bindings stay its own.
- **`temper-client-shell-tui`** holds, as modules: `host`, the TUI's
  one seam (`tui/host.md`, 1); `run`, its loop (`tui/README.md`, 4); `limits`, the client's
  from `temper-client-shell-terminal`'s with the TUI's view and screen
  limits beside them; and `start`, the TUI's arguments and start (`tui/README.md`, 1, `tui/directory.md`, 1
  ): the address, the directory and what it keeps, the engine's link
  file read, the terminal's scheme and depth, the opener, and `run`.
- **The clients' shared start.** The client's `temper` and the box's
  `temper-box` use the same client code, in the shells they link, never
  through another binary (`../draft/deployment-binaries.md`, section 4):
  - `temper [ADDRESS]` calls `temper_client_shell_tui::start::run`, given
    the binary's name for the lines it prints (`tui/README.md`, 1), and
    `temper box` runs the same TUI in front of a `temper-box --front`
    child (`tui/box.md`; `box-binary.md`); the box runs no TUI;
  - `temper ctl` and `temper-box ctl` call
    `temper_client_shell_cli::start::run`, whose lines name `temper ctl`
    in both (`ctl/command-line.md`, 1.1);
  - an argument naming an engine's address or a sign-in link is read by
    `temper_client_shell_terminal::address`, for the TUI and for `temper-box
    open` (`line.md`, 1);
  - the engine's link file is `temper-shell`'s `link_file`, which the
    engine writes and the TUI reads (`tui/directory.md`, 1), and the TUI's state directory
    is made private by `temper-shell`'s `directories`.
- **Step code under the subset:** the domain, the forge's side, the
  view, the TUI's state, view and theme, and the wire. **Ordinary
  Rust:** the terminal's host, the TUI's host and loop, and the
  browser's shell, as a shell is (skein's `ui/README.md`, section 2).
- **The client's limits** are constants of
  `temper-client-shell-terminal`: `jig-client`'s, from jig's small
  profile with the overrides temper's configuration gives the engine's
  documents (`../protocol/configuration/limits.md`); the pages' and
  the forge side's; skein-ui's, derived from the view's (`view.md`, 4); and their
  sum. The client reads no configuration: its only inputs are its two
  arguments (`../protocol/configuration/README.md`, section 7).

The native terminal, the browser and `temper ctl` receive sixteen entries
in a document's short lists, including creating's complete charter and
procedure lists, sixteen in an authority's grants and delegation kinds,
and eighty live delegates in a chat's close. This covers `temper-box init`'s
and the owned example's `standard` profile: its `documents.lines`
sixteen, its grants and executors sixteen, its `tasks.delegates` eighty
(`../protocol/configuration/stock.md`); a settings file and Ask temper's
proposal of settings take as many grants and kinds. Each is
a receiving capacity, independent of how many rows a screen shows. A list
longer than a client receives makes its document undecodable there, and
jig's link drops its stream on such a record: every watch ends, and the
page reads Offline and Behind.

## 3. The client root

Moved to [`root.md`](root.md), section 1.

## 4. Signing in and the small store

Moved to [`sign-in.md`](sign-in.md), section 1.

## 5. Open questions

### 5.1 The pages

### 5.2 A chat

### 5.5 Clients that are not a person's

- **Clients that are not a person's:** a command run once and a
  service acting as a party are the agents' tools, `temper ctl`
  (`ctl/README.md`). A bot: none.

### 5.8 Signing in

Moved to [`sign-in.md`](sign-in.md), section 2.

### 5.10 The client's own

Moved to [`root.md`](root.md), section 2.
