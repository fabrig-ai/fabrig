# Journeys walked in a world

Provisional, 2026-11-19. A browser journey printed in one process, with
no box, browser, recording or network: `../draft/journey-walks.md`,
sections 3, 4.1, 5 and 6, with 8.3 for how a journey moves. A journey
whose `walk` is of version 2 is walked here; every other journey is
still a recording, replayed and walked again as `web/journeys.md` and
`tui/journeys.md` have it, and the writer takes either engine
(`journey-writer.md`, 3).

## 1. In one page

- **A journey is its walk** (section 2): a `walk` file of version 2,
  the drive session's lines with a client named on each, `web mark`
  where a page is printed, and `after NAME` for the journey it starts
  from.
- **Two libraries** (section 3): `temper-client-walk-lines`, the
  lines' grammar, the walk's reader and the two page printers, free of
  the browser kit; and `temper-client-walk-world`, the web client's driver
  on the playground's composition and the walker.
- **A walk in a world** (section 4): the shipped web host against the
  playground's composition on skein's simulator, at fixed facts, each
  line acted as a person acts and each mark printed as a page, the same
  bytes from the same tree every time.
- **The box's peer** (4.4): the world's `temper box`, at the service's
  box pipe, reporting an owned Forgejo's set-up step by step on the
  world's time, as the box reports it, with no step taken through ctl.
- **The program** (section 5): `temper-client-walk print [--out DIR]
  [NAME]...` prints into scratch, laid out as the tree with its
  stylesheets, and `temper-client-walk pages` writes main's pages, for
  the writer alone.
- **A journey moves** (section 6) in a change that gives it a `walk` of
  version 2 and takes its `setup` out; the writer's next run prints it
  in the world and removes its `recording`. `welcome-playground` moves
  first, `welcome-set-up` second.

## 2. The walk

### 2.1 Its directory

A moved journey is a directory under `web/journeys/` whose `walk`'s
first line is exactly `walk 2`, with:

- **`walk`:** 2.2's text, and **`README.md`:** what it walks and what
  each page shows. Sources, any change's.
- **`scripts.json`,** where it has one: the fake model's scripts for its
  steps, in the box's `--scripts` form (`recipe.md`, 2). A source.
- **Its pages,** `01.html` onwards, and the files beside the root:
  the writer's alone (`journey-writer.md`, 2).
- **No `setup`.** A `recording` left from before it moved is the
  writer's to remove (section 6).

### 2.2 Its lines

Text, one line each, in the drive session's form (`drive.md`, 3.1):

| Line | What it is |
|---|---|
| `walk 2` | the first line |
| `after NAME` | the second line, or absent (2.3); NAME a journey's name, bare |
| `web press ROLE NAME [ORDER]`, `web submit ...`, `web toggle ...`, `web input ROLE NAME VALUE [ORDER]`, `web see TEXT`, `web reload PATH` | the drive session's browser lines, read by its grammar (`drive.md`, 3.2) |
| `web mark` | prints the page as drawn then (2.4) |

- **Read by the drive session's grammar,** which gains `web mark`. A
  walk refuses every other line: `ctl`, `forge`, `deployment`, `end`,
  `web look` and `web requests` by name, as a box's or a session's, and
  a `tui` line, which no world walks yet (section 8). An empty line
  is skipped and keeps its number.
- **`web mark` is a journey's line alone:** a recipe refuses it as it
  reads it, and the drive session answers it `failed: line N: web mark
  is a journey's line` (`drive.md`, 3.7).
- **A line's number** is its line in its file, from 1, `walk 2` the
  first.

### 2.3 `after`

- **`after NAME`** names a moved journey under either root. Its lines
  run first, its own `after` before them, and its marks print nothing.
- **Its chain** is the journey and at most eight ancestors, root first,
  refused as a recipe's is (`recipe.md`, 3.2): a ninth ancestor, a
  cycle and a missing journey each refused, naming the journey whose
  `after` names it; so is a NAME whose `walk` is not of version 2.
- **`scripts.json`** files join along the chain, root first, as a
  recipe's do (`recipe.md`, 2), by `temper_fake::llm::ScriptJoin`.

### 2.4 Marks and links

- **A mark prints one page:** the page as drawn when the line is
  reached, every line before it having held, numbered from `01` in the
  walked journey's marks' order. Typing marks nothing, and a walk
  prints exactly its marks.
- **Its link:** the first `web press`, `submit` or `toggle` after a
  mark and before the next one links the marked page to the next page,
  by its act, where its control is on the marked page: the line's
  control sought on the tree as it was marked, by the line's role, name
  and order. A control that typing or another act revealed is not on
  it, and that page leads on by its neighbours, as the last page does.
- **The printed form** is the browser's (`web/journeys.md`, 5.2 and
  5.3): `print::pages` over each mark's HTML, the main region's
  level-1 heading, its acted control and its lead.

