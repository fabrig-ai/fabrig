# Session 05: `temper ctl`

`temper-client-shell-cli` hosts one call on skein's loop, with no `main`:
check DIRECTORY, lock for a writer, sign in, finish kept work, then run
and print `jig::Printed`. Reads/waits use empty memory; `pending` only reads
store. The binary and temper 08 run this same host, one program per call.
Needs at start: **the tools' root**. Steps are `temper-05.1` through
`temper-05.6`; the last also needs **the tools' waits** and **the tools' help**.

## Read first

- temper's `docs/design/client/tools.md`: 4, 5.6, 6.2, 6.3, 7 to 9, 11.1,
  11.3 to 11.5, 12, 15; `client.md`: 6.1, 6.6, 7; `domain/testing.md`: 2.4, 2.6, 6.1.
- jig's `docs/design/client/domain/client.md`: 4, 6, 8.2, 11, 12, 16, 20;
  `docs/design/protocol/client.md`: 8, 10, 12.
- skein's `docs/design/io.md`, 5; `kernel.md`, 6.1, 6.2; `shell.md`, 12, 13;
  `programming-model.md`: 2, 5.2, 6.3, 9; `testing-strategy.md`: 2.1, 2.8, 6, 8, 10.
- This plan's `temper/{02-tools-table,03-tools-root,04-waits}.md`; temper 12's
  **the tools' help**; `temper/09-real-loop-stories.md`, 09.2/09.3;
  `temper/10-process.md`, 10.3; decisions 5 to 8; `findings/temper-area-1.md`, 1 to 3, 8.
- jig's `crates/jig-client-shell-native/src/{shell,files,connections}.rs`:
  `Shell::{adopt,loaded,down,kept,close}`, `Files::progress`, `Connections`;
  `crates/jig-client-protocol-link/src/translate/store.rs`, `restore_table`.
- temper's `crates/temper-client-shell-terminal/src/{client,limits}.rs`:
  `NativeClient`, `kept_key`, `native`; `crates/temper/src/{lib,open}.rs`:
  `Command`, `configure`, `run`, `target`. skein's `crates/skein-shell/src/ring.rs`,
  `prepare`; `testing/skein-world/src/real.rs`, `kill`, `settle_children`, `release_killed`.

## What the design asks

- **4, 8.2:** an ordinary Rust host on skein's loop, over jig's native transport.
- **5.6, 8.1, 9:** host tools; FILE/stdin input, never credentials in argv or output;
  owned, private DIRECTORY; 0o600 `address`, one loopback `http://HOST:PORT` line.
- **7, 8.2:** restore, answer and erase before a new act; no `kept: pending`.
- **8.3:** lock DIRECTORY for acts and sign-in/out, retry at 100 ms, fail past
  60 s; reads/waits never load its store; pending/help take no lock either.
- **6.2, 6.3, 12:** one result, the root's status, kept lines on stderr,
  a 30 s link deadline including an unheard request or answer.
- **11.3, domain/testing.md 2.4:** files, lock and exit proven on the real kernel;
  **11.4:** the executable's paths belong to temper 10.

## Choices this plan makes

- **Two native compositions.** `Shell::loaded` reads token and store before
  transport is ready; discarding records violates decision 5. Acts and sign-in/out
  use `Shell`; reads/waits use public `Connections` over `skein_io::Io`, loading
  token only and answering `Save`/`Erase` in memory. No jig change or dummy directory.
- **One owner of each descriptor.** The host opens DIRECTORY through
  `OpenPrivate`, holds that open for its lock, and gives `Shell` a
  separately opened parent descriptor with DIRECTORY's leaf name.
  temper 08 supplies a parent startup root; 09.2's fresh DIRECTORY open
  is the host's `OpenPrivate`, not a shared or duplicated locked open.
  Native `Shell` needs a leaf name, not `"."`; refine 09.2's startup wording accordingly.
