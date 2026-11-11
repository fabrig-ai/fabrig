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
  reads (`activity`, `proposal`; jig's `client/domain/contract/README.md`,
  sections 4 and 5); what a person does is jig's operations (jig's
  `client/domain/contract/requests.md`), through jig's helpers where
  they fit (3.6).
- **The confirmation is temper's.** An act that cannot be undone says
  what it will do before it is sent, from the facts jig's documents
  give, and every deciding act names the revision the person was shown
  (3.6).
- **LLM accounts, made from the client.** On the accounts pages,
  `Accounts` and its flow `ConnectModel`, a person signs in with an LLM
  provider, declares a program's login, or adds a key, pasted once and
  never kept (`pages/README.md`, `README.md`, 3.6); while no account their work may use is usable,
  a banner on every page says so and offers the three (`pages/accounts.md`, 2). A chat's
  composer and a task's page name the account and model the work runs
  on. A browser opens a sign-in's address in a new tab while the page
  waits; the terminal prints the address and opens no browser (`line.md`, 5).
  The accounts are jig's, kept by its engine
  (`../protocol/credentials.md`, section 3).
- **A first run's playground.** When the person chooses it, the
  engine makes the playground, a fake forge, the scripted model's
  account and the playground's project (`../protocol/setup/playground.md`),
  and the client reads it from the engine (`forge/family.md`, 7): a banner of one line
  on every page says it is the playground and offers "set up temper",
  which resets it and opens the welcome's Forge step, and the
  playground's `NewChat` suggests the requests its model
  plays (`pages/welcome.md`, 2). The printed link's sign-in in a browser lands on the
  welcome where the document shows it, and on the inbox otherwise
  (`web/README.md`, 2.4).
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
  value, sent once and kept by neither the client nor the core (3.6).
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
  the client's private directory (section 4). A native client, the
  TUI, may instead be approved by the person's browser, signed in
  there, on the approval page (`pages/README.md`, `README.md`, 4.1). The forge as people's
  sign-in provider is open (section 5).
- **temper's server owns the clients' listener,** and mounts jig's
  clients' component on it with jig's default HTTP mapping, under
  `/jig`, with its example guards, and routes `/forges/...` for a
  GitHub connect's browser steps (`engine.md`; `forge/served.md`, 1). The
  mapping's engine end
  owns its calls in flight: the server asks for its room before it
  hands it a call. An ordinary call under `/jig` that finds none waits
  for it, its body unread (`web/README.md`, 2.2); once the server is
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
temper-client-domain            the client root: jig-client held, temper's pages, the forge's side (sections 3 and 4)
├── jig-client                  jig's client: the link, watches by handle, the request table, kept work, families; jig's helpers
└── temper-client-protocol-wire the forge family's records, for both ends (4.6)
temper-client-view              the view: temper's pages as skein-ui's tree, temper's words (section 5)
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
└── skein-ui-screen-terminal    skein's full-screen terminal: the keys, the drawing, the terminal on io
temper-shell                    what every binary of temper's starts from: its table's rows and help, the three directories, the engine's link at rest (6.12)
temper-box                      the box's binary; `temper-box open` runs the line client; `temper [ADDRESS]` runs the TUI (../protocol/binaries.md)
temper-client                   the deployment's client: `temper-client [ADDRESS]` runs the TUI, `temper-client ctl` the agents' tools (../protocol/binaries.md)
temper-client-shell-web         the browser: temper's shell, built for wasm32, its host ordinary Rust and a library on every target (web/README.md, 2.1)
├── skein-ui-web                skein's browser parts: the page, the DOM, history, fetch, session storage
└── jig-client-shell-web        jig's client adapter over skein's fetch, and the printed link's sign-in part

for agents
temper-client-tools             the agents' tools on jig-client, beside the view (ctl/README.md, section 4)
temper-client-shell-cli         `temper ctl`, over jig-client-shell-native, and its start (ctl/README.md, section 6)

