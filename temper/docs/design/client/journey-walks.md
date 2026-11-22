# Journeys walked in a world

Provisional, 2026-11-21. A journey printed in one process, with no
box, browser, terminal, recording or network. Every journey on main
has a `walk` of version 2 and is walked here, the browser's and the
TUI's alike. The writer prints them with `temper-client-walk pages`
(`journey-writer.md`, 3); a request's landing prints them into scratch.

## 1. In one page

- **A journey is its walk** (section 2): a `walk` file of version 2,
  the drive session's lines with a client named on each, `web mark` or
  `tui mark` where its client's page is printed, `tui approve` where
  the browser approves what the TUI asked it to browse, and `after
  NAME` for the journey it starts from.
- **Two libraries** (section 3): `temper-client-walk-lines`, the
  lines' grammar, the walk's reader and the two page printers, free of
  the browser kit; and `temper-client-walk-world`, the two clients'
  driver on the playground's composition and the walker.
- **A walk in a world** (section 4): the shipped web host and the
  TUI's shipped host against the playground's composition on skein's
  simulator, at fixed facts, each line acted as a person acts and each
  mark printed as a page, the same bytes from the same tree every time.
- **The TUI on the world** (3.3): its host given keys and read as
  frames, with no terminal; its asks and its stream carried by the
  shipped terminal's own HTTP transport, a process of the world, to the
  service; what it keeps held for the walk; its deadlines on the
  world's time; what it asks to browse given to the walk's browser.
- **The box's peer** (4.4): the world's `temper box`, at the service's
  box pipe, reporting an owned Forgejo's set-up step by step on the
  world's time, as the box reports it, with no step taken through ctl.
- **A sign-in's tab** (4.5): the tab the page opens for a model's
  sign-in, carried by a tab of the world's to the fake issuer and on to
  the engine's listener, so that the account is connected through the
  client, as a box's fake backends connect it.
- **The program** (section 5): `temper-client-walk print [--out DIR]
  [NAME]...` prints into scratch, laid out as the tree with its
  stylesheets, and `temper-client-walk pages` writes main's pages, for
  the writer alone, both roots' journeys.
- **The journeys' walks** (section 6): the welcome, the owned forge and
  model's sign-in, a project landed, the browser's chat and draft, and
  the TUI's approval, chats and design session. Each journey's sources
  say what it walks; only the writer writes its pages.

## 2. The walk

### 2.1 Its directory

A journey is a directory under `web/journeys/` or
`tui/journeys/` whose `walk`'s first line is exactly `walk 2`, with:

- **`walk`:** 2.2's text, and **`README.md`:** what it walks and what
  each page shows. Sources, any change's.
- **`scripts.json`,** where it has one: the fake model's scripts for its
  steps, in the box's `--scripts` form (`recipe.md`, 2). A source.
- **Its pages,** `01.html` onwards, with `01.txt` beside each for a TUI
  journey, and the files beside the root: the writer's alone
  (`journey-writer.md`, 2).
- **No recording is a source.** A leftover `recording` is the writer's
  to remove once the journey prints (`journey-writer.md`, 2).

A journey belongs to the client whose root holds it and prints that
client's pages; its walk may act in the other client too, as a TUI's
sign-in is approved in the browser (`../draft/archive/journey-walks.md`, 3.1).

### 2.2 Its lines

Text, one line each, in the drive session's form (`drive.md`, 2):

| Line | What it is |
|---|---|
| `walk 2` | the first line |
| `after NAME` | the second line, or absent (2.3); NAME a journey's name, bare |
| `web press ROLE NAME [ORDER]`, `web submit ...`, `web toggle ...`, `web input ROLE NAME VALUE [ORDER]`, `web see TEXT`, `web reload PATH` | the drive session's browser lines, read by its grammar (`drive.md`, 3.2) |
| `tui key KEY`, `tui see TEXT`, `tui stop` | the drive session's TUI lines, read by its grammar (`drive.md`, 3.3), KEY as the journeys' file writes a key (`tui/journeys.md`, 3.2) |
| `tui approve` | the browser opens the address the TUI last asked to browse, presses its "Approve", and loads again the address it was on (4.2) |
| `web mark`, `tui mark` | prints that client's page as drawn then (2.4) |

- **Read by the drive session's grammar,** which gains `web mark`,
  `tui mark` and `tui approve`, each printed as its words, bare. A
  walk refuses every other line: `ctl`, `forge`, `deployment`, `end`,
  `web look`, `web requests` and `tui look` by name, as a box's or a
  session's. An empty line is skipped and keeps its number.
- **A journey marks its own client's pages:** a `tui mark` in a journey
  under `web/journeys/`, or a `web mark` under `tui/journeys/`, refuses
  the walk, naming the journey and the line, whether it is the walked
  journey or an ancestor.
- **`web mark`, `tui mark` and `tui approve` are a journey's lines
  alone:** a recipe refuses each as it reads it (`recipe.md`, 3).
  Outside a journey's session, the drive session refuses `tui mark`
  and `tui approve` as journey lines, keeps neither and starts no TUI
  for them (`drive.md`, 3.3). In a session started with `--journey`,
  marks answer looks without being kept and `tui approve` approves
  what the TUI browsed. `temper-client-journeys journey walk NAME`
  walks the chain in one such session on a box (`drive.md`, 2). A
  recipe's walk signs its TUI in as an ordinary session does
  (`drive.md`, 3.3).
- **A line's number** is its line in its file, from 1, `walk 2` the
  first.

### 2.3 `after`

- **`after NAME`** names a journey under either root whose `walk` is
  of version 2. Its lines run first, its own `after` before them, and
  its marks print nothing.
- **Its chain** is the journey and at most eight ancestors, root first,
  refused as a recipe's is (`recipe.md`, 3.2): a ninth ancestor, a
  cycle and a missing journey each refused, naming the journey whose
  `after` names it; so is a NAME whose `walk` is not of version 2.
- **`scripts.json`** files join along the chain, root first, as a
  recipe's do (`recipe.md`, 2), by `temper_fake::llm::ScriptJoin`.

