# playground-skein: contracts

The run's binding calls: the shapes that cross tasks. A task builds its half to this note and
needs no other task's half to build. Names, fields, variants and signatures are binding; the
bodies, private items and the order of private fields are the implementer's. Each section
names the task that writes it.

## 1. `skein-ui-web`'s `bridge` module (task `ui-web-surface`)

`skein/crates/skein-ui-web/src/bridge.rs`, portable, declared `pub mod bridge;` in `lib.rs`
(`skein/docs/design/playground.md`, 5).

```rust
//! The bridge's messages between a page and its world's worker (playground.md, 5).
use crate::{fetch, transport::StreamDuty};
use skein_lib::{Duration, Time, Wall};

/// The world's pace: its nanoseconds per browser nanosecond, in thousandths; 0 is paused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Speed(pub u32);
impl Speed {
    pub const PAUSED: Speed = Speed(0);
    pub const ONE: Speed = Speed(1000);
}

/// A visit's start: the application's story, and the seed and the wall the page drew.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Start { pub story: Box<[u8]>, pub seed: u64, pub start: Wall }

/// An input for the visitor: fetch's duties (ui-web.md, 10).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    Exchange(fetch::Exchange),
    Abandon { exchange: u32 },
    Stream(StreamDuty),
    ReadOn,
}

/// The page's messages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToWorld { Start(Start), Input(Input), Pace(Speed) }

/// Which of a process's outputs a line was printed on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stream { Output, Error }

/// A whole line a process printed, without its newline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Printed { pub process: Box<[u8]>, pub stream: Stream, pub line: Box<[u8]> }

/// The moment an input entered the world: its step count and its instant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stamp { pub step: u64, pub instant: Time }

/// The world's clock, closing each slice's posts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clock {
    pub step: u64,
    pub now: Time,
    pub wall: Wall,
    /// Whether the world waits idle.
    pub idle: bool,
    pub next_due: Option<Time>,
    /// The worker's linear memory in bytes, its high water; 0 natively.
    pub memory: u64,
    /// The last slice's duration in browser time; zero natively.
    pub slice: Duration,
}

/// The world's messages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToPage {
    Report(fetch::Report),
    Printed(Printed),
    Stamp(Stamp),
    Clock(Clock),
    Stopped,
    Failed { words: Box<[u8]> },
}

/// Bytes that are no message of this build's.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Malformed;

/// The worker's ready message: the empty buffer, which no encoded message is.
pub const READY: &[u8] = &[];

impl ToWorld {
    #[must_use] pub fn encode(&self) -> Box<[u8]>;
    pub fn decode(bytes: &[u8]) -> Result<ToWorld, Malformed>;
}
impl ToPage {
    #[must_use] pub fn encode(&self) -> Box<[u8]>;
    pub fn decode(bytes: &[u8]) -> Result<ToPage, Malformed>;
}
```

- **The encoding** is the implementer's, unversioned (playground.md, 5): one tag byte per
  variant first, so no message encodes as `READY`; integers fixed-width little-endian; each
  byte string and each list its `u32` length first. `decode` refuses short input, trailing
  bytes, an unknown tag and a length past the input as `Malformed`, and allocates no more than
  the input's length. `decode(encode(m)) == Ok(m)` for every message.
- **`fetch::Exchange`** gains `Clone, PartialEq, Eq` to its derives and **`fetch::Report`**
  gains `Clone`: compatible, for the messages' derives.

## 2. The platform's two duties (task `ui-web-surface`)

