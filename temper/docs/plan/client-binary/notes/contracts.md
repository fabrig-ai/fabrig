# client-binary: contracts

The run's binding shapes. A task builds its half to the sections its spec
names; the design is `temper/docs/design/protocol/README.md`, sections 4 and 5,
and `temper/docs/design/client/README.md`, section 2, 6.11 and 6.12.

## 1. `temper-shell`, the crate

- `temper/crates/temper-shell`, package `temper-shell`, library
  `temper_shell`, description "What every binary of temper's starts from
  (protocol/README.md, section 5)". Profile `binary`, `clippy.toml` the
  `ordinary` variant (`notes/common.md`, *A new crate*). Root `Cargo.toml`:
  member `"temper/crates/temper-shell"`, and `temper-shell = { path =
  "temper/crates/temper-shell" }` among the workspace's dependencies.
- Its dependencies: `skein-lib`, `skein-shell` (task `shell-local`), then
  `temper-docs` (task `shell-table`). Dev: `skein-scratch`. Nothing of jig,
  smith, the engine, the worker or the client.
- Its modules: `directories` and `link_file` (`shell-local`); `commands`,
  `rows` and `help` (`shell-table`).

## 2. `temper_shell::directories` (protocol/configuration.md, section 12)

Moved from `temper/crates/temper/src/directories.rs`, bodies verbatim:

```rust
pub const SETTINGS: skein_shell::Settings;          // the three directories' table
pub type Argument = Box<[u8]>;
pub type EnvironmentEntry = (Argument, Argument);
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Paths { pub config: Box<[u8]>, pub data: Box<[u8]>, pub cache: Box<[u8]>, pub origins: [skein_shell::Origin; 3] }
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal { Settings(skein_shell::Refusal), Bound(&'static str) } // and its Display
pub fn resolve(arguments: &[Argument], environment: &[EnvironmentEntry], working: &[u8])
    -> Result<(Paths, Box<[Argument]>), Refusal>;   // today's resolve, returning Paths
pub fn private_root(path: &Path, user: u32) -> Result<(), String>;
pub fn make_private(path: &Path, user: u32) -> Result<(), String>;
```

