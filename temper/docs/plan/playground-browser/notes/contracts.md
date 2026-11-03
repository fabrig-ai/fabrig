# playground-browser: contracts

The run's binding calls for the shapes that cross tasks (`temper/docs/design/playground.md`, 3,
4, 6, 7 and 9, on skein's kit: `skein/docs/design/playground.md`, 3 to 7). A task builds its half
to this note; names inside a module are its implementer's, and its report lists the public ones
its readers use. Sections are added as the plan grows; a section once pushed keeps its shapes.

## C1. The visitor in the composition

skein's visitor is one of the composition's processes (`playground.md`, 3: "skein's visitor"),
on every world the composition makes, so the world a browser steps and the world a native test
steps are one.

```rust
// temper/crates/temper-playground/src/programs.rs
pub enum Process<H: Harness> {
    Service(..), Agent(..), GitRoute(..), Git(..), Shell(..), Peer(..), Forge(..),
    Visitor(Box<skein_playground::visitor::Visitor>),   // new, before `Other`
    Other(H::Process),
}
pub enum Role { Service, Agent, Git, Shell, Provider, Forge, Visitor /* new */, Other }

// temper/crates/temper-playground/src/documents.rs
/// The page's fetch bound: temper-client-shell-web's `limits::EXCHANGES + 1`, the bound its
/// browser shell gives its fetch (held equal by a test in tests/playground, C3).
pub const VISITOR_EXCHANGES: u32 = 9;
/// The visitor of a deployment's document: its listener and Host are the document's
/// `application.clients.listener.socket` (`playground.md`, 4); its bounds the playground's own.
pub fn visitor(document: &serde_json::Value) -> skein_playground::visitor::Config;
```

- **Its place:** `story::world_metered` admits the visitor with `World::spawn` right after the
  two fake peers, before any step, so its index is fixed: `pub const VISITOR: usize = 3` in
  `story` (the service 0, the LLM peer 1, the forge peer 2), asserted where it is admitted.
- **Its reach:** `pub fn reach<H: Harness>() -> skein_playground::slices::Reach<Process<H>>`
  in `story`, `at: VISITOR`, its `enter` and `take` calling the visitor's, any other variant
  `unreachable!`.

## C2. The printed link, the factory parameters and the slices

skein's loops (`Slices::slice`, `replay::replay`) call `World::step` and `World::printed`
themselves, with no hook of the application's (`skein/docs/design/playground.md`, 3 and 6.2).
So:

- **The link is printed to the world's printer:** with `Links::Print`, the service's host writes
  each link and a newline to the `skein_sim_world::Printer` the world made for `temper serve`'s
  standard output (`skein/docs/design/world.md`, 2: the printer is the pipe the world captures),
  so `World::printed` holds it after the step that printed it, whichever loop steps.
  `ServiceHost::capture_output(fd, printer)` becomes `ServiceHost::print_to(printer: Printer)`;
  `story::capture` and the service's simulated standard-output pipe go; `story::step` keeps its
  signature and installs the parameters around `World::step`.
- **The factory parameters around any loop:**

  ```rust
  // temper/crates/temper-playground/src/story.rs
  /// The world's factory parameters, which its hosted programs' factories read while it steps.
  pub fn parameters<H: Harness, N: Meter>(world: &World<Process<H>, Machine, N>) -> programs::Parameters;
  ```

  A native loop over skein's (`Memory::slice`, `replay::replay`) runs inside
  `programs::with(story::parameters(world), || ...)`. The worker installs them for the module's
  life (C4).
- **A story in slices:**

  ```rust
  // temper/crates/temper-playground/src/story.rs
  /// A story's world in skein's slices, its visitor reached, at 1× from `browser`.
  pub fn slices(start: Start, budget: u32, browser: skein_lib::Time)
      -> skein_playground::slices::Slices<Process<()>, Machine>;
  /// A slice's budget in steps (`skein/docs/design/playground.md`, 3): the playground's.
  pub const SLICE_STEPS: u32 = 256;

  // temper/crates/temper-playground/src/stories.rs
  /// The story a start message names (`bridge::Start::story`), by its `name`.
  pub fn named(name: &[u8]) -> Option<&'static Story>;
  ```

  `SLICE_STEPS` is a first figure; the browser tier measures a slice against the bar the user
  sets (`playground.md`, 8), and only a later task changes it.

## C3. The page, natively

The browser's page natively, in `temper-playground-world`'s library (`temper/tests/playground`):
temper's web client host, built natively, on skein's in-memory bridge (`playground.md`, 9;
`skein/docs/design/playground.md`, 5), driven by skein's fake person on the host's tree.

