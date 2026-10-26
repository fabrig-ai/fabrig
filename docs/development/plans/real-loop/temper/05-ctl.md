# Session 05: `temper ctl`'s one-shot host

`temper-client-shell-cli` (tools.md, 4) hosts one call on skein's loop,
with no `main`: parse, open DIRECTORY, lock it for a call that sends,
sign in, send what the store kept, run the call, print `jig::Printed`,
exit. temper 08 and 09 host it as a program per call; temper 10 runs
the executable. Needs at start: the tools plan, whole (its root, kept
lines, waits, help and `adopt`; temper `8c02b3b9e`'s repin). Steps
`temper-05.1` to `temper-05.6`. The meeting point first (decisions.md,
5): 05.1 to 05.4 land DIRECTORY and its lock, the native transport, a
read, `sign-in FILE` and the `chat` act, all temper 08.3 needs; 05.5 and
05.6 land the rest. jig's one-shot story (client.md, 20) stays jig's.

## Read first

- This plan's README (its Rules) and `decisions.md` (design question 3;
  decisions 3, 5, 9, 10; choice A); the tools plan's `decisions.md`,
  question 29's correction and question 33.
- temper's `tools.md`: 4, 5.6, 6.1 to 6.3, 7, 8, 11.3, 11.4, 12, 15;
  `testing.md`: 2.4, 6.1. jig's `client/domain/client.md`: 6, 11, 20;
  `protocol/client.md`: 8, 10. skein's `io.md`: 5, 7; `shell.md`: 6,
  12, 13; `lib.md`: 12; `programming-model.md`: 2, 2.1, 4.2, 6.3, 9;
  `testing-strategy.md`: 2.1, 10.
- Code: the tools' `root`, `table`, `limits`, `help`, `jig::{runner,
  json}`, `jig::runner::recovery`, `jig::render::outcome`; the
  terminal's `limits`, `client` (`NativeClient`), `terminal::run`;
  `crates/temper/src/{lib,open,tests}.rs`; jig-client-shell-native's
  `shell`, `files`, `connections`; jig-client-protocol-link's `store`,
  `translate::store`; skein-io's `file`, `file_layer`.

## What the design asks

- **4, 8:** the host of jig's one-shot mode (client.md, 20) in six
  steps; DIRECTORY and `address` (8.1); the lock, and the empty store of
  reads and waits (8.3); 12's bounds, shipped in `limits.rs`.
- **5.6, 6.1 to 6.3, 7:** the host's tools, `pending`; what was kept
  sent and answered first; `kept:`, parked and `dropped:` lines; one
  result; statuses and their words.
- **Design question 3:** on `Shutdown`, end the call, send and print
  nothing more, leave the kept requests, exit 128 + N.

## Choices this plan makes

- **Two native compositions.** Reads and waits: `transport::Memory`,
  native `Connections` over the host's `skein_io::Io`, answering `Save`
  and `Erase` with `Stored` in order (`Connections::down` ignores them).
  Acts and the host's tools: native `Shell`. A `Shell` would load the
  store and send its stream gone (8.3). No jig change (decision 3).
- **One owner per descriptor.** `Startup.parent` goes to the host's
  `FileIo` (`adopt_root`): for a call other than `sign-in`, `OpenDirectory`
  of DIRECTORY's leaf first (ruling 102); `OpenPrivate` of it, then `Lock`
  (calls that send), `Load` of `address`, and, on the memory path, of
  `token` (40 bytes, `no_follow`, as `files.rs` does). For `Shell`, the
  host opens a second descriptor, `Op::Open { root: parent, path: b".",
  how: OpenHow::Directory }` (`Op::is_valid` allows it), which
  `Shell::adopt` owns, with `Settings.directory` the leaf. The lock is
  the host's open, held until the host closes it, once the call has its
  status and `Shell` is empty, before its output drains and it exits
  (tools.md, 8.3); the OS frees it on a kill. Tokens:
  the host's io `host::TAG` (1 << 61, as `terminal/run.rs`), its
  `FileIo` `host::FILES` (1 << 62); `Shell` keeps its own (`files.rs`).
- **The recovery barrier:** `Request::Ready`, `Root::kept() == 0` (the
  party's kept, less those answered) and every `Save`/`Erase` given to
  `Shell` `Stored`, one at a time. A `NotStored` admits no act: exit 1,
  "not kept (DIRECTORY's store not written)" (6.3).
- **The link deadline.** `limits::DEADLINE` bounds sign-in, the first
  `me`, each kept or new request unheard, and the link down; a retry
  never restarts it; a healthy wait outlives it (5.4). Past it: exit 1,
  `temper ctl: the engine at ADDRESS did not answer within 30 s` (design
  question 5).
- **Output.** `jig::Printed` and `Request::Kept` lines pass through; a
  status-1 text failure gets `temper ctl: ` once; JSON and kept lines
  never. The host's own failures are `temper ctl: ` and 6.3's words.
- **A signal:** `Exit::Code(limits::INTERRUPTED)` (130) or
  `limits::TERMINATED` (143), never `process::exit` (shell.md, 13).

## Rules that bind it

Units drive the host by explicit completions and virtual time, under
20 ms, limits from `limits::native()`, never by hand; one behaviour per
test; named consts; no new `#[expect]`; no global stdio, exit or signal
handler; no forge, fake or design edit; the tools' table, root, waits
and help keep their owners.