`temper`'s `directories` keeps `Directories` (its fields and every method) and
its readers' names: `pub use temper_shell::directories::{Refusal, SETTINGS};`,
`pub(crate) use temper_shell::directories::{make_private, private_root};`, and
`pub fn resolve(..) -> Result<(Directories, Box<[Argument]>), Refusal>`, today's
signature, over the shell's, `Paths` taken field by field (`impl From<Paths> for
Directories`).

## 3. `temper_shell::link_file` (client/README.md, 6.12)

Moved from `temper/crates/temper/src/link_file.rs`, bodies verbatim:

```rust
pub const MOST: u32 = 4096;
pub fn path(data: &[u8], runtime: Option<&[u8]>, linux: bool) -> Option<PathBuf>;
pub fn deployment(environment: &[EnvironmentEntry], working: &[u8]) -> Option<Box<[u8]>>; // resolve(&[], ..)'s data
pub fn keep(path: &Path, url: &[u8], user: u32) -> Result<(), String>;
pub fn forget(path: &Path);
pub fn read(path: &Path, user: u32) -> Option<skein_lib::Secret>;
```

`temper`'s `lib.rs` says `pub use temper_shell::link_file;` where it said `pub
mod link_file;`, so `crate::link_file::…` and `temper::link_file::…` read on.

## 4. `temper_shell::commands` and `rows` (protocol/README.md, section 4)

Moved from `temper/crates/temper/src/commands.rs`; each function takes the
binary's name, and the table where it reads one:

```rust
pub enum Who { Documentation, Deployment, Clients, Spawned, TestDoubles } // and word()
pub struct Row { pub words, pub who, pub forms, pub line, pub arguments, pub settings, pub notes } // as today
pub struct Argument { pub shown: &'static str, pub line: &'static str }
pub fn find(rows: &'static [Row], words: &[Box<[u8]>]) -> Option<&'static Row>;
pub fn unknown(name: &str, rows: &[Row], words: &[Box<[u8]>]) -> String; // "{name}: no command{ WORDS}; {name} --help lists them\n"
pub fn command(name: &str, row: &Row) -> String;   // "{name}" or "{name} {words}"
pub fn forms(name: &str, row: &Row) -> String;
pub fn refused(name: &str, row: &Row, why: &str) -> String;
pub fn help(name: &str, rows: &[Row]) -> String;   // `NAME --help`, its first two lines naming NAME
pub fn row(name: &str, row: &Row) -> String;       // `NAME COMMAND --help`
```

`temper_shell::rows`, the rows more than one binary holds, moved verbatim from
`temper`'s table: `pub const HELP: Row` (`["help"]`), `pub const TUI: Row`
(`[]`, `[ADDRESS]`) and `pub const CTL: Row` (`["ctl"]`).

`temper`'s `commands` keeps `pub use temper_shell::commands::{Argument, Row,
Who};`, its own consts, `pub const COMMANDS: [Row; N]` with `rows::HELP`,
`rows::TUI` and `rows::CTL` in their places, `reference()` (the page, the box's
own), and today's functions as one-line wrappers naming `"temper"` and
`&COMMANDS`, their signatures and visibility unchanged.

## 5. `temper_shell::help` (protocol/documentation.md, section 4)

Moved from `temper/crates/temper/src/help.rs`:

```rust
#[derive(Debug, PartialEq, Eq)]
pub enum Request { Index, Page(Box<[u8]>), Search(Box<[u8]>) }
pub fn request(words: &[Box<[u8]>]) -> Option<Request>;
pub fn answer(name: &str, request: &Request) -> Result<String, String>; // "{name}: no page X; {name} help lists them\n", …
```

`temper`'s `help` keeps `pub use temper_shell::help::{Request, request};` and
`pub fn answer(request: &Request) -> Result<String, String>`, naming `"temper"`.

## 6. `temper_client_shell_terminal::address` (client/README.md, 6.1 and 6.11)

Moved from `temper/crates/temper/src/open.rs`' `target`, body verbatim:

```rust
#[derive(Debug)]
pub enum Target { Link { engine: SocketAddr, link: skein_lib::Secret }, Address { engine: SocketAddr } }
#[derive(Debug, PartialEq, Eq)]
pub enum Refused { NotALink(Box<[u8]>), NotLoopback }
pub fn target(argument: &[u8]) -> Result<Target, Refused>;
```

`temper`'s `open` reads it, mapping `Refused` into its own `Refused`'s two
variants of the same names; `open::Refused::line` and every line of `temper
open` are unchanged.

## 7. `temper_client_shell_tui::start` (client/README.md, 6.11 and 6.12)

Moved from `temper/crates/temper/src/tui.rs`, bodies verbatim but the name:

```rust
pub type EnvironmentEntry = (Box<[u8]>, Box<[u8]>);
#[derive(Debug, PartialEq, Eq)]
pub enum Refused { NotATerminal, NotAnAddress(Box<[u8]>), Link, NotLoopback, Directory(String) }
impl Refused { pub fn line(&self, name: &str) -> String }   // "{name}: {why}\n"; Link: "… run {name} http://HOST:PORT and paste it"
pub fn address(argument: &[u8]) -> Result<SocketAddr, Refused>;
pub fn directory(environment: &[EnvironmentEntry]) -> Result<PathBuf, Refused>;
pub fn kept(bytes: &[u8]) -> Option<SocketAddr>;
pub fn selected_address(given: Option<SocketAddr>, link: Option<&[u8]>, saved: Option<SocketAddr>) -> SocketAddr;
pub fn on_box(link: Option<&[u8]>, engine: SocketAddr) -> bool;
pub fn link_path(environment: &[EnvironmentEntry], working: &[u8]) -> Option<PathBuf>;
pub fn read_note(directory: &Path, engine: SocketAddr, user: u32) -> Option<temper_client_tui::Note>;
pub fn opener(environment: &[EnvironmentEntry], linux: bool, executable: &dyn Fn(&Path) -> bool) -> Option<crate::opener::Opener>;
pub fn scheme(environment: &[EnvironmentEntry]) -> Option<temper_client_tui::Scheme>;
pub fn depth(environment: &[EnvironmentEntry]) -> skein_ui_screen_terminal::Depth;
pub fn terminal_refusal(name: &str, given: bool) -> String; // given: NotATerminal's line; else "{name}: no command; {name} --help lists them\n"
pub fn run(name: &str, argument: Option<&[u8]>, clock: &skein_shell::Clock) -> u8;
pub fn local_run(settings: crate::run::Settings, root: Fd, user: u32, seed: u64, signals: Fd) -> crate::run::Run;
pub fn keep_address(parent: &Path, bytes: Box<[u8]>, user: u32, seed: u64, clock: &skein_shell::Clock);
```

It reads `temper_client_shell_terminal::address` (6) and `temper-shell`'s
`link_file` and `directories::make_private` (2, 3). Its `record` recording parent
is under the crate's own `record` feature, which `temper`'s `record` already
turns on. `temper`'s `tui` keeps `pub(crate) fn run(argument: Option<Box<[u8]>>,
clock: &Clock) -> u8`, calling `start::run("temper", …)`.

## 8. `temper_client_shell_cli::start` (client/tools.md, 8.1)

Moved from `temper/crates/temper/src/ctl.rs`, bodies verbatim; its lines say
`temper ctl: ` in both binaries:

```rust
pub fn parse<'a>(words: &[&'a [u8]]) -> Result<(Option<&'a [u8]>, temper_client_tools::jig::call::Parsed), String>;
pub fn run(arguments: &[std::ffi::OsString], clock: &skein_shell::Clock) -> u8;
```

`temper`'s `ctl` keeps `pub use temper_client_shell_cli::start::run;` and
`pub(crate) use temper_client_shell_cli::start::parse;`.

## 9. `temper-client`, the package (protocol/README.md, section 4)

- `temper/crates/temper-client`, package `temper-client`: `[lib] name =
  "temper_client"`, `path = "src/lib.rs"`; `[[bin]] name = "temper-client"`,
  `path = "src/main.rs"`, `test = false`; `build = "build/main.rs"`; `[[test]]
  name = "suite"`, `path = "tests/suite/main.rs"`. Profile `binary`; root
  `Cargo.toml` member (no workspace dependency: nothing links it).
- **The commit:** the build script sets `TEMPER_CLIENT_COMMIT` for the crate
  (`cargo:rustc-env`): the commit's first twelve hex digits, then `-dirty` when
  git reports tracked changes, or `unknown` with no git or no repository. Its
  pure part, `build/commit.rs`' `pub fn word(head: Option<&str>, dirty: bool)
  -> String`, is included by the script and by the library's tests (`#[path]`).
- `src/lib.rs`:

```rust
pub const NAME: &str = "temper-client";
pub fn version() -> String;  // "temper-client {CARGO_PKG_VERSION} ({TEMPER_CLIENT_COMMIT})\n"
// task client-table:
pub const COMMANDS: [temper_shell::commands::Row; 3] = [rows::HELP, rows::TUI, rows::CTL];
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Overview,                                   // --help
    Version,                                    // --version
    Usage(&'static temper_shell::commands::Row), // COMMAND --help
    Help { request: temper_shell::help::Request },
    Tui { address: Option<Box<[u8]>> },
    Ctl { arguments: Vec<std::ffi::OsString> },
}
pub fn configure(arguments: &[Box<[u8]>]) -> Result<Command, String>;
pub fn run(command: Command) -> Result<u8, String>;
```

- `src/main.rs`: `skein_fail_stop::install()`, then `configure(&skein_shell::
  arguments())` and `run`; `Ok(code)` exits `code`, `Err(why)` prints `why` on
  stderr whole and exits 1.

## 10. The story's call (the process tier)

- `temper/crates/temper/tests/support/ctl.rs`' `Call` gains `pub fn by(self,
  program: &Path) -> Self`: the same call run by `program`, as `PROGRAM ctl
  DIRECTORY ARGUMENT…` with the call's stdin, where a plain `Call` runs the box's
  `temper`. `Call::command`'s calls are unchanged.
- The client's file in that tier is the box's binary's sibling,
  `skein_scratch::paths::binary("temper").with_file_name("temper-client")`,
  which the tier builds first: `tools/tiers/packages.toml`'s `[binaries]`
  `process-temper` lists `{ package = "temper-client", bin = "temper-client" }`
  after `smith-fake-claude`'s.