```rust
// temper/tests/playground/src/page.rs
/// What enters the page's inbox: the bridge's raises and the page's own.
pub enum Report {
    Fetch(skein_ui_web::fetch::Report),
    World(skein_ui_web::platform::worker::Heard),
    Carried { reports: Vec<skein_ui_web::transport::Carried>, writes: u32 },
    Due,
}
/// One page's life: the story's world in slices behind `Memory`, the client's host in front.
pub struct Page { /* host, memory, inbox, store, browser instant: the implementer's */ }
impl Page {
    /// As the page's starter does (playground.md, 6): the world started with its visitor, the
    /// printed link awaited among the lines `temper serve` prints, then the client started at
    /// the link's path and query, its secret taken by jig's `sign_in::take`.
    pub fn start(story: &temper_playground::stories::Story, seed: u64, wall: skein_lib::Wall) -> Page;
    /// The person's script walked to its end, or its failure, the browser's time moving only
    /// to the world's and the host's next instants; panics past `bound` of world time.
    pub fn walk(&mut self, person: &mut skein_fake_person::Person, bound: skein_lib::Duration) -> Walked;
    pub fn host(&self) -> &temper_client_shell_web::host::Host;
    /// The session log so far (skein/docs/design/playground.md, 6.1), its build `-`.
    pub fn log(&self) -> Box<[u8]>;
    /// The world, for checks at the end (the forge's main, the processes).
    pub fn world(&self) -> &temper_playground_world::Playground;
}
pub enum Walked { Done, Failed { step: u32, why: skein_fake_person::Why } }
```

- **One loop, the browser shell's:** each inbox report given to the host, its duties carried by
  `jig_client_shell_web::adapter::requests` and `skein_ui_web::transport::carry` over the
  `Memory` platform, `Report::Carried` raised again, the stream read on after a piece, the page
  drawn; each `Memory::slice` inside `programs::with(story::parameters(..), ..)` (C2).
- **Story A's person** (`temper/tests/playground/src/stories.rs`, `pub fn a() -> Vec<Step>`): the
  one-process world's script for the terminal client (`temper/tests/one-process/src/stories/
  playground.rs`, `script`), the same words, since both clients draw temper-client-view's tree:
  the welcome's "Try the playground", the three suggested requests each started and their reply
  lines seen, then the CSV change's page showing it landed.

## C4. The worker's entry

`temper-playground` on wasm32 is the worker's module (`playground.md`, 3: "on wasm32 its start,
the worker's module's entry"):

```rust
// temper/crates/temper-playground/src/worker.rs, #[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() { skein_playground::worker::run(make) }
/// The start message's story by name, its seed and start; the parameters installed for the
/// module's life (one world per worker); an unknown story panics, which the worker posts.
fn make(start: &skein_ui_web::bridge::Start, browser: skein_lib::Time)
    -> skein_playground::slices::Slices<Process<()>, Machine>;

// temper/crates/temper-playground/src/programs.rs, every target
/// Installs the factory parameters with no end: the worker's one world.
pub fn install(parameters: Parameters);
```

- `[lib] crate-type = ["cdylib", "rlib"]`; `wasm-bindgen` under
  `[target.'cfg(target_arch = "wasm32")'.dependencies]`.
- The worker's name in the bundle is `temper_playground` (`--worker
  temper_playground=<module>`), so its files are `temper_playground.js`,
  `temper_playground_bg.wasm` and `boot-worker.js` (`skein/docs/design/playground.md`, 7).

## C5. The shell's feature `playground`

`temper-client-shell-web` gains the feature `playground` (`playground.md`, 6;
`skein/docs/design/ui-web.md`, 18):

- **The platform is a type the feature chooses:** the wasm32 `Shell` is generic over its
  platform, `platform::Browser<Report>` for the shipped shell and `platform::worker::Worker<Report>`
  for the playground, behind one private trait for what the two do differently (the clock, the
  timer); `read_on` goes through `Platform::read_on`. Nothing else of the loop changes.
- **The playground's code builds on wasm32 with or without the feature,** public in `pub mod
  playground` (`src/playground.rs`, portable: the frame's model and words; `src/playground/web.rs`,
  wasm32: the starter, the frame's DOM, the failure page), so the gate's plain wasm32 clippy of the
  package lints it. The feature chooses only the module's entry: `#[wasm_bindgen(start)]` is the
  shipped `start` without it and `playground::web::start` with it.