## 3. The libraries

### 3.1 `temper-client-walk-lines`

`temper/crates/temper-client-walk-lines`, the `harness` profile, out
of `temper-client-journeys-tool`, whose drive session, recipes and
reprint read through it. It names no `skein-browser`,
`skein-fake-person-browser` or box crate.

- **`line`:** the drive session's lines, `Line`, `Record` and the rest,
  moved as they are, with `Web::Mark`.
- **`walk`:** a walk's reader (2.2, 2.3):

  ```rust
  pub struct Roots { pub web: PathBuf, pub tui: PathBuf }
  pub struct Walk { pub chain: Vec<Journey> }      // root first, the walked journey last
  pub struct Journey { pub name: String, pub lines: Vec<Numbered>, pub scripts: Option<Box<[u8]>> }
  pub struct Numbered { pub number: u32, pub line: Line }
  pub struct Unreadable { pub journey: String, pub line: u32, pub why: String }
  pub fn read(roots: &Roots, name: &str) -> Result<Walk, Unreadable>;
  pub fn moved(directory: &Path) -> std::io::Result<bool>;
  ```

  `moved` is true where `directory/walk` exists and its first line is
  exactly `walk 2`, false where there is no `walk` or its first line is
  another; the writer's engines part on it (section 6). `lines` hold
  every line after the header and the `after`, empty ones left out.
  `Unreadable`'s `line` is 0 for the file as a whole.
- **`print`:** the browser's pages (`web/journeys.md`, 5.2 and 5.3),
  `Mark`, `Printed`, `pages`, `beside` and `SCRIPT`, moved, with
  `pub fn heading(tree: &skein_ui::Tree) -> Vec<u8>`, the main region's
  level-1 heading, and `pub const STYLESHEETS: [&str; 4]`, the
  stylesheets a page links, relative to its journey's directory, which
  `pages` writes from.
- **`tui_print`:** the TUI's pages (`tui/journeys.md`, 6), `pages`,
  `beside`, `stylesheet` and `SCRIPT`, moved.

### 3.2 `temper-client-walk-world`

`temper/crates/temper-client-walk-world`, the `harness` profile: the
playground's native page, out of `temper/tests/playground`, whose
stories keep using it, and the walker beside it. It links the
playground's composition and the web shell, and names no browser kit,
box crate or `temper-client-journeys-tool`.

- **`facts`:** `SEED: u64 = 7`; `WALL`, 2026-01-06T09:00:00Z; `SERVICE:
  &str = "127.0.0.1:10001"`; and

  ```rust
  pub fn start(scripts: Option<&[u8]>) -> Result<temper_playground::story::Start, String>;
  ```

  story A's start at `SEED` and `WALL`, its service's clients'
  listener at `SERVICE`, and, where `scripts` is given, the model's
  scripts replaced by them, as `temper_fake::llm::Document::decode_with_scripts`
  replaces them. The forge stays at the composition's address
  (section 8).
- **`Playground`,** the world type: the composition's processes and the
  box's peer (4.4), moved from `temper/tests/playground/src/lib.rs`.
- **`box_peer`:** the box's peer (4.4): `BoxPeer`, a host of the
  world's (skein-shell's `Host`), and `OWNED`, the owned group's script
  as data, a line and its time each, which the tests read.
- **`web::Page`,** the shipped web host on the world (`../playground.md`,
  9), its turn loop as the playground's page has it, with:

  ```rust
  pub fn start(start: temper_playground::story::Start, name: &[u8]) -> Result<Page, Failed>;
  pub fn html(&self) -> String;                  // the page as drawn: skein_ui_web::page::Page::html
  pub fn tree(&self) -> &skein_ui::Tree;
  pub fn wait(&mut self, find: &Find, act: Option<&Act>, within: Duration) -> Result<NodeId, Failed>;
  pub fn act(&mut self, node: NodeId, act: Act) -> Result<(), Failed>;
  pub fn reload(&mut self, path: &[u8], within: Duration) -> Result<(), Failed>;
  pub fn walk(&mut self, person: &mut Person<Address>, bound: Duration) -> Walked;
  pub enum Failed { Bound, World(Box<[u8]>), Stopped, Turns, NoLink }
  ```

  `Find`, `Person` and `Walked`'s steps are skein's fake person's
  (`../testing/fakes.md`, 4); `Act` and `NodeId` skein-ui's; `Duration`
  skein-lib's, in the world's time. `start` makes the world with the
  box's peer at the service's box pipe (4.4), and spends the service's
  printed sign-in link, home `/welcome`, as the page does today.
  `wait` steps the world, the host and the browser's time until `find`
  matches a node of the drawn tree, as `TreeFace` finds it, and with
  `act`, only a node that face would give that act counts; past
  `within` it is `Failed::Bound`, and with nothing due before then the
  browser's time goes to `within`'s end. `act` gives the host
  `Report::Act` and turns. `reload` starts the page again at `path`
  (4.2). A world that fails or stops, a turn bound passed at one
  instant, and a link never printed are `Failed`'s, never a panic:
  `walk`, story A's, keeps its panics.