`skein_ui_web::platform::Platform` gains, after `stream` (ui-web.md, 18: the duties fetch
alone took are the platform's, so both platforms take them alike):

```rust
    /// Abandons an exchange in flight: nothing more is reported of it (ui-web.md, 10).
    fn abandon(&mut self, exchange: u32);
    /// Asks the stream's next piece, after the step that took the last (ui-web.md, 5.2).
    fn read_on(&mut self);
```

`platform::Browser<R>` implements them as `self.fetch.abandon(exchange)` and
`self.fetch.read_on()`; its `pub fetch` field stays, so a shell calling
`platform.fetch.read_on()` still builds.

## 3. The module layout laid down by `ui-web-surface`

So that later tasks own disjoint files of `skein-ui-web`:

- `src/lib.rs`: `pub mod bridge;` and `pub mod log;`.
- `src/log.rs`: its module comment alone, `//! The session log's text and its keeping
  (playground.md, 6).` (task `ui-web-log` writes it).
- `src/platform.rs`: `pub mod worker;`; `src/platform/worker.rs`: its module comment alone,
  `//! The worker's platform, its pure half (ui-web.md, 18).` (task `ui-web-worker-page`).
- `src/tests.rs`: `mod bridge; mod log; mod worker;`, with `src/tests/log.rs` and
  `src/tests/worker.rs` empty but for a module comment.

## 4. The session log (task `ui-web-log`)

`skein_ui_web::log`, portable; the text is `skein/docs/design/playground.md`, 6.1, exactly.

```rust
/// A digest of what the world said between two inputs (playground.md, 6.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Said(pub [u8; 32]);

/// The fold of what the world said: its reports and its lines apart, and its stop.
#[derive(Clone, Debug)]
pub struct Saying { /* private */ }
impl Saying {
    #[must_use] pub fn new() -> Saying;
    pub fn report(&mut self, report: &fetch::Report);
    pub fn printed(&mut self, printed: &bridge::Printed);
    pub fn stopped(&mut self);
    #[must_use] pub fn said(&self) -> Said;
}

/// One kept input: its stamp, the input, and what the world said since the one before.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry { pub stamp: bridge::Stamp, pub input: bridge::Input, pub said: Said }

/// How a log ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    /// The last clock the page heard, and what the world said from the last input to it.
    At { step: u64, instant: Time, said: Said },
    /// The log reached its bound; a replay runs to the last input kept.
    Cut,
}

/// A log read back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Read {
    /// The bundle's digest directory's name; `None` for `-`, a log kept natively.
    pub build: Option<Box<[u8]>>,
    pub seed: u64,
    pub start: Wall,
    pub entries: Vec<Entry>,
    pub end: End,
}

/// Why a text is no log this build reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refused {
    /// The first line names another format.
    Format,
    /// Another version of the format, named as the log names it.
    Version { found: Box<[u8]> },
    /// A line that is not the format's, from 1.
    Line { line: u32 },
}

pub fn read(text: &[u8]) -> Result<Read, Refused>;

/// The page's log, kept while the visit runs (playground.md, 6).
#[derive(Debug)]
pub struct Log { /* private */ }
impl Log {
    /// `build` is the bundle's digest directory's name, `None` natively; `bound` is the
    /// text's most bytes, the application's.
    #[must_use] pub fn new(build: Option<&[u8]>, start: &bridge::Start, bound: u32) -> Log;
    /// An input the page sent, kept until its stamp comes.
    pub fn sent(&mut self, input: &bridge::Input);
    /// A world's message: a stamp keeps the oldest input sent and unstamped; a report, a line
    /// and the stop are folded; a clock marks the end.
    pub fn heard(&mut self, message: &bridge::ToPage);
    /// The file offered: its first line, each kept input's, and its last line.
    #[must_use] pub fn text(&self) -> Box<[u8]>;
}
```

- `read(&log.text())` gives back every kept entry, the seed, the start, the build and the end.
- A stamp with no input waiting is a broken contract: a panic. An input whose line would take
  the text past `bound` (with its last line) cuts the log: `End::Cut`, and nothing more is
  kept.
- `End::At` is the last `Clock` heard and the fold of what came after the last stamp and
  before that clock; with no clock heard yet, the start: step 0, instant 0, and the fold of
  nothing.

## 5. The worker's platform, its pure half (task `ui-web-worker-page`)

`skein_ui_web::platform::worker`, portable (ui-web.md, 18). `Page` is what a page holds of
its world, whichever carries the messages: the worker's page half on wasm32 and
`skein-playground`'s bridge in memory both hold one.

```rust
/// What the page holds of its world: the clock, the pace, the messages held until the
/// worker is ready, the small store in memory and the world's high waters.
#[derive(Debug)]
pub struct Page { /* private */ }
impl Page {
    /// Holds `ToWorld::Start(start)` first, then `ToWorld::Pace(Speed::ONE)`; the clock reads
    /// `start.start` and instant 0 until the world's first clock.
    #[must_use] pub fn new(start: bridge::Start, browser: Time) -> Page;
    /// The worker said it is ready: what was held may go.
    pub fn ready(&mut self);
    /// The messages to post now, in order: none until ready.
    pub fn outgoing(&mut self, out: &mut Vec<bridge::ToWorld>);
    /// An input for the visitor, held until ready.
    pub fn send(&mut self, input: bridge::Input);
    /// The frame's speed; sent unless the tab is hidden.
    pub fn pace(&mut self, speed: bridge::Speed, browser: Time);
    /// A hidden tab pauses the world; the shown tab resumes the frame's speed.
    pub fn visibility(&mut self, hidden: bool, browser: Time);
    /// A world's message at `browser`: a clock moves the platform's clock and the high waters.
    pub fn heard(&mut self, message: &bridge::ToPage, browser: Time);
    /// The world's clock as the platform gives it at `browser`: never back.
    pub fn clocks(&mut self, browser: Time) -> (Time, Wall);
    /// The browser instant at which the platform's clock reaches `deadline`, a world instant;
    /// `None` while paused, while the world steps, or past the world's next due instant.
    #[must_use] pub fn timer(&self, deadline: Time, browser: Time) -> Option<Time>;
    /// The high waters of the world's memory and of its slices.
    #[must_use] pub fn high_water(&self) -> (u64, Duration);
    /// The small store, in the page's memory (the platform's three store calls).
    #[must_use] pub fn read_all(&self) -> Vec<store::Record>;
    pub fn write(&mut self, key: &[u8], value: &[u8]) -> bool;
    pub fn remove(&mut self, key: &[u8]);
}
/// A navigation the worker's platform follows: a path on the page's own origin (ui-web.md,
/// 10's rule), which it shows in the address bar; anything else is not followed.
#[must_use] pub fn shown(to: &[u8]) -> bool;
```

- **The clock's arithmetic** (ui-web.md, 18): after a `Clock` heard at browser instant `b0`,
  while it says idle, `clocks(b)` gives `now + (b - b0) × speed / 1000`, at most its
  `next_due`, the wall moved by as much; while it says busy, or paused, its `now`. A clock or
  a pace change re-bases at its browser instant. What `clocks` gave is never given less
  again: a world's clock behind it is held at what was given until the world passes it.
- **`timer`:** for a deadline at or before what `clocks(browser)` gives, `browser` itself;
  otherwise `b0 + (deadline - now) × 1000 / speed`, rounded up to the nanosecond, when idle,
  not paused and the deadline at or before `next_due`; `None` otherwise. The shell sets it
  again at each clock heard and each pace change.
- **The store** keeps records whole, as session storage does; its bound is
  `store::Store`'s, so `write` always succeeds.
- `shown` is `navigation`'s rule for a path on the origin (one leading `/`, not `//`, no
  `\`), with no `https` target and no loopback.

## 6. The worker's platform, its page half (task `ui-web-worker-web`)

`skein_ui_web::platform::Worker<R>`, wasm32 only, in `src/platform/worker/web.rs`,
re-exported from `platform` beside `Browser`.

```rust
pub struct Worker<R> { /* private: a Page, the web Worker, the Log, the inbox */ }
/// What the world says that is no fetch report, for the application's own variant of `R`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Heard { Printed(bridge::Printed), Stopped, Failed { words: Box<[u8]> }, Clock }
impl<R: 'static> Worker<R> {
    /// Starts the worker from `address`, its starter in the bundle's digest directory, with
    /// the visit's start; its reports go to `inbox`, fetch's through `fetch`, the rest
    /// through `heard`; a timer due through `due`.
    #[must_use] pub fn start(
        address: &str,
        start: bridge::Start,
        build: Option<&[u8]>,
        log_bound: u32,
        inbox: Rc<Inbox<R>>,
        fetch: fn(fetch::Report) -> R,
        heard: fn(Heard) -> R,
        due: fn() -> R,
    ) -> Worker<R>;
    pub fn pace(&self, speed: bridge::Speed);
    pub fn visibility(&self, hidden: bool);
    /// The application's deadline, a world instant, or none.
    pub fn timer(&self, deadline: Option<Time>);
    #[must_use] pub fn clocks(&self) -> (Time, Wall);
    #[must_use] pub fn high_water(&self) -> (u64, Duration);
    /// The session log's text, offered as a file by the frame.
    #[must_use] pub fn log(&self) -> Box<[u8]>;
    /// Ends the worker: a reset, a new story or a failure (playground.md, 7).
    pub fn end(&self);
}
impl<R: 'static> Platform for Worker<R> { /* sends inputs; the store is the Page's */ }
/// The worker's starter in the bundle's digest directory: the directory of the page's
/// starter (`…/boot.js`) with `boot-worker.js` (playground.md, 7).
#[must_use] pub fn worker_address() -> Option<String>;
```

## 7. The bundle's worker module (task `bundle-worker`)

`skein-ui-bundle` (playground.md, 7: the worker's files):

```rust
/// A worker module, bundled beside the page's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Worker { pub name: String, pub glue: Glue }