## Increments

### 05.1 The crate, its host frame, DIRECTORY and its lock
Scope: the crate (tools.md, 4), `arguments`, `OneShot`'s frame (skein's
`Host`, output, exit, a signal's 128 + N), DIRECTORY, `address` and the
lock (8.1, 8.3). Unblocks 09.3's lock pace (`limits::LOCK_TRY`).
Needs: nothing beyond the session's start.
Files: temper's root `Cargo.toml` and `Cargo.lock` (a member, a
workspace line); new `crates/temper-client-shell-cli/` (`Cargo.toml`,
`clippy.toml`, `src/{lib,arguments,directory,host,output,limits}.rs`,
`src/tests.rs`, `src/tests/fixtures.rs`).
Shape:
- `[package]` inherits the workspace's fields, `license.workspace =
  true` among them once temper's workspace names a license
  (`code-organization.md`, 1).
- `forbid(unsafe_code)`; `clippy.toml` as the terminal's; dependencies,
  all root workspace lines: the tools, `temper-client-domain`, `jig-client`,
  `jig-client-shell-native`, `jig-client-protocol-{http,link,wire}`,
  `skein-{io,lib,json,shell}`.
- `arguments::parse` is `table::parse` over the words after DIRECTORY;
  `arguments::class` reads `TOOLS[call.tool]` and compares its `.name`
  with the rows' (`Tool` derives only `Debug`; as `root::step` compares
  a call's tool with `adopt::ROW`): an act if `keyed` is not
  `Keyed::Never`; `rows::{SIGN_IN, SIGN_OUT, PENDING, HELP}`; a wait
  (`rows::WAIT`); else a read. `Parsed::File` is 05.6's: usage until.
- `directory::Directory` over `FileIo`: `OpenPrivate`, `Lock`, `Load`;
  only `sign-in` creates DIRECTORY (ruling 102): any other call first
  sends `OpenDirectory { root: parent, name: leaf, no_follow: true }`,
  which makes nothing; its `NotFound` exits 1, `temper ctl: DIRECTORY does
  not exist`, before any `OpenPrivate`; its `Opened` token is closed
  (`Close`), then `OpenPrivate` as for `sign-in`. On one with no `token`,
  or a zero-byte one (tools.md 8.1, "which a call reads as no token"),
  `temper ctl: DIRECTORY is not signed in` (design question 5);
  of `address` (`limits::ADDRESS` bytes): exactly one numeric
  `http://HOST:PORT` line; 127/8 or `[::1]`, else exit 1 "the engine's
  address must be on loopback", before any token is read.
