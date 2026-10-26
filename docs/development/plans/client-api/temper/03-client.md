# Session 03: temper's client on `jig-client`

jig's client becomes a component an application holds (jig 10 to 13, and
21): the
link speaks jig's messages and holds the credential, the default HTTP
mapping's client end is a crate of its own, the contract and the
request table's records move into `jig-client-domain`,
`jig_client::Client` composes the domain and the link, and the roots and
the reference UI stand on it, routing by asker. temper follows each step
the way jig changes its own reference code, since temper's client root,
native client and view root are copies of jig's (decisions.md, 6;
`findings/temper.md`, 3). First its terminal takes skein's text shell's
loop, `skein_ui_text::Terminal`; then its native client signs in through
the link over jig's new transport; its root takes the contract from
`jig-client-domain`, then `jig-client-view`'s paths; then it holds
`jig-client`, routes by asker, sends the forge's adoption as the
family's bytes and names temper's `Owner` in its wire crate; last, jig's
pages are `jig_client_view::Ui` on the client, arranged by temper's view
(transitional: decisions.md, 7, ended by temper 07.4). `tests/client`
stays joined to temper's engine root throughout.

## Read first

- temper's `docs/design/client.md`: 2, 3 (3.1 to 3.4), 4.5, 4.6, 6
  (6.1 to 6.3, 6.6, 6.7), 7, 10 (the bullets "`tests/client`" and "The
  one-process world").
- temper's `docs/design/credentials.md`, 6; `docs/design/domain/testing.md`,
  2.5 (the person "in front of temper's terminal client") and 6.3.
- skein's `docs/design/ui-text.md`, 7 (its loop).
- jig's `docs/design/client/domain/client.md`: 1, 4, 12;
  `client/domain/root.md`: 3, 4, 5, 7, 15; `protocol/client.md`, 17.4.
- This plan's `jig/10-transport.md`, `jig/11-client-domain.md`,
  `jig/12-client.md`, `jig/13-reference-ui.md` and `jig/21-ops-and-ui.md`:
  their Shapes, and their changes to `ops`'s client root
  (`examples/ops/client-domain` and `client-view`, 21.2's to 21.4's
  commits of their own), to the test application's root
  (`testing/jig-client-test-domain`) and native host
  (`tests/client-protocol/src/native.rs`), and to `jig-client-world`;
  `temper/01-skein-ui-repin.md`, 01.3 (the renderer and command reader
  already on `skein-ui-text`); `decisions.md`, 6 and 7;
  `findings/temper.md`, 2, 3 and 6; `findings/wire-and-client.md`, 3
  and 5.
- skein's `programming-model.md`: 2.1, 3, 4.2, 5.1, 6.3, 9 (by number
  only); skein's `testing-strategy.md`: 2.2, 2.5, 2.8, 7 (by number
  only).

## What the design asks