### 2.4 Marks and links

- **A mark prints one page:** its client's page as drawn when the line
  is reached, every line before it having held, numbered from `01` in
  the walked journey's marks' order. Typing marks nothing, and a walk
  prints exactly its marks.
- **A browser mark's link:** the first `web press`, `submit` or
  `toggle` after a mark and before the next one links the marked page
  to the next page, by its act, where its control is on the marked
  page: the line's control sought on the tree as it was marked, by the
  line's role, name and order. A control that typing or another act
  revealed is not on it, and that page leads on by its neighbours, as
  the last page does.
- **A TUI mark's key:** the first `tui key` after a `tui mark` and
  before the next one is the mark's key, linking the marked frame to
  the next page (`tui/journeys.md`, 6.4), where both hold: no line
  between them but a `tui see` or a `web see`; and the TUI's host, at
  the mark, gives that key an event that is not typing
  (`Host::event_of`, an event but `Event::Typed`), as the recorder
  marks before a key (`tui/journeys.md`, 3.1). Otherwise, and at the
  walk's last mark, the mark has no key and its page leads on by its
  neighbours.
- **The printed form** is the client's: the browser's
  (`web/journeys.md`, 5.2 and 5.3), `print::pages` over each mark's
  HTML, the main region's level-1 heading, its acted control and its
  lead; the TUI's (`tui/journeys.md`, 6.3 and 6.4), `tui_print::pages`
  over each mark's frame and key in `facts::SCHEME`, each mark's
  `NN.txt`, its rows, and `NN.html`, its themed page.

## 3. The libraries

### 3.1 `temper-client-walk-lines`

`temper/crates/temper-client-walk-lines`, the `harness` profile, out
of `temper-client-journeys-tool`, whose drive session, recipes and
recorded-file printer read through it. It names no `skein-browser`,
`skein-fake-person-browser` or box crate.

- **`line`:** the drive session's lines, `Line`, `Record` and the rest,
  with `Web::Mark`, `Tui::Mark` and `Tui::Approve`.
- **`walk`:** a walk's reader (2.2, 2.3):

  ```rust
  pub struct Roots { pub web: PathBuf, pub tui: PathBuf }
  pub struct Walk { pub chain: Vec<Journey> }      // root first, the walked journey last
  pub enum Client { Web, Tui }
  pub struct Journey { pub name: String, pub client: Client, pub lines: Vec<Numbered>, pub scripts: Option<Box<[u8]>> }
  pub struct Numbered { pub number: u32, pub line: Line }
  pub struct Unreadable { pub journey: String, pub line: u32, pub why: String }
  pub fn read(roots: &Roots, name: &str) -> Result<Walk, Unreadable>;
  pub fn moved(directory: &Path) -> std::io::Result<bool>;
  ```

  `moved` is the header predicate: true where `directory/walk` exists
  and its first line is exactly `walk 2`, false otherwise. The program
  uses it to select journey directories and validate a named journey
  (section 5). `lines` hold
  every line after the header and the `after`, empty ones left out.
  `client` is the root the journey was read from, the browser's looked
  at first, as `read` looks up a name. `Unreadable`'s `line` is 0 for
  the file as a whole.
- **`print`:** the browser's pages (`web/journeys.md`, 5.2 and 5.3),
  `Mark`, `Printed`, `pages`, `beside` and `SCRIPT`, with
  `pub fn heading(tree: &skein_ui::Tree) -> Vec<u8>`, the main region's
  level-1 heading, and `pub const STYLESHEETS: [&str; 4]`, the
  stylesheets a page links, relative to its journey's directory, which
  `pages` writes from.
- **`tui_print`:** the TUI's pages (`tui/journeys.md`, 6), `pages`,
  `beside`, `stylesheet` and `SCRIPT`. A page links `../theme.css`,
  `../overlay.css` and `../tui-journey.js`, the files `beside` writes or
  keeps.

### 3.2 `temper-client-walk-world`

`temper/crates/temper-client-walk-world`, the `harness` profile: the
playground's native page, out of `temper/tests/playground`, whose
stories keep using it, the TUI's host beside it (3.3), and the walker.
It links the playground's composition, the web shell, the TUI's shell
and the native client's shell (`jig-client-shell-native`), and names no
browser kit, box crate or `temper-client-journeys-tool`.

- **`facts`:** `SEED: u64 = 7`; `WALL`, 2026-01-06T09:00:00Z; `SERVICE:
  &str = "127.0.0.1:10001"`; `FORGE: &str = "127.0.0.1:10002"`, the
  forge's (4.1); `ISSUER: &str = "127.0.0.1:10003"`, the fake issuer's
  (4.5); for the TUI, `SIZE: skein_ui_screen::Size`,
  80 columns by 24 rows, `VERSION: &[u8] = b"0.0.0 (000000000000)"` and
  `SCHEME: temper_client_tui::Scheme = Scheme::Dark`, the scheme Auto
  draws for a terminal that answers none (`tui/theme.md`);

  ```rust
  pub fn tui_seed(before: u64) -> u64;
  ```

  the seed of the walk's TUI started after `before` others,
  `SEED + 1 + before` (wrapping), so the browser and each TUI draw
  their own request keys, as a person's browser and each run of their
  terminal draw their own seeds: hosts of the same seed sign the same
  user's acts with the same keys, and the service refuses the second
  act's key as naming another act; and

  ```rust
  pub fn start(scripts: Option<&[u8]>) -> Result<temper_playground::story::Start, String>;
  ```

  story A's start at `SEED` and `WALL`, its service's clients'
  listener at `SERVICE`, its forge at `FORGE`, the model's plan signed
  in through the fake issuer at `ISSUER`, story A's bounds that a box
  does not hold lifted (4.1), and, where `scripts` is given, the model's
  scripts replaced by them, as `temper_fake::llm::Document::decode_with_scripts`
  replaces them.