for tests
skein-fake-person               the person, on skein-ui's tree (../testing/fakes.md, 4)
jig-client-test-kit             jig's check, and the faults of jig's client worlds (jig's `client/domain/testing.md`, 6.2 and section 7)
jig-client-fake-person-browser  jig's attack stories (jig's `protocol/browser/testing.md`)
temper-client-world             tests/client: temper's client joined to temper's engine root (section 10)
skein-fake-person-browser       the person in Chromium, through skein-browser (web/testing.md)
temper-client-browser-world     tests/client-browser: the browser tier (web/testing.md)
temper-client-gallery           tools/gallery: the browser's pages in named states, as static documents (web/gallery.md)
temper-client-journeys          tools/journeys: recorded walks replayed through the shipped host and printed as pages (journeys.md)
```

- **The crate graph.** `temper-client-domain` depends on skein-lib,
  `jig-client`, jig's records (`jig-client-protocol-wire`,
  `jig-client-protocol-link`) and `temper-client-protocol-wire`;
  `temper-client-view` on skein-lib, `skein-ui`, `jig-client` (for jig's
  facts and words) and `temper-client-domain`, and on no crate of jig's
  reference UI. Neither links io, HTTP, smith, or an engine crate of
  temper's: `jig-client` emits jig's messages, never HTTP, and turns
  reach it decoded (jig's `client/domain/client/README.md`).
- **`temper-client-protocol-wire`** is a codec crate generated by
  skein's generator from a schema, depending on skein-lib and
  skein-codec alone, no crate of jig's (jig's `protocol/client/wire.md`, section 1, and `protocol/client/README.md`, section 8
  ). temper's server and the client both use it.
- **`temper-client-shell-terminal`** holds, as modules:
  - `client`: the native client, `NativeClient`: temper's client root,
    its view and `jig-client-shell-native`'s transport, and when
    `jig-client-shell-native` keeps the credential's file (section 4);
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
  skein-lib, `jig-client`, `temper-client-domain` and
  `temper-client-view`, whose words it says where it says what the view
  says, through the view's shared surface, `temper_client_view::shared`
  (`tui/pages/README.md`), and links no io.
- **`temper_client_view::shared`** is what the browser's terminal says
  that the TUI says too, as words and data with no tree: functions of
  the root and the view's words, each one the browser's view itself
  builds from, so that the two say the same thing alike (`tui/pages/README.md`, *Its
  words*). The view's builders and bindings stay its own.
- **`temper-client-shell-tui`** holds, as modules: `host`, the TUI's
  one seam (`tui/README.md`, 3); `run`, its loop (`tui/README.md`, 4); `limits`, the client's
  from `temper-client-shell-terminal`'s with the TUI's view and screen
  limits beside them; and `start`, the TUI's arguments and start (`tui/README.md`, 1, 2
  ): the address, the directory and what it keeps, the engine's link
  file read, the terminal's scheme and depth, the opener, and `run`.
- **The clients' shared start.** The client's `temper`, the box's
  `temper-box` and `temper-client` use the same client code, in the
  shells they link, never through another binary
  (`../draft/deployment-binaries.md`, section 4):
  - `temper [ADDRESS]` and `temper-client [ADDRESS]` call
    `temper_client_shell_tui::start::run`, given the binary's name for
    the lines it prints (`tui/README.md`, 1); the box runs that TUI by
    `temper-box internal tui` until C3 (`../cli/README.md`, 6);
  - `temper ctl` and `temper-client ctl` call
    `temper_client_shell_cli::start::run`, whose lines name `temper ctl`
    in both (`ctl/README.md`, 6.1);
  - an argument naming an engine's address or a sign-in link is read by
    `temper_client_shell_terminal::address`, for the TUI and for `temper-box
    open` (`line.md`, 1);
  - the engine's link file is `temper-shell`'s `link_file`, which the
    engine writes and the TUI reads (`tui/README.md`, 2), and the TUI's state directory
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

`temper-client-domain` is temper's client root, in jig's one shape
(jig's `client/domain/root.md`, section 3): `new`, `step`, `fire`,
`next_deadline` and `take`, and `shown`, a count of what is shown,
which the view builds from. It holds `jig_client::Client` as a
component and calls it; the client never calls it back, and the root
routes nothing of jig's (jig's `client/domain/client/README.md`, section 4).

### 3.1 What it holds

- **`jig-client`,** given its limits and a seed at construction.
- **Two children, each an owner** (jig's `client/domain/root.md`,
  section 15): temper's pages, owner `Pages`, which ask the client for
  jig's operations, reads and subjects only; and the forge's client
  side, owner `Forge` (`forge/family.md`); numbered in 4.6. Each is the asker
  of every handle and key it gives `jig-client`, and draws its keys'
  numbers from the seed the shell gives the root.
- **`me`,** watched from start to sign-out under a handle of its own:
  the signed-in party, their projects with their roles and pools, what
  waits for them counted, and each project's merged templates (jig's
  `client/domain/contract/documents.md`, 1). Every page shows the link's state,
  `jig-client`'s `Link`.
- **The page at its address** (`pages/README.md`): its handles, its windows, the
  composer's words, and the open confirmation (3.6), at most one.
- **The project the person is in,** chosen among `me`'s projects; the
  first at start.
- **The pages' acts in flight,** each by its key: which page's card or
  composer sent it, so its answer reaches what shows it (3.3).

Retained state is bounded by the domain's limits: the handles a page
opens, the rows of each window, the composer's bytes, the acts in flight.

- **Internal hand-offs are uncapped.** The pages, the forge's side and
  `jig-client` hand off through queues that grow as they fill. Each
  step settles them; each event's fan-out is bounded by admitted state.
  No client has a watermark: new work enters through capped edges,
  the engine link's exchanges, the kernel's queues and terminal or
  browser input (programming-model.md, 7.1). The terminal, browser and
  CLI shells use uncapped internal hand-offs too.

### 3.2 Its modules

```
temper-client-domain/src/
├── lib.rs          the tree, re-exports
├── boundary.rs     events in (the start, a place gone to, the person's acts, jig-client's events), requests out (jig-client's, a place shown, signing out, kept work dropped, facts)
├── owner.rs        temper's two owners, Pages and Forge, as askers (3.3)
├── domain.rs       Domain: new, step, fire, next_deadline, take, shown; jig-client held
├── route.rs        each event to its child; what jig-client tells, to the child its asker names (3.3)
├── address.rs      temper's addresses and the inbox's filters (3.5)
├── pages.rs        the pages: their handles, the project the person is in, navigation (3.1)
├── pages/          one module per page: what it watches and reads, its state, its acts
├── acts.rs         acts as jig's operations, through jig's helpers; the confirmation (3.6)
├── amend.rs        the amendment editor: a draft of a task's controls, to jig's Amendment (3.7)
├── forge.rs        the forge's client side (section 4), with its modules below forge/
├── limits.rs       jig-client's limits, the pages', the forge side's, and their sum
└── tests.rs
```

### 3.3 Routing

As jig's client root routes (jig's `client/domain/root.md`, section 5),
with temper's two children:

- **Out and in:** `jig-client`'s requests to the transport leave the
  domain as they are, for the shell to carry; the transport's events
  reach `jig-client` as they are.
- **By asker:** what `jig-client` says of a handle or of a key goes to
  `Pages` or `Forge`, matched whole; an asker that names neither is a
  fact (`UnknownAsker`), and reaches no child. The pages hand it to the
  page that opened the handle, or the card or composer that sent the
  key, and drop one for a handle they closed; the forge's side to its
  form, its picker or the forges' page that sent it.
- **The link** goes to both children, and changes what every page
  shows; a page reopens nothing of its own, since `jig-client` keeps the
  watches. Kept work dropped goes to the shell (3.4).
- **A place gone to** goes to the pages, and to the forge's side when
  it is `Adopt { project }`; any other tells the forge's side it left.
  The forge's side watches the deployment's forges for the session
  (`forge/records.md`), whatever page is open.

### 3.4 At start

The shell starts the domain with the small store's records and the
inbox's address, then signs in (section 4), holding back neither. The
domain's children ask their watches from their start, `me` among them,
and `jig-client` opens none on the engine until it is signed in (jig's
`client/domain/client/README.md`, section 9), so no stream leaves without the
credential.

- **Another party's kept work** stays parked and is not shown (jig's
  `client/domain/client/README.md`, section 8); the terminal says so in a line
  of its own (`line.md`, 6).
- **Pending reads at sign-in** end visibly in jig's existing forms,
  on successful installation even before the first `me`, and on
  sign-out (jig's `client/domain/client/README.md`, sections 6 and 9).
  Whoever asked must ask again; no read is replayed automatically.
- **Kept work dropped:** the domain hands `jig-client`'s `Dropped {
  count }` to the shell (jig's `client/domain/root.md`, section 8), and
  the terminal says so in a line of its own (`line.md`, 6).
- **temper's pages keep no record of their own:** a client closed on
  any page opens at the inbox, and a composer's words not sent, or a
  form's values, are not kept across a restart (section 5). A browser
  opens at its address (`web/README.md`, 2.3), and after the printed
  link's sign-in at `Welcome` (`web/README.md`, 2.4;
  `../draft/archive/first-run.md`, 2.3), which shows its first step while the
  store is empty, opens at the step left undone while it is
  unfinished, and moves on to the chats page once it is whole (`pages/welcome.md`, 1).
  On a deployment with no welcome it moves on to the
  inbox, with nothing of the welcome drawn first (`pages/welcome.md`, 1).
  The terminal, which opens at the inbox, reads the same fact for the
  pages it changes.

### 3.6 Requests and confirmations

- **One act, one request,** keyed by the domain and sent through
  `jig-client`, which keeps it until its answer is durable (jig's
  `client/domain/client/requests.md`): a double press joins the
  pending one, and nothing is shown done before its answer.
- **jig's operations, through jig's helpers** (jig's
  `client/domain/client/helpers.md`; jig's
  `client/domain/contract/requests.md`, 2 and 4): starting a chat is
  `shapes::chat`; setting a goal is `shapes::set_goal`, or from a template
  `shapes::set_goal_from` (`pages/goal.md`, 3), a proposal where the person's authority
  does not cover it; closing a chat is
  `shapes::close_chat`, one compound request, with what becomes of each
  live delegate; amending and releasing a held task is one compound
  request; a goal's priority changes by its amendment (3.7).
  Every other act is one operation.
- **Said before it is sent.** An act that cannot be undone (cancelling,
  closing a chat, accepting, rejecting, releasing, removing a forge,
  deleting a project) says what it will do before it is sent, from the facts the document gives (a decision's
  facts, as `jig-client`'s `facts` reads them): what accepting grants
  and costs and the pool after it, what cancelling ends, what releasing
  does (jig's `client/domain/objects.md`, section 1; jig's
  `client/domain/contract/documents.md`, 4). Closing a chat says only what closing
  changes (*The sheet,* below). Removing a forge says what goes and what stays on the
  forge (`forge/deployment.md`); deleting a project asks for the project's
  name, and sends nothing until it is typed whole (`pages/README.md`). Pause and Resume, on a
  chat, a goal or an agent's task, are undone by each other and sent at once. A
  goal's Close says that what landed on its branch stays there (`pages/task.md`, 4). Where the document offers an act
  as proposable (jig's `client/domain/contract/documents.md`, 3), the card says
  so, and to whom; where it offers it as refused, the card says why and
  sends nothing.
- **The sheet.** Every confirmation of `confirm.rs` is one sheet, from the
  look request `close-chat`: the mock's `dialog.sheet`, narrower, centred over the page. It asks in a
  sentence, says only what will change, and names its acts: what keeps things
  as they are first, then the act itself, primary. There is no "Confirm" and no
  "Dangerous action": the sheet is how temper marks an act that can't be undone
  (jig's `client/domain/client/helpers.md`).

  | Confirms | Title | Says | Keeps | Does |
  |---|---|---|---|---|
  | closing a chat | "Close this chat?" | "You can still read it, but it can’t be reopened.", and what it changes | "Keep it" | "Close chat" |
  | "Start over" | "Start over?" | "A new Ask temper chat starts. This one stays in Chats.", and what it changes | "Keep going" | "Start over" |
  | closing a goal | "Close this goal?" | "You can still read it, but it can’t be reopened.", then "What landed on its branch stays there." | "Keep it" | "Close goal" |
  | closing an agent's task | "Close this task?" | "You can still read it, but it can’t be reopened.", and what it changes | "Keep it" | "Close task" |
  | accepting a proposal | "Accept this proposal?" | its facts | "Not yet" | "Accept" |
  | rejecting a proposal | "Reject this proposal?" | its facts and reason | "Keep it" | "Reject" |
  | releasing a task | "Release this task?" | what releasing does; an uncertain effect's two choices | "Keep it held" | "Release" |

  - *What it changes,* under a hairline, each line present only where it holds,
    and no count at zero:
    the run, while one of this task's is live: "The run under way gets N seconds
    to finish, then stops.", the template's grace, or "The run under way stops
    now." with none (a chat at rest closes at once, jig's `domain/tasks/lifecycle.md`,
    9); each live delegate, by its number and title and its fate, "T4 Fix the
    parser is cancelled." or "T5 Review the parser moves to you.", with the other
    fate as "Take it instead" or "Cancel it instead"; the tasks below it that are
    not delegates, "N tasks below it are cancelled.", the document's waiting,
    running, held and closing counts summed, less the delegates listed; "N
    proposals are withdrawn."; and, while any of those holds, "Changes not yet
    begun are withdrawn. Those under way settle.". A chat at rest with nothing
    below it has no hairline and no such line: closing it changes nothing an
    effect is in flight for, and the connector's panel names one that is (jig's
    `client/domain/objects.md`, 11). What closing doesn't change (completed,
    failed and cancelled tasks) is not said.
  - *It stays until it is answered.* The press sends the ask and the sheet stays,
    saying the ask's progress above its acts. Answered done, it goes. Any other
    outcome leaves it open, saying why above its acts, where a press sends again;
    a press that sends nothing closes it, but for a rejection with no reason, which
    keeps it open and says nothing. The act that keeps things closes it at any time,
    Escape pressing it, and an ask in flight goes on, its outcome on the page as before. While the sheet is
    open, the outcome and progress of its own ask are the sheet's alone.
  - *A press stays under the finger* (`web/README.md`, *A press a render can lose*):
    the acts are a group of their own, keyed, and each delegate's row keyed by its
    task, so a line arriving above them, or the page updating behind, keeps each
    button's element.
  - *The terminal* prints the same lines as text, and the acts by their names.
- **Closed ends on one line.** A chat its requester closed ends on one quiet line,
  "Closed at 21:05" (*The Chats page,* its ended line), between two hairlines; its
  bar's status says "Closed", with no time. "Applied" isn't said once it is
  closed, nor "Decision made" for the closing itself: that line says it; nor
  after a chat's Pause or Resume: its status says it (`pages/chat/README.md`, 1, *A person's pause*).
- **It names what the person saw.** Each deciding act names the
  revision of the document the card showed; when the engine refuses it
  as moved, the card asks the person to look again (jig's
  `client/domain/contract/documents.md`, 2). jig's check holds it in temper's
  worlds (`testing.md`).
- **Answered:** the card shows the outcome, or the refusal in jig's
  words with what fits (look again, change what was asked); unknown,
  past the engine's retention, it says so. A goal's page says it in the toast (`pages/task.md`, 4).
- **A key is pasted once, never kept.** Its field is a secret (skein's
  `ui/README.md`, 3.2 and 3.4), and adding or replacing a key is a request
  carrying a value (jig's `client/domain/client/requests.md`; jig's
  `protocol/client/README.md`, section 7): sent once under a name, never in
  the small store, never parked, never sent again by the client. The
  field is emptied as the request leaves, and the key is shown nowhere,
  then or after. Answered unsent or lost, the card says so and asks for
  the key again; the person's second send keeps the first's name, so
  that a request the engine made answers with its first answer, and no
  second account is made. A key empty or past 4 KiB (the bound of
  `ctl/README.md`, section 9) is refused before it is sent; one the
  engine refuses is answered with its refusal.
- **A forge's pasted value is sent the same way.** A GitHub App's
  private key (with no webhook secret: GitHub has none of temper's),
  a Forgejo bot's tokens and their webhook secret, a site
  admin's token, and a replacement of any of them are secret fields,
  and the request is a request carrying a value (jig's
  `protocol/client/README.md`, section 3): the value is sent once, never in
  the small store, never parked, never sent again by the client, and
  the fields are emptied as the request leaves. The request names no
  number for it and the client draws none: jig numbers the value as it
  keeps it (`../protocol/credentials.md`, pasted values). Answered
  unsent or lost, the page asks for the value again, and the second
  send keeps the first's key. A value empty, or a bundle past 4 KiB,
  jig's bound on a carried value (the bound of `ctl/README.md`, section 9),
  is refused before it is sent, in the words of `forge/deployment.md`; one
  the engine refuses, past the
  deployment's `records.value_bytes` among them, shows its refusal
  (`engine.md`). The client reads no configuration, so its bound is jig's.

### 3.7 The amendment editor

One editor drafts a person's amendment of a task, for every page and
command that amends one: the task's pages (`pages/goal.md`, 2, `pages/task.md`, 2) and `temper ctl
task amend` (`ctl/areas.md`). It is `temper-client-domain`'s `amend`
module, beside `acts.rs`, and pure: it reads a document and builds
jig's `Amendment`; it sends nothing, judges no bounds and keeps no
clock. Each caller lays out its own fields, compares the change with
the task's offered `amend` act by jig's admission (`c::admission::
amendment`), and sends it, as `Amend` or `Propose`, as 3.6 says.

- **A draft is made from the task's document** as read,
  `amend::Draft::of(&c::Task)`: its base is the document's
  `spec_revision`, and the controls the amendment names, as the
  document shows them, the same as jig's `OwnerAmendmentState` holds
  them, with the tracking and the LLM choice: its tracking (`priority`,
  tracked when supplied), its spec (`SpecificationDocument::whole()`),
  its authority (`OwnerAuthorityDocument::whole()`), its instructions,
  its dependencies (its links' numbers, in order) and its `llm`.
- **An edit sets one control,** whole, and leaves the rest as the
  base holds them:

  | Edit | Sets in the `Amendment` |
  |---|---|
  | `track(Tracking)` | `tracked`: `Tracked { priority }` or `Untracked` |
  | `deadline(Option<Wall>)` | `authority`: the base's, with its `deadline` replaced |
  | `instructions(Text)` | `instructions`: whole; empty is none, explicitly |
  | `words(Text)` | `spec`: the base's, with its `words` replaced |
  | `dependencies(List<u64>)` | `dependencies`: the whole list |
  | `llm(LlmChoice)` | `llm`: the whole choice |

  An edit equal to the base is no change. A deadline needs the base's
  whole authority and words its whole spec: where the document cut it,
  the edit answers `Cut::Authority` or `Cut::Spec`, and the draft is
  unchanged, so a page shows the field read only and `ctl` says so.
- **Its revision.** `Draft::revision()` is the base's. `Draft::moved
  (&c::Task)` says whether a document read since has another
  `spec_revision`: a page's draft is then stale, and says so before
  sending; `ctl` compares it with its `--rev REV`.
- **The change,** `Draft::amendment() -> Option<Drafted>`: none
  without a change; else `Drafted { revision: Option<Revision>,
  change: c::Amendment }`, the revision absent exactly when the change
  is a tracked task's priority alone (jig's `domain/tasks/changes.md`
  ), and the base's otherwise.
- **What is whose.** The editor holds the rules of a draft (what
  each edit keeps, what is cut, when a revision is named). The words
  and fields are the caller's: a page's form, `ctl`'s flags and its
  lines on stderr.

## 4. Signing in and the small store

How jig signs a client in, and how a native client keeps its credential
and its small store, are jig's (jig's `protocol/client/sign-in.md` and `protocol/client/README.md`, sections 7 and 9
; jig's `client/domain/client/README.md`, sections 8 and 9).
temper's:

- **A sign-in's address in the small store** (`pages/accounts.md`, 1, `ConnectModel`, *A reload
  while it waits*): a browser's shell keeps, in one record of its own under
  the key `sign-in`, beside jig's, the latest answered sign-in's source, its
  address and whether its tab was refused, so that a reload draws the same
  page. The address is the provider's authorization address, with its state
  and challenge and no verifier, which the page's link and the tab's history
  show anyway. A later sign-in's record replaces it, and it goes with the tab.
  The record is read only for that source: another's is no sign-in of the page's.

- **The printed link** (jig's `protocol/client/sign-in.md`): at
  start, `temper-box serve`, or `temper box` before it starts serve
  (`../protocol/setup/box.md`, 2), prints a sign-in link for the deployment's
  first owner (`../protocol/binaries.md`), in the default
  mapping's link route under temper's prefix,
  `http://HOST:PORT/jig/v1/sign-in/link?secret=SECRET`. `temper-box open
  DIRECTORY LINK` checks that the link names the engine's address on
  loopback and that route, and gives the secret to `jig-client` with the
  credential DIRECTORY holds, if any (`SignIn { with: Link { secret,
  held } }`). The secret is in the command line only until the sign-in
  spends it; the client root passes it on and keeps nothing of it. The
  running engine also keeps a local link of its own at rest, never the
  printed one, for a TUI on the same machine (`tui/README.md`, 2, *The engine's link,
  kept at rest*).
- **A sign-in held is kept** (jig's `domain/people/README.md`, section 3): a
  person still signed in who opens a fresh link with the same DIRECTORY
  keeps their sign-in, its number and its expiry, and the deployment's
  sign-ins don't grow. The engine checks the credential offered as any
  exchange's and answers with none; `jig-client` goes on holding the
  one offered, and `DIRECTORY/token` is not written. A credential the
  engine does not know, or one ended or expired, is no refusal: the
  link signs the person in afresh, and its credential is written. A
  link refused, answered not ready, busy or full, or an engine not
  reached, leaves `DIRECTORY/token` as it was.
- **The bearer credential,** the sign-in's number beside 32 random
  bytes, is `DIRECTORY/token`, a private file (`0o600`), which
  `jig-client-shell-native` writes where the host says, when it says.
  Once signed in by a link, the host reads the credential from
  `jig-client`'s `link()`, through the client root, and has the native
  shell keep it, before anything is told of the sign-in; a new sign-in
  by a link has it written anew, and a sign-in kept writes nothing,
  its credential unchanged; signed out, the host has the native
  shell keep none, which erases it. A later `temper-box open DIRECTORY
  ADDRESS` has the native shell read it before its loop (`line.md`, 1), and the
  host gives it back as held (`SignIn { with: Held(credential) }`). It
  is never printed and never an argument, and neither temper's root
  nor its view keeps it.
- **The small store** is `DIRECTORY/store`, `jig-client-shell-native`'s
  file, holding `jig-client`'s records alone: the requests it keeps,
  jig's and the forge's adoptions, pending. temper's pages keep none
  (3.4). It never holds an LLM account's key, token or record, nor a
  forge's pasted value: a request carrying a value is never kept (3.6),
  and a sign-in's record and a forge's credentials are the engine's
  alone (`../protocol/credentials.md`). A pending FORGE request that
  carries no value (a manifest's `Connect`, `Install`, `Forget`,
  `SetUp`, `Reachable`) is kept as any keyed request; one carrying a
  value is not.
- **Loopback only** in standalone (`../protocol/configuration/README.md`,
  section 2; `line.md`, 1), so a bearer token never crosses a network in
  plaintext. Clients between hosts, over TLS, come after standalone.
- **One person.** The deployment's link signs in its first owner;
  signing in through the forge is open (section 5).

### 4.1 A terminal signed in from the browser

jig's third built-in way (jig's `protocol/client/sign-in.md`, *A
sign-in approved by a browser*; `../draft/tui.md`, 4.2): a native
client on the person's machine, the TUI, asks; the person's browser,
signed in by its cookie, approves at the approval page (`pages/README.md`, 2, *The
approval page*); the client takes the sign-in with the secret its ask
was answered with. A box enables it beside the printed link
(`../protocol/configuration/engine.md`, clients).

- **The root's half,** for a native client, in `temper-client-domain`,
  a module of its own, `asked`:
  - *Asking:* `Event::AskApproval`, the host's, while the link is signed
    out; at any other time, or while an ask is in flight, it changes
    nothing. The root sends `jig-client`'s `AskApproval` and holds
    `asked::State::Asking`.
  - *Asked:* on `jig-client`'s `Approval::Asked { secret, code, expires
    }`, the root signs in at once, `SignIn { with: Approval { secret }
    }`, and holds `Asked { code, expires }` alone, for its host to draw.
    The secret goes to `jig-client`, which holds it while the take runs
    (jig's `client/domain/client/README.md`, section 9), never into the
    root's state, a print or a log. An unlinked client (jig's
    `client/domain/client/README.md`, 4.4) leaves the take to its host's link,
    which made the ask and holds the secret: the root holds the code
    and the expiry, and drops the secret it was told.
  - *Not asked:* `NotAsked(Refused | NotReady | Busy | Lost)`, held as
    it came, for its host's line.
  - *The take's outcomes are the link's,* in `jig-client`'s `link()`:
    while it waits, `not_signed_in` is `Approval(Waiting)`, and
    `jig-client` sends the take again each second; refused, expired,
    used and unknown end it, `Approval(..)`; full, `Full { .. }`, its
    `link_until` the approval's own expiry; signed in, the link starts,
    its credential in `link().credential`, which the host keeps as it
    keeps a link's (section 4, *The bearer credential*). Once the link
    leaves signed out, the root drops what it held.
  - *Asking again* is another `AskApproval`: its take replaces the one
    before.
  - `Domain::asked()` gives the state to the host and the view, and the
    root's shown count moves with it.
- **The TUI's flow,** on the TUI's sign-in screen (`../draft/tui.md`,
  4.1), its host and its view, the TUI's crates:
  - *It asks* once each time its sign-in screen opens signed out (`tui/pages/README.md`:
    at its start with no credential kept, and again after a sign-out or
    a kept credential the engine refused), while the root holds no ask:
    the host steps `Event::AskApproval`. A held credential's sign-in in
    flight is not signed out, so nothing is asked until the engine has
    answered it. The screen still takes the printed link, pasted, as
    `../draft/tui.md`, 4.1 has it.
  - *Asking:* until the engine answers, "Asking temper for a code…",
    dim, under the screen's first line.
  - *The tab's address* is the TUI's own, made from its own parts alone
    by `temper-client-tui`'s `approval_address(address, code)`: the
    engine's address it was given or kept, `http://HOST:PORT`, then the
    approval page's printed path, `address::print_approve(Some(code))`,
    `/terminal?code=WDJB-MJHT` (`web/README.md`, 2.3), the code an
    `ApprovalCode`, so of the code's alphabet by its type. An answer
    whose code is off the alphabet is refused by jig's reader, so
    nothing is opened for it. It opens no address the engine sent.
  - *Opening the tab:* on each ask answered, once, the host's duty
    `open` carries the address, and the view draws it as opened until
    the opener's report says otherwise. The run gives the address to the
    opener as its one argument, with no shell between: `xdg-open
    ADDRESS` on Linux, `open ADDRESS` on macOS, a departure, on purpose,
    from `temper-box open`, which opens no browser (`line.md`, 5). The opener's
    stdin, stdout and stderr are none of the terminal's, so nothing it
    says reaches the screen; its environment is the TUI's. Its exit
    ended, the run reports `Opened { opened }`, done on 0. Closing
    aborts and reaps the opener alone, without waiting for it to finish
    by itself or signalling the browser's group (skein's `runtime/io/processes.md`
    , `SpawnAlone`).
  - *The opener* is the binary's choice, at start: none on an SSH
    session (`SSH_CONNECTION` or `SSH_TTY` set) or, on Linux, with no
    display (neither `DISPLAY` nor `WAYLAND_DISPLAY` set); else the
    program of that name in the first absolute directory of `PATH` that
    holds it as an executable file; none when no directory does. With
    none, the run reports `Opened { opened: false }` at the duty itself.
  - *Its lines,* the code and the address drawn, the code as shown:

    ```
    Sign in this terminal
    A tab opened in your browser. Check it shows
    WDJB-MJHT, then press Approve.
    No tab? Open http://127.0.0.1:8367/terminal?code=WDJB-MJHT
    The code expires in 10 min.
    ```

    "The code expires in SPAN." is `view.md`, 6's *span* to the ask's expiry,
    with its tick. On the screen, "Sign in this terminal" is `tui/pages/README.md`'s
    first line, "Sign in this terminal to HOST:PORT", said once; the
    approval's lines follow it, then the paste's, "Paste the link temper
    printed, then Enter." and its field.
  - *No browser to open:* an SSH session, no display, or the opener
    failing (not found, or exiting other than 0): the TUI opens nothing
    more and prints the address and the code, "Open this address in a
    browser that reaches temper:", the address, "and check it shows
    CODE, then press Approve.", then "From another machine, tunnel port
    PORT to this one first.", then the expiry's line. The engine is on
    loopback, so a browser elsewhere reaches it through a tunnel to its
    listener on the same port, which jig's guards require of the
    address (jig's `protocol/browser/README.md`, 12.2).
  - *Asking again:* Enter in the Link field while it is empty, once the
    approval has ended or was not asked, or a pasted link has replaced
    it, asks again (`may_ask_again`), and its line says "Press Enter to
    ask again." The field has the focus on this screen and takes every
    character, `q` and `a` among them (`tui/pages/README.md`), so no letter can be the
    key. While an approval waits, Enter in the empty field is the
    paste's "not a link" refusal, as before.
  - *A pasted link, sent,* replaces the approval's take (jig-client's
    sign-in in flight is the newest one asked): the approval's lines
    give way to the paste's progress and its refusal, and Enter in the
    emptied field asks again.
  - *Each outcome's line,* in place of the screen's last line:

    | Outcome | Its line |
    |---|---|
    | waiting | the screen as drawn |
    | refused | "The browser refused this sign-in. Press Enter to ask again." |
    | expired | "The code expired before it was approved. Press Enter to ask again." |
    | used | "That code was already used. Press Enter to ask again." |
    | unknown | "temper doesn't know that code any more: it restarted, or the code is old. Press Enter to ask again." |
    | full | "All N sign-ins are in use; the earliest ends MOMENT UTC. Press Enter to ask again then." (`view.md`, 1's full, its moments so said) |
    | the take busy, not ready or lost | "temper is busy; still trying…", while `jig-client` sends it again |
    | not asked: refused | "This temper doesn't sign terminals in from the browser. Paste the link it printed instead." |
    | not asked: busy | "Too many terminals are waiting to sign in. Press Enter to ask again in a moment." |
    | not asked: not ready or lost | "temper is starting or stopping, or out of reach. Press Enter to ask again." |

    An ended approval's code, address and expiry are no longer drawn:
    the outcome's line stands under the screen's first line.
  - *Signed in:* the TUI keeps the credential in its directory
    (`../draft/tui.md`, 4.1), as `temper-box open` keeps a link's (section
    4), before anything is told of the sign-in, and draws its first
    screen; the page says "Signed in. You can close this tab." (`pages/README.md`).
  - *Not asked while the TUI signs in from the engine's link* (`tui/pages/README.md`):
    the approval is asked when that ends with the TUI signed out, as the
    sign-in screen opens.
  - *The browser's way in beside the TUI's link:* a TUI signed in from
    the engine's link (`tui/pages/README.md`) spends the engine's local link, never the
    printed link (`tui/README.md`, 2; jig's `protocol/client/sign-in.md`, *The local
    link*), so the link the box's terminal shows stays live for the
    browser, before the TUI signs in or after it. That link is the
    browser's way in, and the approval page (`pages/README.md`) asks for the browser
    signed in first.
  - *Never shown:* the secret and the credential are never drawn on
    the screen or logged. A recording keeps them in the person's
    private file; `add` redacts them before any recording or print is
    written into the journeys (`tui/journeys.md`, 3.1 and section 5).