- **`Find::text_box(name)`,** in skein's fake person: a node of role
  `TextField` or `TextArea` whose whole name is `name`, ordered with
  `nth` among both, as a browser's `textbox` is.
- **`walk`:** the walker (section 4):

  ```rust
  pub fn walk(walk: &temper_client_walk_lines::walk::Walk) -> Result<Vec<temper_client_walk_lines::print::Mark>, Stopped>;
  pub struct Stopped { pub journey: String, pub line: u32, pub why: String }
  ```

  `Stopped` names the chain's journey and the line that stopped, and
  prints as `JOURNEY line N: WHY`.

## 4. A walk in a world

### 4.1 The world and its facts

- **One process and nothing else:** no child, socket, clock,
  environment or network; the walk reads only its chain's files.
- **What runs** is the playground's composition, as story A starts it
  (`../playground.md`, 3), at `facts::start`, with the chain's joined
  scripts where any journey of it holds a `scripts.json`, and the box's
  peer at the service's box pipe (4.4).
- **The browser starts** on the page the printed link opens, signed
  in, at `/welcome`. Nothing else is implied: the walk's first line
  acts on that page.
- **The same bytes every time:** the simulator owns the schedule, and
  the host takes `SEED` and the world's clocks, so a walk takes the
  same turns on any host, on any day.

### 4.2 Each line

Each bound is the drive session's figure (`drive.md`, 3.2), 60 s, in
the world's time.

- **`web press`, `submit`, `toggle`:** `wait` for the control, by its
  role, its whole name and its order from 0 (`Find::exactly_named`,
  then `nth`; `Find::text_box` for `textbox`), with the line's act;
  then `act` on it.