- **`Clients`,** the world's harness, `pub enum Clients {}` with
  `impl temper_playground::programs::Harness for Clients`: its
  `Service` is `()`, its `Process` holds the TUI's connection,
  `tui::Connection` (3.3), and the box's peer (4.4) and the world's tab
  (4.5) beside it under the harness, and every other item the trait's default, the playground's
  own. **`Playground`,** the composition's world type,
  `skein_sim_world::World<Process<Clients>, Machine>`, moved from
  `temper/tests/playground/src/lib.rs`, holds the composition's
  processes, the box's peer and the TUI's connection once a TUI starts.
- **`box_peer`:** the box's peer (4.4): `BoxPeer`, a host of the
  world's (skein-shell's `Host`), and `OWNED`, the owned group's script
  as data, a line and its time each, which the tests read.
- **`web::Page`,** the shipped web host on the world (`../playground.md`,
  9), its turn loop as the playground's page has it, the world's
  owner and the TUI's (3.3), with:

  ```rust
  pub fn start(start: temper_playground::story::Start, name: &[u8]) -> Result<Page, Failed>;
  pub fn html(&self) -> String;                  // the page as drawn: skein_ui_web::page::Page::html
  pub fn tree(&self) -> &skein_ui::Tree;
  pub fn address(&self) -> Box<[u8]>;            // the path and query the page shows
  pub fn wait(&mut self, find: &Find, act: Option<&Act>, within: Duration) -> Result<NodeId, Failed>;
  pub fn act(&mut self, node: NodeId, act: Act) -> Result<(), Failed>;
  pub fn reload(&mut self, path: &[u8], within: Duration) -> Result<(), Failed>;
  pub fn walk(&mut self, person: &mut Person<Address>, bound: Duration) -> Walked;
  pub fn tab(&self) -> Option<&skein_playground::tab::Shown>;   // the tab's last answer (4.5)
  pub enum Failed { Bound, World(Box<[u8]>), Stopped, Turns, NoLink, Closed, NotBrowsed, Asked(&'static str) }
  ```

  `Find`, `Person` and `Walked`'s steps are skein's fake person's
  (`../testing/fakes.md`, 4); `Act` and `NodeId` skein-ui's; `Duration`
  skein-lib's, in the world's time. `start` makes the world with the
  box's peer at the service's box pipe (4.4), and spends the service's
  printed sign-in link, home `/welcome`, as the page does today.
  `address` is the path, with its query, of the address the page last
  moved to (`Memory::shown`), the start's home where it has moved to
  none. `wait` steps the world and both clients (3.3) until `find`
  matches a node of the drawn tree, as `TreeFace` finds it, and with
  `act`, only a node that face would give that act counts; past
  `within` it is `Failed::Bound`, and with nothing due before then the
  browser's time goes to `within`'s end. `act` gives the host
  `Report::Act` and turns, carrying any tab the host asks for (4.5).
  `reload` starts the page again at `path` (4.2). `tab` is the last
  answer the world's tab was given, none before a tab was pointed. A
  world that fails or stops, a turn bound passed at one
  instant, and a link never printed are `Failed`'s, never a panic;
  `Closed`, `NotBrowsed` and `Asked` are the TUI's (3.3). `walk`, story
  A's, keeps its panics.
- **`Find::text_box(name)`,** in skein's fake person: a node of role
  `TextField` or `TextArea` whose whole name is `name`, ordered with
  `nth` among both, as a browser's `textbox` is.