- **The terminal's loop** (`client.md`, 6.2): "One host on skein's loop
  (skein-shell's `drive`...): temper's client root, its view,
  `jig-client-shell-native`'s transport, and skein's text shell's
  terminal (skein's `ui-text.md`, section 7)"; its settings: "the name
  `temper`; the longest line ...; a print interval of one second, a
  guide; and the status region, the banner's link state".
- **The root holds `jig-client`** (3): "It holds `jig_client::Client` as
  a component and calls it; the client never calls it back, and the root
  routes nothing of jig's".
- **Owners and routing** (3.1, 3.3): "Each is the asker of every handle
  and key it gives `jig-client`"; what the client says of a handle or a
  key "goes to the child its asker names, matched whole; an asker that
  names neither is a fact, and reaches no child".
- **The forge's request as bytes, and `Owner`** (4.6): "the request,
  encoded by this crate as the family's bytes at `CLIENT_VERSION`";
  "`Owner`: temper's owners, one enum the engine's root and the client
  root share, with the functions from an owner to its number and back
  ...: `Forge`, `FORGE`'s number; and `Pages`, temper's pages, 2, a
  number no connector takes, never reused".
- **The credential** (7; `credentials.md`, 6): kept in `DIRECTORY/token`
  once signed in by a link, given back as held at the next `temper open
  DIRECTORY ADDRESS`, erased when signed out; never printed, never in
  the small store.
- **The joint world** (10; `domain/testing.md`, 6.3): "temper's client root and
  view, with skein's fake person acting on the tree, joined to temper's
  engine root ..., with jig's client worlds' faults and jig's check".

## Choices this plan makes

- **jig's diffs are the port.** Each increment follows the jig step it
  needs by that step's own changes to `ops`'s client root, the test
  application's root and native host, and `jig-client-world`, file for
  file where temper's copy matches (`findings/temper.md`, 3: 484 of 593
  non-blank lines of the root match `ops`'s); what is temper's own (the
  forge's side, its form, its words, the terminal's lines) changes only
  where those diffs reach it.
- **Each repin pins the landing of the newest jig step its Needs name**
  (README, Rules, "Pins"), never jig's `origin/main`, so that each
  increment follows one jig step's diff within its Size; 03.6 pins jig
  21.4.
- **The credential's file stays the native shell's, at the host's
  word.** jig 10.1 keeps it in `jig-client-shell-native`
  (`Shell::loaded`, `Shell::keep`), written when its host says; temper's
  host calls `keep` with the credential the link (later `client.link()`)
  holds, on `Signing::SignedIn`, and `keep(None)` on `SignedOut`. The
  file, its mode and when it is written are `client.md`, 7's; the code
  that writes it is jig's (a difference from `client.md`, 2's "keeping
  the credential's file", reported as a design question).
- **`Owner` lands with both owners in 03.5, `Pages` without a child until temper
  06.1.** The wire crate's `Owner { Forge, Pages }` is the design's; the
  engine's routing of the forge's family matches it with no `_` arm (a request of
  `Pages`, which no build sends, is answered undecodable). The root's
  own owners, after jig 13.4's pattern (`Owner::{Jig, First, Second}`),
  are `Owner::{Jig, Forge}`: jig's reference UI under jig's asker,
  65536, until temper 07.4, and the forge's side; an asker naming
  `Pages` or no owner is a fact until temper 06.1.
- **The reference UI's own records are not kept** (`client.md`, 3.4:
  "temper's pages keep no record of their own"): from 03.4 temper's root
  drops what jig's reference UI asks to save or erase, so a draft on
  jig's pages is not kept across a restart.
- **jig's check** in this session is the referee of `jig-client-world`
  (`tests/client/src/harness/referee.rs` at jig's base: "what they saw
  is what they decided: revision differs"), which temper's world runs
  through jig's harness and keeps running; jig 15.5's check in
  `jig-client-test-kit` joins temper's stories in temper 06.

## Rules that bind it

- **Each increment whose Needs name a jig step starts with a repin
  commit of its own** onto that step's landing (README, Rules, "Pins"),
  found by `git -C ~/src/rust/jig fetch origin && git -C ~/src/rust/jig log
  origin/main --all-match --grep='^Plan: client-api$' --grep='^Step:
  jig-NN.k$' --format=%H -1`
  with the step's number, never jig's `origin/main`; when `git -C
  ~/src/rust/jig merge-base --is-ancestor <that landing> <temper's
  present jig pin>` succeeds, no repin commit is made. The repin: `cargo update -p jig-domain-core --precise <the landing>`, then `cargo update -p skein-lib --precise <its pin of skein, or temper's
  skein floor while that pin does not hold it>`, none when temper's skein
  already holds it, and `cargo update
  -p smith-host-protocol --precise <its pin of smith>`; then `grep -o
  'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' Cargo.lock |
  sort -u` prints three lines: `jig.git#<the landing>`, smith's line
  equal to that of `git -C ~/src/rust/jig show <the landing>:Cargo.lock`,
  and skein's a commit that holds it or, while it does not hold
  temper's skein floor, the floor (`git -C ~/src/rust/skein merge-base
  --is-ancestor <that commit> <skein's line>` succeeds; README, "Pins",
  "The terminal's room", "The print's room" and "temper's skein
  ahead"); a second revision of a source still there after a
  second `cargo update` is a question.
- **A repin's repairs outside its Shape** follow jig's own diffs and
  are counted with `git diff --numstat origin/main...HEAD -- <the
  changed paths outside the increment's Files> ':!Cargo.lock'`, added
  lines, at most 200 (README, "Pins"); past it, a question.
- **No collapsed wire here:** jig 14 needs jig 21 at its start, so no
  repin of this session (onto jig 10, 11, 13.2, 21.2, 21.4) takes jig
  14's or 22's records.
- **temper's formats** (README, "Format versions"): the client's small
  store records of temper's root move to version 2 in 03.3 (the table's
  records carry jig 11.3's `Held`), and go in 03.5 (the client keeps
  them); temper's engine store's `STORE_VERSION` moves to 2 once, in the
  first temper increment whose repin includes jig 02.3's landing
  (people v3: `git -C ~/src/rust/jig merge-base --is-ancestor <jig
  02.3's landing> <the pinned commit>` succeeds), with
  `store-v2.schema` and `CoreRefused`'s bound from jig's people v3
  ceilings; if temper 02.1 did not, it is 03.2, this session's first
  repin, or the first after it that includes it.
- **jig 08.4's census:** the first increment of this session whose
  repin includes jig 08.4's landing repairs temper's census walk, as the
  README's "jig 08.4's census" says.
- **The meeting point:** each increment's story runs against temper's
  engine root, in `tests/client`, the one-process world or the process
  tier, never only against `jig-client-scripted-engine` (README, Rules).

## Increments

### 03.1 The terminal's loop on `skein_ui_text::Terminal`

Scope: skein's text shell's terminal, `skein_ui_text::Terminal`, as the
terminal's loop: reading lines, the print interval, printing and the
output, under temper's settings (the name `temper`, the longest line, a
one-second interval, the status region the banner's link state); the
host keeps temper's own lines (signing in, its retries, a refused link,
the credential's and the store's notices, signed out) and says
them with `Terminal::say`, each only within `Terminal::room`, keeping
what has no room, an equal line once, and taking no command while a
line waits (`client.md`, 6.2, 6.3, 6.6, 6.7; skein's `ui-text.md`, 3, 4
and 7). `temper: closed` is the terminal's own line (`Terminal::close`
says it): the host never says it. On `q`, the end of
stdin or a termination signal the terminal "starts no new print, writes
what it has begun, and says it is closed (`NAME: closed`); the host then
closes its application" (`ui-text.md`, 7): the host closes the client
once `Terminal::closed()` is true, after the drain, not before it.
temper 01.3 put the renderer and the command reader
on `skein-ui-text`; this increment replaces the rest of `terminal::Loop`.
Needs: the session's start (temper 02.1 on temper's `main`),
skein-01.s2's landing, which holds skein-01.s1's (the settings' lines
and `Terminal::room`; the line bytes, `Terminal::worst_case` and
`Terminal::operations`; README, "Pins", "The terminal's room"), and
forgejo-17's skein step's landing, temper's skein floor, which
skein-01.s2's holds (each print's room from its own tree, so
`Terminal::new` takes temper's view limits; README, "Pins", "The
print's room").
Files: `crates/temper-client-shell-terminal/src/` (names after temper
01): `terminal.rs` (318 lines at the base: `Loop`'s reader, pacing,
printing and output go; its notices' lines stay, said through the
terminal), `lines.rs` (59) and `tests/lines.rs` (39), which go with
`Lines`, the line reader, whose only user is `terminal.rs` (`:4`, `:76`,
`:104`, `:110`), `terminal/run.rs` (327: the host adopts the pipes through
`Terminal::adopt` and routes `Terminal::owns` completions; its own
`Writing` and stdin buffer go), `terminal/native.rs`, `limits.rs`
(`limits::terminal()`'s line, interval and lines feed `Settings`),
`lib.rs`, the crate's `Cargo.toml` (`skein-sim` as a dev-dependency,
below) and `Cargo.lock` (the skein move, first, and the dev-dependency);
`crates/temper/src/open.rs:74-120` (the loop made from
`skein_ui_text::Settings`); tests: `tests/terminal.rs` (244 lines, 12
tests), `tests/support/mod.rs` (271), `tests/fuzzy_terminal.rs`,
`src/tests.rs`. Users found by `git grep -n -e 'terminal::Loop' -e
'Loop::new' -e 'render::Frame' -e 'render::bounds' -- '*.rs'` in temper
(at the base: `open.rs:101`, `tests/support/mod.rs:172`,
`tests/terminal.rs:182`; `terminal/run.rs` writes `super::Loop`).
`crates/temper/tests/support/person.rs:153` names `terminal::Notice`,
which stays.
Shape:
- **The skein move,** 03.1's first commit, on its own (README, "Pins",
  "The terminal's room" and "The print's room"): `cargo update -p
  skein-lib --precise <temper's skein floor>`, none when temper's pin
  already holds the floor (forgejo-17's temper step moved it first);
  after it, `grep -o
  'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' Cargo.lock |
  sort -u` prints three lines, jig's and smith's as before and skein's
  that landing, and `git show --stat --format= HEAD` lists `Cargo.lock`
  alone. A build failure in a jig or smith crate after it is a question.
- `skein_ui_text::Terminal::new(Settings { name: Box::from(*b"temper"),
  longest, interval: Duration::from_secs(1), status: Some(<the banner's
  link region>), lines: limits.lines, line_bytes }, &view_limits)
  .expect("terminal settings")` (`limits::terminal().lines`, 32,
  `limits.rs:181`: temper's lines, `client.md`, 6.2; `line_bytes`
  DIRECTORY's bytes and 160, the line `Loop::worst_case` counts today,
  `terminal.rs:300`, `client.md`, 6.2)
  (`Settings.name` is a `Box<[u8]>`, `skein-ui-text/src/terminal.rs:17`;
  `new` returns `Option<Terminal>`, and `open.rs` expects its other
  startup parts the same way); the host's iteration, in `ui-text.md`,
  7's order: the client's steps, the view built, `Terminal::iterate(now,
  tree, changed)`, each `Command` from `Terminal::commands()` acted on
  the client (`Act` then, when `submit`, `Act::Submit`), the client's
  notices said as temper's lines (`temper: ...`, 6.3's form).