- **The lock survives replacement.** The ring uses `flock(LOCK_EX | LOCK_NB)`;
  separate opens contend in one process too. Close its descriptor after storage,
  output and transport settle. On Kill the OS, or `real::World::settle_children`,
  closes it before the parent's `Wait`: 09.2's probe and 09.3's waiter rely on that.
- **Two recovery barriers:** `Root::kept()` counts `Kept` minus `Answered`;
  the host accounts for every `Save`/`Erase` and `Stored`. Zero kept alone
  does not release step 4. The stream record may remain: no request kept
  does not mean a zero-byte file.
- **Deadlines:** `limits::DEADLINE` bounds sign-in, first `me`, restored
  requests, the call's reads/act and continuous offline intervals; retries
  cannot restart it. A live wait with delivered watches lasts until its own
  timeout. Jig reopens stopping watches; this host's deadline gives exit 1.
- **Output:** pass `jig::Printed` through, prefixing failure words with
  `temper ctl: ` once; never prefix JSON or `Request::Kept` lines.
  `Host::exit` reports the status after all operations and descriptors settle.
- **`pending`:** decode the envelope and jig's versioned records, without
  `Start`; skip `STREAM_KEY`. Print application key `A:N` as temper 03 does,
  tool name and `first_sent` (absent means not yet sent); no second history.
- **Before the run, 05.5:** the design leaves `pending`'s output for an unsupported
  record unspecified. The designer must settle it; do not silently omit such a request.

## Rules that bind it

- Unit tests use explicit completions/time, under 20 ms: no real I/O or sleeps.
  Limits come from the tools' preset and native profile; memory and operation
  bounds include the host's queues and output.
- Runtime I/O uses host submissions: no global printing, exit or signal handler.
  No forge, fake forge, fake LLM or provider edits; `adopt` is an ordinary call.
- Keep the table/root/waits/help owners; STATUS and design remain the designer's.

## Increments

### 05.1 The crate, DIRECTORY and its lock
Scope: the host's input classification, private directory and address,
and lock policy (8.1, 8.3); no command exposed by the binary yet.
Needs: nothing beyond the session's start.
Files: workspace `Cargo.toml`, `Cargo.lock`; new
`crates/temper-client-shell-cli/{Cargo.toml,clippy.toml,src/lib.rs,
src/directory.rs,src/arguments.rs,src/limits.rs,src/tests.rs,src/tests/fixtures.rs}`.
Shape:
- Ordinary Rust, `forbid(unsafe_code)`; workspace dependencies on tools,
  jig-client, jig-client-shell-native, jig-client-protocol-http,
  jig-client-protocol-link, jig-client-protocol-wire, skein-io,
  skein-lib, skein-json and skein-shell. No terminal or view dependency.
- `arguments::parse` calls the tools' table, then classifies read, wait,
  act or host tool. Keep `Parsed::File` for 05.6. `help` needs no directory.
- `directory::Directory` uses `FileIo` for `OpenPrivate`, safe loads,
  `Store`, `Lock` and `Close`. Parse `address` as exactly one numeric HTTP
  socket address; allow 127/8 and ::1, refuse names, paths and other hosts
  before a token is sent. Sign-in supplies its address from the link.
- Take the lock before constructing a disk `Shell` or loading its store.
  `Busy` arms the next try at `LOCK_TRY`; past `LOCK`, print
  `DIRECTORY is busy with another call`, status 1. Other refusals fail at once.
- `limits::native` uses the tools' client preset, reexports its constants,
  and writes native transport/io numbers; compare them through a terminal dev-dependency.