- **`walk`:** the walker (section 4):

  ```rust
  pub enum Marks { Web(Vec<temper_client_walk_lines::print::Mark>), Tui(Vec<temper_client_shell_tui::journey::Mark>) }
  pub fn walk(walk: &temper_client_walk_lines::walk::Walk) -> Result<Marks, Stopped>;
  pub struct Stopped { pub journey: String, pub line: u32, pub why: String }
  ```

  `Marks` holds the walked journey's marks, of its client
  (`Journey::client`), a TUI mark's `live` none. `Stopped` names the
  chain's journey and the line that stopped, and prints as `JOURNEY
  line N: WHY`.

### 3.3 The TUI on the world

`temper-client-walk-world`'s `tui` module, and `web::Page`'s TUI
methods. The TUI's client is `temper-client-shell-tui`'s shipped host,
given keys and read as frames with no terminal
(`../draft/archive/journey-walks.md`, 4.1), as the client's world drives it
(`temper/tests/client/src/tui.rs`), but against the playground's
service, over HTTP, rather than joined to the engine's root.

- **Its host,** `temper_client_shell_tui::host::Host`, at
  `temper_client_shell_tui::limits::tui()`, outside the world as the
  web host is: driven by its reports, read by its frame, with no `Run`,
  terminal, pseudo-terminal or opener.
- **Its connection,** `tui::Connection`, a process of the world,
  admitted as `Process::Other`: the native client's transport,
  `jig_client_shell_native::Transport`, that the shipped terminal's
  `Shell` holds, at the limits and the mapping the shipped run gives it
  (`temper-client-shell-tui`'s `run`, `/jig` with temper's link page),
  over skein's io as the playground's visitor is
  (`skein_playground::visitor::Visitor`), to `SERVICE`. So its asks and
  its stream are the shipped terminal's HTTP: its credential a bearer,
  no browser marker and no cookie (jig's `protocol/client/default-http.md`).
  It is admitted when a TUI starts, so a walk with no `tui` line runs
  the world it ran before, and a TUI stopped closes it.
- **Its start,** `Report::Start`: `SIZE`; no scheme and no background,
  a terminal that answers neither, so it draws `SCHEME`; `VERSION`;
  the address `Origin::Http(SERVICE)`; not local, not boxed, not in
  front and not set up, with no box log; and the records, the
  credential and the note held from the walk's last TUI, none for its
  first. Its seed is `facts::tui_seed` of the number of TUIs the walk
  started before it; its clocks are the world's.
- **Each duty carried,** as the shipped run carries it, by what a world
  has:

  | Duty | In the world |
  |---|---|
  | `asked`: an exchange, `OpenStream`, `CloseStream` | to the connection, between the world's steps; its events back to the host as `Report::Heard`, in order |
  | `asked`: `Save`, `Erase` | the records held by key for the walk, and answered as the shipped shell answers a stored batch |
  | `keep` | the credential held, then `Report::Kept { written: true }` |
  | `note`, `address` | held |
  | `open` | held as the address the TUI last asked to browse, then `Report::Opened { opened: true }`: the walk's browser is there, and `tui approve` opens it |
  | `local` | `Report::Local { link: None }`: a world has no box's link file |
  | `close` | the TUI closed: every later TUI line fails `Failed::Closed` until a `tui stop` |
  | `stop`, `lend`, `unlend` | `Failed::Asked`, naming it: a world plays no box and lends no terminal |

  The world keeps no file (`../draft/archive/journey-walks.md`, 4.1), so what
  the TUI keeps lives as long as the walk, as the client's world keeps
  it (`temper/tests/client/src/tui.rs`).
- **Its deadlines:** `Host::deadline` joins the turn loop's wakes, and
  `Report::Due` is given at it, on the world's time.
- **One loop for both clients:** each turn of the page's world turns
  both hosts: the world sliced, the page's inbox carried to the web
  host, the connection's events to the TUI's host, each host's duties
  carried; the next wake is the earliest of the world's next due, both
  hosts' deadlines and the bound. A browser line waits on the drawn
  tree and a TUI line on the TUI's frame, and both clients go on
  meanwhile, as a person's browser and terminal do.
- **On `web::Page`:**

  ```rust
  pub fn tui(&mut self) -> Option<&mut tui::Terminal>;
  pub fn tui_start(&mut self) -> Result<(), Failed>;
  pub fn tui_key(&mut self, key: skein_ui_screen_terminal::Key) -> Result<(), Failed>;
  pub fn tui_wait(&mut self, text: &[u8], within: Duration) -> Result<(), Failed>;
  pub fn tui_approve(&mut self, within: Duration) -> Result<(), Failed>;
  pub fn tui_stop(&mut self);
  ```

  `tui_start` starts a TUI where none runs, its connection admitted,
  its start given and a turn taken, and is nothing where one runs;
  `tui_key`, `tui_wait` and `tui_approve` start one first. `tui_key`
  gives the host `Report::Key` and turns. `tui_wait` steps until the
  frame's text (`Frame::text`) holds `text`, past `within`
  `Failed::Bound`. `tui_approve` is 4.2's. `tui_stop` closes the
  connection and drops the host, keeping what it held, and is nothing
  where none runs.
- **`tui::Terminal`,** the running TUI:

  ```rust
  pub fn frame(&mut self) -> &skein_ui_screen::Frame;   // the host's frame, as drawn now
  pub fn browsed(&self) -> Option<&[u8]>;              // the address it last asked to browse
  pub fn event_of(&self, key: &skein_ui_screen_terminal::Key) -> Option<temper_client_tui::Event>;
  ```

- **The composition's harness seam:** `temper_playground::story` gains

  ```rust
  pub fn world_with<H: Harness, N: Meter>(start: Start, meter: N) -> World<Process<H>, Machine, N>;
  pub fn slices_with<H: Harness>(start: Start, budget: u32, browser: Time) -> Slices<Process<H>, Machine>;
  ```

  the composition as `world_metered` and `slices` make it, for a
  harness of the caller's, its processes and their order unchanged;
  `world`, `world_metered` and `slices` stay, at `()`.

## 4. A walk in a world

### 4.1 The world and its facts

- **One process and nothing else:** no child, socket, clock,
  environment or network; the walk reads only its chain's files.
- **What runs** is the playground's composition, as story A starts it
  (`../playground.md`, 3), at `facts::start`, with the chain's joined
  scripts where any journey of it holds a `scripts.json`, under the
  harness `Clients`, with the box's peer at the service's box pipe
  (4.4).
- **The forge at `FORGE`,** `127.0.0.1:10002`, the address a person's
  pages drew from a box (`../draft/archive/journey-walks.md`, 4.1): the
  composition started at a forge address of its caller's
  (`../playground.md`, 4), which every part that names the forge takes
  from that one address: the fake forge's listener, the forges' runtime
  destination, the place chooser's port for every setup file and the
  handover `story::prepare` lays, the hosted git's base, and the forge's
  authority pattern. So a page that draws the forge draws
  `127.0.0.1:10002`, as `project-landing`'s does. Story A's own start,
  the page's and its stories', keeps `documents::FORGE`.
- **The model's plan as a box's fake backends give it**
  (`../protocol/setup/box.md`, 5): the deployment's providers made by
  `temper_fake::backends::providers`, provider 1's sign-in registered
  against the fake issuer at `ISSUER` and its callback at
  `127.0.0.1:1455`, `auth/callback`, every model endpoint at the world's
  fake model; the fake issuer, `temper_fake::oauth`'s document at
  `ISSUER` with that callback, at `SEED`, as a box's `fake oauth --seed
  7`, a host of the world's; and the service given its accounts'
  runtime. So the bearer the issuer grants is one the world's model
  takes. Story A's own start holds no issuer and its own providers.
- **The forge's sign-in through temper, as a box's:** the engine's
  OpenID provider as `temper-box init --box` writes it
  (`../protocol/setup/place.md`, 6, *The box's provider*): its signing
  key, `provider.key` in the data directory, made from `SEED` on the
  world's disk, private; no client; `addresses` `[1]`. The owned forge's
  `SetUp` adds the client `forgejo` and the setup file's `sign_in` names
  it, as on a box, and the peer says the group's `source` step (4.4). So
  the engine serves the forge's `temper-forge-sign-in` fact, and a page
  that draws the forge's repository draws its link, `work/hello ↗` to
  `http://127.0.0.1:10002/user/oauth2/temper?redirect_to=%2Fwork%2Fhello`,
  as `project-landing`'s does. The fact reaches the page as a browser's
  does, in the page the clients' listener serves: the walk's start
  adopts a browser bundle of one HTML page and no other file
  (`web/README.md`, 2.2), in memory, so the engine serves that page
  with its forges' metas, and `web::Page`, at its start and at each
  reload, fetches the page at the path it starts at and gives the host
  each `temper-forge-sign-in` meta's content, read by
  `temper_client_view::forge_sign_in::Forges::read` as the web shell's
  start reads them. The native host draws the page; the bundle's page
  is read only for its metas. Story A's own start holds no provider and
  no bundle.