- **Tags.** skein's terminal tags its operations with `1 << 62`
  (`skein-ui-text/src/terminal.rs:12`), the bit `terminal/run.rs:13`'s
  `TAG` uses today: the host's own io (the termination signals) takes
  `1 << 61`, and the host gives a completion to the terminal when
  `Terminal::owns` is true, before it reads its own tag.
- **temper's lines** (`client.md`, 6.3): the host takes every
  `client.notice()` at each iteration, whatever the room (the client's
  queue holds one iteration's notices, `client.rs:87`), in place of
  today's `while self.lines.room() > 0` (`terminal.rs:169`); each
  notice's effect (the retry's deadline and backoff, the failure, the
  close) happens then. Its line goes to a queue of waiting lines; a
  line equal to one already waiting adds none. Of the `Notice`s that
  print a line, `SignInAgain`, `StoreFailed` and `SignedOut` are each
  one line; a `Credential` is a line of its own for each request the
  client sent, so the queue holds `table.requests` (16, `limits.rs:63`)
  plus 3. The host says waiting lines in order while
  `Terminal::room()` is above 0, says none once it has closed the
  terminal, and pops no `Terminal::commands()` while a line waits (the
  terminal then reads no further, `ui-text.md`, 7). Never `say` without
  room: it panics. The waiting queue's capacity, `table.requests + 3`,
  reaches `Loop::new` from its callers, since `limits::Terminal` has no
  `requests` (`terminal.rs:62-90`): `open.rs:101` passes
  `bounds.client.table.requests`, and `tests/support/mod.rs:172` and
  `tests/terminal.rs:182` pass `limits::native()`'s; `Loop::worst_case`
  (`terminal.rs:300-308`, added into the host's memory at
  `run.rs:68-69`) charges that queue, each line at the settings' line
  bytes, in place of `limits.lines`.
- **The host's memory and ring** (`client.md`, 6.8; skein's
  `ui-text.md`, 3, "Its worst case and its operations"): `run.rs`
  (`Terminal::new`, `:62-76`) adds `skein_ui_text::Terminal::worst_case(&settings,
  &view_limits)` to its memory and `skein_ui_text::Terminal::operations(&settings,
  &view_limits)` to its operations, each read from the settings before
  `Terminal::new` takes them, and expected as `Terminal::new` is. Of
  `Loop::worst_case` stays only what `Loop` holds: the waiting lines,
  the directory, the link and the failure line; skein's `worst_case` of
  the view, the print's numbers, the line reader and the output buffer
  leave it, being skein's terminal's. The host's own io, the termination
  signals alone, counts by `skein_io::worst_case` and
  `skein_io::operations` of its own limits, as now.
- temper's `terminal::Loop` keeps only what the text shell does not: the
  sign-in's retry and backoff (6.7), the exit status and its stderr line
  (6.1), the waiting lines and the order in which they are said.
Test shape: the machine world, `tests/terminal.rs` on
`tests/support/mod.rs` (its limits from its own `tree_limits()`,
`support/mod.rs:10`). The world drives skein's terminal through its
pipes as skein's own tests do, on `skein_sim::Sim`
(`tests/ui-text/tests/terminal.rs:20-42` at skein `f12f29d4`): the
crate's dev-dependency `skein-sim`, by the git URL
`tests/one-process/Cargo.toml:44` gives it, at temper's skein pin. Its
tests of the reader and of pacing
(`a_change_within_the_interval_prints_once_when_it_ends`,
`an_act_prints_at_once_after_the_next_build`,
`a_due_print_waits_for_stdout_then_shows_the_tree_then`,
`a_command_cut_at_any_byte_acts_once`,
`a_full_shell_queue_preserves_the_unconsumed_input`) are skein's
`tests/ui-text`'s now and go with the code they test; temper's lines'
tests keep their names and expected values
(`the_end_of_stdin_closes_and_exits_zero_after_the_drain`,
`an_interrupt_closes_without_signing_out`,
`signing_in_without_an_answer_tries_again_after_its_backoff`,
`a_refused_link_exits_one_with_its_line`,
`a_credential_and_store_notice_print_once_in_order`,
`signing_in_cancels_the_pending_link_retry`,
`a_refused_directory_exits_one_with_its_line`). One order changes, under
`ui-text.md`, 7: in `the_end_of_stdin_closes_and_exits_zero_after_the_drain`,
`assert!(world.client.closed, …)` (`tests/terminal.rs:59`) moves after
`world.stdout_takes()`, since the host closes the client after the
drain; its name, its assertions and their expected values stay. One new
test, under 0.5 s (`World::signed_in()` has already printed at t=0, so
both finds start after the output before the notice):
```rust
#[test]
fn temper_lines_are_said_before_the_next_print() {
    let mut world = support::World::signed_in();
    let first = world.all_output().len();
    world.client.notice_at_once(Notice::StoreFailed);
    world.client.change_at(Duration::from_millis(10));
    world.run_until(Duration::from_secs(2));
    let output = world.all_output();
    let line = skein_lib::bytes::find_from(&output, b"temper: cannot write the small store", first).expect("temper's line");
    let page = skein_lib::bytes::find_from(&output, b"== ", first).expect("a print");
    assert!(line < page, "temper's line before the next print");
}
```
(`World::all_output`, the bytes written so far, is new in
`tests/support/mod.rs`, beside `last_output`.) One unit test of the
waiting lines, under 20 ms, in `src/tests.rs`:
`an_equal_line_waits_once_in_the_order_it_came` (`SignInAgain`,
`StoreFailed`, `SignInAgain`, two `Credential`s of two parties waiting,
then a room of 1, then of 8: said `SignInAgain`, then `StoreFailed` and
the two credentials, and nothing more). No world test of a held stdout:
the room and the held input are skein's, tested by skein-01.s1, and the
line bytes, the worst case and the operations by skein-01.s2. The
unit test `the_native_limits_make_the_terminal` (`src/tests.rs`;
`client.md`, 10), which forgejo-17's temper step lands on `Loop::new`,
stays, on `Terminal::new` with this increment's `Settings` and
`limits::view(&limits::native())`'s tree limits: it returns `Some`, and
so do `Terminal::worst_case` and `Terminal::operations` of the same. If
this increment lands first, it adds the test. Under ruling 111 it is
the check of the print's room that runs on carbon-forge.
Proof: the process tier, once: `sh -c 'cargo build -p temper
--bin temper && NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p temper
--profile process -E "test(temper_open_signs_in_prints_the_inbox_and_exits_on_q)"'`:
`temper open`, its stdin and stdout piped, prints `== Inbox == Live`
through skein's terminal and exits 0 on `q` after a drained close (about
1 minute; 1.1 s of test at the base, `logs/w9/nextest-process-base-private.txt`).
Don't: copy any of skein's terminal into temper, its count among it
(its worst case and operations are `Terminal::worst_case` and
`Terminal::operations`); add a command
(`client.md`, 6.5); change one of temper's lines; bound temper's lines
by `Terminal::work_pending` or by a count of its own of what the terminal
holds; move jig's pin.
Size: at most 200 lines (mostly removals).
Removes: `terminal::Loop`'s line reader, print pacing and output
buffer, `terminal/run.rs`'s `Writing` and stdin buffer.