Test shape: `src/tests.rs`, extended throughout; its fixtures answer
`Directory` file requests with shipped limits and explicit time. Reference:
```rust
#[test]
fn a_busy_directory_is_retried_at_100_ms_and_refused_past_60_s() {
    let mut f = fixtures::directory_for_act();
    f.busy();
    assert_eq!(f.next_deadline(), Some(f.started() + limits::LOCK_TRY));
    f.advance_past(limits::LOCK);
    assert_eq!(f.failure(), Some((1, b"DIRECTORY is busy with another call".as_slice())));
    assert!(!f.store_loaded());
}
```
Others: `only_acts_and_sign_in_and_out_take_the_lock`,
`non_loopback_is_refused_before_a_token_is_sent`,
`unsafe_directories_and_files_are_refused`, `address_is_one_http_line`,
`the_transport_profile_matches_native`; each under 20 ms.
Proof: unit, `cargo nextest run -p temper-client-shell-cli -E
'test(a_busy_directory_is_retried_at_100_ms_and_refused_past_60_s)'`,
under 1 s. No world hosts this crate yet; 08's sign-in story checks
DIRECTORY 0o700 and token/store/address 0o600; 09.3 proves contention.
Don't: lock `store`; pass a lock descriptor to another program; shorten
the lock wait to make a test fast.
Size: at most 280 production lines.
Removes: nothing.

### 05.2 The native transport without a file store
Scope: native HTTP with memory acknowledgements for reads/waits (8.3).
Needs: the crate and native limits (05.1).
Files: new `src/transport.rs`; `src/{lib,limits,tests}.rs`, `src/tests/fixtures.rs`.
Shape: `transport::Memory` joins public `Connections::{down,up,take}` to
`skein_io::Io`, with bounded socket and operation maps. No `Shell` or files.
Queue `Save`/`Erase` and return one `Stored` per request in order after the
step. Carry exchanges, streams, close and their deadlines through native
`Connections`; do not implement HTTP or jig's retries again. The adapter
publishes queues, deadlines and bounds for the host, which 05.3 builds.
Test shape: `src/tests.rs`; fixtures script transport completions under
05.1's profile, recording submitted operations. The reference test:
```rust
#[test]
fn memory_store_replies_are_in_order_without_file_operations() {
    let mut f = fixtures::memory_transport();
    f.save_stream();
    f.erase_stream();
    f.iterate();
    assert_eq!(f.events(), [Event::Stored, Event::Stored]);
    assert!(f.submissions().is_empty());
}
```
Others: `an_exchange_uses_native_http`, `a_closed_stream_settles_its_socket`;
each under 20 ms, no real socket.
Proof: unit, `cargo nextest run -p temper-client-shell-cli -E
 'test(memory_store_replies_are_in_order_without_file_operations)'`, under
1 s. No caller yet; 05.3 connects it to the root, 08 to the real kernel.
Don't: instantiate `Shell` and discard loaded records; create a dummy directory.
Size: at most 240 production lines.
Removes: nothing.

### 05.3 Reads with no disk store
Scope: a skein `Host` around the root; reads through native HTTP with an
empty memory store, output and settled exit (4, 8.2, 8.3).
Needs: the memory transport (05.2).
Files: new `src/{host,output}.rs`; `src/{lib,limits,tests}.rs`,
`src/tests/fixtures.rs`; `crates/temper-client-tools/src/root.rs`
(a read-only `Root::client` accessor if temper 03 has not supplied it).
Shape:
- The host routes file, transport and output completions under disjoint tokens.
- Open/read only token and address. Start with `Start { kept }` empty,
  then `SignIn { with: Held(token) }`; wait for `Request::Ready` before
  the call. 05.2 answers storage in memory; never instantiate `Shell` here.
- Drive the root's `step`, `fire`, `next_deadline`; observe
  `Root::client().link()`. Track an outstanding read's first issue even
  if the stream stays live; the exchange's retries cannot extend 30 s.
- `output` owns stdout/stderr writes, including partial writes. Drain
  them, close streams and files, then report `Exit::Code(printed.status)`.
  `Host` implements every skein method, including `next_policy_deadline`,
  `worst_case`, `operations`, `is_empty` and `exit`.