- **A box's bounds:** where story A's configuration
  (`temper_playground::documents::configuration::story_a`) holds a bound
  tighter than a box's, and a step a box takes is refused by it, the
  walk's start takes the deployment's default for that bound, as a
  box's deployment does: `tasks.spec_bytes`, from 187 to 131072 bytes,
  for the goal's specification from a draft; and
  `people.amendment_bytes`, from 1218 to 135168 bytes, for that
  specification with its authority in the person's create request.
  Story A's own start keeps its bounds.
- **The browser starts** on the page the printed link opens, signed
  in, at `/welcome`. Nothing else is implied: the walk's first line
  acts on that page.
- **The TUI starts** at the walk's first `tui` line, not signed in, at
  3.3's start: 80 columns by 24 rows, its build `0.0.0
  (000000000000)`, the service's origin, a terminal that answers no
  scheme. A walk that needs it signed in says `tui approve`.
- **The same bytes every time:** the simulator owns the schedule. The
  browser's first start takes `SEED`; each reload takes the next seed
  from a `skein_lib::Rng` started at `SEED`, as a browser load supplies
  fresh entropy, so its request keys name new acts, not an earlier
  load's. Each TUI takes `facts::tui_seed`, and both clients the world's
  clocks, so a walk takes the same turns on any host, on any day.

### 4.2 Each line

A browser line's bound is the drive session's figure (`drive.md`, 3.2),
60 s, and a TUI line's its own (`drive.md`, 3.3), 30 s, in the world's
time.

- **`web press`, `submit`, `toggle`:** `wait` for the control, by its
  role, its whole name and its order from 0 (`Find::exactly_named`,
  then `nth`; `Find::text_box` for `textbox`), with the line's act;
  then `act` on it.
- **`web input`:** the same, its act `Act::Input` with VALUE.
- **`web see TEXT`:** `wait` for `Find::text(TEXT)`, no act.
- **`web reload PATH`:** the page started again at PATH: the world, its
  cookies and the page's small store kept; a new host and its page,
  with the sign-in ways the start had and the store's records, no
  secret, and the served page's forge metas and a fresh seed from
  4.1's sequence, as a browser's reload builds them
  (`temper-client-shell-web`'s start); then `wait` for the main region.
- **`web mark`:** in the walked journey, a mark: the page's HTML, its
  heading (`print::heading`), and the tree kept for the link (2.4); in
  an ancestor, nothing.
- **`tui key KEY`:** `tui_key`, the key given as a terminal gives it;
  no bound.
- **`tui see TEXT`:** `tui_wait` for TEXT, 30 s.
- **`tui stop`:** `tui_stop`. The next `tui` line starts a TUI again
  from what the last one held, its records, its credential and its
  note, as a person's next `temper` starts from its files.
- **`tui approve`:** `tui_approve`, within 30 s for the whole line:
  wait until the TUI has asked to browse an address on the service's
  origin, `http://SERVICE`, which may hold already; the page reloaded
  at that address's path and query, as `web reload` does; button
  "Approve" found and pressed, as `web press` does; then the page
  reloaded at the address it showed when the line began
  (`Page::address`). The approval reaches the TUI by the TUI's own
  asks, which its next `tui see` waits for.
- **`tui mark`:** in the walked journey, a mark: the frame as drawn, and
  its key (2.4); in an ancestor, nothing.

### 4.3 A stop

