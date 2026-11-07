# temper's client

Provisional, 2026-10-14, revised 2026-11-07. temper's client: temper's own
presentation on jig's client, `jig-client`. Its crates; its client root,
which holds `jig-client` and temper's pages, the LLM accounts' and the
forges' among them; the forge's client family, its words, its one form
and its requests; its view, on
skein's UI kit; the terminal that `temper open` runs, on skein's text
shell; signing in and the small store; what temper's server does for it;
and how it is tested. jig's client is jig's `client/domain/client.md`, its
API jig's `client/domain/contract.md` and its protocol jig's
`protocol/client.md`: this document takes them as given and cites them.
The forge's parts of the client are `forge.md`, the browser `browser.md`,
and the agents' tools, `temper ctl`, `tools.md`. temper is the source
of truth for what it draws; the mockups (`mockups/`) design what it
does not draw yet: their look, their parts and their order, their
words and where each act leads. This document holds what each page
reads, watches and does, and cites them for the rest
(`mockups/README.md`, section 1). What is still open is listed in
section 11.

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
  (3.5); "How work runs" and a proposal of settings' review (5.13).
  The mockups design them: their look, parts, words and acts. What no page
  shows is open (section 11).
- **Its cards are its own.** A proposal, a question, a held task, a
  person task, a result, an entry of activity, an outcome and each
  confirmation are temper's nodes, drawn from the object's document as
  `jig-client` delivers it and from `jig-client`'s `facts` (jig's
  `client/domain/client.md`, section 19), as jig's
  `client/domain/objects.md`, section 2 allows an application. temper
  calls no renderer of jig's reference UI, and copies none of its
  modules, functions or phrase tables.
- **On jig's API alone.** What a page shows, it watches through jig's
  subjects (`me`, `inbox`, `tasks{query}`, `task`, `tree`, `run`,
  `accounts`) or
  reads (`activity`, `proposal`; jig's `client/domain/contract.md`,
  sections 6 and 7); what a person does is jig's operations (jig's
  `client/domain/contract.md`, section 3), through jig's helpers where
  they fit (3.6).
- **The confirmation is temper's.** An act that cannot be undone says
  what it will do before it is sent, from the facts jig's documents
  give, and every deciding act names the revision the person was shown
  (3.6).
- **LLM accounts, made from the client.** On the accounts pages,
  `Accounts` and its flow `ConnectModel`, a person signs in with an LLM
  provider, declares a program's login, or adds a key, pasted once and
  never kept (3.5, 3.6); while no account their work may use is usable,
  a banner on every page says so and offers the three (5.5). A chat's
  composer and a task's page name the account and model the work runs
  on. A browser opens a sign-in's address in a new tab while the page
  waits; the terminal prints the address and opens no browser (6.5).
  The accounts are jig's, kept by its engine
  (`../protocol/credentials.md`, section 3).
- **A first run's playground.** When the person chooses it, the
  engine makes the playground, a fake forge, the scripted model's
  account and the playground's project (`../protocol/setup.md`, 8.4),
  and the client reads it from the engine (4.8): a banner of one line
  on every page says it is the playground and offers "set up temper",
  which resets it and opens the welcome's Forge step, and the
  playground's `NewChat` suggests the requests its model
  plays (5.6). The printed link's sign-in in a browser lands on the
  welcome where the document shows it, and on the inbox otherwise
  (`browser.md`, 2.4).
- **The forge's family.** Its words for the connector, a repository, a
  branch, a pull request and each kind of write need no document (4.1).
  Its other parts are documents its client side reads, with requests of
  their own (`forge.md`); a part it does not read says "Shown on the
  forge." (4.2). Its form is the adoption, with a picker of the
  repositories temper can reach on the chosen forge and "Not listed"
  for the others, sent through `jig-client`'s request table as the
  forge's bytes, like any other keyed request (4.3 to 4.7).
- **Forges connected from the client.** On the forges' pages, `Forges`
  and its flow `ConnectForge`, a person whose role holds the
  deployment's connectors (in standalone, the owner) sets up or
  connects a forge, which the set up or the connect makes as a record
  the store keeps (`../protocol/forge.md`, 2.1 and 2.2), installs
  GitHub's App on another account, replaces a forge's key or tokens,
  forgets a managed forge's admin's token and removes a forge: the
  FORGE requests (section 8; `forge.md`, 5.4). A pasted value rides its request as jig's carried
  value, sent once and kept by neither the client nor the core (3.6).
  A connect's or an install's link is printed, never opened (6.5).
- **One crate of records.** `temper-client-protocol-wire` carries the
  forge family's records: the adoption, its outcome and the small
  store's record of a pending one (4.6), the FORGE requests and their
  outcomes (section 8), and those of `forge.md`, section 9. temper's
  server decodes them before the engine's root, which locates the
  repository and adopts it on the person's behalf, or runs the connect
  (section 8).
- **One view, every shell.** `temper-client-view` builds temper's pages
  as skein-ui's tree (skein's `ui.md`); the terminal prints that tree
  with skein's text shell (skein's `ui-text.md`); the browser, its
  second shell, maps it with skein's browser parts (`browser.md`).
- **A terminal first.** `temper open` runs temper's client, its view and
  skein's text shell over `jig-client-shell-native`, jig's native
  transport (section 6).
- **Signed in by jig's printed link,** on loopback, and kept signed in
  across restarts by its bearer credential, which the terminal keeps in
  the client's private directory (section 7). The forge as people's
  sign-in provider is open (section 11).
- **temper's server owns the clients' listener,** and mounts jig's
  clients' component on it with jig's default HTTP mapping, under
  `/jig`, with its example guards, and routes `/forges/...` for a
  GitHub connect's browser steps (section 8; `forge.md`, 9.1). The
  mapping's engine end
  owns its calls in flight: the server asks for its room before it
  hands it a call. An ordinary call under `/jig` that finds none waits
  for it, its body unread (`browser.md`, 2.2); once the server is
  stopping it answers that call itself, as it answers a path no one
  claims: `503`, with no body, closing the connection (jig's
  `protocol/client.md`, 17.4). A public read that finds none takes its
  own busy answer (17.5 there). The server's connections are its own
  bound, never the engine end's.
- **Joined to the engine from the start.** `tests/client`
  (`temper-client-world`) runs temper's client and view, with skein's
  fake person, against temper's engine root (section 10;
  `../domain/testing.md`, 6.3).

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
temper-client-shell-tui         the TUI's host, its seam, and its loop over the native transport and skein's full-screen terminal (6.13, 6.16)
└── skein-ui-screen-terminal    skein's full-screen terminal: the keys, the drawing, the terminal on io
temper                          the binary; `temper open` runs the terminal, `temper [ADDRESS]` the TUI (../protocol/README.md, section 4)
temper-client-shell-web         the browser: temper's shell, built for wasm32, its host ordinary Rust and a library on every target (browser.md, 2.1)
├── skein-ui-web                skein's browser parts: the page, the DOM, history, fetch, session storage
└── jig-client-shell-web        jig's client adapter over skein's fetch, and the printed link's sign-in part

for agents
temper-client-tools             the agents' tools on jig-client, beside the view (tools.md, section 4)
temper-client-shell-cli         `temper ctl`, over jig-client-shell-native (tools.md, section 8)

for tests
skein-fake-person               the person, on skein-ui's tree (../domain/testing.md, 4.4)
jig-client-test-kit             jig's check, and the faults of jig's client worlds (jig's `client/domain/testing.md`, 6.2 and section 7)
jig-client-fake-person-browser  jig's attack stories (jig's `protocol/browser.md`, section 15)
temper-client-world             tests/client: temper's client joined to temper's engine root (section 10)
skein-fake-person-browser       the person in Chromium, through skein-browser (browser.md, section 4)
temper-client-browser-world     tests/client-browser: the browser tier (browser.md, section 4)
temper-client-gallery           tools/gallery: the browser's pages in named states, as static documents (browser.md, section 7)
temper-client-journeys          tools/journeys: recorded walks replayed through the shipped host and printed as pages (journeys.md)
```

- **The crate graph.** `temper-client-domain` depends on skein-lib,
  `jig-client`, jig's records (`jig-client-protocol-wire`,
  `jig-client-protocol-link`) and `temper-client-protocol-wire`;
  `temper-client-view` on skein-lib, `skein-ui`, `jig-client` (for jig's
  facts and words) and `temper-client-domain`, and on no crate of jig's
  reference UI. Neither links io, HTTP, smith, or an engine crate of
  temper's: `jig-client` emits jig's messages, never HTTP, and turns
  reach it decoded (jig's `client/domain/client.md`).
- **`temper-client-protocol-wire`** is a codec crate generated by
  skein's generator from a schema, depending on skein-lib and
  skein-codec alone, no crate of jig's (jig's `protocol/client.md`,
  sections 3 and 11). temper's server and the client both use it.
- **`temper-client-shell-terminal`** holds, as modules:
  - `client`: the native client, `NativeClient`: temper's client root,
    its view and `jig-client-shell-native`'s transport, and when
    `jig-client-shell-native` keeps the credential's file (section 7);
  - `route`: each of the root's requests to the native shell, the host
    or one of temper's lines;
  - `terminal`: skein's text shell's terminal with temper's settings,
    its own lines and the sign-in's retries (6.2, 6.6, 6.7); and
    `Terminal`, the one host of skein's loop, composing the native
    client with stdin, stdout and the termination signals (6.2). The
    worlds put skein's fake person where `temper open` puts the
    terminal (section 10);
  - `limits`: the client's limits and their sum.
- **`temper-client-tui`** holds, as modules: `state`, the TUI's own
  state beside the client root (6.14); `event`, what its view makes of a
  person's acts; `words`, its own phrases; `view`, its pages as
  skein-ui's tree; `look`, its owner and the names of its regions and
  tokens; `theme`, its two themes (6.15); and `limits`, its tree's. It
  depends on `skein-ui`, `skein-ui-screen`, skein-lib, `jig-client`,
  `temper-client-domain` and `temper-client-view`, whose words it says
  where it says what the view says (6.14), and links no io.
- **`temper-client-shell-tui`** holds, as modules: `host`, the TUI's
  one seam (6.13); `run`, its loop (6.16); and `limits`, the client's
  from `temper-client-shell-terminal`'s with the TUI's view and screen
  limits beside them.
- **Step code under the subset:** the domain, the forge's side, the
  view, the TUI's state, view and theme, and the wire. **Ordinary
  Rust:** the terminal's host, the TUI's host and loop, and the
  browser's shell, as a shell is (skein's `ui.md`, section 2).
- **The client's limits** are constants of
  `temper-client-shell-terminal`: `jig-client`'s, from jig's small
  profile with the overrides temper's configuration gives the engine's
  documents (`../protocol/configuration.md`, section 7); the pages' and
  the forge side's; skein-ui's, derived from the view's (5.4); and their
  sum. The client reads no configuration: its only inputs are its two
  arguments (`../protocol/configuration.md`, section 9).

The native terminal, the browser and `temper ctl` receive sixteen entries
in a document's short lists, including creating's complete charter and
procedure lists, sixteen in an authority's grants and delegation kinds,
and eighty live delegates in a chat's close. This covers `temper init`'s
and the owned example's `standard` profile: its `documents.lines`
sixteen, its grants and executors sixteen, its `tasks.delegates` eighty
(`../protocol/configuration.md`, 13.4); a settings file and the
configuring agent's settings form take as many grants and kinds. Each is
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
routes nothing of jig's (jig's `client/domain/client.md`, section 4).

### 3.1 What it holds

- **`jig-client`,** given its limits and a seed at construction.
- **Two children, each an owner** (jig's `client/domain/root.md`,
  section 15): temper's pages, owner `Pages`, which ask the client for
  jig's operations, reads and subjects only; and the forge's client
  side, owner `Forge` (section 4); numbered in 4.6. Each is the asker
  of every handle and key it gives `jig-client`, and draws its keys'
  numbers from the seed the shell gives the root.
- **`me`,** watched from start to sign-out under a handle of its own:
  the signed-in party, their projects with their roles and pools, what
  waits for them counted, and each project's merged templates (jig's
  `client/domain/contract.md`, 5.1). Every page shows the link's state,
  `jig-client`'s `Link`.
- **The page at its address** (3.5): its handles, its windows, the
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
  (4.6), whatever page is open.

### 3.4 At start

The shell starts the domain with the small store's records and the
inbox's address, then signs in (section 7), holding back neither. The
domain's children ask their watches from their start, `me` among them,
and `jig-client` opens none on the engine until it is signed in (jig's
`client/domain/client.md`, section 12), so no stream leaves without the
credential.

- **Another party's kept work** stays parked and is not shown (jig's
  `client/domain/client.md`, section 11); the terminal says so in a line
  of its own (6.6).
- **Pending reads at sign-in** end visibly in jig's existing forms,
  on successful installation even before the first `me`, and on
  sign-out (jig's `client/domain/client.md`, sections 7 and 12).
  Whoever asked must ask again; no read is replayed automatically.
- **Kept work dropped:** the domain hands `jig-client`'s `Dropped {
  count }` to the shell (jig's `client/domain/root.md`, section 8), and
  the terminal says so in a line of its own (6.6).
- **temper's pages keep no record of their own:** a client closed on
  any page opens at the inbox, and a composer's words not sent, or a
  form's values, are not kept across a restart (section 11). A browser
  opens at its address (`browser.md`, 2.3), and after the printed
  link's sign-in at `Welcome` (`browser.md`, 2.4;
  `../draft/archive/first-run.md`, 2.3), which shows its first step while the
  store is empty, opens at the step left undone while it is
  unfinished, and moves on to the chats page once it is whole (3.5,
  The welcome). On a deployment with no welcome it moves on to the
  inbox, with nothing of the welcome drawn first (3.5, The welcome).
  The terminal, which opens at the inbox, reads the same fact for the
  pages it changes.

### 3.5 Its pages

Before its document arrives, a task page says “Reading task N…”, or one
sentence explaining a lasting refusal; after a transient read end it asks
again while open, waiting one second, doubling to at most thirty seconds.
A conversation's activity reads, a chat's and an agent's task's, follow
the same rule: a read that ends transiently (busy) is asked again from
the position it asked for while the page is open, on the same waits, so
a committed reply always arrives without a further delivery to prompt it
(jig's `client/domain/client.md`: a read is one attempt, and its caller
asks again).

No page claims empty until its read has answered. "Nothing needs you.",
"No chats yet.", "None.", "No updates." and "No goals yet." stand only on a
list whose read has answered with its value. A read that ends without one
(busy, unreachable, refused, past a bound) is no answer, and neither is a
watch that has delivered nothing, whether it is opening or Behind. While a
read hasn't answered, on first load or after the link comes back, a page
keeps what it last knew, rows and words alike, through every refresh; with
nothing known, a section draws its title with no line under it.

Each page has an address, and watches what it shows while it is open;
leaving it closes its handles. Its mockup designs what temper does not
draw yet: its look, its parts and their order, its words and where its
acts lead (`mockups/README.md`, section 1). Lists of tasks are
task queries (jig's `client/domain/contract.md`, 6.1): the chat and
goal templates' labels are jig's defaults, which temper's configuration
keeps (`../protocol/configuration.md`, section 2).

| Address | Mockup | Shows | Watches and reads | Acts |
|---|---|---|---|---|
| `Home { project }` | the greenfield mock's `home.html` | a project's first page: its intro, "To begin" while it shows, what needs the person, its chats (*Home,* below) | `me`; the `forges` watch; the person's dismissals (4.6); the project's chats' `tasks{query}`, as `Chats` reads them; `inbox` for that project, as `Inbox` reads it; `settings`, to say whether a way of working is configured | open `Configure`; adopt the first repository again, or open the adoption form; skip "To begin"; a question's choice, reply and Later, and a card's acts, in place |
| `Settings { project }` | the greenfield mock's `settings.html` | How work runs, the first tab of a project's settings: Configure temper, what the project shows, its ways of working, its status, what needs the person, while the person is away and telling them (*Settings,* below; 5.13) | `me`; the session's `settings` watch and the project's kinds 6 and 7 (5.13); the project's chats' `tasks{query}`, as `Chats` reads them | jig's `ChangeSettings` of one item (5.13) |
| `Repositories { project }` | the greenfield mock's `settings.html`, its tab, and `chats.html`'s band with no repository | the second tab: the project's repositories, or that it has none, and "Adopt a repository" (*Settings,* below) | `me`; the `forges` watch; the playground's read (4.8) | go to `Adopt { project }` |
| `Models { project }` | the look request `model-picker`'s `journeys/model-picker/07.html` | the fifth tab: what the picker offers, by provider, an owner turning a model on or off and adding one by name (*Settings,* below) | `me`; the accounts document; jig's settings, for an owner's act | none |
| `Builders { project }` | none: the greenfield mock's tabs say "not drawn in this experiment", so its How work runs tab's language | the sixth tab: the deployment's hosts, each with its slots and its connection, read-only (*Settings,* below) | `me`; jig's `deployment` document, for a person with a role in the deployment | none |
| `Configure { project }` | the greenfield mock's `chat.html?id=new&configure=1` | the configuring chat before its first words, under `Settings`, in the project's frame (*The configuring chat's page,* below) | `me`; the project's chats' `tasks{query}`, as `Chats` reads them | start the configuring chat, from its composer or a starter |
| `Inbox` (the root) | the greenfield mock's `needs.html`; the terminal's, `inbox.html` | in the browser, *Needs you* (below): what needs the person, one card at a time, across projects or filtered by project, kind and goal (its root), a proposal of settings in each project it shows in (jig's `domain/engine.md`, 17.6), its card, count and Accept as in its chat. In the terminal, the list: proposals, questions, held tasks, person tasks; then updates (results, replies); its filters; "+ New chat", a link to `NewChat` (`/chats/new`) | `inbox` with its filters; the `proposal` or `task` an entry's card needs | a question's choice, reply and Later (*What the client keeps,* below); accept ("Accept"), reject with a reason, release, leave held, take, hand back; in the terminal, answer in words |
| `Chats { filter }` | the greenfield mock's `chats.html` | a page of the frame (*The Chats page,* below): its head; while the project has no repository, its band; the list, under All, Live, Drafts or Closed, with "+ New chat"; "More closed chats" | `tasks{query}`: requester the person, the chat label, its live group by last activity, its ended pages by when each ended; the project's documents, as it opens (*A chat's document,* below); the person's facts (4.6) | choose the list's filter; more closed chats |
| `NewChat { draft }` | the greenfield mock's `chat.html?id=new`, and `&draft=` for a draft; the playground's start at the first run's first step | a new chat in the open project, on the layout every chat takes (*A new chat,* below), in the playground's project the playground's start (4.8, 5.6); with `draft`, the empty document beside it (*A chat's document,* below) | as `Chats`; the playground, from the forge's side (4.8) | start a chat, from the composer or a suggested request; "+ Draft" |
| `Chat { task }` | the greenfield mock's `chat.html` | the layout every chat takes (*One layout for every chat,* below): its bar, with its path, its heading, its number, its status, Pause or Resume, and Close, and its spend in the view alone; the context line, only in the playground's project while the playground runs (4.8, 5.6); the conversation (below): its request, the agent's words with its calls folded, the person's words, provisional text, and its end once it has ended; each task it delegated as that task's card, at the call that made it (a `delegate`, a `change` or a `goal`), or else at the result it sent back, or else after the conversation while it runs (5.7); the proposals waiting in its subtree (jig's `client/domain/objects.md`, section 10) and the decided proposals of settings its activity says it made, each at the call that made it where its turn names it, and otherwise after the turn that was running when it was made, so that what comes later comes after it (5.13); the composer, while the chat lives: the chooser's feedback above it, its text area "Message" with the placeholder "Reply to T6", T6 the chat's number, shown while it is empty and never its name, and "Send", the hint "Enter sends · Shift+Enter for a new line", or "Sending resumes this chat." on a chat the person paused (*A person's pause,* below), then a message's refusal or failure, if any: a message or the chat's start accepted draws no status of its own, neither at the conversation's end nor in the composer, since the conversation shows its words, "Being sent", then "Sent" (below); while Message is not offered, the feedback alone; an ended chat draws no composer | `task`, `activity` a page at a time, earlier and later pages at the conversation's log edges (5.15), `run` while one is live; each `proposal` its `task` names, or its activity's lines say it made; each delegated `task` its calls, results and live delegates name, up to the domain's `reads` (5.7); its document, read again when its run writes it (*A chat's document,* below); the person's facts (4.6) | message, pause, resume, close, change the next run's model, accept or reject a proposal in place; choose Chat, Split or Draft; "+ Draft"; start a goal from its document; on a configuring chat, start over |
| `Goals { project, filter }` | the greenfield mock's `goals.html` | the project's goals: its intro, its filters, a row a goal, each its state and its report's headline (*Goals,* below) | `me`; `tasks{query}`: the project, the goal template's label, tracked, by priority, its live group watched, its ended pages by when each ended; a row's report, one read at a time; `inbox`'s needs for that project | choose the filter; more goals |
| `Task { task }` | the greenfield mock's `goal.html`, for a goal; `task.html` otherwise | a goal: its header, its needs as bands, its report, its tasks, landing, how it runs and what it decided, what the person sent (5.8), with Amend, Pause or Resume, Close and its composer (5.11); a change: its header and "View on forge ↗", its steps, its step's line, its hold, its tasks live and ended, its gates at its head and its bounds (5.10); an agent's task: its lineage, its brief, its run's heading, its conversation as a chat's, its result once ended and its plan's held cards (5.9), with Amend, Pause or Resume, Close and its composer (5.11), and its own held card while it is held for any cause but the person's own pause | `task`, `tree`, `activity` a page at a time, earlier and later pages at an agent's conversation's log edges (5.15); `run` while one is live; a change's panel, through the forge's side (`forge.md`, 9.2); a goal's or a procedure's ended delegates, a page read once and again on a change; a goal's report, its own line and its needs (5.8) | as its document offers them (5.11): on a goal amend, pause, resume, close, message, and answer its needs; on an agent's task message, amend, pause, resume, close, release its own held task or one of its plan |
| `Changes { project }` | the greenfield mock's `changes.html` | what is on the way to the project's landing branches and what last landed there (*Changes,* below; `forge.md`, section 3) | the subject `changes`, through the forge's side (4.2, 4.6); a `task` per row shown, for where it came from | none: each row links to its change's `Task` |
| `Adopt { project }` | none: 4.4 | under its path (*Settings,* below), the adoption form, into `project` (4.4), with the picker of the repositories temper can reach on the chosen forge | `me`; the chosen forge's deployment part, for its kind, mode, user and installations (`forge.md`, 5.4); the picker's page, a `Reachable` request (section 8) | adopt a repository (4.4) |
| `Forges` | `forges.html` | inside Settings' frame while a project is open (*Settings,* below): the person's forges, each row folded until opened: its connection, its mode and access, on GitHub temper's App and its installations, a connect under way and how the last ended, its load; then the playground's forge, folded, while the playground is read (4.8); "+ Connect a forge", to `ConnectForge`, but while the playground runs (`forge.md`, 5.4) | the forge's, through the forge's side (4.2) | connect a stopped forge; replace a key or tokens; install on another account; new repository; forget the admin's token; set up again; remove a forge, confirmed (section 8) |
| `ConnectForge { forge }` | `connect-forge.html` | under its path (*Settings,* below), connecting one more forge, `forge` the choice it is at: the shared choice cards or a way’s panel, then the open row on Forges, adopting nothing (`forge.md`, 5.4) | the forge's, through the forge's side (4.2) | connect; connect as its admin; set up a Forgejo here (section 8) |
| `Welcome { step }` | `welcome.html`; the first run's step 0 | the first run's welcome, by the store's state, empty, unfinished or whole (below): "Try the playground" or "Set up temper", the latter a wizard of Welcome, Forge, Model and Ready, its `step` (5.6; `../draft/archive/first-run.md`, 2.3) | the playground's read (4.8); `accounts`; the `forges` watch, for a person whose role holds the deployment's forges; the forge's, as `ConnectForge` reads it | start the playground; start a forge cut short over, or go on with it; `ConnectForge`'s and `ConnectModel`'s acts |
| `Accounts` | `accounts.html` | inside Settings' frame while a project is open (*Settings,* below), and there "Sign out" at its foot, in a browser at a phone's width alone: the person's LLM accounts, then, for an owner, the deployment's, each row folded until opened: its LLM provider and display name, its holder, its deciding source's state and its line (5.5, An account's line); opened, that state's words and acts, its trial's line under the head and before the sources (`../domain/trial.md`, 6.2), its sources in order, each with its state in words and, for a program's, on which hosts (5.5), what it pays for, the runs using it and the tasks waiting for it (facts jig gives the deployment's accounts alone for now, so a person's own leave them out: 5.5) and its acts; the checks under way, and how the last ended; then the playground's account, folded, while the playground is read (4.8); "+ Connect a model", to `ConnectModel`, but while the playground is read | `accounts` (jig's `client/domain/contract.md`, 7.1); `me`'s LLM providers | sign in again or add a key from a state's acts, as `ConnectModel`'s ways; sign a source out; add a key to an account; replace a key; remove a source or an account; check a program's login again; move an account, or a source, up or down; resume a limited source |
| `ConnectModel { way }` | `connect-model.html` | under its path (*Settings,* below), connecting one more LLM account, `way` the way it is at, or none at the choices: the welcome’s cards (5.5, The ways), then a sign-in’s wait or a key’s or program’s panel; success returns to Accounts with a confirmation line, and other endings keep their words and acts (5.5, A sign-in) | `accounts`; `me`'s LLM providers | sign in with an LLM provider; add a key as an account of its own; declare a program's login |
| `Proposal { proposal }`, later (5.13) | none: the user's (5.13) | a proposal of settings: temper enforces and your agents follow, its effect on running work, the settings folded; withdrawn when stale; accepted (5.13) | the `proposal`; `settings`; each item's setting before | accept, choosing whether running work moves; reject with a reason; change something, in the chat |

- **Words and facts:** a chat's status from jig's facts (jig's
  `client/domain/client.md`, section 19), in jig's words, with temper's
  own in places ("Waiting for you", "Parked"); a conversation from
  jig's thread model; jig's enums and a decision's facts in jig's
  words, under temper's phrasebook (5.1);
  each card temper's own (section 1).
- **One layout for every chat** (the greenfield mock's `chat.html`): every
  chat and `NewChat` take the layout of the configuring chat's page
  (`temper-session`; *The configuring chat's page,* below). The rail stays
  on every chat: the user's choice in `configure-chat-in-settings`, which
  supersedes the mock's rail-less chat. On a phone, at 760 px or less, rail
  and tab bar go. A chat differs from a configuring chat only in its path: a
  chat's path is "PROJECT / Chats /", "‹" on a phone a
  link to `Chats`; a configuring chat's is still through `Settings`, the
  rail current on Settings.
- **A chat's bar** holds its path, its level-1 heading named "TN · TITLE", the
  chat's task number and its title (a chat's first message, whole), and at its
  right, in this order:
  1. *Its number:* "T4", the same pixels as the mock's `.byline`. The browser
     alone draws it, as the terminal's heading says it; on a phone it goes.
     The model and effort are the composer's, and the account leaves the bar
     (*The work's account and model,* below). A configuring chat's is a chat's.
  2. *Its status,* quiet: 12 px, faint, in lower case by the look alone.
     It is the colour of `.state-needs` while it waits for the person.
     A run that can't start says so ("Waiting for an account"), that colour
     too, and has no dots: the dots say a run is working, never that it
     can't start (jig's `client/domain/client.md`, 19.1).
     Every ended chat says "Closed", with no time: its end's line has the time
     (3.6, *Closed ends on one line*).
     A chat the person paused says "Paused", quiet; one held for any other
     reason says "Held", and its card says why (*A person's pause,* below).
  3. *Pause or Resume, and Close,* while allowed, each quiet and small. The
     mock has neither; they are the only way to pause or end a chat.
  4. *Its own spend,* "N spent · of B", in the view and hidden in the
     browser.

  The bar's right part is a flex row.
  After the number, once a chat has a document, its mode group; with none,
  "+ Draft"; then "Start goal…" or its goal's link; on a configuring chat
  "Start over" (*A chat's document,* below).
- **A chat's heading is short** (the request `chat-title`): it shows its
  title's short form, which a browser draws alone and the terminal after
  "TN · ". The short form is worked out in the view, in one function
  (`pages/chat.rs`):
  - *its words:* the title from its first character that isn't white space
    (a space, a tab, a carriage return or a line feed), up to its first line
    feed or through its first sentence's end, whichever comes first, with the
    white space at its end dropped. A sentence ends at a `.`, `!` or `?`
    followed by white space or by the title's end, so a `.` inside a word
    ("3.5", "lib.rs") ends none;
  - *its length:* at most 72 characters, counted as Unicode scalar values.
    Past that, it is cut on a word: the words before the last space or tab
    within its first 72 characters, with any `,`, `;`, `:` and white space at
    their end dropped, then "…", so that the heading is at most 72 characters.
    Where that leaves no words, a first word of more than 71 characters, it is
    cut within it after its 71st character;
  - *the whole title* stays in the heading's name, "TN · TITLE", and where
    the short form isn't the whole title it is the heading's hint as well,
    which a browser shows on hover (`skein/docs/design/ui.md`, 3.2); a title
    shown whole has no hint. The person's message, the task card and the stored title keep it
    whole. Only a chat's heading is short: an agent's page keeps "TN ·
    TITLE" whole.

  The path yields to the right part: the crumbs take `flex: 1 1 320px`, so
  the right part keeps the bar's row while the path has 320 px, and the
  heading's ellipsis takes up the rest. Below that the row wraps, and a
  phone keeps the heading above the controls.
- **A person's pause** is a chat held by a placed hold of that person's own,
  mode Stop, over the chat's own subtree: the chip's hold names it, not lifted,
  its owner the party reading (jig's `client/domain/contract.md`, 5.6). Nothing
  else is a pause: a hold of another's, of a task above it, or with a reason of
  its own says "Held", with its card and why, and has no "Resume": the card
  keeps its acts.
  - "Pause" is `Ask::Stop`, while Stop is allowed and the chat isn't paused.
    "Resume" takes its place on a paused chat where Release is allowed, and is
    `Ask::Release` naming the hold's revision. Each is sent at once (3.6). The
    status says it happened: an accepted Pause or Resume draws no status under
    the conversation; a refusal says why there.
  - **Sending resumes.** A message to a paused chat, where Release is allowed,
    is one act: `Ask::Compound` of its `Message`, then a `Release` naming the
    hold's revision (jig's `client/domain/contract.md`, 3.4; jig's
    `domain/tasks.md`, section 7: no message releases a task, and words to a
    task its party stopped come with a release). The composer's hint says so
    while it is paused, "Sending resumes this chat.", in place of its usual
    words. It commits whole or not at all: a refusal, the hold's revision moved
    or the words refused, leaves the chat paused and nothing in its inbox, and
    the composer says why as for a lone message. Any other chat's message is a
    lone `Message`.
  - An agent's task's Pause and Resume are these acts (5.11); a message to it
    resumes nothing (section 11).
- **A conversation's turns** are named "You" when the turn is the
  person's own, "Temper" when it is the agent's, and by the party's
  name otherwise (`chat.html`), on every page that draws a
  conversation. A chip's executor keeps jig's "Agent". On a chat's
  page each turn's time follows its name, "HH:MM", under the day's
  rule (5.12): a line's time for the agent's words and the person's,
  and for the request the conversation's first line's; words not yet
  read by the engine have none.
- **A chat's status,** in its bar and on its line of the Chats
  page, is jig's, narrowed as the mockups' "Where each value comes
  from" says: "waiting on 1 delegate", "waiting on N delegates" while
  it waits on its delegates, N its chip's live delegates, as a goal's
  header says it (5.8). On the chat page, the delegated tasks' documents
  reconcile that count while deliveries catch up: a card still running
  counts until its final document arrives, and an ended card no longer
  counts even if the chat's document still lists it. It says "waiting on
  your words" while it waits on news and its wake policy wakes it on
  words (its document's `wake`), from
  the person signed in, who requested it; jig's word otherwise. While
  a question a delegate asked it waits in the person's inbox, jig's wait
  is on the person, before its delegates, and it says "Waiting for
  you". A line of the chats page reads the same wake policy, from its
  chip's (`OwnerChipDetails`' `wake`), and says "waiting on your words"
  as the bar does: a chat at rest after its reply, which only its
  person's words or Close move on (jig's `domain/tasks.md`, 5.8). A
  chat whose host refuses its start says jig's "Cannot start yet", and
  the feedback above its composer says why (below); a line of the Chats page holds
  no run wait, so it is never refused, and says jig's word for its
  stage (jig's `client/domain/client.md`, 19.1). A chat held on a
  configured bound (jig's `Exceeded` hold) says jig's phrase for that
  hold, as its hold card does: "Held: the model's reasoning went past
  its configured allowance of 2048 bytes". A chat the person paused says
  "Paused", on its bar and on its line on Chats and on Home (*A person's
  pause,* below). Any other hold says jig's "Held", and the chat's page says why.
- **A chat's conversation** is the chat's own, drawn from jig's thread
  (jig's `client/domain/objects.md`, section 10), never its runs'
  mechanics (`chat.html`, the mockups' first run):
  - *its request,* first, under "You" (or the requester's name): the
    chat's specification's words (its document's `spec`), a paragraph
    for each of its lines, where the window holds the conversation's
    start, its first line with no older page; a chat whose spec is
    empty has no such turn, and opens on its first words line;
  - *the agent's words,* each assistant message's text under "Temper",
    whole within the receiving bounds (5.4), read as Markdown by
    `skein-ui`'s reader (skein's `ui.md`, section 10): paragraphs, lists,
    quotes, code, and tables at the top level. A browser draws a table
    as the mockups' (`md-table`), and leaves out a header row whose
    cells are all empty, as they do. A provider's opaque block, a turn's
    reasoning, is not drawn, and nothing stands in its place. A
    conversation says nothing of what was cut: no notice, at a text's end,
    a turn's or a window's;
  - *the words being written,* the turn being written, under "Temper",
    whole from its start up to `long_text` (5.4), a page opened late
    from what the engine retains (section 11), read by `skein-ui`'s
    `read_growing` (skein's `ui.md`, section 10): a block a blank line
    has closed stays on the page as it was, only the last changes as
    words arrive, and no marker shows being written. A small block,
    "▍", in `temper-writing-end`, stands after the last word. The
    committed turn draws the same blocks in the same place, and only
    the block goes. Before its first word the turn draws nothing, and
    the three dots stand (below);
  - *its calls,* where they fall among the agent's texts: each
    delegation as its task's card (5.7), a document's write as its tool
    line (*A chat's document*, *The tool line*), and every other call in *a run*: every
    call between two of the agent's texts, across the turns that made
    them, drawn once as one line. A text, a person's words, a card, a
    document's write or any other drawn line ends a run; a line the chat
    hides does not. A call that a card stands for is neither counted,
    named nor listed. The line is one disclosure (skein's `ui.md`, its
    disclosure), `temper-work-run`, in a turn of its own:
    - *its name,* one phrase a tool, in the order first called, joined
      by ", ", the first capitalised unless it is a tool's own name:
      "Read 7 docs, checked settings 3×", "read_file ×2". The phrases are
      the view's table, one function (`pages/chat_line.rs`), tool by tool,
      starting from the tools a chat calls most. A tool in it counts what
      it touched, not its calls: `read_docs` is "read N docs", its distinct
      topics, each search one, a call whose input can't be read one
      ("read 1 doc"); `settings` is "checked settings N×", once "checked
      settings"; `propose_settings` is "proposed a change N×", once
      "proposed a change", and under way "proposing a change", none of them
      naming its input; `read` and `Read` are "read N files", `list` and `Glob`
      "listed N folders", `write` and `Write` "wrote N files", `edit` and
      `Edit` "edited N files" and `remove` "removed N files", each its
      distinct arguments, a call whose input can't be read one; `search`
      and `Grep` are "searched the files N×", once "searched the files";
      `shell` and `Bash` are "ran N commands", each call one. Any other
      tool is its name, counted: "read_file ×2", once "read_file". A run of
      one call alone says what it touched: "Read the page hold", "Searched
      the docs for WORDS", "Checked template 2", "Read src/main.rs",
      "Searched for WORDS", "Ran cargo test", its counted phrase where the
      input can't be read ("Read 1 file"); any other tool still its name;
    - *its description,* faint: the time the run took, then nothing else.
      "29 s", under a minute in whole seconds, then "4 min": from the
      line before the run's first turn to the run's last turn. A committed
      turn says only when it ended, so the line before it says when it
      began: any line of the task's activity, drawn or not (a turn, the
      person's words, the run's beginning). A run with no such line, at the
      window's head or the second run of one turn, measures from its first
      turn's end. Under a second has no time and no description;
    - *a failed call* adds "1 failed" ("N failed") before the time in the
      description, "1 failed · 29 s", the description then in the signal's
      colour, `temper-work-failed`, and read after the name with a comma
      the stylesheet draws;
    - *while it runs:* the conversation's last run, while the chat's
      status is active (`chats::working`) or an agent's run is out (5.9),
      has `temper-work-running` and no time. It counts what has ended, and
      its description is the call under way, from the run's `call` (jig's
      `RunState`), the phrase's present form in faint words: "checking
      template 2", "reading index", "reading src/main.rs", "searching for
      WORDS", "running cargo test"; any other tool, and a call whose input
      can't be read, its name. The stylesheet adds the "…", and the
      terminal, which has none, writes it. With no call under way, no
      description. A call under way with none ended is the line alone, its
      name the present form capitalised;
    - *opened,* `temper-work-calls`, a row a call in order, the call under
      way last: a paragraph named "TOOL ARGUMENT: OUTCOME", the tool in
      code, the argument in a few words (`read_docs`: "index", "change,
      from 1438", a search's words; `settings`: "the inventory", "charter
      1", "history"; any other tool: its input's first text or number
      member, "task 3", a text's first line; a live call's input cut at its
      bound as far as its first member closes; nothing where the input
      can't be read), and, in
      `temper-work-out`, its outcome's first line, a failed call's
      "failed: LINE" in the signal's colour. Never a result's body. A row
      is cut at 160 bytes, and a name at 256 at a phrase's end, with "…".
    A browser shows it folded until the person opens it. The terminal
    prints the name, then its description after a colon, and its rows
    under it, as text; the fake person finds them whole;
  - *the person's words,* the thread's words and each words line a
    person sent (jig's `OwnerActivity::Words`, by a person), under
    "You" or the party's name, a paragraph for each line, with "Being
    sent" or "Sent" until the engine has it, and no word once read;
    an answer the person gave in the inbox to a question a delegate
    asked is not among them: the answer is the asker's, in its
    activity (`../domain/agent.md`, 8.2), and the conversation shows
    the notice the chat heard as one line under "You", "You answered
    question N in your inbox". The notice carries the answer words cut
    at 256 bytes on a UTF-8 boundary and marked when cut
    (`../domain/agent.md`, section 5);
  - *a delegate's words,* each words line a task sent the chat (jig's
    `OwnerActivity::Words`, by a task), a delegate's question among
    them, under the task's name, "TN", on the agent's side, a paragraph
    for each line;
  - *not drawn:* a run's beginning and end, the task's milestones, a
    turn's opaque blocks (a model's reasoning, with nothing in their
    place), and every message a turn's person, tool or system speaks, which are
    the run's inputs: its opening, its calls' results, and what woke
    it, a result delivered among them. A result block still anchors
    its task's card where the call that made it is not in the window
    (5.7), drawn on the agent's side, never under "You";
  - *its end,* once the chat has ended, after the conversation: one
    line, "Closed at" and the time it ended ("Closed at 21:05", 5.12's
    clock), the status's word (3.5's chats), then, for a report,
    " · its report, “REPORT”", the
    report's words whole; a failed chat's line is "Failed", its time
    and its reason ("Failed 21:05 · REASON"); a chat cancelled by the
    party that requested it was closed, and its line is "Closed at" and its
    time ("Closed at 21:05"), with no reason; one cancelled by anyone else
    is "Cancelled", its time and its reason ("Cancelled 21:05 ·
    REASON"). Who closed it is the ending's `by` against the chat's
    requester, both in its document, never the reason's words. The
    result is not drawn again.

  Its look is the greenfield mock's `.thread` and `.thread-inner`: a column
  720 px wide and centred, gap 20, whose scroll stays at the end while the
  person was at the end. The conversation is still a log.
  - *The person's and the agent's turns* are `.turn-you` and `.turn-agent`,
    the agent's Markdown with its first and last margins zero. A turn's
    name and clock are hidden from sight and stay for readers and the
    terminal, but "Being sent" and "Sent": 12 px, faint, right aligned under
    the bubble. The day's line is hidden; a marker is `.meta-line`.
  - *A delegate's words* look like `.turn-agent`, with "TN" kept visible,
    12 px and faint, above them.
  - *A call's line,* `.turn-tool`, opens with the document's icon where it
    is a run's and the pen where it is a write's. A run's line is muted, its
    description faint, with a "›" that turns when it opens; open, its rows
    are indented 24 px, a row wraps at a phone's width, the tool 12 px mono,
    the argument muted, the outcome faint. A delegated change and a goal are
    one line each (5.7).
  - *While it works:* three dots, `.typing`, at the end of the log, named
    "Writing", while the chat's status is Starting, Writing, Working or a
    tool's, the window's last line is not provisional text holding a word, and no run is
    running at the conversation's end. The terminal
    prints nothing new for it. *The words being written* look like
    `.turn-agent`, their name hidden as a committed turn's, their "▍"
    faint and in the line of the last word; the terminal prints the "▍"
    as it prints a status. A numbered list's `ol` is 22 px in, its
    markers faint.
  - *A card in the conversation* (a proposal, a held task, a question's
    answer) is `.proposal`: white, radius 12, its shadow, 20 px 22 px; a
    kind's title in `.proposal-title`; `.proposal-split` and
    `.proposal-state`; its first button `.btn.primary.small`, the rest
    `.btn.quiet.small`.
  - *Reading back:* "Latest", `temper-latest`, a browser's alone: a small
    pill, centred 12 px over the pane's foot, its " ↓" the stylesheet's, on
    a button *to log end* (skein's `ui.md`, 3.2) beside the log, shown while
    the log is away from its end (`data-log-away`, skein's `ui-web.md`, 7).
    At a phone's width it stands in a row of its own below the log instead,
    so nothing lies beneath it and it covers none of a card's controls.
    Pressed, the view returns to the end and it goes. Where the window has
    let the newest lines go the log continues and the pill stays visible,
    even at the window's foot. A press on a chat reads the end again;
    on an agent's page it reads the next later page (5.15).
    Sending on a chat returns to the end too. New words arriving while
    reading back don't move the view.
  - *"Loading earlier messages…",* `temper-thread-loading`, faint and
    centred at the head of a chat's conversation or an agent's, a status
    named "Loading earlier messages", only while an older read is out.
    A failed read says "Couldn't load earlier messages" there, in the same
    look; the next move toward the head tries again (5.15). Nothing once
    the chat's start is shown.
  - *Earlier lines* load as the log's head nears the view (5.15), with no
    "Older" button, on a chat's page and an agent's. *Its end* has
    `.meta-line`'s look, and an ended chat has no composer.
- **A new chat** (the greenfield mock's `chat.html?id=new`): `NewChat`,
  at `/chats/new`, for the open project. The domain's `Chats` state holds
  its words and start, and `NewChat` opens that state with the page's
  watches, so that `Act::StartChat`, `Act::ChatWords` and `Act::Suggest`
  are the page's, as they are the configuring chat's. On the layout every
  chat takes:
  - *Its bar:* the path "PROJECT / Chats /", "‹" on a phone a link to
    `Chats`, and the level-1 heading "New chat". No project yet, in the
    playground's start, the path is "Chats /" alone.
  - *In the thread,* centred, `.meta-line`: "Say what you want; the agent
    works in OWNER/NAME.", OWNER/NAME the project's home repository, read as
    Home reads it; or, with none, "Say what you want. PROJECT has no
    repository yet, so the agent can talk and plan but not change code."
  - *The composer,* the session's: its name and placeholder "What do you
    want done?", focused as the page opens, the hint "Enter sends ·
    Shift+Enter for a new line". There is no "Start" button: the send arrow
    is the act, grey while the words are empty or the composer cannot send.
    Sent, while it is out, the arrow is disabled and the words stay; made,
    `Chat { task }` opens in place of this address; refused, jig's words
    stay under the composer, the words kept.
  - *With no usable model,* "No model yet · Connect a model" sits above the
    composer (5.5).
  - *The playground's start* (4.8, 5.6): its status, its failure and "Try
    again" and its context line draw above the `.meta-line`'s place, and its
    suggestions as `.starters` under it.
  - *With a draft,* `NewChat { draft: true }`, `/chats/new?draft=1`: "+ Draft" in
    the bar (`.btn.small`, while `drafts_offered`, *A chat's document,* below)
    moves there. The empty document in Split, `.meta-line` "A chat that writes a
    draft. Say what it should hold.", the placeholder "What should this draft
    hold?". The first message starts the chat with, under the person's words
    after a blank line, "Write this up as a draft beside us."; those words are
    visible as sent. The chat opens in Split.
- **Where the focus stands on a chat** (the request `ui-composer-focus` of
  2026-11-06):
  - *Started by sending:* a chat a person has just started opens with its
    composer focused, as soon as its composer shows (skein's `ui-web.md`, 7):
    from a new chat, its draft, `Configure`, a starter and a suggested request
    alike. "Start goal" is none of these: it opens the goal's page, its heading
    focused. A start refused stays on its page and moves nothing.
  - *Opened any other way,* its composer asks for nothing. From a list or a
    link, a new entry, the chat opens on its heading, so that a person reading
    with a screen reader or the keyboard lands on what the page is, though the
    heading is drawn after the page (`browser.md`, 2.3). An address, a reload,
    Back and Forward make no new entry, and take the focus to no heading
    (skein's `ui-web.md`, 7).
  - *After a send,* by Enter or by "Send", whether or not it went out, the focus
    is in the composer, on a chat's, a goal's and an agent's task's (5.11) alike:
    "Send" disables as the words clear, and a disabled button holds no focus.
    "Pause", "Resume" and "+ Draft" send too, and ask for nothing.
  - *While the rename's form is open* it keeps the focus (5.3), and a composer
    that has asked for it takes it back as the form closes.
  - *On a phone* the same acts ask the same, as a phone's messaging app does:
    the keyboard stays up after a send, and a chat opened from a list raises
    none (section 11). The terminal draws no focus.
- **A chat's document** (the greenfield mock's `chat.html`; the look request
  `configure-chat-draft`, for every chat): a chat keeps one Markdown document
  beside its conversation, which its agent writes (`../domain/agent.md`, 8.3).
  It is the lowest-numbered note in the chat's own scope whose description
  does not begin `work:`; working notes are never drawn, and a chat with only
  working notes has no document. Its description is its title, its body Markdown,
  its name the mock's `slug` of the title, 28 characters, plus `.md`:
  `away-mode.md`. There are no file drafts: a document
  is always a chat's.
  - *Offered:* `pages::drafts_offered(root, project)` is that project's kind 7
    `drafts` (5.13), true in the stock. It gates the Drafts filter, the Chats
    lede's words, "Start goal…", the rail's "Goal" and "+ Draft" on a chat that is
    no configuring chat. A configuring chat offers "+ Draft" whatever `drafts` says:
    `pages::chat_offers_draft(root, task)`, which the bar and the domain's taking of
    the act both read, is true on a configuring chat and otherwise `drafts_offered`.
  - *Read live, with no new watch:* `Read::Notes { scope: NoteScope::Task {
    project, task }, description: None, after: None }`, on the same retries as an
    activity read, once the task's document has arrived, and again when the
    thread shows a `note` call not seen before and when the run's state moves off
    a call to `note`. One read is out at a time; a read wanted while one is out
    is asked once more on its answer. The page watches the task and its run, and
    only the task's own run writes the note, so nothing else moves it. The newest
    answer replaces the document, and a read refused or empty leaves it as it
    was.
  - *Its mode:* `Mode { Chat, Split, Draft }`, the page's state for that chat, kept
    while the page is open; a chat opened again starts in Chat, unless `NewChat {
    draft }` opened it in Split. Nothing stores it; the terminal's look, which is
    another thing, is kept per browser (*A chat's terminal*, below).
  - *The tool line:* a `note` call whose answer parses (`../domain/agent.md`,
    8.3) is a `.turn-tool` in the conversation: the pen, "Wrote" for a first
    write, else "Edited"; `.file` NAME; a `.turn-tool` detail, "64 lines" or "+18
    −6"; "Show", a button. It isn't counted in a run, and ends one; a failed one is a call of its
    run. The domain parses the words, as
    `delegation::results` does. "Show" moves to Split (on a phone, Draft) unless
    already in Split or Draft, and moves the focus to the section holding that
    call's `first`, which brings it into view and flashes it (`.doc-sec.flash`,
    `@keyframes flash`).
  - *The bar,* after the number, once a document exists: `.seg`, a group named "The
    draft", "Chat", "Split" and "Draft", each its icon (`chat.svg`, `split.svg`,
    `doc.svg`), the chosen `aria-pressed`. `Ctrl+\`, skein's primary key (⌘ on a Mac), steps through
    those shown, the next mode's button carrying the shortcut. On
    a phone, Chat and Draft alone; Split left on a wide screen shows as Draft. With
    no document, while `chat_offers_draft` and the chat hasn't ended: `.btn.small`
    "+ Draft", which follows the first message: `Configure`, before a configuring
    chat's first message, has nothing to write up and draws none. `.btn.primary.small` "Start goal…" once the document has a section,
    on a chat of a project with a home repository with no goal linked; with a
    linked goal, a link `.btn.small` "Goal TITLE →" to `Task { task }`. Not on a
    configuring chat, which has "Start over": it asks "Start over?", "A new
    configuring chat starts. This one stays in Chats." through `crate::confirm`
    (3.6, *The sheet*), closes the chat with
    the chat's own close, which stays in Chats, Closed, its conversation and
    document kept, and opens `Configure { project }`. The composer's hint gains
    " · Ctrl+\ switches the draft" while a document shows.
  - *The layout,* one for both kinds of chat, from `configure-chat-draft`'s rules:
    `temper-mode-chat`, `-split` and `-draft` on the session. Chat: the
    conversation alone. Split: the conversation and the document in two halves, the
    document on white behind a rule, the rail folded to a 56-pixel strip holding the
    star, a link to `Home`, and "Show the rail", drawn as `files/menu.svg`, which
    opens the rail over the page until a press outside it or Escape. Draft: the
    document alone, the rail back, the composer kept at the foot on white. The pane
    keeps a scrollbar's gutter on both edges, so that its column and the composer
    share a centre. On a phone the document takes the screen's width. In the
    terminal, in Chat, the conversation alone; in Split or Draft, the document after
    it as a region named its title.
  - *The pane,* `.temper-doc`, `.temper-doc-inner`: a column of at most 640 pixels.
    Its line, `.temper-doc-path`: at its left `docs/design/draft/NAME.md`, or on a
    configuring chat "T7 · not proposed yet", "Proposal 12 · waits for an owner" or
    "Accepted at 14:03", from its proposal cards; at its right "edited 1 min ago",
    `TimeForm::Ago`, lower case. The body is split at each `#` and `##` line, each
    section read by `skein_ui::markdown::read(.., 1, b)` into a slot, `.temper-doc-sec`, keyed
    by its heading and its place among its namesakes, so that a change keeps the others'
    nodes. `#`, level 2, is drawn as the mock's `.doc-inner h1`, and `##`, level 3, as its `h2`; the first section's first
    paragraph as `.doc-muted`. Tables as `configure-chat-draft`'s. A fenced block
    reads as code, but for a flow (`skein/docs/design/ui.md`, 10.1), which is drawn.
    temper's page routes skein-ui's owner to its drawing code and gives its names
    beside temper's, so a flow draws in any Markdown a person reads here, a turn's as
    a document's. Its look is `browser.md`, 2.5's.
  - *The tint,* `.temper-doc-changed`: the section holding the newest `note` call's
    `first`, or for a first write the first `##` section; under it `.temper-doc-note`
    "Added by T4 3 min ago" or "Changed by T4 just now".
  - *Empty,* "+ Draft" pressed and nothing written: `.doc-empty` "The draft appears
    here as you write it together.", the path's left the mock's `untitled.md`, its
    right "writing".
  - *"+ Draft" on a chat:* goes to Split with the empty pane, and sends the chat
    the message "Write this up as a draft beside us." The `.meta-line`, when the
    thread is empty: "A chat that writes NAME. Say what it should hold."
- **Starting a goal from a chat's document** (the greenfield mock's `startSheet` and
  `startGoal`): "Start goal…" opens the one sheet to start a goal (5.14), given
  `docs/design/draft/NAME.md`, the chat and a snapshot of the document's body; its
  "Name" is the slug without `.md`, and its ways and default are 5.14's.
  - *Sent:* its words, within `long_text` on the client's side and `tasks.spec_bytes`
    on the engine's, refused in place if over, are "NAME", a blank line, "Implement
    this draft. First commit it, as it stands, at docs/design/draft/SLUG.md.", a blank
    line, the body, a blank line and the line `goal`'s root gives a goal, so that
    charter 1's instructions read it as a goal (`../domain/agent.md`, 4.3.2), a
    constant the root and the client reach, in `temper-client-protocol-wire`; then
    5.14's `Draft:` and `Chat:` lines, and `Asks: always` where it is chosen.
  - *Answered:* 5.14's ends; on `Judged::Create` the fact `GoalFrom` is sent (4.6)
    and the goal's `Task { task }` opens.
  - *After:* the fact's chat shows `.turn-tool`: pen, "Started goal", `.file` TITLE, its
    state, "Open", placed among the turns by the fact's `at`. Goals an
    agent started with the `goal` tool keep their line in the conversation (5.7).
- **A chat's terminal** (the look requests `chat-terminal`, `chat-terminal-font`,
  `chat-terminal-draft` and `chat-terminal-esc` of 2026-11-07, their pages
  `journeys/chat-terminal/`): a browser's other look of a chat's pages, `Chat`,
  `NewChat` and `Configure`, as a coding agent's terminal, full screen. It is the
  same page, its state, reads and acts, drawn otherwise: its commands are the
  page's own acts, and nothing new crosses to the engine but the reads named
  below, which other pages make already. The terminal shell (section 6) is
  another thing: it prints these pages as it always has.
  - *Its look, kept per browser:* `pages::terminal::Look { terminal, theme, font }`
    on `Pages`, the same on every chat the browser opens, so that another chat,
    `/chats`, `/new` and Start over open in it. `theme` is `Dark` or `Light`,
    `font` `DejaVu` or `Plex`; a browser with no record of its own starts
    `{ terminal: false, theme: Dark, font: DejaVu }`. The shell reads it at start
    from the kept store's record `look` (skein's `ui-web.md`, 10), three bytes,
    one for each field, in that order, `0` or `1`, `1` the terminal, `Light` and
    `Plex`; a record of any other form is none. A change writes the record, in
    the step that made it, through a host duty, and a write refused changes
    nothing on the page. Another tab reads it at its own start.
  - *Entered:* the button ">_", named "Terminal, full screen", in a
    browser's bar after the mode group or "+ Draft" and before "Start goal…" or
    "Start over", on every chat, `NewChat` and `Configure`, on a phone too:
    `terminal::Act::Enter`, which sets `terminal` and asks the shell for full
    screen in the same step (skein's `ui-web.md`, 11, *Full screen*). The press
    draws the terminal in that step, before the browser answers: every terminal
    act, and a list's closing on any event, moves the pages' shown count, as
    each change the view draws does (section 3). Full screen refused, the
    terminal stays, not full screen.
  - *Left:* "Exit" and `/exit`, `terminal::Act::Leave`: `terminal` unset, full
    screen left. The browser leaving full screen (its Escape, a long Escape where
    the terminal holds the key, its own control) leaves the look as it is: the
    terminal stays, not full screen, and an open list closes (*Keys,* below). A
    reload, which leaves full screen, opens the terminal again, not full screen,
    since only a person's press may ask for it.
  - *Where it draws:* in a browser, on `Chat`, `NewChat` and `Configure` while
    `terminal` is set; every other page, and the terminal shell's, as before.
    The frame draws no rail and no tab bar around it: the page's main is the whole
    window. Its regions in order: the top line, the conversation (with the draft
    beside it), the running line, the box and its list, the quiet line.
  - *The top line:* the bar, on one line. At its left "✦", the project's name, a
    faint "/", the level-1 heading, named as the bar's, showing the title as the
    bar's does, and the chat's number "T2". At its right "Draft" and "Exit". On a
    phone the star and the title alone: the project, its "/" and the number go.
    "Draft" is a button pressed while the document shows: `Act::Mode` to Split,
    or back to Chat, its shortcut `Ctrl+\`; with no document, where
    `chat_offers_draft` and the chat hasn't ended, "+ Draft", `Act::AddDraft`;
    otherwise none. Nothing else of the bar draws: its status, Pause or Resume,
    Close, Start over, "Start goal…", the mode group and the spend are commands
    and the quiet line's.
  - *The conversation:* the page's own, every line as the browser draws it, in the
    terminal's look (`browser.md`, 2.5, *A chat's terminal*): the person's words on
    a band after "❯", the agent's after "✦", calls folded under "▸", a note's
    write "✎ Wrote NAME · 30 lines Show". The speakers' names and the turns'
    times stay in the tree and the look hides them. A fold whose call failed shows
    its first failure's line on its folded row, "failed: WORDS", where today it
    says "1 failed". The column stays left, 100 characters at most.
  - *The running line,* above the box, while the browser would draw the chat's
    working dots, or the chat waits on its delegates or on an account: "✦",
    pulsing, then the bar's status words ("working", "waiting on 1 delegate",
    "Waiting for an account"). It draws no clock.
  - *The box:* the composer, "❯" before its text area, its name, placeholder,
    hint's keys and bounds as a chat's (*Where the focus stands*, above), with no
    "Send". On a chat that has ended, which a browser draws with no composer, the
    terminal keeps the box for commands alone: its placeholder "/new starts another
    chat"; words that are no command note "This chat is closed. /new starts
    another." Under it, *the quiet line:* at its left "/ for commands", or "paused ·
    a message resumes it" on a chat the person paused (*A person's pause*); at its
    right the picker's closed line, "Claude Opus 5.5 · high", a button that opens
    `/model`'s list, then " · 778 spent", the chat's own spend. On a phone the spend
    goes. While a list is open the quiet line gives it its place.
  - *Commands,* typed in the box: words that begin with "/" and hold no white space
    open the list under the box (skein's `ui.md`, 3.2, *options*), named
    "Commands": each command the page offers now whose name begins with the words,
    in the table's order, its name and its words. Enter presses the active one, Tab
    completes its name, a click presses it. Enter on words that name no command
    sends nothing and notes "Unknown command /WORDS. /help lists them.". A command
    pressed or sent clears the box.

    | Command | Its words | What it does | Offered |
    |---|---|---|---|
    | `/model` | Change the model, from the next turn | its list (below) | where the picker is |
    | `/effort` | Change the effort | its list (below) | where the chosen model has efforts |
    | `/draft` | Show or hide the draft | as "Draft" | where "Draft" or "+ Draft" is |
    | `/pause` | Pause this chat; a message resumes it | `Act::Stop` | where Pause is |
    | `/close` | Close this chat; it stays in Chats | `Act::Close`, its confirmation as a list | where Close is |
    | `/start-over` | Start a new configuring chat | `Act::StartOver`, its confirmation as a list | where Start over is |
    | `/goal` | Start a goal from the draft | "Start goal…", its sheet | where "Start goal…" is |
    | `/new` | Start a new chat | `Act::Go` to `NewChat` | always |
    | `/chats` | Open another chat | its list (below) | always |
    | `/theme` | Dark or light | the other theme | always |
    | `/font` | DejaVu Sans Mono or IBM Plex Mono, to compare | the other font; notes "Font: NAME." | always |
    | `/help` | Commands and keys | notes the commands and keys | always |
    | `/exit` | Leave the terminal | `terminal::Act::Leave` | always |

  - *A command's list,* under the box in the commands' place, its head a line of
    its own, its choices the list's options, the current one selected; taking one
    closes it, Esc closes it, and words typed close it:
    - `/model`: "Model. A change applies from the next turn."; the picker's
      menu's models (*The work's account and model*), each its shown name and,
      muted, what pays ("your subscription"), "default" after it on the default's;
      a provider with no model its name and "Connect a plan or key ›", a link to
      `ConnectModel`. Taking a model is `choice::Act::Model`. "All models" stays the
      browser look's.
    - `/effort`: "Effort. A change applies from the next turn."; the chosen model's
      efforts, `choice::Act::Effort`.
    - `/chats`: "Chats"; the person's live chats in the project, newest activity
      first, each "TN" and its title, as the Chats page's Live filter reads them
      (`tasks{query}`, its first page), read when the list opens; taking one is
      `Act::Go` to its `Chat { task }`.
    - `/close` and `/start-over`: the confirmation's question as the head ("Close
      this chat? It stays in Chats.", "Start over? A new configuring chat starts.
      This one stays in Chats.") and its two acts as choices, "Close" or "Start
      over" first, then "Keep it open" or "Keep this one"; a confirmation that also
      asks its delegates' fates draws as the browser's sheet, over the terminal.
  - *The terminal's notes:* lines the page writes after the conversation, in the
    terminal's look: `/help`'s table of commands and keys, an unknown command's,
    `/font`'s, a closed chat's. They are the page's state, at most 8, the oldest
    going first, gone when the page closes; nothing sends or keeps them.
  - *Keys:* Enter sends, Shift+Enter makes a new line, `Ctrl+\` shows or hides the
    draft, PgUp and PgDn read back (skein's `ui-web.md`, 7), Esc closes a list and
    otherwise does nothing; in a list, the arrows move, Tab completes, Enter takes.
    In full screen, where the browser lets the terminal hold Escape (Chromium's), a
    short Esc is the terminal's and a long one leaves full screen; where it doesn't
    (Firefox, Safari), one Esc leaves full screen and closes an open list with it.
    The desktop's own shortcuts stay the browser's.
  - *The draft beside it:* in Split the document in the right half, behind a rule,
    on a ground of its own; on a phone, at 760 px or less, the document takes the
    conversation's place and the box stays at the foot, as Split shows as Draft
    there. It is the page's own pane (*A chat's document*), its rules shared with
    the browser's look under the terminal's tokens (`browser.md`, 2.5). The pane's
    first paragraph, in both looks, reads its words as Markdown's inline runs,
    emphasis and strong among them, in its muted style.
  - *A goal's question,* where a delegate of the chat asked the person one: while
    the terminal draws a chat whose wait is on the person, the page reads the inbox
    of the chat's subtree, its questions (`Read::Inbox` with the chat as its root,
    as a tracked task's page reads its own), into the page's inbox, and draws its
    first question under the box as a list: its question line as the head, its
    parsed choices as options, the recommended one selected, then "Later";
    choosing answers it as Needs you does (`needs::Act::Choose`, `needs::Act::Later`).
    A question with no choices draws its line and "Answer in Needs you ›", a link to
    its card. The browser's look reads no inbox here.
  - *A proposal or an Accept card* in the conversation, where the browser's look
    draws its card: one line, its kicker ("Proposal") in the accent and its title, its
    acts after it ("Accept", "Reject…"), its facts folded under it as a call's are;
    Accept's confirmation as `/close`'s list. A delegation's card is one line too:
    "TN · KIND", its title, its state and "Open".
- **The Chats page** (the greenfield mock's `chats.html`): `Chats { filter }`,
  a page of the frame and no pane, a main with the token `CHATS_PAGE`,
  drawn as the mock's `.main` and `.col`, in this order:
  - *Its head,* `.intro`: `.kicker` "PROJECT · chats"; the level-1
    `.headline` "Talk to an agent about OWNER/NAME.", OWNER/NAME the
    project's home repository, read as Home reads it, or "Talk to an
    agent." with none; `.lede` "A chat answers a question or changes the
    code.", or, while `drafts_offered`, "A chat answers a question or changes
    the code, and can write a draft beside it. A draft becomes a goal for bigger
    work."
  - *With no repository* (*A project with no repository,* below), `.band.quiet-band`:
    `.tag` "No repository yet", "Chats in PROJECT talk and plan, but change
    no code until a repository is in." and a `.btn.small` link "Adopt a
    repository", to `Adopt { project }`.
  - *The list,* `.section`: `.section-head` with `.filters` at its left,
    "All", "Live", "Drafts" while `drafts_offered`, and "Closed",
    pressed-or-not buttons that filter in the page, reading nothing, the filter
    being in the page's address (`browser.md`, 2.3), a filter chosen a move, as
    the inbox's; and `.btn.small` "+ New chat" at its right, a link to `NewChat`.
    Drafts holds the rows whose chat has a document. The page reads
    `NoteScope::Tasks { project }` as it opens, bodies left out, and watches the
    person's facts; a note written later is seen on the next open.
  - *Its rows,* `.goal-rows` and `.goal-row`: the live group's lines by last
    activity and the ended page's by when each ended, merged newest first,
    a tie by the lower number first, each row one link: its number's, whose
    `::after` covers the row. A row's top line, `.top.mono`, holds at its
    left "T9", then for a configuring chat " · Configuring", for a chat with a
    document " · writes away-mode.md", then for a goal the person started from it
    " · started goal TITLE", TITLE from the goal's chip by `Subject::Chips`, and at
    its right its status, " · " and its age. The status is the bar's (one word for
    one state in both places, and "Closed" for every ended chat, as the
    filter says it; the user's pick, 10-23), in lower case by the look
    alone (the terminal keeps its words), `.state-needs` while it waits on
    the person: the tokens `STATUS_YOURS` and `STATUS_WAITING`. Its age is
    Home's short form (5.12, *span*): when it ended or was last active.
    Under it `.line`, its title, `.goal-row.ended .line` on an ended chat.
    A row names no project: the page is one project's. Its own spend, "N
    spent", "1 delegate" or "N delegates" while it has live ones, and on an
    ended line whose retention is Kept "Closed. Its transcript is kept."
    stay in the view and are hidden in the browser; no other line claims
    that its transcript stays.
  - *With no line to show,* once its reads have answered, `.empty`, "None.",
    under every filter.
  - *"More closed chats",* under "All" or "Closed" while an older page of
    ended chats remains, a `.btn.quiet.small` under the rows, reads it, in
    place of the last.

  On a phone, as the mock: `.headline` 25 px, `.goal-row { padding: 12px 0
  }`, `.goal-row .line { font-size: 15px }`; the frame's top bar and tab bar
  stay.
- **The inbox in the terminal** (`inbox.html`; the browser's is *Needs
  you,* below) shows what needs the person, then the
  updates, each entry its card (section 1), under two headings, each
  with how many entries it lists beside it, the updates' as new ones
  since what is read leaves them (below): "Needs you 2", "Updates 2
  new", or "0"; "2+" and "2+ new" where the inbox's document says its
  list was cut (jig's `more`, an owner inbox's next page, a filtered
  page's `cut`), so that a count never says less than there is; with
  none, once that section's read has answered, "Nothing needs you." or "No
  updates.", and no count before it. Its first heading's
  row holds "+ New chat", a link to `NewChat` (`/chats/new`, its text area
  focused: no act of its own), then
  its filters as folded menus, in order: projects (only with two or more),
  goals, then Kind. Each menu names its choice: "All projects" or the
  project's name; "All goals" or "Goal · TN"; "Kind" or "Kind · Question".
  The project menu holds All projects and each project; the goal menu holds
  All goals and the Goal field; Kind holds All kinds and each kind. A choice
  moves to the filtered inbox's address and folds its menu (`browser.md`, 2.3).
- **A card that needs the person** begins with its kind's tag, then
  "PROJECT › TN TITLE", for its own task alone, and its wait at the right
  (5.12). A proposal holds no proposing task's title, so its line stops at
  TN; questions, held tasks and person tasks use their documents' titles.
  A batch proposal keeps the mockup's Why, Goal, Landing and Budget rows:
  its reason quoted, its executor and specification, its landing resources
  and grants with priority, and its funding once. Template and charter
  names are not supplied, so their existing numbered words remain. Its
  acts are "Accept", "Reject with reason" and "Open TN ↗"; a chat's card
  keeps "Open in inbox ↗".
- **An update's card** has its heading as the link that opens it and marks
  a result seen. It repeats no project line or task-number link. The heading
  says "TN TITLE replied", "TN TITLE is done", or "TN TITLE ended";
  while its document is unread, the existing "Task N" words are the heading's
  link. Then the result's first line and the unread count and age (5.12).
  A reply has no preview: its document carries no reply's first line.
- **A question** in the inbox is a delegate's, asked of a task the person
  requested, their chat among them. It has no heading of its own: its line,
  then its words as written. "Answer in words" is folded until opened,
  holding "Answer words" and "Answer"; "Open TN ↗" opens the asked task.
  The answer goes to that task, which relays it to its delegate. "Pending"
  and "Answered" keep today's request outcomes, and a refusal stays under
  its card. This is the terminal's card; the browser's reads the
  question's words as *Needs you* (below) says.
- **Seen** is how far the person has read: jig's read positions (jig's
  `client/domain/pages.md`, section 3), one over their results and one
  over each chat's turns. An update is seen when it is opened, and no
  page offers "Mark seen":
  - *a chat's turns,* once its page shows them: as its window loads,
    and as turns arrive while it is open, through the newest its window
    holds;
  - *a result,* when the person follows its entry's link from the
    inbox: through that result, and so through every result before it,
    since jig keeps one position over them. A result opened from
    another page, or its entry left unopened, stays unread.

  A position only moves forward, with one `Seen` in flight for each;
  one refused or failed is sent again at the next move. What is read
  leaves the inbox's updates, and a task forgotten once read is erased
  then (jig's `domain/people.md`, section 6). The pages draw no mark of
  their own for what is unread, beyond the inbox's counts of it
  (above).
- **A task's page** is of its kind, as `task.html` lays each out: a
  tracked task's, a goal's (5.8); an agent's task's, for an agent's
  task with no priority (5.9). A task with the chat label opens as
  `Chat { task }`. A procedure's task shows a tracked task's header
  and plan, with its goal and its plan as "Its tasks", its ended tasks
  among them, and a change's task its panel (5.10); a person
  task shows a tracked task's header and plan only (section 11).
- **The forge's parts** of a page (a goal's issue and changes on its
  page; changes in flight and the project's health on `Changes`) are
  `forge.md`'s, and "Shown on the forge." while the forge's side does
  not read them (4.2).
- **Links are by number:** a chip, an entry or a line naming a task
  opens its page, from any page (jig's `client/domain/pages.md`,
  section 2).
- **The accounts page** is jig's reference UI's accounts, on a page of
  temper's own (jig's `client/domain/pages.md`, sections 7 and 11),
  with temper's words (5.5), in the forges' page's shape (with the
  user, 2026-10-23). It shows no value. Its acts are jig's accounts'
  requests (jig's `client/domain/contract.md`, 3.2), each sent as 3.6
  says. Its parts, in order:
  - *"Your accounts · N",* level 2, with "+ Connect a model", to
    `ConnectModel`, in its head; then each of the person's accounts as a
    disclosure, folded, named by the account's heading (5.5, An
    account), its summary its deciding source's state in words and the
    account's line (5.5, An account's line). Opened, only what the summary
    does not say (the request `connect-flows`): no name, models, "Ready" or
    "Pays for" again. The state's words and acts (5.5, Each state); its
    sources in order, each with its kind and state in words; on the
    deployment's accounts
    "Used by" and "Waiting for it", whose facts jig gives the
    deployment's accounts alone for now (a person's own leave them
    out, and its line its use: the user's pick, 10-23); and
    the account's acts: sign a source out, add a key to the account,
    replace or remove a key, remove the account, resume a source, move
    the account or a source, check a program's login again;
  - *the deployment's accounts,* for an owner, the same way under
    "Deployment accounts";
  - *the playground's account,* in a section of its own, folded, while
    the playground is read (4.8).

  A state's "Continue with ChatGPT" and "Use an API key" (5.5, Each
  state) are `ConnectModel`'s ways, sent from the row, a sign-in coming
  back to the page; the page adds no account of its own: that is
  `ConnectModel`'s.
  - *Its acts are the rows':* each names what it acts on, its holder,
    account and source, from the row it is pressed in, so that no act
    asks for a number. "Add an API key" and "Replace" open a secret
    field in their row, with "Add key" or "Replace key"; a value
    written in one row is dropped when another row's field is written.
    "Use an API key" in a state's acts is the row's "Add an API key".
    "Continue with ChatGPT" in a row is a sign-in for the account's
    provider, held by the account's holder: its address, then its end,
    are drawn in the row (5.5, A sign-in), as `ConnectModel`'s Signing
    in draws them, its paste among them (`ConnectModel`, The paste). "Use the API key first" moves the account's first
    key after its deciding source to that source's place. "Check
    again" checks the program's login of the source it names. "Sign
    out", "Remove" and "Remove account" ask to confirm first (3.6).
    The page has one request out at a time, as each page has: while it
    is out, every row's acts are disabled, and its outcome is drawn
    under the row that sent it.
  - *The playground's account,* under the level-2 heading "The
    playground", is one disclosure, "Scripted model · the playground’s ·
    held by the deployment", holding
    "It plays the playground’s scripts: it answers the suggested
    requests, and nothing else. It costs nothing, and nothing leaves
    this machine.", from the playground's read alone (4.8). An account
    of the playground's LLM provider (`PlaygroundAnswer.provider`) is in
    neither "Your accounts" nor "Deployment accounts", nor in their
    counts. While the playground is read, the page offers no "+ Connect
    a model" and no "Connect a model": there is nothing to add while it
    runs (4.8).
  - *With none of the person's own,* "Your accounts" holds "No model
    of yours yet." and "Connect a model", to `ConnectModel`.
  The frame's lede says "The LLM accounts temper’s agents run on for you. A
  task keeps its account: while the account is limited, the task waits for
  it."; with no project it follows the page's heading. Each folded row
  shows its name, holder, deciding source’s tag and summary. Opened, its state
  adds only what the summary does not say: its source order and source acts,
  the rule and use facts, then its account acts. Its name, models, Ready
  and Pays for are not repeated in the opened body.
  "Deployment accounts" appears only when the owner’s document holds accounts.
- **`ConnectModel`** (the request `connect-flows`) connects one more LLM
  account: the welcome's cards, panels and words, under Settings' path
  and in its column, with no list of steps. Choose; then a way's page, a
  sign-in's wait or a key's or a program's panel; then it lands on
  `Accounts`.
  - *Choose* is the heading "Connect a model", the lede "temper’s agents
    run on a plan you sign in to, or on an API key.", then the welcome's
    cards, the same function: a card for each plan the welcome names, its
    sign-in or its subscription. A card a way of which the person has an
    account says "✓ Connected" in its note's place, its button "Add another
    plan", and is no default: the default, its button primary, is the first
    way not yet connected. Under the cards, "Or an API key", a row for each
    key way with "Add key". The playground's scripted model, `demo`, is no
    way to connect (4.8). A way adds a new account to the person's own
    accounts, a key's to the deployment's when an owner has chosen
    deployment accounts; a plan's sign-in and a program's login are always
    the person's own, since only a person holds a plan (jig's
    `domain/accounts.md`, 3.2), and their ways are not offered while the
    deployment's are chosen. An owner has one quiet link at the page's
    foot, "Use deployment accounts", or "Use your accounts" while they are
    chosen, yours at the flow's start; the choice is the flow's state, kept
    across its steps, not its address.
  - *Signing in* is a note in the way's card's place: the way's button
    opens a new tab and sends the sign-in, whose answer is the address
    the tab goes to. The provider sends that tab to the engine's
    listener, and the page, which stayed, shows the sign-in's end, read
    on the `accounts` watch: Accounts with its confirmation, or the ending’s words and acts (5.5,
    A sign-in). The terminal prints the address and opens no browser
    (6.5). temper detects no browser elsewhere (section 11, A browser on another
    machine): the page offers the paste to everyone, below.
  - *While it waits,* a sign-in under way that the page draws is a note
    in the way's card's place, the same as the welcome's and drawn by
    the same function (*The welcome*, its Model step, *The sign-in
    under way*).
  - *From another computer,* for a browser not on the engine's
    machine, whose provider sends it to a loopback address where
    nothing answers. No entry says so: that person takes the same page,
    and *The paste* (below), folded under the note as "Signed in, but
    the page couldn’t be reached?". The tunnel still works, and needs
    neither.
  - *The paste:* "When ChatGPT is done, its tab shows a page that
    can’t be reached. Copy that page’s whole address from the address
    bar, come back here, and paste it. If a page opens instead, such
    as another program’s sign-in, close that program’s sign-in and
    start again." A field, "Address of the page that couldn’t be
    reached", and "Finish connecting", which sends jig's `FinishSignIn`
    for the sign-in's source, carrying the address, its surrounding
    spaces and line breaks trimmed, as its value (jig's
    `client/domain/contract.md`, 3.1 and 3.2). The
    address is held as a key's value is: the page's state alone, never
    in its address, the small store or a draft, dropped once sent, or
    when the page or the way is left; its field asks the browser to
    keep nothing (`autocomplete="off"`, no spelling check), and is
    disabled while empty or past the bytes `me` states. While the
    request is out, the button reads "Finishing…"; answered
    *finishing*, the field goes and the card reads "Finishing your
    sign-in…" until the `accounts` watch shows the end, Accounts or the
    ending’s words (5.5, A sign-in). Refused, the field stays empty
    with the refusal's words above it (5.5, The paste), and the sign-in
    still waits; refused *not waiting*, the card follows the watch: an
    end it shows, or, with the sign-in gone, "This sign-in has ended.",
    and the way's button again. *Lost* or *unsent* asks for the paste
    again: the client never sends it twice by itself.
  - *The sign-in's tab,* in a browser, in this order. The way's button
    sends the sign-in, and in that press's own turn, before the sign-in
    is carried, the root asks its shell to open a tab (`Request::Tab`,
    `Tab::Open`), which holds "Opening ChatGPT’s sign-in…" until the
    address comes: a browser blocks as a pop-up a tab opened once an
    answer has come back (skein's `ui-web.md`, section 10). The sign-in's
    answer, `AccountSignInStarted`, names its source S and the address.
    The root asks its shell to point the tab at the address
    (`Tab::Point`), its opener cut, once, from that answer: never from its
    state, so that no build or reload asks again. The page's address then
    names the source, in place of its history entry: `ConnectModel`'s at
    the way with the source,
    `/accounts/connect?provider=N&way=sign-in&source=S`, or the welcome's
    Model step at the way with the source (`browser.md`, 2.3). An answer
    with no address, refused or lost, closes the tab (`Tab::Close`), and
    the page says why, as 5.5 has it; so does an answer the page has
    left. A browser that refuses the tab is told to the root
    (`Event::TabRefused`): nothing is pointed, and the note offers the
    address as the person's own click (*The sign-in under way*). A browser
    that opens the tab and hides it is served by "Open it again". The
    terminal's shell opens no tab, and prints the address (6.5). Nothing
    leaves: the page holds the sign-in from its answer. The accounts page's
    row signs in the same way, its tab opened and pointed the same, and
    stays on its page with its link "Open the sign-in page", with no address
    of its own to name the source.
  - *Its end,* the page moves on by itself, on the `accounts` watch: to
    `Accounts` with its line (*Ready's place*, below), or the welcome's
    Ready, its address replacing the step's, or to the ending's words and
    acts. A hidden tab lets its
    stream go after a minute (skein's `ui-web.md`, section 10), so a slow
    sign-in moves on when the person returns, its stream opened again;
    the page says nothing of this. The new tab ends on the listener's
    fixed page, "Sign-in received. You can close this tab.", which never
    says how the sign-in ended (skein's `oauth.md`, 6.5): that's the
    page's to say, and temper closes no tab it has pointed (section 11).
  - *A reload while it waits,* the page is a new client at the address
    with the source (`browser.md`, 2.3): it takes the source's sign-in
    as its own and draws the same page, while the `accounts` watch lists
    it under way, then its end. The small store keeps the sign-in's
    address beside its source, and whether its tab was refused, so that
    the note reads the same and "Open it again" stays (section 7). Until
    the watch has delivered, the note alone; a source the watch shows
    neither under way nor ended (another's, or one whose end a later
    ending has replaced) is no sign-in of the page's: the way's card
    shows again, with its button, and the address drops the source.
  - *Details* is the way's field: a key's secret field and "Add key"; a
    program’s panel, its command and Copy, the folded “Use another login
    name” field and “I’ve signed in”. Declaring a program's login
    asks its name, 1 to 32 of `a` to `z`, `0` to `9` and `-`, `claude-`
    and the person's first name by default (`claude-alice`), and shows
    the command to run on each host, as *The printed command* (4.6) has it,
    after the words of `program_environment` (4.6) where it has any
    (`../protocol/credentials.md`, section 3). The tools' printed act
    includes the box's directory placeholders and the signed-in skip
    rule (`tools.md`, 5.5, area 5).
  - *Ready's place:* an account made lands on `Accounts`, its address
    replacing the way's, with the line "Added your ChatGPT plan. temper’s
    agents now run on it." (5.5's words) and a quiet "Start a chat", to
    `NewChat`, above the rows, and goes when the person leaves the page or
    acts on it. A plan's "Manage usage" is in its row.
  - *Its address* is its way (`browser.md`, 2.3): none at Choose; a
    way's provider and kind once one is chosen, Details and Signing in at
    the way's address. A key's and a program's card
    on Choose is a link to the way's address, Details; a sign-in's is a
    button that sends the sign-in and moves to the way's address,
    Signing in. Each way's page has a quiet "Other ways", to Choose, beside
    its main button, and its path's back link goes there too. A
    way the configured providers do not offer, or the playground's
    provider's (4.8), shows Choose.
  - *At a sign-in's address with none under way,* as a link to it, a
    reload or the banner's way opens it, Signing in shows the way's
    card again, with its button, which sends the sign-in. A sign-in
    started from another client is not this page's, unless its address
    names it by its source (above): its end is the accounts page's.
  - *Its state and acts* are a page of the domain's own, its way, its
    key's value, its program's name and its request, built from the
    accounts' requests (jig's `client/domain/contract.md`, 3.2); the
    welcome's Model step holds the same state (below). A move between
    its steps keeps them; leaving the flow drops them, the key's value
    first.

  The first run reaches its ways on the welcome's Model step (below),
  and its "Continue with ChatGPT" signs in with the default document's
  OpenAI (`../protocol/setup.md`, 8.3).
- **The forges' page** is the forge's part of the deployment's
  connectors (jig's `client/domain/pages.md`, section 7), on a page of
  temper's own (`forge.md`, 5.4). It is read for a person whose role
  on project 0 is `Owner`, in standalone the deployment's owner; any
  other's read is refused, and the page says so in jig's words, and a
  request the root refuses for the role says "Your role does not hold
  the deployment's forges." Its acts are the FORGE
  requests (section 8), each sent as 3.6
  says; a pasted key or token is a secret field (3.6). A connect's or
  an install's link is where the page's own tab goes, once (`Request::Leave`,
  from the answer), the request naming
  the page's address as its `back` (section 8); GitHub's last step
  sends the tab back there (`forge.md`, 9.1), and the page shows a
  whole address's link as an external link meanwhile (6.5); a
  connect's path (section 8) leaves at once, and none is drawn. How a
  connect ends is read on the page, never assumed from its answer. Its
  rows are the forges the store keeps, made by a set up or a connect
  (`../protocol/forge.md`, 2.1 and 2.2); "Remove" on a row, confirmed,
  sends `RemoveForge` (section 8; `forge.md`, 5.4). While the
  playground runs, it lists the playground's forge and offers nothing
  to add (4.8).
- **The welcome** (`welcome.html`) is one page, `Welcome { step }`, its
  step Welcome, Forge, Model or Ready, at `/welcome` and
  `/welcome?at=welcome`, `forge`, `model` or `ready` (`browser.md`, 2.3); its
  words are its mockup's (5.6). The tab's title is "Welcome · first run ·
  temper" on every step (`browser.md`, 2.5).
  - *Whether it shows* is the document's
    (`../protocol/configuration.md`, section 3), which the domain learns
    from the deployment's read (4.6): the forge's side reads it when a
    link first becomes live and each time one becomes live after it was
    not, as it reads the playground (4.8), and keeps the last answer. A
    read refused, failed, ended or not served, with no answer kept,
    reads as no welcome. Until it has answered, the page draws temper's
    name alone: no line of the box's, no step and no heading. With no
    welcome, the page at any step, `/welcome` and each `/welcome?at=`
    alike, moves on to the inbox, its address replacing the welcome's,
    and the store's state is not judged: the rest of this entry holds
    where the document shows it.
  - *Its frame* is temper's brand link to the welcome, `/welcome`
    (named "temper"), temper box's line and the steps alone: no person's name or "Sign
    out", no project, navigation or link state, and no accounts' banner
    (5.1); the playground's line while the playground is leaving, as on
    every page but `Chats` and `NewChat` (4.8, 5.1, 5.6). temper box's line says what this
    temper is for, in fixed words: "**temper box** · on this machine,
    to try it · more later, from Settings · Run temper on a server ↗",
    the last an external link to temper's help on running a server
    (`temper/docs/help/server.md`), the welcome being a quick way to
    try temper, not a deployment's wizard (the user, 2026-11-04). Each
    step's heading is the page's level-1 heading.
  - *Its steps,* at the frame's end, are a list named Steps of four links,
    each to its own step in every state: Welcome to `/welcome?at=welcome`,
    Forge, Model and Ready to their `?at=`, with no choice or way. They
    are the welcome's own, not the flows' list (5.2, A flow's parts), which
    `ConnectForge` and `ConnectModel` keep. Each draws its number or its
    tick in a ring, then its label; below 760 px only the current step's
    label shows. The step at the address is current, always, a choice or a
    way in it among them: Ready stays current, its "4" black, when its
    forge and model are done. A step is ticked when it is done and not
    current: Welcome is done at any other step or once a real forge is
    kept; Forge once a real forge is kept, even under way; Model once a
    model is usable; Ready never. The terminal prints the list as text,
    its links as the acts it offers any link (section 6).
  - *`/welcome?at=welcome`* opens the Welcome step, its link's address,
    whatever the store's state, once its facts are in; only a whole state
    moves on, to the chats page, as `/welcome` does.
  - *The store's state* is judged at `/welcome`, once each time
    the page opens, when its facts are in, from the forges the store
    keeps, the accounts and `me`'s projects (`../protocol/setup.md`,
    6.7 and 8.4; `../draft/first-run-projects.md`, 4). A
    real forge is one of the `forges` watch's rows other than the
    playground's (4.8); it is connected when its row says connected
    and, on GitHub, an installation is accepted (`forge.md`, 5.4), an
    owned or managed Forgejo's first repository being made before it
    connects (`../protocol/setup.md`, 6.7); a model is usable as below;
    and the person has a project when `me` lists one in which they hold
    a role. No adoption is read.
    - *Empty:* no real forge kept and no model usable. The step shows
      its cards (*The Welcome step,* below).
    - *Unfinished:* neither: a real forge kept, but none connected, or
      no model usable, or a model usable with no real forge, or no
      project. The page opens at the step left undone, Forge while no
      real forge is connected, else Model while no model is usable,
      else Ready, its address replacing the welcome's.
    - *Whole:* a real forge connected, a model usable and a project.
      The page moves on to the chats page,
      its address replacing the welcome's, so that back does not return
      to it.

    For a person whose role does not hold the deployment's forges
    (above), the forges are not theirs to set up, and the model alone
    decides: whole once one is usable, empty otherwise. A model is
    usable, with no playground read, when the `accounts` document says
    so, its `usable`, which counts the configured model too (jig's
    `client/domain/contract.md`, 5.1); and while a playground is read,
    when an account of that document is usable whose LLM provider is
    not the playground's, or, for an owner, one of the deployment's
    (4.8). While the playground runs, its forge and its model count for
    nothing, so the store reads as it would without it: empty, unless
    an account of the person's own, added before, is usable, which
    makes it unfinished at Forge (4.8). Its facts are
    in once the deployment's read and the playground's have answered,
    a playground or none,
    the `accounts` document is delivered, its accounts as far as it
    shows them, and the `forges` watch has delivered its rows. A read
    refused, failed, ended or not served is in, with nothing in it: no
    playground, no account, no forge, so that the page never waits on
    a read that will not answer. Until its facts are in, the step shows
    its heading alone. Its other steps never move on of their own.
  - *The playground is offered* where the document shows the welcome,
    while no real forge is kept: the
    Welcome step's card, "Try the playground", and the foot's "Try the
    playground instead" on Forge and Model. Once one is kept it is
    offered nowhere, and a start is refused as not offered
    (`../protocol/setup.md`, 8.4): an unfinished forge is started over
    first (below), and a whole one never. A real account alone does not
    end it (4.8).
  - *The Welcome step* draws "Set up temper"'s card, then, while the
    playground is offered, "Try the playground"'s and the note under them;
    not offered, both are left out. With a real forge kept, the card's
    button reads "Go on setting up" and goes to Ready when a model is
    usable, else to Model; with none, it reads "Set up temper" and goes to
    Forge.
  - *"Try the playground"* sends `StartPlayground` while no playground
    is read (4.8) and opens `NewChat`, in the playground's project
    once its read names it, which the start's acts make
    (`../protocol/setup.md`, 8.4); until then `NewChat` shows the
    playground's start under way, as in that project.
    *"Set up temper"* opens its Forge step. Its card comes first, the
    default, its button the step's one primary act, the playground's
    card after it, its button secondary (the user, 2026-11-04). It
    names the forge and the model in short, in fixed words: "Your
    forge: a Forgejo temper runs for you, or GitHub. Your model:
    ChatGPT or Claude.", the forges and the plans v0.1.0 supports (the
    user, 2026-10-26), not the offered `me.providers` ways.
  - *Forge* is `ConnectForge`'s Choose and Details (`forge.md`, 5.4),
    its Choose two cards, Forgejo's first, the default, whose one way,
    "Run on this machine", opens Forgejo's sheet (below), then GitHub's
    (`forge.md`, 5.4); a Forgejo deployment of the
    person's own is connected from the forges page, not here (the user,
    2026-11-04). Its choice is in the welcome's address (`browser.md`,
    2.3);
    inside a choice, the foot's one "‹
    Back" returns to Forge's Choose. The welcome has no "‹ All
    forges"; `ConnectForge` keeps it. Its foot, on Forge and Model, is
    "‹ Back" and, while the playground is offered, "Try the playground
    instead": a set up half done is left by no act of the foot's
    (the user, 2026-11-04). The step connects a forge and
    nothing more, on every kind: it adopts no repository and asks no
    project's name (`../draft/first-run-projects.md`, 4).
    *Forgejo's sheet* (the user, 2026-11-04) is Forgejo on this
    machine's Details on the welcome: a modal sheet over Choose, at its
    address (`browser.md`, 2.3), headed "Forgejo on this machine", with
    "temper starts it here and connects to it; you connect your model
    meanwhile."; then what it starts with, "Start it with", three
    choices, each a card with its line, the chosen one's own fields
    under it:
    - "An empty repository", "A README and a starter CI check.", the
      default, its Organization and Name `work` and `hello` until
      written;
    - "Copies of public repositories", "Copied into your Forgejo from
      GitHub or another git host, to play around with. Nothing is sent
      back.", its addresses "one per line", `github.com/owner/name`;
    - "Nothing for now", "Your first project opens with no repository.
      Make or copy one later from temper’s UI, or adopt one you push
      yourself.".

    Its foot is "Cancel", also Escape, back to Choose, then the primary
    "Run on this machine". Run sends the `SetUp` that makes the forge
    (`forge.md`, 5.4), its first repository as the choice says
    (`../protocol/setup.md`, 6.7):
    - an empty repository, with the names written, the repository made
      unadopted; with either blank, one alert, "Both an organization and a
      name.", and nothing sent;
    - copies, each non-empty line read as an address: `http://` or
      `https://`, `.git` and trailing `/` stripped; `HOST/OWNER/NAME`, or
      `OWNER/NAME` meaning `github.com`; then the source's own check
      (`../draft/repository-copies.md`, 5.4). A line that does not read
      alerts `Not an address: LINE. Write it as github.com/owner/name.`;
      none at all alerts `At least one address, as
      github.com/owner/name.`; more than 16 alerts `At most 16 addresses.`.
      Nothing is sent until they read. The forge connects with no
      repository made, and the copies wait for Ready;
    - nothing for now, which makes none. Ready offers "My project" and the
      plain lead.

    Refused, the refusal shows above the foot, in the set up's words
    (4.6); while a set up is pending, or the page is out of date, Run
    is disabled. Once the `SetUp` is accepted, the welcome moves on to
    Model, the set up running on.
    *A forge cut short:* a forge of the person's kept and not finished,
    not connected, or on GitHub connected with no accepted
    installation, is no choice. Forge's Choose shows it in their place
    (`../protocol/setup.md`, 6.7), in a card headed "not finished":
    "FORGE · not finished", FORGE its name (`forge.md`, 5.4); what its
    removal leaves, in the words of Remove on its row (`forge.md`,
    5.4); then "Start over", which sends `RemoveForge` with no
    confirmation of its own, the card having said what goes, and, once
    it is answered removed, opens the step's choices again,
    `/welcome?at=forge`, the set up's state dropped. An owned Forgejo
    set up afresh then takes its binaries from the cache
    (`../protocol/setup.md`, 6.6), its directory prepared anew. On
    GitHub, whose App's records are kept as they come, "Go on" comes
    before "Start over": it goes on to the App's installation, as
    Connecting's "Install on GitHub" does (`forge.md`, 5.4), and
    "Start over" removes the forge, the card saying that the App on
    GitHub stays the person's to delete. While the removal is out, both
    are disabled. An account added before stays: it is the person's,
    not the set up's, and the Model step shows it ready.
    A forge of the person's under way or connected is no choice either
    (*The Forge step with a forge,* below); but a GitHub
    forge with no accepted installation, or a Forgejo connecting
    as its admin, shows `ConnectForge`'s step for it in the step, its
    `back` the welcome's Forge step, `/welcome?at=forge&forge=N`, so
    that the tab GitHub sends back finds its choice there. While the
    step's address names a forge (`forge=N`) that is connected, as *The
    store's state* says, the welcome moves on to Model, its address
    replacing the step's, so that "‹ Back" from Model, to
    `/welcome?at=forge`, stays on Forge. Its first repository, made on
    an owned or managed Forgejo, waits unadopted: the person adopts it
    later, from the project's "Adopt a repository" (4.4).
    *The Forge step with a forge:* for a personal forge under way or
    connected, GitHub with an accepted installation among them, the step
    is the heading "Connect a forge"; the lead "One forge is enough to
    begin; add others later, from Settings."; the forge's status, in the
    column, not the Model step's pane: under way a spinner, the forge's
    name, a muted line of `welcome_line`'s words and the disclosure "Each
    step"; connected a tick, the name and "connected as LOGIN · ORG/NAME
    made", on GitHub "temper’s App on ACCOUNT", ACCOUNT the first accepted
    installation's, with no count of repositories; then the primary
    "Continue", to Ready when a model is usable, else to Model; then the
    foot. A set up failed, a forge cut short, GitHub's installation and
    Forgejo's admin steps keep what this entry says of them.
    *Model* is `ConnectModel`'s Choose and Details, its state and acts
    (above), its way in the welcome's address (`browser.md`, 2.3).
    Its lead is "temper’s agents run on a plan you sign in to.". Its
    ways sit directly under the lead, with neither the standalone
    Choose container nor its holder controls, as two cards, one for each
    plan its card names (the user, 2026-11-03), the plan alone, no key
    (the user, 2026-11-04: a key is added from the accounts page):
    "ChatGPT", for the `openai` provider, "Your ChatGPT plan: sign in,
    and temper’s agents use its included usage.", the muted "Sign in,
    in a new tab · about a minute", and "Continue with ChatGPT", the
    default, the step's one primary act; and "Claude", for `anthropic`,
    "Your Claude subscription, through Claude Code on this machine, with
    a config of its own, apart from your own Claude Code.", the muted
    "One command in a terminal, on this machine", and the secondary link "Use your Claude subscription".
    A provider offering neither its sign-in nor its program has no
    card. No other provider's ways are on the welcome, the box's fake
    model among them; with neither provider's plan offered, the
    step shows Choose's cards as `ConnectModel` does. Entering from a deployment
    chooser resets the holder to the person's own. Inside a way, the
    foot's "‹ Back" returns to
    Model Choose; on Choose the foot's returns to Forge. Standalone
    `ConnectModel` shares these cards and panels. An account added there is the accounts page's too; a
    sign-in holds the page, and moves on to the welcome's Ready,
    `/welcome?at=ready` (`ConnectModel`, *Its end*).
    *A model usable,* at any way or none, for whoever comes back to the step
    once a model is usable, since "I’ve signed in", a sign-in’s end and a key
    added go on to Ready at once: the heading "Connect a model", no lead; a
    status, ticked, "✓ ACCOUNT", in the words of Ready's Model
    line (below); the primary "Continue", to Ready; the foot. On the
    welcome it stands in for the choices and for a made sign-in's card,
    with its facts, "Continue with ChatGPT", "Manage usage" and "…is on
    the Accounts page".
    *A way's page* has its own level-1 heading, no lead, then one panel:
    - *Key,* "An OpenAI API key": "From OpenAI’s platform, under API keys.
      What temper’s agents use is billed to the key’s organization."; the
      field "Key, pasted once", the panel's width; "Add key", disabled
      until typed; and "Nothing checks a key when it is added:
      a wrong one shows once OpenAI refuses a run." Once it is added and a
      model usable, the step moves on to Ready, its address replacing the
      step's; refused, the page stays with the refusal's words.
    - *A held sign-in* takes *The sign-in under way*, below.
    - *Claude,* "Your Claude subscription": "temper sets Claude Code up on
      this machine in an environment of its own: its settings, sessions
      and sign-in stay apart from your own Claude Code. In a terminal
      here:"; the command (4.6, *The printed command*), NAME the program's
      default name (5.5), with no field for it, after the words of
      `program_environment` (4.6) where it has any; the muted "It makes that
      environment, then Claude Code asks you to sign in to your
      subscription there. temper never reads its login."; the primary "I’ve
      signed in". "I’ve signed in" sends `DeclareProgram` under
      NAME unless an account of the person's has that source, then the
      accounts page's `CheckProgram { source }`, as `temper ctl account
      verify` does, and the step moves on to Ready at once, its address
      replacing the step's: nothing on this page waits or changes first.
      The check's end shows on Ready (*The try,* below). A refused
      `DeclareProgram` or `CheckProgram` says the refusal's words under
      Ready's Model line.
    - *The command's Copy,* on each of the welcome's command blocks, in a
      browser alone: a quiet "Copy" in the block's top right corner, whose
      right padding keeps the text from running under it at every width. A
      press writes the command's exact text, as one line, an effect asked in
      the press's own turn (skein's `ui.md`, section 13; `ui-web.md`, section
      11). It reads "Copied" for 2 s. On plain http, where a browser has no
      clipboard, and on any refusal or failure, it reads "Couldn’t copy" for
      2 s, and the text stays selectable. A press within the 2 s writes again,
      and an answer to an earlier press says nothing. The terminal has no
      button.
    - *A plan's sign-in,* its way at its address with none asked, or one
      ended with no model usable, "Signing in to PLAN": `ConnectModel`'s card
      for the way, or its ending's words and acts, in the panel.

    *The sign-in under way* is a note in the card's place: a spinner,
    "Waiting for ChatGPT’s sign-in, in the new tab", the muted "Sign in and
    approve temper there. This page moves on by itself.", once the address
    is known "Open it again ›", the address as an underlined external link,
    a tab of its own, and under the note the disclosure "Signed in, but the
    page couldn’t be reached?", small and muted, holding *The paste*, its
    field the note's width. A browser that refuses the new tab reads
    "ChatGPT’s sign-in", no "in the new tab", with "Open ChatGPT’s sign-in ›"
    as the primary button in the link's place, the person's own click. The
    terminal reads "ChatGPT’s sign-in" too, with no spinner, and its link reads
    "Open the sign-in page", printed with the address (6.5). `ConnectModel`
    draws the same note, link and disclosure.
    *Finishing a paste* is the same note, a spinner and "Finishing your sign-in…". A sign-in's
    other endings (declined, held, refused, the port taken) keep their
    words and acts, drawn as `ConnectModel`'s are (5.5, A sign-in). After
    the choices and before the foot, once a forge is acted on, a region
    named "Forge" holds the forge's line and its steps, so that Model
    says what Forge did: while it sets up or connects, "FORGE ·
    ORG/NAME · CONNECTION", ORG/NAME its first repository, and a disclosure, "Each
    step", holding `ConnectForge`'s Setting up steps; once connected,
    as *The store's state* says, ticked, "✓ FORGE · connected as LOGIN
    · ORG/NAME made", its first repository made,
    LOGIN temper's on the forge (`forge.md`, 5.4). A forge with no first repository, GitHub's
    or a Forgejo connected with its bot's tokens, reads "FORGE ·
    CONNECTION" while it connects, and once connected, ticked, "✓ FORGE
    · CONNECTION". In the browser, in every state, FORGE is strong and
    each detail a muted text node on a line of its own, the repository
    before the connection while under way or failed. While setup runs or
    the first repository is being made, a textless, indeterminate progress
    node named by CONNECTION follows the paragraph before "Each step",
    styled `temper-flow-busy`; GitHub's installation waits have none.
    While the row says how far its fetch has gone, that node is the
    fetch's, styled `temper-flow-meter`: named "Fetching Forgejo" or
    "Fetching its runner", its text the percentage or the bytes as the
    connection says them (`forge.md`, 5.4), and with a size its fraction,
    the bytes written of the size, so that its bar fills (`browser.md`, 2.5).
    Its muted detail then names the binary alone, "Setting up: fetching
    Forgejo", the figure being the bar's, and the paragraph's name says
    both, as CONNECTION does everywhere else.
    The terminal has no bar and says the line in that order. Done or
    failed, there is no bar. The browser's stacked line has no dot or
    tick in the text: the stylesheet draws a quiet ring, or a green
    tick once connected. From 1280 px the region is a 272 px pane to
    the right of the 680 px column, its top at the heading's, with a
    hairline at its left; narrower it stays after the choices, with a
    hairline above it. The terminal prints "Forge:" and the line's
    words indented under it, joined with dots and a leading "✓ " once
    connected. The line's name is its words without the tick. The line
    names no project, since Ready's act is what puts its first
    repository in one; a set up or a first repository that failed keeps all
    five steps, with its failed step marked ✕, preceding steps done
    and later steps to come, followed by "STEP failed: WHY"
    (`forge.md`, 5.4). With no failed step named, nothing is marked and
    the line is "Set up failed: WHY". The line is one
    paragraph, from the facts Ready reads on the forge's row (the
    person's forge, its connection and its first repository, or the
    set up's request until the row names it), and every line but the
    ticked one keeps "Each step". *The seam:* the Model step's
    state, acts and Choose are `ConnectModel`'s; the step's frame, heading,
    lead, ways' pages, notes, set up line and foot the welcome's.
    *The try,* an account tried once it becomes usable on the welcome's Model
    step or Ready, or on `ConnectModel`, runs in the background and nothing
    waits for it. Its task, model selection, payer and end are
    `../domain/trial.md`, 2 to 5; its facts and the lines they feed are 6.1 to
    6.4. It is not a chat: it lists nowhere, adds nothing to Home's counts and
    writes no project's document. One short turn is paid by the person, as
    for a chat: included usage on a plan, or the key's price. Its first
    committed turn answers it; a run ending otherwise or a minute ends it
    failed, cancelling the task so that no second turn is spent. It is
    Model's line in Ready's pane (*Ready,* below), the same parts as Forge's
    line, drawn by one function. Its steps are the list Forge's has, "Each step" its name, each with its state in
    words: for a Claude subscription "Signed in to your subscription",
    "Claude Code starts on this machine", "Claude answers"; for a plan's
    sign-in "Signed in to your PLAN plan", or for a key "Key added", ticked at
    once, a direct account having no program to start, then "PLAN answers",
    "ChatGPT answers". ACCOUNT below is "Claude subscription", "ChatGPT plan",
    "OpenAI API key", in Ready's Model line's words; WHO the PLAN.
    - *Running:* the quiet ring, ACCOUNT, strong, the muted "Trying: STEP", STEP
      the step under way; the moving bar Forge's has, named by those words;
      then the list, folded under "Each step", the step under way marked.
    - *Answered:* ticked, ACCOUNT, strong, the muted who it is, as Ready's
      Model line has it, the muted "answered in N s", and no "Each step". The
      page does not move; the person is already on Ready.
    - *Stopped:* "✕", ACCOUNT, strong, the muted "WHO didn’t answer" in the
      signal's colour; why, the run's ending or the wait in the person's
      terms (`../domain/trial.md`, 6.4), a paragraph,
      `temper-welcome-status-why`; `temper-welcome-status-acts`, "Try again",
      outlined, and the act that answers the reason, quiet, "Sign in again", a
      link to the way's page, or, with none, "All ways"; then the list, folded
      under "Each step", the step that stopped "✕" and in that colour, those
      before it ticked. Where the account is not usable, the page is Almost
      ready and its act disabled, as for any model not usable. Where only the
      try failed, the account being usable, the page stays Ready and its act
      works: a failed try keeps no one out. "Try again" checks the host again
      at a host's stop; past that, it runs another trial. A wait that won't
      settle in a moment says why at once, while its trial still runs; no
      "Try again" is offered until the trial ends.

    An account just declared counts as usable for Ready's heading and act, and
    the steps' tick, from "I’ve signed in" until its host's check says
    otherwise, so that the page does not go from Almost ready to Ready a moment
    later. Leaving Ready stops nothing: the trial belongs to the account,
    not the page. Removing its account cancels it; a restart cancels live
    trials and loses their lines, starting nothing (`../domain/trial.md`, 5).
    Home's line under its intro says "WHO didn’t answer when tried", with
    "Try again", while a trial is failed (`../domain/trial.md`, 6.3); answered
    or running, it says nothing. The accounts page holds the same lines under
    the account the run took, or, until then, the account that started it
    (`../domain/trial.md`, 6.2). The terminal says the same lines as text,
    indented under the pane's name, with no bar: "ACCOUNT · Trying: STEP",
    "✓ ACCOUNT · WHO IT IS · answered in N s", "ACCOUNT · WHO didn’t answer".
    *Ready* is headed "Almost ready", with "Your first project can be
    created once your forge and model are ready.", until a real forge
    is connected and a model usable, then "Ready", with "Everything is
    connected. Create your first project, with ORG/NAME in it, and
    start working.", ORG/NAME the first repository its act will adopt
    or the first copy its act will ask, "with 2 repositories in it" for
    more than one copy (below), or, with none, "Everything is connected.
    Create your first project and start working."; then its one form
    (below), and after the form and before the foot, the pane, the region
    the Model step has, in its look and its place at each width, named "Forge
    and model", with two lines, Forge's then Model's, always, each the same
    parts: a mark, its name, strong, and each detail muted on a line of its
    own. Forge's is the Model step's region's line, as *The store's state*
    says: ticked, "FORGE · connected as LOGIN · ORG/NAME made" once connected,
    the last part while a first repository was
    made, and no such part for copies, which run at Ready's act, or
    "FORGE · CONNECTION" (with its "Each step" folded), or
    "Forge none yet · Connect a forge", one line, the link, to the Forge
    step, after " · ", as Model's is; Model's is *The try* above, and while
    a model is usable, as *The store's state* says (the one test Ready and
    the welcome share), with no try's fact, ticked, "ACCOUNT", the first usable account of
    the `accounts` document but the playground's (4.8), by the kind of
    its usable source and who it is: "PLAN plan · WHO" for a
    signed-in plan ("ChatGPT plan · owner@example.com"), "PLAN
    subscription · WHO, signed in on this machine" for a program's, WHO
    the email its LLM provider gave (jig's `domain/accounts.md`, 3.7)
    where it gave one, and otherwise the account's display, or for a
    program's the source's name;
    "BRAND API key · billed to the key’s organization" for a
    key, an account of no named provider by its whole heading (5.5,
    An account); no model or effort, which are chosen apart from the
    welcome (5.5; the user, 2026-11-04); or "MODEL" where no such
    account is usable (the configured model), MODEL a new chat's as
    the composer's line names it (5.5); or, while
    a sign-in of the person's is under way on the `accounts` document
    and no model is usable, "Model PLAN plan · signing in…", PLAN
    its LLM provider's plan (5.5), with no act, so that a person come
    back from the sign-in reads it as under way; or "Model none yet"
    with "Connect a model", to the Model step, and under it, where the
    person's last sign-in (the `accounts` document's `last_end`) ended
    with no account usable, that ending's words and acts as
    the welcome mockup draws them: declined, held or misconfigured,
    using `ConnectModel`'s words (5.5, A sign-in), with a fresh
    "Continue with ChatGPT", and "Use an API key" for declined when
    offered. Any other refusal or failure says so in its words of 5.5,
    A sign-in, with the same fresh "Continue with ChatGPT": a sign-in
    that ends with no model usable never leaves Ready silent. These
    read `OwnerAccounts.last_end`, without retaining the signing page. The port-taken fact is not in that document
    (M-j); Not eligible remains Future in the mockup. There is no model
    line until the `accounts` document is delivered. After the lead,
    Ready's one form, its first project
    (`../draft/first-run-projects.md`, 4): the field "Your first
    project", holding the offered name until the person writes another,
    at most jig's 64 bytes, kept while the page is left and come back
    to; then its one act, "Create your project", and "Rename it
    whenever you like.". While no real forge is connected or no model
    is usable, the act is disabled and the field's Enter makes
    nothing; once both are, the act, or Enter, makes the project as
    named, without its outer blanks, adopts the first repository into
    it and opens it, not waiting for the adoption, so that nothing is
    half made. With no name written, it makes nothing and says "A name
    for it: NAME, or one of yours.", NAME the offered name:
    - *The offered name:* the first repository's (below) or the first
      copy's with its first letter in capital, "Hello" for `work/hello`,
      "Widget" for `github.com/acme/widget`, its bytes otherwise
      as they stand; "My project" when the rows name neither
      (GitHub, "Nothing for now"), or the name is past jig's 64
      bytes. It follows the `forges` watch until the person writes in
      the field; what they wrote, even emptied, is theirs from then on.
    - *Its request:* jig's `MakeProject`, named as written, on policy 1,
      the stock policy (`../protocol/configuration.md`, 13.4), in the
      deployment's scope (jig's `domain/people.md`, 5.1), one keyed
      request (3.6) under a key of the welcome's own, named
      `welcome-default` and the attempt's number, one attempt per
      press, never the forge's set up keys. While it is out, the field
      and the act are disabled, the act reading "Creating NAME…".
    - *Made,* `ProjectMade { project }`: the welcome opens that
      project as a project's button does (5.3), choosing it once `me`
      lists it, and shows its Home, `/projects/<project>`, then adopts
      (below).
    - *The name taken:* jig's refusal of a name another project holds
      opens the person's project of that name, as `me` lists it, as
      Made does, adopts into it, and makes no other name. With no such
      project of the person's in `me`, it is refused as below.
    - *Its adoption:* the first repository is the one the adoption
      form opens filled for (4.4): made, adopted nowhere, on the real
      forge numbered lowest, as the rows read when the project is made.
      With one, the act sends, once and in the project's scope, the
      forge's family's `Adopt`, the one `temper ctl forge setup
      --project` sends (`../protocol/setup.md`, 6.7), under the same
      key's name: owned, the project's home, its prefix `temper/`,
      landing on `main`, with CI and no checks. With none, nothing is
      sent. It is out until its answer, and Home says it (*Home,*
      below); the page and the welcome wait for none of it. Refused or
      unanswered, the project stays made with no repository (*A project
      with no repository,* below), and the adoption is offered again
      from Home, never from Ready, with the refusal's words. Once a
      refusal has answered, each attempt names a key of its own, since
      the engine answers a named request's repeat with its first
      answer.
    - *Its copies:* with a row whose `starts_with` names copies, the act
      sends, in the project's scope, one `MakeRepository` (4.6;
      `forge/making.rs`) per address in the order given, its org and name
      the source's, `into { project, home }`, `home` true for the first,
      each under a key of its own, `welcome-copy-N`, N the address's place.
      All are sent at once, not waiting for the project's page: the
      engine's rows queue them. A taken name's project takes them the same
      way. Home says how they run (*Home,* below).
    - *Any other refusal, or no answer* (unknown past the retention):
      nothing is made; under the act, "Your project was not made: WHY",
      WHY jig's words for its refusal, or "It is not known whether your
      project was made.", and the act is offered again, a new attempt
      under a new key.

    Under the form, while a forge is under way and not ready, "Forgejo is
    still setting up; this page moves on by itself."; then the pane; then the foot, "‹
    Back" to Model and, while the playground is offered, "Try the
    playground instead", as on Forge and Model. "Set up again" keeps its
    place, at its natural width.
    On a connected owned Forgejo whose page facts offer sign-in through
    temper (`forge.md`, 5.4), "Open Forgejo ↗" ends the Forge line, with
    the same `ROOT/user/oauth2/temper` address as Forges; no other
    forge's link is drawn. In the ticked line, the browser's ORG/NAME is
    the repository's link where it has an address (`forge.md`, 5.4, *A
    repository's link*): "ORG/NAME ↗", then " made".
- **With no project yet** (`../draft/first-run-projects.md`, 2 and 4): until the
  playground's start or the welcome's Ready makes one, the
  Chats page's head and list and `NewChat`'s layout give way to the level-1
  heading "No project yet" and "Setting temper up makes your first
  project.", with "Set up temper", to the welcome, which opens at the
  step left undone (The welcome, above); and, while the
  playground is offered and none is read (3.5, The welcome; 4.8),
  "Try the playground". With no welcome (3.5, The welcome) the page
  offers neither, and its line reads "Make one from the project
  switcher.": "+ New project" there makes one
  (`../draft/first-run-projects.md`, 7). The navigation offers no project's entries
  (5.3), and the project's place in it reads "No project yet".
- **A project with no repository** (`../draft/first-run-projects.md`,
  5): a project has none while the `forges` watch has delivered its
  rows and none of their `repositories` names it (4.6); with the
  watch refused, ended or not delivered yet, nothing is said. It is
  usable meanwhile: its chats start and run, a chat that names no
  repository running with no workspace (`../domain/agent.md`, 4.2).
  Home says so in its second step (*Home,* below). Its
  Chats page says so in its band: "No repository yet", and "Adopt a
  repository", a link to the adoption form into this project, `Adopt {
  project }` (4.4), and `NewChat` in its thread's line. The
  playground's project, while the playground is read, shows its
  context line instead (5.6). Where the page draws it is the look's
  (`journeys.md`, 7); its words and its link are these.
- **Home** (the user's greenfield mock's `home.html`; `journeys.md`,
  7): `Home { project }`, at `/projects/<project>`, is the page the
  welcome's Ready and "+ New project" (5.3) open, and the brand's link.
  Going to it opens that project as a project's button does (5.3); with
  no such project in `me`, it says "Project N is not one of yours.", N
  the address's number, and the chats' link. One column of prose, in
  this order:
  - *Its intro:* a kicker, the current day and time, updated each minute
    (the user, 2026-11-04), the level-1 heading and
    a lede, as *To begin* or *Home on any project* (below) says.
  - *A failed try:* under the intro, "WHO didn’t answer when tried" and
    "Try again", while one of the person's trials is failed, whatever the
    intro says (`../domain/trial.md`, 6.3). Running or answered adds no line.
  - *To begin* (`../draft/to-begin.md`, 2 to 4), while it shows (below).
  - *Needs you* and *Chats,* as on any Home, and left out, empty, while
    To begin shows, since its steps stand for them.

  **To begin,** on a project's Home while a step is left and the person
  has not skipped it on that project, a project the welcome made or any
  other, the playground's as any: nothing records which project the
  welcome made, and every fact below is read, none kept but the skip, so
  a refresh, a restart of the engine or another device computes the
  same. While it shows:
  - *The intro:* the kicker; the headline "PROJECT is ready.", never a
    count, since nothing in To begin is owed; the lede, "Your agents run
    on your ACCOUNT, and Forgejo on this machine." (ACCOUNT the kind of
    the first usable account of the person's, as Ready's model line
    chooses it: "ChatGPT plan", "Claude subscription", and with a key
    "Your agents run on an API key"), ", and FORGE holds your code."
    for another connected forge, each half left out with no such fact.
  - *More accounts:* after the lede, on its line where it fits and under
    it on a phone, a link to `Accounts` naming what the person does not
    hold yet: "Add Claude or another plan" with a ChatGPT plan alone,
    "Add ChatGPT or another plan" with a Claude subscription alone, "Add
    another account" with both or with a key, and with no usable account
    at all. Its words offer other providers and models and promise no
    capacity, since a limited task waits for its own account (jig's
    `domain/accounts.md`, 4.3).
  - *The card:* a region named "To begin", its head holding the level-2
    heading "To begin", the tag "Optional" and, at its right, the button
    "Skip"; under them a line by what is left: "Three steps left. Skip
    them if you like; they stay in Settings and Chats.", "Two steps left.
    Skip them if you like; they stay in Settings and Chats.", "Two steps
    left. Skip them if you like; they stay in Settings." with the first
    and second left, "One step left. Skip it if you like; it stays in
    Settings." with the first or the second left, and "One step left.
    Skip it if you like; it stays in Chats." with the third. Then one row
    a step in this order, a done step keeping its place: its mark, drawn
    by style, the step's number or a green tick once done (the terminal
    says them in words); its title, a level-3 heading, over one short
    line; its act at the row's right while the step is left. Done, the
    title is muted and regular (`temper-home-done`).
    1. "Tell temper how you work", "In your own words: how you plan, review
       and land work. temper proposes how it will run that, and nothing
       changes until you accept.", with "Start", filled, the act of
       *The first chat* (below). Done once a configuring chat runs or ran
       in the project, or a line of the `settings` document follows a
       change; done, its title alone. A configuring chat is a row of the
       project's chats, as `Chats` reads them, whose chip's executor is
       the charter of template 3 as `me` lists it
       (`../protocol/configuration.md`, 13.4): a row names its label,
       which is a chat's, and no template. With no template 3 in `me`
       the page it opens starts the stock chat, and only a setting's change marks
       the step done. A person who cannot read that document sees no
       change.
    2. The repository, in the first state that holds: adopted (a row's
       `repositories` names the project), done, "ORG/NAME is in PROJECT",
       "Chats can change its code now.", with no claim that CI passed, since
       the forge rows supply no check result; ORG/NAME the project's first
       adopted (5.3), one line on a laptop; with the rows' copies all in,
       done, "acme/widget is in Widget", or "acme/widget, acme/other are in
       Widget", its line "Chats can change its code now, on your Forgejo;
       nothing goes back where it was copied from.", or "…their code…
       where they were copied from."; while a row's `makings` holds a copy
       into the project, not done, "Copy" and a line per copy in the order
       asked: "Copying acme/widget from github.com…" for the one running,
       "Waiting to copy" for one after it, "Adopting acme/widget into
       Widget…" for one made, "acme/widget is in Widget", ticked, for one of
       the rows' in, and for one failed the forge's words and "Copy again",
       which asks it once more; an empty one the sheet below asked says "Add
       a repository" and "Making ORG/NAME…" in the same way, and a repository
       the sheet asked leaves the lines once in and reads as any in the
       project, since the rows' intent alone names copies; with the rows'
       copies not all asked, after a restart between Ready's project and its
       copies, not done, on the Home that is open, since the rows don't say
       which project Ready made, "Copy ADDRESSES" with "Copy", filled, one act
       for them all, under Ready's keys so that the engine answers a copy
       already asked with its first answer, an address being asked once a
       making row or the forge's repositories holds its org and name; while
       Ready's adoption is
       out, not done, with no act, "Adopting ORG/NAME into PROJECT…",
       "Chats can change its code once it is in."; with a first
       repository to adopt, not done, "Adopt a repository", "ORG/NAME,
       made on your Forgejo at set up, is in no project yet.", the
       refusal's words after it (4.7) when Ready's adoption was refused,
       with "Adopt", filled, which sends that adoption again; with none,
       on GitHub, not done, "Adopt a repository", "temper’s App reaches
       its repositories on ACCOUNT.", ACCOUNT the first accepted
       installation's, with "Adopt", filled, to `Adopt { project }`,
       whose picker is the mock's; with none, on an owned or managed
       Forgejo connected, not done, "Add a repository", "Your Forgejo has
       none yet. Until one is in, chats talk and plan but change no code.",
       with "Make an empty one" and "Copy a public one", small, and the
       quiet "Adopt another…", to `Adopt { project }`; otherwise, not done, "Adopt a
       repository", "Until one is in, chats talk and plan but change no
       code.", with "Adopt", the same. The adoption being out is the
       session's fact: after a reload it reads as a first repository to
       adopt until the rows say it is in, and a first "Adopt" names
       Ready's key, so that the engine answers the one adoption.
       *The sheet of "Make an empty one" and "Copy a public one"* is the
       page's state, not an address: a modal sheet with the look of
       Forgejo's on the welcome (*Forge,* above), "Add a repository to
       PROJECT", the muted "Made or copied on your Forgejo, then in this
       project at once.", the legend "A repository", "An empty repository"
       with its two fields and "Copies of public repositories" with its
       addresses, opened at the one pressed; "Cancel", also Escape; and the
       primary "Add it". Its fields are read and its alerts said as the
       welcome's are. It sends `MakeRepository` `into { project, home: true
       }`, the first of several the home, under keys of its own; the step's
       lines above say the rest, with no toast. It is for this state alone:
       `../draft/repository-copies.md`, 9's "Add repositories" stays
       Settings'.
    3. "Start a chat", "Ask a question or ask for a change.", with "New
       chat", to `NewChat`. Done once the project holds a chat
       other than a configuring one, live or ended, as `Chats` reads
       them; done, its title alone.
  - *"Skip"* sends the FORGE request `Dismiss { to_begin(project) }`
    (4.6), one act, one request (3.6), and the card goes at once, while
    it is out and for as long as the person's dismissals name the
    project, so another device's Home drops it when the skip commits.
    Refused, or lost, the card comes back with "temper couldn't keep
    that. Try Skip again." under its line; *unknown,* the subject
    answers it, and nothing is said. A skip is never undone, and nothing
    is lost by it: the first and second steps' acts stay in Settings, the
    third's in the rail.
  - *Until it is known:* To begin waits for the facts it reads. Until the
    person's dismissals, the `forges` rows, the project's chats and the
    `settings` document are delivered, it is left out, so that a project
    skipped or done never flashes it. A `forges`, chats' or `settings`
    watch refused or ended counts as delivered, its step left; the
    `dismissed` watch refused or ended leaves To begin out, since
    nothing then says it was not skipped.

  **Home on any project,** To begin gone, skipped or with every step
  done (the mock's everyday `home.html`; section 11 holds what it draws
  that no client is given). The page watches the subject `status` (4.6)
  while open and sends `Looked` once on entering it, not while To begin
  shows; once To begin goes, by a skip or a last step, that entry's look
  is sent. Until the look is answered and `status` is delivered, or with
  its watch refused or ended, the intro has its kicker's day and time and
  its headline by the needs alone, and the foot is not drawn:
  - *The intro,* one function's composition, so a later part of it
    joins there:
    - *the kicker:* the day and time, then, with a `previous` over a
      minute old, " · you last looked SPAN ago". SPAN is its own word,
      not 5.12's short span: "a minute", "2 minutes", "an hour", "9
      hours", "a day", "3 days", each rounded down;
    - *the headline:* the landed sentence, then a space and the needs
      sentence, by the project's open cards (*What the client keeps,*
      below): "Nothing needs you.", "One thing needs you.", "Two things
      need you." and so on in number words to "Nine", digits after,
      "N+ things need you." where the read is cut, the
      project's name until its needs have answered. With N as `landed`
      less `seen`, and a brief new to the person where its `landed` is above
      `seen` or its `at` is after `previous`, the landed sentence is a new
      brief's `sentence`; else "One change landed since you last looked." or
      "N changes landed since you last looked."; for 0, with `landed` above 0
      and a `last_landed`, "The last change landed SPAN ago.", SPAN the
      kicker's word, "just now" under a minute; nothing with nothing landed.
      So a landing never stands under "Nothing needs you." alone, whatever
      became of the brief: a refused, failed or unfunded run isn't the
      person's error, and Home has no word for one. "overnight" is the status
      agent's word, never the computed sentence's;
    - *the lede and the byline:* the brief's `lede`, if the brief is new as
      above, then "Written by the status
      agent at 07:42", its `at` by 5.12's *clock*, in `temper-home-byline`,
      12 px in the faint colour. Otherwise neither: the computed Home has
      no prose of its own.
  - *Needs you,* a region named "Needs you": what needs the person in
    this project, the groups of `Inbox` but its updates, read as `Inbox`
    reads them filtered by the project, with the project's changes waiting
    on a review on the forge (*Needs you,* below, *An approval*). Its head is the level-2 heading
    "Needs you", "Needs you · N" in the signal colour while something
    does, and, at its right on a wide page while more than one does,
    "Answer in a row", to that filtered *Needs you*. Empty, once its read has
    answered: "Work carries on, and this page tells you what changed when you come back.", the
    headline above having said "Nothing needs you." once.
    - *Answered lines,* under the head: what was answered in the last 2
      minutes (*What the client keeps,* below): the mock's `.answered`,
      a tick by style, "TITLE:" a link to its task, the label, and at
      the right "Undo" while its answer is held.
    - *The first,* a card (`temper-ask`, the mock's `askCard`): a small
      mono line, the asked task's link as the inbox's card has it, its
      kind and age ("Add a README · a question · 4 min"), " · later" on
      a card marked later; then its parts, by its kind as on *Needs you*
      (below), but: the question in `.q`, 17px; a question's `.choices`
      in a row, min-height 40, then quiet "Reply" and quiet "Later", in
      place, the reply form under them; on a proposal or a held task its
      acts, quiet "Open" and quiet "Later"; on an approval its review link
      where it has one, then quiet "Open" and quiet "Later" (`forge.md`, 4.1).
    - *The others,* a line each under it, the whole a link that opens
      *Needs you* at that card, at most five: its SHORT at the left
      (*Needs you,* below; a funds row "TITLE: S of L used"), at the
      right its kind and age.
    - *On a phone* the card and the lines give way to one block on a pale
      signal ground, "TITLE: SHORT", the whole a link to *Needs you*,
      with "Answer" across it for one card and "Answer, one at a time"
      for more; the browser's alone.
    - *While To begin shows* (the mock's `freshHome`): the card alone,
      with no lines under it and no "Answer in a row".
  - *Chats,* a region named "Chats": its head the level-2 heading
    "Chats" and, at its right with any chat, "All N", named "All N
    chats", "All N+" where the rows are cut, to `Chats`; a line per chat
    at most five, live by last activity and then ended, each a link to
    its `Chat`: its title, then at the right its state and age,
    "waiting on you · 4 min", "closed · 1 h", any other state as `Chats`
    words it, the age as 5.12's span. Empty, once its reads have answered:
    "No chats yet." With no goal
    in the project (`Domain::has_goals`), once the chats are delivered,
    after its rows, in the foot's face: "A chat suits one change. For
    bigger work, a chat can write a draft beside it and start a goal from
    it: a plan, its tasks, landing."; and "Show goals", to
    `Settings`, once `Settings` has that switch, not before.
  - *Goals,* in Chats' place with a goal in the project, a region named
    "Goals": its head the level-2 heading "Goals" and, at its right, "All
    N", to `Goals { project, filter: All }`; then `Goals`' row (*Goals,*
    below), live first and then ended, as many as the larger of 5 and the
    live goals. Its reads are that page's.
  - *The foot,* `temper-home-facts`, after the last section, 13.5 px and
    muted: "Landing: ", then the landing's line (*Changes,* below), the
    same function's, from `queued`, `last_landed`, `branch` and `red`:
    "3 queued · last landed 6 min ago"; "main last moved 1 h ago ·
    nothing queued"; "nothing has landed yet"; then " · main red" in the
    signal's colour while `red`; then " · " and "Changes", a link to
    `Changes`. BRANCH is `branch`, "landing" where it is empty. No green
    time is kept, so none is said.

  In the rail and the tab bar, "Needs you"'s count takes the signal
  colour.

  What Home does not draw: the composer and the full chats' list, which
  stay `Chats`'s, and the composer's picker. With the
  `forges` watch refused or ended, step 2 is "Adopt a repository" with
  no ORG/NAME. In the terminal the headline, To begin with its three
  steps, and the sections of Home on any project keep their headings,
  each line one readable run of words; the kicker, the headline and the
  lede read as one run of words, the foot is a paragraph, and a question's
  choices print as buttons.
- **Needs you** (the user's greenfield mock's `needs.html`; `needsPage`):
  `Inbox { filter }` in the browser, its address and its filters as they
  were (`browser.md`, 2.3). It draws one card at a time: no level-1
  "Inbox", no sections, no filter menus and no "+ New chat". Updates
  leave the browser's page and no page replaces them; `Seen` marking
  stays as it is. Its unfiltered address, `/`, works across projects,
  the project's name before the owner on the mono line. One card is
  shown of the open ones (*What the client keeps,* below): the one a Home
  row asked for, else the first; until the inbox has answered, the column
  is empty, so that "Nothing needs you." never flashes, and what it said
  stands through each refresh after that.
  - *The column,* as the mock's `.col.narrow.flow`: 600 wide, centred,
    gap 32.
  - *Its head,* `.flow-head`: `.dots`, a figure named "2 of 3", a pip per
    item, 22 by 4, radius 2, `--gf-line-2`, the done ones first and ink;
    at the right "2 of 3" as `.kicker`. The total is what is answered of
    this session's page plus the open; where the read is cut, "2 of 3+".
  - *The card,* on no ground (`.ask`, no background, no shadow, padding
    0), gap 16:
    - a `.mono` line: TITLE, a link to the task, · kind · age, as Home's,
      " · later" with it; the kind "a question", "a proposal", "held", "a
      person task", "an approval", or "funds" for a held task whose reason is
      `HoldReason::Budget`;
    - `h1.flow-q`, the question line: 26px/1.35, 500, -.01em; 23px on a
      phone;
    - `.ctx`, `--gf-ink-2`, if it has a context;
    - a disclosure "What the agent saw", if the agent said: its summary
      width fit-content, 14px muted, its paragraph margin-top 6; closed
      at first;
    - `.choices` in a column: min-height 54, padding 10 18, radius 12,
      16px; the recommended one, with none the first, primary, and on a
      question alone `.rec`, "recommended", at the far right, 12px, 400,
      opacity .72;
    - `.pair`, two columns, gap 10: "Reply…" and "Later", min-height 46,
      radius 12, no border, `--gf-hover`, centred; on a proposal, a held
      task or a person task other than one in words the first is "Open",
      to its task's or chat's page, and with no such page it is left out;
    - the reply form under the pair, hidden until "Reply…": a textarea,
      min-height 72, `--gf-line-2`, radius 8, `--gf-paper`, its
      placeholder "Your words go to TITLE’s agent"; "Send reply" (primary,
      small) and "Cancel" (quiet, small).
  - *By its kind.* A question reads its words by their line form
    (`../domain/agent.md`, 8.2); words in no such form are answered by
    "Reply…" alone. A proposal's question line is its reason's first
    sentence, without a heading's marks, its facts and rows as `.ctx` beneath, then Accept (primary,
    with no "recommended") and Reject (plain, its reason's textarea as on
    the terminal's card), through the confirmation they have (3.6). A
    held task: its title and its words, its facts as the terminal's card
    has them, then Release (primary) and Leave held (plain, its reason's
    textarea as Reject's). A funds card: "S of L used." as the
    question, in the task page's money form, plain numbers, its title as
    owner, then Release and Leave held; there is no "Add $10". Its hold's
    words say what a person can do: accept a proposal for more budget, if one
    waits, then release it; otherwise close it and start the work again once
    their pool has room, which closing an open chat makes. A person task: its
    title, its spec, its acts as plain buttons. *A person task in words,*
    one addressed to a person with a report as its contract, as a goal
    asks the person who started it (`../domain/agent.md`, 8.2), reads as a
    question: its spec's words by the same line form, its kind "a
    question", its choices, "Reply…" and "Later". Choosing or replying
    answers it in those words, naming the revision the card showed;
    Later keeps it open. With a task as requester, its line's TITLE is
    "Task N", N its requester, a link there, since the card reads no second
    task; another requester keeps the person task's title and link.
    Its answered line's title is its own. In the inbox's filter it stays
    a person task. On the terminal's inbox, Answer sends its words with
    the shown person revision. A role's task or a contract other than a
    report keeps the ordinary person-task card.
  - *An approval,* a change waiting on a review on the forge: a row of the
    subject `approvals` (4.6), its kind "an approval", its question, context,
    review link, "Open" and "Later" as `forge.md`, 4.1 has them, its key
    `ObjectKey::Task` of the change, so that Later works on it as on any card.
    The open cards are the inbox's needs and these rows together: the inbox's
    first, in its order, then the rows in theirs, and those marked later last.
    A filter by project takes the project's rows, one by goal the rows of the
    changes under that goal, and one by kind none, since no kind of the
    inbox's is theirs. They join once the inbox has answered, and until the
    `approvals` watch has delivered there are none. On the terminal's inbox
    there are none either.
  - *Whole,* since a person decides on these words. The engine's read of a
    person task cuts its words at `documents.text_bytes`, so a card whose words
    are cut reads the rest through `TaskText`, as a goal's page does (5.8), up to
    `long_text`, which bounds a task's words (5.4). Only the whole parses by the
    line form, so its context, "What the agent saw" and its choices draw once it
    has it all; until then the card reads as words in no such form, ending in
    `…`, answered by "Reply…" alone. A read that ended short is asked again at the
    inbox's next refresh. Words with more than 16 choices read as words in no such
    form, whole. A question by `message` has no such read: its words stay as the
    document has them.
  - *Cut,* what a card says while its words stay short of the whole: a person
    task's whose read ended, a `message` question's cut by the document's bound.
    Under the words, in `.ctx`: "This question is cut here." and the link "Read
    the rest", to its task's page (the requester's, for a person task in
    words), whose conversation holds them. No question is cut without it.
  - *SHORT,* what a next-up line or a Home line says of a card: a
    question's line, or a person task in words', from the document's bounded
    words; a proposal's reason's first
    sentence; a held task's or another person task's title, a funds card's
    "S of L used"; an approval's question.
  - *`.next-up`,* centred, 13px, `--gf-faint`: "Next: TITLE · SHORT", or
    "The last one".
  - *Answered,* under it when any: the section title "Answered", then
    the `.answered` lines.
  - *Done,* with none open, `.col.narrow` with no head: the kicker "Needs
    you", the level-1 heading "Nothing needs you.", the lede "Every
    question is answered. Work carries on; Home tells you what changed.",
    the `.answered` lines, and a plain button-link "Home" (to the open
    project's `Home`, else the first project's).
  - *Its acts.* A choice answers and moves to the next card. "Send reply"
    answers in words. "Later" marks it and moves on, the toast saying
    "Later: it stays in Needs you, after the others."
- **What the client keeps** of what needs the person: in `Pages`, not on
  a page, for this session, gone on reload or on sign-out, none of it a
  stored record, and at most 64 each of the cards marked later, the answered
  lines, the refused answers and the proposals of settings, the oldest going
  first, a line still held never:
  - *Later,* the cards marked later. *The open cards* are the needs'
    groups as `Inbox` reads them, less those answered, in the inbox's
    order, those marked later last.
  - *Answered,* a bounded list: each card's object, project, title,
    label and when. The label is the choice's words, or `Replied: “…”`,
    its words cut at 57 characters plus "…" past 60. A line shows on
    *Needs you* for 10 minutes and on Home for 2, and an answered card
    leaves the open ones at once.
  - *At,* the card a Home row asked for, until it is answered or marked
    later, or the person leaves the page.
  - *The held send.* jig has no retraction, so a choice or a reply is
    held in the client for 5 seconds, a deadline of the domain's as
    `next_deadline` has. The card moves to answered at once, with the
    toast "Answered: LABEL" and its "Undo". Undo restores the card, a
    reply's words with it, unseen until the form opens, and sends
    nothing. At 5 seconds the `Answer` goes, its words the choice's
    or the reply's; after that its line shows no "Undo". A refused answer
    puts the card back with the refusal's words on it, as does an answer
    the engine did not keep. A tab closed inside the 5 seconds sends
    nothing and the question stays open. Accept, Reject, Release, Leave
    held, Take and Hand back send at once, with no Undo, and leave no
    answered line.
  - *A press stays under the finger.* When the inbox's document changes,
    a card still in it keeps the page it shows until its new read answers,
    and keeps it if that read fails. At a filtered address the cards stay
    as they are until the section reads have answered, a section whose read
    failed keeping its own. So a choice on the page stays pressable, in the
    same element, through the read: nothing a change of the inbox does draws
    a card without its choices, or takes one away, but the card leaving the
    rows. A press of a drawn choice that a bound refuses (its words, or 64
    answers already held) moves nothing and says so, in the toast "That
    answer wasn't sent. Press it again.": an answer never goes unsent with
    nothing shown.
  - *The toast,* one at a time, 4.5 seconds, with one optional act; drawn
    with the status role at the end of the frame, in a region named "Notices"
    (skein's `docs/design/ui.md`, 14: a page's child is a landmark) that has
    no look of its own, in the mock's `.toast`:
    fixed, centred, bottom 28 (88 on a phone), an ink ground.
  - *The rail's count:* one standing `inbox` watch while a project's
    frame shows, the rail being what reads it, so that a client on no
    project's page asks nothing more; its needs' rows counted by the open
    project, less those answered, a row counting in each project it shows in
    (jig's `client/domain/contract.md`, 5.1: `shows_in`), so that a proposal of
    settings counts in the projects it belongs to, and for one of the
    deployment's in each the person owns; "N+" where its document is cut, and no
    count before it is delivered. A card marked later still counts. The open
    project's rows of `approvals` add to it, "N+" where that document is cut too.
    `me`'s `needs_total`, with every row of `approvals` added, stays the banner's
    navigation's.
  - *The approvals,* the forge side's watch of the subject `approvals` (4.6),
    held for the session as `dismissed` is, from when `me` is first delivered
    while the link is live: a watch refused or ended is kept as such, with no
    rows, and not opened again until the next sign-in.
- **Changes** (the user's greenfield mock's `changes.html`; `forge.md`,
  section 3): `Changes { project }`, at `/projects/<project>/changes`, a
  project page in the rail's frame, a column as Home's, at most 720 px
  wide and centred, its parts 60 px apart, 32 on a phone. BRANCH below is
  the name of the one branch the queues go into; with none queued, the
  document's `landing` (4.6); with more than one, or no name to say, the
  heading says "landing" and the lede "its branch". In order:
  - the kicker, "PROJECT · landing";
  - the heading, at level 1: "Nothing is on the way to BRANCH." with none
    queued; "One change is on the way to BRANCH."; from two to nine the
    number in words, "Three changes are on the way to BRANCH."; from ten
    in digits, "12 changes are on the way to BRANCH.";
  - the lede: the landing's line, then ". Each change is checked on top
    of BRANCH before it lands.". **The landing's line,** one function's,
    its parts joined by " · ": with something queued, "3 queued" and, once
    anything has landed, "last landed 6 min ago"; with nothing queued and
    something landed, "main last moved 1 h ago" and "nothing queued"; with
    neither, "nothing has landed yet"; then, for each branch `red`, "main
    red", in the signal's colour. The times are 5.12's *ago*. No green
    time is kept, so none is said;
  - "Queue", a section, with its rows: at the left the change's title, a
    space and, in the mono face, where it came from; at the right, on the
    first, "checking" in the good colour and " · checks started 6 min
    ago", by 5.12's *ago*, and on the others "queued 2nd", "queued 3rd",
    "queued 4th". Past the document's rows, a last row, not a link, "and
    N more". With no rows, "Empty.". With several branches holding rows, a
    section each, "Queue · owner/name BRANCH", by the document's order;
  - "Landed recently", the same section: the title, where it came from,
    and at the right its time by 5.12's *clock*, newest first. With no
    rows, its rule alone.

  *Where a change came from:* the title of its task's `goal`, the chip
  in its `lineage_chips` naming that task; else the root of its
  `lineage_chips`, "chat T6", where that root is on the charter of the
  project's chat template; else nothing. The page reads each row's task, as a task's
  page reads a chip, those of the rows shown alone; until one arrives,
  its row has no mono part. Every row but "and N more" is a link to its
  change's `Task { task }`. Until its first document, the page is its
  kicker; with its watch refused or ended, its kicker and "Shown on the
  forge." (4.2). The terminal prints the page; the view is shared.
- **Settings:** a project's settings are tabs, each a project page in the
  rail's frame, "Settings" current, in one `.col` as Home's (720 wide, gap
  60; 32 on a phone):
  - *Its head:* the kicker "PROJECT · settings", then, by tab, the level-1
    heading and the lede:

    | Tab | Heading | Lede |
    |---|---|---|
    | How work runs | How work runs in PROJECT. | Everything the other pages do for this project comes from here, so each project, and each person, can work its own way. |
    | Repositories | What PROJECT works on. | The repositories chats and goals in PROJECT read and change. |
    | Forges | Where your code lives. | Each forge temper is connected to, or setting up. Add one to bring its repositories to temper. |
    | Accounts | What agents run on. | The LLM accounts temper’s agents run on for you. A task keeps its account: while the account is limited, the task waits for it. |
    | Models | What the picker offers. | The models your accounts can run, by what pays for them. Each model can be left off the picker; a task keeps the model it has. |
    | Builders | Where agents work. | The machines temper’s agents run on, each with its slots and how many are in use. |

  - *Its tabs:* a navigation named "Settings", links, the current one
    current, wrapping on a phone: "How work runs", to `Settings { project }`, at
    `/projects/<project>/settings`; "Repositories", to `Repositories {
    project }`, at `/projects/<project>/settings/repositories`; "Forges", to
    `Forges`; "Accounts", to `Accounts`, these two with the marks 5.3 gives
    them; "Models", to `Models { project }`, at
    `/projects/<project>/settings/models`; and "Builders", to `Builders {
    project }`, at `/projects/<project>/settings/builders`.
  - *`Forges` and `Accounts`* keep their addresses and paths, so that a
    connect's `back`, To begin's links and their tests hold. Whenever the
    page has a project, as the rail does (5.3), they draw inside this frame,
    with no level-1 heading of their own and no page bar; their rows follow
    the tabs. With no project they take the same column, sections and rows
    under their own level-1 heading, "Forges" or "Accounts", and that tab's
    lede, with no kicker and no tabs. Choosing a project on them changes
    nothing but the words naming the project.
  - *Sign out* is the last row of the `Accounts` tab: "Signed in as NAME",
    its login in small type, and "Sign out"; with "Sign out did not reach
    the engine." beside it as 6.6 says. A browser draws this row at 760 px and
    under alone, where the rail has no foot; wider, "Sign out" is the person's
    menu's (5.3). With no project, the banner has them (5.1).
  - *How work runs* is 5.13.
  - *A repository being made,* on the `Repositories` tab, is a row among them,
    by the forge's row's `makings` whose `into` names this project (4.6), the
    request `connect-flows`: ORG/NAME, then "Copying · FORGE", or "Making ·
    FORGE" for an empty one. A failed one: "Copy failed · FORGE", or "Make failed
    · FORGE", then its own line, "Forgejo said: “WORDS”", with "Copy again", which
    opens the sheet (4.4) on Copy public with its address filled, or, for a name
    taken, "Adopt it", the sheet on Adopt with its row picked, and "Dismiss"
    (`DismissMaking`, 4.6).
  - *Repositories:* a section whose head holds the title "Repositories · N"
    and "Add repositories", to `Adopt { project }` (4.4); then a line per
    repository of the project, from the `forges` watch's rows whose
    `project` matches: `owner/name`, in the browser the repository's link
    where it has an address (`forge.md`, 5.4, *A repository's link*), and in
    the line's time place its forge's name. The line itself is no link. The
    playground's project shows its repository from the playground's read
    (4.8), plain. With none, a quiet band:
    the tag "No repository yet", "Chats in PROJECT talk and plan, but change
    no code until a repository is in.", and "Add repositories".
  - *The rows,* on `Forges` and `Accounts`, are the greenfield mock's language
    applied to what the client has, since its tabs draw neither: every fact,
    act, form and state of 4.8, 5.5 and `forge.md`, 5.4 stays, and only its
    look is this. Each is a folded disclosure in `.lines`' place: a hairline
    under it, 14 px 4 px, no card and no shadow. Its summary is its name in
    ink, 15 px, its second line in `small`, 13.5 px, muted, and "⌄" at the far
    right, faint, as `.switcher > summary::after`. Opened, its content is 14 px
    under the summary, indented nothing, its facts `.setting` rows. A section's
    head is as Repositories': `.section-title`, and the section's act at its
    right, a `.btn.small`. Buttons are `.btn`, `.btn.primary`, `.btn.quiet` and
    `.btn.small`; fields `.text`. The tabs' marks stay.
  - *The `Forges` tab:* a section, "Your forges · N", with "+ Connect a forge" in
    its head, not while the playground runs. Until the watch answers, "Your
    forges" alone, claiming nothing. With none, and the playground not running, a
    quiet band: the tag "No forge yet", "No forge of yours yet.", and the
    primary "Connect a forge". Then a row per forge (`forge.md`, 5.4). While the
    playground is read, a section of its own holding its forge's row.
  - *The `Accounts` tab:* a section, "Your accounts · N", with "+ Connect a
    model" in its head, as 3.5's accounts page has its act and condition. Until
    the document is read, "Your accounts" alone. With none, a quiet band: the
    tag "No account yet", "No model of yours yet.", and, not while the playground is read, the
    primary "Connect a model". Then a row per account, its summary its line (5.5, An account's
    line). "Deployment accounts", for an owner, is a section of the same rows;
    the playground's account, a section of its own; then *Sign out*.
  - *The `Models` tab* (the look request `model-picker`, its 07;
    `../draft/model-catalogue.md`, sections 3, 4 and 6) draws the project's
    offer in `me` and works nothing out, so that it holds no model the offer
    leaves out:
    - *A section per provider* `me` offers, in `me`'s order, its title the
      provider's word, "Claude" for `anthropic`, as the picker's headings say it.
      Under it, in `small`, what pays and where its models came from: "Your Claude
      subscription.", "Your ChatGPT plan." or "Your API key.", from the person's
      usable accounts of that provider in the accounts document, one clause each,
      then one clause on its list, from those accounts' `list`s (jig's
      `client/domain/contract.md`, 5.1), PROVIDER its word as the section's title:
      - where one's last attempt failed, "Couldn't read PROVIDER's list · 2 hours ago.",
        by the newest `tried` of those failed;
      - else where one has a good read, "Read from PROVIDER 2 hours ago.", by the
        newest `read`;
      - else "These come with temper."

      Where any of them has a `list`, "Read again" follows the clause, in
      `.temper-settings-small`: a press asks jig's `ReadLists` with the section's
      provider, says "Reading…" and is off until that answer and the `accounts` watch
      has moved past what it showed, and a refusal shows under the clause in the
      words of an account act's refusal.
    - *A provider with no model offered* has its section's title and its line alone,
      as the picker's: "Connect a plan or key ›", to `ConnectModel`, with no account,
      or the account's own fix, such as "Sign in again ›", to `Accounts`. It has no
      row and no switch, the stock's models included. So with one account the page
      holds that provider's models alone.
    - *A row per offered model,* `.setting`, in the offer's order: its shown name;
      in `small`, "NAME · EFFORT", its name and default effort, with no effort for
      a model that declares none, then " · the default" on the model the chat
      template's `resolved_default` names, " · added by you" on one a person
      added, " · estimated prices" on one whose prices are estimated, and " · new" on
      one whose `new_until` is later than now; at its
      right, "On" or "Off", whether the picker shows it. An owner in the deployment
      has a switch beside the word, named "SHOWN in the picker": a press asks jig's
      `ChangeModel` (jig's `client/domain/contract.md`, 3.2), the switch holding
      its old state until the answer, and a refusal shows under the row in the
      words of a change's refusal. Anyone else has the word alone.
    - *How many were left out,* in `.facts` under the sections, where `me` says
      the catalogue left any out: "N more models don't fit the catalogue." (jig's
      `client/domain/contract.md`, 5.1).
    - *"Add a model by name",* a section, for an owner in the deployment: a
      provider's select, of the providers `me` offers but the playground's; a
      field whose accessible name is "The model’s name", drawn by its placeholder
      "model-name" alone; "Add", the three on one row at every width; and, in
      `small`, "For a model the list doesn’t show. It runs through the account you
      have with that provider." The name's shape is checked here as the engine
      checks it, 1 to 128 bytes of letters, digits, `-`, `.`, `_` and `:`, and "Add"
      is off until it holds; a name of that provider's in the offer says
      "PROVIDER already has that model." A press asks `ChangeModel`. The engine
      makes the model (jig's `domain/engine.md`, 17.11), and the field empties;
      its row then reads "added by you". A refusal shows under the form.
    - *The last line,* in `.facts`: "To change a default or a role’s models, tell
      Configure temper.", its last words a link to `Configure { project }`.
    - *In the terminal,* the same sections as lines, a row's word and name on one line,
      and the switch and the form as the terminal has them on How work runs (5.13).
  - *Builders* reads jig's `deployment` document, watched while the tab is open
    by a person with a role in the deployment, as `Accounts` watches it for an
    owner. It draws `Deployment.hosts`, as far as the document's configured rows
    hold them. A section, "Builders · N", N its rows; then a line per host, as
    Repositories' are, no link and no act: the host's name, or "Builder
    CHANNEL" with none; in `small`, "2 slots · 1 in use" ("1 slot" for one); in
    the line's `.when`, "Not connected" while it is not connected, else
    "Draining" while it drains, else "Connected". Under them, `.facts`: "Each
    agent’s role names which builders it runs on. Tell Configure temper to
    change it." With no host connected, `.empty`, "No builder is connected.",
    above any lines left. A cut document says so, as `Accounts` does. With no
    role in the deployment, one line, "Builders are seen by the deployment’s
    owners.", and no section. Until the document is read, the title "Builders"
    alone, claiming nothing. There is no capacity graph. The word on the page
    is "builder"; the code's stays host.
  - *The flows' paths:* `ConnectForge` and `ConnectModel` share the
    welcome’s cards and panels in Settings’ column. With a project open,
    the browser draws PROJECT / Settings / Forges / Connect a forge, or
    PROJECT / Settings / Accounts / Connect a model. Links go to the
    project’s Home, Settings and the tab; the last part is plain. Inside
    a way, the path also links back to the choices, with the way’s name
    last. The quiet Other ways or Other forges stays beside the main act.
    On a phone the path’s “‹” links back to the choices inside a way, or
    to the tab at the choices. With no project there is no path, and the
    terminal uses its rail’s Settings. `Adopt` draws the Add repositories
    sheet over Repositories, whose path remains the tab’s (4.4).
    A flow’s completion returns to its tab; Start over on a forge cut
    short returns to Forges.
  - "Settings" is current on every tab, on `ConnectForge`, `ConnectModel`,
    `Adopt` and `Configure`, and on a configuring chat. A project's Delete,
    when it is built, is on How work runs (section 10).
- **Away** (the greenfield mock's `home.html`, `phone.html` and
  `settings.html`'s `#away`; `../domain/forge.md`, 5.8 and 5.9): a person's
  switch on a project, while which the goals they asked for decide for
  themselves what its kinds allow and show it afterwards. It is the
  rail's, Home's and Settings', and the mock draws nothing of it in Needs
  you, nor does the client.
  - *Shown* while any holds, one function's answer (`Domain::away_shown`),
    so the rail and Settings agree: the project has a goal
    (`Domain::has_goals`), the party's row there is on, or the project's
    `Ways.away` (5.13). A fresh project has no switch, as the mock's first
    run.
  - *The control,* in the rail's foot above Settings, a button, `State::Pressed` while on,
    in `temper-away`, the mock's `.away`: 8 below it, full
    width, 8 10 padding, 1px `--gf-line`, radius 8, `--gf-surface`, 13.5 px
    muted, a 30 by 18 track holding a 14 px knob, by style; on, the ink ground,
    white words, the knob at the right. Its words: "Away" off; "Away until
    08:00" on, `until` by 5.12's *clock*; "Away" alone on with `until` none.
    On a phone the foot is hidden: a second button, `temper-away-chip`, the
    mock's `.away-chip`, at the top bar's right (5.3) beside the project switcher,
    min-height 36, radius 18: "Away: off", "Away until 08:00", "Away" alone
    on with `until` none; the browser's alone. In the terminal the rail's
    button, numbered as any. While its request is out it is disabled.
  - *Its act* sends `SetAway` (4.6): to off, `on` false, the row's `until`
    and `kinds`; to on, the row's `kinds`, or 7 with no row
    (implementation, retry, wording), and `until` none where the row's is
    none, otherwise the next 08:00: the first wall after now whose hours
    in the person's zone (5.12) read 08:00, computed by one pure function,
    `forge::away::next_eight`. A row's `until` once past means nothing but
    that the choice is 08:00.
  - *Past `until`* agents are told to ask (`../domain/forge.md`, 5.8) and
    the control draws off, with no write: the build asks to wake at
    `until`. `Domain::away_on` holds the rule, the row and the wall.
  - *Its toasts,* Needs' toast slot (*What the client keeps,* below), on
    the request's answer: on, "Away until 08:00: agents decide within
    PROJECT’s limits." with the act "Limits", to Settings; "Away: agents
    decide within PROJECT’s limits." with `until` none; off, "Away is off:
    questions wait for you again."; refused, "Away didn’t change. Try again."
  - *Home, while a decision shows.* A row (`decided`, 4.6) shows for 24
    hours from its `at`. With any showing and To begin gone, *Home on any
    project* takes this form, in the intro's one function; the build asks to
    wake when the oldest row stops showing:
    - *the kicker:* "While you were away · 23:10 to now", or "· 23:10 to
      08:03" once off, by 5.12's *clock*: `since`, then `off`, or `until` once
      past. With its row undelivered, "While you were away" alone;
    - *the headline:* "57 changes landed." where `landed` less the row's
      `landed` is above 0, "One change landed." for 1, nothing for 0 or
      with the row undelivered; then "Four things were decided for you."
      in number words to "Nine", digits after, or "One thing was decided
      for you." in place of the needs sentence. The lede and the byline as
      *Home on any project* has them;
    - *"Decided for you · 4,"* the first region after the intro, every row
      showing counted. A `section-head`, with "Limits" at the right, to
      Settings. `temper-decided` (the mock's `.decided`, a top rule), a
      `temper-decision` a row (`.decision`), at most 7, the newest kept,
      oldest first: mono "TITLE · 01:05", TITLE the task's, its `at` by
      5.12's *clock*; muted, the question; `temper-decision-took` (500), what
      it took; `temper-decision-why` (14 px, muted, none on a phone), why. At
      the right, by the first that holds:
      - the task ended: `temper-decision-late` (`.late`), "landed 03:12 · a new
        goal can undo it", its ended time, "ended 03:12 · a new goal can undo
        it" for one that did not end done;
      - overturned: `.late`, "Overturned: it asks you again";
      - the task live: "Overturn" (`.btn.small`);
      - its read not delivered, or refused: nothing, TITLE reading "Task N".

      An overturned row's took is struck through, in `--gf-faint`
      (`temper-decision-overturned`), ended or not. Each shown row's task is
      read (`Read::Task`), as the inbox reads its cards, within `reads`;
    - *Needs' head:* "Still needs you · 2", or "Still needs you" with none,
      and the rest of *Needs you* as it is.

    Empty rows, the switch off and old: Home is as *Home on any project* has
    it. Nothing of this is drawn while To begin shows.
  - *Overturn* sends `Overturn` (4.6). Answered, the client sends the
    task its existing message (`Ask::Message`), in words: "I overturn your
    decision. QUESTION You took TOOK. Ask me again with message, as a question,
    and wait for my answer."; sent, the toast "Overturned: its goal asks you
    again, under Still needs you." The goal's question comes as any other. A
    refused message (the task ended between) leaves the row marked, and the
    toast "It has ended; a new goal can undo it." `Overturn` refused,
    overturned or unknown: no message, the toast "That was overturned already."
  - *Settings' section,* `#away`, on *How work runs* after *What needs you*
    (5.13), while the switch is shown: `section-title` "While you’re away,
    agents may decide"; a `.setting` "Kinds of decision", small "Each shows
    afterwards as decided for you, with Overturn.", `.checks`:
    "Implementation choices within a task", "Retries and splitting a failing
    task", "A design document’s wording", "Adding a dependency", and locked and
    unticked "Budgets, sign-ins, machines, what is hard to reverse: always
    wait"; a `.setting` select "Away until", small "When it isn’t turned off by
    hand.", options "08:00, the end of quiet hours", "I turn it off". Each writes
    through `SetAway`, `on` as the row stands, the select's first choice sending
    the next 08:00 and its second none. They work while off: the row keeps them.
    With no row they read the defaults: the first three ticked, 08:00. "Limits",
    on the toast and on Home, opens Settings with this section focused, so a
    browser brings it into view: `Address` holds no anchor, and the page takes
    the request from `Pages` once.
- **Goals** (the greenfield mock's `goals.html`), at `/projects/<project>/goals`,
  in the project's frame, is the project's goals, the filter in its address:
  `GoalsFilter`, All, Needs, Live or Ended, whose query is `?state=needs`,
  `?state=live` or `?state=ended` (`browser.md`, 2.3). Choosing one is a move.
  A goal is a tracked agent's task (5.8) at its own `Task` address, so that
  Goals is the list and draws no goal page. Its parts, in order:
  - **Its intro:** "fabrig · goals", the project's name then the word in
    lower case; the level-1 heading: "5 goals, 3 under way.", the first number
    every row and the second those live and those needing the person; "1 goal,
    1 under way." in the singular; "No goals yet." with none, once its reads
    have answered, and no heading before; "More than N
    goals, M under way." while rows remain unread. Then "A goal turns a draft
    into landed work, its own way: W.", W the project's ways in order, lower
    case, joined by ", " (*Ways,* below).
  - **Its filters:** "All", "Needs you", "Under way", "Ended", the chosen one
    pressed; at the section head's right, the `.link` "Start one from a draft",
    opening 5.14's sheet with no draft, while the project has a tracked
    template. Under them, the rows, or, once its reads have answered, "None.";
    "Needs you" is state needs,
    "Under way" running, planning and paused, "Ended" landed, closed and
    failed.
  - **"More goals",** a quiet button after the rows while the ended page has
    more, reading the next ended page in its place as "more closed chats"; the
    heading's numbers keep counting the rows read before it.

  What it reads: the query `TaskQuery { project, label: the template's,
  tracked, by priority }`, the label template `shapes::GOAL`'s, its live group
  watched and its ended group read by when each ended, as `Chats`; `listed`
  puts goals in the project's index. One read per row, one at a time, as the
  inbox's cards: its report (*A goal's report,* below). The needs are `inbox`'s
  group for the project, read again as the watch or `me`'s `needs_total`
  moves, each row's `selection` naming its goals.

  **A row** is one link to the goal's `Task`, public as `goals::row`, since
  Home's area will reuse it: its top line the chip's title (its spec's first
  line) at the left and at the right "6 / 9 · " and its state's word; then its
  headline. "6 / 9" is done, then total, of the goal's direct delegates, not
  counting cancelled ones, as its task line counts them (jig's
  `client/domain/contract.md`, 5.1), true past one page; it is left out, with
  its " · ", while there are none. An ended row is marked.

  **A goal's state** is a pure function, `goals::state`, the first that matches:

  | State | When | Word |
  |---|---|---|
  | needs | a needs row's `selection` names the goal | "needs funds" where the goal's own chip is held for `HoldCategory::Exceeded`, else "needs you" |
  | paused | live, held by a placed hold of mode Stop over its own subtree | "paused by you" |
  | planning | live, no report, no delegates | "planning" |
  | running | any other live | "running" |
  | landed | ended, done | "landed " and its end in the clock form (5.12) |
  | closed | ended, cancelled | "closed" |
  | failed | ended, failed | "failed" |

  A row's **headline** is its report's. With no report: "Planning from the
  draft." planning; "N of M tasks done." running or paused, N its direct
  delegates ended done, whatever they did, since `TaskLine` counts no landings
  (section 11); "Landed." landed;
  "Closed by you." closed; "Failed." failed; a goal that needs the person
  takes running's words, or planning's with no delegates.

  **Ways** are one public function, `goals::ways`, of the view, the project's
  templates with `tracked` set, in number order, each with a name and a line.
  Template `goal`: "Task by task", "The same plan; each task lands on its own
  once checked." Template `goal for a draft`: "Plan, tasks, land whole", "A plan
  from the draft, its tasks on a goal branch, landed as one change once all are
  checked." Any other: its own name, no line. The filter's and the page's
  words are drawn in the terminal as a page of its own, heading, filters as
  numbered buttons, a row a numbered link with its state and headline.
- **A goal's report** is the note `../domain/agent.md`, 8.3 shapes. `report::parse`
  in the client's domain, pure, reads a note as `Report { headline, paragraphs,
  left, risk, decided, at }`, a line's start alone making `Left:`, `Risk:` or
  `Decided:`; the read is `Read::Notes { scope: NoteScope::Task { project, task
  }, description: Some("report: "), after: None }`, the newest by `changed`
  whose description opens so, since jig's filter finds its words anywhere. Notes
  have no watch: a page reads again when the goal's `last_activity` moves. No
  note yet is a normal state, never an error.
- **A suggested request** (`chats.html`) starts a chat with the
  cue's words as its first message, as the composer's send does: one
  act, one request (3.6), the suggestions and the send disabled while
  it is pending. The engine puts that message in the run's brief, and
  the scripted model finds the cue there (`../protocol/setup.md`, 8.4), so
  the chat plays the cue's script; the composer's text is left as it
  was.
- **The first chat** (`../draft/configuring.md`, 5.1): Home's first step
  offers "Tell temper how you work", and its "Start" opens `Configure {
  project }` (below): a step says what the person does, and what it opens
  is named "Configure temper". Settings keeps its "Configure temper"
  (*Settings,* below), and "How work runs" makes the same offer (5.13). The
  chat starts with the person's own words, never with canned ones. The
  Chats page makes no such offer.
- **The configuring chat's page** (the mock's
  `chat.html?id=new&configure=1`, in the project's frame):
  `Configure { project }`, at `/projects/<project>/settings/configure`,
  under `Settings`, so that it reads as configuring temper and not as
  one more chat. It opens that project as `Home` does; with no
  such project in `me` it says what `Home` says. Each main of this
  layout (`temper-session`) is the page's height, in this order:
  - *Its top bar:* the path, a navigation named "Where": on a wide page
    PROJECT, a link to `Home`, and "Settings", a link to `Settings`, each
    followed by "/", then the level-1 heading "Configure
    temper", the page's title too; on a phone "‹", a link named "Settings", alone before the
    heading. With a composer its bar names no model: what the chat's runs use is the picker's,
    under its composer (*The work's account and model,* below): on a
    configuring chat, what its next run takes, as any chat's; with no chat
    yet, what one would, template 3's `resolved_default` in `me` (5.5), and a
    choice made here is the chat's when it starts. It never sets one LLM
    provider's account beside another's model; with no usable account the
    composer says what 5.5 says then, and there is no picker.
  - *The conversation,* a region named "Conversation", its opening a
    page as `Home` is, starting 28 pixels from the conversation's top, with no
    kicker: the level-2 heading "How should temper run PROJECT?", drawn
    as Home's headline; the lede "Tell it how you work, in your own
    words or by pointing at a document."; a list named "What it sets",
    each item a term in ink and its line in grey after a pale dot: "How
    work runs: how chats and goals start, are checked and land",
    "Models: which plan or model each role runs on" and "Approvals: what
    waits for you, and what lands on its own"; the note "It proposes the
    settings; nothing changes until you accept.", small and muted; then,
    30 pixels under the note, the section "Ways to begin", headed as Home's
    sections: its starters as cards, as the welcome's choices, three to a
    row where they fit and one a row on a phone, the whole card the button,
    its padding 14 and 16 pixels, its title over a line in grey:

    | Title | Line | The words it sends |
    |---|---|---|
    | "Small changes, alone" | "One chat per change, landed once CI passes." | "I work alone on small changes: one chat per change, landed once CI passes." |
    | "A design draft first" | "Agents plan its tasks; they land together." | "I write a design draft first; agents plan its tasks and land them together once checked." |
    | "Models by role" | "Claude for reviews, my ChatGPT plan for the rest." | "Use Claude for reviews and my ChatGPT plan for everything else." |

    "Models by role" goes with a single account: it stands only while the
    offer in `me` names two accounts or more, and its line and its words name
    the first two, "A for reviews, B for the rest." and "Use A for reviews and
    B for everything else.", each as the accounts page names its kind ("Claude",
    "my ChatGPT plan", "my OpenAI API key"), so that it names only what is connected.

    A fourth starter, a button drawn as a link, "Not sure yet? Ask what
    temper can do.", which sends "Not sure yet. What can temper do?",
    stands in the section's head, at its right. Under the cards, a wide
    card of temper's own way (`../draft/own-way.md`, 2), a group named
    "temper’s own way": its title, "At scale, as temper builds itself";
    under it its steps in small grey, a list named "Its steps: designers
    cut a draft into parts, coding agents make them in parallel, the goal
    lands whole", "Draft", "The cut", "Coding agents", "Goal’s branch" and
    "main", an arrow ending each line where they wrap; at its right two
    outlined buttons, "See how it works" (*temper's own way, beside the
    page,* below) and "Start from it", a starter that sends "Set PROJECT
    up the way temper builds itself, sized for this machine."; on a phone
    the buttons stand under the words, each half the width. The terminal
    has no style to tell a term from its line: each item and each card of
    the three is one line of words, "TERM: LINE", and "Ways to begin" is
    its section's name, once; the wide card's title, steps and buttons
    follow in that order, the steps a list with no arrows.
  - *The composer,* a form named "Configure temper", at the
    foot, focused as the page opens: its text area "Describe how you
    work, or what to change", that its placeholder too, its text 14
    pixels in from the composer's edge; "Send", the send arrow on ink in a rounded
    square, grey while disabled, which it is while the text is empty;
    under it the chat page's hint, "Enter sends · Shift+Enter for a
    new line". Its picker is on that row, as a chat's: a choice there is this
    chat's alone, and the configuring role's models stay a setting (5.13),
    which this chat can change. On a
    phone the frame's top bar and tab bar go, so that the composer keeps
    the foot, and with them the model's line and the composer's hint,
    as the configure-chat look request draws it.

  The conversation and the composer's foot each keep a scrollbar's gutter
  on both edges and hold one column, at most 720 pixels and centred, so
  that the opening, a chat's turns and the composer share their edges
  whether or not the conversation scrolls, in every mode, the column
  shifted by half a scrollbar where the platform's scrollbars take room. On
  a phone
  neither keeps a gutter and both stand 16 pixels in, the conversation
  scrolling with no scrollbar drawn, the opening 20 pixels from the top,
  the question at 26 pixels and the cards keeping their 14 and 16 pixel
  padding. At 1366 by 680 the whole opening and the
  composer show with no scrolling.

  A starter or "Send" starts the chat, the words it sends the first
  message, as a suggested request does: one act, one request (3.6), from template 3
  (`../protocol/configuration.md`, 13.4) while `me` lists it among the
  project's templates, else template 1, with its template's own models.
  While it is out, the four starters, "Start from it" and "Send" are disabled
  and the words stay; "See how it works" starts nothing, and stays enabled;
  made, the chat's page opens; refused, jig's words stay under the
  composer, the words kept. The chat names no repository.
- **temper's own way, beside the page** (`../draft/own-way.md`, 2 and 3):
  "See how it works" opens the reference, "How temper builds temper", in
  the pane of *A chat's document* (below), in Split, the rail folded, the
  page's main carrying that mode's style. No chat is made and nothing
  changes. Every project offers it: no project way hides the card or the
  pane, so a fresh project's page, Goals hidden, has both, as has a page with
  drafts turned off.
  - *The bar* gains the mode group at its right, after the line of what the
    runs use, once the reference has opened: "Chat", "Split" and "Draft", a
    group named "How temper builds temper, beside the chat", with a chat's
    document's shortcut, and the composer's hint is a document's. "Chat"
    folds the pane, the group staying. The mode is the page's, in no address
    and kept nowhere: leaving the page or a reload closes the reference.
  - *In Split,* the ways to begin are one a row, each title and its line on
    one line where they fit, wrapping at a narrower width, the wide card's
    buttons under its words. *On a phone* there is
    no Split: the reference alone, as Draft, the composer under it, as a
    chat's document is.
  - *The pane:* its line, in small mono, "temper’s own way · a reference"
    and "changes nothing"; the page's title, as a document's level-2
    heading; its summary in grey; then its sections, read by skein-ui's
    reader (`skein/docs/design/ui.md`, section 10). The reference is the
    stock page `how-temper-builds-temper`
    (`../protocol/documentation.md`, 2.2), compiled into the client, so
    that opening it reads nothing and fetches nothing.
  - *Its parts:* section 1, the picture, is a numbered list of the flow's six
    steps, a name in ink and its caption, with one number and no bullet.
    Section 5's table has a row name in ink over the reader's cells, and the
    note closing it is muted. The page holds no figures and no link to any.
  - *The terminal* prints the pane after the composer, as it prints a chat's
    document: the line, the title, the summary, each section with its
    heading.
  - *A chat started beside it,* by a starter or "Send", opens the chat's page
    as from the page alone, and the reference closes; the chat's own document
    comes when it writes one. The configuring agent reads the same page
    (`../domain/agent.md`, 8.1).
- **A configuring chat** is a chat whose chip's executor is the charter
  of template 3 as `me` lists it (*Home,* above). In `Chats` it is listed
  with the others, tagged "Configuring". Its `Chat { task }`, opened from
  there, from its start or from its proposal's review (5.13), has this
  layout: the same top bar, its path still through `Settings`, its
  heading the chat's number and title as a chat's is, then its number,
  its status, its spend, Pause or Resume, and Close; the
  conversation as a chat's in place of the starters; the composer of
  this layout, its text area "Message" as a chat's, and none on an ended chat; "Settings" is the
  rail's current entry. Its address stays `/chats/<task>`. A chat from
  template 1 is any chat.
- **The work's account and model.** The model and effort sit at the composer,
  on every chat: a new chat's, Configure temper's, a chat's and a configuring
  chat's (the look request `model-picker`). A choice is the task's LLM choice,
  from the chat's next run (jig's `domain/tasks.md`, sections 3 and 7).
  - *The picker* is on the hint's row, at the composer's right edge: a
    disclosure named "Model" (skein's `ui.md`, its disclosure), which folds
    away, described by its closed line. A browser shows the line and a chevron;
    the terminal and the fake person read "Model: LINE" and find its content
    whole. On a phone the hint goes and the picker stays.
  - *Its closed line* says the model and effort Create or the next run would
    take, with no account: the draft's LLM choice, or with none the supplied
    `resolved_default` of the chat's template (jig's `client/domain/contract.md`,
    5.1), a configuring chat's its charter's template, any other's template 1;
    "the default model" where neither names one. "Claude" goes before the shown
    name of a model of the `anthropic` provider and no other provider's name
    goes before one, so "Claude Opus 5.5 · high" and "GPT-6.1 Sol · high". Its
    effort is the one the choice names or, where it names none, that model's
    default in the offer. What a task holds stays its own: a model the offer
    leaves out is still named, and its wait says why (5.5).
  - *With no composer,* a closed chat, one with a result or ended, says it in its
    bar instead, in the bar's muted form where the number was: "T2 · GPT-6 Astra ·
    high", in the closed line's shown names, in the browser and the terminal
    alike. It uses the latest named completion kept by the thread
    (jig's `client/domain/client.md`, section 10), the effort only where
    that turn asked for one. With no model read it says what its
    next run would have taken: its LLM choice, or with none its template's
    `resolved_default`, as *Its closed line* has them. "T2 · Using the default
    model" is only where neither names one, where temper can't know. A run
    with no readable named turn can leave an earlier model or this preview;
    neither proves what that run used (section 11). Until the
    conversation is read to its end the bar says the number alone, and names no
    model it would take back. A closed agent's task says the same words without
    the number, which its heading holds, in its taskbar (5.9), with no line or
    picker at its composer. A goal's page draws no such line. A phone's bar shows
    no byline, so a closed chat's words are left out there as its number is.
  - *The menu* rises from it, by what pays, drawn from the project's offer in
    `me` (jig's `client/domain/contract.md`, 5.1), working nothing out, so that
    no page shows what the offer leaves out. In order:
    - A heading per account of a model's `accounts`, in the offer's order: its
      provider, "Claude" for `anthropic`, and muted what pays, "your
      subscription" for an `anthropic` plan or program, "your ChatGPT plan" for
      an OpenAI plan, "your API key" for a key. An account the accounts document
      doesn't hold takes its provider's name alone. Two headings of one provider
      that would read alike say who each account is instead (*An account*, 5.5). A model
      with no accounts, the playground's, goes under its provider's name.
    - Under each, a button per model `picker` shows, by its shown name, pressed
      where it is the choice, which draws a tick; "default", muted, on the model
      `resolved_default` names, under its first account. Pressing one takes it at
      its default effort; where its provider has two accounts, also preferring
      the account it is under. Pressing the row marked "default" returns to the
      draft's own choice where that names the same model or none.
    - A provider with no model in the offer is one line and no model, the
      stock's included: its name and "Connect a plan or key ›", a link to
      `ConnectModel` with no way chosen; or, where the person holds an account of
      it that has stopped being usable, its name and that account's own fix as
      the accounts page words it ("Sign in again ›"), a link to `Accounts`. The
      playground's provider has a line only while the playground runs.
    - "Effort", a group of the chosen model's efforts as the provider says them,
      the chosen one pressed; none where the model declares none.
    - "All models" and how many the offer holds, where the picker shows fewer.
  - *All models* takes the menu's place: "‹ Models", back to it; a field "Find a
    model", matching any part of a shown name or a name, in any case; every model
    of the offer, by account as above; and "Choose what the list shows, in Settings",
    a link to `Models { project }` for the picker's project. A model whose `new_until`
    is later than now has `.temper-model-tag`, "new", beside its shown name.
  - *A started chat:* the menu opens with "A change applies from the next turn."
    One press of a model or an effort applies it, an amendment or, where the task
    offers only that, a proposal. While it is sent that line says "Changing…";
    refused, it says why, and the choice is the chat's own again.
    An accepted amendment changing only the LLM choice sends no message and
    starts no run. A turn under way keeps its choice; the person's next words
    stop a live run behind the choice and start the next on the new choice,
    with those words retained and no try spent (jig's `domain/tasks.md`,
    section 7).
    A model change in a resumable window first asks the saved window's model,
    when configured, for a summary, then the new model for the answer.
    A changed effort alone keeps the window. Across an endpoint, dialect,
    account or backend boundary, or without the saved program session, the
    conversation's text, calls and results carry over, with opaque replay
    dropped and the summary on the new model (smith's `domain/session.md`,
    8.2). A provider change chooses an account again (jig's `domain/tasks.md`,
    section 7). A smaller window follows the same compaction rules. If the
    conversation still cannot fit, the chat is held with its transcript kept
    and nothing cut: "the run no longer fits the model's window of N tokens"
    (jig's `client/domain/contract.md`, 5.5). The person can choose another
    model on the held chat.
  - *A chat whose choice can't be changed:* the same words with no chevron, not a
    button. No usable model: 5.5's "No model yet" above the composer, and no
    picker.
  - *The terminal:* the same list as lines, its headings and models in order, then
    "Effort:" and its efforts beneath.

  A task's wait for its account and a refused next run stay
  above the composer. An agent's task's page, which keeps its
  own composer (5.9), keeps the line (5.5) and the picker together in it. A refused next run says "Next run refused
  · WHY", WHY the refusal's cause in the words of jig's reference view
  (jig's `client/domain/client.md`, 19.1), each sentence ending with a
  full stop: for a host's hello, "An engine host was refused at HH:MM"
  or "A worker was refused at HH:MM" (5.12's clock), then its cause, in
  the words jig's reference view gives a hello's refusal. A run no worker
  can ever take, a refused hello with its figures among them, says 5.5's
  "No worker can take this: ..." instead. A refusal is not a reply: the chat
  starts nothing, and keeps its composer, so the person can choose
  another model. The scripted model is offered where `me` serves it,
  in the playground's project alone (4.8). Setting a goal names its planner's model and
  effort, the template's by default. A goal's page draws no model, effort
  or account line; that presentation is deferred in section 11. An open
  agent's task's page shows them in its composer's line, as a chat's (5.9);
  a task waiting
  for its account
  says so, with the account's acts (5.5; jig's `domain/authority.md`,
  8.3).
- **Deleting a project** (jig's `domain/people.md`, 5.2): a project's
  settings offer its owners "Delete", confirmed by the project's name
  (3.6), which sends jig's `RemoveProject` by
  `shapes::remove_project`, with the chat template's grace (jig's
  `client/domain/client.md`, section 19; `client/domain/contract.md`,
  3.2). Once it is answered, the project
  leaves `me`, and with it the navigation and every list (5.3); its
  work is jig's to end and erase. The settings' other parts are
  section 11's.

### 3.6 Requests and confirmations

- **One act, one request,** keyed by the domain and sent through
  `jig-client`, which keeps it until its answer is durable (jig's
  `client/domain/client.md`, section 6): a double press joins the
  pending one, and nothing is shown done before its answer.
- **jig's operations, through jig's helpers** (jig's
  `client/domain/client.md`, section 19; jig's
  `client/domain/contract.md`, 3.2 and 3.4): starting a chat is
  `shapes::chat`; setting a goal is `shapes::set_goal`, or from a template
  `shapes::set_goal_from` (5.14), a proposal where the person's authority
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
  `client/domain/contract.md`, 5.4). Closing a chat says only what closing
  changes (*The sheet,* below). Removing a forge says what goes and what stays on the
  forge (`forge.md`, 5.4); deleting a project asks for the project's
  name, and sends nothing until it is typed whole (3.5). Pause and Resume, on a
  chat, a goal or an agent's task, are undone by each other and sent at once. A
  goal's Close says that what landed on its branch stays there (5.11). Where the document offers an act
  as proposable (jig's `client/domain/contract.md`, 5.3), the card says
  so, and to whom; where it offers it as refused, the card says why and
  sends nothing.
- **The sheet.** Every confirmation of `confirm.rs` is one sheet, from the
  look request `close-chat`: the mock's `dialog.sheet`, narrower, centred over the page. It asks in a
  sentence, says only what will change, and names its acts: what keeps things
  as they are first, then the act itself, primary. There is no "Confirm" and no
  "Dangerous action": the sheet is how temper marks an act that can't be undone
  (jig's `client/domain/client.md`, 19).

  | Confirms | Title | Says | Keeps | Does |
  |---|---|---|---|---|
  | closing a chat | "Close this chat?" | "You can still read it, but it can’t be reopened.", and what it changes | "Keep it" | "Close chat" |
  | "Start over" | "Start over?" | "A new configuring chat starts. This one stays in Chats.", and what it changes | "Keep going" | "Start over" |
  | closing a goal | "Close this goal?" | "You can still read it, but it can’t be reopened.", then "What landed on its branch stays there." | "Keep it" | "Close goal" |
  | closing an agent's task | "Close this task?" | "You can still read it, but it can’t be reopened.", and what it changes | "Keep it" | "Close task" |
  | accepting a proposal | "Accept this proposal?" | its facts | "Not yet" | "Accept" |
  | rejecting a proposal | "Reject this proposal?" | its facts and reason | "Keep it" | "Reject" |
  | releasing a task | "Release this task?" | what releasing does; an uncertain effect's two choices | "Keep it held" | "Release" |

  - *What it changes,* under a hairline, each line present only where it holds,
    and no count at zero:
    the run, while one of this task's is live: "The run under way gets N seconds
    to finish, then stops.", the template's grace, or "The run under way stops
    now." with none (a chat at rest closes at once, jig's `domain/tasks.md`,
    5.8); each live delegate, by its number and title and its fate, "T4 Fix the
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
  - *A press stays under the finger* (`browser.md`, *A press a render can lose*):
    the acts are a group of their own, keyed, and each delegate's row keyed by its
    task, so a line arriving above them, or the page updating behind, keeps each
    button's element.
  - *The terminal* prints the same lines as text, and the acts by their names.
- **Closed ends on one line.** A chat its requester closed ends on one quiet line,
  "Closed at 21:05" (*The Chats page,* its ended line), between two hairlines; its
  bar's status says "Closed", with no time. "Applied" isn't said once it is
  closed, nor "Decision made" for the closing itself: that line says it; nor
  after a chat's Pause or Resume: its status says it (3.5, *A person's pause*).
- **It names what the person saw.** Each deciding act names the
  revision of the document the card showed; when the engine refuses it
  as moved, the card asks the person to look again (jig's
  `client/domain/contract.md`, 5.2). jig's check holds it in temper's
  worlds (section 10).
- **Answered:** the card shows the outcome, or the refusal in jig's
  words with what fits (look again, change what was asked); unknown,
  past the engine's retention, it says so. A goal's page says it in the toast (5.11).
- **A key is pasted once, never kept.** Its field is a secret (skein's
  `ui.md`, 3.2 and 3.4), and adding or replacing a key is a request
  carrying a value (jig's `client/domain/client.md`, section 6; jig's
  `protocol/client.md`, section 10): sent once under a name, never in
  the small store, never parked, never sent again by the client. The
  field is emptied as the request leaves, and the key is shown nowhere,
  then or after. Answered unsent or lost, the card says so and asks for
  the key again; the person's second send keeps the first's name, so
  that a request the engine made answers with its first answer, and no
  second account is made. A key empty or past 4 KiB (the bound of
  `tools.md`, section 12) is refused before it is sent; one the
  engine refuses is answered with its refusal.
- **A forge's pasted value is sent the same way.** A GitHub App's
  private key (with no webhook secret: GitHub has none of temper's),
  a Forgejo bot's tokens and their webhook secret, a site
  admin's token, and a replacement of any of them are secret fields,
  and the request is a request carrying a value (jig's
  `protocol/client.md`, section 5): the value is sent once, never in
  the small store, never parked, never sent again by the client, and
  the fields are emptied as the request leaves. The request names no
  number for it and the client draws none: jig numbers the value as it
  keeps it (`../protocol/credentials.md`, pasted values). Answered
  unsent or lost, the page asks for the value again, and the second
  send keeps the first's key. A value empty, or a bundle past 4 KiB,
  jig's bound on a carried value (the bound of `tools.md`, section 12),
  is refused before it is sent, in the words of `forge.md`, 5.4; one
  the engine refuses, past the
  deployment's `records.value_bytes` among them, shows its refusal
  (section 8). The client reads no configuration, so its bound is jig's.

### 3.7 The amendment editor

One editor drafts a person's amendment of a task, for every page and
command that amends one: the task's pages (5.8, 5.9) and `temper ctl
task amend` (`tools.md`, 5.5). It is `temper-client-domain`'s `amend`
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
  is a tracked task's priority alone (jig's `domain/tasks.md`, section
  7), and the base's otherwise.
- **What is whose.** The editor holds the rules of a draft (what
  each edit keeps, what is cut, when a revision is named). The words
  and fields are the caller's: a page's form, `ctl`'s flags and its
  lines on stderr.

## 4. The forge's client family

The forge's client side is the client root's child for the forge
connector, an owner numbered as its connector (jig's
`client/domain/root.md`, sections 2 and 15). It shows the forge's parts
that need no document, reads the documents of the others (`forge.md`),
holds one form, the adoption, with its picker, sends the forges'
page's requests (section 8), and reads, starts and resets the
playground (4.8).

### 4.1 Words that need no document

A connector, a resource or a kind needs no document: its words come from
its number or its path.

- **The connector:** "forge".
- **A resource,** by its path (`../domain/forge.md`, section 2):

  | Path | Words |
  |---|---|
  | `forge` HOST OWNER NAME | OWNER/NAME on HOST |
  | … `branch` S1 … Sn | branch S1/…/Sn of OWNER/NAME on HOST |
  | … `pull` N | pull request #N of OWNER/NAME on HOST |
  | … `issue` N | issue #N of OWNER/NAME on HOST |
  | two segments: a forge's number (2 bytes) and a repository's id (4 bytes), big-endian, as an adoption names it | repository ID on forge F |
  | three segments: a forge's number (2 bytes) and a repository's id (4 bytes), big-endian, then a branch's name, as the hub names a delegate's branch (`temper-engine-domain`'s forge route) | branch B of repository ID on forge F |
  | anything else | its segments joined by `/`, as they are |

  jig's documents name an adopted repository by its forge's number and
  its id (`../domain/forge.md`, section 4), and the client shows those;
  the forge's documents name it by owner and name (`forge.md`, 2.5).
- **A resource pattern** (a grant's): the words of its segments as
  above, its last segment as it is; an open end is its prefix followed
  by `…` ("branch temper/7/… of acme/shop on forge.example").
- **A kind,** by the forge's numbering (`temper-engine-domain-forge`'s
  `effect_shape`; `../domain/forge.md`, section 6):

  | Kind | Words | Kind | Words |
  |---|---|---|---|
  | 1 | read | 9 | create a branch |
  | 2 | push | 10 | edit a pull request |
  | 3 | open a pull request | 11 | update a pull request from its base |
  | 4 | merge a pull request | 12 | close or reopen a pull request |
  | 5 | post a review | 13 | edit, close or reopen an issue |
  | 6 | set a commit status | 14 | set reviewers |
  | 7 | post a comment | 15 | delete a branch |
  | 8 | create an issue | any other N | kind N |

  These are the forge client's writes in words (its `Write`
  vocabulary), with reading and pushing. A new kind is a new row here
  in the same change that numbers it.

### 4.2 The forge's other parts

Every other part of the forge's that jig's documents name (a panel, a
result, an effect, an outbox entry, a hold, a project's part, the
deployment's part: jig's `client/domain/root.md`, section 6) needs a
document, which the forge's side reads or watches through the family's
own reads and subjects (`forge.md`, section 9). Wherever a page would
show a part the forge's side does not read, temper's view shows one
paragraph:

> Shown on the forge.

Such a part never shows loading, and never an error. A change's
procedure part on its task's page is not such a part: the forge's side
watches its panel (`forge.md`, 9.2), and the page draws it, or says it
is loading, unknown or cannot be shown (5.10).

### 4.3 Where the form is

`Adopt { project }` draws the “Add repositories” dialog over Settings’
Repositories (4.4). Settings’ button and links to adoption open it; Cancel
or Escape returns to Repositories. A project with no repository retains
its Chats band’s entry, which reaches this same address. Home’s second
step can adopt the offered first repository directly, open its own
empty-or-copy sheet, or link to adoption (3.5). Those entry points are
unmocked by `connect-flows`; the request changes the Settings destination.

### 4.4 The sheet "Add repositories" and its adoption form

`Adopt { project }` is Settings' Repositories with a dialog over it, a
sheet from the bottom on a phone (`../draft/repository-copies.md`, section
9; the request `connect-flows`). Its button on that tab, and every link
that said "Adopt a repository", go to it; its Cancel and its Escape go to
`Repositories`. A reload draws it again, empty.

- **Its head:** "Add repositories to PROJECT" (`project N` until `me` names
  it), and the sources, one choice of three: "Adopt", "Make empty" and "Copy
  public". The head stays where it is as the source changes; only the foot
  moves. With no Forgejo that takes them (owned or managed), the last two go,
  and the control with them.
- **Adopt:** a Forge field, where there are several (never "Forge number");
  an Account field, GitHub's, one per installation, where there are several,
  "LOGIN · selected repositories" or "LOGIN · suspended" where they are; then
  the picker of `forge.md`, 5.1; "Not listed", folded; "More", folded, its
  summary saying what it's set to ("More · Owned, it runs CI"), holding the fields
  below: "What temper does with it" and its three roles, "Branch prefix",
  "Landing branch", "Checks: gate numbers, separated by spaces or commas",
  "It runs CI" and "Make it the project's home". A chosen new forge or account
  clears the search, the rows and the picks.
- **Make empty:** "Organization" (`work`), "Name", then "On FORGE, with a
  README and a starter CI check.". The button says "Make ORG/NAME", or "Make
  it" until both are typed.
- **Copy public:** "Their addresses, one per line", the browser's caption over
  a field four lines high; the terminal prints the field's name once.
  Read as typed, a line at a time, each with its own words: "Not an address:
  LINE. Write it as github.com/owner/name."; "HOST isn’t a host. Did you mean
  github.com/OWNER/NAME?", with "Use it"; and, for a name the target forge
  has already, among the rows read of it and its makings, "OWNER/NAME is
  already on this forge.", with "Adopt it", which moves to "Adopt" with its
  row picked. Then "Copied into FORGE from GitHub or another git host. Each
  keeps its owner and name, and nothing is sent back.". The button says "Copy
  OWNER/NAME", "Copy N repositories" or "Copy", and is disabled while a line
  has words.
- **Forge:** the last two sources take a Forge field of their own only where
  there are several Forgejos.
- **Its foot:** "Cancel", quiet, and the primary, disabled until it can send.
- **Sent:** adoptions one at a time, each a keyed `Adopt`; `MakeRepository`
  for each line or the one name, `into` this project, `home` the first where
  the project has no home. All kept or adopted, the sheet closes to
  `Repositories`. A refusal keeps the sheet open, with what is left picked or
  written, and its words above the foot; those already done are off it.

The adoption form, under "More" and the picker. The engine
judges the adoption, and a refusal at admission (the party's role in
the project) is shown in jig's words (4.6). Its fields are
`temper-engine-domain-forge`'s `Adoption`'s but the repository's id,
which the forge connector locates by owner and name
(`../domain/forge.md`, section 4), and the forge's host, which temper's
server fills from the forge's record (section 8). A row chosen in
the picker, below the forge's field, fills the owner and the name (the
picker, below). In order:

| Field | Node, named | Default | Rule |
|---|---|---|---|
| forge | the Forge field, by name | the lowest listed, or a first repository's forge (below) | one the store does not keep is refused as no such forge |
| owner | text field "Owner" | empty, or a first repository's organization (below) | required |
| name | text field "Name" | empty, or a first repository's name (below) | required |
| role | choice group "Role": "Owned: temper pushes, opens pull requests and lands", "Fork: temper contributes through a fork", "Context: temper only reads" | owned | one of the three |
| home | check "Make it the project's home", for a single pick | checked only if the project has no home | a context repository cannot be the home; hidden for plural picks |
| prefix | text field "Branch prefix" | `temper/` | required |
| landing | text field "Landing branch" | empty, or `main` for a first repository (below) | required: the forge connector refuses an adoption without one |
| ci | check "It runs CI" | checked | without CI, at least one check |
| checks | text field "Checks: gate numbers, separated by spaces or commas" | empty: none | numbers from 1, each once, at most the client's limit on checks |

The foot’s primary names the selected repository or counts the picks;
request state and refusals remain above the foot.

**From a project with no repository** (3.5; `../draft/first-run-projects.md`,
4 and 5): the chats page's "Adopt a repository" opens this form into
its project, always a project's: no form makes a project. While a
set up's first repository is made and adopted nowhere
(`../protocol/setup.md`, 6.7), as the `forges` watch's rows say it (a
real forge's row whose `first` is made, and no row's `repositories`
naming its organization and name, 4.6), the form opens filled for it,
so that adopting it is one choice away: its forge's number, its
organization as the owner, its name, and `main`, the branch its
`Create` made, as the landing branch; the other fields at their
defaults. With several, the one on the forge numbered lowest. The person may
change any field, and "Adopt" sends `Adopt` alone.

**The picker** is below the optional Forge and Account fields. It reads
`Reachable` pages from 1, retains rows in forge order up to `picker.rows`,
and reads on when the list nears its end or a search needs more rows
(`forge.md`, 5.1). More than eight rows, or more still to read, gives it
“Find by name”, its own height and a count; a small completed list has
none of those extras. A row is a checkbox named OWNER/NAME, marked private
where supplied. “In PROJECT” and “Copying…” rows are disabled. Picks
survive search, but a change of forge or installation clears them. No
Next or First page button is drawn. “Not listed” holds Owner and Name
and the forge-specific remedy; a completed search with no matches opens
it with the typed name. GitHub’s account field appears only for several
installations; the wire carries no selected/all mode, so its remedy asks
the person to check existence and grant the repository to the App.

- **The ids ride, never shown.** A row carries the forge's id for the
  repository and its owner's id, besides the owner's login, the name and
  whether it is private; the form keeps the two ids for the `Adopt`
  request while "Owner" and "Name" are the row's, and drops them when
  either is edited by hand (4.6).
- **Refused, in words.** A forge not connected (`NotConnected`):
  "Forge F is not connected.", linked to the forges' page, and the primary
  disabled for that forge until a later page lists its repositories. A
  forge the store does not keep: "No forge F is configured." Refused for the
  person's role, which does not hold the deployment's connectors:
  "Your role does not hold the deployment's forges: give the owner and
  the name.", and the person adopts by owner and name, the
  engine judging the adoption as any other (section 11). A refusal at
  admission: jig's words. A forge that did not answer: "Forge F did not
  answer.", with "Try again". Busy, another read of the forge's picker
  running or the setup's host full: "Forge F is busy.", with "Try
  again". On a managed or owned Forgejo, the admin's token missing or
  not a site admin's: "Forge F's admin's token is missing." or "Forge
  F's admin, NAME, is not a site admin.", linked to the forges' page.
  None of these empties the fields.
- **Read on as `forge.md`, 5.1 has it.** The form keeps the rows read, up to
  `picker.rows`; an answer for a forge or installation no longer chosen is
  dropped. A page that answers no rows ends the reading: more no longer
  follow.

- **Every field is bounded** by the client's limit on a field's bytes,
  set no larger than the forge connector's name bound in temper's small
  profile (`temper-engine-domain-forge`'s `name_bytes`). A value past
  it is not taken: the field keeps its last value and is marked
  invalid, with the bound in words ("At most N bytes.").
- **Checked before sending:** each rule of the table that the draft
  alone decides (all but whether the forge is kept) is the
  form's own check, never one of what the person may do. A field that
  fails is marked invalid, with one status line under it saying the
  rule in words ("Required.",
  "Checks are numbers from 1, each once.", "A context repository
  cannot be the project's home.", "Without CI, name at least one
  check."), and nothing is sent. The engine checks everything again.
- **Adopt** makes one keyed request through `jig-client` (4.6), as the
  forge's family's bytes, made once (jig's `client/domain/client.md`,
  section 6). While it is pending the button is disabled and a status
  says its progress as `jig-client` gives it, in jig's words.
- **The forge by name, home by what is read.** The form chooses among the
  deployment's forges by name; with the forge's rows not read, the source
  waits for them. Home is on where the project has none it can read, and the
  engine refuses a second home, saying so.
- **After the outcome,** the outcome stays shown until the person edits
  a field or leaves the page. Adopted, the fields go back to their
  defaults (written by the domain, so the view writes them).
- **Out of date:** when the engine does not serve this client's version
  of the forge's records (jig's `protocol/client.md`, section 9), the
  page says "This temper is out of date for adopting repositories: run
  the temper the engine runs.", the button is disabled, the picker
  sends nothing, and a pending adoption stays in the small store.

### 4.5 Its address and state

- **Address:** `Adopt { project }`, one of temper's addresses (3.5).
- **State:** each field's text and the domain's count of its own
  writes; the pending request's key and progress, at most one; the last
  outcome; the version range the engine serves for the forge's records;
  the source; the forge and the installation chosen; the search; the rows
  read, the last page's number and whether more follow, or its refusal; its
  pending `Reachable` key, at most one; the picks, each with its ids; the
  empty source's org and name; the addresses as typed. Bounded by its limits:
  the field bytes, the checks, `picker.rows` rows kept, a constant of 2000
  in `temper-client-domain`, and as many picks.
- **Records:** none of its own; its pending adoption is `jig-client`'s
  kept request (4.6).

### 4.6 The records

`temper-client-protocol-wire`, generated from one schema (jig's
`protocol/client.md`, sections 3 and 11):

- **`FORGE`:** the forge connector's number, 1, which is the family's
  owner number on the wire, under which its records travel as bytes
  (jig's `protocol/client.md`, section 11), and the engine's one
  configured connector (`engine.connectors`,
  `../protocol/configuration.md`, section 2).
- **`Owner`:** temper's owners, one enum the engine's root and the
  client root share, with the functions from an owner to its number and
  back (jig's `client/domain/root.md`, section 15): `Forge`, `FORGE`'s
  number; `Pages`, temper's pages, 2; and `Runner`, the agents' tools'
  runner (`tools.md`, section 4), 3: numbers no connector takes, never
  reused. `Pages` and `Runner` ask only for jig's operations, reads and
  subjects, so no request, read or subject of their own reaches the
  engine.
- **`CLIENT_VERSION`:** the version of every record below, 1 at first,
  2 since the playground's read, 3 since a change's panel, 4 since a forge's
  set up and the deployment's forges, 5 since jig's client wire 13
  carried in its refusals, 6 since the first repository (`SetUp`'s
  fields and the row's `first`), 7 since the playground's state (its
  read's `state` and its start), 8 since a sign-in's return, since gone,
  9 since connecting a GitHub App (`Connect`,
  `Install`, `ForgeRefusal`), 10 since the forges row's installations
  and last connect's end, 11 since GitHub's manifest (`Connect`'s
  `GitHubManifest` and the row's `app`). It stays at 11 until the first
  release, as `../protocol/README.md`, section 9 says (skein's
  `codec.md`, section 5). A request, an outcome, a read, a subject or a
  field added to the family (the FORGE requests of section 8, the
  picker's ids in `Adopt`, the forge's kind in the adopted outcome, the
  playground's read, a change's panel) changes its schema, generated code and
  one set of golden bytes in place. The version moves at a release.
  The engine serves exactly one version (jig's `protocol/client.md`, section 9): a client of another
  is out of date for the forge (4.4), and its kept adoption is dropped
  as kept work at start (6.6; jig's `protocol/client.md`, section 10).
- **The request, `Requesting`:** which of the family's requests it is,
  `Request`, as `Reading` names a read: `adopt`, `Adopt`, `set_up`,
  `SetUp`, `connect`, `Connect`, `install`, `Install`, `reachable`,
  `Reachable`, `accept`, `Accept`, `remove_forge`, `RemoveForge`,
  `make_repository`, `MakeRepository`, `dismiss_making`, `DismissMaking`,
  `dismiss`, `Dismiss`, `looked`, `Looked`,
  `start_playground` and `end_playground`, the last two with no fields
  (below). Each is answered by the family's one record,
  `Answer`, whose outcome is one of either request's (below); the client
  refuses as undecodable an outcome that does not fit the request it
  sent.
- **`Adopt`:** the project (u32); the forge's number
  (u16); the owner, the name, the prefix and the landing branch (byte
  strings); the role (owned, fork or context); home and ci (booleans);
  the checks (a list of u32); and, when the owner and name are a picker
  row's, the forge's id for the repository and its owner's id (u64
  each, both or neither).
- **Its outcome,** `Answer`'s:
  - *adopted,* with the repository as the forge connector recorded it:
    its forge's number, its kind and its id; its owner and name; its role;
    whether it is the project's home; what temper may do there (the
    forge's `Kinds`: read, push, open, land, review, status, comment,
    issue, branch); whether branch protection was read (unknown,
    absent, or a rule); its merge style; and the landing branch's rules
    as read, each with the setting that would let temper land where it
    keeps temper from landing (`forge.md`, 5.1);
  - *refused by the core,* when the core refused the adoption the root
    made on the party's behalf: jig's refusal facts, carried whole as
    `jig-client-protocol-wire` encodes them, and shown in jig's words
    for its refusals;
  - *refused by the forge,* with the forge connector's reason: no such
    forge; not connected; not installed; not granted, with the
    installation's page; not found; no access; refused; in flight; too
    large; unreachable; invalid answer (4.7).

  A refusal at admission (signed out, the party's role in the project,
  a key sent again with another request, a limit) is not an outcome:
  it is jig's answer, which never reaches the connector (jig's
  `client/domain/contract.md`, 3.3), and the form shows it in jig's
  words.
- **The forge a request makes or names, `Making`:** `kept`, `Kept`, a
  kept forge's number (u16); or `new`, `NewForge`, a forge for the
  request to make, `ForgeAsk`: `github`; `owned`; `managed`,
  `ManagedForge`, a Forgejo of the person's connected as its admin:
  its API's host (at most 255 bytes) and port (u16), its TLS server
  name (at most 255 bytes, none for plaintext), its git base and its
  web address (at most 2048 bytes each, the web address optional) and
  its git program (at most 4096 bytes, none for the worker's); or
  `connected`, the same `ManagedForge`, a Forgejo of the person's
  connected with its bot's tokens, its mode connected
  (`../protocol/setup.md`, section 4). A new forge is made as a record
  in the request's decision and numbered next, never reused
  (`../protocol/forge.md`, 2.1 and 2.2); the client finds its number
  on the `forges` watch's rows once the request is answered.
- **`SetUp`:** of the deployment's scope (section 8): the forge,
  `Making`, a kept owned Forgejo or a new one; and `first`, `SetUpFirst`,
  with no project (`../protocol/setup.md`, 6.7): `empty`, `FirstEmpty`, the
  org and the name (byte strings, bounded as `Adopt`'s owner and name), made
  unadopted, and `playground`, a bool, the repository made as the
  playground's copy, `playground/widget` with its files, the org and name then
  `playground` and `widget`; `copies`, `CopyAddresses`, at most
  `FIRST_COPIES` (16) addresses, each at most 512 bytes in the form the
  client's check gives (`../draft/repository-copies.md`, 5.4), which make no
  repository at set up; or `nothing`. Its outcome, `Answer`'s: *setting up*, the box asked
  for the forge's group; *set up refused*, with why: not offered (the
  forge's connection does not offer it, or no such forge), not under
  the box, busy (`../protocol/forge.md`, 2.8),
  or invalid, a name Forgejo
  would refuse (the org a login of at most 40 bytes, the repository
  1 to 100 bytes of letters, digits, `.`, `_` or `-`, neither `.` nor
  `..`; skein `forge-setup.md`, 4.9) or past the engine's
  `name_bytes`; or *unknown*, its
  outcome no longer held (section 8), the same unknown as an
  adoption's. A refusal at admission is jig's answer, as for `Adopt`.
- **`DismissMaking`:** of the project it names, or of the deployment with
  none (section 8), as `MakeRepository`'s `into` gives its scope, so that a
  key sent again has the same scope and is replayed once the row is gone: the
  forge's number (u16), the org and the name, bounded as `MakeRepository`'s,
  and `project`, none or a u32, the failed row's `into` as the client reads
  it on the `forges` watch. It erases a failed row (`../domain/forge.md`,
  4.5), freeing its place in `forge.makings`, and is answered *kept*; with no
  row at the name, so that a request sent again joins, *kept* too; with a row
  whose `into` is not the project it names, refused as `not_owner`; with a
  row that is not failed, refused as `not_failed`.
- **`MakeRepository`:** of the project `into` names, or of the deployment
  with none (section 8): the forge's number (u16); the org and the name
  (byte strings, bounded as `Adopt`'s owner and name); `from`, `MakeFrom`:
  `empty`, or `copy`, `MakeCopy`, its address (at most 512 bytes), the
  form `https://HOST/OWNER/NAME` the client's check gives
  (`../draft/repository-copies.md`, 5.4); and `into`, none or `MakeInto`:
  the project (u32) and `home` (bool). Its outcome, `Answer`'s, kept as an
  adoption's is (section 8): *kept*, the row written; *make refused*,
  `MakeRefusal`: no such forge, not offered, not owner, invalid, home
  held or full (`../domain/forge.md`, 4.5); or *unknown*, as `SetUp`'s.
  A refusal at admission is jig's answer, as for `Adopt`. The course of
  what it asked is the forge's row's, never the answer's.
- **`StartPlayground`:** of the deployment's scope (section 8), with
  no fields: the start makes the playground's records
  (`../protocol/setup.md`, 8.4). Its outcome, `Answer`'s: *running*,
  the `Playground` row running, its account kept and its repository
  adopted; *start refused*, not offered (a real forge kept); *start
  failed*, with why, `StartFailure`: the group's failure, with the
  box's `WHY` (a byte string of at most `box.line_bytes`), the account
  refused, the adoption refused, or the forge not connected in time;
  or *unknown*, its outcome no longer held (section 8), as `SetUp`'s.
  A refusal at admission is jig's answer, as for `Adopt`.
- **`EndPlayground`:** of the deployment's scope, with no fields: the
  playground's reset (`../protocol/setup.md`, 8.4). Its outcome:
  *reset*, once the reset's last act has erased the `Playground` row;
  *end refused*, `EndRefusal`, not offered (no playground); or
  *unknown*, as `SetUp`'s.
- **`RemoveForge`:** of the deployment's scope, the forge's number
  (u16) (`../domain/forge.md`, 4.1). Its outcome: *removed*, the forge
  ended in the request's decision, its removal's kept acts following
  it; *remove refused*, `RemoveRefused`, with a `ForgeRefusal` (below):
  `role`, `no such forge` (none kept, or one ended already), or `not
  taken`, the playground's forge, which only its reset removes; or
  *unknown*.
- **`SetAway`:** of the project it names, from any party with a role
  there, as `Dismiss` (`../domain/forge.md`, 5.8): `project`, u32; `on`,
  bool; `until`, an optional u64, nanoseconds since the epoch as
  `ChangesLanded.at`; `kinds`, u8, its bits past 15 refused by validation.
  Its outcome: *away*, `away`, no fields, once the row has committed;
  *forge refused*, `too large`, the family's bound reached; or *unknown*,
  no longer held, which the subject `away` answers.
- **`Overturn`:** of the project it names, from any party with a role
  there (`../domain/forge.md`, 5.9): `project`, u32; `number`, u32. Its
  outcome: *overturned*, `overturned`, no fields, once the row has
  committed; *overturn refused*, `overturn_refused`, `OverturnRefusal`:
  `already` or `unknown`; or *unknown*, no longer held, which the subject
  `decided` answers.
- **`Dismiss`:** of the project it names, from any party with a role
  there (`../domain/forge.md`, 5.5): `what`, `PersonFact`, a fact the party
  keeps: `to_begin`, `ToBegin`, the project (u32), "Skip" on that project's
  "To begin"; or `goal_from`, `GoalFrom`, the project (u32), the chat (u64), the
  goal (u64) and `at` (u64), that the person started that goal from that chat's
  document, at the client's clock.
  The root names the project as the request's scope, whatever scope the record
  came under.
  Its outcome: *dismissed*, `dismissed`, no fields, once the row has
  committed, and the same for a second, under any key, from any device;
  *forge refused*, `too large`, the family's bound reached (the forge
  reason of 4.7); or *unknown*, its outcome no longer held (section 8),
  as `SetUp`'s, which a client answers from the subject `dismissed`
  below. A refusal at admission is jig's answer, as for `Adopt`.
- **`Looked`:** of the project it names, from any party with a role
  there, as `Dismiss` (`../domain/forge.md`, 5.6): `project`, u32, the
  person entering that project's Home, not while To begin shows. Its
  outcome: *looked*, `looked`, no fields, once the row has committed;
  *forge refused*, `too large`, the family's bound reached; or *unknown*,
  no longer held, which the subject `status` answers.
- **`Connect`:** of the deployment's scope (section 8): the forge,
  `Making`, a kept forge or a new GitHub, managed or connected Forgejo,
  and how it connects, `How`: `github_app`, `GitHubApp`,
  the App's id (u64, never 0) and optional `read_only` (bool, absent
  meaning false), set by ctl connect `--read-only` and kept on the forge
  record (`../protocol/setup.md`, 17.7); `github_manifest`,
  `GitHubManifest`; `forgejo_admin`, `ForgejoAdmin`; or
  `forgejo_tokens`, `ForgejoTokens`, with no field, a connected
  Forgejo's bot's tokens (section 8's table). The App's key is no
  field: it rides beside the record as jig's carried value, a bundle of
  one `Pasted` item, and so do a Forgejo's tokens, a bundle of
  `ApiToken`, `GitToken` and `WebhookSecret` when given
  (`../protocol/credentials.md`, 2.2). Its outcome,
  `Answer`'s: *connecting*, the run started; *forge refused*, with why,
  `ForgeRefusal` (below); or *unknown*, as `SetUp`'s.
- **`Replace`:** of the deployment's scope: the forge's number (u16).
  The new key or tokens are no field: they ride as its carried value,
  a bundle of one `Pasted` item, a GitHub App's new key, or of a
  connected Forgejo's `ApiToken`, `GitToken` or both, and
  `WebhookSecret` when it changes (`../protocol/credentials.md`, 2.2).
  Its outcome is a `Connect`'s: *connecting*, the run started; *forge
  refused*, `ForgeRefusal`; or *unknown*. It takes a GitHub forge or a
  Forgejo of the connected mode, connected or stopped
  (`../protocol/setup.md`, section 4 and 17.3).
- **`Install`:** of the deployment's scope: the forge's number (u16)
  and `back`, an optional byte string checked at decode (section 8). Its outcome: *a link*, `Link`, the install page's
  address (a byte string of at most 2048 bytes); *connecting*, an
  `Install` that stopped, its failure said on the forge's row; *forge
  refused*, `ForgeRefusal`; or *unknown*.
- **`Accept`:** of the deployment's scope: the forge's number (u16)
  and an installation's id (u64, never 0), one that exists already on
  GitHub (`../protocol/setup.md`, 17.4). Its outcome: *accepted*,
  `AcceptedInstallation`, with the installation's id, once it is kept,
  and the same when it was accepted already; *forge refused*,
  `ForgeRefusal`, as `Install`'s, before any run; *unknown*; or, its run
  stopped, *forge refused*, `ForgeRefused`, with the reason nearest the
  stop: `not found`, GitHub having no such installation of the App;
  `no access`, GitHub refusing the App's key or its read; `unreachable`,
  GitHub down or past the run's deadline; `too large`, the accepted set
  full (`forges.installations`); `invalid answer` otherwise. The forge's
  row is unchanged by a stopped `Accept`.
- **`ForgeRefusal`,** a `Connect`'s, a `Replace`'s, an `Install`'s, an
  `Accept`'s or a `RemoveForge`'s, by the root before any run: `role`,
  the sender not the deployment's owner; `no such forge`; `not taken`,
  the forge's kind or mode does not take the request (a Forgejo's
  install, the playground's fake, a `Replace` on a managed or owned
  Forgejo, a `ForgejoTokens` `Connect` on a GitHub or managed forge);
  `connected`, a `Connect` on a forge connected or connecting already,
  whose key or tokens `Replace` replaces; `not connected`, an `Install`
  or an `Accept` on a forge with no App, or a `Replace` on a forge
  neither connected nor stopped, nor connecting; `busy`, a run holding
  the forge (a `Replace` on a connecting forge among them), or
  `setup.runs` full; `no key`, a `Connect` or a `Replace` that takes a
  carried value and has none; `first is bot`, a `ForgejoAdmin` `Connect`
  whose `first` organization is the bot's login, case-folded
  (`../protocol/setup.md`, 5.5). Its carried value, if any, is named unused
  (section 8).
- **The small store's record of a pending request** is `jig-client`'s
  kept request, its request encoded by this crate as the family's bytes
  at `CLIENT_VERSION` (jig's `protocol/client.md`, section 10).
- **The FORGE requests and their outcomes:** `Connect`, `Install`, `Accept`,
  `Replace`, `Forget`, `SetUp`, `Reachable`, `RemoveForge`, `Dismiss`,
  `StartPlayground` and `EndPlayground`, beside `Adopt` (section 8).
- **The playground's read, `playground`:** of the deployment's scope,
  with no fields, and no subject: what it answers changes with the
  `Playground` row, made by a start and erased by the reset (4.8;
  `../protocol/setup.md`, 8.4). Its answer is none, with no row, or
  `PlaygroundAnswer`: its forge's number (u16); its project (u32),
  once its start has made it, else none; its model's LLM provider
  (u32); its repository's owner and name (byte strings, bounded as
  `Adopt`'s); its state, `PlaygroundState`, starting, running or
  resetting; and its cues, at most 8, each 1 to 256 bytes of text, in
  the configuration's order (section 8). On the wire it is
  `Reading`'s `playground`, `Reading` naming which of the family's
  reads a read is; none is the read's empty answer.
- **The deployment's read, `deployment`:** of the deployment's scope,
  with no fields and no subject: what it answers is the document's, so
  it changes only with a start. Its answer is always a record,
  `DeploymentAnswer`: `welcome` (bool), whether the document shows the
  welcome (`../protocol/configuration.md`, section 3); and `owned`
  (bool), whether Forgejo on this machine can be set up: the document
  has `application.machine.owned` and the engine runs under the box
  (`../protocol/setup.md`, 7.4). A first run's store keeps no forge, so
  no row of the `forges` watch can say so (`forge.md`, 5.4), and a wait
  for a host reads it as the fact of a box (5.5); and
  `program_login` (optional bytes, at most 8,192), *the printed command*
  below. On the wire it is `Reading`'s `deployment`, beside `playground`.
- **The printed command:** what the pages print works when pasted. Where
  the deployment's one worker is on the engine's machine
  (`../protocol/configuration.md`, section 1, the standalone shape),
  `program_login` is that machine's whole command but NAME, the running
  binary's word and the three directories in effect, each as a word
  (*The words of a command for this machine*): `~/temper program-login
  --config '…' --data '…' --cache '…'`. The pages print it with NAME after "On this machine,
  run:". It is a path of a machine that the party signed in to it
  may read: a read carries a sign-in (jig's `protocol/client.md`), and no
  answer goes before one. A path that isn't UTF-8, a line past its
  bound, and a deployment whose workers are elsewhere send none: the pages
  then print the placeholders the tools do, `temper program-login --config
  CONFIG --data DATA --cache CACHE NAME`, "On each host, with its
  directories:". Once the owner's `program_environment` answer comes, it
  replaces this command: an empty answer prints `temper program-login NAME`,
  and any other prints its words followed by `program-login NAME`.
- **The program's environment, `program_environment`:** of the deployment's
  scope, with no fields and no subject: what `temper program-login` needs
  around it to act on this deployment, from this machine's shell. It is the
  party's read, and only the deployment's owner has it, at any time, since the
  welcome's Model step shows them the command during set up and Connect a model
  after it; anyone else's is refused, and their page prints the command without
  its words. Its answer is
  the record `ProgramEnvironmentAnswer`: `environment` (bytes), the words the
  command starts with, which a page draws before `program-login NAME`. They are
  none when `temper` runs this binary and each of the three directories
  (`../protocol/configuration.md`, 12) is the one the command finds by itself
  in the environment the engine started in. Otherwise they are the word that
  runs the binary, after `TEMPER_CONFIG=…`, `TEMPER_DATA=…` and `TEMPER_CACHE=…`,
  in that order, each only where its own directory isn't that one: a box with
  its data alone moved says `TEMPER_DATA=~/x temper`. Where that environment
  gives the command no directories by itself, no `HOME` or XDG variable placing
  them, all three are named. It is one string, composed where the
  binary and the directories are known, so that the client holds no path as a
  fact, quotes nothing and decides no default; it is a read of its own,
  separate from `DeploymentAnswer`. A page asks when its address names a
  program's way, the only place that draws the command with these words;
  Accounts' has none of them (section 11). On the wire it is `Reading`'s
  `program_environment`.
- **The words of a command for this machine:** how `program_environment`'s are
  made.
  - *The word that runs the binary.* The running process's executable, less the
    ` (deleted)` Linux adds to one rebuilt while it runs. The word is `temper`
    when the first executable file named `temper` on the process's `PATH` is
    that file, by canonical paths: only the first counts, as that is what a
    shell runs, and an entry of `PATH` that isn't absolute counts for nothing,
    since the person's shell stands elsewhere. Otherwise it is the executable's
    path, as a word (below). An executable that can't be read gives `temper`.
  - *A path as a word,* for the executable and the three `TEMPER_` values alike:
    repeated slashes collapsed, then a leading `$HOME`, whole components of it,
    as `~/`, and the path alone as `~`. It stays unquoted where it can, so that
    the shell expands `~/`, which it does right after `=` in an assignment
    prefix too. A path needing quotes quotes only what follows `~/`
    (`~/'my dir/temper'`), and whole where no `$HOME` leads it (`'/my dir/temper'`).
    `$HOME` unset, relative or `/` leads nothing.
  - *The bound.* The words fit the answer's 49,216 bytes: a word for the binary
    that would pass it gives way to `temper`.
- **A change's panel, the subject `change`** (`forge.md`, 9.2): of a
  project's scope, its window unused, naming the change task's number
  (u64). On the wire it is `Watching`'s `change`, `Watching` naming
  which of the family's subjects a watch is, as `Reading` names a read.
  Its document, `ChangePanel`:
  - the task (u64); the repository's forge's number (u16), its owner
    and name; the change's branch and its landing branch (byte strings,
    each bounded as `Adopt`'s); its head: none before its branch is
    pushed, or a commit (32 bytes, as the forge connector keeps one);
  - its pull request: none yet, or its number (u64) and its address on
    the forge (a byte string of at most 2048 bytes, empty for a forge
    with no address: GitHub's `web`, Forgejo's `git` base, none for a
    fake, `forge.md`, 9.2);
  - its step: producing, opening, recreating, reopening, checking,
    gating, queued (its place, from 1 (u32), and the tasks ahead of it,
    nearest first, at most 64 (u64 each)), first, updating, resolving,
    repairing (its reason: CI, a gate by its number (u64), or a semantic
    conflict), landing, or landed (its merge commit, 32 bytes); verifying,
    red and cleared also carry the merge and at most 32 tier strings of at
    most 8192 bytes each. A tier string names its run and every full failing
    ID owned by or suspected of that landing, or explicitly `many tests`.
    The bound holds the 16 admitted IDs of up to 256 bytes each, their
    separators and tier/run words; IDs are never cut to fit the wire;
  - its hold: none, or why: stalled; past its repairs, its resolutions
    or its updates; its branch moved; its branch missing; its pull
    request closed; its pull request retargeted; a delegate or effect
    failed; a person rejected a gate. A held change's step is the one it
    was held at;
  - the task its step runs: none, or its number (u64);
  - CI at its head: unknown, pending, passed or failed;
  - its gates, at most 64, each with its number (u64), its kind (review,
    approval or check), whether it blocks (bool), and its verdict at the
    head: none, pending, approved, or changes asked;
  - its repairs, its resolutions and its updates, each used of its
    limit (u32 each);
  - its questions (`../domain/forge.md`, 8.11): its route, none, foreman
    or up; the questions taken, used of its limit (u32 each); and its
    open questions, at most 8 in the order taken, each with its asker's
    task (u64) and the question's number there (u32), the first line of
    its words (at most 256 bytes, cut on a UTF-8 boundary and marked),
    and where it stands: taken, with its foreman (its task, u64), asked
    of the change's requester, or answered and held for the change's
    dependencies. A change past its bound
    shows the question that held it among them, as its hold says.
- **The deployment's forges, the subject `forges`** (`forge.md`, 5.4
  and section 9): of the deployment's scope, with no fields, its window
  unused; on the wire `Watching`'s `forges`. Its document, `Forges`, a
  `ForgeRow` for each forge the store keeps and has not ended, at most
  `FORGES` (16), in number order, the playground's fake among them while
  it runs (`../protocol/forge.md`, 2.1): its number (u16); its canonical resource `host` (bytes, at most
  2,048, from the git authority including its explicit port, the same
  identity adoption uses); its kind (Forgejo or GitHub); its mode
  (connected, managed, owned or fake); and its connection: not
  connected; not set up; setting up, with its step when the box has
  named one (fetch, prepare or run), and while it fetches and the box
  has said how far, `fetched`: the binary (Forgejo or its runner), the
  bytes written (u64) and the size (u64), if known
  (`../protocol/setup.md`, 6.6), which a client reads as none where the
  size is zero or the bytes pass it, and which changes in place, no
  version kept for an older side; set up failed, with its optional step (fetch, prepare, run or connect)
  and why (text of
  at most 256 bytes, the box's line bound); connecting; connected, with
  the user's id (u64) and login (a byte string bounded as `Adopt`'s);
  or stopped, with the component's reason;
  `admin`, whether the forge is managed, its connect taking a site
  admin's token; `read_only`, its App's read-only choice
  (`../protocol/setup.md`, 17.7), false on Forgejo; `ended_stop`, none
  or `AdminStop`: not admin, refused with status (u16) and `SetupStage`
  (admin, bot, token, org, repository, file, app, installation or state),
  unmarked, or other. The diagnostic `ended` stays separate; the client
  selects its stop's words from this typed fact, never its text;
  its first repository, `first`: none, or the set up's org, name,
  whether it is the playground's copy (`playground`, appended), runner
  label (at most 64 bytes) and its state, `FirstState`, with no project:
  making; made, with the repository's id (u64);
  or failed, with its step, `CreateStep`: admin, org, repository (its
  bot's access with it) or file (`../protocol/setup.md`, 6.7); its
  intent, `starts_with`, `StartsWith`: `repository`, that `first` or none
  known yet; `copies`, `CopyAddresses`, the set up's, kept after they are asked;
  or `nothing`; its
  installations, `installations`, on GitHub, at most
  `INSTALLATIONS` (64, the configuration's `forges.installations` at
  most it) in id order, empty on Forgejo, each
  `Installation`: its id (u64), its account, none until the forge has
  heard it or its id (u64) and login (a byte string bounded as
  `Adopt`'s), whether it is suspended and whether it is a stray, never
  accepted through temper's link (booleans); its last connect's end,
  `ended`: none, or `ConnectStopped`, why the run of a `Connect` or an
  `Install` stopped, text of at most 256 bytes, in the setup host's
  words (`forge.md`, 5.4); on GitHub, its App's way, `app`: none,
  `Manifest` or `Pasted`, how the App its records hold was connected
  since the engine started, the forge domain's memory alone, so none
  after a restart and while no App is connected (`forge.md`, 5.4,
  Connecting); its `web` address, none on a forge with none; the
  repositories adopted on it, `repositories`, at most 16, each
  `Adoption`, its owner, name and project (u32); on an owned Forgejo,
  `owned`, `OwnedForge`, the Forgejo and runner versions the box pins
  (at most 64 bytes each, the runner's optional) and the path of the
  person's first password's file, never the password (at most 512
  bytes; `forge.md`, 5.4); `makings`, the repositories being made or
  failed on it, at most `MAKINGS` (64) in key order, each
  `RepositoryMaking`: its org and name, its `from` and `into`, and its
  state, `MakingState`: waiting; making; made, with the id (u64) and the
  branch (bounded as `Adopt`'s landing branch); adopting, with both; or
  failed, `MakingFailed`, with its step (`MakeStep`: admin, org,
  repository, file or adoption), its reason (`MakeFailure`: taken, empty,
  timed out, refused with status (u16), not allowed or other) and the
  forge's words (at most 256 bytes); and its
  offer, the forge domain's (`../domain/forge.md`, 4.1): offered, not
  under the box, or not offered, which is what "Set up a Forgejo here", "Set up again"
  and the words without the box follow (`forge.md`, 5.4). The read of the forges, the
  tools' `forge list` (`tools.md`, 5.5, area 6), is the watch's first
  document. Text prints `host: HOST`, and JSON carries `host` unchanged
  for a charter document's `forge` (`../protocol/configuration.md`, 13.7).
  *The client's forge side watches it once,* for the
  session, from when `me` is first delivered while the link is live,
  for every person, so that the engine judges the role: a watch
  refused or ended is kept as such, with no rows, and not opened again
  until the next sign-in. Its rows feed the Forges page,
  `ConnectForge`, the welcome and its store's state (3.5), and the
  Forges entry's dot.
- **A person's Away switch, the subject `away`:** of the party's own scope,
  with no fields, its window unused; on the wire `Watching`'s `away`, one
  subject for the deployment and each watcher's own rows, as `dismissed`
  (below). Its document, `Aways`: `rows`, at most 1,024, each
  `AwayRow { project, since, until, kinds, landed, off }`. *The client's forge
  side watches it once,* for the session, as it does `dismissed`, a watch
  refused or ended kept as such, with no rows: then the switch draws off and
  its act is not offered.
- **What was decided for a person, the subject `decided`:** of the project's
  scope, on the wire `Watching`'s `decided`, `DecidedSubject { project }`, its
  window unused. Its document, `Decisions`: `project`; `rows`, at most 64, each
  `Decision { number, task, at, question, took, why, overturned }`, the asking
  party's alone, `took` the option's words (`../domain/forge.md`, 5.9). Home
  watches it while open, as it does `status`.
- **A person's facts, the subject `dismissed`:** of the party's own
  scope (jig's `client/domain/contract.md`, 7.5), with no fields, its
  window unused; on the wire `Watching`'s `dismissed`. One subject for
  the deployment: the core names each watcher's party, and each
  watcher gets its own party's document (`../domain/forge.md`, 5.5). Its
  document, `Dismissals`: `dismissed`, the party's rows in their key's
  order, at most `DISMISSED` (8,192), each a `PersonFact`: `ToBegin {
  project }`, or `GoalFrom { project, chat, goal, at }`, that the person started
  goal `goal` from chat `chat`'s document at the client's clock's `at`. A start whose
  `forge.judge_projects` is past an eighth of that bound is refused, a party
  holding at most one `ToBegin` a project (`../protocol/configuration.md`,
  section 7). A party's rows are bounded by `DISMISSED` too, and all rows by
  `limits.dismissed()`: over either, `Dismiss` answers too large, and the link
  alone is lost: the goal has started, and the client draws no refusal for it. A
  fact reaches the watch of every client of the party's once it
  commits. *The client's forge side watches it once,* for the session, as
  it does `forges`, from when `me` is first delivered while the link
  is live, for every party: a watch refused or ended is kept as such,
  with no rows, and not opened again until the next sign-in. Its rows are
  the forge side's facts, which Home reads to compute To begin (3.5).
- **A project's changes, the subject `changes`:** of the project's scope
  (jig's `client/domain/contract.md`, 7.5), for a party with a role
  there, its window unused; on the wire `Watching`'s `changes`,
  `ChangesSubject`: `project`. `Changes`' page watches it while open. Its
  document, `ChangesPanel`:
  - `project`;
  - `landing`, the branch to name while nothing is queued: the newest
    landing's, else the default branch of the project's home repository,
    else of its first, else empty;
  - `queues`, at most 32, by repository then branch, one per landing branch
    with ready changes, a change owning its queue, or red (`forge.md`, section 3), each a
    `ChangesQueue`: `repository`, `owner/name`; `branch`; `red`; and
    `changes`, in order (`../domain/forge.md`, section 9), each a
    `ChangesQueued`: `task`, `title` (at most 256 bytes), `ready_since`
    and, on a queue's first row, `checks_since`, when the step it is at
    began, both nanoseconds since the epoch;
  - `more`, the rows past `forge.changes_rows` (32 in `small`), counted
    across the queues, which fill in order; the changes queued are the
    rows listed and `more`;
  - `landed`, at most `forge.changes_landed` (8 in `small`), newest first,
    across the project's repositories, each a `ChangesLanded`: `task`,
    `title` and `at`, the landing log's (`../domain/after-landing.md`,
    section 10);
  - `landed_more`, what `landed` cut.

  It is marked as a change enters, leaves or moves in a queue, as the step
  of the change that owns one changes, as a branch's red changes and at each
  merge.
- **What changed since the person last looked, the subject `status`:** of
  the project's scope, for a party with a role there, its window unused;
  on the wire `Watching`'s `status`, `StatusSubject`: `project`. `Home`
  watches it while open. Its document, `ProjectStatus`, is each watcher's
  own, the core naming the watcher's party (`../domain/forge.md`, 5.6):
  - `project`;
  - `landed`, u64, the changes landed into every landing branch of the
    project; `last_landed`, the time of the latest, option;
  - `brief`, option, `StatusBrief`: `sentence` (at most 200 bytes), `lede`
    (at most 600), `at`, and `landed`, the count it covers;
  - `previous`, option, the party's last look of the sitting before; and
    `seen`, u64, the `landed` count then;
  - `queued`, u32, what the subject `changes` counts; `red`, whether the
    home repository's landing branch is red; and `branch`, its name, at
    most 1024 bytes, empty with no home.

  Times are nanoseconds since the epoch. It is marked with the subject
  `changes`, and at a look and a brief.
- **The changes waiting on a person's review, the subject `approvals`:** of
  the party's own scope, with no fields, its window unused; on the wire
  `Watching`'s `approvals`. One subject for the deployment: the core names
  each watcher's party, and each watcher gets its own rows, which the root
  chooses by the watcher's role (`../domain/forge.md`, 8.12). Its document,
  `Approvals`: `rows`, at most 32, oldest wait first, each an `ApprovalRow`:
  - `task`, the change's; `project`; `goal`, option, the goal it was made under;
  - `forge`, u16; `kind`, `Kind` (`forgejo` or `github`); `owner` and `name`,
    at most 1024 bytes each; `pull`, the pull request's number; `address`, at
    most 2048 bytes, its files page on the forge's web address
    (`forge.md`, 4.1), empty for none;
  - `title`, at most 256 bytes; `since`, when the change began waiting at
    its head;
  - the approval it waits on: `scope`, 0 for the deployment's rules or the
    project's number; `rule`; `role`; `required`; `counted`, the people
    counted so far;
  - `behind`, the changes waiting behind it in its landing branch's order;
    `whole`, whether its reviews were read whole;
  - `yours`, `ApprovalYours`: `counts`, `no_account`, `too_few` or
    `other_role`, whether the watcher's review would count and, if not, why;

  then `more`, the rows past the bound. Times are nanoseconds since the
  epoch. Every client of the party's watches it once, for the session (3.5,
  *What the client keeps*). The engine fills `kind` and `address` from the
  forge's configuration as it does a change's panel's pull request's
  (`forge.md`, 9.2).
- **A proposal's items, the read `ProposalItems`:** its answer,
  `ProposalItemsAnswer`, holds the items and `owned_approval`, whether one of
  them is a landing rule with an enforcing approval covering a writable
  repository on an owned Forgejo (`../domain/forge.md`, 8.12), for its card's
  words (5.13).
- **The forge's documents' records:** its other reads, subjects and
  documents, and its other requests, beside `Adopt` and `SetUp`
  (`forge.md`, section 9).
- **Bounds:** the schema's ceilings bound every record at both ends,
  and the generated worst cases price each. Within them, the client
  bounds its fields by its own limits (4.4), and the engine's forge
  connector refuses what is past its own as too large (4.7), so a
  record a client can make always decodes.

### 4.7 The outcome in words

| Outcome | The form says |
|---|---|
| adopted | "Adopted OWNER/NAME on forge F as ROLE." then "It is the project's home." if so; "temper may: KINDS." in the forge's words; "temper's forge user can only read OWNER/NAME on forge F: work there fails until it has write." if it may not push and the role isn't context; on GitHub, "Commit statuses stay temper's own: GitHub's rules cannot require them."; "temper's forge user may not read the branch protection: the forge shows it to repository administrators only." if unknown; "It merges by STYLE."; then each rule as `forge.md`, 5.1 words it, on Forgejo its branch protection alone |
| refused by the core | jig's words for its refusal |
| no such forge | "No forge F is configured." |
| not connected | "Forge F is not connected.", linked to the forges' page (3.5) |
| not installed | "temper's App is not installed on OWNER.", with "Install on another account" (`forge.md`, 5.5) |
| not granted | "OWNER/NAME is not granted to temper's App.", linked to the installation's page on GitHub (`forge.md`, 5.5) |
| not found | "No repository OWNER/NAME on forge F that temper's forge user can see." |
| no access | "temper's forge user may not read OWNER/NAME on forge F."; `temper ctl` adds "Forgejo collaborator reads can require repository admin even when temper's forge user already has write. A repository administrator must grant temper's forge user admin; then run repository adopt again with a new key, preserving its options, and verify the human's required project role with project show. If setup could not grant access on a managed or owned Forgejo, its site admin's token needs write:repository. Check the setup failure in the server's log." |
| refused | "The forge connector refused it: another project writes OWNER/NAME, the project has a home already, or branches under PREFIX are not this deployment's." |
| in flight | "OWNER/NAME is being adopted already; try again when that ends." |
| too large | "A field is past the forge's bounds, or the forge's repositories are full." |
| unreachable | "Forge F did not answer; nothing was adopted. Try again." |
| invalid answer | "Forge F answered what temper did not expect; nothing was adopted." |
| unknown past the retention | "It is not known whether OWNER/NAME was adopted." |

KINDS lists what temper may do, in this order and these words: read;
push; open pull requests; land; review; set statuses; comment; create
issues; create and delete branches. STYLE is "merge", "squash" or
"rebase". The forge connector maps the forge client's errors to these
reasons (section 8). The links are the addresses the forge's record keeps,
shown as external links and never opened by the client (6.5).

### 4.8 The playground

The playground is ordinary records, made when the person chooses it:
a fake forge, the scripted model's account, the deployment's, and the
playground's project, with `playground/widget` adopted into it; its
reset removes them, and the person is on the welcome's Forge step as if
they had never tried it (`../protocol/setup.md`, 8.4). The forge's side
reads it, starts it and resets it; the engine makes and removes it. The
client removes no account and closes no chat of its own.

- **Read once a link is live.** The forge's side reads `playground`
  (4.6) when the link first becomes live, again each time it becomes
  live after it was not, again once a start or a reset it sent is
  answered, and again while its answer reads starting or resetting,
  until it reads running or none. It keeps the last answer, and drops
  the answer of a read it has replaced. None, and no answer yet, both
  mean no playground is shown. *The playground is read* while its last
  answer names one, starting, running or resetting; *it runs* while
  that answer reads running.
- **Started when chosen** (`../protocol/setup.md`, 8.4). "Try the
  playground", the welcome's, sends `StartPlayground` while the
  playground is offered (3.5, The welcome) and none is read, one start
  at a time, held by the forge's side across pages, and opens
  `NewChat`. While the start is out, or the read answers starting,
  the playground's start on `NewChat` says it is under way, its
  suggestions waiting; once it failed, why, with "Try again", which
  sends a start under a new key (5.6). Answered running, or unknown,
  the side reads the playground again. It sends no start of its own
  beyond the person's: a start a restart cut short is the engine's to
  mend, with no client (`../protocol/setup.md`, 8.4).
- **Reset by "set up temper".** The banner's "set up temper" (5.6)
  sends `EndPlayground` while a playground is read and no reset is
  under way. From its sending, the side holds the playground as
  leaving, and the banner says "Leaving the playground…"; once the read
  answers resetting, `NewChat` shows no playground's start.
  Answered reset, or its read answering
  none while it is leaving, the side drops the playground and opens the
  welcome's Forge step, `Welcome { Forge }` (`/welcome?at=forge`): the
  store as a person who pressed "Set up temper" from the welcome finds
  it (3.5, The welcome). Answered not offered, there being no
  playground, it holds it as leaving no more, and reads again. A read
  answering resetting, a reset another client sent or one a restart
  cut that the engine resumes (`../protocol/setup.md`, 8.4), holds it
  as leaving the same way, until the read answers none.
- **The scripted model, the playground's alone.** Only the playground's
  project allows it, and no other project does (`../protocol/setup.md`,
  8.4; jig's `domain/accounts.md`, 4.2). The client names its LLM
  provider from the read and leaves its account out of the accounts'
  lists and counts and its provider out of the ways to connect a model
  (3.5, 5.5). A model is usable for the welcome, Ready and the
  composer's line by an account of another provider: the person's own,
  from the `accounts` document, or, for a person whose deployment role
  is owner, the deployment's, from the `deployment` document (jig's
  `client/domain/contract.md`, 7.1), an account held by the deployment,
  uncut, with a usable source; that document counts only by such an
  account, and while it is not delivered, or is cut, it adds none. An
  account of the person's own added while the playground runs, or
  before a start over, stays theirs: it is not the playground's, and
  its reset leaves it.
- **One `deployment` watch,** for a person whose deployment role is
  owner alone, while `me` says the role and a playground is read; once
  either ends, the side drops it. For any other person it opens none.
- **What shows:** while the playground runs, or is leaving, the
  banner's one line above the main of `Chats` and `NewChat`, and while it
  is leaving on every other page as well (5.1, 5.6); the playground's start,
  on `NewChat`, while a playground is read that is not resetting
  and the person is in its project, or in none while its project is
  not made yet (3.5); the context line, on the playground's start and on a
  chat's page in its project, while it runs; on the Forges page its
  forge, and on the accounts page its account, each in a section of
  its own, neither page offering anything to add (3.5; `forge.md`,
  5.4). Each is worded in 5.6. Nothing hides them but the reset.
- **State:** the last answer, the pending read's key, at most one, the
  start's key while it is out, at most one, and its failure, the
  reset's key while it is out, at most one, whether it is leaving,
  whether the link was live at the last pass, and the `deployment`
  watch's key and the playground's provider while it is open, at most
  one. Bounded by the record's bounds (4.6).

## 5. The view

### 5.1 Its shape

`temper-client-view` builds temper's pages as skein-ui's tree (skein's
`ui.md`), one owner, temper's:

- **Its state:** skein-ui's view (the last tree, a spare, node ids never
  reused); temper's phrasebook, English, with jig's words for jig's
  enums and a decision's facts (`jig-client`'s `words`) under it,
  overridden phrase by phrase where temper words them otherwise
  ("Funds 40 from your pool · 120 left, 80 after"), and
  temper's own phrase for what `jig-client`'s `words` does not say (an
  executor, an outcome); temper's bindings for the last tree; its
  limits; the instant its words next change (5.12).
- **`build`:** at the shell's time, when the domain's shown count moved
  or that time has reached the instant its words next change (5.12),
  it builds the page in one outline (skein's `ui.md`, section 14), in
  the mockups' order:
  - *the rail (5.3):* on a project page, every page but `Welcome`
    while a project is open and the link is neither signed out nor out
    of date, in place of the banner below. In the browser the tab bar
    (5.3) follows the main;
  - *a banner,* on any other page: temper's name; the project the
    person is in alone,
    as the switcher, a disclosure named by the open project's name,
    "Projects" while none is open (section 11), which holds the
    projects to choose from, the open one first and pressed, then the
    others, each a button by the name `me` gives it, which no other
    project of the deployment shares (jig's `domain/core.md`, 6.1),
    its visible and accessible name the same words; with no project,
    "No project yet" in its place (3.5, With no project yet);
    the navigation (5.3) after the switcher, as in
    the mockups; while the playground runs and is not leaving (4.8),
    "Set up temper", a link that sends `EndPlayground` (5.6); the
    link's state, one word for each of
    `jig-client`'s `Link` states ("Signed out", "Connecting" while
    starting, "Live", "Offline", "Out of date"), and "Behind" while a
    page's watch reopens; the person's name and "Sign out"; all but
    temper's name (text, not a link: the navigation's Inbox and Chats
    lead home; the user's pick, 10-23) in one disclosure, "Menu",
    which a browser folds
    behind its ☰ at phone width alone (`browser.md`, 2.5) and the
    terminal and the fake person find whole; on `Welcome`, temper's
    name alone until the deployment's read answers, then temper's
    name, temper box's line and the welcome's steps alone (3.5, The
    welcome);
  - *the accounts' banner,* while no account the person's work may use
    is usable (the `accounts` watch; jig's
    `client/domain/contract.md`, 7.1), but in the playground's project
    while the playground runs or is leaving, whose work its scripted
    model runs (4.8): a region named "LLM account", its words and three buttons
    (5.5), on every page but the accounts page, `ConnectModel` and
    `Welcome`;
  - *the playground's banner:* a region named "Playground", one line,
    its words and its link "set up temper" (5.6); on `Chats` and
    `NewChat`, while the playground runs or is leaving
    (4.8); on every other page, `Welcome`, the accounts and the forges
    pages and a chat's page among them, while it is leaving alone;
    above the main, after the accounts' banner where both show,
    the rail's or the banner's "Set up temper" being the way out while
    it runs (the user, 2026-11-03); while the link is signed out or out
    of date, none;
  - *the main:* the page at the address (3.5), its title its level-1
    heading, but `Home`'s, whose headline changes with what it says:
    "Home".

  Then skein-ui diffs it and keeps the tree.
- **The pages not about the work** (jig's `client/domain/pages.md`,
  section 8). While the link is signed out or out of date, `me` has
  ended (jig's `client/domain/client.md`, 8.2). The banner is then
  temper's name and the link's state alone, no navigation is offered,
  and the main, in place of the page at the address, is one paragraph
  of temper's words with no heading:
  - *signed out:* in the shell's words, which the view takes at
    construction: the terminal's "Signed out." (6.6); the browser's
    "Signed out. To sign in, open a sign-in link temper printed."
    (`browser.md`, 2.4);
  - *signed out, with a way of signing in:* in the browser alone, in
    place of the paragraph, the sign-in page: temper's mark, the
    heading "Sign in", and a link for each way the view was given at
    construction, "Continue with Google" to `/sign-in/google?then=PATH`,
    `PATH` the page's printed address (`../protocol/sign-in.md`,
    section 7);
  - *signed out by Sign out, with a forge that signs in through
    temper:* the sign-in page, under its heading, for each such forge,
    "You signed out. FORGE keeps its own sign-in: sign out there too.",
    FORGE the forge's name (`forge.md`, 5.4), read from the page's facts
    (`../protocol/sign-in.md`, 9.3 and 9.6);
  - *signed out at the authorization:* the `Authorize` place's sign-in
    page, "to continue to FORGE" under its heading, its link's `then`
    the authorization's address (`../protocol/sign-in.md`, 9.3);
  - *signed out, the link's sign-in not taken:* while the link's view
    says why its last sign-in installed nothing and the link stays
    usable (jig's `client/domain/client.md`, section 12,
    `not_signed_in`), the paragraph says so in place of the shell's
    words: full, "All N sign-ins are in use; the earliest ends MOMENT
    UTC. This link stays usable until LINK UTC; after that, opening it
    has `temper serve` print a fresh one." (N the deployment's room,
    MOMENT the earliest saved expiry and LINK the link's own, each with
    its date beside its clock, "Oct 24, 09:30", as the accounts page says
    its moments, 5.5: a sign-in's life is a day or more, so a bare clock
    would name tomorrow's as today's; the minute is the expiry's own, so
    the link serves a little past what it says); busy, "temper is busy. This
    link stays usable until it expires: open it again in a moment."; not
    ready, "temper is starting or stopping. This link stays usable until
    it expires: open it again in a moment.";
  - *signed out, the link spent:* while the link's view says why it can't
    sign in, the paragraph says so in place of the shell's words and of the
    sign-in page, in either shell, saying only what whoever holds the link already knows: used,
    "This link was already used. A fresh one is in your terminal."; past
    its expiry, "This link has expired. A fresh one is in your terminal.";
    any other, a real link of an older box or one that never was, "This
    link isn't from this temper. Use the one its terminal printed." The
    reason is the engine's: it remembers only its last 8 used or expired
    links in memory, so a forgotten link, after a restart included, reads
    as unknown (jig's `protocol/client.md`, section 8). A page reached with
    no link keeps the shell's words;
  - *out of date,* the engine serving no version of jig's records that
    this client speaks: "This temper is out of date: run the temper the
    engine runs." What is pending stays in the small store (jig's
    `client/domain/client.md`, 8.2).

  Both are phrases of temper's phrasebook: `jig-client`'s `words` has
  none for the link's states, and temper arranges no page of jig's
  (section 1). The page at the address is shown again once the link is
  starting, live or offline. When jig's records are served and the
  forge's are not, the link stays live, and the form says its own out
  of date (4.4).
- **`act`:** a shell's act on a node is resolved by skein-ui in the
  last tree to its binding, and decoded by temper's bindings into the
  domain's action. A node gone, or a token of an older tree, decodes to
  nothing (skein's `ui.md`, section 6).
- **Every node a person acts on has a role and a name,** since skein's
  fake person finds nodes only by them; nothing a person must know is
  said by style alone (skein's `ui.md`, section 8).
- **The look:** the mockups' (`mockups/style.css`),
  as style tokens each shell interprets or ignores; the terminal prints
  none, and the browser's stylesheet draws them (`browser.md`, 2.5).
  The composers are text areas that submit on Enter (skein-ui's
  `SubmitsOnEnter`), for the browser, and whose words may hold line
  breaks, counted within their bounds (5.4; `browser.md`, 2.5).

### 5.2 The forge's view

- **Words:** the forge's phrases in temper's phrasebook: the connector,
  resources, patterns and kinds (4.1); "Shown on the forge." (4.2);
  a change's panel's: its tag, its pull request's line, its steps,
  its step's lines, its holds, its gates' rows, its bounds and its
  panel's slot words (5.10); the form's title, field names, roles,
  rules and button (4.4), its title by the project's number included;
  the picker's nodes, "Not listed" and its refusals (4.4); the outcome
  (4.7); the form's out-of-date sentence (4.4); the forges' page's
  (`forge.md`, 5.4 and 5.5); the navigation entries. Names (owners,
  repositories, branches, logins) are copied in, never translated.
- **The parts:** a connector, resource, pattern or kind is one text
  node of its words; a part the forge's side does not read is the
  paragraph of 4.2; a part it reads is shown as `forge.md` says.
- **The form's page:** a level-1 heading (the title), then the form's
  nodes in the order of 4.4: the forge's field, the picker's nodes,
  text fields with their values and write counts, the role as a choice
  group of three choices, home and ci as checks, the button, and after
  them a status for progress and an outcome, or an alert for a refusal.
- **The forges' page:** a level-1 heading, "Forges", then "Your
  forges" with their count and "+ Connect a forge", then a disclosure per
  listed forge, folded, named by the forge's name, its summary the
  forge's number, kind and `web` address, its connection and its line,
  holding its rows and its actions, "Remove" last, its confirmation in
  its place (`forge.md`, 5.4); the playground's forge in a section of
  its own; a pasted value a secret field (3.6), a
  link an external link (6.5), and a connect's or an action's status
  under its button. While the `forges` watch is refused, the page says
  so in jig's words; until it has delivered, its headings alone.
- **`ConnectForge`:** its Settings path and level-1 heading, “Connect a
  forge”; the shared choice cards or the selected way’s panel, with its
  fields and acts. No Steps list or outer Choose card. A running or
  connected forge opens its row on Forges, where progress and installation
  acts live (`forge.md`, 5.4).
- **The accounts page:** its level-1 heading, “Accounts”, its lede and
  account groups, each row a folded disclosure. Its summary names the
  account, holder, deciding source’s tag and line (5.5). Opening adds
  source order, source acts, rule, use facts and account acts, without
  repeating the summary’s name, models, Ready or Pays for. A secret field
  is the row’s (3.6), and an act’s status stays under its row. Refused
  reads say why; undelivered reads draw only the headings.
- **`ConnectModel`:** its Settings path and level-1 heading, “Connect a
  model”; shared plan cards, then API key rows, or the selected way’s
  panel and acts. A sign-in’s address is an external link (6.5). Success
  lands on Accounts with its confirmation and Start a chat, without a
  Ready page. The holder switch is one quiet link at the foot.
- **A flow's parts,** shared by `ConnectForge`, `ConnectModel` and the
  welcome, are the view's `flow` module. Choice cards, panels and controls
  use the shared connection tokens (`browser.md`, 2.5); progress is a list
  whose lines retain their state in words. Forges rows use the compact
  live list. The welcome alone keeps its frame’s Steps links (3.5).
  Settings flows have no steps bar, outer Choose card or Ready card.
- **Bindings:** writing a field, choosing a role, toggling home or CI,
  choosing the picker’s forge or installation, checking repository rows,
  searching and nearing the list’s end, sending its plural adoption,
  making or copying, each forge action and navigation entry.
  Typed input is made here: the forge number's digits become a number,
  the checks' words a list, or nothing when they are not one, which the
  form marks invalid.
- **No drawings.**

### 5.3 Navigation

**The rail** (the user's greenfield mock's `home.html`), on a project
page (5.1): a navigation named by the open project, in this order:

- temper's name, a link to the open project's `Home`, named "temper
  home";
- the switcher (*Projects in the rail,* below);
- "Home", to the open project's `Home`; "Needs you", to *Needs you*
  filtered by the open project, with the count of what needs the person
  there (*What the client keeps,* 3.5), omitted when zero; "Goals", to the project's `Goals` with the filter All, only while the project's
  kind 7 shows them or the project holds a goal (`Domain::has_goals`, below), the setting hiding an offer and not a person's own work; "Chats", to `Chats`; and "Changes", to `Changes { project }`, only while
  kind 7 shows them; current on `Home`, on `Inbox`, on Goals, a tracked goal and a task whose document names a goal, on
  `Chats`, `NewChat` and `Chat`, but a configuring chat's, whose current entry is
  "Settings", and on Changes. On Goals, on a goal and on Changes their
  entry stays shown, so that nothing goes from under the person; a hidden
  page keeps its address, hiding removing the way in and not the page.
  Until kind 7 is read, or if its read is refused, the optional entries
  are hidden (5.13), but for Goals while the project holds a goal;
  Whether a project has a goal is `Domain::has_goals(project)`, a fact the
  domain keeps whatever the rail does, from a live watch and one ended
  read of the goals' query with window 1, per open project;
- `div.temper-rail-new`, one dashed box split in two: "+ New chat", named "New
  chat", a link to `NewChat` (`/chats/new`); and at its right a 36-pixel button
  behind a dashed line, named "More ways to start work", drawn as a chevron, its
  name hidden by style, opening skein-ui's menu (`Role::Menu`; skein's `ui-web.md`,
  section 6), a panel under the entry 288 pixels wide, past the rail's edge and
  over the page. Its items, a name and a line each: "Chat", "Ask a question or ask
  for a change.", to `NewChat`; "Goal", "Agents plan a larger change and land it.",
  to `NewChat { draft: true }`, a goal being started from a draft, while
  `drafts_offered` (3.5); then, after a rule, "All ways to start work", muted, one
  line, to `Settings { project }`, or `HowWorkRuns` once it has an address. Each
  half takes the hover's grey; open, the button keeps it and its chevron turns ink.
  Skein's keys, arrows and Escape; a press on an item or outside closes it; a focused item
  takes the hover's grey. A phone doesn't show the entry. Which templates it offers
  is fixed until `../draft/template-places.md` is taken up;
- at its foot: the Away switch, while 3.5's *Away* shows it; while the
  playground runs and is not leaving (4.8),
  "Set up temper", as the banner has it (5.1); the link's state, as
  5.1 has it, the browser hiding it while live; "Behind" while a
  page's watch reopens, beside the link's region in the browser, so
  that it shows while the link is live, and in the region in the
  terminal, whose first line reads it (6.4); "Settings", to `Settings
  { project }`, current on every tab and page 3.5's *Settings* names,
  with the dots "Forges" and "Accounts"
  carry below, each its state in words; the person's name, a line in the
  terminal and the person's menu in a browser; and "Sign out did not reach
  the engine." while `me` or the page cannot say otherwise (6.6), the
  terminal's "Sign out" being the `Accounts` tab's (3.5);
- the person's menu, a browser's: the person's name is the button of
  skein-ui's menu (`Role::Menu`; skein's `ui-web.md`, 7), in a
  `div.temper-rail-person`, drawn as the foot's other rows. It opens a panel
  above the row, as wide as it, in the look of the switcher's menu. Its items:
  "Accounts", to `Accounts`, and "Sign out", the `Accounts` tab's act. A press
  on an item, Escape and a press outside close it, the focus back on the row
  unless that press took it elsewhere. Open, the row keeps the hover's grey. A
  phone has no foot, and signs out on the `Accounts` tab (3.5).

"Forges", "Accounts" and "Adopt a repository" are no entries of the
rail's navigation: they are `Settings`' tabs and the Repositories tab's (3.5). On a phone, at 760 px and under, the rail is a top
bar showing the switcher alone, and the tab bar closes the page: in the
browser, a navigation named "Sections" after the main, with "Home",
"Needs you", then "Goals" and "Chats" while the rail shows Goals, "Chats"
and "Settings" while it doesn't, drawn at phone width alone
(`browser.md`, 2.5); with Goals shown, Settings is reached by the
switcher's "Settings for PROJECT". The Away switch is a chip in the top
bar, on the rail's terms. The terminal has no tab bar.

**The banner's navigation,** on a page that is no project's (5.1).
In the mockups' top bar's order: "Inbox", with the count of what needs
the person from `me`, omitted when zero; "Chats"; then, for the project
the person is in, "Changes"; and after them "Adopt a
repository", to `Adopt { project }`; then "Forges", to `Forges`, with
a dot, amber while a forge sets up or connects and red while one needs
the person (`forge.md`, 5.4), its state also in words, "a forge is
setting up" or "a forge needs you", a text node of its own after the
entry's name; current on `Forges` and `ConnectForge`; and last,
"Accounts", to `Accounts`, with a dot, amber while an account is
limited and red while one needs the person (5.5, Each state), its
state also in words, "an account is limited" or "an account needs
you", a text node of its own after the entry's name, read from the
person's own accounts, the playground's provider's left out: red for a
deciding source refused, needing a sign-in, misconfigured or
unloaded, before amber for one limited; current on `Accounts` and
`ConnectModel`; both whatever the project. With no project chosen, the
project's entries are not offered. The mockups' "Tasks", a list of a
project's tasks, is not in the navigation (the user's pick, 10-23): Goals covers what a task list is for. Their "Workers", the deployment's
hosts, is no entry either: Settings' Builders tab reads them (3.5), and
changing them is the engine's configuration's. "Adopt
a repository" and "Forges" are offered whatever the person's role: the
engine judges each, and a refusal says why, since the client never
computes what a party may do.

**Projects in the banner** (`../draft/first-run-projects.md`, 7). The
banner names the project the person is in alone, by the name `me` gives
it, and a chevron; pressed, a switcher lists the person's other
projects from `me`, the open one marked, each choosing its project as a
project's entry does. On a narrow screen the switcher is the menu
panel's first row (`browser.md`, 2.5).

**Projects in the rail:** the same switcher, a bordered box with its
chevron, its menu a panel under it: each project, the open one first
and pressed, then, after a hairline, "Rename PROJECT" where the
rename is offered, the form "New project", and "Settings for PROJECT",
to the open project's `Settings`. Each project's row says, in small
type at its right, what it holds, from the `forges` watch's rows
(4.6), the playground's project from the playground's read (4.8): its
one repository, "work/hello"; with several, its home then how many
more, "infra/deploy +2"; with none, "no repository"; and nothing
while the watch is refused, ended or not delivered. A long name is cut
with an ellipsis before its count, never the count. `me` does not say
which repository is a project's home (4.4), so its first adopted, the
first its rows list on the forge numbered lowest, stands in for it.
The repositories in a row follow the forge's repository ids ascending,
not the time they were adopted.
In the browser the row is two siblings, since a link can't sit inside a
button: the project's name, the button as above, and the small words. A
repository with an address (`forge.md`, 5.4, *A repository's link*) is
the link "work/hello ↗" there, in the same place, size and colour,
underlined on hover, and opens the repository's page in a new tab; the
project's name alone switches project. "+2" stays outside the link, which
goes to the repository named. "no repository", the playground's and one
with no address stay plain words.
With no home known, "3 repositories" is the row's words; the stand-in
leaves that state unreached until the client is given the home. In the
terminal a row reads "PROJECT · work/hello", one run of words. Choosing
a project keeps the page where it can (3.5): on `Home`, `Changes`, `Settings`,
`Repositories` and `Configure`, that project's page; on `Forges` and
`Accounts`, the same page, the words naming the project changed.

The acts of both, each one act and one request (3.6):

- **Rename,** on the open project's row in the banner, and after the
  rows in the rail, offered while `me`'s role on
  that project is owner, the role jig gives, which the context line
  reads too (5.6); jig judges the request (jig's `domain/people.md`,
  5.1). Pressed, the open project's row is the form, "Rename PROJECT",
  in the banner and the rail alike, and the rail's "Rename PROJECT"
  leaves its acts while the form is open. The form holds the name's
  field, filling the row, its value the project's name and the focus on
  it, then "Save", primary and small. The field's Enter or "Save" sends
  jig's `Rename { project, name }`, its name without the spaces at its
  ends, the form's acts disabled while it is out; Escape closes the form
  and sends nothing: the form's "Cancel" names it (skein's `ui.md`,
  6.1), which the terminal shows and the browser's sheet hides. An empty
  name, or the name unchanged, sends nothing and closes the form,
  keeping the old name. Answered renamed, the form closes, and `me`
  brings the new name wherever the project's name shows; refused (the
  name taken, its shape, the role), the form stays and says why under
  its field. A closed form leaves the focus on the switcher's summary
  (skein's `ui-web.md`, 7), or on a page's own field where the page asks
  it (a new chat, Configure, a composer after a send, 3.5).
- **A new project:** the list ends with a form, "New project", holding
  a name's field, "+ New project" its placeholder, and "Create".
  "Create", or the field's Enter, sends jig's `MakeProject { name,
  policy: 1 }` in the deployment's scope, on the stock policy
  (`../protocol/configuration.md`, 13.4); an empty name sends nothing.
  Answered made, the field empties and the client opens the project
  once `me` lists it, on its `Home`, with no repository (3.5).
  Offered whatever the person's role, as above: jig judges it (an owner
  of the deployment), and a refusal says why under the field.
- **Its state:** the rename's form, at most one, with its project and
  words; the new project's words; each within jig's name bound; at most
  one request out of either, with its key; and the refusal, with the
  form it answers. Choosing another project, or a page of no project,
  closes the rename's form.

### 5.4 Limits

The view's limits are skein-ui's (skein's `ui.md`, section 15), derived
from the domain's: each page's nodes at its conversation's bytes (5.15)
and its other windows' most rows, the
forge's form and its outcome's lines, the kinds' list at its longest,
and one paragraph per part not read. Their text across a tree,
`text_bytes`, is the nodes outside a window's turns at a node's most
text, and each of a window's lines at what its reader may make of its
turn's bytes (skein's `ui.md`, section 10) and its disclosures' names:
a lower cap would be a guess, which skein-ui's builder turns into a
panic on the page past it. Their patches are four a node, the most a diff emits (skein's
`ui.md`, section 7). skein's text shell counts its worst-case print from these limits,
and reserves for each print only what that page's tree may print
(6.8).

A note's body and a task's words are `long_text`, 131,072 on each of temper's
clients, past a field's `text` (jig's `client/domain/client.md`, section 14), and
a chat's document's section is read within the view's `text_bytes`, which prices one.

A line of a conversation holds words the store took, and they are `long_text` too:
a person's message or reply, up to `people.words` (8,192 on `temper init`'s box),
and an agent's message, question or answer and a result, up to
`tasks.message_bytes` and `tasks.result_bytes` (10,660 on it). Each draws whole in
the browser and the terminal, never cut (jig's `client/domain/client.md`, section
16). The window's bytes do not move: a line weighs what it holds within
`document_bytes`, including `LINE_ROOM` for each line and its parts, so a window
holds fewer lines the longer they are. The view prices the bytes at Markdown's
maximum nodes and text, plus ten structural nodes per `LINE_ROOM`, with room for
their names and text. A person's words waiting to be read total at
most `document_bytes` too (jig's `client/domain/client.md`, section 10), and the
view prices them so. A line past `long_text`, or a page of lines past
`document_bytes`, has its page refused and its read ended (5.15): no box `temper
init` makes reaches one (section 11).

A composer, a chat's message, a new chat's first words, an agent's or a goal's
message or a question's reply, takes what a person types or pastes up to what one
message holds: `Me`'s `words_bytes`, the engine's `people.words`, within `long_text`
(jig's `client/domain/contract.md`, 5.1). Past that nothing is cut, dropped or kept
stale: the words stay as typed, Send is disabled, Enter makes a new line, and a line
under the composer says how far over they are, counting the characters past the
bound: "That's 3,200 characters over what one message holds." The client holds an
edit's first `long_text` bytes and counts the characters past them in that line. A
browser's field keeps them, since a field's value is written only when its owner's
count moves (skein's `ui.md`, 3.4), and a composer drawn anew shows what the
client holds. A suggestion's words and a choice's stay within `CUE_BYTES`, as a
cue's and a button's. The view prices one composer's value at `long_text`.

A chat's first message reads back whole. Past the engine's `documents.text_bytes`
(512 on `temper init`'s box) its spec comes marked cut, and the chat's page reads
the rest through `TaskText`, as a goal's page does (5.8), drawing what it has, ending
in `…`, until it has it all or the read ended. The view prices that line at
`long_text`. It prices a question read whole (3.5, *Whole*) the same, one to a page,
with at most 16 choices (`question::CHOICES`).

temper's client takes every document within the bounds the engine's
configuration gives its documents. A path's segments have a bound of
their own, 9, apart from the windows' rows (jig's
`client/domain/client.md`, section 14), at least the most the engine
admits in a name (`../protocol/configuration.md`, section 7), so a page
of the forge's work is never refused for a path. A path longer than a
node's text shows its last segments after the mark `…` (as jig's
`client/domain/view.md`, section 7 cuts one), which adds nothing to the
view's limits. An authority's entries have a bound of their own, 16,
apart from the windows' rows (jig's `client/domain/client.md`, section
14), at least the most the engine admits in an authority's list
(`../protocol/configuration.md`, section 7), so a task's page is never
refused for its authority, nor an amendment drafted from part of one.
Each of an authority's lists shows at most the rows per window, then
jig's mark (jig's `client/domain/view.md`, section 7), which adds
nothing to the view's limits.

Both presets receive the dark theme's turn and the CSV chat's turns whole:
three rows (`rows`, which bounds a turn's messages and a message's blocks
as well as a page's lines), 512 bytes of field text (`text`), and 131,072 bytes
of a turn (`long_text`): the turn being written (jig's thread's `live`), and a
committed turn's shared text (`turn_text`, which jig's transcript prices its
calls' names first: jig's `client/domain/client.md`, section 14). The CSV
chat's first turn fits with its
`delegate` input of about 1.2 KB, so its card appears at that call (5.7).
A conversation's window is bounded by `document_bytes`, 1,048,576, not
16 lines (5.15). jig's thread charges the bytes and `LINE_ROOM`, 64 bytes,
for each line, message, block, object and part. Its `lines` stays for jig's
reference UI and tools; temper sets it to `document_bytes / LINE_ROOM`,
16,384, which the bytes reach first. The view prices its window from those
bytes, including Markdown and ten structural nodes per room. Earlier and
later reads let whole lines go at the opposite end, and read them again
when that edge nears the view; nothing says how many left. No second page
is read merely because the window has room: the browser measures the log,
and the terminal opens with one page. An empty page with an older cursor
reads on in either shell. The engine's pages keep their own rows (its
`documents.rows`, at most the client's). Committed turns on both the chat
and agent task pages are priced by `turn_text`; cards and other fields
retain `text`.

### 5.5 LLM accounts' words

temper's phrasebook words what jig's accounts document holds (jig's
`client/domain/contract.md`, 7.1; jig's `domain/accounts.md`, 3.5 and
section 5); the providers' names and their plans' are the
configuration's. The owned example and the default document offer
OpenAI's ChatGPT sign-in and API keys (`../protocol/setup.md`, 8.1).
The words for each offered source are:

- **The providers' words.** A provider is named by its configured name,
  but for the names the owned example ships, which temper's phrasebook
  words as their makers do: `openai` reads "OpenAI", its signed-in plan
  "ChatGPT"; `anthropic` reads "Anthropic", its program's plan "Claude".
  Any other provider's plan reads as its name. jig configures no plan's
  display name yet (jig's `protocol/configuration.md`, section 3), so
  the plan is branded by its provider until it does.
- **The welcome's model sentence** names the plans v0.1.0 supports,
  "ChatGPT or Claude", in fixed words rather than the offered ways
  (3.5, The welcome; the user, 2026-10-26).
- **The ways,** on `ConnectModel`'s Choose (3.5) and in the banner, one for
  each source a configured provider offers, in provider number order,
  but the playground's provider (`PlaygroundAnswer.provider`, 4.8),
  whose account is the deployment's: a sign-in to a plan, "Continue with PLAN" ("Continue
  with ChatGPT"), and to no plan, "Sign in to PROVIDER"; a program's
  login, "Use your PLAN subscription" ("Use your Claude subscription");
  a key, "Use an API key", or "Use a PROVIDER API key" ("Use an OpenAI
  API key") when more than one shown provider offers keys. Each way
  sends what the account form's act for it would: a sign-in is jig's
  `SignIn` for that provider, held by the person (`Own`); a key's,
  `AddKey` joining no account; a program's, its declaration. On
  the choices, the welcome and Settings use the same plan cards, with
  ChatGPT first, then Claude where offered. “✓ Connected” replaces a
  connected way’s note; “Add another plan” replaces its button, and the
  first unconnected way becomes primary. API keys are rows under “Or an
  API key”, each with “Add key”; the scripted model is never offered.

- **Details:** a key's heading is "An OpenAI API key", then "From
  OpenAI’s platform, under API keys. What temper’s agents use is billed
  to the key’s organization.", the secret "Key (pasted once)" field and
  the primary “Add key”. A program’s panel is “Your Claude subscription”,
  with “temper sets Claude Code up on this machine in an environment of its
  own: its settings, sessions and sign-in stay apart from your own Claude
  Code. In a terminal here:”, the command with Copy, then “It makes that
  environment, then Claude Code asks you to sign in to your subscription
  there. temper never reads its login.” (`../protocol/setup.md`, 6.6).
  “Use another login name” folds the Name field, defaulting to `claude-`
  and the person’s first name. “I’ve signed in” declares and checks the
  login; the command stays while the check is out. Each panel keeps
  “Other ways” beside its main act.
- **The banner:** "Agents need an LLM account to run." and "Continue
  with ChatGPT", "Use your Claude subscription", "Use an API key", one
  for each way the configured providers offer (The ways, above), each
  a link to `ConnectModel` at its way. While no account is usable and no
  playground is read (4.8), the composer says "No model yet · Connect
  a model", offers no picker and keeps its draft editable; its button and
  Enter cannot send. Unknown usability keeps the existing behaviour.
- **Completion:** Settings returns to Accounts, with “Added your OpenAI
  API key”, “Added your ChatGPT plan” or, once the host check succeeds,
  “Added your Claude subscription”, and its follow-up words, plus the quiet “Start a chat”. The
  line clears on leaving or acting. Manage usage stays in the account’s
  row. The program’s command, Copy and host checks stay in its way’s
  panel and account row, not a separate Ready card. The welcome keeps
  its Model completion and its own Ready step (3.5).
- **An account** is its provider and who it is, "ChatGPT ·
  alice@example.com"; a key-only account, "OpenAI API key"; a program's,
  "Claude subscription · claude-alice". Who it is is the email its LLM
  provider gave (jig's `domain/accounts.md`, 3.7) where it gave one, and
  otherwise its display name, or for a program's the source's name. Its card's heading follows with
  its provider and holder: "ChatGPT · alice@example.com · OpenAI · held
  by Alice", the holder the person's name from `me` for their own, and
  "held by the deployment" for the deployment's; a key-only account's
  name already holds its provider, so its heading doesn't repeat it:
  "OpenAI API key · held by Alice". The card's region is
  named by that heading, so a person and a screen reader find an
  account by its name. Each source is its kind in words, "ChatGPT
  sign-in", "API key", "Claude Code login NAME", then its state.
- **An account's line,** its folded row's summary: its models, what it
  pays for, then "2 tasks waiting", "used by T4, T9" or "no task uses
  it now", a waiting task never also a user. What it pays for, its
  "Pays for": a plan's, "Your own work, and projects where you’re the
  only member"; a key's or a program's login's with no uses given,
  "Your own work" (jig's `domain/accounts.md`, 4.2).
- **A run's wait,** above any chat's composer, a configuring chat's too, and
  on an agent's task's page, says in the person's words why its run can't
  start, from the task's `run_wait`, and offers the act where there is one:
  - *No account for it:* a task given a model, "No account can run MODEL
    yet.", MODEL that model; a task given none, whose `llm` is then none
    (jig's `client/domain/contract.md`, 5.1), "No account can run this chat
    yet.", naming no model it wasn't given; each with "Connect a model", to
    `ConnectModel` with no way chosen.
  - *An account's login on no host:* "ACCOUNT's login isn't signed in on a
    host that can run this.", the command (below) where the accounts
    document names the source, and a link to `Accounts`.
  - *A source held:* its state's words and its act, as the accounts page has
    them (`Limited`'s Resume, `NeedsSignIn`'s sign-in, `Refused`'s key).
  - *A credential:* "This work needs a credential from its forge." and a link
    to `Forges`.
  - *A host:* on a box, where the deployment's read says `owned` (4.6),
    "Waiting for other work to finish. It starts by itself."; a box's
    worker runs with its engine (`../domain/worker.md`, section 6), and
    a person who has only run `temper box` doesn't know "host". Otherwise,
    and until that read answers, "No host has room for this, or none is
    connected. It starts when one does." Either way, with nothing to press.
  - *No worker can ever take it:* jig's `Refused(Bounds { kind, why,
    figures })` (jig's `client/domain/contract.md`, 5.1), every connected
    worker too small for a bound of its charter, says "No worker can take
    this: ASKED, and the largest worker takes OFFERED.", box or not, one
    form for every bound (the request `ui-wait-words`, 2026-11-07), with
    nothing to press:

    | Bound | ASKED | OFFERED |
    |---|---|---|
    | turns | its charter asks 100,000 turns | 800 |
    | time | its charter asks 2 h a run | 1 h |
    | waiting | its charter asks 30 min of waiting | 10 min |
    | checks | its charter keeps 5 min for checks | 10 min, said "and the quickest worker's checks take 10 min" |
    | window | its charter's model at position 1 asks a window of 200,000 tokens | 128,000 |
    | output | its charter's model at position 1 asks 64,000 output tokens | 32,000 |
    | spend | a run here may spend 5,000 | 2,000 |
    | message | a message here may be 65,536 bytes | 16,384 |
    | inbox | a start here sends 12 messages | 8 |
    | unacknowledged | a run here keeps 64 turns unacknowledged | 32 |

    Counts and spend are grouped digits, spend in the units a chat's spend
    is said in, and durations are said in minutes and hours as the
    page says a span; the figures come in the bound's unit, durations in
    milliseconds. Where each worker falls short of another bound, jig
    names none: "No worker can take this: each falls short of another
    bound its charter asks." On the engine's own host, a kind temper never
    runs on (`../domain/root.md`), "The engine's own host can't take this:
    ASKED, and it takes OFFERED." Without figures, "No worker can take
    this." then the cause as a refused hello gives it (3.5).

    A refused hello whose cause is an agent's shortfall with figures, no
    worker admitted at all, says the same of the refused worker: "No worker
    can take this: charter 3 asks 100,000 turns, and the worker refused at
    10:42 takes 800." (5.12's clock), its charter named, as the refusal may
    be another charter's. Every other refused hello keeps 3.5's "Next run
    refused · WHY". The status above the composer is "Cannot start yet"
    for both.
  - *A command it names exists:* a wait's words in the page and the
    terminal name no ctl command; one that names a help page names
    `temper help NAME` with NAME a page (`../protocol/documentation.md`), as a
    shortfall of the checks timeout does: "See temper help settings for
    held revisions."

  A wait about to end (jig's `client/domain/client.md`, 19.1) stays
  "Starting". A Chats row holds no wait, and says "Starting".
- **The composer's line** names the account and model the work's next
  run takes, just above its field on an agent's task's page, with the picker
  (3.5, *The work's account and model*), from what
  jig gives every page: `me`, the accounts document
  and its readiness. When the welcome's model usability is known false
  and no playground is read (4.8), every composer says “No model yet ·
  Connect a model”, the link to `ConnectModel` with no way chosen.
  It sits above the composer, there is no picker, the send arrow is
  disabled, and Enter sends nothing; the draft stays editable. Unknown usability keeps
  today's behavior. Otherwise:
  - *MODEL* is the draft's model (a task's LLM choice; a new chat's,
    its template's), then " · EFFORT", the effort the choice names or, where
    it names none, that model's default in the offer.
    Where neither the task nor the template names a model, it is the supplied
    `resolved_default` of the page's template in `me` (jig's
    `client/domain/contract.md`, 5.1), a configuring chat's its charter's
    template, any other's template 1, what a run's check would take now for
    a task keeping no account, in charter order, under the effective
    authority and the current receiving party as payer.
    The composer never infers a default from the project catalogue's
    order, including a clipped catalogue. Only with no preview, where temper
    can't know, it uses "Using the default model", and the terminal takes the
    same words. A closed chat says what it ran on instead (3.5, *The work's
    account and model*). The preview is an observation, not a
    reservation: the draft keeps configured intent, and each run's check
    takes the models of a task given none (jig's `domain/accounts.md`, 4.3).
  - *The account* is named only when the payer is you: your new chat,
    or a task funded by your pool. The accounts document lists only
    your own accounts, in `(place, number)` order. In this slice the
    view translates jig's account selection and coverage (jig's
    `selection.rs` and `coverage.rs`): an eligible account preference,
    then an eligible kept account, wins, even while held; else the
    first account of MODEL's provider whose coverage serves you and
    whose deciding source is usable. Explicit uses replace defaults;
    absent uses serve the holder. A plan serves its holder. Program
    endpoints skip direct sources, and direct endpoints skip program
    sources; a held deciding source stops that account, and a waiting
    direct source does too. A "later" source names no account where
    jig's Later and Limited would choose differently: a usable one, and
    an unusable program source before one that would be chosen. A
    program source without usable evidence may pass to the next program
    source. Mixed provider offers do not establish a model's endpoint
    kind, so name no account. Another payer's work, or a cut document or
    account, names no account.
    The source words the line, with the provider and plan words above from
    `me`: "Using your OpenAI API key · MODEL" for a key; "Using ChatGPT
    plan · MODEL · Manage usage" for a signed-in plan, the last an
    external link where the document gives a safe manage-usage address;
    "Using your Claude subscription, through Claude Code · MODEL" for
    a program's login.
  - *The readiness cross-check:* the payer's row must read `Usable`
    before any account is named. The document's source `usable` is a
    fact, while jig's selection reads its grant at the moment it
    chooses: these may differ across expiry. When the translation
    picks an account but readiness does not read `Usable`, the line
    falls back to "Using MODEL", as does an eligible preferred or kept
    account whose deciding source is held or waiting.
  - *The playground:* its line is keyed by MODEL's provider, the
    delivered `PlaygroundAnswer.provider`, because its account is the
    deployment's and is outside the viewer's accounts document. Unless
    an eligible own preferred or kept account wins, use 5.6's "Using
    Scripted model · Default". An unknown or missing row keeps the
    playground's line, including on another member's chat.
  - *Unsettled:* with no choice and no preview, "Using the default model".
    Otherwise, when no account is named, "Using MODEL". A model a task or
    its draft holds that the offer leaves out reads "Using MODEL" too: the
    offer gives no provider for it, what a task holds stays its own, and
    its wait says why.
- **Each state,** with its acts:

  | State | Words | Acts |
  |---|---|---|
  | usable | "Ready" | — |
  | limited, a plan's | "temper reached its usage limit on your ChatGPT plan. Tasks on it wait." | "Manage usage", "Resume", "Use the API key first" |
  | limited, with the reset the provider gave | "Limited by OpenAI until 14:00. Tasks on it wait." | "Resume", "Use the API key first" |
  | refused: plan usage declined | "You signed in without plan usage." | "Continue with ChatGPT", "Use an API key" |
  | refused: not eligible | "This ChatGPT account can't be used for plan usage in temper. Plan usage needs an eligible ChatGPT plan." | "Use an API key", "Sign in to another account" |
  | needs sign-in | "Signed out of ChatGPT." | "Continue with ChatGPT" |
  | needs sign-in, a program's, on a host | "Claude Code isn't signed in for 'claude-alice' on HOST." and the command to run there | "Check again" |
  | misconfigured | "OpenAI refused temper's sign-in. Check the configuration's registration." | — |
  | unloaded | "Can't be used on HOST: WHY." | — |

  Never a reset the provider did not give, and never "your plan is
  empty" (jig's `domain/providers.md`, 2.5). A limit may be the
  person's own cap for temper, which "Manage usage" reaches. A limited
  account's tasks wait for it, and temper never moves them to another
  account (jig's `domain/accounts.md`, 4.3). "Use the API key first" is
  offered where the account holds a key after the limited source: it
  moves the key up, so that the account's waiting tasks go on on it,
  and moves no task. A plan's row shows its limit's state and "Manage
  usage" alone: its usage is ChatGPT's settings' to show (jig's
  `domain/providers.md`, 2.6), and temper reads no plan's windows.
- **A sign-in:** no words while it waits in a browser, which is at the
  provider (3.5, `ConnectModel`); in the terminal, "Open this address in
  a browser:" and the address, then the paste (below), as an external
  link (6.5). Then "Signed in to ChatGPT as alice@example.com", or
  "Signed in to ChatGPT" while the account has neither an email nor a
  display (its card's heading then drops that part), "temper’s agents
  now run on your ChatGPT plan.", its account, the model a new chat's
  first run takes and what it pays for; or,
  for a subject the person already has, "That's alice@example.com
  again. To add another account, sign out of ChatGPT in your browser
  first, then Continue with ChatGPT."; for a subject another person
  holds (jig's `domain/accounts.md`, 3.3), "Another person here holds
  that ChatGPT account. To use one of yours, sign out of ChatGPT in
  your browser first, then Continue with ChatGPT."; for a listener's
  port another program holds, before the browser leaves, "temper can’t
  listen on 127.0.0.1:1455: another program has it, such as Codex’s own
  sign-in. Finish or close that sign-in, then Continue with ChatGPT.";
  or why it was refused, in Each state's words. Each ending offers
  "Continue with ChatGPT" again, and a refusal Each state's acts.
  Each reads jig's ending on the `accounts` document (jig's
  `client/domain/contract.md`, 7.1): signed in, `Made`; the same
  account again, `Renewed`, named by who it is (*An account*, above) where
  that says anything, and "your ChatGPT account" otherwise ("That's your ChatGPT account
  again. To add another account, …"); signed in without
  plan usage, `Made` or `Renewed` with `declined`, in "refused: plan
  usage declined"'s words and acts, the account made all the same;
  another holder's subject, `Refused` `Held`; misconfigured, unloaded
  and signed out, `Failed` `Misconfigured`, `Unloaded` and
  `SignInAgain`; any other refusal or failure in jig's words.
- **The paste's refusals,** above its field, the sign-in still
  waiting (3.5, `ConnectModel`, The paste): *address*, "That isn't
  this sign-in's address. Copy the whole address of the page that
  couldn't be reached, from the address bar."; *too many*, "That's
  too many addresses from other sign-ins. Continue with ChatGPT to
  start again." and its button; *state*,
  "That address is from an earlier sign-in. Paste the one from this
  sign-in's tab, or start again."; a value past the bytes `me` states,
  before sending, "That's longer than a sign-in's address. Copy the
  address bar alone."; *not waiting* with the sign-in gone, "This
  sign-in has ended." and "Continue with ChatGPT"; *lost* or *unsent*,
  the field empty, "That address wasn't sent. Paste it again."; a
  sign-in that refused too many ends as a misconfigured one does, in
  its words. Four have
  no fact of jig's yet, and are not drawn until jig gives one: the
  listener's port another program holds, "not eligible", plan usage
  declined at the provider rather than at its consent, and a missing
  scope in words of its own rather than its number (section 11).
- **A plan's first use:** a card in the conversation of the run that
  first used it, from the document's fact (jig's `domain/accounts.md`,
  section 5): "You're using your ChatGPT plan. Eligible requests from
  temper use your plan's included usage. You can see it, and set a
  weekly cap for temper, in ChatGPT's settings." and "Manage usage". It
  is part of the conversation, so nothing is kept to dismiss it
  (section 11).
- **Signing out:** "Signed out of ChatGPT." once the revocation is
  confirmed; "temper removed its tokens, but ChatGPT didn't confirm.
  Disconnect temper in ChatGPT's settings, under Apps." otherwise; for a
  program's login, "Signed out on every machine.", or the hosts that did
  not.
- **Plain words:** temper says that it runs Claude Code, and names no
  page, button or feature after it (jig's `domain/providers.md`, 3.2).

### 5.6 The first run's words: the playground and the welcome

temper's phrasebook words the playground (4.8; the mockups' first run)
by the person's name for it (with the user, 2026-10-23;
`../draft/archive/first-run.md`, 2.3); OWNER/NAME is the playground's
repository, and the cues the playground's, copied in from its read:

- **The banner,** one line, in the frame, above the main of `Chats`
  and `NewChat` while the playground runs, and on every page while it is
  leaving (5.1): "This is the
  playground: a scripted model and in-memory forge. Use the wizard to
  set up temper", "set up temper" its link, which sends `EndPlayground`
  (4.8); while it is leaving, "Leaving the playground…" in its place,
  with no link (5.1). Beside it, the rail's foot offers "Set up temper",
  or the frame's banner leads its corner with it on a page that is no
  project's, the same act, while the playground runs and is not
  leaving: the way out, prominent (5.1, 5.3).
- **The playground's start,** on `NewChat`, at the head of its thread
  (3.5; the first run's first step): the status "Ready"; while its start is
  out or its read answers starting (4.8), the status "Starting" and its
  suggestions disabled; once the start failed, the status "Not
  started"; the context line; once the start failed, under it "The
  playground did not start: WHY." with "Try again", WHY "its fakes did
  not start (BOX'S WHY)", "the scripted model's account was refused",
  "OWNER/NAME was not adopted" or "Playground forge did not connect";
  then the page's `.meta-line` (3.5), and under it a list, `.starters`, named "Suggested requests"
  of one button per suggested request, the playground's `llm.asked`
  cues in order and never its `llm.cued` (`../protocol/setup.md`,
  8.4), each named by its cue's words, kept for the tests that walk
  the playground by them while the look's invitation is gone (the
  user, 11-03; section 11). The model's name is the picker's closed line's
  alone. A chat starts with its first
  message, so the start has no task and no number; once a suggestion
  or the composer starts the chat, its page opens in place of `NewChat`
  and keeps the frame (3.5).
- **The context line,** at the thread's head (5.15), in `.meta-line`'s look: on a chat's page,
  "PROJECT · OWNER/NAME on Playground forge", PROJECT the project as
  the switcher names it; then "Signed in
  as the deployment’s owner" while `me`'s role on the deployment is
  `Owner`, else "Signed in as the project’s owner" while the person's
  role on the project is `Owner`, else "Signed in as NAME", the
  person's name as the rail shows it. On `NewChat`,
  OWNER/NAME alone, since the switcher names the project (5.1). OWNER/NAME is a node of its
  own, for the look.
- **The composer's line,** for the playground's LLM provider: "Using
  Scripted model · Default" (5.5). The scripted model's provider serves
  one model, so the line names no other. In the playground's project,
  the one that allows it, a new chat's line is the scripted model's; a
  chat the scripted model ran keeps its line after it, since it keeps
  its account (jig's `domain/accounts.md`, 4.3; `../protocol/setup.md`, 8.3).
- **Its names:** the phrasebook names the playground's provider and
  model "Scripted model" and its forge "Playground forge", from the
  read's numbers, whatever names the records hold
  (`../protocol/setup.md`, 8.4). Its project is an ordinary project,
  named by the name its record holds wherever a project's name shows,
  in the switcher, the context line, the chats, a task's page and the
  inbox: "Demo" as its start makes it, its new name once renamed. No
  fixed word names it.
- **The welcome** (`welcome.html`; `../draft/archive/first-run.md`, 2.3): its
  steps' words, their order and their acts are its mockup's
  (`mockups/welcome.html`, at the states its worlds list), the set up
  card's model sentence among them (5.5). Beyond them: a move is a link
  (`browser.md`, 2.3); the foot's "‹ Back" goes to its step's Choose
  inside a choice, otherwise to the step before, and its "Try the
  playground instead" shows while the playground is offered (3.5, The
  welcome). A forge cut short reads "FORGE · not finished", with "Go
  on" on GitHub and "Start over" (3.5, The welcome). *Where it departs
  from its mockup, for now* (`../draft/first-run-projects.md`, 4; the
  mockup is not edited): the Forge step's Details have no "Into a new
  project, named" and the step no "Choose a repository to start in";
  the forge's line names no project, "FORGE · ORG/NAME · CONNECTION"
  and "✓ FORGE · connected as LOGIN · ORG/NAME made, in no project
  yet"; Ready's pane has no "Project PROJECT,
  with ORG/NAME" line, and its act opens the project without waiting
  for the adoption, in place of "ORG/NAME is ready", "It opens once
  ORG/NAME is made." and "Adopt a repository"; and the chats page of a
  project with no repository says "No repository yet", with "Adopt a
  repository" (3.5).
  *Where it departs from its mockup, since 2026-11-04* (the user's
  greenfield pass; the mockup is not edited): the frame holds temper
  box's line and the steps, and no person's name or "Sign out"; the
  Welcome step's "Set up temper" card comes first and is the default,
  its words "Your forge: a Forgejo temper runs for you, or GitHub. Your
  model: ChatGPT or Claude.", the playground's card after it, "A
  scripted model and a forge, built in. A taste of how temper feels,
  with nothing to connect.", and under them "Tried the playground
  first? Set up whenever you like; the playground then steps aside.";
  Forge's cards are Forgejo's, "A forge of your own: temper starts it
  on this machine and manages it. You can import existing repositories
  into it, from GitHub or elsewhere.", with "Run on this machine"
  alone, which opens Forgejo's sheet, then GitHub's; Model's lead is "temper’s agents run on a plan you
  sign in to.", its cards the plans alone, each with its muted line;
  the forge's ticked line reads "✓ FORGE · connected as LOGIN ·
  ORG/NAME made"; the foot has no "Later"; and
  Ready is the user's greenfield mock's: "Ready" once its forge and
  model are, "Almost ready" until then, each with its lead, then the
  first project's form, the pane's lines a strong name then what they
  say, the model's naming the account and no model (the request
  `model-check-pane`), its form's
  name the first repository's in capital, or "My project", until the
  person writes another, "Create your project" and "Rename it whenever
  you like.", the lead naming the repository the project starts with,
  as the mock's Ready does (3.5, The welcome).
  *Its look* is the welcome's own, scoped to its frame (`browser.md`,
  2.5, The welcome's look).
- **Not drawn:** the mockup's placeholder, "Try a suggested request…"
  (section 11).

### 5.7 A delegation's card

A chat's delegation shows as the card of the task it made, at the call
that made it, or else at the result it sent back: a delegation as the
chips of the tasks it made, and their results (jig's
`client/domain/objects.md`, section 10), in one card per task, as
the mockups' first run shows it. For a change, it is the first form of
the change's chip in a chat (`forge.md`, 2.4 and 2.5).

- **Which tasks.** A `delegate` call that did not fail names the tasks
  it made: by the call's `objects`, each `Task(N)`, when jig's thread
  keys them there, and otherwise by its acknowledgement, temper's own
  words ("Delegated tasks: 2, 3", `temper-engine-smith`'s answers),
  which the domain reads N from (section 11). The chat's task document
  is not enough: it lists the chat's delegates while they live only.
- **Where it shows.** Once, at the first of its anchors the
  conversation's window holds: its call, or else a result block that
  carries its result, whose summary has a line "Result from task N:"
  after the inbox entry's "M: " (temper's inbox words,
  `temper-engine-domain`'s `inbox_words`); or else the chat's heard
  result from the task (its activity's `Heard`, a result from task N),
  which comes before the chat's resumed turn carries a result block and
  is never drawn as words of its own; or else, while the task
  lives, after the window's last line, as one of the chat's live
  delegates (its document's `delegates`). The chat's window holds a
  small conversation whole, its turns with their calls'
  acknowledgements (5.4), so the first run's change shows at the call
  that made it; in a longer conversation the call may leave the
  window, and the card shows at its result, or after the window while
  the task lives. Those words name a person's number as well as a task's: a
  result shows a card only once its task's document says this chat
  requested it (its `requester`), and keeps its work summary
  otherwise; such a number still takes one of the watches while the
  thread names it.
- **What it reads.** The chat page watches each such task's document
  (`Subject::Task`), as the task's own page does: its chip, its result
  contract, its hold and its result, which stay readable once it has
  ended. When the chat hears a delegate's result, it reads that task's
  document again once, so a cached running card takes its durable result
  without waiting for the chat's resumed turn or the watch's end.
  It watches those its thread's calls, results and heard results name, the
  newest first, then its live delegates, up to the domain's `reads`;
  one past them, or that neither names any longer, is unwatched, and
  leaving the page closes them all. A delegation whose document has
  not arrived, or past the bound, shows as its anchor's work summary,
  as any call does.
- **Its parts,** in the mockup's order, a region named "Task N":
  - a line with its number and kind, "TN · CHANGE", and its state; the
    kind is its result contract's: CHANGE, REPORT, VERDICT, APPROVAL;
  - its title, the chip's (the spec's first line), as a heading;
  - its result's words, once it has ended;
  - for a change, its steps, a list named "Steps": "Branch pushed",
    "Pull request opened", "CI passed", "Landed";
  - a link to its page, named "Task N" and reading "Task N ↗" (3.5,
    "Links are by number").
- **Its state,** in temper's words, from its document: "Running" while
  it lives, "Held" while held (why, on its page and in the inbox);
  once ended, "Landed" for a change done with a change's result,
  "Completed" for any other done (jig's word, as the task's page says
  it), "Failed", "Cancelled" (cancelled or withdrawn), and "Ended"
  otherwise.
- **Its result's words:** a landed change's, "CI passed. Landed ·
  merge commit ABCDEFG", ABCDEFG the first seven characters of the
  result's words' first line, which is the merge commit's id in hex (the
  engine's landing words, whose second line names the pull request,
  `../domain/root.md`, section 6); any other result's words as they are; a failed
  task's reason.
- **The steps are a stand-in:** none ticked while the change lives, all
  four once it has landed, none otherwise. A ticked step is skein-ui's
  `completed` item, its tick the look's (STEP_DONE): a screen reader
  reads "Branch pushed completed" and the terminal prints "- Branch
  pushed (completed)" (skein's `ui.md`, 3.2), so nothing is said by
  style alone (5.1). The change's live stages are later (section 11).
- **What it stands for:** the call's work summary, its result's, and
  the line that heard the task's result, whose words are the card's: a
  chat that shows a task's card shows none of them.
- **In the browser it is one line** (the greenfield mock's `.turn-tool`),
  the view and the terminal holding every part above:
  - *a delegated change:* the pen icon, "Change", its title in `--gf-ink-2`,
    its state in lower case, "held" in `--gf-signal` while held, its result's
    words once it has ended ("CI passed. Landed · merge commit 1a2b3c4"),
    then a link to its page, underlined as `.turn-tool button`. Its line of
    number and kind and its steps are hidden in the browser. The link's name
    stays "Task N"; its text is "Open" in the browser, "Task N ↗" in the
    terminal;
  - *a goal,* a delegate that is tracked, from its `goal` call or its
    document: the same line with "Started goal", its title in `.file` mono,
    its state and "Open".
- **The look:** the card's tokens (CARD and TASK_CARD, META, TITLE,
  DETAILS, a status's) and three of the steps: STEPS, STEP and
  STEP_DONE (`browser.md`, 2.5); on a chat's page these draw the line
  above, and elsewhere the mockup's `.steps`, `.step` and `.step.done`.
- **Where it is decided:** the domain's chat page holds each delegation
  (its task and its handle), and a pure function of the domain derives
  a card's facts from a task's document: its kind, state, title, words
  and steps ticked. The view lays them out with the phrasebook's words.
- **The first run:** the chat is task 1 and the change it delegates
  task 2, so the person reads "T2 · CHANGE", Running, then Landed
  (`../draft/archive/first-run.md`, 2.1), and after the card the chat's next
  call, folded, "1 tool call", described "wait" (3.5).

### 5.8 A goal's page

A goal is a tracked agent's task (3.5). Its page is the greenfield mock's
`goal.html` (`greenfield/README.md`), drawn by the view's `pages/goal.rs`,
where `pages/task.rs`'s `main` hands over for `Shape::Tracked`. An agent's
task's page (5.9) and a change's or a person's (5.10) are other pages, and
they are where a goal's task rows lead: the mock has no task page.
`temper ctl task show` keeps the facts this page doesn't say (`tools.md`,
6.1): what was asked whole, its dependencies, its activity, its budget and
authority.

The page is `.col.wide`, in the order below. Its words are exact. Its state,
`S`, is `goals::state` (3.5, *Goals*), of its chip, whether the reader has
an open need on it, whether it has a report, and its direct delegates' total.

- **What it reads,** beside the task's document, its tree, its latest
  activity page and its ended delegates' page, as a procedure's page reads
  them (5.10):
  - its report (3.5, *A goal's report*), read again when the document's
    `last_activity` moves;
  - its own task line, for its counts: `Read::Tasks` by `TaskQuery { project,
    requester, label, tracked, by number }`, all four from its document,
    following the live group's and then the ended group's cursor until a row
    is the goal, at most 8 pages, and read again when the document's
    `changed` or `last_activity` moves. Until it is read, and when no page
    held it, the counts are what the page holds: the delegates' total its
    chip's `live_delegates` and the ended page's rows, cancelled ones left
    out; done, the ended rows that ended done;
  - its needs: the inbox's `Inbox`, `Filter { project, goal: Some(task) }`,
    opened once the document says the task is tracked and running inside the
    goal's page, its cards the page's as Needs you's are (3.6).
- **The header,** `header.goal-head`:
  - `p.meta.mono`, items separated by `<span>·</span>`: `<b>TITLE</b>`; `S`'s
    word in its `state-*` class, as a goal's row says it (3.5); "6 of 9 tasks"
    when there are tasks; "31 of 40 spent", the budget's numbers as supplied,
    no currency; "since " and `made` in the clock form (5.12).
  - `<h1>`: the report's headline. With none: planning, or live with no tasks yet, "Planning from NAME:
    reading it and the designs it cites.", NAME the draft's file from the
    goal's words (below), or "Planning: reading what was asked." without one;
    closed, "Closed by you. What landed on the goal branch stays there.";
    landed, the result's first line, or "Landed."; failed, "Failed: " and the
    failure's reason; otherwise "N of M tasks done.", as a Goals row's (3.5).
  - `p.byline`: "Written by its agent at " and the note's `changed` in the
    clock form. No byline with no report.
  - `.acts`: "Draft" (`.btn.small`), when the goal's words name a draft and a
    chat: to that `Chat`. "Chat T2", when the requester is a task: to that
    chat. Neither otherwise. While live: "Amend", "Pause" or "Resume", "Close",
    5.11's. Ended, none of the three.
- **Needs you bands,** one `section.band` named "Needs you" per open need of
  the goal, in the inbox's order, between the header and the report:
  `<p><span class=tag>Needs you · KIND</span>QUESTION</p>`, KIND and QUESTION
  as Needs you's card reads them (3.5, *Needs you*): "a question", "a
  proposal", "held", "a person task", "funds". Then:
  - the page's first question whole, as Needs you's card draws it (3.5, *Whole*,
    *Cut*): its context, "What the agent saw" and its cut line, in `.ctx`. A later
    question on the page draws its QUESTION from the document's bounded words and
    "Answer", so that one page holds one whole question, as 5.4 prices it;
  - the parsed choices of a question or a person task in words, in
    `.choices`, answering as Needs you does, with no Reply and no Later;
  - no choices: `.btn.primary` "Answer" for a question or a person task,
    "Open" otherwise, to `Inbox` filtered by this goal;
  - a funds card, with no "Add $10" and no "Stop the goal": its Release and
    Leave held as Needs you draws them (3.6), or "Open" when neither is
    offered;
  - an approval, a change under the goal waiting on a review on the forge
    (`forge.md`, 4.1), KIND "an approval": its review link as `.btn.primary`
    where the card offers one, else "Open", to the change's `Task { task }`.
- **`.two`,** the report and the side, wrapping by `.two`'s own flex basis, not
  by a media rule.
- **`article.report`,** heading "Report" (`.section-title`):
  - each paragraph as `<p>`;
  - planning: `.typing`, three dots, a `Figure` named "Planning", under the
    words "Reading NAME and the designs it cites, to propose its tasks.", or
    "Reading what was asked, to propose its tasks." with no draft; no Left or
    Risk;
  - `<p><strong>Left:</strong> …</p>`, `<p><strong>Risk:</strong> …</p>`, when
    the report has them;
  - a report-less ended goal: the result's words, or a failure's reason, as
    paragraphs;
  - `p.byline`: "By its agent at TIME". The mock's "earlier reports" is not
    drawn: one note is revised in place.
- **`aside.side`,** its parts in order:
  - **Tasks** (`.section-title`), the goal's direct delegates alone: its tree's
    depth-1 rows and the ended page, merged by number, a delegate in both
    drawn once as the tree's; a delegate's own delegates aren't drawn; cancelled
    ones are left out.
    - `figure.bar` (`Role::Figure` in the browser, `Role::Image` in the terminal),
      named "N landed of M", a `<span>` per task in
      order, class `landed` for a landing, `running` for a running one,
      none otherwise. A *landing* is a delegate ended done with a change's
      result that landed (`ResultValue::Change { kind: 1, .. }`, as a delegation's card
      reads it, 5.7); a person task's answer, an advisor's report
      or another result is done, and not landed.
    - Each task not landed, `p.task-line`, with class `running` when
      running: the title, a link to its `Task`, then its state: "done" and its end
      in the clock form; "running";
      "waits on TITLE" for one dependency, or "waits on task N" where the title
      is not among these rows; "waits on N tasks" for several; "waits on you"
      when an open need of the goal's is that task's; "waits on its resources";
      "held", in `state-needs`; "failed"; otherwise "waits".
    - `Role::Disclosure` "N landed" (`.link`, 13px, `margin-top: 6px`), folded,
      holding each landed task's `p.task-line`: its link, then its end in the
      clock form.
    - No tasks: `p.facts` "Proposing tasks from the draft…" planning, or "No
      tasks.".
    - While the ended page has more after it or was cut, "More of its tasks are
      not shown." as `.facts`.
  - **Landing,** `p.facts`: landed, "Landed at 01:12." in the clock form;
    closed, "Closed before it landed."; failed, "Failed before it landed.";
    live, holding a goal branch, "Lands whole once all nine are checked.", the
    delegates' total in words through nine, then a digit, or "Lands whole once
    all its tasks are checked." with no tasks yet; live with no branch, "Each
    task lands on its own once checked." A goal branch is a resource of the task
    whose path ends in the name `temper/<root>/g<root>`, `../domain/forge.md`,
    11.1's. The mock's "last updated from main" is not drawn: no read says it.
  - **How it runs,** `p.facts`: the way, "Plan, tasks, land whole" with a goal
    branch, "Task by task" without, then " · always asks you" when the
    goal’s words hold the exact line `Asks: always`, otherwise " · follows the
    away switch". One pure domain function, `goal::always_asks(words) -> bool`,
    reads that line; a phrase inside a sentence does not count.
  - **Decided by the goal,** only with a report's `Decided:` line: `p.facts`,
    its items as written, joined by " · ".
- **`.sent`,** when the person has sent words: each of the goal's
  `Activity::Words` by the person in the latest activity page, `p.you`, oldest
  first, then the pending one, "Being sent". No "Its agent:" line: a goal's
  answer is its next report.
- **The composer,** 5.11's, last, under a `max-width: 720px` box.
- **The goal's words,** which Start goal writes (5.14) and the page reads back
  by a line's start, in `goal::draft_of(words) -> (Option<path>, Option<chat>)`,
  pure and the domain's: `Draft: PATH` and `Chat: T4`.
- **Where it is decided:** the domain's `pages/goal.rs` holds what the page
  reads and decides, pure where it can be: its counts, its headline's case, a
  task's line, its landing's case, `draft_of`. The view words and lays them out.
- **The terminal:** the view is shared and the page keeps its meaning. The
  regions print in order: the header, the needs, "Report", "Tasks", "Landing",
  "How it runs", "Decided by the goal", the composer. The bar prints by its
  name, "6 landed of 9"; the disclosure's rows print under it.
- **Not drawn,** the mock's: "earlier reports"; the goal's budget as money;
  reordering a goal; and every part of the page as it was ("Task details", the
  pagebar, "What was asked", "Result", "Plan" and its filter, dependency lines,
  "Latest activity", "Budget and authority", the model choice).
- **The look:** `browser.md`, 2.5, *A goal's look*.
A plan on the other pages, an agent's task's (5.9) and a change's or a person's
(5.10), keeps its tree of rows, by these rules:

- **The header,** on a change's or a person's page: "Task details", then the
  task's number, its executor and its project; its state follows the title,
  then who asked.
- **A task's state,** on those pages, in the mockup's words, in lower case:
  - in the header, from what the engine computes the task waits on
    (`Task.waits_on`): on its dependencies, "waiting on task M" for
    one, "waiting on N tasks" for several, N their supplied `count`;
    on its delegates, "waiting on N delegates" ("1 delegate"), N its
    chip's `live_delegates`; on a party, "waiting on an answer from
    you" when the person is that party or holds that role, "waiting
    on an answer from person N" or "from role N" otherwise; on news,
    "waiting on news". Waiting on nothing, it says what a row would;
  - on a plan row, which has no `waits_on`, from its chip's owner phase
    (jig's `client/domain/objects.md`, section 3): waiting on its
    dependencies (the phase's `dependencies`, those not yet ended),
    "waiting on task M" on one, "waiting on N tasks" on several, N
    those listed, and "waiting on more than N tasks" when the list was
    cut (`more`); waiting on no dependency but on its holdings
    (`holds`), "waiting on its resources";
  - running, in both, with the work its owner observed
    (`observation`): "running · writing", or "running · a tool ·
    NAME", the tool's name as supplied; "running" alone when no
    observation is supplied, which jig allows, or for any other work;
  - ended, with its owner's ending: "Completed", "Failed" or
    "Cancelled", as 5.7 words them;
  - otherwise, jig's word for its phase in lower case: "held",
    "retrying", "waiting for news". A held task's card and a
    delegation's card keep theirs (3.6, 5.7).

  Each carries its phase's status token: a waiting state waiting
  (`STATUS_WAITING`, the mockup's amber), a cancelled one quiet, a
  running one active, a completed one done, a failed one held.
- **A plan's rows' order,** a tree of its delegates: the roots by number, the
  order they were made.
  An ended delegate is drawn before the first live root with a higher
  number, or after the last live root when none has one; a live root's
  own delegates stay under it, in the tree's order. A delegate in both
  the tree and the page, one that ended after its tree was delivered,
  is drawn once, as the tree's row. An ended delegate's own delegates
  are not read: its page shows them (section 11).
- **An ended row** is a plan row (below) whose state is its ending,
  "Completed", "Failed" or "Cancelled", with its spend and with no
  dependency line or held card; its "Task N" link is muted, as the
  mockup's done row.
- **A plan row's cells:** the "Task N" link and the task's title; its
  executor; its state; its spend, "N spent", what the task spent
  itself (its chip's `budget.spent`; not the row's `spent`, which adds
  what the ended tasks it funded spent), in the deployment's unit, a
  whole number. The cells have no heads, since tree items are not a
  grid's cells; the mockup heads its columns "Task", "Kind", "Model",
  "On", "State" and "Spent" (a default the user may change, section
  11). A procedure's part follows on a line of its own, as before. At
  phone width the name is on a first line, and the executor, state
  and spend on a second; no cell is hidden.
- **The dependency line:** under a row whose task has dependencies
  (its `dependencies`), a list named "Dependencies of task N", one item
  per dependency in the supplied order, keyed by its task as an object,
  holding its "Task M" link and its state: a live dependency's as
  above, an ended one's supplied status ("Completed", "Failed",
  "Cancelled"), and none when none was supplied. When the row's list
  was cut (its `cut`), a last item, with no key: "More dependencies
  not shown." The browser shows the list's name before its items. In a
  row's tree item come the row, its dependency line, its held task's
  card (3.6), then its children.
- **An added dependency's mark:** in its item of the dependency line,
  after its link and its state, read only, for a dependency an
  amendment added (the row's chip's `dependencies`, jig's
  `Dependency.added`, matched by number): a paragraph, "Added by you"
  when the person signed in added it, "Added by party P", "Added by
  task M" or "Added by the deployment" otherwise; then, while it keeps
  the row's task (jig's `keeps`, jig's `domain/tasks.md`, 5.7), " ·
  keeps task N from ending". No time is drawn: `ctl task show TASK`
  prints when (`tools.md`, 6.1). A dependency the facts
  don't name, or one not added, has no mark; the line keeps its shape
  (section 11).

### 5.9 An agent's task's page

An agent's task's page (3.5) is that of an agent's task with no
priority and not the chat label: a goal's delegate, the agent a
change's procedure makes, a task an agent's task delegated. It shows
its task document, its conversation as a chat's page holds one, and its
owner tree's held cards, as `mockups/task.html` draws its agent kind,
in the chat's look. It draws each amount and each state as the
documents supply them (jig's `client/domain/contract.md`, 5.1), and
acts on it as its document offers (5.11).

- **Its parts, in order:** the lineage bar; the taskbar, with its acts
  (5.11); "Task brief"; the run's heading, then its own held card while
  it is held (5.11); the earlier read's loading or failure status (5.15);
  the conversation, read by its log's edges with no "Older" button;
  its acts' outcome; "Result", once it has ended; "Plan",
  while its tree holds a delegate or a held task's card; the composer
  while Message is offered (5.11); then the confirmation while one is
  open (3.6). At phone width the order is
  the same: the lineage bar keeps one line, its titles cut short, and
  the taskbar's parts wrap.
- **What scrolls,** in a browser: the log alone, as on a chat's page, so
  that PgUp and PgDn from the message box, "Latest" and the kept place
  (5.15) act on it as there. From "Task brief" to "Plan", the parts above
  stand in their order inside the log, around its lines, and "Latest"
  beside it, after it, as a chat's. The log follows its end, so that the
  page opens there and the brief is read back to (section 11). The
  amendment's editor (5.11) stands above the log, where it is seen when
  it opens, since it takes no focus; it scrolls itself where a short
  window can't hold it. A terminal scrolls nothing of its own: it prints
  those parts around its log, in the same order.
- **The lineage bar:** its parent, the last of its lineage
  (`lineage_chips`, root first), as its "Task N" link and its title,
  after an arrow; then the task itself, "Task M" and its title. With no
  lineage supplied, the parent is its requester's "Task N" link when a
  task asked it, and there is none otherwise. The arrow and the slash
  between them are the look's.
- **The taskbar:** its level-1 heading "TN ·
  TITLE", as a chat's; its state, in 5.8's words, from the same
  function (`pages::task::header_state`), with its status token; then
  its executor, "Agent", and its own spend, "N spent", its `budget.spent`
  as a plan row's (5.8); on a closed task, then what it ran on (3.5, *The
  work's account and model*).
- **"Task brief":** a region, the mockup's "TASK BRIEF", whose capitals
  are the look's: its specification as written (`spec`, read as Markdown); who asked, in 5.8's words ("Asked by
  task N"); then its dependency line, as a plan row's (5.8): the list
  "Dependencies of task N", one item per dependency of its document
  (`dependencies`), in order, keyed by its task, with its "Task M"
  link and its state, and an added one's mark (5.8), from the
  document's `dependency_facts`. The document's `cut` does not say
  which of its lists was cut, so the line has no last item (section
  11). The mockup's worlds declare no dependency: the line is
  temper's, for rung B2 (`../domain/testing.md`, 2.7).
- **The run's heading,** one, for its current or its last run, from its
  chip's owner phase and its `attempt` (jig's `client/domain/objects.md`,
  section 3):
  - "Run N · live now" while a run is out: a claimed or a running
    run, or a run closing, N its attempt;
  - "Run N · waiting" while the task lives and no run is out, N its
    last run's attempt;
  - "Run N · ended" once the task has ended, N its last run's attempt;
  - "No run now" while it lives and has had no run, and no heading
    once it has ended without one.

  Under a live or a waiting run's heading, "Started MM-DD HH:MMZ" when
  its chip supplies `run_since`, when the run was placed, in universal
  time. A run's earlier runs are the
  conversation's "Run N began" and "Run N ended": the page has no run
  groups.
- **The conversation,** from the task's bounded thread (jig-client's
  `thread`), read as the chat's page reads it, is drawn as work, not as
  a chat's (3.5): its latest window; committed turns, the agent's
  "Temper" over its texts; its calls in runs, as a chat's (3.5), one
  line a run, running while `task::run` says a run is out and the words
  being written haven't begun, a `delegate` call's a row too, since a
  delegation's card is the chat's (5.7), and a run's results its rows'
  outcomes, a message that holds only those drawing no "Completed"; "Run N began" and
  "Run N ended"; and a live run's words being written, as the chat's.
  No three dots: its run's heading says it is live. "Latest" stands over
  the log's foot while its end is out of view, as a chat's (3.5).
- **Its result,** once it has ended: a region "Result" after the
  conversation, its state and its words as a goal's "Result" says them
  (5.8), from the same function (`pages::delegation::facts`):
  "Completed", "Landed" (done with a change's result, as 5.8 says),
  "Failed", "Cancelled" or "Ended", and the report's words, or a
  failed task's reason. The mockup's "RESULT" capitals are
  the look's. A task that has not ended has no such region.
- **Its plan,** while its owner tree holds a delegate, or a held task's
  card, its own or a delegate's: "Plan", by 5.8's rules for a plan, its rows,
  their dependency lines and held tasks' cards. The mockup draws none on an
  agent's page: its held cards are what this page may act on.
- **Against its mockup,** the page's way:
  - links and the lineage name a task "Task N", the mockup's "T12"
    (3.5, "Links are by number"); the executor is "Agent", where the
    mockup names the charter (3.5);
  - the playground's banner stays where the frame draws it, before the page,
    where the mockup draws it under the taskbar (5.1);
  - its calls in runs, as a chat's (3.5), where the mockup folds them
    by message ("7 tool calls", then "read ×6 · recall"), and its results
    its rows' outcomes;
  - not drawn, since no document supplies them: the charter's, model's,
    account's, worker's and pool's names ("Implementer · Sonnet ·
    medium", "on s1", "holds a slot of T12's pool"); whose
    specification it took ("from T3's spec"); a run's own spend ("N
    spent so far"); and a turn's time, which a chat's page alone says
    (5.12). Nor its context (`Task.context`), which
    is optional;
  - its acts are 5.11's, with their own departures there.
- **Where it is decided:** the domain's task page has the shape
  `Agent`: an agent's executor, no priority, not the chat label, which
  opens as `Chat { task }` (3.5). It holds the task's thread as the
  chat's page holds one: made with the chat's bounds, its latest page
  read as the page opens, read again on each change of the document,
  given the run's live text while the `run` watch is open, and its run
  ended as the chat's is when no run is out. A pure function of the
  domain derives the run's heading from the chip (`pages::task::run`);
  the state and the result are 5.8's. The view lays them out with the
  phrasebook's words.
- **The look:** its own, MAIN and AGENT_PAGE on its main, TASKBAR on its
  taskbar, THREAD on what follows it and the conversation's tokens, so
  `temper.css`'s rules for this page draw them; and AGENT_LINEAGE
  on the lineage bar (a node
  carries two tokens at most, and its main has MAIN and AGENT_PAGE),
  AGENT_BRIEF on the brief, AGENT_RUN on the run's heading and its
  start, AGENT_RESULT on its result, and AGENT_FOOT on the keyed slot
  holding the acts' outcome, result and plan after the lines. `task-pages.css`
  draws them as the mockup's
  `.agent-lineage`, `.agent-brief`, `.agent-run-heading`,
  `.agent-run-meta` and `.agent-result` (`browser.md`, 2.5).

### 5.10 A procedure's task's page

A procedure's task, a change among them (`../domain/forge.md`, section
8), shows its header, "Task details", and its plan (5.8's rules for a plan), as a
person task does, with its goal and "Its tasks". A forge procedure's task, whose
panel the forge's side watches while its page is open (`forge.md`,
9.2), draws its panel as `task.html`'s change does: its steps, its
hold, its gates and its bounds. Each fact is drawn as the panel
supplies it: the page computes none (jig's `client/domain/contract.md`,
5.1).

- **Its parts, in order:** the bar, a change's opening with the "Changes"
  breadcrumb, to its project's `Changes`; the header; "View on forge ↗"; the
  change's card, holding its steps, its step's line, its hold, "Its
  tasks", "Gates at its head" and its bounds. At phone width the order
  is the same, and the steps take three columns. Until its panel is
  shown, and for any other procedure, the page is the bar, the header
  and "Its tasks".
- **Its goal:** in the header's first line, after its project and its
  procedure's part, a slot of "Goal" and the "Task G" link to the goal
  it serves, the document's `goal` (the nearest tracked task at or
  above it, itself included: jig's `domain/core.md`, 6.2), when that is
  another task. A tracked procedure's task serves itself and one under
  a chat serves none; neither shows the slot.
- **The header, its panel shown:** its first line begins with the tag
  "Change · Task N" where 5.8's says "Task N" and its executor, then
  its project, then its goal's slot; the procedure's part is the panel
  itself, so the line has no "Shown on the forge.". Its title follows,
  its state and who asked, as 5.8's; then a line naming its pull
  request, "PR #N · OWNER/NAME · BRANCH → LANDING", or "No pull
  request yet · OWNER/NAME · BRANCH → LANDING" before one is opened,
  the repository, its branch and its landing branch as supplied.
- **The header, its panel not shown:** 5.8's, its procedure's part
  saying where the panel is (jig's `client/domain/root.md`, section 6):
  "Loading the change from the forge." until its first delivery; "The
  forge knows no change for this task." when the forge answers unknown
  (a task that is no change, or one whose subject left the engine);
  "The forge cannot show this change." when its watch ends for any
  other reason. A procedure the forge does not run says "Shown on the
  forge." as before (4.2).
- **"View on forge ↗":** an external link (6.5) to its pull request's
  address on the forge, in a toolbar after the header, only when the
  panel supplies an address (`forge.md`, 9.2); a forge with no address
  gives none, as the playground's fake forge does.
- **Its steps:** a list of six, as the mockup's: "Producing",
  "Opening", "Checking", "Gating", "Queued", "Landed". The one its step
  is at is current, those before it done, and all six done once it has
  landed. The steps 2.1 draws beside the way show at the one they
  serve: recreating and reopening at "Opening"; a repair of CI at
  "Checking", of a gate at "Gating", of a semantic conflict at
  "Queued"; first, updating and resolving at "Queued"; landing at
  "Landed", current. "Queued" reads "Queued · Nth" while it has a place
  in its queue, "Queued · 1st" when first, the place an English
  ordinal ("1st", "2nd", "3rd", "4th", "11th", "12th", "13th",
  "21st").
- **Its step's line,** after the steps, then the "Task N" link of the
  task its step runs, when the panel names one (`runs`):

  | Step | Its line |
  |---|---|
  | producing | "Producing its branch." |
  | opening | "Opening its pull request." |
  | recreating | "Making its branch again at the head temper last knew." |
  | reopening | "Reopening its pull request." |
  | checking | "Waiting for CI at its head." |
  | gating | "Waiting for its gates at its head." |
  | queued | "Nth in the queue for LANDING, after Task A, Task B and N more.": the tasks ahead it names, nearest first, as links; "and N more" for those it does not name; "after N tasks" when it names none; and no "after" when nothing is ahead |
  | first | "First in the queue for LANDING." |
  | updating | "Updating from LANDING." |
  | resolving | "Resolving its conflicts with LANDING." |
  | repairing | "Repairing: CI failed at its head.", "Repairing: gate N asked for changes." or, for a semantic conflict, "Repairing: CI failed at its updated head.", which blames neither the change nor what it was updated with |
  | landing | "Merging at its head." |
  | landed | "Landed · merge commit ABCDEFG", its first seven digits, as a delegation's card says it (5.7) |

- **Its hold,** while it is held, a notice after its step's line: "Held
  while STEP: REASON.", STEP the step it was held at in lower case as
  the table names it ("first in the queue" for first), and REASON:

  | Hold | Its reason |
  |---|---|
  | stalled | "its wait stalled" |
  | repairs | "its repairs are spent" |
  | resolutions | "its resolutions are spent" |
  | updates | "its updates are spent" |
  | moved | "its branch was moved on the forge" |
  | missing | "its branch is missing on the forge" |
  | closed | "its pull request was closed on the forge" |
  | retargeted | "its pull request was retargeted on the forge" |
  | failed | "a delegate or a write failed for good" |
  | rejected | "a person rejected a gate" |

  The reasons are 6.1's and 6.2's, in the panel's measure (`forge.md`,
  section 6): the held card says the rest.
- **Its tasks:** a region and tree named "Its tasks", the tasks its
  procedure made (the agent producing a change's branch, its repairs
  and reviews), where a goal's and a person task's say "Plan". Its
  live tasks are 5.8's plan rows. Its ended ones are 5.8's ended
  rows, read once from the engine, its children in the order they
  were made (`Read::Tasks`, a `TaskQuery` by `parent` and the ended
  phase, `QueryOrder::Ended`, by number), and read again when its tree
  or its document changes, one read out at a time; they are drawn in
  5.8's rows' order. It has no filter and no counts, as its mockup
  draws none. Until that read answers, an empty tree says
  "No live tasks." as 5.8's; after it, a region with no row says "None
  yet." while the task lives, "None." once it has ended. When the
  read's page holds less than all of them, a last line: "More of its
  ended tasks are not shown." The page reads no older page (section
  11). Every procedure's task reads them, its panel shown or not.
- **Gates at its head:** a region of rows, each a name and its state.
  First "CI at its head": "Not yet" while CI is unknown, "Running"
  while pending, "Passed", "Failed". On a landed change, unknown CI is
  "Not read since the restart": a landed change keeps the verdict last
  read at its head with its record, through a restart too, so its CI is
  unknown only when no step read one at that head between the engine's
  last start and the landing, and no step reads it after
  (`../domain/forge.md`, 8.8). Then each gate, in the supplied
  order: "Gate N · review", "· approval" or "· check", then "·
  blocking" or "· advisory", and its verdict: "Not yet" with none,
  "Waiting" pending, "Approved", "Changes asked". With no gate, a last
  row, "No other gate."
- **Its bounds,** the card's last line: "Repairs N of M · resolutions N
  of M · updates N of M", each used of its limit (2.2).
- **Against its mockup,** the page's way:
  - links name a task "Task N", the tag "Change · Task N", the mockup's
    "T28" and "Change · T28" (3.5, "Links are by number");
  - not drawn, since the panel does not carry them (`forge.md`, 9.2):
    the "Fixes finding" notice (findings are a gap,
    `mockups/README.md`); the head's requirements, its heads in order,
    CI's contexts, the pull request's title and mergeability, how long
    a step has waited and when a stall began;
  - not drawn: the mockup's case name beside the tag, a reference
    world's;
  - not drawn: the bar naming its way as 5.1's does; "Its tasks"'s
    subtitle ("made by the change procedure…"); and "its landing rules
    add none" after "No other gate.", which the panel cannot tell from
    a rule not read;
  - its tasks' rows are 5.8's plan rows, with their executor, state
    and spend, where the mockup draws each as a gate's row with its
    kind and phase; a gate is named by its number and kind, where the
    mockup names it by the landing rule that adds it;
  - its hold, the step's line and its bounds are temper's: the mockup
    draws none.
- **Where it is decided:** the domain's task page holds the forge's
  side's panel (`forge::panel`, its state loading, shown, unknown or
  ended) and its ended children's page, its one read and whether it is
  stale; pure functions of the domain's forge side read the panel: the
  step's slot and the steps done (`forge::change::slot`), the line's
  facts and the hold's step. The view lays them out with the forge's
  words (4.2).
- **Its words and look:** "Goal", the existing word; "Its tasks", "None
  yet.", "None." and the more line, the page's; the change's words, a
  forge table beside its parts' (`forge::words`). CHANGE on the card,
  CHANGE_TAG on the header's tag, STEPS, STEP and STEP_DONE as a
  delegation's card's (5.7) and STEP_CURRENT on the current step,
  CHANGE_HOLD on the hold, GATE on each row of "Gates at its head",
  META on the pull request's line and the bounds, TASK_ACTS on the
  toolbar; `task-pages.css` draws them as the mockup's `.tag.accent`,
  `.steps`, `.step.current`, `.notice` and `.gate` (`browser.md`, 2.5),
  the steps in six columns, three at phone width.

### 5.11 A task's page's acts

A goal's page (5.8) and an agent's task's page (5.9) act on their task
as a chat's page acts on its chat (3.5): through the acts its document
offers (`Task.acts`, read through `acts::offers`), each sent as 3.6
says. An act the document does not offer is not drawn, not even
disabled. One it offers is drawn by its admission (jig's
`client/domain/contract.md`, 5.3), as a held task's card draws its own (3.6):

- *allowed* or *bounded:* its button;
- *proposable:* its name, then the card's "This needs a proposal to"
  and to whom, and no button: the page proposes none of these acts;
- *refused:* its name, then the refusal in jig's words, and no button:
  the page sends nothing.

Its rules on a goal's page:

- **Where they are:** `.acts` in the header (5.8), "Amend", "Pause" or
  "Resume" (`.btn.small`), "Close" (`.btn.quiet.small`), keeping `.btn.small`'s
  height at phone width, wrapping as drawn. Ended, none of the three.
- **Pause** sends `Ask::Stop` at once, no confirmation. A person's Stop is a
  placed hold over the goal's subtree. Its toast: "Paused: running tasks stop,
  nothing new starts."
- **Resume,** on a goal in state paused alone: `Ask::Release` naming the hold's
  revision, at once, no confirmation. A subtree hold's root released lifts it
  whole. Its toast: "Resumed." A goal held for any other cause has no Resume:
  its band has the need.
- **Close** opens the confirmation (`confirm.rs`, `Role::Dialog`, 3.6, *The sheet*):
  "Close this goal?", then "You can still read it, but it can’t be reopened." and
  "What landed on its branch stays there.", "Keep it", "Close goal". It sends
  `Ask::Cancel` with the reason "Closed from its page". A goal branch nothing
  landed from stays (`../domain/forge.md`, 11.1), so the sentence holds. The
  acts stay drawn behind the dialog.
- **Amend** opens the amendment's editor below, between the header and the
  bands, in the page's look: `.field`, `.text`, `.btn`.
- **The composer,** while live and Message offered: the mock's `.composer`, as
  the chat's: its placeholder and name "Tell TITLE something", Send disabled
  while empty, Enter sends, Shift+Enter a new line, the focus in it after a
  send (3.5), growing to 220 px; then
  `.composer-hint` "Its agent reads it at once. A change of plan becomes an
  amendment it shows you first." While Message is refused as `NotWaiting`, the
  composer's place holds nothing; ended, no composer.
- **The toast** is the frame's (3.5, *Needs you*: one at a time, one optional
  act). It says Pause's and Resume's done words, a Pause's, Resume's, Close's or
  Message's refusal in jig's words, with "Look again" after a moved one, which
  closes it, and "Unknown" past the engine's retention. Nothing is shown done
  before its answer; a pending request's double press joins it (3.6).
- **No own held card, no held rows:** a goal's page reads no card of its own and
  none for a delegate: a held delegate's Release is on its own page, and a
  need of the reader's is a band.

Its rules on an agent's task's page:

- **Where they are.** In its taskbar after its spend, as a chat's chatbar
  holds its acts: "Amend", "Pause" or "Resume", "Close"; and its composer last
  (below).
  The taskbar's acts wrap as a chat's do. While the page's confirmation is
  open, the taskbar draws no act: the dialog is the one place to answer.
- **Its own held card.** While the page's task is held for any cause but
  the person's own pause, its held task's card (3.6), with Release and its
  confirmation, follows the agent's run heading. The task is no row of its
  own plan, so its plan never draws it; a delegate's held card stays in its
  row. Release names the hold's revision, and a moved one's refusal asks the
  person to look again (3.6).
- **The composer,** while Message is offered: the chat's composer (3.5, 5.1),
  with its model choice line, its text area "Message", its hint and "Send",
  preceded by the note "Steer this task · words reach its live run at the next
  turn." A message sent shows in the conversation as a chat's person's words
  do, "Being sent", then "Sent". The engine offers Message on every live
  agent's task and refuses it while the task waits to start (`NotWaiting`): the
  composer's place then holds "Message", then a note in the steer note's look,
  temper's "Messages reach it once it runs." (any other refusal: its supplied
  words, in the same look, since why Message isn't offered is no outcome), and
  its model choice line alone.
- **Pause and Resume** are a chat's (3.5, *A person's pause*): `Ask::Stop` at once,
  and on the person's own pause "Resume", `Ask::Release` naming the hold's revision,
  at once. The page's state says "Paused"; neither draws a done word, and a refusal
  says why as any of its acts' does.
- **Close,** confirmed (3.6, *The sheet*): "Close this task?" saying "You can still
  read it, but it can’t be reopened." and what it changes, then "Keep it" and "Close
  task". It sends `Ask::Cancel` with the reason "Closed from its page".

On both:

- **Amend** opens the amendment's editor, laid out where the page says (above):
  a region "Amend task N". The draft
  is the amendment editor's (3.7), `amend::Draft::of` the task's
  document as read when it opens, and the page lays out two of its
  edits, no other:
  - "Specification", a text area holding the spec's words, its edit
    `words`; where the read cut the spec (`Cut::Spec`), the words read
    only and "This task's specification is too long to edit here: use
    `temper ctl task amend`";
  - on a goal, "Tracked", a check, and "Priority", a number, its edit
    `track`: `Tracked { priority }` or, unchecked, `Untracked`.

  Then "Send amendment" and "Discard". "Discard" closes the editor and
  drops its draft. "Send amendment" with no change (`amendment()` is
  none) sends nothing and says "No change". While a document read since
  has another spec revision (`Draft::moved`), the editor says "This task
  changed since the editor opened" and draws no "Send amendment": the
  person discards and opens it again on the new document. Otherwise it
  sends `Ask::Amend` with the drafted change, the drafted revision
  (none for a tracked task's priority alone, 3.7) and the reason
  "Amended from its page"; a refusal as moved in between says so with
  "Look again" (below). Its outcome is the editor's: pending, then the
  answer in jig's words in the region, a refusal as moved with "Look
  again". A done amendment leaves the region holding its outcome alone,
  where it was made, until "Amend" opens another draft. Amend is drawn
  by its admission as every act is: proposable, the page proposes
  nothing. The model choice for the next runs stays the choice line's
  (3.5), its own amendment, not repeated in the editor. Instructions,
  dependencies, the deadline, wake and an addressee are `temper ctl
  task amend`'s alone (`tools.md`, 5.5).
- **Their outcome.** One request per kind of act at a time; a double
  press joins the pending one (3.6). On an agent's task's page: pending, its
  progress as a card says it; answered, the outcome in jig's words, or the
  refusal in jig's words, with "Look again" after a moved one; past the
  engine's retention, "Unknown". It follows the conversation, as a chat's. On a
  goal's page, the toast (above).
- **Reading the conversation,** on an agent's task's page, is the chat's
  (5.15): the log's head asks for earlier activity and its foot for later
  activity, through the same thread, with no "Older" button. Latest returns
  to the window's foot and asks for later activity while the end is missing (5.15).
- **Against the mockup,** an agent's task's page's way:
  - the composer's text area is named "Message", the chat's, where the
    mockup's is "Write to agent task" with a placeholder "Write to
    T12…" (section 11, the agent's placeholder); its hint is the
    chat's;
  - the composer carries the chat's model choice line, which the
    mockup's agent page does not draw (3.5);
  - the composer shows while Message is offered, the mockup's while the
    task lives, and a refused Message says why in its place (3.6);
  - each confirmation, and an act's outcome and refusal, are 3.6's: the
    mockup's buttons do nothing;
  - its own held card, which no mockup world draws;
  - the acts hidden while the confirmation is open.
- **Where it is decided:** the domain's task page holds the composer's
  words, its pending requests by kind and the last outcome, as a chat's
  page does, and the chat's acts (`Act::MessageWords`, `Send`,
  `Older`, `Newer`, `Latest`) are routed to whichever of the two pages is open; a goal's and an
  agent's task's Pause and Resume (`Act::Pause`, `Act::Resume`), and a chat's
  (`Act::Stop`, `Act::Resume`), are sent at once; an agent's
  task's Close and a goal's Close (`Act::Close`) open their confirmation first, as
  `Confirm` with the page's task as its object, and `Act::Confirm` sends them.
  An agent's task's page reads a card for its own task when its document is
  held, as it reads one for a held row. The view draws each act by
  `acts::offers` and `acts::allowed`.
- **The look:** on a goal, `browser.md`, 2.5, *A goal's look*. On an agent's
  task's page, AGENT_STEER on the composer's note, the mockup's
  `.agent-steer-note`; TASK_AMEND on the editor's region. The taskbar's
  acts and the composer keep the agent's page's tokens (5.9). `task-pages.css` draws
  them (`browser.md`, 2.5).

### 5.12 Times

- **The view's clock is the shell's.** The view builds at the wall
  time of the shell's iteration (6.2; `browser.md`, 2.1) and says each
  time the domain's documents carry against it, both nanoseconds since
  the epoch. The same documents at the same time give the same words.
- **Four forms, the mockups' own** (`mockups/pages.js`). For a time T,
  E is the whole minutes from T to now, and 0 when T is ahead of now,
  as a machine's clock behind the engine's makes it:
  - *ago,* when something last happened: "just now" while E is 0;
    under 12 hours, E as a span with " ago" ("12 min ago", "1 h 28 min
    ago"); then by the calendar's day, "at HH:MM" on now's day,
    "yesterday" on the day before, the weekday ("Tue") within the six
    days before that, and earlier its date ("20 October"), with its
    year when not now's;
  - *span,* how long something has waited: "under 1 min" while E is
    0; "M min" under an hour; "H h M min" under a day, "H h" on the
    hour; then "N d", in whole days;
  - *clock,* when a moment of a conversation was: "HH:MM" on now's
    day, the weekday and "HH:MM" ("Tue 07:20") within the six days
    before it, and earlier its date and "HH:MM" ("20 October 07:20"),
    with its year when not now's;
  - *the day's rule,* a line before the first turn of each day a
    conversation's window holds, whose turns then say "HH:MM" alone:
    "Today · Thursday 22 October", "Yesterday · Wednesday 21 October",
    or the day's name and date ("Tuesday 20 October"), with its year
    when not now's.
- **The person's zone** sets the calendar's days and the hours the
  forms say: the browser's, in minutes east of universal time, an input
  of the shell's (`browser.md`, 2.1), read at start, so that a change of
  daylight saving shows at the next load. The terminal has none: universal
  time. *ago* and *span* are differences, and it moves neither. The words
  name no zone, as the mockups' don't.
- **The tick.** With each build, the view keeps the instant at which a
  word it drew next changes: the next whole minute of an ago under 12
  hours or of a span under a day, a longer span's next whole day, and
  the next midnight while it drew a word of the calendar's (a later
  ago, a clock, a day's rule). The shell wakes at that instant, and the
  view builds again although the domain's shown count has not moved
  (6.2; `browser.md`, 2.1). A page that draws no time asks for no wake.
- **Where they are said:** a chats line's last activity or end, an ago
  (3.5); a chat's turns and its end, clocks under the day's rules
  (3.5); a card's wait, a span, in the inbox and on a chat's page; an
  update's arrival in the inbox, an ago (3.5). The task pages and the
  accounts page say their moments as their sections do (5.5, 5.8,
  5.9), in universal time, and so does a full deployment's earliest end
  (5.1, the pages not about the work).
- **Fixed in the tests.** The view world, the terminal's goldens and
  the gallery build at a fixed time: the fixtures' `NOW`, and each of
  the gallery's worlds its own (`browser.md`, 7.2). A test of a form
  steps that time to either side of the bound it reads (section 10).

### 5.13 How work runs, and a proposal of settings

The settings, their revisions and their proposals are
`../protocol/configuration.md`, section 13, and jig's
`domain/engine.md`, section 17; the draft is `../draft/configuring.md`,
section 5.

- **The tab,** `Settings { project }` (3.5, *Settings*), under its tabs: its
  sections 44 apart, each a section title and its rows, each row a label,
  small words and a control at its right, a phone stacking them:
  - *Configure temper,* with no title: "Tell it how you work, in your own
    words or by pointing at a document. It proposes the settings; nothing
    changes until you accept."; "Start", primary, to `Configure { project }`;
    or "Go on", to the person's newest configuring chat open in the project
    (3.5). To begin's step keeps its words, "Tell temper how you work". The
    configuring page is a page of its own, its path PROJECT / Settings /
    Configure temper; it is not folded into this tab or docked beside it.
  - *What the project shows,* live, kind 7: a row each, the state's word then
    a switch:

    | Label | Small | Words | Member |
    |---|---|---|---|
    | Chats can write drafts | A chat keeps a document beside it, in docs/design/draft/, that a goal can start from. | On / Off | `drafts` |
    | Goals | Bigger work from a draft: a plan, tasks, landing. | Shown / Hidden | `goals` |
    | Changes | The landing queue, as a page. | Shown / Hidden | `changes` |
    | Away switch | Lets agents decide within limits while you’re away. | Shown / Hidden | `away` |

  - *Ways of working,* live, kind 7's `way`, only while Goals is shown: a row
    per template of the project with `tracked` set, in number order, named and
    worded as the Goals page words them, one table for both: its name; its line, then " Used by
    N." or " Used by none yet." where a goal's template is recorded; a radio,
    " The default". A stored `way` of 0, or of a template no longer offered,
    reads as the first. A goal's start preselects it.
  - *Status,* disabled, each showing what temper does today: "Status lines and
    reports", "Written words, beside the counts temper keeps itself.";
    "Rewritten"; "The brief on Home", "What changed since you last looked.", "On".
  - *What needs you:* "Questions a goal can answer from its plan", live, kind
    6: "Answered by the goal, kept in its decisions" (`foreman`), "Asked of me
    too" (`up`), "Not asked: each agent decides alone" (`none`); "Funds",
    disabled, "Ask when a budget is used up".
  - *While you're away, agents may decide,* after What needs you, shown only
    while the Away switch is; its controls write through `SetAway`.
  - *Telling you,* disabled: "Notifications", "Only when something needs you;
    never for status.", "Off"; "Quiet hours", "None".
  - A disabled section's last line says once "Not built yet: this is what
    temper does today."; Funds' small words say the same. No control does
    nothing: whoever builds a mechanism later turns its row on.
- **What the client holds:** jig's `settings` watch, for the session, as the
  `dismissed` and `forges` watches are held, Home reading the same one. It reads
  kind 7 of the open project as the project opens, and kind 6 on the tab,
  through the forge's reads in that project's scope (`forge.md`, section 9),
  and again whenever the document's line for that setting says it changed and
  when the link comes back. A transient read failure (busy, unreachable, local
  read capacity, or sign-in still settling) retries after one second, doubling
  to at most 30 seconds; a successful answer resets the wait. Kind 7 retries
  while its project is open, kind 6 while the tab is open; leaving that scope
  drops its retry. Until first read, or after a lasting refusal, the optional entries
  read as hidden, with 5.3's exceptions for a current page and a project holding a
  goal; `drafts_offered` is false, with the configuring chat's exception (3.5).
  The tab's live controls draw the stock, disabled; a value of none reads as
  the stock (`../protocol/configuration.md`, 13.4).
- **Its acts:** each live control sends at once jig's `ChangeSettings` with one
  item, running work staying, against the newest revision the person has, the
  one exception to the pages sending proposals alone. The control is disabled
  while it is out, and so are the others of kind 7, since they write one record;
  the value asked shows meanwhile. A refusal is said under its row in jig's
  words, `full` and a person who is no owner too, the control back at its
  value. A `stale` refusal is sent once more against the revision it names.
  Answered, the client takes the value at once and reads it again.
- **The terminal:** the switches as checks, the selects as selects, a disabled
  one marked so; it all prints.
- **What the settings change:** the rail and the tab bar (5.3), `goals` hiding
  the offer of Goals and not a goal the project holds; `drafts`,
  `pages::chats::drafts_offered`, the chats' offer of a draft, a configuring
  chat's apart (3.5), and nothing else;
  and these open the tab, `Address` having no anchors: Home's "Show goals",
  "See the settings" and "Limits".
- **The card in a chat:** a proposal of settings is a card at the call that
  made it where its turn names it, and otherwise after the turn that was running
  when it was made, as every proposal is (3.5; *Where a card stands,* below): the title "How temper will run
  PROJECT", PROJECT its `projects`' names joined by ", " and " and ", or, with
  none, the deployment's, "How temper will run your projects"; the proposal's reason, in the person's words, its Markdown drawn as the Accept sheet draws it (3.6);
  then each item, small and faint in the card's type, in the words of `temper ctl` but
  for temper's kinds of a project: kind 7 as "Drafts: on", "Goals: shown",
  "Changes page: shown", "Away switch: shown" and "Way: NAME", kind 6 as
  "Questions: answered by the goal", "Questions: asked of me too" or "Questions:
  not asked"; a template by its name, as "Way: NAME" names one, or "template 4"
  where the person's `me` names none; any other as "charter 2", "landing rule 1".
  A proposal's heads name jig's own settings alone, so a live card takes its
  application's items and project's policies from the proposal's review, and
  those values from the forge's read of a proposal's items under the review's
  `key`, which takes a role in the deployment
  (`../protocol/configuration.md`, 13.2). Until those values arrive, kinds
  6 and 7 are named as `temper ctl` names them. With no review, no application
  or project-policy rows can be recovered from the heads. A decided card has
  no review: it keeps the kind 6 and 7 values it read in this session, and
  otherwise draws its heads alone. Then
  "Accept", primary, jig's `Decide` with running work staying, and "Change
  something", quiet: it focuses the composer, brings it into view and sets its
  placeholder to "What should change?", until something is sent or the proposal
  is decided, when the placeholder goes back. A decision goes under the proposal's
  own `project`, never the chat's: a proposal of settings is the deployment's,
  whichever chat made it, and jig knows it under no other project. The card has
  no "Reject": the person rejects it on its card in *Needs you* (3.5). Saying
  what to change here has the agent propose again, whose fresh proposal replaces
  the one that waits (`../domain/agent.md`, 8.1): the new card stands below. An
  "Accept" pressed on a replaced card, in a tab that hadn't heard, is refused and changes
  nothing, and the refusal ends the card as withdrawn: "Replaced by the proposal below",
  or "Withdrawn · nothing changes" until it has read the later one. Decided, it ends in a muted state line in place of its
  acts, however the decision was made (*Needs you*, another tab, `temper ctl`), the
  chat's task document telling it with no reload: accepted, "✓ Accepted · new work
  follows it"; rejected, "Rejected · nothing changes"; withdrawn, "Withdrawn · nothing
  changes", or, a later proposal of settings of the chat's having replaced it, dimmed,
  "Replaced by the proposal below". While it is pending, a person who may not accept it
  sees neither act: "Waiting for an owner", the parties it waits for as jig's proposal
  names them, "Owners of this deployment" for a holder that is the deployment's policy.
  Before its document is read it says none of these. Nothing of it
  says which items temper enforces and which agents follow: that is the review
  page's, below, once it is built. In the inbox, its card shows the same lines
  (3.5).
- **What a rule on a box holds against:** where the forge's read of the
  proposal's items answers `owned_approval` (4.6), a landing rule of it with an
  enforcing approval covering a repository on an owned Forgejo, its card says,
  after its items and in their look, "On this machine temper's agents and CI
  jobs run as you, so this stops their mistakes, not one trying to get past
  it." (`../domain/forge.md`, 8.12; `../draft/forge-identity.md`, section 6),
  in a chat and in *Needs you* alike, and its review page says it under its
  plain words once that page is built (below). Until the read answers, and
  for a decided card with no review, it says nothing of it.
- **Where a card stands:** jig's activity says when a proposal was made: a
  `Decided` line naming it, with `made` (jig's `client/domain/contract.md`, 5.1).
  With no call naming it, its card stands after the first committed turn after that
  line, and with no such turn yet, its turn still running, at the conversation's end.
  So a new pending card is the last thing in the chat, the chat stays at its end as
  usual, and the reply after an Accept follows the card. A pending card whose line
  isn't in the window (a delegate's has none) goes by its waiting time: after the first
  turn committed at or after it, or at the end. A card whose place is past the window's
  end isn't drawn until the window gets there. Cards at one place stand in the order
  they were made.
- **A decided card after a reload:** that same line brings its card back, read as a
  pending one is: jig answers an ended proposal with its reason, items and `projects`.
  It stands at its place, through a reload and paging back, with its state line. Past
  the bound of what jig retains (`inbox_messages`, its read answering `Gone`), no card
  stands. A proposal that isn't of settings draws no card once decided.
- **"Decision made":** a line of a proposal's making draws no row, being no
  decision; and where a card stands, its proposal's lines draw none. A card and a row
  never both say it. A proposal with no card standing keeps its row.

**Later: the fuller page.** Roles, ways to start work, rules, running work and
history join the tab, the review page opens, and the card gains "Review". The
pages then read jig's `settings` document and each setting they show whole
(`setting`, jig's `client/domain/contract.md`, 5.1 and 6), again as the document
says it changed; temper's kinds' values, and a proposal's items of them, from
the forge's reads (`forge.md`, section 9). A party sees the deployment's settings
with a role in the deployment, and a project's with a role in that project, as
the document gives them; the client hides nothing of its own.

- **More of the page:** the newest revision, "Revision 8, accepted by Pat today
  at 14:03" (who decided it and when, 5.12; "temper's defaults" for the
  first start's and "temper 0.4's defaults" for a release's); each
  pending proposal of settings as a line, "Proposal 13 waits for an
  owner. Lee asked for a Reviewer role, from "Reviews"", with "Review".
  Then five sections, each a level-2 heading with its count and a line
  of what it holds, its rows folded until opened, as the accounts' are
  (5.5):
  - *Roles,* the charters: each its name, "a role · temper's stock" or
    "added in revision 6", a tag, and its line, a sentence of its
    instructions' first paragraph; opened, its instructions (temper's
    for a stock one, its own otherwise), its models, where it runs, how
    it waits and its budget, and how many tasks run it, by revision;
    the forge's charter says "Made by temper when a change needs a fix,
    a rebase or a gate".
  - *Ways to start work,* the templates: each its name and line;
    opened, its role, its budget and deadline, and how it closes.
  - *Rules,* what temper holds work to: the deployment's landing rules
    and, for the project chosen, its own, each its branches and what
    it needs ("Needs CI passed · up to date · gate passed · 1
    approval by an owner") and who decides it ("Decided by acme/app's
    owners: Pat, Kim"); the policy new projects start with; and jig's
    rules, "Everywhere, always". A project's people and its own policy
    are its own page's, not this one's.
  - *Running work,* the revisions held: the newest, "Newest · 14 tasks
    · new work takes it", and each older one, its tasks' count, and
    "Move them" (jig's `MoveToNewest`), confirmed (3.6) as "Move 9
    tasks to revision 8 at their next run? A task whose role revision 8
    removed, or whose model it no longer allows, finishes on revision
    7."; the answer's count of those that could not move is said after
    it.
  - *History,* the revision records the store keeps, newest first:
    each its number, what it did in a line (a proposal's title, or "a
    release's", "the first start's"), who decided it and when, "Show",
    and "Restore…", which proposes the settings as that revision left
    them, against the newest (jig's `domain/engine.md`, 17.10), saying
    which of temper's kinds' settings it leaves out, their past values
    not kept, and opens that proposal's review.
- **Its tags:** "Default", a setting no change is over; "Changed", one
  a change is over, with "changed in revision N"; "Added", one no
  default has.
- **Proposals, but for the tab's own rows** (`../draft/configuring.md`, section
  10; jig keeps the owner's direct change in its API, and the pages send
  `ChangeSettings` only for a row of the tab that is one setting, above). A later row's act is "Change it in a chat", which
  starts the configuring chat with the row named in its first message.
  "Reset to default…" on a changed default, "Remove…" on an added
  setting and "Restore…" in the history are the person's own proposals
  (jig's `Propose`), each confirmed (3.6) and opening its review page.
- **The card, with the review page:** at most five lines and "and N more",
  then "Review", to its review page; nothing is accepted from it once the
  review page reviews it.
- **The review page:** the level-1 heading "Proposal 12 · from "How we
  work" · against revision 7"; "Made by T21 How we work, for Pat, today
  at 13:56"; then two blocks:
  - *temper enforces,* "temper holds work to these, whatever an agent
    does.": the changes of every setting but a charter's instructions,
    grouped as How work runs groups them (roles, ways to start work,
    rules), each before and after in words, with "new" and "removed";
  - *your agents follow,* "Agents read these at each run and are
    trusted to follow them; temper does not check.": the changes of a
    charter's instructions, each as the role and what it now reads.
  Then *Effect*: "New work follows this from the moment it is accepted.
  Nothing restarts."; "9 running tasks are on revision 7:" with two
  choices, "Let them finish on revision 7", chosen, and "Move them at
  their next run"; and "A task whose role this removes, or whose model
  it no longer allows, finishes on revision 7." Then "Show the
  settings", folded, the items as the settings name them (charter 4,
  its models, before and after), and the acts: "Reject…", a reason,
  sent as jig's `Decide`, which the agent hears and revises from;
  "Change something", which returns to the chat with the person's
  words; and "Accept", jig's `Decide` with the
  choice of running work, naming the proposal by its proposer and
  number (jig's `client/domain/contract.md`, 3.2 and 5.2); the revision
  it was made against is its own, and one made against an older is
  withdrawn, never accepted (jig's `domain/engine.md`, 17.6).
- **What the review page says plainly:** a landing rule naming a role
  nobody holds cannot be accepted: the refusal names the affected
  project, role, required and eligible forge-person counts, and asks
  for known forge people with the role or removal of the approval
  (`../domain/forge.md`, 8.9). An owner login or display name supplies
  no forge identity. A change of jig's rules that removes the
  landing's requirements (`../protocol/configuration.md`, 13.3),
  "temper no longer holds landing to its rules: the change procedure
  still waits for them."; a role removed while tasks run it, "2 tasks
  hold it on revision 7"; a landing rule with an enforcing approval on an
  owned Forgejo, the card's sentence for it (above).
- **Stale:** a proposal made against an older revision is withdrawn by
  the decision that made the newer (jig's `domain/engine.md`, 17.6). Its
  review page keeps its items, dimmed, under the banner "Withdrawn:
  made against revision 7; now 8.", what revision 8 changed in a line
  and who made it, and "T21 How we work is redoing it against revision
  8; its new proposal comes to your inbox."; it offers no Accept, only
  "Open the chat". In the chat the card says "Withdrawn: made against
  revision 7; now 8", and the agent, told so, proposes again against
  the newest, a new card after it.
- **Accepted:** the review page says "Accepted by Pat at 14:03:
  revision 8. New work takes it now.", with what became of running
  work ("9 tasks finish on revision 7", or "moved"), and "How work
  runs"; in the chat, the card's line "Accepted at 14:03: new work
  takes it now." A refusal of Accept (out of shape, judged, full) is
  said under the acts in jig's words, the forge's for a landing rule
  it refused (`../protocol/configuration.md`, 13.2), and the proposal
  stays pending; full names the oldest revision held and offers "Move
  them" there.
- **Drawn ahead of jig:** the names of a revision's tasks, "Show them"
  and how many of them could not move are not in jig's facts (section
  11); the page draws counts alone until they are. A charter's
  documents, "Owners also read acme/handbook process.md", wait on jig
  (`../protocol/configuration.md`, section 11).

### 5.14 Starting a goal

The sheet, the mock's `startSheet` (`greenfield/`), `pages/start_goal.rs` in the
view: a `Role::Dialog` over the page (`dialog.sheet`, `::backdrop`), opened by
`Act::OpenStart { draft: Option<Box<[u8]>>, chat: Option<u64> }`, closed by
Cancel or Escape. A domain struct on `Pages` holds its fields, so that any page
can open it. `draft` is the draft's path in the repository
(`docs/design/draft/away-mode.md`), and its FILE the path's last segment.

- **A form,** in order:
  - `h2` "Start a goal from FILE", then `.muted` 14px "The goal takes the draft
    as it is now. Later edits reach it through Amend." With no draft given: `h2`
    "Start a goal", and a `.field` "Draft", a text field, placeholder
    "docs/design/draft/name.md", which Start requires;
  - `.field` "Name": `input.text`, FILE without `.md`, following the Draft field
    until the person edits it;
  - `fieldset`, legend "How it runs": a `ChoiceGroup` of `label.option` per way
    (the project's tracked templates in number order, 3.5, *Ways*):
    `<b>WAY</b><small>TEXT</small>`. The default adds " PROJECT’s default." to
    its text and is chosen: the way kind 7 names (5.13). After them, `.link` 13px muted
    "The project’s ways of working", to `Settings { project }`;
  - `fieldset`, legend "When it needs you": a `ChoiceGroup`, drawn as the ways:
    "Follow the away switch", chosen by default, small "Asks while you’re here;
    while you’re away, decides within the project's limits and tells you.";
    "Always ask", small "Waits for you on every question it can’t answer from
    the plan." `Act::StartAsks { always: bool }` changes this choice, as
    `Act::StartWay` changes the way. Both choices are disabled while a request
    is out.
  - `.foot`: `.btn.quiet` "Cancel", `.btn.primary` "Start goal", disabled while
    Name or Draft is empty or a request is out.
- **Not drawn:** "Budget": nothing stands behind it yet (section 11).
- **With nothing to fund it:** while the project's first creating bound gives no
  budget (`creating.within.budget` 0) and the way's budget isn't a fixed amount,
  `.sheet-error` above the foot, and "Start goal" is disabled; `Me` is watched, so
  it clears when a chat closes. While the party's pool has reservations: "Nothing
  is left in your pool to fund a goal: your open chats hold it. Close one you’ve
  finished with, and you can start this."; with none, "Nothing is left in your pool
  to fund a goal." Funding's rules are as ever. Stock reaches it: a role 2 member's
  pool is one chat's spend, so one open chat takes it, and an owner's is eighty's
  (`../../help/budget.md`).
- **Start** calls `shapes::set_goal_from(me, project, number, GoalDraft { words,
  executor, budget, deadline: None, priority: None, llm: None }, "Started from
  its draft")`: `executor` the template's charter, else `Agent { charter: 1 }`;
  `budget` the first creating bound's (jig's `client/domain/contract.md`, 5.3).
- **The goal's words,** which the goal's page reads back (5.8):

  ```
  NAME

  Build docs/design/draft/FILE.
  Draft: docs/design/draft/FILE
  Chat: T4
  ```

  The first line is the task's title, as jig takes it. `Chat:` only when a chat
  opened the sheet, whose document's snapshot then stands in the "Build" line's
  place (3.5). "Always ask" appends the exact line `Asks: always`;
  "Follow the away switch" adds no asking line. Nothing stores the choice
  apart from these words.
- **Its ends:** `Judged::Create`: sent, and on its answer to the goal's page, which
  shows planning. `Judged::Propose`: sent, the sheet closes, toast "Proposed to
  WHOM: it starts once accepted." `Judged::Refused`, `NotOffered` or a refused
  answer: `.sheet-error` in the sheet, in jig's words, and it stays open.
- **Where it opens:** on a chat, its "Start goal…" (3.5); on Goals, the `.link`
  "Start one from a draft" at the section head (3.5, *Goals*), with no draft, shown
  while the project has a tracked template: `Act::OpenStart { draft: None, chat:
  None }`, a button in the link's look, as it moves nowhere, for the project in the
  address; a draft without a chat, or a chat without a draft, opens nothing. A goal
  started this way asks its person as a chat's does (`../domain/agent.md`, 8.2); its
  words have no `Chat:` line. The rail's "Goal" stays what 5.3 has it, a new chat for
  a new draft.
- **The chat's thread:** a person's Create leaves no turn, so its "Started goal"
  line is drawn from the fact `GoalFrom` (3.5).
- **The look:** `browser.md`, 2.5, *A goal's look*.

### 5.15 Reading a conversation

A chat's conversation and an agent's task's (5.9) are read by scrolling
(`../draft/chat-reading.md`, section 5). jig's thread holds the window
(jig's `client/domain/client.md`, section 10), and skein's page part
measures it (skein's `ui-web.md`, section 7).

- **The log is bound** while an earlier page remains or the window has let
  its newest lines go, its act decoding to `Older` at its head and `Newer` at
  its foot. There is no "Older" button. Earlier lines load when the head
  comes within one and a half screens, whatever brought it there. The same
  rule opens a chat: a long one opens with at least a screen and a half,
  a short one whole.
- **The window** fills `document_bytes`, with no count of lines: its `lines`
  is what its bytes hold (`thread_limits`). `Limits::window_bytes` names a
  narrower window, of at least `long_text` and at most `document_bytes`, where
  a bundle asks for one: the browser tier's alone (`browser.md`, 4.3). The
  page's bounds are priced from `document_bytes` either way (5.4). Reading
  earlier past the window lets the newest whole lines go,
  and reading later the earliest; they come back when scrolled to, and nothing
  says how many left.
- **While a window has let its newest go** the log *continues* (skein's
  `ui.md`, 3.2), so it isn't followed and "Latest" shows, even at its foot.
  New words and a run's live text wait: they show when the end is back.
- **A delivery that makes the view stale** rebuilds the semantic page tree,
  rereads the held Markdown and compares the old and new trees across the
  window. Unchanged builds are gated; the diff patches changed DOM nodes in
  place. No parsed Markdown or conversation subtree is retained. Following
  reads aggregate log geometry; holding an away reader can scan entries for
  its anchors, including on a patch outside that log (skein's `ui-web.md`, 7).
- **Each line's outer node is keyed,** a turn by its name, a run of calls by
  its first turn's, a person's words by their message, an activity line by its
  position. So a page read in above is an insert, nothing already drawn is
  rewritten, and the page part can keep the words where they are.
- **A chat's log head holds nothing that stays while lines arrive below it,** but
  the status below: the page part holds the first entry in view. So the context
  line (5.6) draws once the window holds the chat's start, and a day's label is
  keyed by its date and the first line under it, so that a page of the same day
  read in above replaces it. An agent's task's log keeps its brief, run heading
  and own held card before its lines (5.9); their effect on reaching earlier
  lines is section 11's *An agent's page opening at its end*.
- **While a read is out** the log is *busy*, and so it stays after a failure,
  which the page part then leaves alone until the person moves. At the head:
  "Loading earlier messages…" while an earlier page is out, "Couldn't load
  earlier messages" in the same look after one failed, and nothing once the
  chat's start is shown. A read at the foot says nothing, but a chat's first read,
  failed with no line to show, says the same at the head. An answer the client
  cannot decode is such a failure (jig's `client/domain/client.md`, section 16): it
  asks nothing again until the person moves, or a new line is committed, and the
  page's other lines and acts go on. Lost exchanges and busy reads keep their
  transient retries.
- **"Latest":** the view returns to the window's foot. On a chat where the
  window let the end go, a press reads the end again (`Thread::latest`),
  which replaces the window. A person sending on a chat does the same
  through the log's end request,
  which `Chat` counts. An agent's page has the same pill; while its window
  has let the end go, a press asks for the next later page (`Thread::newer`),
  and the log, following, asks for each next until its end is back (skein's
  `ui-web.md`, section 7). Its log has no end request, so sending there does
  not request the end.
- **The terminal:** the log is bound and continues as in a browser, with no
  measures: `up` reads earlier and `down` later (skein's `ui-text.md`, section
  6), and each prints the whole window again. These are lines followed by
  Enter, as are the escape sequences PgUp and PgDn make, not raw keys.
  It opens with one page. The line under its bound log's name says
  `(up, down: more lines)`; there is no pill, and sending on a chat reads
  the end again.

## 6. The terminal shells

### 6.1 `temper open DIRECTORY LINK_OR_ADDRESS`

- **DIRECTORY** is the client's private directory. The terminal opens
  its parent as a root (skein-shell's `open_root`) and gives its last
  component to `jig-client-shell-native`, whose files make it `0o700` if
  it is absent and refuse it unless it is a directory of this user's
  with no group or other bits (skein's `io.md`, 5.3). It holds the
  bearer credential's file and the small store's, both the native
  shell's (section 7).
- **LINK_OR_ADDRESS** is one of:
  - the link `temper serve` printed,
    `http://HOST:PORT/jig/v1/sign-in/link?secret=SECRET`: the client
    signs in by it first, offering the credential DIRECTORY holds, if
    any, beside it, so that a person still signed in keeps their sign-in
    (section 7);
  - the engine's address, its clients' listener's `http://HOST:PORT`,
    with or without a final `/`: the client signs in with the credential
    DIRECTORY holds. Before its loop, `jig-client-shell-native` reads
    `DIRECTORY/token` once, as it loads the small store, and the host
    gives it to `jig-client` as held (section 7).

  HOST is an IP address, and must be on loopback (in `127.0.0.0/8`, or
  `[::1]`).
- **Exit status:** 0 once the client closed and drained (6.6); 1 when
  it cannot start, with one line on stderr:

  | Cause | stderr |
  |---|---|
  | not two arguments | `temper open: usage: temper open DIRECTORY LINK_OR_ADDRESS` |
  | not an `http://` link or address, or HOST not an IP address and port | `temper open: not a link or an address: ARG` |
  | HOST not on loopback | `temper open: the engine's address must be on loopback` |
  | DIRECTORY refused | `temper open: DIRECTORY: ` and what skein's io found unsafe or failed |
  | an address given and no token in DIRECTORY | `temper open: not signed in: give the link temper serve printed` |
  | the link refused by the engine | `temper open: the link was refused: use the fresh link temper serve printed` |
  | the engine's sign-ins full (6.7) | `temper open: all N sign-ins are in use; the earliest ends at T; this link stays usable until U; after that, temper serve prints a fresh one when it is tried` |

  A panic aborts the process (fail-stop); the small store and the keys
  make what was pending safe to send again at the next `temper open`.

### 6.2 The loop

- **One host on skein's loop** (skein-shell's `drive`; skein's
  `shell.md`, section 12): temper's client root, its view,
  `jig-client-shell-native`'s transport, and skein's text shell's
  terminal (skein's `ui-text.md`, section 7), with SIGINT and SIGTERM
  from skein-shell's `open_termination_signals`, adopted by io.
- **The text shell's settings** (skein's `ui-text.md`, section 3): the
  name `temper`; the longest line, 4096 bytes; a print interval of one
  second; the status region, the link's state (5.1); temper's
  lines, 32 (6.3); and their line bytes, DIRECTORY's bytes and 160:
  6.6's signed out holds DIRECTORY and 84 bytes, and a service's
  credential's line its credential's base64 and at most 57.
- **Time and the seed** are skein-shell's: its clock once per
  iteration, and a seed from the kernel at each start (`seed`), from
  which the client root's children draw their keys' numbers (3.1).
  The view builds at the iteration's wall time, and the loop's
  deadline is the earlier of the domain's and the instant the view's
  words next change (5.12).
- **Each iteration:** the transport iterates; its loaded records start
  the client (3.4); signing in, by the link or with the token DIRECTORY
  holds (section 7); the token written once signed in by a link; the
  transport's events into the domain; the domain's `fire`; the
  transport's requests carried; the view built; each whole line made
  into one command and done; a print, when one is due; the output
  written.

### 6.3 When it prints

As skein's text shell prints (skein's `ui-text.md`, section 4), at the
interval of 6.2. Before the first tree (signing in), only temper's own
lines, each starting `temper: `.

- **temper's lines wait for room.** The host says each of temper's
  lines only within the room skein's terminal gives (`Terminal::room`).
  A line with none waits in the host, in the order they came; a line
  equal to one already waiting adds none (a sign-in tried again, the
  small store refused again). The waiting queue is uncapped; terminal
  credit and the command hold below bound what waits, without an
  output-count reservation.
- **While a line waits, the person's commands wait.** The host takes no
  command from the terminal while one of temper's lines waits in it, so
  no act of the person's makes a new line; SIGINT and SIGTERM still
  close. What waits is then at most one of each line but a service's
  credential, and one credential for each request already sent, at most
  the request table's requests.
- **Notices are taken whatever the room.** The host takes each of its
  client's notices at every iteration, so a held stdout delays temper's
  lines and the person's commands, never the client: the sign-in's
  retries keep their backoff (6.7). Lines that wait when the terminal
  closes are not printed.

### 6.4 How it prints

As skein's text shell prints (skein's `ui-text.md`, section 5). The
first line is `== TITLE == STATUS`: TITLE the page's level-1 heading,
STATUS the link's state (5.1), with "Behind" after it while the page
is behind. With the repository sheet open, the underlying page is
Repositories and the dialog is named “Add repositories to PROJECT”. Its
source choices, repository checkboxes, fields and foot actions receive the
same numbered terminal controls as the other pages. Private, already in
the project and copying marks remain words; the terminal prints refusals
and request state beside the affected controls. The sheet has no Next
button: list demand and search read subsequent pages (4.4).


### 6.5 Numbers and commands

skein's text shell's (skein's `ui-text.md`, section 6). temper adds no
command.

- **A sign-in's address** is an external link (skein's `ui.md`, 6.2),
  which the terminal prints as `TEXT <URL>`, with no number (skein's
  `ui-text.md`, section 5), and never opens: it starts no browser and
  no program. The person opens the address in a browser on the
  engine's machine, where the provider's redirect reaches the engine
  (`../protocol/credentials.md`, section 3), or in one elsewhere, and
  pastes the address it ends at into the page's field, which the
  terminal draws as the browser does (3.5, `ConnectModel`, The paste);
  the page shows the sign-in's end when the `accounts` watch tells it. A browser opens the address
  in a new tab instead (3.5, `ConnectModel`).
- **A forge's connect or install link** is printed the same way, as
  `TEXT <URL>` with no number, and never opened; the terminal names no
  `back`, so the setup route ends on its own page. A connect link's
  address is on the clients' listener (`forge.md`, 9.1), for example
  `http://127.0.0.1:PORT/forges/1/github/manifest?state=STATE`; an
  install link's is GitHub's,
  `https://github.com/apps/SLUG/installations/new?state=STATE`, whose
  end GitHub sends back to the clients' listener. The person opens
  either in a browser that reaches the clients' listener, on the
  engine's machine in standalone, and the forges' page shows the
  connect's end when the engine tells it (`forge.md`, 5.4).
- **A key's field,** a secret, prints `(hidden)` once written, and the
  command `n KEY` writes it (skein's `ui-text.md`, sections 5 and 6);
  the terminal's line editing echoes the key as it is typed, which
  skein's text shell cannot turn off (skein's `ui-text.md`, section 9).
  `temper ctl account add-key` reads a key from a file instead
  (`tools.md`, 5.5, area 5).
- **A forge's pasted value** prints the same way: each of the forges'
  page's secret fields (3.6) prints `(hidden)` once written, and the
  terminal echoes it as it is typed. A PEM key spans lines, which one
  command cannot write, so from a terminal an App's key is given with
  `temper ctl`, which reads every pasted value from a private file and
  never from an argument (`tools.md`, 5.5, area 6).

### 6.6 The link, signing out and closing

- **The link's state** is the first line's (6.4). Its change changes
  the tree, so the page is printed again.
- **Signing out** is `Settings`' "Sign out" (3.5), or the banner's on
  a page that is no project's: the domain asks it, and
  the host gives `jig-client` `SignOut`, which sends jig's sign-out.
  Signed out: the host has the native shell keep none, which leaves
  DIRECTORY's token a zero-byte file, read as none; the main says
  "Signed out." (5.1), the first line reads `== temper == Signed out`
  (skein's `ui-text.md`, section 5: TITLE is "the shell's name when
  there is none"), and temper prints `temper: signed out; to sign in
  again, run temper open DIRECTORY LINK with a link temper serve
  printed`. A sign-out that did not reach the engine leaves the person
  signed in (jig's
  `client/domain/client.md`, section 12): the rail or the banner says
  "Sign out did not reach the engine.", and "Sign out" is offered
  again.
- **Another party's kept work:** while the small store holds requests
  of a party other than the one `me` confirms, temper prints `temper:
  work of another party is kept in DIRECTORY, and waits for them` once.
- **Out of date** (5.1): the first line reads `== temper == Out of
  date`, and the main says "This temper is out of date: run the temper
  the engine runs."; temper prints no line of its own. What is pending
  stays in DIRECTORY's small store, closed as below.
- **Kept work dropped** (3.4): when `jig-client` drops kept requests at
  start, temper prints `temper: N requests kept in DIRECTORY were not
  sent: another version of temper made them` once, N the count it told.
- **A service's credential,** when an answer carries one (jig's
  `protocol/client.md`, section 8), is printed once: `temper:
  credential for service N, shown once: CREDENTIAL`, the credential in
  base64url. The client keeps no copy of it.
- **Closing:** `q`, the end of stdin, SIGINT or SIGTERM. The client does
  not sign out. skein's text shell closes (`temper: closed`); no new
  exchange starts; `jig-client-shell-native` finishes the small store's
  last write, then closes its connections and files; the process exits
  0. What is pending stays in DIRECTORY's small store, and the next
  `temper open` on the same DIRECTORY sends it with its keys, within
  the engine's retention.

### 6.7 When the engine is unreachable

- **While signing in by the link,** an exchange with no answer, or
  answered not ready (the engine stopping), prints `temper: signing in:
  no answer from the engine; trying again`; one answered busy prints
  `temper: signing in: the engine is busy; trying again`. Either is
  made again, the same link with the same held credential, after a
  backoff of 250 ms, doubling to 8 s. `q` gives up and exits 0. A
  link answered refused, used, expired or unknown exits 1 with the same
  refusal words (6.1). Temper prints a fresh link when none is live or
  reserved (jig's `protocol/client.md`, section 8).
- **The engine's sign-ins full** exits 1 with `temper open: all N
  sign-ins are in use; the earliest ends at T; this link stays usable
  until U; after that, temper serve prints a fresh one when it is tried`
  (6.1), N the deployment's room, T the earliest saved expiry among its
  sign-ins and U the link's own, in universal time to the second,
  `YYYY-MM-DDTHH:MM:SSZ`. Nothing is written to DIRECTORY, and the link
  is not spent: the same `temper open DIRECTORY LINK`, once a sign-in
  has ended and before U, signs in. Full waits for a sign-in to end,
  hours away, so the terminal does not wait for it.
- **Once signed in,** the link is `jig-client`'s to judge (jig's
  `client/domain/client.md`, 8.2). The first line says its state, and
  the page is printed again at each change. Requests made meanwhile are
  kept in the small store and shown as parked; nothing the person did
  is lost, and `q` keeps it in DIRECTORY.
- **A small store that cannot be written** stops that step's requests,
  whose cards say they were not sent (`NotStored`), and prints `temper:
  cannot write the small store in DIRECTORY`.

### 6.8 Pure where it decides

The renderer and the command reader are skein's text shell's, step code
under the subset (skein's `ui-text.md`, section 3). The host adds the
terminal's operations (`Terminal::operations`) to its ring's, beside
its client's and its own; it copies none of the terminal's count. The
terminal starts from temper's view limits, as `temper open` makes it,
in a unit test (section 10). What temper decides is in its domain and
its view, both step code; golden text fixes how temper's pages print
(section 10).

### 6.9 While a run's text streams

The terminal is chiefly the real loop's test surface (`../domain/testing.md`,
2.4): people use the browser, or the TUI (6.11). While a
run's text streams, the terminal reprints the whole page at each print
interval (6.3), as skein's text shell does, with the chat's provisional
turn as the page draws it (section 3.5); the print interval is the only
pace it keeps. No rule of skein's text shell is added for streaming.
A command typed while text streams is done as at any time (6.5).

### 6.10 A page's address

The terminal opens at the inbox (3.4) and reaches each page by its
numbered link (6.5), never by a page's address: `temper open` takes the
engine's address or the sign-in link (6.1). A browser opens a page by
its address: a reload, or `/tasks/N` typed or opened, is a new client
at that address (`browser.md`, 2.3), and an inbox's address holds its
filter, so that `/?kind=question` opens the inbox filtered. Of that
open, the path and its load are the browser's; the rest is the client
root's, the same under both shells: a root started at a page's address,
its pages opening that page, asking its reads and drawing it once they
answer. A task address shows its reading status before the document, and
the page asks again after a transient end, as in 3.5. The client's joint
world, `tests/client`, starts its clients so, with no shell
(`fixtures::task_page`), and proves the root's part:
in rung B2's story (section 10) a fresh client started at a delegate's address
draws its heading and its "Task brief".

### 6.11 The TUI: `temper [ADDRESS]`

The TUI (`../draft/tui.md`) is a person's full-screen terminal client:
temper's client root, a view of its own and skein's full-screen shell
(skein's `ui-screen.md`), over the native transport `temper open` uses.
Its first pass (`../draft/tui.md`, section 4) signs in by the box's
printed link, pasted, and draws a first screen that proves the
connection, live (6.14). `temper open` stays the tests' line client
(6.9).

- **Bare `temper`** runs the TUI when stdin and stdout are both
  terminals (skein-shell's `terminal_modes` answers for descriptors 0
  and 1), at the address its directory keeps (6.12), else at
  `http://127.0.0.1:8367`, the clients' port `temper init` prefers
  (`../../help/reference/commands.md`). Otherwise it prints the line it
  printed before the TUI, `temper: no command; temper --help lists
  them`, and exits 1.
- **`temper ADDRESS`** names the engine's clients' listener,
  `http://HOST:PORT`, with or without a final `/`, HOST an IP address on
  loopback, read as `temper open` reads an address (6.1). A box whose
  port fell back to another is named by its address.
- **The link is never an argument,** as `temper ctl`'s isn't
  (`tools.md`, 8.1): the TUI takes it pasted on its sign-in screen
  (6.14), and refuses it as ADDRESS.
- **One argument at most,** an address: a first word that is no
  address and no command is no command, as before
  (`temper: no command WORD; temper --help lists them`).
- **Exit status:** 0 once the TUI closed (6.16); 1 when it cannot
  start, with one line on stderr:

  | Cause | stderr |
  |---|---|
  | ADDRESS given, stdin or stdout not a terminal | `temper: the TUI needs a terminal on stdin and stdout` |
  | ADDRESS not an `http://` address, or HOST not an IP address and port | `temper: not an address: ARG` |
  | ADDRESS a sign-in link | `temper: a sign-in link is never an argument: run temper http://HOST:PORT and paste it` |
  | HOST not on loopback | `temper: the engine's address must be on loopback` |
  | two arguments or more | `temper: invalid arguments` and the usage, `usage: temper [ADDRESS]` |
  | the directory unresolved or refused (6.12) | `temper: DIRECTORY: ` and what was refused |

  A panic aborts the process (fail-stop), the terminal left as it was
  found (6.16).

### 6.12 Its directory

- **Where:** `$XDG_STATE_HOME/temper/tui` where `XDG_STATE_HOME` is an
  absolute path, else `$HOME/.local/state/temper/tui` (skein-shell's
  `Base::State` for the tool `temper`, then `tui`); with neither, the
  start is refused. It is the TUI's alone, never `temper open`'s or
  `temper ctl`'s, and no argument names it.
- **Private,** as `temper open`'s (6.1): the binary makes it and its
  missing parents `0o700` and refuses it unless it is a directory of
  this user's with no group or other bits (skein's `io.md`, 5.3), then
  opens its parent as a root and gives `tui` to
  `jig-client-shell-native`, as `temper open` gives DIRECTORY's last
  component.
- **What it holds:**
  - `token`, the bearer credential, and `store`, the small store, both
    the native shell's files, kept as section 7 says;
  - `address`, the address the TUI last signed in to, `http://HOST:PORT`
    and nothing else, `0o600`. The binary reads it once at start, at
    most 64 bytes; one absent, empty, unreadable or not an address
    reads as none, and the default holds. Once the TUI has closed, if it
    signed in at an address the file did not hold, the binary writes
    the file whole through skein's io (skein-shell's `store_file`, a
    synced temporary replacing it).
- **One sign-in at a time:** the token is the engine's it was made at.
  Started at another address, the TUI offers it there, the engine
  there refuses it, and the TUI erases it and asks for that engine's
  link (6.14).

### 6.13 Its host and the seam

- **One seam,** shaped as the browser's host's (`journeys.md`, section
  2): `temper-client-shell-tui`'s `host::Host`. What reaches it is a
  report, what it asks back is its duties, and it lays the frame. The
  loop (6.16), the client's joint world (section 10) and the TUI's
  recorder and replay (`../draft/tui.md`, section 5) all call it, and
  nothing else of it.
- **It holds** temper's client root, `temper-client-domain`'s `Domain`,
  linked, as `temper open`'s native client holds it (6.2); the TUI's
  state and view (6.14); the focus; and the frame. It holds no io, no
  file, no clock and no randomness of its own: what it does is a
  function of its reports, its clocks and its seed.

  ```rust
  pub struct Start {
      pub size: skein_ui_screen::Size,
      pub scheme: temper_client_tui::Scheme,          // 6.15
      pub address: Box<[u8]>,                         // HOST:PORT
      pub kept: Vec<jig_client::Record>,              // the small store's records
      pub credential: Option<skein_lib::Secret>,      // DIRECTORY/token's
  }
  pub enum Report { Start(Start), Due, Key(skein_ui_screen_terminal::Key), Resize(skein_ui_screen::Size),
      Heard(jig_client::Event), Kept { written: bool } }
  pub struct Duties {
      pub asked: Vec<jig_client::Request>,            // to the native shell, in order
      pub keep: Option<Option<skein_lib::Secret>>,    // the credential to write; Some(None) erases it
      pub address: Option<Box<[u8]>>,                 // the address signed in to (6.12)
      pub open: Option<Box<[u8]>>,                    // an address for a browser
      pub close: bool,
  }
  impl Host {
      pub fn new(settings: &Settings, seed: u64, now: Time, wall: Wall) -> Host;
      pub fn clocks(&mut self, now: Time, wall: Wall);
      pub fn report(&mut self, report: Report) -> Duties;
      pub fn frame(&mut self) -> &skein_ui_screen::Frame;
      pub fn tree(&self) -> &skein_ui::Tree;
      pub fn event_of(&self, key: &skein_ui_screen_terminal::Key) -> Option<temper_client_tui::Event>;
      pub fn deadline(&self) -> Option<Time>;
  }
  ```

- **The reports:** `Start`, once and first, with the window's size, the
  scheme, the address, and the small store's records and the
  credential as the native shell loaded them; `Due`, the root's
  deadline reached; `Key`, one key as skein's decoder made it (skein's
  `ui-screen.md`, 6.1); `Resize`, the window's new size; `Heard`, one of
  the transport's or the small store's events, as the native shell gives
  them; `Kept`, the credential's write ended, written or not. Any report
  before `Start` is the host's defect.
- **The duties** of each report: the root's requests for the native
  shell, in order, each routed as `temper open` routes them (6.2); the
  credential to keep, or none to erase it; the address signed in to,
  once, the first time the link is live with a party; an address for a
  browser to open, which nothing asks in this pass; and closing, once.
  After closing, every report but `Heard` and `Kept` does nothing, so
  the native shell's last write still lands.
- **The credential:** after each report the host reads it from
  `jig-client`'s `link()`, through the root. One that differs from the
  one kept is a duty to keep it, before the first screen is drawn on it;
  none, while the link is signed out by any means (a sign-out
  elsewhere, a sign-in ended, a held credential the engine does not
  know), is a duty to erase it. A sign-in kept by the engine leaves the
  credential as it was, and writes nothing (section 7).
- **A key** goes through skein's key reader (skein's `ui-screen.md`,
  6.2) on the tree and the focus the last frame laid, so that a key
  acts on what the person saw: an act on a node becomes the TUI's event
  through its view, if the view names one (6.14); the focus moved draws
  again; Ctrl+C closes. `Host::event_of` gives the event a key would
  make, without giving it.
- **The frame:** `Host::frame` lays the view's tree with skein's
  renderer at the window's size, in the start's theme, with the host's
  focus (skein's `ui-screen.md`, section 5). After each report the
  host has already built its view from the root and the TUI's state,
  so `Host::tree` is the tree the next key reads.
- **Shaped for the recording now:** a recording is the start's facts,
  then turn by turn the clocks, each report with its duties, each key
  with its event, and a mark for each frame the person saw; a replay
  gives the same reports to a fresh host and finds the same duties and
  frames. The browser to open is a duty now, so that the browser's
  approval (`../draft/tui.md`, 4.2) and the recording add no field to
  the seam.

### 6.14 Its pages

Two screens, each a tree of the TUI's view, built from the root's link
and `me` and the TUI's own state: signing in, while the link is signed
out, and the first screen once signed in. Each has a framed top, the
region `tui-top`, titled "temper", and a bottom row, the region
`tui-bottom`, spread: what closes the TUI, and the link's state. Their
frames are illustrative: their words and look are refined on the
recording (`../draft/tui.md`, section 5).

- **The TUI's state,** `temper-client-tui`'s `Tui`, holds what the root
  does not: the address, the field's value, the sign-in's progress
  (asked, answered, lost, or the TUI's own refusal of what was pasted),
  whether the link was signed out after a sign-in, and whether the
  credential's last write failed. The root stays the one source of the
  link's state and of `me`.
- **Its words:** where the TUI says what `temper-client-view` says, it
  says the view's phrases (`temper-client-view`'s `words`): the link's
  states, "Live", "Connecting", "Offline", "Signed out" and "Out of
  date" (5.1); a link used, expired or not this temper's, and the
  sign-ins full; "No project yet"; "This temper is out of date: run
  the temper the engine runs."; "Signed out.". Its other words are its
  own phrases, under an owner of its own (skein's `ui.md`, section 9).
- **The link's state** is the bottom row's last child, a status in the
  view's words after the TUI's own `●`: "Live" once `me` is read on a
  live link, "Connecting" while it starts or before `me`, "Offline",
  "Signed out", "Out of date", each in its token's paint (6.15).

**Signing in**, illustrative:

```
╭─ temper ─────────────────────────────────────╮
│ Sign in this terminal to 127.0.0.1:8367      │
│ Paste the link temper printed, then Enter.   │
│ Link: ••••••••••••••••••••••••••             │
│ Signing in…                                  │
╰──────────────────────────────────────────────╯

 Ctrl+C to quit                     ● Signed out
```

- **The paste:** one text field, "Link", a secret, drawn masked (skein's
  `ui-screen.md`, 5.4), that submits on Enter, its placeholder the
  link's shape at the TUI's address, and the focus asked for it as the
  screen opens. A paste fills it whole (skein's `ui-screen.md`, 6.2). A
  `q` types in it, so the bottom row says "Ctrl+C to quit".
- **Enter** checks the field with jig's parser of the printed link
  (`jig-client-protocol-http`'s `printed`, under the default mapping's
  prefix `/jig`; section 7), and that its `HOST:PORT` is the TUI's
  address:
  - not a link: "That is not a sign-in link: paste the whole link
    temper printed.";
  - a link for another address: "That link is for HOST:PORT. To sign
    in there, run temper http://HOST:PORT."; nothing is sent;
  - a link for this address: the host asks the root `SignIn { with:
    Link { secret, held } }`, held the credential the directory holds,
    as `temper open` does (section 7), and the field is emptied; the
    secret stays in the TUI's state only until then.
- **While it signs in:** "Signing in…". `jig-client` sends the link
  again by itself, after a backoff, when the engine answers not ready
  or busy, or no answer comes (jig's `client/domain/client.md`, section
  12), and the TUI says which: "Signing in: the engine is busy; trying
  again.", "Signing in: the engine is starting or stopping; trying
  again.", "Signing in: no answer from the engine; trying again.".
- **Refused:** a link used, expired or not this temper's, or the
  sign-ins full, in the view's words (5.1); a link refused otherwise,
  "This link was refused. Use the fresh one temper printed.". The field
  waits for another.
- **Room for the browser's approval:** the paste is the last way of
  signing in on the screen, under its own lines, so that the approval
  (`../draft/tui.md`, 4.2) is laid above it.
- **Signed out elsewhere:** the link signed out after a sign-in, by any
  means (a sign-out from another client, the sign-in ended or expired,
  a held credential the engine does not know): the token is erased
  (6.13), and the screen shows again under the view's "Signed out.".

**The first screen**, illustrative (`../draft/tui.md`, 4.3):

```
╭─ temper ─────────────────────────────────────╮
│ Connected to 127.0.0.1:8367 as Owner         │
│ Projects: Hello · Demo                       │
╰──────────────────────────────────────────────╯

 q to quit                                ● Live
```

- **What it reads:** `me` (3.1), its party's name and its projects'
  names, through `root.pages().me(root.client())`, and the link's state,
  live: each change is drawn at the next frame. With no project, the
  view's "No project yet".
- **Before `me`:** "Connecting to HOST:PORT", the state "Connecting".
- **The engine stopped and started again:** the state goes "Offline",
  then "Live" again; `me` stays drawn as last read, and nothing is lost,
  since `jig-client` keeps the watches and opens them again (3.3).
- **Out of date:** the view's "This temper is out of date: run the
  temper the engine runs." in place of the lines, the state "Out of
  date".
- **q to quit:** a button whose shortcut is `q`, no modifier (skein's
  `ui.md`, 6.1); its press closes the TUI (6.16).
- **A sign-in not kept:** when the credential's write failed, a line
  "This sign-in could not be kept: the next temper asks for a link
  again.".

### 6.15 Its theme

- **Two themes,** dark and light, `skein_ui_screen::Theme`s that
  `temper-client-tui` gives by its `Scheme` (`Dark`, `Light`), whole
  (skein's `ui-screen.md`, section 3), from the browser's chat terminal
  (`browser.md`, 2.5, *A chat's terminal*) and the palette its
  stylesheet, `temper.css`, holds there:

  | Colour | Dark | Light | What it paints |
  |---|---|---|---|
  | bg | `#191816` | `#fffefd` | the base's background |
  | ink | `#e8e3da` | `#34312c` | the base's text |
  | dim | `#9a9388` | `#7a736a` | the hints, "Connecting", the sign-in's progress |
  | faint | `#6a655c` | `#b5aea3` | the border, the placeholder |
  | sel | `#2a2825` | `#f4f0e9` | the focus's background |
  | accent | `#e08a68` | `#b8664e` | the title, "Offline", "Signed out", "Out of date", a refusal |
  | good | `#8fc28f` | `#3f7d4e` | "Live" |

  The terminal's `--line` is left out: a border's glyphs in it would
  barely show.
- **Its regions,** placed by name (skein's `ui-screen.md`, 5.2):
  `tui-top` at the top, its children on lines, framed by `╭ ╮ ╰ ╯`, `─`
  and `│` in the faint colour, its first level-1 heading in the border
  as its title, one column of padding; `tui-bottom` at the bottom,
  spread along one row, one column of padding. Nothing else is placed;
  the body is empty in this pass.
- **Its tokens:** `tui-title` (accent, bold), `tui-hint` (dim),
  `tui-live` (good), `tui-waiting` (dim), `tui-down` (accent) and
  `tui-refused` (accent), under the TUI's owner, its names' prefix
  `tui` (skein's `ui.md`, section 8).
- **Its glyphs:** `❯` the focus mark, `•` the mask; the focus's paint
  the selection's background.
- **Chosen by the binary** from the environment: light where
  `COLORFGBG`'s last field, the background the terminal declares, is 7
  or 15, else dark; the colours' depth 24-bit where `COLORTERM` is
  `truecolor` or `24bit`, else the 256 indexed ones (skein's
  `ui-screen.md`, section 7).

### 6.16 Its loop and closing

- **One host on skein's loop** (skein-shell's `drive`),
  `temper-client-shell-tui`'s `run::Run`: the host (6.13);
  `jig-client-shell-native`'s `Shell`, its transport and its files, as
  `temper open`'s; skein's full-screen terminal on stdin and stdout
  (skein's `ui-screen.md`, section 8); and SIGINT, SIGTERM and SIGWINCH
  from skein-shell's `open_signals` with `Watch::TerminationAndWindow`,
  adopted by an io of its own. A completion goes by its tag: the
  terminal's (`Terminal::owns`), the signals' (bit 61), else the native
  shell's.
- **Its settings,** `temper-client-shell-tui`'s `limits`: the client's
  and its connections' from `temper-client-shell-terminal`'s
  `limits::native()`; the TUI's tree's from `temper-client-tui`'s
  `limits`; a frame of at most 200 rows by 500 columns; a paste of at
  most 512 bytes, the link's field's bound; a frame drawn at most every
  20 ms.
- **Each iteration:** the native shell iterates; once it has loaded the
  token and the small store, `Start`, at the terminal's size; then the
  credential's write ended, the transport's events, the window's size
  when it changed, the keys, and the root's deadline when it is due,
  each a report whose duties are carried at once: the requests to the
  native shell, the credential kept or erased, the address noted for
  the binary, closing. Once anything was reported, the host's frame
  goes to the terminal, which draws the newest at its pace. No key is
  taken and no frame drawn before `Start`.
- **Time and the seed** are skein-shell's, as `temper open`'s (6.2).
- **Closing:** `q` on the first screen, Ctrl+C anywhere, the input's
  end, SIGINT or SIGTERM. The TUI doesn't sign out. The terminal
  finishes the write it began and writes its `LEAVE`; the native shell
  finishes the small store's last write, then closes its connections
  and files; the binary sets the terminal's modes back, writes the
  address (6.12) and exits 0. What is pending stays in the small store,
  as `temper open`'s does (6.6).
- **Its startup** is the binary's (skein's `ui-screen.md`, section 8):
  the input set raw with the interrupt keys passed as bytes
  (`Keys::Pass`), so that Ctrl+C is a key; then the restoration
  skein-shell's kept modes make, `Kept::on_abort(stdout, LEAVE)`, given
  to skein-fail-stop's hook before the loop and taken back after it,
  before the modes are set back, so that a panic leaves the terminal as
  it was found and its report reaches the person's own screen (skein's
  `programming-model.md`, section 2).
- **A directory the native shell's files refuse** after the binary
  checked it: the run closes, its `LEAVE` written, and the binary prints
  `temper: DIRECTORY: cannot open its files` and exits 1.
- **macOS** builds it with skein-shell's and skein-io's portable parts
  alone.

Its tests are section 10's; its later passes, section 11's.

## 7. Signing in and the small store

How jig signs a client in, and how a native client keeps its credential
and its small store, are jig's (jig's `protocol/client.md`, sections 8,
10 and 12; jig's `client/domain/client.md`, sections 11 and 12).
temper's:

- **A sign-in's address in the small store** (3.5, `ConnectModel`, *A reload
  while it waits*): a browser's shell keeps, in one record of its own under
  the key `sign-in`, beside jig's, the latest answered sign-in's source, its
  address and whether its tab was refused, so that a reload draws the same
  page. The address is the provider's authorization address, with its state
  and challenge and no verifier, which the page's link and the tab's history
  show anyway. A later sign-in's record replaces it, and it goes with the tab.
  The record is read only for that source: another's is no sign-in of the page's.

- **The printed link** (jig's `protocol/client.md`, section 8): at
  start, `temper serve`, or `temper box` before it starts serve
  (`../protocol/setup.md`, 7.2), prints a sign-in link for the deployment's
  first owner (`../protocol/README.md`, section 4), in the default
  mapping's link route under temper's prefix,
  `http://HOST:PORT/jig/v1/sign-in/link?secret=SECRET`. `temper open
  DIRECTORY LINK` checks that the link names the engine's address on
  loopback and that route, and gives the secret to `jig-client` with the
  credential DIRECTORY holds, if any (`SignIn { with: Link { secret,
  held } }`). The secret is in the command line only until the sign-in
  spends it; the client root passes it on and keeps nothing of it.
- **A sign-in held is kept** (jig's `domain/people.md`, section 3): a
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
  shell keep none, which erases it. A later `temper open DIRECTORY
  ADDRESS` has the native shell read it before its loop (6.1), and the
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
- **Loopback only** in standalone (`../protocol/configuration.md`,
  section 2; 6.1), so a bearer token never crosses a network in
  plaintext. Clients between hosts, over TLS, come after standalone.
- **One person.** The deployment's link signs in its first owner;
  signing in through the forge is open (section 11).

## 8. On the engine's side

The engine's service is `../protocol/README.md`'s (section 6). What it
does for the client:

- **The clients' listener** is `temper-service`'s: TCP on the loopback
  address temper's configuration names
  (`../protocol/configuration.md`, section 3), at a fixed port, never 0,
  which `temper init` chooses, since a GitHub App's redirect and setup
  addresses name it (`../protocol/configuration.md`, the clients'
  listener; `../protocol/setup.md`, 7.1). On
  it, it mounts jig's clients' component (`jig-protocol-engine-clients`)
  with jig's default HTTP mapping's engine end
  (`jig-protocol-engine-clients-http`) under the prefix `/jig` (jig's
  `protocol/service.md`, section 2). A request's path is offered first
  to jig's component, which says whether it claims it (jig's clients'
  listener claims `/jig/...`); then to temper's forge routes,
  `/forges/...` (the routes, below); then to the browser's page and
  bundle, the binary's own or the one the configuration names, or else,
  from a binary built without its own and a configuration naming none,
  to the page for no bundle (`browser.md`, 2.2 and 2.6). Every other
  path is answered not found.
- **The routes,** temper's beside jig's: a GitHub connect's browser
  steps, `GET /forges/N/github/manifest`, `redirect` and `setup`, N the
  forge's number, each the setup's host's run for the state its query
  names (`../protocol/forge.md`, the setup's host). Their answers, their
  headers, their `302`s and their refused pages are `forge.md`, 9.1's.
  The state is their only authenticator: 128 bits or more, good once,
  for its life (`setup.state_life_ms`), for that forge alone; they take no
  cookie and no credential of jig's. A query malformed or past
  `routes.query_bytes` is answered `400`, a method other than `GET`
  `405`, and a forge the store does not keep or not GitHub `404`. No client of
  temper's calls them: GitHub sends the person's browser there.
- **A client's leaving, on the listener.** `temper-service` reads each
  connection's events below skein's server. When the peer ends a
  connection before its call is finished, it aborts the connection, as
  it does one that fails, and the call's owner hears the connection
  closed: for a call under `/jig`, jig's engine end, which takes it as
  the call's end (jig's `protocol/client.md`, 17.4: "A peer's end is
  the call's end"). So a stream ends when the client closes it or its
  process ends, and its room among the profile's streams is free for
  the next call at once (`tools.md`, 8.2).
- **The printed link opened in a browser:** a `GET` of the link's route
  spends nothing, and the mapping leaves it to the application's page
  (jig's `protocol/browser.md`, section 11). With a bundle, temper's
  server answers it with the page, which signs the browser in by it
  (`browser.md`, 2.4); with none, the page for no bundle, which says
  so and names `temper open` (`browser.md`, 2.2), with
  `Referrer-Policy: no-referrer`, and the link is `temper open`'s alone
  (6.1).
- **jig's example guards, as they are** (jig's `protocol/browser.md`,
  12.2), the Host allow-list holding the listener's own address. The
  terminal sends its credential as a bearer token; the cookie is the
  browser's (`browser.md`).
- **jig's clients' component** (jig's `protocol/client.md`) serves jig's
  wire's versions and, for the family `FORGE`, `CLIENT_VERSION` alone
  (4.6). temper gives it no `Mint` in standalone: the printed link and
  services' credentials are jig's own ways of signing in (jig's
  `protocol/client.md`, section 8).
- **The forge's request, decoded before the root** (jig's
  `protocol/client.md`, section 11). An application request of the
  family `FORGE` at `CLIENT_VERSION` is decoded by
  `temper-client-protocol-wire`; one that does not decode, or of
  another family or version, is answered undecodable, as jig's
  reference service answers it. The routing fills the adoption's host
  from that forge's `git` base, as its record keeps it: its host, with
  `:port` when it names one, as the forge's resource paths have it
  (`../protocol/forge.md`, 2.1). It leaves the host empty for a forge
  number the store does not keep, and hands the root the party's
  keyed request: its reply, its sign-in, its key, the digest of the
  record's bytes, and the adoption, or another of the FORGE requests
  (below) with the number and digest of the value it carried.
- **The FORGE requests** beside `Adopt` (`forge.md`, 5.1 and 5.4), each
  keyed, each naming the deployment as its scope, `Dismiss` and `Looked`
  alone naming a project (jig's
  `protocol/client.md`, section 11). `forge` in a `Connect` or a
  `SetUp` is `Making` (4.6): a kept forge's number, or a forge to make,
  which the request's decision makes as a record of the forge
  connector, numbered next, before its run starts
  (`../protocol/forge.md`, 2.1 and 2.2):

  | Request | The value it carries | What the engine does | Its outcome |
  |---|---|---|---|
  | `Connect { forge, how: GitHubManifest { back? } }` | none | the setup's host's `Manifest` run, for the person's own account, always (`forge.md`, 9.1), ending `Done(Manifest { state, target, manifest })`, from which temper renders the manifest's page (`forge.md`, 9.1); the page and `back` held with the state | a link to the manifest's page for a fresh state (the routes, above): with `back`, a path on the page's own origin, `/forges/N/github/manifest?state=S`; without it, the terminal's whole address on the clients' listener |
  | `Connect { forge, how: GitHubApp { id, read_only?, key } }` | `key`, the App's private key, PEM as GitHub gives it; `id` rides in the request | `Paste`: the key decoded and matched to the App `id` names, the App's user found, its hook pointed at `https://SLUG.invalid/` unless `read_only` leaves it as read (no webhook secret on GitHub: changes are heard from the App's delivery log), its permissions and events checked | connecting |
  | `Connect { forge, how: ForgejoAdmin { first? } }` | the site admin's token, a bundle of one `AdminToken` item, at most 4 KiB in all | `Reconcile` with the paste, its `AdminOrigin` write kept, then `Create` of `first` when given; only after Done is `AdminToken` kept, then the forge connects; every end erases the carried value; never `TakeOver` (`../protocol/setup.md`, 5.5); a managed forge (`ForgeRow.admin`) offers the client's admin Details, a new one's `ManagedForge` naming its addresses, with `first`'s org, repository and runner label | connecting; then the watched row, its typed `ended_stop` on a stop, or `first` made, unadopted |
  | `Connect { forge, how: ForgejoTokens }` | the bot's API token and git token, and the hooks' secret when given: `ApiToken`, `GitToken` and `WebhookSecret` in one bundle | `Check { forge }`: both tokens answer as one user, not a site admin, and on a kept forge as its kept user; on Done the run's `User` write kept, then the tokens and the secret under the forge's numbers, then `Origin`, its login kept as the record's bot, and the forge connects; every end erases the carried value (`../protocol/setup.md`, section 4); `forge` a kept connected Forgejo, not connected or stopped, or a new one, `connected` naming its addresses | connecting; then the watched row, connected as its login, or its last connect's line on a stop |
  | `Install { forge, back? }` | none | `Install`, GitHub alone; `back` held with the state | a link, the install page for a fresh state |
  | `Accept { forge, installation }` | none | `Accept` with no state, GitHub alone (`../protocol/setup.md`, 17.4) | accepted; forge refused, not found |
  | `Replace { forge }` | a new App key, one `Pasted` item; or a new API token, git token or both, and the hooks' secret when it changes | `Paste` with the kept App id, or `Check` against the kept user: it must answer as the forge's user; each new record is kept over the old, then the forge connects with them, the calls its old access began ending `Forbidden` (`../protocol/forge.md`, 2.2) | connecting; then the watched row, connected, or its last connect's line on a stop, the forge as it was |
  | `Forget { forge }` | none | the admin's token erased; a managed forge becomes connected (`../protocol/credentials.md`) | forgotten |
  | `SetUp { forge }` | none | an owned forge, new, or kept and not set up, or whose set up failed, whose setup says `"start": "asked"`: the routing asks the box for the forge's group, `start FORGE` on its pipe (`../protocol/setup.md`, 7.1), refused "not under the box" without a pipe and "busy" while a group starts; at the box's `started`, its handover taken, then reconciled and connected (`../protocol/forge.md`, 2.2) | setting up |
  | `RemoveForge { forge }` | none | the forge ended in the request's decision, then removed whole by the kept acts that follow it: its live work cancelled, its box group stopped and its directory removed, its records erased (`../domain/forge.md`, 4.1); refused for the playground's forge, whose reset removes it | removed; remove refused, `ForgeRefusal` |
  | `MakeRepository { forge, org, name, from, into }` | none; its scope the project `into` names, or the deployment | the row kept in the request's decision; the setup then runs `Create` or `Copy` and the connector adopts into `into` (`../domain/forge.md`, 4.5; `../protocol/setup.md`, 6.9) | kept; make refused, `MakeRefusal`; then the watched row |
  | `SetAway { project, on, until, kinds }` | none; its scope the project | the party's `Away` row moved in the request's decision, and the subject `away` marked, a row that comes out the same writing nothing (`../domain/forge.md`, 5.8) | away; forge refused, too large |
  | `Overturn { project, number }` | none; its scope the project | the party's `Decided` row marked overturned in the request's decision, and the subject `decided` marked (`../domain/forge.md`, 5.9) | overturned; overturn refused, already or unknown |
  | `Dismiss { what }` | none | the party's `Dismissed` row saved in the request's decision, and the subject `dismissed` marked, a row held already writing nothing (`../domain/forge.md`, 5.5) | dismissed; forge refused, too large |
  | `Looked { project }` | none; its scope the project | the party's `Looked` row moved in the request's decision, the subject `status` marked, and, at a new sitting, the status agent's run asked (`../domain/forge.md`, 5.6 and 5.7) | looked; forge refused, too large |
  | `StartPlayground` | the scripted model's key, which the routing reads from its file at each start and carries as a pasted value | the playground's start (`../protocol/setup.md`, 8.4): the `Playground` row written starting when none, its fake forge's record made and the box asked for its group, then the scripted model's account added and its repository adopted into its project, each unless done; refused "not offered" while a real forge is kept | running, once all are done; start failed, naming why |
  | `EndPlayground` | none | the playground's reset (`../protocol/setup.md`, 8.4): the `Playground` row resetting in one decision, then its kept acts in order, the project deleted, the fake forge removed, the scripted account removed, the row erased | reset, once the row is erased; end refused, not offered, with no playground |
  | `Reachable { forge, installation?, page }` | none | on every kind, the setup's host's `Reachable { forge, installation?, page }` plan (skein's `forge-setup.md`, 4.8; `../domain/forge.md`, what the picker offers): on GitHub one accepted installation's repositories, `GET /installation/repositories` paged by number on a token minted for the page; on a connected Forgejo what the API token can push to; on a managed or owned Forgejo every repository the site admin can see by id ascending, read on the admin's token, which stays the setup's host's (`../protocol/forge.md`, the setup's host) | a page, `{ rows, more }`: at most `picker.page` rows on every forge, clipped to the forge's own page cap, each the repository's id, its owner's id and login, its name and whether it is private; `more` set when the page is full |

  `first` is a `FirstAsk { org, repository, label }`: the
  first repository's fields, as `SetUp`'s, with no project, and `label`, the runner label
  its starter workflow runs on (default `docker` in the client and ctl;
  1 to 64 of letters, digits, `.`, `_` and `-`).

  Names in a carried value's braces are its bundle's items
  (`../protocol/credentials.md`, pasted values), never fields of the
  record, which holds the forge, the kind of connect, and a GitHub App's
  id. `installation` is an accepted installation's id, GitHub alone: one
  the forge has not accepted, or one given for a Forgejo, stops the plan
  (`Installation`, `Missing` or `Unexpected`); `page` a page's
  number, from 1, and the picker reads the next page when its list nears
  the end or a search still needs rows; a full last page may be followed
  by an empty answer. No Next or First page control is drawn. The plan’s
  `reachable` limit is given from `picker.page`. The request names no
  project (section 11).
  The forge's kind, mode, user and installations are the deployment
  part's, never the page's; an installation's page there is built by the
  engine from the forge's `web` address, as its record keeps it (`forge.md`, 5.5).
- **A carried value, as jig carries it** (jig's `protocol/client.md`,
  section 5). jig's clients' component strips the value at decode and
  keeps it in the secrets' store under temper's tag, at a number it
  gives it from 2^63 up (`../protocol/credentials.md`, pasted values),
  before the core sees the request; the core gets the number and the
  value's digest, and the value never reaches the core, a document or a
  log. The request names no number and the client draws none. A value
  is at most 4 KiB, jig's bound on a carried value. A carried value the
  core leaves unused, its request refused, replayed, joined to another
  or cut by a stop, is named in one output of the core's, which temper's
  root forwards to jig's clients' component to erase.
- **`back`,** on a GitHub connect and an install, is the page's address
  the setup route sends the browser back to once GitHub is done
  (`forge.md`, 9.1). The service checks it at decode, before the root:
  one leading `/`, not `//`, no `\`, no scheme, of visible ASCII, at most
  512 bytes; one that is not is answered undecodable. None, from the
  terminal, leaves the route's own page.
- **Who may:** a person whose role on project 0 is `Owner`; in
  standalone, the deployment's owner, for every request above. jig admits each by key and
  refuses at admission as for any request, in jig's words; the root
  refuses any other party's, before a forge is called or a run starts,
  as the outcome refused (role).
- **The root runs it** (`../domain/root.md`; `../protocol/forge.md`,
  the setup's host): it keeps, in memory, a connect's or a replacement's number and
  digest in the connect's own record, refuses it or asks the routing to
  run its plan, and answers. A connect's end, connected or why not, is
  the forges' page's to read (`forge.md`, 5.4), never the answer's: a
  run's failure, "the pasted value changed", a GitHub App lacking
  permissions or events, each named (the setup's host's `Lacking`;
  `forge.md`, 5.5), are said there; a connect a restart cut is said
  nowhere until a connect is kept (`../protocol/credentials.md`, 2.2 and
  section 8). The outcomes:
  - *connecting:* the run started; a `Manifest` or an `Install` that
    stops, other than busy, answers it too, and its failure is said on
    the forge's row as the last connect's end, never in the answer;
  - *a link:* the address to open, printed and never opened (6.5);
  - *forgotten:* the admin's token erased;
  - *accepted:* an installation kept by `Accept`, with its id; an
    `Accept` that stops answers *forge refused* with its reason (4.6);
  - *a page:* the picker's (4.4);
  - *refused:* `Connect`'s, `Install`'s and `Accept`'s as `ForgeRefusal` (4.6) has
    them: the role; no such forge; the forge's kind or mode does not
    take it (a Forgejo's install, a forget on a forge not managed, a
    replacement on a managed or owned forge); busy, since a forge runs
    one writing plan, a connect, an install or a reconcile, and one
    `Reachable` beside it, and a second of either kind is refused, as is
    one past `setup.runs`, which counts both; or, for `Reachable`, the
    forge's reason below, an installation the forge has not accepted or
    one given for a Forgejo (the plan's `Installation` stop), and on a
    managed or owned Forgejo the admin's token's, missing (`Missing`) or
    not a site admin's (`NotAdmin`), as the plan's first step has them
    (skein's `forge-setup.md`, 4.3 and 4.8).
- **The root routes an adoption** (`../domain/forge.md`, section 4;
  `../domain/root.md`, 5.4): jig's people child admits it by key, and
  refuses at admission as jig's answer; admitted, it goes to the forge
  connector, which refuses a forge it does not keep (no such forge) or not
  connected (not connected), locates the repository by owner and name
  (not found when the forge has none), and makes jig's adoption request
  on the party's behalf
  (`adopt_repository_ask`), which the core judges as the party's own.
  The forge connector's adoption then reads what temper may do there.
  The outcome is kept under the request's key for the retention and
  answered once durable (jig's `client/domain/contract.md`, 3.3). The
  forge keeps at most `forge.applications` (32) such outcomes and the
  requests in flight beside them; one admitted while they are full is
  refused at once, jig's `Limit` of kind `ApplicationAnswers` with that
  bound, which jig keeps under the key (jig's `domain/people.md`, its
  application requests), never answered busy until the retention ends.
  The key keeps that refusal for the retention, so ctl says it as
  "temper keeps at most N answers to requests like this within a day:
  ask again later, under a new key", N the bound.
- **The forge's errors as reasons,** by the forge connector
  (`temper-engine-domain-forge`'s `reason`, from `skein-forge`'s
  `Error`, skein's `forge.md`, section 3.4): `NotConnected` is not
  connected; `NotInstalled` is not installed; `Missing` is not granted
  on a repository whose account has an installation of selected
  repositories, with that installation's page, and not found
  otherwise; `Forbidden` is no access; `Refused` is refused; `Busy` is
  in flight; `TooLarge` and `Full` are too large; `Unavailable`,
  `Timeout` and `RateLimited` are unreachable; any other is an invalid
  answer. A `Reachable` whose plan stops on the forge's error carries the
  same reasons.
- **The outcome, encoded** by `temper-client-protocol-wire`, goes back
  as the family's answer to jig's clients' component, an adoption's and
  every other FORGE request's alike, kept under the request's key for
  the retention, as an adoption's is, at the store's tag 15
  (`../protocol/store.md`), a refusal among them. An outcome that
  carries a link or a page is kept in memory alone, until its state's life ends
  (`setup.state_life_ms` after `Done`): the same key sent again while it
  lives answers the same link and starts no run; after its end, or a
  restart, the key is unknown, as one past the retention is. A
  `SetUp`'s outcome is kept in memory alone too, as the ask it answers
  is (`../protocol/forge.md`, 2.8): the forge connector holds the
  latest per kept forge, and one more for a number it does not keep,
  under its request's token, and the same key sent
  again while it is held answers the same outcome and asks the box
  nothing; once a later `SetUp` of that forge replaces it, or after a
  restart, the key answers unknown.
- **A forge's set up, answered** (`../protocol/forge.md`, 2.8): the
  root admits `SetUp` by key, of the deployment's scope, and the forge
  connector by the forge's connection; the routing's box link then
  answers it, the connector's own refusal among what it answers: the
  box asked, *setting up*, or refused, *not under the box* or *busy*,
  or the connector's *not offered*. The answer is the request's
  outcome, and the set up's course, its steps, its `started` and its
  `failed`, is the forge's row's (`forge.md`, 5.4), never the answer's.
- **The deployment's forges, watched** (4.6; `forge.md`, 5.4): the
  routing hands the root the watch of `forges`, with the deployment's
  scope; the core admits it on the party's deployment role and asks the
  forge for the document, and again each time the forge marks it: at
  every connection the routing tells it and at the box's close
  (`../domain/forge.md`, 4.1). Every field of a row is the forge
  domain's: the routing encodes the document as it is, adding nothing.
- **The forge's reads and watches** are decoded by
  `temper-client-protocol-wire` the same way and go to the root
  (`../domain/root.md`, 5.3; `forge.md`, section 9). One the wire has
  no record for does not decode: the routing answers the read with no
  record (unknown) and refuses the watch as unknown, without the root,
  as jig's reference service answers an undecodable one.
- **A change's panel** (4.6; `forge.md`, 9.2): the routing hands the
  root the watch, with its scope and the change's number; the core
  admits it on the party's role in the project and asks the forge for
  the document (`../domain/root.md`, 5.3), and again each time the
  forge marks its subject. Each document leaves held in the decision
  that made it, released once that decision's commit is durable, and
  the routing encodes it, building the pull request's address from
  that forge's `web` and its kind (`forge.md`, 9.2).
- **The playground's read** (4.6, 4.8) is answered by the root's forge
  connector, to any signed-in party, from the `Playground` row
  (`../protocol/setup.md`, 8.4): none with no row, and otherwise its
  forge, its project once made, its model's LLM provider, its
  repository's owner and name, its state, starting, running or
  resetting, and its `llm.asked` cues in order.
- **The playground's start and reset, answered**
  (`../protocol/setup.md`, 8.4): the root admits `StartPlayground` and
  `EndPlayground` by key, of the deployment's scope, as `SetUp`; the
  forge connector answers a start once its acts are done, or at the
  first that fails, and a reset once its last act has erased the row,
  and holds each answer in memory alone, as `SetUp`'s: the latest
  one's, under its request's token, the same key sent again while it is
  held answering the same outcome; after a restart, the key answers
  unknown. A start or a reset while another of its kind is under way
  takes that one's answer. The course is the read's, the forge's row's
  and the `accounts` document's; the answer says only how it ended.
- **A forge's removal, answered** (`../domain/forge.md`, 4.1): the
  root admits `RemoveForge` by key, of the deployment's scope, and the
  forge connector answers it in the decision that ends the forge,
  removed, or refused; the removal's kept acts go on after the answer,
  resumed at each start until the forge is gone. The forge leaves the
  `forges` watch's rows in that decision.

## 10. Testing

The tiers, the fakes and the budgets are `../domain/testing.md`'s (sections 2,
4 and 6.1); the client's joint world is its 2.1 and 6.3. The client's
own tests:

- **How work runs and a proposal of settings** (5.13): step tests of each control's request, refusal and stale; the rail and tab bar under each value;
  the card from jig's documents; a world test that accepts a proposal of kind 7 and sees `me`'s project show it; and the gallery's and goldens' pages
  (`../domain/testing.md`, 6.4).
- **Credential-boundary consumption** is checked in the existing
  domain and composed-client fixtures, including the terminal shell's
  startup `Held` sign-in after its children have asked reads (3.4).
  Every cancelled read is consumed visibly and its asker must ask again;
  no new retry policy is implied. The preview's existing view tests
  render jig's supplied `resolved_default`, with catalogue and charter
  orders opposed, explicit choice and no preview, preserving omitted
  request intent (5.5).
- **Step tests:** the domain's pages (what each watches and reads as it
  opens and closes, each act as the operations it sends, the
  confirmation's rule of 3.6, routing by asker; a chat's delegations,
  5.7: a call's tasks by its objects and by its acknowledgement, a
  failed call's and another tool's none, the newest watched up to
  `reads`, each unwatched as it leaves, and a card's facts from a
  task's document in each state; a change's and an agent's task's: a
  task's state from what it waits on and from its chip, in each waiting
  shape, running with and without an observation, and in each ending,
  and a dependency's from each kind of link; a goal's, 5.8: its report and
  its line read and read again when the document moves, its line past a
  page and not found, its counts from each, its inbox filtered by it and
  answering through its cards, `draft_of`; an agent's task's, 5.9: its
  shape from its executor, priority and label, its thread as a chat's
  (opened, a page, a document's change, a run's text and its end), and
  its run's heading in each phase, with an attempt and none; a
  goal's and a procedure's task's, 5.8 and 5.10: its ended delegates
  read as it opens, by its parent and the ended phase, read again on
  its tree's or its document's change, one read out at a time, one
  asked while another is out read once that answers, and the read's
  page cut; an agent's task with no priority reading none; the panel's
  slot in each state; each step's slot and the steps done; a task's
  page's acts, 5.11: a goal's Pause and Resume sent at once, naming the hold's
  revision, its Close's reason and each toast; an agent's task's Pause and Resume the same;
  a chat's Pause and Resume sent at once, its status from whose hold it is, and its message
  while paused one compound of the words and the release, whole or refused;
  Message, Stop, Cancel and Release each as its
  request, Cancel only once confirmed, one request per kind
  at a time, an act not offered, proposable or refused sending
  nothing, a card read for its own task once held, and the log's earlier
  and later acts as the chat's reads (5.15)); the forge side's steps
  (the form's rules, its request, every outcome, the version check; the
  picker’s pages by number, scroll and search demanding the next page,
  a full last page followed by an empty answer, its ids kept and dropped,
  a page answered late dropped, busy
  and the admin's token's refusals in words, "Not listed"'s words from
  each forge's read and installations; each FORGE request
  and outcome of section 8, a carried value's fields emptied as it
  leaves and asked again when lost, under the first key; a change's
  panel watched as a change's task page opens and closed as it leaves,
  in the task's project's scope, each delivery decoded and kept whole,
  a document of another version or one that does not decode ending the
  watch as unknown, and unknown and ended kept apart from loading);
  the view (each page from fixture states, read by skein's fake
  person's queries; a delegation's card in each state, in place of its
  call's summaries and its heard result; a goal's page, 5.8, `tests/goal.rs`:
  every state's header, each fallback of its heading, Landing's words through
  nine and past, each task line's state, the bar's name and classes, a landed
  fold, the bands by kind with choices and with none, the report's parts and a
  report-less end, the composer's offers, and the chat's activity unchanged; an agent's task's page, 5.9: its parts in
  order, its lineage bar with a parent's chip, a requester alone and
  neither, its brief with dependencies and none, its run's heading in
  each phase with a start and none, its conversation as a chat's, its
  result in each state and none before it ends, and its plan only with
  a delegate or a card; a task's page's acts, 5.11: each act by its
  admission (a button, proposable, refused, and not offered: not
  drawn), a goal's acts in order, Pause and Resume sent at once, Close's dialog,
  the taskbar's acts, gone while the confirmation is open, the composer with its
  note and a refused Message in its place, the Close dialog's words, its own held
  card where 5.11 puts it, and each outcome: pending, done, refused, moved, a
  goal's in the toast; the
  amendment's editor on a goal and on an agent's task, a cut spec read
  only, "No change", a moved draft with no "Send amendment", and its
  request's revision named or not as 3.7 says; a procedure's task's
  page, 5.10: its goal's link when it serves another task and none
  when it serves itself or none, and "Its tasks" over its plan where a
  person task's says "Plan"; its header with its panel shown and its
  part's words loading, unknown and ended; "View on forge ↗" with an
  address and none; each step at its slot with its line, the task it
  runs and none, a queue's place with the tasks ahead named, more and
  none; each hold's notice; CI and each verdict at its head, blocking
  and advisory, and no gate; its bounds; and its ended tasks among its
  live ones by number, "No live tasks." before the read, "None yet.",
  "None." and the more line; the sheet, 5.14: its three ends, its fields and what
  disables Start, each way and the default; the forge's words
  for every
  path shape and every kind, the form and its picker in each state,
  every outcome, the forges' page with a forge in each state of
  `forge.md`, 5.4);
  `temper-client-protocol-wire` (golden bytes, bounds and fuzz targets,
  as skein's `codec.md`, section 6 has them); and the engine's routing
  of the forge's family both ways, a change's panel's watch to the root
  with its scope, and its document encoded with the pull request's
  address on a GitHub forge, on a Forgejo and with no `web`.
- **No welcome** (3.5, The welcome), in the domain and the view: the
  deployment's read, its answer kept across a link's gap and a read
  that ends with no record; `Welcome` at each step moving on to the
  inbox, drawing temper's name alone before it; no playground offered;
  the chats page with no project; and, with `temper-service`'s real
  answer, one person signing in on a document with no welcome and
  finding the inbox.
- **The playground and the welcome** (3.5, 4.8), in the domain and
  the view: the welcome's three states, empty with its two cards,
  unfinished opening at Forge or at Model, whole moving on to the
  chats page, from the forges' rows and the accounts, the playground's
  rows and the scripted model counting for neither; the playground
  offered while no real forge is kept and nowhere once one is, a real
  account alone leaving it offered; a start sent once, its failure's
  words and "Try again"; the playground's pages, Forges listing its
  forge and Accounts its account, neither offering anything to add;
  the banner's one line and its "set up temper", sending
  `EndPlayground`, "Leaving the playground…" until the reset answers
  or the read answers none, then `Welcome { Forge }`, and a read
  answering resetting showing the same; a model usable by another
  provider's account, the person's own, and for an owner the
  deployment's too, an undelivered or cut `deployment` adding none, the
  owner's one watch open while a playground is read and dropped after;
  a forge cut short shown "FORGE · not finished", "Start over" sending
  `RemoveForge` at once and opening the Forge step's choices once
  removed, "Go on" first on GitHub; Remove on a forge's row, its words
  for each kind and mode, sent only once confirmed; Delete on a
  project's settings, sent only once its name is typed.
- **The accounts pages' steps and views:** in the domain, the
  accounts page's acts each as its request from the row's names, one
  out at a time, those that confirm only once confirmed, a key's value
  dropped when another row's field is written and as it leaves, "Use
  the API key first" as its move, a row's sign-in followed to its end;
  `ConnectModel`'s address read and printed for each way, a way not
  offered or the playground's showing Choose, its state kept across its steps
  and dropped as it leaves, each way as its request, a sign-in's start,
  its watched end and each of jig's endings, and the welcome's Model
  on the same state. In the view, the accounts page in each state of
  5.5's table, folded and opened, with each act and its outcome under
  its row, the deployment's and the playground's sections, and none of
  the person's own; `ConnectModel` at each way, its completion line on
  Accounts, and each ending’s words and acts; the welcome’s Model
  step with no forge's line, and with each (3.5, The welcome): under
  way with its steps and named indeterminate bar, ticked, none adopted
  and failed without a bar, GitHub's waits without a bar, the repository
  before the connection under way and failed, and the terminal's line
  in that order with no bar; the navigation's
  dot in each colour and none; the banners' ways.
- **Golden text:** temper's pages as skein's text shell prints them,
  kept as files beside the tests and changed only by hand: the inbox
  with entries of each kind, the chats, a chat with its conversation and
  a proposal in place, a chat with a delegation's card running and one
  landed (5.7), a project's goals, a goal's page in the regions' order and its
  bar by name (5.8), an agent's
  task with its brief, run, conversation and result (5.9), the
  changes, the adoption form in each state of 4.4 and 4.7,
  its picker on a GitHub and on a Forgejo forge, a forge not connected
  and "Not listed", signed out, the accounts' banner, `ConnectModel`
  with a sign-in's address, as the terminal shows it, a key's field
  written, and an account in
  each state of 5.5, and the forges' page with a forge in each state of
  `forge.md`, 5.4, a connect's link, and a pasted value's field written
  (`(hidden)`).
- **The terminal's machine world,** in `temper-client-shell-terminal`'s
  own tests (`../domain/testing.md`, section 7): the host against a scripted
  stdin and stdout and a scripted transport: signing in by the link and
  by a kept token, a page printed after an act, the end of stdin, an
  interrupt, `q` drained. And, in a unit test, the terminal made from
  `limits::native()`'s limits, as `temper open` makes it, so the gate
  meets a refusal of temper's own limits before the process tier does.
- **The TUI** (6.11 to 6.16): golden frames, `Frame::print`s laid by
  skein's renderer in the TUI's theme, kept beside `temper-client-tui`'s
  step tests and changed only by hand: the first screen at 80 by 24 and
  40 by 12, dark and light, and signing in at 80 by 24. Rows of its
  state and view (each sign-in line, each link state, each act's
  event), of its host (the pasted link's checks, the credential's
  duties, the reports' order, a key's event), of its loop on skein's
  simulated machine (each way out writing the terminal's `LEAVE` last
  and exiting 0), and of the binary's parts as functions (the command's
  parse, the directory's resolution, the scheme and the depth). Nothing
  opens a terminal or a browser.
- **The TUI in the joint world** (`../domain/testing.md`, 6.3): its
  shipped host, its root linked, its requests carried to the engine's
  clients' component as a native shell carries them, the component
  given a printed link: signed in by the link pasted, drawing `me`; the
  engine restarted under it, "Offline" then "Live" and `me` still drawn;
  signed out elsewhere, back at signing in with the token erased.
- **`tests/client`** (`temper-client-world`), the joint world
  (`../domain/testing.md`, 2.1 and 6.3): temper's client root and view, with
  skein's fake person acting on the tree, joined to temper's engine
  root through jig's clients' component, with no transport between,
  with jig's client worlds' faults and jig's check (jig's
  `client/domain/testing.md`, sections 5, 6 and 7). The forge's request
  and outcome are decoded and encoded by the engine's routing (section
  8), exhaustive both ways. Stories: signing in and seeing the inbox; a
  chat started and answered; a chat paused, resumed and paused again, and a
  message to it resuming it, answered; a goal proposed in a chat, accepted in the
  inbox, and gone from the chat's card; a
  held task released from its own page; a goal's page saying which
  task a delegate waits on, then, the first delegate ended, the second's line
  saying what it does next; an ended
  goal's result; a goal's delegate opened from its task line, showing its
  brief, its run's heading and its committed turn, the goal reached
  again by its lineage bar, and, once the delegate has reported, its
  run ended and its result; from a running delegate's page, a message
  sent and read by its scripted run's next turn; that delegate paused at once,
  saying "Paused" with no held card of its own, and resumed, running again; a
  delegate closed through its sheet, and ended; a
  goal closed from its page, confirmed, and its delegates ended; a goal paused and
  resumed from its page; a waiting delegate's Message refused, its page
  saying why and sending nothing; an amendment of a goal's
  specification from its page, and one refused as moved asking to look
  again; a goal with an advisor and two dependent delegates (rung B2,
  `../domain/testing.md`, 2.7): a goal's advisor
  reports, and its page, opened from the goal's task line, shows its result and
  reaches the goal by its lineage bar; the goal declares two delegates, the second
  depending on the first in a later call: the second's task line says "waits on
  TITLE", and its own page's state and dependency line say which; the first
  reports, the goal's page counts it, and the second runs; the second reports, then
  the goal, and its page says it landed; along the way, each delegate
  stays in the goal's "N landed" once it has ended, saying when; and a fresh
  client started at a delegate's address draws its heading and its
  "Task brief", the client root's part of the browser's open of a
  page by its address (6.10); a repository adopted
  from `Settings`' link, its outcome read and the engine holding it;
  refused: not found, a second home, a field the form refuses; a part
  of the forge's saying "Shown on the forge."; a change into the fake
  forge's repository, handed its pushed branch, its panel watched from
  its task's page as it goes from opening to landed: its deliveries
  following its steps in order, never going back, its pull request's
  number, CI passed at its head,
  its gate's verdict, and at its end its merge commit, the watch kept
  open with nothing moving it; that page read at each step, its steps,
  its step's line and its pull request's line as 5.10 draws them, to
  "Landed · merge commit" with all six done; an ended procedure's "Its
  tasks" listing its ended tasks with their endings; a task that is no
  change watched as unknown, its part saying so; the banner with no
  account, its key way followed to `ConnectModel`'s Details, a key
  added there, Accounts with its confirmation, the banner gone, and the
  key in no record of the small store; a key's request lost and sent again by the person,
  making one account; a person on `ConnectModel`'s Choose pressing
  "Continue with ChatGPT", Signing in showing its address, the
  watched sign-in made, and Accounts with its confirmation; the press’s
  `Tab::Open` in the press’s own turn, before any answer, then one `Tab::Point` for the
  address from the answer, with no `Leave`, the page's
  address naming the source in place of its entry; a refused or lost
  answer's `Tab::Close`, and an answer the page has left; a tab refused,
  no `Tab::Point`, and the note's primary button; none again on a
  rebuild or from a root made at the source's address, which draws the
  same page, its address and refusal kept in the small store, then its
  end; from the welcome's Model step, the same, moving on to
  `/welcome?at=ready` by itself on the `accounts` watch; a root made at
  Ready drawing no model line until the
  `accounts` document is delivered, then "Model: ChatGPT · signing
  in" while the sign-in is under way, then the account made; a program's login declared from
  its panel, with its command and Copy; the accounts page's rows
  folded, one opened and a key added to its account from the row; a
  limited account's words, "Use the API key first" moving its key
  up, and its task waiting; on the fake forge's GitHub, the picker's pages by
  installation, a row chosen and adopted, its ids in the request, a
  repository of a selected installation not granted and "Not listed"
  saying so with the installation's page, and a forge not connected
  offering none; on the fake forge's Forgejo, a forge connected from the
  forges' page by its pasted tokens, the value in no record of the small
  store, no document and no answer, a pasted request lost and sent again
  making one connect, a replacement answering as another user refused,
  and a person whose role does not hold the deployment's connectors
  refused; the welcome's Model step reading its forge's line from the
  `forges` watch's row, under way while the forge connects, then
  ticked with ORG/NAME once it is connected, its first repository made
  and unadopted; the welcome's Ready offering the first repository's
  name in capital, or "My project" with none, and making its first
  project once, as named, and nothing with no name, opening it on its
  Home, opening the person's project of that name when the name is
  taken, and offering its act again after any other refusal; its
  adoption sent once, after the project is made and never before, the
  project open while it is out, nothing sent with no first repository,
  and a refusal leaving the project made and the adoption offered from
  Home; "Skip" on Home reaching every client of the person, holding
  across a restart, and leaving another person's To begin and the
  person's other projects' as they were; the configuring chat started
  from `Configure`, from template 3, and `Settings`' "Configure temper"
  then naming it; a project with no
  repository saying so on its chats page, with "Adopt a repository";
  with jig's faults (drops,
  the engine restarting, a double press, a reload with an adoption
  pending, kept work), each request made once. jig's check holds in
  every story, said in temper's phrasebook, which temper's world gives
  jig's referee (jig's `client/domain/testing.md`, 6.2).
- **The routes** (section 8; `forge.md`, 9.1), in temper's clients'
  world (`../domain/testing.md`, 2.1): requests to the service's
  routes against the fake forge's GitHub, each answer, its headers and
  its `302`; a state used, unknown or past its life refused; a query
  past `routes.query_bytes`, a method other than `GET` and a forge not
  GitHub refused; a path jig claims never reaching them. No test calls
  github.com or uses a real credential.
- **The one-process world** (`../domain/testing.md`, 2.5): skein's fake person
  in front of temper's terminal client (its client root, view and
  transport), joined through bytes to `temper-service` on skein's
  simulator: the same stories through jig's default HTTP mapping and
  `temper-client-protocol-wire`, killed and restarted with an adoption
  pending.
- **The real loop's terminal** (`../domain/testing.md`, 2.4): temper's client
  root and view under skein's text shell, over jig's native transport,
  in the same loop as the service, driven by a typist that types
  commands and reads printed pages, with a fake LLM, forge and git. Its
  live-text story: a reply held after its first piece is read printed
  before its "▍", then replaced by the committed turn.
- **The process tier** (`../domain/testing.md`, 2.6): the real `temper`
  binary; skein's fake person through temper's client signs in by the
  printed link, adds the deployment's key on `ConnectModel`, and
  adopts a repository on an owned Forgejo
  (`../protocol/setup.md`, section 2), with "It runs CI" checked, since
  its runner runs Actions. ChatGPT's sign-in from the welcome's Model
  step, against the fake issuer, the person's root a browser's: its
  `Leave` followed through the issuer and the listener's `303` to
  `/welcome?at=ready`, Ready there naming the account, and the account
  still signed in on the accounts page after a restart. And `temper open` itself, stdin and stdout
  piped: it signs in by the printed link, prints the inbox, adopts a
  repository by commands, and exits 0 on `q` after a drained close.
  The first run (`../draft/archive/first-run.md`, 2.1): the chat's card for the
  change it delegated read landed, with its words (5.7). And
  jig's attack story that needs no browser, a rebound Host, against
  `temper serve`'s clients' listener: a request naming a host off the
  list is answered `421`, with nothing done (jig's
  `protocol/browser.md`, section 15). The cross-site request and the
  framed page need a browser, and come with temper's browser tier
  (`browser.md`, section 4).

## 11. Open questions

- **The TUI's later passes** (6.11 to 6.16): the browser's approval
  above the paste (`../draft/tui.md`, 4.2), the recording at the host's
  seam (its section 5), and everything of its sections 3 and 7. Default:
  the first pass alone, in `../draft/tui.execution.md`'s order.
- **The TUI's address kept at closing** (6.12): the binary writes it once
  the loop has ended, so a TUI killed outright keeps the address it
  read. Default: so; writing it at the sign-in would give the loop file
  io of its own.
- **The TUI's scheme from `COLORFGBG`** (6.15): a terminal that declares
  no background is drawn dark. Default: so, until skein's full-screen
  shell asks the terminal its colours (skein's `ui-screen.md`, section
  10).
- **The TUI while offline** (6.14): the first screen keeps "Connected to
  HOST:PORT as NAME" while the state says "Offline". Default: so, its
  words refined on the recording.

- **The chat terminal after landing** (3.5): temper/ui compares
  `chat-terminal`, `chat-terminal-font`, `chat-terminal-draft` and
  `chat-terminal-esc` with the shipped view at 1440, 1000 and 420 px,
  with menus and disclosures open and every state their messages name
  (`../draft/look-requests.md`, sections 4 and 5). Include both themes
  and fonts, the draft and its figure, questions, compact cards and
  closed chats. Check that pressing `>_` draws the terminal before any
  further input, including when full screen is refused, and that Exit
  and closing a list redraw at once with no command words in the box.
  Real keys check Chromium's short and long Escape,
  Exit, desktop shortcuts, Firefox and Safari's one-Escape fallback,
  and unavailable or refused keyboard lock, including insecure contexts.
  Default: the shared page's acts, bounded notes, kept look, focused
  tables and gallery states; compilation and print checks establish no
  rendered equality or real-key result. The departures in
  `README.status.md`, 3.5 remain temper/ui's: budget words, settings
  proposal controls, case-sensitive commands and background focus.

- **The approval flow after landing** (3.5, 4.6, 5.13): check the card's
  rendered look, the owned-Forgejo review link's sign-in, and a real human
  review on each forge. The joint-world Forgejo story holds the card and
  its files-page link in place of a browser step (`../domain/testing.md`,
  6.3); GitHub's composed box story and live proof are separate
  (`../domain/testing.md`, section 11). Default: client and view tables,
  with no approval added to the terminal inbox in this round.
- **Run-wait words after landing** (3.5, 5.5; the request
  `ui-wait-words`): temper/qa checks a charter grown past its admitted
  worker's bounds on the page and in the terminal, alongside ctl's
  `next run:` explanation. Default: the fleet, core and wire tables,
  and the view tables for every bound in both shells; no browser story,
  walk or render check within the plan.
- **The terminal's forge-key command** (3.5, 6.5): its hint says
  `temper ctl forge replace N FILE`, without the client directory or
  `--key KEY` that ctl requires (`tools.md`, 5.5, area 6, and 8.1).
  Completing that command and checking its pointer is separate wording
  work. Default: the browser's paste or a complete ctl call from its
  help; the run-wait pointer checks of 5.5 cover waiting lines alone.
- **The chat heading's rendered look** (3.5; the request `chat-title`):
  temper/ui compares the request's page and named states with the shipped
  view after landing, at 1440, 1000 and 420 px, including open disclosures
  and menus (`../draft/look-requests.md`, section 4). Default: the focused
  short-title and heading tables; they establish no rendered equality.
- **A model-written chat title** (3.5): a later request may replace the
  first-sentence short form. Default: derive the heading in the view and
  keep the stored title whole, as the `chat-title` request asks for now.

- **Account email after landing** (3.5, 5.5; the request
  `ready-account-email`): temper/ui checks a real ChatGPT sign-in and a
  real Claude Code login on Ready, the Model step, the model-check pane,
  the picker and Accounts, including the terminal's words. The check
  establishes whether the real provider's claims yield an email and
  compares the rendered look. Default: focused view and integration
  checks with fake verified claims and program statuses; browser and
  process stories and the real-login render check follow landing.

- **Settings, Models after landing** (3.5): temper/ui compares page 07 of
  `model-picker` with the shipped page at 1440, 1280, 1000 and 420 px,
  including the named states and screenshots (`../draft/look-requests.md`,
  section 4). Default: the client world's switches and adding across
  reloads, reads, failures, retries, retained names and unchanged reads;
  view tables and gallery states for the read ages and expiring new tags;
  a fake-provider row through a worker and agent. Browser and real-provider
  checks remain after landing.
  The comparison includes Provider's chevron, the single-row add form
  and its section headings. Settings' shared H1 and the open project
  menu row keep their frame's design, as the repair request leaves them.

- **A document's drawing after landing** (3.5): temper/ui looks at a flow
  in a chat and in its document, including both layouts and the narrow
  fallback (`browser.md`, section 5). Default: skein's acceptance cases
  and temper's host test hold the scene's bytes; the rendered comparison
  follows landing.

- **An agent's page opening at its end** (5.9, 5.15): a long conversation's
  page opens below its brief and, on a held task, below its own held card;
  the taskbar says the state. Earlier lines load as the log's head nears, so
  a brief past a screen and a half above a cut window lets a reader pass the
  oldest held line before the read starts; the lines then arrive under the
  brief, the view kept. Default: so, until temper/qa's walk after landing
  says otherwise. Otherwise the card moves to the log's foot, or the brief
  draws only once the window reaches the start, as a chat's context does.

- **The agent-page keys after landing** (5.9, 5.15; the request
  `ui-agent-keys`): temper/qa reads back from Message with PgUp and PgDn
  on an agent's page and a goal's untracked page, while text streams and
  after the turn ends, wide and at 420 px. The check includes a screen per
  press, focus kept in Message, the kept place as words arrive, Latest,
  long briefs and results, the plan and held cards, and opening the amendment's
  editor in a short window. Default: the focused stylesheet and view-order
  tables; no new browser story. The keys and layout are checked after landing.

- **A PgUp while earlier lines are being read** (5.15; the request
  `ui-agent-keys`): at the log's head a press has no room to move and is
  not kept for the arriving lines. Reads chain from the log's measures,
  not the presses. Default: no queued press. Keeping a bounded page request
  and replaying it after the view is restored is a candidate in skein's page
  part; it needs per-log state and a bound on repeated presses, and does not
  itself shorten the read's wait.

- **The connect-flows comparison** (3.5, 4.4, 5.2): temper/ui compares
  request pages 01–23 with main, including named states and open menus
  and disclosures at 1440, 1000 and 420 px, with screenshots also at
  1280 px (`../draft/look-requests.md`, 4). Default: the source and print
  account is distinct from rendered equality; differences return as
  findings after landing. Compare the request-owned parts, including
  Settings' select chevrons, the sheet's chooser, captions, list and empty
  search, More's summary, connection actions and each model way's path;
  the shared H1 and open project menu row keep their frame's design.
- **One repository sheet from Home** (3.5, 4.3, 4.4): the
  repository-copies follow-up, owned by temper/ui, unifies Home's own
  empty-or-copy sheet with Add repositories and draws the entry from a
  project with no repository and To begin's second step. Default: keep
  the current entries and Settings destination; `connect-flows` leaves
  those paths unmocked (`../draft/repository-copies.md`, 9 and 11).

- **A failed link beside a live cookie** (5.1): the used, expired or
  unknown paragraph remains until another sign-in, even if the cookie
  still signs in on a reload. Falling back to the transport is jig's
  client question (jig's `client/domain/client.md`, section 18).
  Default: no fallback; reload to use the cookie.

- **The person-started question flow after landing** (3.5, 5.8; the
  requests `ui-goal-asks` and `ui-goal-asks-2`): check a question past the
  document's text bound, its context, evidence and choice buttons under
  Needs you and on the goal's band; starting from Goals by a typed Draft
  path; the empty-pool and open-chat funding explanations; and person and
  advisor tasks finishing without a landing. Browser, walk and render
  checks follow landing. Default: focused sheet-to-answer stories from a
  chat and Goals, and shared domain and view tables, with no new browser test.
- **More than one whole question on a goal** (3.5, 5.4, 5.8): the page's
  price includes one `long_text` question. Default: the first whole,
  later bands' bounded first line and Answer to Needs you; extending the
  price or drawing a question's whole words on demand is later work.
- **A message question read whole** (3.5, *Whole*, *Cut*): a message has
  no `TaskText` read for its words. Default: the document's bounded words,
  an explicit cut notice and a link to its task's conversation. Reading
  that question whole needs a read of its own or carrying its whole words.

- **A closed page's latest-run model** (3.5, 5.9; the request
  `ui-model-line`): a run with no committed turn, a tail page with no turn,
  or an undecodable or nameless newest turn can leave an earlier model or
  a next-run preview. Keeping the latest run's model independently of
  readable turns is jig's `client/domain/client.md`, section 18. Default:
  the last named completion, then the task's choice or its template's
  `resolved_default`; until caught up, no model line. A turn's missing
  effort stays unknown, never filled from today's offer.
- **A goal's model, effort and account** (3.5): the goal page has no such
  line. A place for those words and their amendment remains to be designed.
  Default: no line; a closed agent's task uses its taskbar (5.9).
- **The closed model line after landing** (3.5, 5.9; the request
  `ui-model-line`): checking the muted header and taskbar in a browser and
  a real configuring chat remains after landing. Default: the view tables
  cover both shells and the joined client story carries smith's bytes to
  the terminal's closed header; no browser, walk or render check is a
  condition for completing the plan. A phone retains its hidden byline.

- **The fresh project's draft flow after landing** (3.5, 5.3, 5.13):
  checking the configuring chat with drafts off, and a fresh chat's draft,
  goal and navigation on a real box. The view tables and `chat-draft`'s
  fake-model journey cover the offers and flow. Default: browser, walk and
  render checks follow landing, under the `ui-drafts-default` request;
  they are not conditions for completing its plan.

- **The trial's later choices** (3.5, *The try*, 5.5): whether Settings
  offers "Try it", whether deployment accounts are tried, how a second
  account is tested while selection stays a new chat's, and retaining a
  failed line across a restart are `../domain/trial.md`, section 8.
  Its project-only key limit stands there too. Default: the account that
  a new chat selects, personal accounts alone, and no retained line or
  additional Settings act.

- **A stopped account beside a usable one** (3.5): the provider's fix line
  is drawn only when it has no offered model. A separate fix for each stopped
  account, even beside a usable account of that provider, remains the catalogue's
  requirement (`../draft/model-catalogue.md`, section 3). Default: the provider's
  line until those account rows are designed.

- **Sign-in words in the terminal and command help** (6.1, 6.6): the browser's
  signed-out paragraph names the link temper printed (5.1; `browser.md`, 2.4),
  while `temper open`'s refusals, the terminal's link instructions and the
  command's help still name `temper serve`. A person may have run `temper box`.
  Default: those words stay until a wording change covers the terminal and
  command help; the request `ui-qa-small` names the browser paragraph alone.
  The used, expired and unknown answers also retain the terminal's and
  `temper ctl`'s generic refusal words: `ui-small-1` changes the page's
  paragraph, with reason-specific command refusals left for later.
- **A line past `long_text`** (5.4): `standard` with no override takes a message of
  262,144 bytes (`../protocol/configuration.md`, 13.8), past `long_text`, and a page
  of its `documents.rows` lines can pass `document_bytes`. Its read ends and says so
  (5.15), where a line would otherwise draw whole. Only a deployment with larger
  models reaches it. Default: so; or `long_text` and `document_bytes` grow with such
  a profile, or the engine stops a page at a client's bound.
- **A new chat's first words** (5.4): the engine takes a task's spec up to
  `tasks.spec_bytes`, 131,072, and its composer stops at `words_bytes`, 8,192, which
  jig's `client/domain/contract.md`, 5.1 gives for a spec, a message and an answer
  alike. Default: one bound for every composer.
- **An edit past `long_text`** (5.4): the client keeps its prefix and a count of
  the characters beyond it; the browser keeps the whole value only while that
  field stays mounted and its value count stays unchanged. Keeping the whole
  edit across a new drawing, or in the terminal, needs room beyond that bound.
  Default: the bounded prefix and count, with Send disabled while any lies beyond.
- **A fresh configuring chat from its composer** (3.5, *The configuring chat's
  page*): a command ("/clear") once the composer takes commands. Default: "Start
  over" in its bar.
- **The line's words** (3.5, *its calls*; `../draft/chat-reading.md`, 7): the user
  reads the table's verbs on a real chat. `read_file` and the forge's other reads
  stay their names, as the draft's example has it, and so do `recall` and `message`,
  whose first members are a number or a body, which read badly as an argument.
  Default: the table as 3.5 has it.
- **A run's time** (3.5, *its calls*; `../draft/chat-reading.md`, 7): a committed
  turn says only when it ended. Storing when it began would change a stored format
  and a protocol, for a figure the line before it gives nearly as well: it holds the
  model's writing of any text that opened the turn, and at a window's head or for a
  turn's second run is often no time at all. The user reads it on a real chat.
  Default: from the line before, as 3.5 has it.
- **The sheet the request `ui-chat-draft` of 2026-11-05 draws** (3.5, 5.14): its
  "Budget", in the deployment's unit, and its ways by their fixed words, "Plan, tasks,
  land whole", "Task by task" and "Plan, then wait for me", the last sent by
  `shapes::set_agreed_goal_from`, which stands built with nothing sending it; and a
  proposed goal's line above the composer. Default: 5.14's one sheet, its budget the
  creating bound's, its ways the project's templates, a proposal its toast.
- **The reference's picture as a flow** (3.5, *temper's own way, beside the page*;
  `../draft/own-way.md`, 3): the reference shows no figures, the user's word of
  2026-11-06, and a flow is one path, its parts in parallel a caption until they
  branch. Default: the numbered list.
- **The reference kept beside a chat** (3.5, *temper's own way, beside the page*):
  `../draft/own-way.md`, 3, has a question about it be the configuring chat's,
  beside it. Default: a chat started there closes it, and a question about it
  goes to a chat that no longer shows it.
- **The table's QA row** (3.5, *temper's own way, beside the page*): it says "Later"
  until QA after landing is built (`../domain/findings.md`, section 8), then names
  its range as the other rows do. Default: "Later".
- **A repository being made, on a page** (4.4; `../draft/repository-copies.md`,
  sections 7, 9 and 10): the forges' rows carry `makings`, `temper ctl
  forge list` says them, and Home's second step says a project's while To
  begin shows (3.5, *Home*), and Settings' Repositories says each of the
  project's, with a failed one's "Copy again" and "Dismiss" (3.5, *Settings*).
  Default: with To begin skipped, on any other page, no line says a row.
- **A copy Home's sheet asked, once in** (3.5, *Home*): the request `ui-welcome`
  of 2026-11-05 has it ticked, then "Chats can change its code now…". Its
  making row is erased with its adoption, and the rows' intent names the set
  up's copies alone, so saying it takes a fact that a repository was copied.
  Default: it reads as any repository in the project.
- **Ready's note on a forge not Forgejo's** (3.5, *Ready*): "Forgejo is still
  setting up; this page moves on by itself." shows while any forge is under
  way and not ready, so a GitHub forge connecting or waiting for its
  installation reads as Forgejo. That request has those words alone.
  Default: as it stands.
- **The first run walked whole, kept** (3.5, *The welcome*; `journeys.md`, 9):
  that request has Forgejo then Claude or ChatGPT then Create, and "Nothing for
  now" then "Make an empty one", walked in a browser by a throwaway driver.
  `welcome-set-up` stops at Model, within `again`'s 40 seconds, and
  `project-landing` starts on Home; the process tier and the client's world
  hold what lies between, apart. Default: so.
- **A project's age and a first CI run on Home** (*Home,* 3.5; the
  user's, each a new fact of a protocol): the user's greenfield mock's
  kicker says "PROJECT · made just now" and its repository's step "its
  CI ran on main and passed.". No client is given when a project was
  made, nor a repository's CI outside a change. The user kept both facts
  out of the protocol (2026-11-04): the kicker is the current day and
  time, as on any Home (`../draft/to-begin.md`, 2), and the step says
  "Chats can change its code now.". Supplying a creation age or a first CI
  result needs its own protocol decision.
- **Which decision closed a chat** (3.6, *Closed ends on one line*):
  `Activity::Decided` carries a revision alone, so the view can't tell the closing's
  from an earlier decision. Telling them apart takes a new fact of a protocol, the
  user's. Default: a chat its requester closed hides every "Decision made".
- **A moved delegate's own tasks, on a close's sheet** (3.6, *The sheet*): jig's
  impact counts every task below the chat, with no count per delegate, so "N tasks
  below it are cancelled." takes off the delegates listed and still counts a moved
  delegate's own tasks, which closing doesn't cancel. A count per delegate is a new
  fact of a protocol, the user's. Default: they stay in N.
- **A project's only goal ending while Home is open** (3.5, 5.3):
  `Domain::has_goals` keeps Goals in the rail and tab bar whatever kind 7
  says, and Home reads it to choose Goals or Chats once To begin is gone.
  The fact reads the ended group once, so once the live watch holds no row
  it says false of a project with a goal. Home shows Chats, and with Goals
  hidden and no Goals or goal page current, Goals leaves the bar until the
  project is opened again. Default: one read, as 5.3 has it.
- **How work runs' fuller page** (5.13, *Later: the fuller page*): roles, ways to
  start work, rules, running work and history on the tab; `Proposal { proposal
  }`; and the card's "Review", with the grouping of what temper enforces and what
  agents follow that the mock's `.proposal-split` draws. Default: the tab's rows
  and the card alone.
- **How work runs' disabled rows** (5.13): Status, Funds and Telling you are
  drawn disabled, showing what temper does today, since no setting stands behind
  a report, the brief, funds or a notification. Default: whoever builds one turns
  its row on.
- **How many goals take a way** (5.13): " Used by N." and " Used by none yet."
  need a goal's template, and jig's `TaskLine`, what a client is given of a goal,
  records none. Default: the sentence is left out.
- **What a chat did, on its line** (the same request): its chat's line
  says "landed · 1 h" where the chat's delegates landed. The chats' list
  gives a chat's state, not what its delegates landed, so saying it takes
  one read per row. Default: the chat's state as `Chats` words it,
  "closed · 1 h".
- **How a project lands, on To begin's second step**
  (`../draft/to-begin.md`, 3): its line names how the project lands, by
  temper's merge queue or by its landing turn, and no client is given a
  project's `by` (`../domain/landing.md`, 13.1). A new fact of a protocol,
  the user's. Default: "Chats can change its code now." on every project,
  which names no way of landing.
- **An answer past what the engine retains** (3.5, *the words being written*;
  `../draft/chat-reading.md`, 3): a page following it from its start shows it whole
  up to `long_text`. One opened later shows the end the engine retains, 65,536
  bytes, until the turn commits, since one delivery stops at the same bound (jig's
  `client/domain/client.md`, section 18). A gap between suffix snapshots keeps
  the held prefix incomplete until a full-origin snapshot or commitment, even
  below `long_text`; recovering the missing provisional bytes needs a read
  the thread does not ask for. Default: keep the prefix and its reading position.
- **A turn too large for a page** (5.4; `../draft/chat-reading.md`, 4): a turn past
  `document_bytes` on the wire, 1,048,576 bytes, reads as too large, with no calls'
  names (jig's `client/domain/client.md`, section 18). Default: its line says so,
  and the tools read it by its parts.
- **A project's home in the switcher** (5.3, 4.4): `me` does not say
  which repository is a project's home, so the first listed stands in
  for it, and a row with several repositories never reads "3
  repositories", the state `project-home`'s hand-made page draws. The
  project's own document does say its home, per project; reading it for
  each row is the candidate, or a fact of `me`'s, which is a protocol
  change and the user's. Default: the stand-in.
- **A whole welcome's page** (3.4): a browser at `/welcome` once the
  welcome is whole moves on to the chats page, where Ready's act and "+
  New project" open a project's Home (3.5, 5.3). Default: the chats
  page, since the look request `project-home` named Ready's act alone.
- **The asker's lineage on a question's card** (`mockups/world-solo.js`):
  it needs the asker's documents on the inbox page. Default: the asked
  task's link (3.5). A question's choices are its words' (`../domain/agent.md`,
  8.2), and a retraction of a sent answer is jig's, which has none: the
  client holds an answer before it goes (3.5).
- **Later and the answered lines, kept** (3.5, *What the client keeps*; the
  request of 2026-11-05 for *Needs you* decides session order and no store
  record): a card marked later, or a line answered, surviving a reload or
  reaching another device, which takes a stored record. Default: this
  session's alone.
- **The browser's updates** (3.5, *Needs you*; that request): they leave the
  browser's page and no page replaces them, `Seen` marking staying as it is.
  Default: the terminal's list alone draws them.
- **"Add $10" on a funds card** (the greenfield mock's `needs.html`; that
  request puts budgets last): an act that widens the budget in place. Default:
  Release and Leave held, the hold's own acts.
- **A chat's own question, as a card** (the mock's `n-license`; that request):
  a chat's question in its words stays in its conversation, with no inbox
  entry. Default: as it is.
- **The rail's "N+"** (3.5, *What the client keeps*): the standing document's
  `next` says more follows of needs or of updates alike, since a page holds
  every need and then the updates (`../../help/inbox.md`), so that a long list
  of updates, which no page of the browser's reads, says "N+" of a whole list of
  needs. It never says less than there is. Default: "N+", as that request has it.
- **"Inbox" on the banner** (5.3): a page that is no project's keeps the
  banner's "Inbox", with `me`'s count, to the unfiltered *Needs you*, where the
  rail says "Needs you"; that request names the rail and the tab bar alone.
  Default: "Inbox".
- **A goal hearing a choice, as a story** (3.5, *What the client keeps*;
  `../domain/agent.md`, 8.2): that request asks one live walk of a scripted goal
  asking in the line form, a person pressing its recommended choice on Home and
  the goal hearing those words, whose scripts are throwaway. No world keeps it:
  the parser, the stock sentence, the session and the view each hold their own
  side. Default: those tests.
- **A Reject on a chat's card** (5.13; the request `ui-chat-accept` of 2026-11-06,
  which counts Reject among the card's acts): the mock's `chat.html` draws "Accept" and
  "Change something" alone, and the reason's box stands on the card in *Needs you*.
  Drawing one takes a view change and that box; its ask already goes under the
  proposal's own `project`. Default: none; a person rejects in *Needs you*, and
  asking for a change has the agent propose again (5.13).
- **Calls naming their proposals** (3.5, 5.13): filling a committed call's
  `objects` needs the core's turn record, its archived form and a mapping from
  the call's position to the proposal's number; proposals from a verdict or a
  procedure have no call. Default: keep the call anchor where one is supplied,
  and otherwise place the card by its making line, or its waiting time, as 5.13
  specifies. Filling those call objects is separate work.
- **Application rows after a decision** (5.13): retaining every application
  item and project-policy row, and restoring kind 6 and 7 values after a
  reload, needs more than the session's cached values. A person without a
  review has no such rows to read. Default: a live card uses its review; an
  ended card uses its heads and the kind 6 and 7 values already read in this
  session, with no new proposal-value request.
- **Latest on a phone, checked by a press** (3.5; the request `ui-proposal-2`
  of 2026-11-06): after landing, check that Latest's row leaves a card's
  controls reachable at a phone's width. Default: the stylesheet assertions
  hold its place in flow in both chats and agent pages; measuring and pressing
  in a browser follow landing.
- **The real-model proposal-card walk** (5.13; the request `ui-proposal-card`
  of 2026-11-06): QA repeats the two-proposal, rejection from *Needs you* and
  reload walk at 1440 and 420 pixels on a real model after landing. Default:
  that check follows the fake-box render proof; scripted stories alone do not
  settle what a person meets with a real model.
- **What a real box's walk has not shown** (3.5; `../protocol/setup.md`, 6.6; the
  requests `ui-final-walk` of 2026-11-05 and `ui-walk-3` of 2026-11-06): the pinned
  Claude Code fetch, from a box with directories of its own up to the sign-in prompt
  and reused on a second run, and a real browser's Sign out ran by hand, on fresh
  loopback deployments, and no test keeps them. Not shown: a provider's
  sign-in or a paid call; TLS, a proxy or a browser on another machine; Accounts
  with a project or an account on it; the small engine profile whole, whose client
  limits alone ran; another platform's build, a failed or corrupt download, two
  fetches at once; a copy from a public host. Default: none, until a plan walks them.
- **A real check of a model's command alone** (3.5, *The welcome*; 4.6; the request
  `ui-walk-3` of 2026-11-06): the page's way to that command passes a forge's "Run on
  this machine", which is already fetching Forgejo when Model offers it, so a check
  allowed the pinned Claude Code download as its one real service leaves that bound
  by walking the page. Nothing says Forgejo's setup is wrong. Whether such a check
  allows both downloads, or starts past the forge. Default: it allows both, and says so.
- **The rename's opening and closing, as the mock draws them** (5.3; the request
  `ui-rename-focus` of 2026-11-06): the mock selects the name as the form opens,
  folds the menu as it closes and says "Renamed" in a toast. A view can neither
  select a field's words nor fold a disclosure (skein's `ui-web.md`, 15), and that
  request names none of the three. Default: the name unselected, the menu open with
  the focus on its summary, and no toast.
- **A press that starts a chat, on a phone** (3.5, *Where the focus stands on a
  chat*; the request `ui-composer-focus` of 2026-11-06): the tree has no width, so
  a starter's or a suggested request's press asks for the composer's focus there
  too, and a browser that lets a press's activation outlive the answer may raise
  a keyboard the person did not ask for. Telling them apart takes a kit mechanism,
  a focus request that raises no keyboard (skein's `ui-web.md`, 7). Default: the
  same focus at every width.
- **A start refused, by its press** (3.5, *Where the focus stands on a chat*; the
  request `ui-composer-focus` of 2026-11-06): the send arrow disables while a start
  is out, and a disabled button holds no focus, so a start refused after a press
  leaves the focus where the browser put it, and the person clicks the composer to
  try again. That request does not ask for it. Default: a refused start moves
  nothing.
- **A composer that comes back** (3.5, *Where the focus stands on a chat*; the
  request `ui-composer-focus` of 2026-11-06): a composer's request rides its insert
  (skein's `ui-web.md`, 7), so a chat that loses Message and regains it, after a
  send or "Change something", takes the focus again, from a person typing elsewhere
  on that page at that moment. Default: not guarded.

- **Pages the mockups do not show:** a project's page with its
  resources, findings, notes and settings (policy, people and roles,
  spend; jig's `client/domain/pages.md`, section 6, and the forge's
  parts in `forge.md`, 5.3 and 7.3), but the settings' Delete (3.5); the deployment's page but its LLM
  accounts, which the accounts page shows an owner, and its forges,
  which the forges' page shows (3.5); the "Tasks" list of a project,
  which the mockups draw, after jig's pages (`client/domain/pages.md`, 6.3
  and 7); of the deployment's hosts, which the mockups draw as "Workers",
  all but what Settings' Builders tab reads (3.5): its capacity and its runs;
  a procedure's task beyond 5.10's page, and a person task beyond a
  tracked task's header and plan; making a service and showing its
  credential once; another party's kept work. Default: not in temper's
  client; the person acts on them through the engine's configuration,
  or not at all.
- **A repository's role, health and landing rules on the Repositories
  tab** (3.5; `forge.md`, 5.2 and 5.3). Default: not drawn, as the user's
  request of 2026-11-05 settled: a line says its name and its forge.
- **The form's place:** among a project's settings (`forge.md`, 5.3),
  which `Settings` (3.5) links and does not hold. Default: a page of its
  own, from `Settings`, Home and the banner's navigation.
- **The forge's `Refused`, split** into its causes (another project
  writes the repository; the project has a home; another deployment's
  branches under the prefix), so the form names the one. Default: the
  sentence of 4.7 naming the three.
- **The forge as people's sign-in provider:** temper's own routes on its
  clients' listener run the forge's OAuth flow, read who the person is,
  and ask jig's clients' component to mint a credential for that
  identity (`Mint`, with the forge's provider number, a provider of
  kind application in jig's `providers` member, the person's id there,
  their login and name; jig's `protocol/client.md`, section 8).
  A browser then holds it as the default mapping's cookie; a terminal
  needs a flow of its own, such as a device code. Default: the
  deployment's link only.
- **What a page keeps across a restart:** where the client was, the
  composers' words and the form's values, as records of temper's own,
  which the terminal keeps beside `jig-client`'s small store, never in
  it (jig's `protocol/client.md`, section 10). Default: none; the client
  opens at the inbox.
- **Clients that are not a person's:** a command run once and a
  service acting as a party are the agents' tools, `temper ctl`
  (`tools.md`). A bot: none.
- **Several terminals:** each `temper open` with its own DIRECTORY is a
  client of its own. Two on one DIRECTORY share its token and write its
  small store whole over each other. Default: not supported and not
  detected; a lock on DIRECTORY is the candidate, which `temper ctl`
  takes for its own calls (`tools.md`, 8.3).
- **A fresh link from the client:** default: the person runs `temper
  open` with the last link, whose refusal makes `temper serve` print a
  fresh one.
- **The Away switch and quiet hours** (3.5, *Away*): "08:00" is the mock's
  end of its quiet hours, a fixed hour here. Quiet hours and notifications
  are later parts, and a setting of the hour would come with them. The
  terminal reads no zone (5.12), so its 08:00 is universal time's.
- **A browser on another machine, detected.** A sign-in's redirect
  reaches the engine on loopback; a browser elsewhere lands on its own
  error page, and its person finishes by pasting that page's address
  (3.5, `ConnectModel`, From another computer), or through a tunnel
  (the walk `box`, its account step). Nothing asks which: the page
  offers the paste to everyone (the user, 2026-11-06). Candidates for telling by
  itself: before the browser leaves, the page asks the listener for a
  one-time code the engine gave it for this sign-in, and only a browser
  that gets it is sent on by itself, the others offered the paste
  first; or a connection from an address not the engine's
  machine's goes so at once. Later: a client relaying the redirect
  (jig's `protocol/llm.md`, section 10). Default: undetected, the paste
  offered to everyone.
- **The first use of a plan,** in the terminal: a card in the
  conversation, or a card that waits to be dismissed, another kind of
  `Dismissed`, keyed by its account (`../draft/to-begin.md`, section 5).
  Default: a card in the conversation (5.5), with nothing to dismiss.
- **A key typed in the terminal** is echoed by its line editing (6.5).
  Default: accepted on one person's machine; `temper ctl account
  add-key` reads one from a file.
- **The picker for a person who may adopt but not connect:** the
  `Reachable` request's who-may is the deployment's connectors', so a
  project's maintainer without that role sees no list. Default: refused
  for the role (4.4), and the person adopts by owner and name, which
  the engine judges by the project's role as any adoption; a read the
  project's role may make is the candidate.
- **The picker's page as a read:** a page could be a read of the forge's
  family rather than a keyed request. Default: a request, since its
  answer is a fresh read of the forge by the setup's host's plan; keyed
  as any request, and its answer to a
  form that has closed dropped.
- **What the picker's ids do at adoption:** `Adopt` carries the row's
  repository id and owner id (4.6), while the forge connector locates a
  repository by owner and name (`../domain/forge.md`, section 4).
  Default: the connector locates by owner and name as any adoption, and
  refuses as not found one whose id is not the carried one, so a
  repository renamed or replaced between the page and the adoption is
  never adopted in its stead; the domain's page is asked to say so.
- **The project's own repositories in the picker:** `Reachable` names
  no project, so the engine cannot leave out what the project holds,
  as `../domain/forge.md`, what the picker offers, says the list does,
  and the client holds no list of a project's repositories
  to leave them out itself (4.3). Default: every reachable repository
  is listed, and adopting one the project holds is refused by the
  engine as any adoption; the request naming the project, the engine
  leaving those out, is the candidate (`forge.md`, section 10).
- **A connect's end told:** the forges' page reads how a connect ended
  on the deployment's part. Default: that read, watched while the page
  is open; no notice in the inbox.
- **The playground's and the agent's placeholders** ("Try a suggested
  request…"; "Write to T12…", the
  agent's task's composer, 5.11): skein-ui's text area takes a
  placeholder, which a chat's composer and `NewChat`'s, "What do you want
  done?" in every project, the playground's start among them, draw
  (3.5). Default: neither drawn, the agent's waiting for its page's
  owner.
- **The composer's selected account and source** (5.5): jig's
  `PayerReadiness` could carry the account and source its selection
  chose (`jig-domain-core`'s `catalogue.rs` already calls `select`).
  The view could then read jig's choice for every payer without
  translating selection over the viewer's own account slice. Default:
  name an account only in that slice, with the readiness cross-check
  and expiry fallback in 5.5; other work names the model alone, with
  the playground provider's exception.
- **A time's moment on demand:** an ago ("12 min ago") has no title
  or hover saying its moment. Default: none; the view can give the time
  node its moment as a hint (`skein/docs/design/ui.md`, 3.2).
- **The terminal's reprint:** skein's text shell prints the whole page
  when it changes (skein's `ui-text.md`, section 4), so a page with a
  time under 12 hours old prints again each minute while it is shown.
  Default: as printed; a terminal shell that says every time as its
  moment, with no wake, is the candidate.
- **A delivery beside a long conversation** (5.15;
  `../draft/chat-reading.md`, 3): retaining parsed closed blocks or a conversation
  subtree needs invalidation rules and proofs; reducing page bookkeeping and
  away-reader anchor scans needs the same. Default: rebuild and diff, keeping
  DOM identities. The measured whole-bound delivery cost is in
  `temper/docs/development/browser-budget.md`; the away-reader scan's share
  remains untimed. The browser tier's smaller window leaves a person's cost
  at the whole bound in place.
- **Reading a conversation in every browser** (5.15; `../draft/chat-reading.md`, 5):
  the position staying put when lines load above is for the user to read on a
  real chat. Default: the browser tier runs Chromium, including with
  `overflow-anchor: none`; Firefox and Safari themselves are read, not run.
- **A turn with a call before its first text** (3.5, 5.15;
  `../draft/chat-reading.md`, 3): keeping that text's nodes through commitment
  needs its key independent of the preceding call or card's place. Default:
  keep the current block keys until this ordering is reproduced; the
  `ui-pgup` request uses a render driver at 1440 and 420 px for its proof,
  with no new browser story, since the tier is at its budget.
- **An agent's page going to its end** (5.15; `../draft/chat-reading.md`, 5): its
  log has no end request, so a person sending from far back stays where they read,
  and Latest reaches the end a page at a time. A chat's `to_end` and
  `Thread::latest` on a task's page would close both. Default: as it stands.
- **The terminal's keys and first page** (5.15; `../draft/chat-reading.md`, 5): its
  input is never raw (skein's `ui-text.md`, section 7), so PgUp and PgDn are lines
  with Enter, and with no view to measure it opens on one page. Default: as it
  stands; keys of their own are a change of skein's shell.
- **The rest of a run's line** (3.5, 5.9; `../draft/chat-reading.md`, 2 and 7): words
  for tools past the table, which are the user's to read on a real chat. Default: any
  other tool is its name. A run of one turn has its time as *A run's time* says, and an
  agent's page has a running run and Latest as a chat's, sending not pressing it.
- **An agent's conversation drawn as a chat's** (5.9): the agent's
  page draws runs as a chat does, and keeps the run markers
  as work lines, which a chat hides (3.5).
  Default: as work; the chat's drawing is the candidate, the task
  pages'.
- **Which person messages are the person's** (3.5): jig's turns keep no
  provenance, so a chat draws none of a turn's person messages, and
  the person's words from the thread's words alone. A charter whose
  person's words reach the chat only inside a turn would show none of
  them. Default: as drawn; a turn's message kind from smith's record
  is the candidate, jig's.
- **Seen while no one looks:** a chat's page open in a tab the person
  is not looking at, or in a terminal left open, marks the turns that
  arrive as seen (3.5). Default: open is seen; the page's visibility,
  an input of the browser's shell, is the candidate.
- **A delegation's tasks, keyed** (5.7): jig's thread keeps a
  `delegate` call's `objects` empty, so the domain reads the tasks
  from temper's acknowledgement and, at a result, from temper's inbox
  words: words of the engine's that the client depends on, each marked
  so where it is written and where it is read. Default: the objects
  when jig's thread keys them (a jig change and its repin), the words
  meanwhile, both ends temper's and joined by the process tier's first
  run (section 10).
- **A change's live stages** on its card (5.7; `forge.md`, 2.1): the
  steps are a stand-in, ticked from the result. The engine publishing
  the change's step as its task's part, which the forge's side reads,
  is the candidate, about 1.5 to 3 thousand lines. Default: the
  stand-in. The change's task page draws its live step from its panel
  (5.10); the card does not watch it.
- **"Landed into main":** the
  branch a change lands into is a parameter of the forge's procedure,
  which jig's contract does not type, and the landing words name the
  merge commit alone. Default: "Landed · merge commit …"; the result's words
  naming the branch are the candidate.
- **"CI passed" on a card** says what the stand-in's steps say: a pull
  request merged by someone else is landed like any landing
  (`forge.md`, 6.2), CI or not. Default: the words, until the live
  stages.
- **The pull request's link on the card** ("OWNER/NAME#N on
  Playground forge"): the forge's part, which the forge's
  side does not read yet (4.2). Default: the link to the task's page.
- **An added dependency** on a task's page (5.8, 5.9):
  `Dependency.added` says when and by whom a dependency was added.
  Default: a mark in its item of the dependency line, who added it and
  whether it keeps the task, which keeps the line's shape; when, in
  `ctl task show TASK` alone.
- **The rest of a goal's authority:** its tools, grants, executor kinds
  and note scopes, which jig supplies as codes and patterns, not
  words. Default: not on the goal's page.
- **A goal's page against its mock** (5.8), each a default the user may change:
  - task rows are links, the one departure: a held delegate's Release is on its
    own page;
  - no "earlier reports" and no reorder; no budget field and no "Add $10": budgets
    matter least;
  - no drain: Pause is jig's Stop, so its toast says running tasks stop, where the
    mock's say they finish. jig's Drain and a party's `Hold` are not built;
  - "Close" is `Ask::Cancel`, where the mock says "Stop": Pause, Resume and Close are
    one vocabulary across a chat, a goal and an agent's task.
- **A message to a paused goal or agent's task** (3.5, *A person's pause*): the request
  `ui-pause-close` makes sending resume a chat alone. Default: words alone, which wait for
  Resume, since a goal's release lifts its whole subtree's hold; an agent's task that resumes
  on a message is the candidate.
- **A goal's own line** (5.8): no key selects one task, so a goal's counts follow
  `Read::Tasks` by its project, requester, label and tracked for at most 8 pages,
  and a party with more goals than that sees what the page holds. Default: so,
  since the request `ui-goal-page` keeps jig's part to `made`; a key selecting one
  task is the candidate (a jig change).
- **An agent's task's page against its mockup** (5.9), each a default
  the user may change:
  - its dependencies, which the mockup draws on no agent's page: the
    dependency line under its brief;
  - a cut document's dependency line: no last item, since the
    document's `cut` does not say which list was cut; a cut of its own
    on the dependencies, as a plan row's, is the candidate (a jig
    change);
  - its calls: runs as a chat's, one line a run (3.5), a row a call,
    a result its row's outcome, which the mockup doesn't draw;
  - its plan: drawn while its tree holds a delegate or a held card,
    where the mockup draws none;
  - its acts are 5.11's.
- **A task's page's acts** (5.11), each a default the user may change:
  - an agent's task's Pause sent at once, as a chat's and a goal's, though that
    page acts on work the person may not be watching: Resume undoes it (3.6). A
    sheet on it is the candidate;
  - Close on a goal's and an agent's task's page alone; a procedure's Cancel, which
    no page draws, is reached from `temper ctl`;
  - the amendment's editor on the pages: the specification's words
    and, on a goal, its tracking and priority, the model choice being
    the choice line's; the rest of an amendment is `temper ctl
    task amend`'s. A draft whose task moved is discarded and opened again,
    not carried onto the new revision: the person rereads what
    changed.
- **A change's page against its mockup** (5.10), each a default the
  user may change:
  - "Its tasks"'s rows: 5.8's plan rows, with their executor, state and
    spend, where the mockup draws each as a gate's row, its kind and
    its phase, and heads the region "made by the change procedure…";
  - a gate named "Gate N · kind", where the mockup names the landing
    rule that adds it, and "No other gate." with no "its landing rules
    add none", since the panel carries the gates, not the rules;
  - its step's line, its hold and its bounds, which the mockup does
    not draw;
- **Older ended delegates and a goal's older activity** (5.8, 5.10):
  a goal's tasks and a procedure's "Its tasks" read one page of their
  ended delegates, the first made, and say "More of its tasks are not
  shown." or "More of its ended tasks are not shown." past it; a goal's
  page reads its latest activity's page for what the person sent, not the
  older ones. Default: one page of each, since a procedure makes a handful
  and a goal's page is read for what is under way. Reading at a list's edge
  is the candidate, as an agent's task's conversation does (5.15): the
  ended page's `after` as the next query's
  start, and the activity's older pages by their `before`; with the
  view's words and tests and the client world's story, about two hours
  of work.
- **A sign-in's endings jig gives no fact for** (5.5, A sign-in): the
  listener's port another program holds, before the browser leaves; a
  ChatGPT account not eligible for plan usage; plan usage declined at
  the provider, apart from its consent's `declined`; and a missing
  scope said in its own words, where jig gives its number. Each waits
  for a fact of jig's ending (jig's `client/domain/contract.md`, 7.1;
  the providers' facts, jig's `domain/providers.md`, 2.5); until then,
  each reads as the ending jig gives, in its words. Default: none drawn
  before jig's fact.
- **A sign-in made with no model usable** (3.5, *Ready*; 5.5, A sign-in):
  a sign-in that ends `Made` or `Renewed`, not declined, while no account
  is usable has no words: 5.5's "temper’s agents now run on your ChatGPT
  plan." would be false there. Ready's rule stands, that such an ending is
  never silent; the words are the user's.
- **A sign-in started from another client** (3.5, `ConnectModel`): a
  page opened at a sign-in's way, by a reload or a link, knows no
  sign-in of its own, and offers the way's button again; the accounts
  document lists the sign-ins under way, but not which client started
  each. The page's address names its sign-in by its source
  (`ConnectModel`, *A reload while it waits*), so a page reloaded, or a link
  copied from it, shows that sign-in; one opened at the way without a
  source does not. Candidate: the page shows the person's sign-in under
  way for that provider, with its end. Default: the way's button
  again.
- **The new tab closing itself** (3.5, `ConnectModel`, *Its end*): temper's page
  could keep its hold on a tab it pointed and close it at the sign-in's end,
  but a provider's opener policy may sever that hold. Default: the page lets a
  pointed tab go, and the person closes it. **A forge's departures** (3.5,
  `ConnectForge`): GitHub's App and install still leave in this tab, and may
  want the same. Default: as they are (`../draft/sign-in-new-tab.md`, 6).
- **The person's login over 760 px** (3.5, *Sign out*; 5.3; the request
  `ui-owner-menu`): the sheet hides the `Accounts` tab's row whole, so a browser
  that wide reads the person's name on the rail's foot, and their login on no tab
  of `Settings`. Candidate: the sheet hides the row's button alone, its words
  staying. Default: the whole row.
- **Which tasks hold a revision, and which could move** (5.13). jig's
  `settings` document counts the live tasks holding each revision, but
  a task query has no revision among its keys (jig's
  `client/domain/contract.md`, 6.1), and nothing says before a move or
  an acceptance how many tasks could not move. Default, asked of jig:
  a task query keyed by revision, its live group only (a scan, which
  costs live tasks no more than any key), and a proposal's or a move's
  count of tasks that could not move; without it, "Running work" and the
  review page draw counts alone and say in words which tasks stay.
- **The switcher's name** (5.1): the look requests name it "Project:
  NAME" apart from its summary's words, NAME; skein's disclosure names
  its summary by its words alone (skein's `ui-web.md`, section 6).
  Default: named NAME, until skein's disclosure carries a label of its
  own.
- **The playground's suggested requests** (5.6): the look requests
  (`journeys.md`, 7) drew the chats page without the playground's
  invitation, its suggestions among it; the user kept the suggestions
  while tests walk the playground by them, to revisit (11-03).
  Default: `.starters` under the playground's start, on `NewChat`.
- **A taken name at Ready** (3.5, The welcome;
  `../draft/first-run-projects.md`, 9): a person who made a project of
  the name Ready's field holds, the offered one unless they wrote
  another, before finishing the welcome, from the banner's new project
  (5.3) or ctl. The user's to decide: open it, or make "Hello 2".
  Default: Ready opens the person's project of that name and makes no
  other
  name, the reading the draft's status records while the user is
  away.

- **Whether `owned` asks the box** (`../draft/serve-without-box.status.md`):
  the draft offers Forgejo on this machine from the forges' rows, and a
  first run's store keeps none. Default: the deployment's read says
  `owned` (4.6), the document's member and the box both, so that
  `temper serve` on a box's document draws no card its `SetUp` would
  refuse. The user's to decide: keep it; have `owned` read the document
  alone, the card then refused "not under the box"; or have the engine
  list an offered owned row from the machine at a first run, which
  changes what `forges` answers and what a first run's store reads as.
- **A box with no `owned` part** (4.6, 5.5): `temper box` on a document with
  no `application.machine.owned` reads `owned` false, so a wait for a host
  there keeps the words for many hosts. `temper init`'s box document has the
  part. Default: so, over a new fact on the wire, until a person meets it.

- **A command for anyone but the owner** (3.5, 4.6): `program_login` goes to any
  party signed in, its paths with it, so a person who isn't the owner reads on
  Accounts the directories `program_environment` refuses them. It names all three
  by their flags, at their defaults or not, where the owner's words name each only
  off its own: the request `box-small-2` of 2026-11-06 asks that of
  `program_environment` alone. Default: so, what's printed working when pasted,
  until a deployment says otherwise.
- **This deployment's directories under "each host"** (4.6, 5.5): Connect a model's
  Ready says "Run this on each host temper’s agents run on" over "On this machine,
  run:" and the command with `program_environment`'s words, which are this
  deployment's directories and this machine's binary, and another host's may
  differ. Default: so, a box being one host.
- **A `~/` in another's shell** (4.6): the words' `~/` is the engine's `$HOME`,
  and a shell of another user on this machine reads its own. Default: so, the
  owner being the person at this machine.
- **A Copy on the other commands** (3.5, *The command's Copy*; 4.6): the welcome's
  block has one, as the request `box-small` of 2026-11-06 asks; `ConnectModel`'s
  command and Accounts' have none. Default: so, until a person asks for one.
- **The command before its read answers** (4.6): while `program_environment` is
  unanswered, or if its read fails, a page with no `program_login` draws the
  placeholders and says nothing. Default: so.
- **A wait said twice** (3.5, 5.5): on a chat's page the bar and the wait's own
  line both say "Waiting for an account", and with no usable account "No model yet
  · Connect a model" stands beside the wait's "Connect a model". Default: so, until
  a walk reads them.
- **A directory that isn't UTF-8** (4.6): `program_environment` carries its
  bytes as they are, and the page draws text. Default: the command isn't
  drawn as it would run; the flags `../../help/box.md`, section 5, shows
  still do.
- **A goal row's count of landings** (3.5, *Goals*; 5.8): `TaskLine` counts a goal's
  delegates ended done whatever they did, so a row and the goal's headline say "N of
  M tasks done.", and only the goal's own rows, whose chips carry their results, say
  "landed". Default: so; or jig's `Endings` counts change results, a stored and a wire
  field.