Test shape: `src/tests.rs`; `fixtures::read` drives the real root/client
with transport completions; `opened` records submitted opens. Reference:
```rust
#[test]
fn a_read_never_opens_the_store_or_retires_another_stream() {
    let mut f = fixtures::read(&["me", "--json"]);
    f.sign_in_and_deliver_me();
    f.drain();
    assert!(!f.opened(b"store"));
    assert!(!f.sent_stream_gone());
    assert_eq!(f.exit(), Some(Exit::Code(0)));
}
```
Others: `a_read_waits_for_me`, `an_unheard_read_exits_1_at_the_link_deadline`,
`partial_output_writes_finish_before_exit`, `host_bounds_cover_its_holders`;
each under 20 ms, virtual time for deadlines.
Proof: unit, `cargo nextest run -p temper-client-shell-cli -E
'test(a_read_never_opens_the_store_or_retires_another_stream)'`, under
1 s. 08's sign-in and first chat story connects the host on the real
ring and checks its exit and resource release; that harness is not built yet.
Don't: load then discard records, suppress storage replies, or implement jig's reconnection policy.
Size: at most 300 production lines.
Removes: nothing.

### 05.4 Acts: recover, erase, then send
Scope: disk `Shell`, the one-shot ordering and recovery barrier (7, 8.2).
Needs: the hosted reads and settled output (05.3).
Files: new `src/recovery.rs`; `src/{host,transport,limits,tests}.rs`,
`src/tests/fixtures.rs`; `crates/temper-client-tools/src/root.rs` (kept-line format).
Shape:
- Once locked, adopt a separately opened parent in native `Shell`;
  `loaded` gives the token and records. Send `Start` with those records,
  then sign in. Set root's kept-line JSON mode before `Start`; jig resends.
- `Recovery` waits for `Ready`, `Root::kept() == 0`, and all storage
  terminals successful. Root's count is its `Told::Kept` minus
  `Answered`; stream records are not requests. Drain the root's outputs
  and complete the erase's `Stored` before admitting `Event::Call`.
- Queue `Save`/`Erase`; give `Shell` only one at a time until its
  `Stored`/`NotStored`, so its file edits cannot merge two promised
  terminals. No new act after a failed write. A printed act also waits
  for its erase to be stored before successful exit.
- Each restored request has a 30 s answer deadline from recovery;
  the new act has one from its issue. Offline and initial-link deadlines
  also apply. A deadline or termination signal stops new work, drains
  already issued persistence, closes transport and preserves unanswered
  records. Never sign out as part of stopping a call.
Test shape: `src/tests.rs`; extend 05.3's driver with `Shell` file completions:
```rust
#[test]
fn a_new_act_waits_for_the_kept_answers_erase_to_be_stored() {
    let mut f = fixtures::act_with_kept("same-name");
    f.answer_kept();
    assert_eq!(f.kept(), 0);
    assert!(!f.new_act_sent());
    f.store_erase();
    assert!(f.new_act_sent());
    assert_eq!(f.kept_lines(), 1);
}
```
Others: `two_writes_receive_two_ordered_terminals`,
`failed_erase_does_not_admit_the_new_act`,
`an_unheard_act_is_kept_after_30_seconds_even_with_a_live_stream`,
`a_signal_closes_without_erasing_unanswered_work`,
`a_kept_json_line_is_written_without_a_prefix`; each under 20 ms.
Proof: unit, `cargo nextest run -p temper-client-shell-cli -E
'test(a_new_act_waits_for_the_kept_answers_erase_to_be_stored)'`, under
1 s. 09.2's lost-answer story checks the durable record and lock release
after Kill; 09.3's killed-holder story checks the next writer recovers it.
Don't: use a count of `Answered` alone; retain an old application key
for the new call; erase work on link failure; add a kill seam to the host.
Size: at most 280 production lines.
Removes: nothing.

### 05.5 Sign-in, sign-out and `pending`
Scope: 5.6's file-backed host tools, using the same host and lock.
Needs: the disk host and ordered persistence (05.4).
Files: new `src/{signing,pending}.rs`; `src/{host,directory,tests}.rs`,
`src/tests/fixtures.rs`.
Shape:
- `sign-in FILE|-` reads a bounded link, parses it with
  `jig_client_protocol_http::printed`, checks loopback, and sends
  `With::Link`. Keep the resulting `client.link().credential` with
  `Shell::keep`; wait for `kept`, the address's atomic `Store` terminal,
  and live `me`. Store `http://HOST:PORT\n`, never the link's secret.