A line that does not hold stops the walk: `Stopped`, naming the chain's
journey, the line's number and what it sought, as `button "Try the
playground" not found within 60 s`, `"then press Approve." not seen in
the TUI within 30 s`, `the TUI asked to browse no address on the
service within 30 s`, `the TUI closed`, `the TUI asked to lend its
terminal, which a world does not play` or `the world failed: WORDS`.
Its marks print nothing. A chain that does not read stops its walk as
well, with `Unreadable`'s journey, line and why.

### 4.4 The box's peer

The world's `temper box`, as far as the service meets it: the box's
pipe (`../protocol/setup/box.md`, 1), played by `box_peer::BoxPeer`, a
host of the world's, on the world's time. Nothing else of the box runs:
no group's steps, no listener of its own, no `box.log`, no record under
`box/`.

- **The pipe:** the service adopts the box's two pipes at its start,
  `temper_service::Service::adopt_box`, within the 256 bytes a line the
  composition gives the box's lines, their other ends the peer's, as
  `temper box` gives serve its pipes at each spawn. The one-process
  world's box, `temper/tests/one-process/src/ready_copies.rs`, is the
  service's parent and answers a start alone; the peer starts from it.
- **Each line read whole** to its line feed, one ask at a time, and
  answered as the box answers it (`../protocol/setup/box.md`, 1, the
  table and the paragraph after it):
  - **`start N` for an owned group,** its setup file of the owned kind
    on the world's disk (`temper_forge_setup::setup_file::path`): the
    script below, from the turn the line is read;
  - **`start N` for the playground's group,** of the fake kind: no
    answer. The composition models that group standing from its start,
    its handover laid before the service starts
    (`temper_playground::story::prepare`), and its forge connects by the
    engine's own read of it, as story A's does with no pipe. A `started`
    would start the engine's handover again, or stop it as busy, so the
    peer leaves the ask open, as the composition leaves it today;
  - **`start N` for a group whose script has ended:** `started N`; while
    a script runs, for another forge, `failed N busy`;
  - **`stop N`** and **`remove N`:** `stopped N` and `removed N`, at once;
  - **any other line,** or one past 256 bytes: ignored, as the box
    ignores one.
- **An owned group's script,** `OWNED`: each line at its time after the
  `start` was read, in the world's time. Its figures are Forgejo's 100 MB
  and its runner's 20 MB, round sizes that draw a percentage; two
  seconds between lines, so that the page draws each line's state
  before the next arrives. What the client draws of each is
  `forge/deployment.md`'s, *Setting up, in the row*, and its table's
  *setting up* row:

  | After | Line |
  |---|---|
  | 0 s | `step N fetch` |
  | 2 s | `fetched N forgejo 25000000 100000000` |
  | 4 s | `fetched N forgejo 100000000 100000000` |
  | 6 s | `fetched N runner 10000000 20000000` |
  | 8 s | `fetched N runner 20000000 20000000` |
  | 10 s | `step N prepare` |
  | 12 s | `step N run` |
  | 14 s | the handover laid, then `started N` |

  A group whose setup file names a `sign_in` (4.1) takes the box's
  `source` step after `run`: `step N source` at 14 s, `source N added`
  at 16 s, then the handover and `started N` at 18 s
  (`../protocol/setup/box.md`, 1). A later `start N` for that group says
  `source N added` again before its `started N`.

- **The handover:** at `started N`'s time, before the line, the peer lays
  what the box's preparation leaves the engine
  (`../protocol/setup/owned.md`, 5): `admin_token` in the group's
  directory, as its setup file's entry names it
  (`../protocol/setup/place.md`, 3), on the world's disk, private, holding
  the admin token the world's fake forge takes, as `story::prepare` lays
  the playground's. The forge is the composition's, at its own address,
  where the composition's place chooser puts every setup file's port
  (`temper_playground::documents::FORGE`). The engine then takes the
  handover, reconciles, connects as temper and makes the first
  repository, as with a box's Forgejo.
- **No step through ctl:** the world holds no ctl. An owned set-up goes
  on by the walk's acts, the peer's lines and the engine's own steps
  alone, and the client draws each as it draws a box's.
- **The same bytes every time:** the script's times are the world's, so
  a walk meets each line at the same turn on any host.

### 4.5 A sign-in's tab

The tab a model's sign-in opens (`../draft/sign-in-new-tab.md`, 2),
carried as a browser carries it, so that an account is connected through
the client (`../draft/archive/journey-walks.md`, 4.1).

- **The page's tab duties,** `temper-client-shell-web`'s `Duties::tab`,
  are carried in the turn that gives them, before its transport, as the
  shipped browser carries them: `TabAct::Open` opens the world's tab,
  which a world never refuses, so the host is given no
  `Report::TabRefused`; `TabAct::Point { to }` sends it to `to`;
  `TabAct::Close` closes it, any visit under way abandoned. A tab opened
  again replaces the last.
- **The world's tab,** `skein_playground::tab::Tab`, skein's (skein's
  `playground.md`, 4), a host of the world's: a `GET` of the address on
  a connection of its own, its `Host` the address's authority, with no
  cookie and no `Origin`, as a new tab whose opener was cut sends it; a
  redirect, 301, 302, 303, 307 or 308, followed to its `Location`, at
  most 8; the last answer kept as `tab::Shown`, its address, status and
  body up to the tab's bound. An answer refused, reset or past its
  bound, or a ninth redirect, is kept as the last. The tab draws
  nothing a mark prints.
- **The way there and back:** the fake issuer grants at once
  (`../protocol/setup/box.md`, 5) and redirects to the engine's
  callback, `http://127.0.0.1:1455/auth/callback?code=…&state=…`; the
  engine's listener answers "Sign-in received. You can close this
  tab." and exchanges the code with the issuer; the page, still on its
  address, moves on by its `accounts` watch, to Ready on the welcome
  (`../draft/sign-in-new-tab.md`, 3). That is the page the sign-in
  returns to, its account connected through the client.
- **A tab that does not get there** leaves the page waiting, as a
  person's would: the next line that needs the account stops the walk
  by its bound (4.3).
- **The same bytes every time:** the sign-in's state, verifier and code
  come from the world's seeds, and the tab moves on the world's time.

## 5. The program

`temper-client-walk`, `temper/bin/temper-client-walk` and its crate
`temper/crates/temper-client-walk-tool`, laid out as every binary of
temper's (`../testing/tools.md`, 3). It links `temper-client-walk-lines`
and `temper-client-walk-world`, and nothing of the box, the browser kit or
the drive session: it starts no process, as the gallery's print starts
none (`web/gallery.md`, 1). It reads the journeys of the checkout it
is built from, as `temper-client-journeys` does, under both roots.

### 5.1 `print`

`temper-client-walk print [--out DIR] [NAME]...`:

- **Which:** each NAME, in the order given, or every journey, the
  browser's root's in name order, then the TUI's. A NAME is looked up
  as `walk::read` looks it up, the browser's root first; a NAME that is
  not a directory with a version-2 `walk` is its usage.
- **Where:** DIR, made where missing; with no `--out`, a new directory
  under the platform's temporary directory, `temper-client-walk-`
  then a suffix of its own. A DIR inside either journeys root of the
  checkout is its usage: the pages there are the writer's.
- **Laid out as the tree,** from the repository's root down: a browser
  journey's pages at `DIR/temper/docs/design/client/web/journeys/NAME/`,
  replaced whole, and in that root the files `print::beside` writes,
  with each stylesheet of `print::STYLESHEETS` that the checkout holds,
  and each file it names by a relative `url()`, copied to its path; a
  TUI journey's pages, each mark's `NN.txt` and `NN.html`, at
  `DIR/temper/docs/design/client/tui/journeys/NAME/`, replaced whole,
  and in that root the files `tui_print::beside` writes, with the
  checkout's `overlay.css` copied to its path. A root's files go only
  where a journey of it printed. So a page opens from DIR as from the
  checkout, and its bytes are those `pages` would write.

### 5.2 `pages`

`temper-client-walk pages`, the writer's (`journey-writer.md`, 4):

- **Each journey** of both roots, the browser's first, each in
  name order, walked.
- **Printed in place:** a walk that reached its last line writes its
  journey's pages, a file only where its bytes differ, removes a
  numbered page past its count, an `NN.txt` as an `NN.html`, and
  removes its `recording` where one is left. The files `beside` and
  `tui_print::beside` write go only where they differ; an `overlay.css`
  is made empty where missing and never written otherwise.