### 03.2 The link signs in over jig's new transport

Scope: temper's native client follows jig 10: the link signs in and
holds the credential, given the printed link's secret as `With::Link`
or the kept credential as `With::Held`; the host has the native shell
keep the credential on `Signing::SignedIn` and drop it on `SignedOut`,
before the terminal says so; the link's exchanges and the stream travel
over `jig-client-protocol-http` in the shell (`client.md`, 6.1, 6.6,
6.7, 7; `credentials.md`, 6).
Needs: jig 10.3 (the link's vocabulary, `jig-client-protocol-http`, the
native shell on it), and temper-03.s1's landing (ruling 153), whose repin
holds jig 10.3: 03.2 makes no repin commit, and its lines are its Shape's.
What temper-03.s1 carried of the client to compile (ruling 155) is not
done again.
Files: `crates/temper-client-shell-terminal/src/`: `client.rs` (391
lines at the base: the sign-in at :110-116, `sign_in` and `signing` at
:180-189), `route.rs` (:52-57, the link's requests to the shell;
`Abandon` goes), `terminal/native.rs` (75 lines: `signing` maps the
link's `Signing`), `limits.rs` (`link::Limits::new(client, exchanges)`,
the shell's `deadline`), `src/tests.rs:86-92`, `src/tests/fixtures.rs`;
`crates/temper/src/open.rs` (the printed link's errors from
`jig_client_protocol_http::Unlinked`); the users jig 10.1 lists for the
repository downstream of jig: `crates/temper/tests/support/person.rs:116-151`,
`tests/one-process/src/world.rs:266-322` (`Signing::SignedIn` at :303,
`Fact::StreamDropped` at :311-312), `tests/one-process/src/referee/client.rs`
(the link's `translate`). Found by `git grep -n -E
'Signing::|set_token|\.signing\(\)|jig_client_shell_native|jig_client_protocol_link'
-- '*.rs'` in temper (at the base, under the names `jig_shell_native`
and `jig_protocol_client_link`: 8 files and 14 lines, 9 files and 10
lines, `logs/w9/users-jig-client-crates.txt`).
Shape:
- `NativeClient::new(.., start: Start)`: `Start::Link(printed)` asks
  `Shell::sign_in(&printed)` for the secret and gives the link
  `Event::SignIn { with: With::Link(secret) }`; `Start::Token` gives the
  credential `Shell::loaded()` returns as `With::Held(credential)`, and
  none is the terminal's "not signed in" line (6.1's table, unchanged).
- On the link's `Request::Signing { signing }`, `route::signing` gives,
  in order: for `SignedIn { party }`, `Up::Keep(link.credential())`
  (the host's `Shell::keep`) then the terminal's line; for `SignedOut`,
  `Up::Keep(None)` then its line; `Again`, `Refused`,
  `Lost` as `terminal/native.rs::signing` maps the shell's today.
- `route.rs`: the link's `Exchange { exchange, credential, message }`,
  `OpenStream { credential }` and `CloseStream` to the shell as they
  are; the shell's `Reply`, `Lost`, `Streamed` and `StreamGone` to the
  link.
Test shape: the machine world and the crate's unit tests keep their
names and expected values (`exchanges_and_store_writes_go_to_jigs_shell`
loses its `abandon` row, an input the Shape removes). New unit tests in
`src/tests.rs`, on `src/tests/fixtures.rs`, each under 20 ms:
`a_signed_in_signing_keeps_the_credential_before_any_notice`,
`a_signed_out_signing_drops_the_kept_credential`,
`a_kept_credential_signs_in_held_with_no_exchange`; the reference test,
on a new function `route::signing(signing, credential) -> List<Up>`
(the host's order, step code):
```rust
#[test]
fn a_signed_in_signing_keeps_the_credential_before_any_notice() {
    let ups = route::signing(link::Signing::SignedIn { party: fixtures::PARTY }, Some(&fixtures::credential()));
    assert_eq!(ups.as_slice(), [Up::Keep(Some(fixtures::credential())), Up::Notice(Notice::SignedIn)]);
}
```
(`fixtures::PARTY` and `fixtures::credential()` are new beside
`fixtures::CREDENTIAL`; `Up::Keep` is the Shape's.)
Proof: the process tier, once, as 03.1's command: `temper open` signs in
by the printed link through the link's sign-in over jig's transport,
prints `== Inbox == Live`, keeps `client/token` at `0o600`
(`crates/temper/tests/process.rs:28`) and exits 0 (about 1 minute).
Don't: keep a credential in temper's root or view; send a sign-in
exchange for a held credential; change the terminal's lines.
Size: at most 150 lines.
Removes: `route.rs`'s `Abandon` arm; the shell's sign-in calls in
`client.rs` but `Shell::sign_in`.

### 03.3 The contract from `jig-client-domain`; the table's records with their askers

Scope: temper follows jig 11: the contract's types from
`jig_client_domain::contract`, the link's judgment of the hello
(`Event::Hello { served }`), the link's payload types (`Family`, `Ask`,
`Readable`, `Watchable`, `Readout`, `Answered`) from
`jig_client_domain`, and the request table's records as jig 11.3's
`Held { key, request }`, each key carrying its asker (jig's reference UI
65536, the forge's side 1), as jig 11.3 changes the test root's records;
the client wire at v8.
Needs: jig 11.5 (11.3's `Held` and v8 among it).
Files (the contract's path and `Event::Hello`'s users moved with
temper-03.s1's repin, ruling 155: 03.3 does what remains): the contract's path, `jig_domain_client::contract` to
`jig_client_domain::contract`: 21 files and 40 lines of temper at the
base (`git grep -c 'jig_domain_client::contract' -- '*.rs'`,
`logs/w9/counts.txt`; jig 11.1 counts 24 and 55 with the nested
blocks), by the `sed` jig 11.1 gives, nested `use` blocks split by hand; `Event::Hello`'s users:
temper's client domain's and client view's test fixtures, the
terminal's `route.rs:121`; the link's payload types: the terminal's
`route.rs` and `route/family.rs`; `Saved::Table`'s: the terminal's
`route.rs:61-72`; the root's records: `crates/temper-client-domain/src/{store.rs
(VERSION 1 to 2, `Record::Table { record: RequestRecord<Held> }`),
domain.rs (`RequestTable<Owner, Held>`), route.rs (:24-60, the
restore), owner.rs}`; the client wire's `v7` to `v8` (8 lines in
temper at the base, `git grep -n -E 'client_wire::v7|v7 as w|wire::v7'`): `crates/temper-service/src/{family.rs:105,
family/refusals.rs:3, clients.rs, tests/clients.rs, tests/clients/peer.rs}`,
`tests/codecs/tests/client_wire.rs`, `tests/one-process/src/referee/{client.rs,
client/cache.rs}`. Found by `git grep -n -E
'jig_domain_client::contract|Hello \{|Saved::Table|client_wire::v7|v7 as w'
-- '*.rs'` in temper.
Repin repairs of 03.3's pin, jig `755e02e53` (ruling 164), within the
README's 200 lines: jig 07.2's `ProcedureStep` in place of
`ProcedureAction` through temper's engine (`boundary.rs:8,364-369`,
`lib.rs:39`, `route.rs:34,100-101`, `route/forge_route.rs:1835-1906`,
`tests/engine/domain/tests/root_routes.rs:1654-1659`), and of 04.1, what
the compile needs: `Limits.templates` (`limits.rs:29-54`), `core_limits`
(`:109-143`) and the service's `templates: jig.core.templates`
(`crates/temper-service/src/limits.rs:264-278`); and, since the pin's
configuration reader refuses `chats` (ruling 169), 04.1's `ENGINE`
conversion as 04.1's Files give it (`crates/temper-service/src/tests/fixtures.rs:46`,
:65, and :436 where the pinned types ask it).
Shape: as jig 11.3 changes `testing/jig-client-test-domain/src/{store.rs,
domain.rs}`: the root's table keeps `Held { key: Key { asker, number },
request }`, the asker from the root's owner (`Owner::Jig` 65536,
`Owner::Forge` 1); temper's client store `VERSION` 2, a record of
version 1 dropped at start as one of another version is (`route.rs`'s
restore, unchanged in that).
Test shape: `crates/temper-client-domain/src/tests.rs` (213 lines) and
`tests/form.rs` keep their names and expected values, moved to `Held`
as jig 11.3 moves the test root's; one new unit test, under 20 ms:
`a_restored_record_of_version_one_is_dropped_as_invalid`.
Proof: the joint world, once: `cargo nextest run -p
temper-client-world -E 'test(a_reload_with_an_adoption_pending_sends_it_once)
| test(a_double_press_of_adopt_adopts_once)'`: an adoption kept as a
`Held` record across a reload is made once by temper's engine root, and
a double press joins one request (two stories, under 0.5 s each; both
pass at the base, `logs/w9/nextest-base-private.txt`).
Don't: re-export the contract from temper's crates; change the forge's
form; keep a `v7` path.
Size: at most 120 lines (the path rewrite counted apart, as jig 11.1
counts its own).
Removes: `jig_domain_client::contract`'s paths, `v7`'s.

### 03.4 The client's records, and `jig-client-view`'s paths

Scope: temper follows jig 12's surface changes to the link and jig 13.1
and 13.2: the link's small-store names (`Save { record }`, `Erase { key
}`, `Stored`, `NotStored`, `Record { key, bytes }`), its limits from
`jig_client_domain::Limits`; temper's crates name the client's types
from `jig-client` alone, which re-exports `jig-client-domain`'s (jig
12's choice): `jig_client_domain::` paths become `jig_client::`, and
the manifests take `jig-client` in place of `jig-client-domain`; the
small store holds the client's records
alone, the reference UI's records (its place, its drafts) dropped by
temper's root (Choices); `jig_domain_client::` paths to
`jig_client_view::` (`jig-domain-client` moved into `jig-client-view`).
Needs: jig 13.2 (`jig-domain-client` gone; jig 12 among it).
Files: `crates/temper-client-shell-terminal/src/{client.rs:328,
route.rs:52-53, 61-72 (13 lines, jig 13.1's count), limits.rs}`,
`src/tests.rs:53-82`; `crates/temper-client-domain/src/{store.rs
(`Record::Jig` goes), route.rs, boundary.rs}`; the path rewrite, 190
lines in 37 files of temper at the base (`git grep -c
'jig_domain_client::' -- '*.rs'`, `logs/w9/counts.txt`, less 03.3's
contract paths; jig 13.2 counts 175 in 42), by jig 13.2's `sed`
(`jig_domain_client::` to `jig_client_view::`), and the manifests
naming `jig-domain-client` (`crates/temper-client-domain`,
`crates/temper-client-view`, `crates/temper-client-shell-terminal`,
`crates/temper`, `tests/client`, `tests/one-process`). Found by `git grep
-n -w -e jig_domain_client -e 'Saved::' -e 'StoreRead' -- '*.rs'` and
`git grep -n jig-domain-client -- '*Cargo.toml'` in temper.
Repin (ruling 188): temper's jig pin is `755e02e5`; the repin onto jig
13.2 is the first past jig 03.1 to 09.4, and repairs them as the
README's "What a temper repin repairs" says, within its 200 lines and
jig 08.4's census (README). A re-check at jig `3f2ae98ad` found, in
`temper-engine-domain`: `Event::ForgetChat` gone (`archive.rs`),
`Now::LoadEndedChat` now `LoadEndedRead { request, task, party }`
(`route/now.rs`), `Held::Load(_)` (`route/held.rs`), `Event::Application`'s
`scope: ApplicationScope` (`route/applications.rs`), `restore_inbox_row`
on a core `Record` (`restart.rs`), `Stored::EndedRead`, `Ask::Compound`
and `Ask::Create` (`store.rs`); in `temper-protocol-store`,
`src/records/application/refusal.rs:290-370` over people v3's
`Refusal` (`NotWellFormed`, `proposable: Option<Accepter>`), and
`CoreRefused`'s bound (`schema/store-v2.schema:177`, 23), which its
drift test settles; `config::What::Operations` in
`crates/temper-service/src/startup/problems.rs` and
`crates/temper-forge-setup/src/problem.rs`; and
`crates/temper/tests/support/forgejo.rs`'s overrides (:135-168) gain
`("people.operations", 5)`. temper 04.1's Files name the last three;
each 03.4 does, 04.1 then leaves. The crates downstream of
`temper-engine-domain` were not compiled: fix any further error as the
README says, and list it in the commit body.
Shape: no new type. The root's `Request::Save` and `Erase` carry the
table's records alone (until 03.5 gives them to the client); the
reference UI's `Save` and `Erase` of its records are dropped by the root
with no fact.
Test shape: the tests keep their names and expected values but the
paths; one new unit test, under 20 ms, in
`crates/temper-client-domain/src/tests.rs`:
`the_reference_uis_records_never_reach_the_small_store`.
Proof: the joint world, once: `cargo nextest run -p
temper-client-world -E 'test(signing_in_shows_the_inbox_from_the_engine_root)
| test(a_reload_with_an_adoption_pending_sends_it_once)'` (under 0.5 s
each): the person signs in on jig's pages from their new crate, and a
kept adoption is made once after a reload.
Don't: keep `jig-domain-client` in a manifest; alias its paths.
Size: at most 80 lines (the path rewrite counted apart). By ruling
201, temper-03.s2 carries this Scope, Files, Shape and Removes; 03.4
keeps its Test shape and Proof on s2's landing: at most 30 lines.
Removes: `jig-domain-client` from temper's manifests;
`store::Record::Jig`.

### 03.5 The root holds `jig-client`; the forge's request as bytes; `Owner`

Scope: temper's root holds `jig_client::Client` in place of its request
table, as jig 21.2's commit changes `ops`'s root: its children ask
through it, jig's reference UI through `jig_client_view::bridge` (jig's
transitional bridge, removed by jig 21.3), the forge's side under its
asker with its adoption as the family's bytes
(`temper_client_protocol_wire::adopt` at `CLIENT_VERSION`) and its
outcome decoded from the family's answer; the root passes the client's
transport requests out in their order and routes each tell by its
asker, exhaustively, `Link` to each child; `Owner` in the wire crate,
matched with no `_` arm by the engine's routing of the forge's family; the forge's
form out of date from `client.link().not_served` (`client.md`, 3.1, 3.3,
4.4, 4.5, 4.6; jig's `root.md`, 3 to 5, 7, 15). temper's client limits
declare the forge, `FORGE` at `CLIENT_VERSION`, in
`jig_client_domain::Limits.owners` (jig 12.4; jig's `client.md`, 8.2),
so `not_served` names it from the hello on, before any adoption is
sent, and a kept adoption of another version is dropped by `jig-client`
at start. The terminal's host starts the root at load and signs in
after it, as 1e8ed63a does (`client.md`, 3.4: `Start { kept }` "before
any other event, then signs in"; "The shell holds back neither the
start nor the sign-in"): jig's reference UI asks its frame's watch at
its start, and `jig-client` opens it once signed in (jig's `client.md`,
8.1: "A watch waits for a sign-in"), from jig-12.s1.
Needs: jig 21.2 (`ops`'s root on `jig_client::Client`, jig 13.4's
native host and worlds among it); jig-12.s1 (a watch asked while signed
out opened once signed in), whose landing a second repin commit takes
by the rule above, with `jig-12.s1` for `jig-NN.k`.
Files:
- `crates/temper-client-protocol-wire/src/lib.rs` (`Owner`, `number`,
  `owner`, beside `FORGE` and `CLIENT_VERSION`; the sketch below,
  compiled at the base);
- `crates/temper-client-domain/src/{domain.rs (the table at :15, `take`
  at :58-92, `answered` and `ask` at :103-162 go), route.rs, boundary.rs,
  owner.rs, store.rs (goes), limits.rs, forge.rs (its `Ask` as bytes;
  `Served` from `link().not_served`), forge/form.rs}`, `Cargo.toml`
  (`jig-client`, `temper-client-protocol-wire`);
- `crates/temper-client-view/src/root.rs` and `limits.rs`, as jig
  21.2's commit changes `examples/ops/client-view`'s routing;
- `crates/temper-client-shell-terminal/src/{client.rs, route.rs,
  route/family.rs (209 lines: goes, the forge's side encodes its own
  request), limits.rs, terminal/native.rs}`;
- `crates/temper-service/src/family.rs:22-28` (the family matched with
  `owner(family)`: `Some(Owner::Forge)` decodes the adoption,
  `Some(Owner::Pages)` and `None` answer undecodable, as the base
  answers another family), `crates/temper-service/src/clients.rs:84-90`
  (the served list names `number(Owner::Forge)`);
- the forge number's users, `git grep -n -E
  'wire::FORGE|client_wire::FORGE|fixtures::FORGE\b|forge::Config' --
  '*.rs'` in temper: the client domain's and the view's tests,
  `tests/client/src/fixtures.rs:15` (`FORGE` is `0` at the base: the
  wire crate's from here);
- `tests/client/src/{app.rs, front.rs, translate.rs, translate/forge.rs}`
  (on jig 13.4's `jig-client-world`), `tests/one-process/src/{world.rs,
  referee/client.rs}` and `crates/temper/tests/support/person.rs` (the
  native host as jig 13.4 changes jig's: the root holding the client).
Shape:
- `temper_client_protocol_wire::{Owner { Forge, Pages }, number(Owner)
  -> u16, owner(u16) -> Option<Owner>}` (sketch).
- `temper_client_domain::owner::Owner::{Jig, Forge}` with `asker(self)
  -> Asker` (65536, `number(wire::Owner::Forge)`) and `of(Asker) ->
  Option<Owner>`, as jig 13.4's test root has `Owner::{Jig, First,
  Second}`; the root's `Event` gains `Client { event: jig_client::Event
  }` and loses `Answer`; its `Request` gains `Client { request:
  jig_client::Request }` and loses `Send`, `Save`, `Erase` and
  `Jig(JigOut::{Read, Open, Continue, Close})`, as 13.4 and 21.2 change
  theirs.
- The forge's side: `Ask::Family(Family { family:
  number(wire::Owner::Forge), version: CLIENT_VERSION, bytes:
  adopt(&record) })` under `Key { asker: Owner::Forge.asker(), number
  }`, its numbers drawn from the forge's seed; its outcome
  `w::AdoptAnswer::decode` of the family's answered bytes;
  `Served::OutOfDate` while `client.link().not_served` holds the
  forge's number.
- The terminal's host: once after start, while a kept request's
  `client.pending(key).party` differs from the party `client.link()`
  names, it says `temper: work of another party is kept in DIRECTORY,
  and waits for them` (`client.md`, 6.6), and not again.
Test shape: `crates/temper-client-protocol-wire/src/tests.rs`,
`crates/temper-client-domain/src/tests.rs` and `tests/form.rs`, the
service's `src/tests/family.rs`, and the joint world. The reference
unit test:
```rust
#[test]
fn each_owner_has_its_number_and_back() {
    for (owner, number) in [(Owner::Forge, 1), (Owner::Pages, 2)] {
        assert_eq!(crate::number(owner), number);
        assert_eq!(crate::owner(number), Some(owner));
    }
    assert_eq!(crate::owner(65_535), None, "a number no owner of this build has");
}
```
Tests, each under 20 ms (unit) or 0.5 s (world):
`each_owner_has_its_number_and_back`,
`an_adoption_is_the_forges_family_bytes_under_its_asker`,
`a_kept_adoption_of_another_version_is_dropped_and_its_form_shows_none_pending`
(jig-client drops a kept request of another family version at start
and tells its owner, `client.md`, 4.6; it replaces the base's restore
test of the root's records, which 03.5 removes),
`another_partys_kept_work_prints_its_line_once` (the terminal's machine
world: `temper: work of another party is kept in DIRECTORY, and waits
for them`, printed once, `client.md`, 3.4 and 6.6, when a kept request's
`client.pending(key).party` is not the party `client.link()` names),
`an_answer_under_the_forges_asker_reaches_the_form`,
`an_asker_naming_no_child_is_a_fact_and_reaches_none`,
`a_family_the_engine_does_not_serve_marks_the_form_out_of_date`,
`a_request_under_the_pages_owner_is_answered_undecodable` (service).
Proof: the joint world, once: `cargo nextest run -p
temper-client-world -E 'test(a_repository_adopted_from_the_navigation_link_is_held_by_the_engine)'`
(under 0.5 s; 0.03 s at the base): the adoption crosses `jig-client`'s
table as the forge's bytes, temper's engine routing decodes it by
`Owner`, and the engine holds the repository. And the one-process world,
once: `cargo nextest run -p temper-one-process-world -E
'test(an_adopted_repository_gets_its_change_pushed_through_git)'` (about
3 s; 2.2 s at the base): the same through HTTP bytes and the native
host holding the client. After the repin onto jig-12.s1, the land
gate's run of `temper-one-process-world` is the check that the five
stories failing at `tests/one-process/src/referee/client.rs:97` under
1e8ed63a pass: no other run is wanted.
Don't: decode a family's bytes in `jig-client`; keep a map from keys to
children or a `_` arm on an asker (jig's `root.md`, 5 and 10); give
jig's reference UI an asker of temper's; hold back the root's `Start`,
or give `SignIn` before it, in the terminal's host; change the
referee's rule that every client response is 200
(`tests/one-process/src/referee/client.rs:97`).
Size: at most 300 lines; ruling 238 re-estimates it with the repin
onto jig 21.2 and rulings 236 and 237: aim 750, stop at 850 added lines
by code-organization.md's command, the repin's repairs included; the
repairs outside the Files aim 300, stop at 350 (README, Pins); ruling
241 moves them to aim 400, stop at 450; ruling 244 to aim 500, stop
at 550, for the gate's repairs. The repin onto jig-12.s1 changes
`Cargo.lock` alone; any repair it brings counts in ruling 244's 550.
Removes: the root's `RequestTable`, `store.rs`, `Owner::Kept`, the
terminal's `route/family.rs`, the forge's `Config { connector, version
}` (the wire crate's `Owner` and `CLIENT_VERSION` in its place).

### 03.6 jig's pages as `jig_client_view::Ui`; `tests/client` on jig's client world

Scope: temper follows jig 21.3 and 21.4: jig's reference UI is
`jig_client_view::Ui`, stepped by temper's root with the person's acts,
the places gone to and what the client tells under jig's asker, lent
`&jig_client::Client` (the bridge goes); its requests on
`client.pending(key)`; the forge's side watches `me` under a handle of
its own for its party and the link, in place of the by-value hand-offs
(`Confirmed`, `Link`, `FrameIs`); temper's view arranges
`jig-client-view`'s pages (transitional, decisions.md, 7, ended by
temper 07.4); `tests/client` on jig 21's `jig-client-world`, with
skein's fake person and the world's referee, jig's check
(`client.md`, 3.1, 10; `domain/testing.md`, 6.3).
Needs: jig 21.4 (21.3's `Ui` among it).
Files: `crates/temper-client-domain/src/{domain.rs, route.rs,
boundary.rs, forge.rs (`Event::Confirmed` gives way to its `me`
handle), parts.rs, limits.rs}`; `crates/temper-client-view/src/{root.rs
(:69-75, `jig_client_view::jig` and `after_page` with the `Ui`),
limits.rs, forge/page.rs}`; `crates/temper-client-shell-terminal/src/{client.rs,
limits.rs, route.rs}`; `tests/client/src/{app.rs, front.rs, translate.rs,
fixtures.rs}` (as jig 21.3 changes "the worlds' fronts"),
`tests/one-process/src/world.rs`, `crates/temper/tests/support/person.rs`.
Found by `git grep -n -e 'bridge::' -e 'Event::Confirmed' -e FrameIs
-e 'Request::Link' -- '*.rs'` in temper. Ruling 257 (jig 21.4's
re-check at jig `48a745eb9`) adds the users of `JigOut::{Answered,
Unknown}` and `KeptGone` outside these:
`crates/temper-client-shell-terminal/src/client/tests.rs:46` and
`crates/temper-client-view/src/tests/forge_page.rs:201`; the others
(`route.rs:108-117`, `forge.rs:95`, the shell's `client.rs:325` and
`route.rs:37`, `tests/client/src/app.rs:125`, `translate.rs:271`) are in
them.
Shape:
- The root: `ui: jig_client_view::Ui`, made with temper's navigation
  (`Inbox`, `Conversations`, `Deployment`, as
  `temper_client_domain::navigation` gives them today) and home
  `Inbox`; `Ui::step(now, wall, &client, Event::Client { told }, out)`
  for each tell under `Owner::Jig.asker()`; its `Request::Client {
  asked }` given to the client in the same step; its `Save` and `Erase`
  dropped (Choices); its `ShowPart` and `PartsLeft` to the forge's side
  as `parts.rs` routes them today.
- The forge's side: a `me` handle of its own (`Watch { handle: Handle {
  asker: Owner::Forge.asker(), number: 0 }, subject: me }`), its party
  from that document, its link from `Told::Link`.
- The `Ui` asks no sign-in, reports no fact and continues no watch (jig
  21.3; jig's answer of 2026-10-17): the root's arms for the reference
  UI's `Continue`, `SignIn` and `Fact` (`crates/temper-client-domain/src/route.rs:115,
  :125-126, :150` at temper `ba298dd5`) go, and the terminal shell's
  that carry them (`crates/temper-client-shell-terminal/src/route.rs:196,
  :201, :203`); temper signs in by the printed link (`client.md`,
  section 7). `Told::Dropped`, which names no asker, goes to the `Ui`
  too (jig's `root.md`, 8).
Test shape: `tests/client/tests/{stories.rs, adoption.rs}` and
`fuzzy_client.rs` keep their names and expected values; the client
domain's and the view's tests keep theirs on the `Ui`'s surface; one
new unit test, under 20 ms:
`the_forges_side_learns_its_party_from_its_own_me_handle`.
Proof: the joint world, once: `cargo nextest run -p
temper-client-world -E 'test(signing_in_shows_the_inbox_from_the_engine_root)
| test(a_held_chats_escalation_released_from_its_task_page)'` (under
0.5 s each; both pass at the base): the person signs in and releases a
held chat on `jig_client_view::Ui`'s pages, against temper's engine
root, the referee's rule "what they saw is what they decided" holding.
Don't: copy a page of jig's into temper; keep the reference UI's
records; keep a by-value hand-off between the root's children.
Size: at most 250 lines.
Removes: the bridge's uses, the forge's `Event::Confirmed` and the
root's `FrameIs` arm.

## Interface sketches

`temper_client_protocol_wire::Owner` and its two functions compile at
the base (appended to `crates/temper-protocol-client-wire/src/lib.rs`),
and the root's `owner::Owner` with `asker` and `of` (as
`temper_client_domain::sketch_owner`, `jig_client::Asker` stubbed as a
newtype of `u32`), both under `logs/w9/clippy-sketches-3.txt`. The
root's routing names jig 12's and 21's types and is checked at the
ready check.

```rust
// crates/temper-client-protocol-wire/src/lib.rs (03.5)
/// temper's owners on jig's client wire, which the engine's root and the
/// client root share (client.md, 4.6; jig's `client/domain/root.md`, 15).
pub enum Owner {
    /// The forge connector's client side: its family's records travel under its number.
    Forge,
    /// temper's pages: jig's operations, reads and subjects only, no record of their own.
    Pages,
}
/// The owner's number, never given to another owner.
pub const fn number(owner: Owner) -> u16 {
    match owner {
        Owner::Forge => FORGE,
        Owner::Pages => 2,
    }
}
/// The owner a number names; none for a number no owner of this build has.
pub const fn owner(number: u16) -> Option<Owner>;
```

```rust
// crates/temper-client-domain/src/owner.rs (03.5): the root's children, after jig 13.4's test root
/// A child of temper's root, the asker of the handles and keys it gives the client.
pub enum Owner {
    /// jig's reference UI, under jig's asker, 65536: transitional, removed by temper 07.4.
    Jig,
    /// The forge's client side, under `wire::number(wire::Owner::Forge)`.
    Forge,
}
impl Owner {
    pub fn asker(self) -> jig_client::Asker;
    /// None for an asker no child of this build has: `Pages` until temper 06.1, or another's.
    pub fn of(asker: jig_client::Asker) -> Option<Owner>;
}
```

```rust
// crates/temper-client-domain/src/route.rs (03.5): what the client tells, by asker
match Owner::of(told_asker(&told)) {
    Some(Owner::Jig) => jig_step(root, told),        // the bridge in 03.5, the Ui in 03.6
    Some(Owner::Forge) => forge_step(root, told),    // decodes the family's bytes
    None => out.push(Request::Fact { fact: Fact::UnknownAsker }),
}
// Told::Link reaches each child before this match.
```

## Questions to raise when reached

- If jig 21.3's `jig_client_view::Ui` waits for its `Save` to be stored
  before it shows a page (so that dropping its records stalls it),
  temper's root keeps them under the reference UI's keys: raise it,
  since `client.md`, 3.4 says temper's pages keep no record.
- If a pinned jig step's landing has no `Step:` trailer (a landing
  outside the run), the repin takes the first landing after it that
  has: raise it only if that brings the next increment's Shape.

## Done when

- 03.1 to 03.6 have landed on temper's `origin/main`.
- `git -C <worktree> grep -n -e jig-domain-client -e 'jig-view' --
  '*Cargo.toml'` prints nothing.
- `git -C <worktree> grep -n -w RequestTable -- crates/temper-client-domain/src`
  prints nothing.
- `cargo nextest run -p temper-client-world`, once, prints a
  `Summary` line with no test failed, the base's 11 stories among those
  run under their names.