- `sign-out` sends jig's `Event::SignOut` and waits for its terminal
  and the token's erasure through `Shell::keep(None)`. As native does,
  track signing exchanges; do not mistake an initial SignedOut state
  for completion. The lock covers both host tools' whole runs.
- `pending` loads only `store`, without token, address, lock, client or
  transport. Bound its envelope by native `body_bytes`; use `Reader`,
  `Kept::decode`, `TableEntry::decode`, `translate::store::restore_table`,
  and the pinned version/owner checks. Reject malformed envelopes;
  inspect no payload of an unsupported record. Do not rewrite the file.
- Print rows in storage-key order: `A:N TOOL FIRST_SENT`, or one JSON
  object with `pending` rows (`key`, `tool`, `first_sent`, integer UTC
  nanoseconds or null). Empty text is empty, as 09.2/09.3 assert.
  Identify tool names from the stored asks; no forge payload is decoded
  here. The served forge family identifies `adopt` through its owner.
Test shape: `src/tests.rs`; fixtures encode records through jig's wire
codecs and the native envelope, not a client created by `pending`.
```rust
#[test]
fn pending_reads_only_the_store_and_starts_no_client() {
    let mut f = fixtures::pending_with_one_chat();
    f.drain();
    assert_eq!(f.loaded_names(), [b"store".as_slice()]);
    assert!(!f.client_started());
    assert!(f.network_submissions().is_empty());
    assert_eq!(f.pending_tools(), ["chat"]);
}
```
Others: `sign_in_keeps_token_and_address_before_success`,
`sign_out_waits_for_the_token_erase`, `a_refused_link_never_prints_its_secret`,
`pending_ignores_the_stream_record`, `bad_store_bytes_are_refused_without_writes`;
each under 20 ms.
Proof: unit, `cargo nextest run -p temper-client-shell-cli -E
'test(pending_reads_only_the_store_and_starts_no_client)'`, under 1 s.
08's sign-in story checks private modes and persisted address on the
kernel; 09.2 checks pending before and after recovery. No process proof here.
Don't: load token or address for `pending` despite 8.1's general wording;
print credentials; change the native store's format or a forge interface.
Size: at most 300 production lines.
Removes: nothing.

### 05.6 Waits, help and the executable
Scope: the complete command line (8.1), wait deadlines, help, and the
binary's `ctl` branch; the host is now callable by temper 08 and 10.
Needs: the host tools (05.5); the tools' waits; the tools' help.
Files: `src/{arguments,host,output,tests}.rs`, `src/tests/fixtures.rs`;
new `crates/temper/src/ctl.rs`; `crates/temper/src/{lib,tests}.rs`,
`crates/temper/Cargo.toml` (the cli dependency).
Shape:
- Resolve `Parsed::File` through temper 12's JSON parser. Read TEXT `-`
  to EOF within `ask_bytes`; reject a second stdin use, including TEXT
  under `--call -`. Finish validation before opening DIRECTORY.
- `help` without DIRECTORY and `DIRECTORY help` use temper 12's renderer;
  `help TOOL --json` uses its table record. No directory or client for help.
- Waits take 05.3's memory path. Include the root's deadline in both
  deadline methods; process inputs before firing timers. A delivered
  condition wins over its timeout in that iteration. A healthy unchanged
  wait may exceed 30 s; a stopping/offline engine may not extend the link
  deadline by reopening a watch. Keep exits 0/2/3 from `Printed`.
- `Command::Ctl { arguments }`, `configure`, `run` and `USAGE` delegate
  to `ctl::run`. It parses before startup, supplies parent/stdio/signal
  descriptors, effective user and seed, and drives `Host` with a `Kernel`.
  It returns the host's exit; only existing `main` converts to `ExitCode`.