- `Busy` arms a try at `LOCK_TRY` (100 ms); past `LOCK` (60 s; tools.md
  8.3, 12), exit 1 "DIRECTORY is
  busy with another call"; `Failed` exits 1 at once; `Refused` names
  the part (io.md, 5.3).
- `limits::native()` builds `Limits { root, connections, io }` from
  `temper_client_tools::limits` and the terminal's numbers, copied (no
  terminal dependency, tools.md 4); reexports the tools' consts;
  `limits::{INTERRUPTED, TERMINATED, ADDRESS}`.
Test shape: `src/tests.rs`; `fixtures::Frame` answers `OneShot`'s
submissions (`Complete`s built by hand) at a `Time` it advances, under
`limits::native()`; `fixtures::CHAT` is `["chat", "hello", "--project",
"shop", "--key", "start-chat"]`, as the tools' `CREATE_CHAT_CALL`.
```rust
#[test]
fn a_busy_directory_is_refused_after_60_s_of_retries() {
    let mut f = fixtures::Frame::act(fixtures::CHAT);
    f.lock_busy();
    assert_eq!(f.next_deadline(), Some(f.started() + limits::LOCK_TRY));
    f.advance_past(limits::LOCK);
    assert_eq!(f.stderr(), b"temper ctl: DIRECTORY is busy with another call\n");
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
```
Others: `calls_that_send_nothing_take_no_lock`, `a_name_address_is_refused`,
`a_non_loopback_address_exits_1_before_the_token`,
`a_refused_directory_exits_1`, `an_absent_directory_is_never_made` (no
`MakeDirectory` or `OpenPrivate` submitted; exit 1),
`a_zero_byte_token_is_not_signed_in`, `sigterm_exits_143_printing_nothing_more`,
`sigint_exits_130`, `partial_writes_finish_before_exit`; each under 20 ms.
Proof: unit, `cargo nextest run -p temper-client-shell-cli -E
'test(a_busy_directory_is_refused_after_60_s_of_retries)'`:
no world hosts the crate until 08.3; 09.3 proves contention on the kernel.
Size: at most 290 production lines.
Don't: lock `store`; shorten `LOCK` for a test; add a `main`; detect
`temper open` on the same DIRECTORY (tools.md 15: not supported, the
lock `temper ctl`'s alone).
Removes: nothing.

### 05.2 The memory transport and a read
Scope: `transport::Memory` and a read through it: token, `Start` empty,
`SignIn { with: Held }`, `Ready`, the call, the link deadline (8.2, 8.3).
Needs: the frame (05.1).
Files: new `src/transport.rs`; `src/{host,limits,tests}.rs`,
`src/tests/fixtures.rs`, the crate's `Cargo.toml` (the dev-dependency).
Shape:
- `Memory` joins `Connections::{down, take, up, next_deadline}` to the
  host's `skein_io::Io` (a socket per `Conn` to the engine), with
  bounded maps; HTTP and retries stay native's. `Save`/`Erase` queue a
  `Stored` each, in order. `CloseStream` and an io `Abort` of the host's
  sockets end it. Its memory sum: `Shell::worst_case`'s published
  formula, as the terminal's `limits::shell`.
- `OneShot` feeds `root::step`, `fire`, `next_deadline` into skein's
  methods; observes `Root::client().link()`; tracks each read's first
  issue against `DEADLINE`; never opens `store`.
Test shape: `src/tests.rs`; `fixtures::Engine` answers the host's
sockets with HTTP replies of wire records (`jig_client_test_records::
frame_record` for `me`) as `jig_client_protocol_http`'s mapping reads
them (`jig-client-test-records.workspace = true`, a dev-dependency).
```rust
#[test]
fn a_read_never_opens_the_store() {
    let mut f = fixtures::Engine::read(&["me", "--json"]);
    f.sign_in_then_deliver_me();
    f.drain();
    assert!(!f.opened(b"store"));
    assert_eq!(f.exit(), Some(Exit::Code(0)));
}
```
Others: `memory_saves_are_stored_in_order_without_file_operations`,
`a_read_waits_for_me`, `an_unheard_read_exits_1_at_the_link_deadline`,
`a_read_sends_no_stream_gone`, `host_bounds_cover_its_holders`; each
under 20 ms, deadlines in virtual time.
Proof: unit, `cargo nextest run -p temper-client-shell-cli -E
'test(a_read_never_opens_the_store)'`: the real engine and kernel need
08.3's world, which needs 05.4 too.
Size: at most 280 production lines.
Don't: instantiate `Shell` and drop its records; implement jig's retries.
Removes: nothing.

### 05.3 `sign-in FILE`
Scope: the first `Shell` composition: the link, `token` and `address`
written, `me` live (5.6, 8.1, 9). With 05.2, 08.3's sign-in step.
Needs: a read (05.2).
Files: new `src/signing.rs`; `src/{host,directory,limits,tests}.rs`,
`src/tests/fixtures.rs`.
Shape:
- Locked; reads FILE by `FileIo` within `limits::LINK` (new);
  `jig_client_protocol_http::printed` parses it first, since
  `Shell::sign_in` refuses a host not `Settings.engine`; loopback checked;
  `limits::settings(host, leaf)`; the second parent open is adopted.
- `Shell::sign_in` gives the secret: `SignIn { with: Link }`. At
  `credential` in `link()`, `Shell::keep`; done at `kept() ==
  Some(true)`, `address`'s `Store` terminal and `me` live: `me`'s text.
- The secret is never printed nor in argv (9); a refused link exits 1.
Test shape: `src/tests.rs`, `fixtures::Engine` with `Shell`'s file
completions (`fixtures::Disk`).
```rust
#[test]
fn sign_in_keeps_its_record_before_success() {
    let mut f = fixtures::Disk::sign_in(fixtures::LINK);
    f.answer_sign_in_then_me();
    assert_eq!(f.exit(), None);
    f.store_token_with_address();
    assert_eq!(f.stored(b"address"), Some(b"http://127.0.0.1:4000\n".as_slice()));
    assert_eq!(f.exit(), Some(Exit::Code(0)));
}
```
Others: `a_non_loopback_link_exits_1_before_sending`,
`a_refused_link_never_prints_its_secret`, `sign_in_takes_the_lock`;
each under 20 ms.
Proof: unit, `cargo nextest run -p temper-client-shell-cli -E
'test(sign_in_keeps_its_record_before_success)'`: modes 0o700
and 0o600 need the kernel, 08.3's.
Size: at most 200 production lines.
Don't: store the link; send the token before the loopback check.
Removes: nothing.

### 05.4 The `chat` act, after what was kept
Scope: the act path on `Shell`: `Start` with the loaded records, the
recovery barrier, then the act, durable (7, 8.2). Every act takes it;
this proves `chat`. Unblocks 08.3, 09.1, 09.2.
Needs: `sign-in FILE` (05.3).
Files: new `src/recovery.rs`; `src/{host,limits,tests}.rs`,
`src/tests/fixtures.rs`.
Shape:
- Locked; `loaded()` gives token and records; `kept_json`, then
  `Start { kept }`, `SignIn { with: Held }`. `loaded()` still `None`
  once `Shell` is idle: exit 1, "DIRECTORY's store cannot be read".
- `Recovery` holds `Event::Call` until the barrier; `Request::Kept`
  lines (kept, parked, `dropped:`) go to stderr unprefixed.
- Each restored request's `DEADLINE` runs from recovery, the act's
  from its issue; past it, exit 1 with the record kept (6.3).
Test shape: `src/tests.rs`; `fixtures::kept_chat` and
`fixtures::other_party` encode records as `start_store` reads them.
```rust
#[test]
fn a_new_act_waits_for_the_kept_answers_erase_to_be_stored() {
    let mut f = fixtures::Disk::act_with(fixtures::kept_chat());
    f.answer_kept();
    assert_eq!(f.root_kept(), 0);
    assert!(!f.act_sent());
    f.store_erase();
    assert!(f.act_sent());
    assert!(f.stderr().starts_with(b"kept: chat, "));
}
```
Others: `a_kept_line_names_no_name_or_key`, `a_dropped_record_prints_its_line`,
`another_partys_record_prints_the_parked_line`
(decision 9), `a_failed_erase_admits_no_act`,
`two_writes_get_two_ordered_terminals`, `an_unheard_act_stays_kept`,
`a_bad_store_exits_1_cannot_be_read`, `sigterm_mid_act_keeps_its_record`,
`a_json_kept_line_is_unprefixed`; each under 20 ms.
Proof: unit, `cargo nextest run -p temper-client-shell-cli -E
'test(a_new_act_waits_for_the_kept_answers_erase_to_be_stored)'`;
recovery across a kill needs the kernel, 09.2's; the parked line stays
unit, area 1 having no second party (decision 9).
Size: at most 280 production lines.
Don't: count `Answered` alone; erase on a link failure; add a kill seam.
Removes: nothing.

### 05.5 The other acts, sign-out, `sign-in -` and `pending`
Scope: acts differing from `chat` (`close`, a drawn key, `adopt`), the
rest of 5.6. Unblocks 08.4's `close`, 10's `sign-in -`, 13.4.
Needs: the act path (05.4).
Files: new `src/pending.rs`; `src/{signing,host,tests}.rs`,
`src/tests/fixtures.rs`, the crate's `Cargo.toml` (`jig-client-domain`);
temper-client-tools' new `src/jig/render/pending.rs`, `render.rs` (`pub
mod pending`), `src/tests/jig.rs`, new `src/tests/jig/pending.rs`, and
`src/jig/rows.rs` (`SIGN_OUT`'s `line`, "End sign-in and erase the
token", becomes "End sign-in and leave the token empty": tools.md 5.6,
"leaves `token` empty, zero bytes, read as signed out"; ruling 102),
`src/guide.rs` with `src/tests/golden/{help,help_pending}.txt` and
`src/tests/help.rs` (`PENDING`'s stale `create` becomes `chat`, the word
`Operation::of` prints, `findings/code-checks.md`); root
`Cargo.toml`, `Cargo.lock` (a `jig-client-domain` workspace line, its
URL copied from the root's jig lines); and the nine comments in
`crates/temper-client-tools/src/` that cite `tools.md` or `testing.md`
bare, which cite `client/tools.md` and `domain/testing.md`, their paths
since temper `5587ae49b`, as `client/README.md` is cited.
Shape:
- `sign-in -` reads stdin to EOF within `limits::LINK`. `sign-out`:
  `Event::SignOut`, its terminal and `Shell::keep(None)` (a zero-byte
  `token`, read as none); a signed-out start is not its end.
- `pending`, unlocked: `Shell` with `limits::settings` (engine
  `127.0.0.1:0`, never dialled), `loaded()`, then `start_store`'s
  checks: `v9::Kept::decode`, `TABLE_VERSION`, `STREAM_KEY` skipped,
  `restore_table`, the declared `Spoken` owners. Unreadable: counted.
- `render::pending(rows, unread, json)`: `Operation::of`'s word,
  `party N`, `first sent HH:MM UTC` (the tools' `document::utc`, already
  `pub(super)`, so `render::pending` sees it; design question 6) or `not sent`; JSON `operation`,
  `party`, `first_sent` (jig's integer or `null`), `unread`; then
  5.6's unread line. A file that is not a small store's exits 1,
  "DIRECTORY's store cannot be read" (5.6).
Test shape: `src/tests.rs` with `fixtures::Disk`; the renderer in the
tools' `src/tests/jig/pending.rs`, over `fixtures::kept` (a `stop`,
party `PARTY`, never sent).
```rust
#[test]
fn a_pending_row_reads_operation_party_not_sent() {
    let printed = render::pending::pending(fixtures::kept(1).as_slice(), 0, false);
    assert_eq!(&*printed.stdout, b"stop, party 73, not sent\n");
}
```
Others: `pending_lists_each_kept_request` (`chat, party 73,
not sent`, no network, unlocked), `pending_counts_unread_records_in_its_line`,
`pending_skips_the_stream_record`, `pending_json_carries_its_rows_with_unread`,
`a_file_that_is_no_store_cannot_be_read`, `sign_out_waits_for_the_empty_token_stored`,
`sign_in_dash_reads_stdin`, `close_kept_line`,
`a_drawn_key_is_resent_from_directory`, `adopt_is_an_act`; each under 20 ms.
Proof: unit, `cargo nextest run -p temper-client-shell-cli -E
'test(pending_lists_each_kept_request)'`: the unread line has
no real-loop story (ruling 130), `sign-in -` through pipes temper 10's.
Size: at most 300 production lines (the renderer's included).
Don't: lock or dial for `pending`; rewrite `store`; print a key.
Removes: nothing.

### 05.6 Waits, help and `temper ctl`
Scope: waits on the memory path, `help`, `--call`, TEXT `-`, and the
executable's `ctl` branch (8.1). Unblocks 08.4, 09.3, 10.1, 13.5.
Needs: the other acts and `pending` (05.5).
Files: `src/{arguments,host,tests}.rs`, `src/tests/fixtures.rs`; new
`crates/temper/src/ctl.rs`;
`crates/temper/src/{lib,tests}.rs`, `crates/temper/Cargo.toml`.
Shape:
- `Parsed::File`: FILE, beneath `Startup.file_parent` as temper-05.s1
  reads sign-in's (ruling 111; `arguments::file_root` names its
  directory), or stdin, then `jig::json::call`. TEXT `-`
  reads stdin within `ask_bytes`; a second `-` is usage. All before
  DIRECTORY opens. A decoded call that gives an argument of the type a
  file, a path or `-` (`sign-in`'s), is usage too, and opens no root
  (tools.md, 8.1, from the designer's answer of 2026-10-19): 05.6
  landed without it (ruling 119), and temper-05.s2, after 05.6
  (ruling 123), adds it, with `a_call_that_names_a_file_is_usage` (`--call` FILE and
  `-`).
- `help [TOOL]` and `DIRECTORY help`: `help::text`; `help TOOL --json`:
  `jig::json::write_tool`. No DIRECTORY, client or lock.
- Waits: the root's deadline in both deadline methods; a stopping or
  offline engine cannot extend `DEADLINE`; 0, 2, 3 from `Printed`.
- `Command::Ctl { arguments }`, `configure`'s `"ctl"`, `run` and
  `USAGE`; `ctl::run(&arguments, &Clock) -> u8` builds `Startup`
  (`seed()`, `effective_user()`, `open_termination_signals()`, and
  `file_parent` by `skein_shell::open_root` of `arguments::file_root`), drives
  `OneShot` with `Kernel` and converts its `Exit`, as `open::run`.
Test shape: `src/tests.rs`; `crates/temper/src/tests.rs` (`configure`,
`USAGE`).
```rust
#[test]
fn a_stopping_engine_exits_the_wait_only_at_the_link_deadline() {
    let mut f = fixtures::Engine::wait(&["wait", "task", "41"]);
    f.engine_stopping();
    assert_eq!(f.exit(), None);
    f.advance_past(limits::DEADLINE);
    assert_eq!(f.exit(), Some(Exit::Code(1)));
}
```
Others: `a_live_wait_passes_30_seconds`, `a_wait_timeout_exits_3`,
`help_opens_nothing`, `call_stdin_cannot_also_give_text`,
`usage_exits_1_before_directory_opens`, `ctl_is_configured`; each
under 20 ms, 30 and 300 s in virtual time.
Proof: unit, `cargo nextest run -p temper-client-shell-cli -E
'test(a_stopping_engine_exits_the_wait_only_at_the_link_deadline)'`: a
stopping engine on cue is not the real loop's (decision 10); 08.4 joins
a wait to another call's act; temper 10.1 runs the executable (choice
A); 10.2 cuts an act with SIGTERM.
Size: at most 260 production lines.
Don't: a second `main`; re-render a wait's result; a process test here.
Removes: nothing.

## Interface sketches

`OneShot` owns `Startup`'s descriptors and the lock; `Shell` owns the
second open of `parent`. `exit` ends the call, whatever ended it.
```rust
// crates/temper-client-shell-cli/src/host.rs
use skein_io::kernel::Fd;
/// What `ctl::run` (or a world) opened; `parent` is DIRECTORY's parent.
pub struct Startup {
    pub parent: Option<Fd>,
    /// The directory of the FILE the call reads (temper-05.s1, ruling 111).
    pub file_parent: Option<Fd>,
    pub stdin: Fd, pub stdout: Fd, pub stderr: Fd,
    pub signals: Fd, pub user: u32, pub seed: u64,
}
/// One parsed call; `directory` is DIRECTORY's leaf, `None` for help.
pub struct Invocation {
    pub directory: Option<Box<[u8]>>,
    pub parsed: temper_client_tools::jig::call::Parsed,
}
/// The one-shot host: skein's loop drives it to one `Exit`.
pub struct OneShot { /* root, transport, files, deadlines, output */ }
impl OneShot {
    /// # Errors
    /// Refuses limits the client cannot represent.
    pub fn new(call: Invocation, startup: Startup) -> Result<Self, jig_client::Unusable>;
}
impl skein_shell::Host for OneShot { /* every method, exit included */ }
// src/arguments.rs
use temper_client_tools::jig::{argv::Usage, call::Parsed};
pub fn parse(words: &[&[u8]]) -> Result<Parsed, Usage>;
// src/limits.rs
/// The shipped bounds (tools.md, 12): the tools' root, native, io.
pub struct Limits {
    pub root: temper_client_tools::root::Limits,
    pub connections: jig_client_shell_native::Limits,
    pub io: skein_io::Limits,
}
pub fn native() -> Limits;
pub fn settings(engine: skein_io::Addr, directory: &[u8]) -> jig_client_shell_native::Settings;
// temper-client-tools/src/jig/render/pending.rs
pub fn pending(rows: &[skein_lib::RequestRecord<jig_client::Held>],
    unread: u32, json: bool) -> temper_client_tools::jig::Printed;
// crates/temper/src/ctl.rs
pub fn run(arguments: &[std::ffi::OsString], clock: &skein_shell::Clock) -> u8;
```

## Questions to raise when reached

- **The kernel refuses `Op::Open` of `b"."`** (08.3's world shows it):
  raise it; the second descriptor is then a design question for skein.
- **A unit over 20 ms** (the HTTP-level `fixtures::Engine`): made
  cheaper, never excepted (testing.md 6.1).

## Done when

Each in `~/src/rust/temper`, after `git fetch`:
- `git grep -q -F 'temper-client-shell-cli' origin/main -- Cargo.toml crates/temper/Cargo.toml`
- `git grep -q -F 'Command::Ctl' origin/main -- crates/temper/src`
- `git grep -q -F 'pub struct OneShot' origin/main -- crates/temper-client-shell-cli/src`
- `git grep -q -F 'DIRECTORY is busy with another call' origin/main -- crates/temper-client-shell-cli/src`
- `git grep -q -F "DIRECTORY's store cannot be read" origin/main -- crates/temper-client-shell-cli/src`
- `git grep -q -F 'pub fn pending(' origin/main -- crates/temper-client-tools/src/jig/render`
- `test -z "$(git grep -n -E 'fn main\(|process::exit' origin/main -- crates/temper-client-shell-cli)"`