/// `bundle` with a worker module; `bundle(inputs, glue, out)` is `bundle_with(inputs, glue,
/// None, out)`.
pub fn bundle_with(inputs: &Inputs, glue: &Glue, worker: Option<&Worker>, out: &Path)
    -> Result<Digest, Refusal>;

/// The worker's starter, beside the page's `boot.js`.
pub const WORKER_STARTER: &str = "boot-worker.js";
```

`Inputs`, `Glue`, `Refusal` and `bundle` keep their shapes: a clash refuses as
`Refusal::SameName` naming the worker's module path. The binary gains `--worker NAME=PATH`,
once at most.

## 8. `skein-playground`'s surface (task `playground-crate` lays it down)

The crate at `skein/crates/skein-playground`, its world at `skein/tests/playground`
(`skein-playground-world`). Modules, each its task's file:

| Module | File | Task |
|---|---|---|
| `visitor` | `src/visitor.rs`, `src/visitor/` | `playground-visitor` |
| `slices` | `src/slices.rs` | `playground-slices` |
| `memory` | `src/memory.rs` | `playground-memory` |
| `replay` | `src/replay.rs` | `playground-replay` |
| `worker` (wasm32) | `src/worker.rs`, `src/worker/` | `playground-worker` |

### 8.0 The visitor (task `playground-visitor`)

```rust
/// What the application gives its visitor (playground.md, 4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    /// The application's listener for its clients, in the world.
    pub listener: core::net::SocketAddr,
    /// The `Host` it sends, the application's name for that listener (`name:port`); its
    /// `Origin` is `http://` and the same.
    pub host: Box<[u8]>,
    /// The exchanges it holds in flight, the page's fetch bound.
    pub exchanges: u32,
    /// The cookies its jar holds, and the most bytes of one, its name and value.
    pub cookies: u32,
    pub cookie_bytes: u32,
    pub io: skein_io::Limits,
    pub http: skein_http::client::Limits,
}