- **The page's text** is `temper/crates/temper-client-shell-web/playground/index.html`, given to
  skein's bundler by `--page` (its `{bundle}` marker), holding the frame, the loading line, the
  failure section and `<div id="app">`; its style `playground/playground.css`. The frame's
  elements are found by `id`s that page and code share as constants in `playground.rs`.
- **The report the frame raises:** `Report::Frame(playground::Control)`, `Control` one of
  `Pause`, `Speed(bridge::Speed)` (½×, 1×, 10×: 500, 1000, 10000), `Reset`, `Story(&'static
  str)`, `Log`; `Report::World(worker::Heard)` for the worker's raises.
- **Stories the frame offers:** `playground::STORIES: &[(&str, &str)]`, each a name and its
  description, the description `include_str!` of `temper/crates/temper-playground/stories/<name>/
  description.txt`, so the page links no world.
- **The log by a key:** the frame's "Download the session log" saves `Worker::log()` as
  `temper-playground.log`, the way `record`'s `save_recording` saves a recording.

## C6. Served

- **One policy function** (`playground.md`, 7; `temper/docs/design/client/browser.md`, 2.2), in
  `temper/crates/temper-service/src/clients/policy.rs`:

  ```rust
  /// Which page the policy is for: the service's clients' listener or the static playground.
  pub enum Page { Service, Playground }
  /// The policy's fields; the playground's CSP adds `worker-src 'self'`.
  pub fn fields(page: Page) -> [(&'static [u8], &'static [u8]); 5];
  pub const FILES_CACHE: &[u8] = b"public, max-age=31536000, immutable";
  pub const PAGE_CACHE: &[u8] = b"no-cache";
  /// The playground's `headers` text for a bundle under `directory` (`/bundle/<digest>`).
  pub fn headers(directory: &[u8]) -> Vec<u8>;
  ```

  `FIELDS` stays, equal to `fields(Page::Service)`; `add` and `serve_call` use the function and
  the two constants.
- **The `headers` file:** lines of `<path>: <Name>: <value>`, ASCII, one per header, each ending in
  a newline, in this order: each policy field for `/*` (every answer); `<directory>/*:
  Cache-Control: <FILES_CACHE>`; `/*: Cache-Control: <PAGE_CACHE>`. A host applies the most
  specific path's `Cache-Control`. The build writes it from the template
  `temper/crates/temper-client-shell-web/playground/headers`, which is `headers(b"{bundle}")`'s
  text with `{bundle}` for the directory (a temper-service test holds the two equal), replacing
  `{bundle}` by `/bundle/<digest>`.
- **`temper/scripts/bundle.sh --playground OUT`:** the shell with `--profile web --features
  playground`, `temper-playground` with `--profile playground`, both for wasm32; skein's bundler
  with `--worker temper_playground=...`, `--page`, `--prefix /bundle/` and the shell's
  stylesheets and files; then `OUT/headers`. The root `Cargo.toml` gains `[profile.playground]`,
  `inherits = "web"`, `opt-level = "z"`, after `[profile.web]`.

## C7. Story A's kept log

- `temper/tests/playground/tests/fixtures/story-a.log`: the session log of the native walk of
  story A (C3) at the walk's seed and `Wall::EPOCH`, build `-`. The walk's test asserts its log is
  the kept one byte for byte; with `SKEIN_CODEGEN_WRITE` set it writes the file first, as temper's
  goldens do (`temper/crates/temper-client-shell-terminal/src/tests/render.rs`, ~34).
- A change that makes it differ is a new log, recorded again, or a finding (`playground.md`, 9):
  the branch that records it again says why in its commit.

## C8. The built playground on disk and its static host (adds to C6)

- **On disk:** skein's bundler writes `OUT/index.html` and `OUT/<digest>/...`; with `--prefix
  /bundle/` the files' addresses are `/bundle/<digest>/...`. `bundle.sh --playground` adds
  `OUT/headers` (C6). A host maps `/bundle/` to `OUT/`, answers any other path with
  `OUT/index.html`, and applies `OUT/headers` (most specific path first for `Cache-Control`).
- **The browser tier's host** is `temper-client-browser-world`'s binary `playground-serve`
  (`temper/tests/client-browser/src/bin/playground-serve.rs`): `playground-serve OUT` binds
  `127.0.0.1:0`, prints its address as the tier's servers do, and serves as above, GET and HEAD
  only, one connection at a time.
- **The tier's variable:** `TEMPER_TEST_PLAYGROUND`, the absolute `OUT` the tier's runner built
  with `temper/scripts/bundle.sh --playground` (`tools/tiers/design.md`, 2).