Test shape: `src/tests.rs`, existing binary dispatch tests; the host reference:
```rust
#[test]
fn a_stopping_engine_exits_the_wait_only_at_the_link_deadline() {
    let mut f = fixtures::wait_started();
    f.engine_stopping();
    assert_eq!(f.exit(), None);
    f.advance_past(limits::DEADLINE);
    f.drain();
    assert_eq!(f.exit(), Some(Exit::Code(1)));
    assert!(!f.opened(b"store"));
}
```
Others: `a_live_unchanged_wait_passes_30_seconds`,
`a_wait_timeout_keeps_status_3_and_its_digest`,
`help_opens_nothing_and_uses_the_tools_renderer`,
`call_stdin_cannot_also_supply_text`, `usage_fails_before_directory_open`;
each under 20 ms. All 30/60/300 s tests advance virtual time.
Proof: unit, `cargo nextest run -p temper-client-shell-cli -E
'test(a_stopping_engine_exits_the_wait_only_at_the_link_deadline)'`,
under 1 s. 08's “one client's wait met by another client's act” proves
the assembled host; 10.3's `support::ctl` runs the executable for argv,
stdin, printing and OS exit conversion. Kernel modes and locks stay in 08/09.
Don't: make a second `main`; exit on `WatchEnd::Stopping`; intercept
or re-render a wait's result; add a process-tier lock or modes test.
Size: at most 280 production lines.
Removes: nothing.

## Interface sketches

```rust
// crates/temper-client-shell-cli/src/host.rs: owned descriptors, no global stdio.
/// Supplied by the caller, owned by this call; parent is absent for directory-free help.
pub struct Startup {
    pub parent: Option<Fd>, pub stdin: Fd, pub stdout: Fd, pub stderr: Fd,
    pub signals: Fd, pub user: u32, pub seed: u64,
}
/// One parsed invocation; the directory is a leaf beneath Startup.parent, absent for help.
pub struct Invocation { pub directory: Option<Box<[u8]>>, pub parsed: jig::call::Parsed }
/// One call on skein's loop; completion is Host::exit after settlement.
pub struct Host { /* root, native transport, files, deadlines, output */ }
impl Host { pub fn new(call: Invocation, startup: Startup) -> Result<Self, jig_client::Unusable>; }
// crates/temper-client-tools/src/root.rs: the root still owns and steps its client.
impl Root {
    pub fn client(&self) -> &jig_client::Client;
    /// Set before Start: recovery's kept lines precede Event::Call and its json field.
    pub fn kept_json(&mut self, json: bool);
}
```

05 owns the host. 08 maps inherited pipes and a fresh parent root into
`Startup`; the host opens DIRECTORY. 10.3 spawns the executable and writes
stdin. Neither caller implements storage, lock retry or call termination.

## Questions to raise when reached

- Unit tests past 20 ms or 08's ownership checks failing: raise measured holders/times and limits.

## Done when

Each in `~/src/rust/temper`, after `git fetch`:
- `git grep -q -F 'temper-client-shell-cli' origin/main -- Cargo.toml crates/temper/Cargo.toml`
- `git grep -q -F 'Command::Ctl' origin/main -- crates/temper/src`
- `git grep -q -F 'temper ctl: ' origin/main -- crates/temper-client-shell-cli crates/temper/src/ctl.rs`
- `git grep -q -F 'Request::Lock' origin/main -- crates/temper-client-shell-cli/src`
- `git grep -q -F 'DIRECTORY is busy with another call' origin/main -- crates/temper-client-shell-cli/src`
- `git grep -q -F "the engine's address must be on loopback" origin/main -- crates/temper-client-shell-cli/src`
- `git grep -q -F 'fn exit(' origin/main -- crates/temper-client-shell-cli/src`
- `test -z "$(git grep -n -E 'fn main\(|process::exit' origin/main -- crates/temper-client-shell-cli)"`