/// A program of the world: fetch's inputs in, fetch's reports out, over skein's HTTP client.
#[derive(Debug)]
pub struct Visitor { /* private */ }
impl Visitor {
    #[must_use] pub fn new(config: &Config) -> Visitor;
    /// An input, between steps; it acts on it at its next turn.
    pub fn enter(&mut self, input: bridge::Input);
    /// Its reports since the last take, in order.
    pub fn take(&mut self, reports: &mut Vec<fetch::Report>);
}
impl skein_shell::Host for Visitor { .. }
```

An application's process type holds a `Visitor` and gives `Reach` (8.1) functions calling
`enter` and `take` on it.

### 8.1 The slices (task `playground-slices`)

```rust
/// How the loop reaches the visitor among the world's processes (playground.md, 3).
#[derive(Debug)]
pub struct Reach<P> {
    /// The visitor's index among the world's processes: one the world starts itself before
    /// any hosted child, so a child dropped never moves it.
    pub at: usize,
    /// Gives the visitor an input, between steps (world.md, 5).
    pub enter: fn(&mut P, bridge::Input),
    /// Takes the visitor's reports, in order.
    pub take: fn(&mut P, &mut Vec<fetch::Report>),
}
impl<P> Clone for Reach<P> { .. }   // and Copy: fn pointers and an index

/// When the loop wants its next slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wake {
    /// The world is busy: at once, whatever the speed.
    Now,
    /// Idle: the browser instant at which the next due instant comes at this speed.
    At(Time),
    /// Paused, settled, or nothing due: at the next message.
    Message,
}