- **`web input`:** the same, its act `Act::Input` with VALUE.
- **`web see TEXT`:** `wait` for `Find::text(TEXT)`, no act.
- **`web reload PATH`:** the page started again at PATH: the world, its
  cookies and the page's small store kept; a new host and its page,
  with the sign-in ways the start had and the store's records, no
  secret, as a browser's reload builds them
  (`temper-client-shell-web`'s start); then `wait` for the main region.
- **`web mark`:** in the walked journey, a mark: the page's HTML, its
  heading (`print::heading`), and the tree kept for the link (2.4); in
  an ancestor, nothing.

### 4.3 A stop

A line that does not hold stops the walk: `Stopped`, naming the chain's
journey, the line's number and what it sought, as `button "Try the
playground" not found within 60 s` or `the world failed: WORDS`. Its
marks print nothing. A chain that does not read stops its walk as
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

## 5. The program

`temper-client-walk`, `temper/bin/temper-client-walk` and its crate
`temper/crates/temper-client-walk-tool`, laid out as every binary of
temper's (`../testing/tools.md`, 3). It links `temper-client-walk-lines`
and `temper-client-walk-world`, and nothing of the box, the browser kit or
the drive session: it starts no process, as the gallery's print starts
none (`web/gallery.md`, 1). It reads the journeys of the checkout it
is built from, as `temper-client-journeys` does, and takes the
browser's root (section 8).

### 5.1 `print`

`temper-client-walk print [--out DIR] [NAME]...`:

- **Which:** each NAME, in the order given, or every moved journey of
  the browser's root, in name order. A NAME that is not a moved journey
  is its usage.
- **Where:** DIR, made where missing; with no `--out`, a new directory
  under the platform's temporary directory, `temper-client-walk-`
  then a suffix of its own. A DIR inside either journeys root of the
  checkout is its usage: the pages there are the writer's.
- **Laid out as the tree,** from the repository's root down: each
  journey's pages at `DIR/temper/docs/design/client/web/journeys/NAME/`,
  replaced whole, and the files `beside` writes in that root; and each
  stylesheet of `print::STYLESHEETS` that the checkout holds, and each
  file it names by a relative `url()`, copied to its path. So a page
  opens from DIR as from the checkout, and its bytes are those `pages`
  would write.

### 5.2 `pages`

`temper-client-walk pages`, the writer's (`journey-writer.md`, 4):

- **Each moved journey** of the browser's root, in name order, walked.
- **Printed in place:** a walk that reached its last line writes its
  journey's pages, a file only where its bytes differ, removes a
  numbered page past its count, and removes its `recording` where one
  is left. The files `beside` writes go only where they differ.
- **A walk that stops** leaves every file of its journey as it was.
- **It writes nothing else:** its other directories are the journeys
  tool's (`journey-writer.md`, 3).

### 5.3 Its lines and exit

- **Stdout, a line a journey,** in the order walked: `printed NAME`,
  `fresh NAME` (`pages` alone: no byte differed), or `stopped NAME:
  WHY`, WHY section 4.3's; then, for `print`, a last line, `printed
  into DIR`. Every other word goes to stderr.
- **Its exit:** 0, every walk printed; 1, a walk stopped, once the
  others are printed; 2, its usage; 3, the print could not finish, a
  root unreadable or DIR unwritable, the pages as it left them.

## 6. A journey moves

- **Its change** gives it a `walk` of version 2 and a `README.md` saying
  what the world draws, and takes its `setup` out where it has one. It
  prints the journey into scratch, `temper-client-walk print NAME`, and
  checks the pages against its README. It commits no page and removes
  no `recording`: the delivery check keeps every change but the
  writer's off them (`journey-writer.md`, 5).
- **The writer** prints it in the world from that landing on, with
  `temper-client-walk pages`, which removes its `recording`; the
  journeys tool passes a moved journey by (`journey-writer.md`, 3).
- **A request** for a moved journey is compared by a print into
  scratch, `temper-client-walk print --out DIR NAME`, DIR's pages
  against the request's commit's, equal where their facts agree
  (`web/look-requests.md`, 3).
- **`welcome-playground` moves first:** its `walk`, the welcome seen and
  marked, "Try the playground" pressed, the playground's New chat seen
  and marked. It has no `setup` on main: its walk was its recording's.
- **`welcome-set-up` moves second,** on the box's peer (4.4): its `walk`,
  the welcome seen and marked; "Set up temper" pressed, the Forge step's
  choices seen and marked; Forgejo's "Run on this machine" pressed, its
  sheet seen, an empty repository chosen as it defaults, `work/hello`,
  and marked; the sheet's "Run on this machine" pressed; then the Model
  step marked at each of the box's steps as the peer reports it,
  fetching at its first figure, preparing and starting; and marked once
  its pane says the forge is connected as temper and `work/hello` made.
  Seven pages, as before. Its `setup` goes; its `recording` is the
  writer's to remove.

## 7. Its tests

Focused, and native: no box, browser or terminal. A walk runs only in
its own process.

- **`temper-client-walk-lines`,** step, in its own tests: each line read
  back, `web mark` among them; a walk's lines refused by name, a `ctl`
  line and a `tui` line among them; `after` joining a chain root first,
  and a cycle, a ninth ancestor, a missing journey and one not of
  version 2 refused; `web mark` refused in a recipe; `moved` on `walk
  2`, `walk 1` and no `walk`; the printers' tests as they were.
- **The driver,** simulated, beside the playground's stories: a control
  that appears late found within its bound; one that never does
  `Failed::Bound` at the bound's end in the world's time; a reload that
  keeps the small store and the ways; a text area found as a text box;
  the HTML holding what the tree draws; story A walked as before.
- **The box's peer,** simulated, beside the driver's: an owned set-up
  asked from the welcome, each of `OWNED`'s steps drawn in order, under
  way from its line's time and before the next line's, the steps before
  it done, then the forge connected as temper and `work/hello` made,
  with nothing in the world but the composition and the peer; the
  playground tried as before, its group's start left open. The peer's
  answers, step, where they are a function of its own: an owned start,
  a fake one, a start after the script and one during it, `stop`,
  `remove`, and a line past 256 bytes.
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
  page alone; a directory with no `walk` of version 2 left as it was.
- **The binary's dependencies:** `fabrig-boundary` holds that
  `temper-client-walk`'s normal dependencies name no `skein-browser`,
  `skein-fake-person-browser` or `temper-client-journeys-tool`.
- **The writer's second engine,** in the journeys tool's suite
  (`journey-writer.md`, 7).
- **No test walks main's journeys** or compares a page of main's
  (`../draft/journey-walks.md`, 10). Each test sits in its layer within
  that layer's budget as it stands; the cost of a journey's walk in a
  world is measured as the program first prints one, and kept in
  `../../development/limits-worlds.md`, with each journey's that moves
  after it.

## 8. Later

- **The TUI's journeys,** with `tui` lines, `tui approve` and `tui
  mark`, and the TUI's root in the program, once the TUI's host runs on
  the simulated service (`../draft/journey-walks.md`, 4.1).
- **The forge at its fixed address,** `127.0.0.1:10002`, which the
  first journey that draws it brings, as `project-landing`'s first page
  does; and **a sign-in's tab** carried to the fake issuer and back
  (`../draft/journey-walks.md`, 4.1).
- **The playground's group answered** as the box answers it, once the
  composition no longer lays that group's handover before its start
  (4.4).
- **The old engine goes** with the last journey that moves: `again`,
  the `setup` reader and the version 1 `walk`
  (`../draft/journey-walks.md`, 8.1).