## 5. Open questions

### 5.1 The pages

### 5.2 A chat

### 5.5 Clients that are not a person's

- **Clients that are not a person's:** a command run once and a
  service acting as a party are the agents' tools, `temper ctl`
  (`ctl/README.md`). A bot: none.

### 5.8 Signing in

- **Approval between hosts and managing sign-ins** (4.1): a deployment
  at a name, Google on the web, the code typed there with the ask's origin
  and age shown, and the web's list of a person's sign-ins, each ended
  from there (`../draft/tui.md`, section 7). Default: loopback, approved
  by a browser signed in by the printed link, or by Google's way where it
  is given (`pages/README.md`, 2, *The approval page*).
- **A failed link beside a live cookie** (`view.md`, 1): the used, expired or
  unknown paragraph remains until another sign-in, even if the cookie
  still signs in on a reload. Falling back to the transport is jig's
  client question (jig's `client/domain/client/README.md`, section 15).
  Default: no fallback; reload to use the cookie.

- **The forge as people's sign-in provider:** temper's own routes on its
  clients' listener run the forge's OAuth flow, read who the person is,
  and ask jig's clients' component to mint a credential for that
  identity (`Mint`, with the forge's provider number, a provider of
  kind application in jig's `providers` member, the person's id there,
  their login and name; jig's `protocol/client/sign-in.md`).
  A browser then holds it as the default mapping's cookie; a terminal
  needs a flow of its own, such as a device code. Default: the
  deployment's link only.

### 5.10 The client's own

- **A moved delegate's own tasks, on a close's sheet** (3.6, *The sheet*): jig's
  impact counts every task below the chat, with no count per delegate, so "N tasks
  below it are cancelled." takes off the delegates listed and still counts a moved
  delegate's own tasks, which closing doesn't cancel. A count per delegate is a new
  fact of a protocol, the user's. Default: they stay in N.
- **What a page keeps across a restart:** where the client was, the
  composers' words and the form's values, as records of temper's own,
  which the terminal keeps beside `jig-client`'s small store, never in
  it (jig's `protocol/client/README.md`, section 7). Default: none; the client
  opens at the inbox.