- **A walk that stops** leaves every file of its journey as it was.
- **A directory with no `walk` loses its numbered pages,** keeping its
  other files. Nothing else is written (`journey-writer.md`, 2).

### 5.3 Its lines and exit

- **Stdout, a line a journey,** in the order walked: `printed NAME`,
  `fresh NAME` (`pages` alone: no byte differed), or `stopped NAME:
  WHY`, WHY section 4.3's; then, for `print`, a last line, `printed
  into DIR`. Every other word goes to stderr.
- **Its exit:** 0, every walk printed; 1, a walk stopped, once the
  others are printed; 2, its usage; 3, the print could not finish, a
  root unreadable or DIR unwritable, the pages as it left them.

## 6. The journeys' walks

- **Sources and pages:** each journey has a `walk` of version 2 and a
  `README.md` saying what the world draws, with `scripts.json` where
  it needs the fake model's turns. A change edits those sources as a
  test, committing no page or recording (`journey-writer.md`, 5). The
  writer prints the pages after landing, removing leftover recordings
  and surplus pages once the walk completes.
- **A request** is compared by a print of every journey into scratch,
  `temper-client-walk print --out DIR`, DIR's pages against the
  request's commit's, equal where their facts agree
  (`web/look-requests.md`, 3).
- **`welcome-playground`:** the welcome seen and marked, "Try the
  playground" pressed, the playground's New chat seen and marked.
  Two pages.
- **`sign-in-approved`:** the TUI's approval frame seen by "then press
  Approve." and marked, `tui approve`, the launchpad seen by "Setting
  temper up makes your first project" and marked: two pages, on the
  playground's service.
- **`welcome-set-up`,** on the box's peer (4.4): the welcome seen and
  marked; "Set up temper" pressed, the Forge step's choices seen and
  marked; Forgejo's "Run on this machine" pressed, its sheet seen,
  an empty repository chosen as it defaults, `work/hello`, and marked;
  the sheet's "Run on this machine" pressed; then the Model step marked
  at each of the box's steps as the peer reports it, fetching at its
  first figure, preparing and starting; and marked once its pane says
  the forge is connected as temper and `work/hello` made. Seven pages.
- **`project-landing`,** on the forge at `FORGE`, the box's peer and a
  sign-in's tab (4.1, 4.4 and 4.5): Forgejo run on this machine with
  its sample `work/hello`, "Continue with ChatGPT" pressed and its tab
  carried to the issuer and back, "Create your project" pressed once
  Ready offers it, Home's "work/hello is in Hello" seen, `/projects/1`
  reloaded, and Home marked once it says "Hello is ready.", the words
  a fresh project's Home draws while To begin shows. One page.
- **`chat-draft`,** `after project-landing`, with its `scripts.json`:
  eight pages, a chat through its draft to a goal and back. Its scripts
  are the fake model's turns for its chat, its draft's two revisions
  and its goal's report, with the world's task and note numbers; its
  goal starts in the world on the bounds a box holds (4.1).
- **The TUI's longer journeys** are each `after project-landing`, with
  their own `scripts.json` with the world's task and note numbers.
  Each walk prints exactly its marks, and a mark's key
  links its page to the next (2.4).
  - **`launchpad-chat`:** the approval walked by `tui approve`; Hello's
    launchpad seen by "Ask temper" and marked; `/` typed, the commands
    seen and marked, Escape its key; "What can you tell me about
    Hello?" typed and marked, Enter its key; the Ask temper chat seen by
    its scripted answer, "Hello from the local fake model.", and marked.
    Four pages. Its scripts are that answer alone, cued by the question.
  - **`launchpad-drawn`:** `/change Add a CSV export to the report`
    typed and marked, Enter its key; the chat's reply seen, `/new` typed
    and marked, Enter its key; the launchpad with the chat's question
    under Needs you seen and marked, `n` its key; the walk's prompt seen
    and marked, Enter, taking Comma, its key; the launchpad marked once
    it says "Nothing else needs you.". Five pages. Its scripts are the
    chat's reply starting a goal, and that goal's question to the chat,
    its wait and its finish once answered.
  - **`design-session`:** Hello's launchpad marked; `/draft Plan a
    farewell line for hello.` typed and marked, Enter its key; the
    session marked at half, at all and folded, each linked by Ctrl-P
    (`tui/pages/design-session.md`, 1); half with "1" typed and marked,
    Enter its key; the reply saying nothing blocks the work marked;
    `/goal` typed and marked, Enter its key; its ways' prompt marked,
    Enter, taking Task by task, its key; the returned chat marked with
    its goal linked, once `/goal`, typed and cleared, is no longer
    offered. Ten pages. Its scripts are the drafter's turns, cued by the
    opening words and by the answer, and the started goal's report and
    finish.

## 7. Its tests

Focused, and native: no box, browser or terminal. A walk runs only in
its own process.

- **`temper-client-walk-lines`,** step, in its own tests: each line read
  back, `web mark`, `tui mark` and `tui approve` among them; a walk's
  `tui key`, `tui see`, `tui stop`, `tui approve` and `tui mark` read,
  and its lines refused by name, a `ctl` line and `tui look` among them;
  a mark of the other client's refused, in the walked journey and in an
  ancestor; `client` set by the root; `after` joining a chain root
  first, and a cycle, a ninth ancestor, a missing journey and one not
  of version 2 refused; `moved` on `walk 2`, `walk 1` and no `walk`;
  the printers' tests as they were.
- **The drive session and the recipes,** in the journeys tool's own
  tests: `web mark`, `tui mark` and `tui approve` refused in a recipe,
  naming its line; outside a journey's session, the drive session
  refusing `tui mark` and `tui approve`, keeping neither and starting
  no TUI; a recipe's walk signing its TUI in. A journey's session
  starts without an implied root load or TUI approval; marks answer
  looks without being kept, and `tui approve` signs the TUI in.
  `journey walk NAME` walks a chain root first in one session, with
  its scripts, without ancestors' marks or added resets; its first
  failure names the journey and file line, and stops it.
- **The driver,** simulated, beside the playground's stories: a control
  that appears late found within its bound; one that never does
  `Failed::Bound` at the bound's end in the world's time; a reload that
  keeps the small store and the ways; a text area found as a text box;
  the HTML holding what the tree draws; story A walked as before. For
  the TUI: started, its approval frame drawn from the service, its
  browse held; `tui_approve` approving what it browsed, the TUI then
  drawing its launchpad; a TUI not approved staying on its approval,
  `Failed::Bound` at the bound's end; a chat's reply streamed from the
  service and drawn; a TUI stopped and started again starting signed
  in from what it kept; the same start and keys drawing the same
  frames.
- **The box's peer,** simulated, beside the driver's: an owned set-up
  asked from the welcome, each of `OWNED`'s steps drawn in order, under
  way from its line's time and before the next line's, the steps before
  it done, then the forge connected as temper and `work/hello` made,
  with nothing in the world but the composition and the peer; the
  playground tried as before, its group's start left open. The peer's
  answers, step, where they are a function of its own: an owned start,
  a fake one, a start after the script and one during it, `stop`,
  `remove`, and a line past 256 bytes.
- **The forge at `FORGE`,** simulated: a walk's world with its fake
  forge listening at `127.0.0.1:10002`, an owned set-up's forge
  connected there and `work/hello` made, as the box's peer's test walks
  it; story A's own start with its forge at `documents::FORGE`, as
  before.
- **A sign-in's tab:** the world's tab, simulated, in skein's own
  tests: a `GET` answered, a redirect followed to its last answer, a
  ninth redirect kept as the last, an answer past its bound kept as
  failed, no cookie or `Origin` sent. In a world, simulated, beside the
  box's peer's: a walk of the tests' own from the welcome through
  Forgejo run on this machine and "Continue with ChatGPT" reaching
  Ready, its "Create your project" offered, the ChatGPT plan connected
  through the client, the tab's last answer the listener's "Sign-in
  received. You can close this tab.", printed twice, the same bytes.
- **Scripts along a chain,** simulated: two journeys of the tests' own,
  each with a `scripts.json`, the walked one `after` the other, a chat
  answered from each one's script; a cue in both stopping the walk by
  the journey whose scripts did not join.
- **A goal from a draft,** simulated: a walk of the tests' own in the
  playground's project, its own `scripts.json`: a chat, Split, "Write it
  up from this chat" and its scripted document, "Start goal…", the
  sheet's "Start goal", the goal's page with its report and its
  "Draft" link, and that link back to the chat. In an owned project,
  another walk sets up Hello on `work/hello`, signs in with ChatGPT,
  reloads Home and Chats, then starts a chat answered from an `asked`
  cue. Its new chat waits for the repository's line before marking;
  its second turn revises the draft, its goal prints a report and
  lands, and its Draft link returns to both replies. Printed twice,
  the same bytes, including after those reloads.
- **The TUI's longer walks,** simulated, in the program's suite, each a
  TUI journey of the tests' own after a walk of its own that reaches a
  project, with its own `scripts.json`: a `/change` whose chat starts a
  goal that asks the person, walked by `n` and answered by Enter, the
  launchpad then needing nothing, its marks linked by `n` and Enter;
  and a `/draft` session at half, all, folded and half again, each of
  the first three marks linked by Ctrl-P. Each printed twice, the same
  bytes.
- **Each journey's scripts read:** every journey of the
  checkout read by `walk::read`, its chain's scripts joined as the
  walker joins them, with no world started: its sources read, never a
  page of main's or a walk of it.
- **The walker and the program,** simulated, on journeys of the tests'
  own in scratch roots: walked and printed; printed twice, the same
  bytes; a control that never appears stopping the walk by its journey
  and line, the others still printed, exit 1; the acted control
  linked, by act, and one that typing revealed not linked; a walk
  through an owned set-up, marked while a step is under way and at its
  end, printed twice, the same bytes; an
  ancestor's marks printing nothing; a page opening from the scratch
  directory with its stylesheets at their paths; fewer marks leaving
  fewer files; `pages` on a tree whose pages are current writing
  nothing, and with one word of a page changed by hand writing that
  page alone; a directory without a version-2 `walk` left as it was.
  For the TUI: a TUI journey walked and printed, `NN.txt` and `NN.html`
  in DIR's TUI root with `theme.css`, `tui-journey.js` and the
  overlay; a mark's key linking, and a typed key or a `tui approve`
  between them not linking; a TUI line that never holds stopping the
  walk by its journey and line; `pages` removing an `NN.txt` past the
  count.
- **The binary's dependencies:** `fabrig-boundary` holds that
  `temper-client-walk`'s normal dependencies name no `skein-browser`,
  `skein-fake-person-browser` or `temper-client-journeys-tool`.
- **The writer's run,** in the journeys tool's suite
  (`journey-writer.md`, 7).
- **No test walks main's journeys** or compares a page of main's
  (`../draft/archive/journey-walks.md`, 10). Each test sits in its layer within
  that layer's budget as it stands; the cost of a journey's walk in a
  world is kept in `../../development/limits-worlds.md`.

## 8. Later

- **A forge's departures** carried by the world's tab as a model's
  sign-in is, once a journey walks GitHub's App or install
  (`../draft/sign-in-new-tab.md`, 6).
- **The playground's group answered** as the box answers it, once the
  composition no longer lays that group's handover before its start
  (4.4).
- **A journey after a recipe,** so that a start QA already keeps need
  not be walked through the client too (`recipe.md`, 3.2).
- **The gallery's states as walks,** once a world reaches them, their
  answers then the engine's own (`web/gallery.md`).
- **A recorded file printed across layouts,** by the plans of
  `../draft/layouts-both-ways.md`, 4, so that printing a person's
  recording needs no build of the deployment's commit.
- **The box's `mark` compared with the world's page,** by their
  accessible text, as a check that a world draws what a box does.
- **The writer's future** is `journey-writer.md`, 8: temper running it,
  and the playground's `story-a.log` made by a test rather than kept.
