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