/// The worker's loop over a world whose processes include the visitor.
pub struct Slices<P, M = NoMachine> { /* private */ }
impl<P: Host, M: Machine> Slices<P, M> {
    /// At 1×, its browser time counted from `browser`; `budget` steps a slice, at least 1.
    #[must_use] pub fn new(world: World<P, M>, reach: Reach<P>, budget: u32, browser: Time) -> Self;
    /// Queued; it enters at the next slice.
    pub fn input(&mut self, input: bridge::Input);
    pub fn pace(&mut self, speed: bridge::Speed, browser: Time);
    /// One slice at `browser` (playground.md, 3): its posts appended in 5's order, its clock
    /// last, with `memory` 0 and `slice` zero for the caller to fill.
    pub fn slice(&mut self, browser: Time, posts: &mut Vec<bridge::ToPage>) -> Wake;
    #[must_use] pub fn world(&self) -> &World<P, M>;
    pub fn world_mut(&mut self) -> &mut World<P, M>;
}
```

- Lines printed come from `World::printed()`, each `skein_sim_world::Printed` mapped to
  `bridge::Printed` (`Stream::Output` and `Stream::Error` alike).
- `Stopped` is posted once, at the slice that finds the world settled; a settled world's
  slices post their clock alone and wake at `Message`.

### 8.2 The bridge in memory (task `playground-memory`)

```rust
/// The bridge in memory (playground.md, 5): a `Platform` that hands the transport's duties
/// to the slices in the same thread, and raises their reports as the worker's platform does.
pub struct Memory<P, M, R> { /* private: a Page, the Slices, the Log, the inbox */ }
impl<P: Host, M: Machine, R> Memory<P, M, R> {
    #[must_use] pub fn new(
        slices: Slices<P, M>,
        start: bridge::Start,
        log_bound: u32,
        inbox: Rc<Inbox<R>>,
        fetch: fn(fetch::Report) -> R,
        heard: fn(platform::worker::Heard) -> R,
    ) -> Self;
    /// One slice at the test's browser instant; its posts raised in the inbox.
    pub fn slice(&mut self, browser: Time) -> Wake;
    pub fn pace(&mut self, speed: bridge::Speed, browser: Time);
    pub fn clocks(&mut self, browser: Time) -> (Time, Wall);
    #[must_use] pub fn timer(&self, deadline: Time, browser: Time) -> Option<Time>;
    #[must_use] pub fn log(&self) -> Box<[u8]>;
    /// The last navigation `worker::shown` accepted; natively none is followed.
    #[must_use] pub fn shown(&self) -> Option<&[u8]>;
    #[must_use] pub fn slices(&self) -> &Slices<P, M>;
    pub fn slices_mut(&mut self) -> &mut Slices<P, M>;
}
impl<P: Host, M: Machine, R> Platform for Memory<P, M, R> { .. }
```

`platform::worker::Heard` is section 6's enum; `playground-memory` needs it portable, so
`ui-web-worker-page` defines `Heard` in `platform/worker.rs` (portable) and
`ui-web-worker-web` re-uses it.

### 8.3 The replay (task `playground-replay`)

```rust
/// Where a replay first differed from its log, naming the build the log came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Differs {
    /// The log's line, from 1: the input's, or the last line's.
    pub line: u32,
    pub build: Option<Box<[u8]>>,
    pub why: Why,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Why {
    /// The world reached the stamp's step at another instant, or the instant at another step.
    Moment { expected: Stamp, found: Stamp },
    /// What the world said differs from the log's digest.
    Said,
}

/// Replays `log` over `world`, built by the application from the log's seed and start: each
/// input enters at its stamp and each digest is checked; it runs to the log's end and hands
/// the world back unsettled, with no teardown of its own to check.
pub fn replay<P: Host, M: Machine, H: Meter, T: Turns<P>>(
    world: World<P, M, H>,
    reach: Reach<P>,
    log: &log::Read,
    turns: &mut T,
) -> Result<World<P, M, H>, Differs>;
```

`turns` is how an application's referee observes each turn (`skein_sim_world::Turns`), as the
harness's run does; `()` observes nothing.

### 8.4 The worker's parts (task `playground-worker`)

```rust
/// The worker module's loop (wasm32): posts ready, waits for the start, makes the world with
/// `make`, then runs slices on messages and on its one timer, posting each message encoded,
/// its buffer transferred; a panic posts `Failed` with its words before the module traps.
pub fn run<P: Host + 'static, M: Machine + 'static>(make: fn(&bridge::Start, Time) -> Slices<P, M>);
```
