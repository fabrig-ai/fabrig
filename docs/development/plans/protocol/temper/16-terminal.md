# Session 16: the terminal shell, `temper open`, `temper sign-in`

A person uses temper from a terminal: `temper open DIRECTORY
LINK_OR_ADDRESS` runs 12.1's native client with a terminal in front of
it, which prints jig-view's tree as text and takes one command per line
on stdin. What the terminal decides is in two pure functions, step code
in `temper-shell-terminal`: the renderer (`render`: the tree to lines,
with the print's numbers), fixed by golden text, and the command reader
(`command`: stdin's bytes to lines, a line to a command or a refusal).
The loop (`terminal`, ordinary Rust) joins stdin, stdout, the
termination signals and the native client on one host of skein's loop;
its machine world drives it against a scripted client in the crate's own
tests. `temper open` comes to the binary with its arguments and exit
lines and a process test, and `temper sign-in` signs an LLM account in
at a terminal, copied from smith's local shell. The last increment adopts
a repository by commands on the throwaway Forgejo. Six increments: two of
step code proven by unit tests, the loop by its machine world, three at
the process tier.

## Read first

- `docs/design/client.md`, section 6 (all), 7 and 10;
  `docs/design/credentials.md`, sections 3 and 7;
  `docs/design/protocol.md`, section 4; `docs/design/configuration.md`,
  sections 2 and 9; `docs/design/domain/testing.md`, 2.6 and 6.1.
- `findings/jig-client.md`, sections 4 and 6, Discrepancies 2 to 5, 12,
  15, 17; `findings/skein-smith.md`, sections C, D and E.
- `decisions.md`, 6, 13 and 17; `code-organization.md`, sections 2, 5
  (the golden files, the terminal world's share) and 6 (the step
  modules' check).
- This plan's `temper/12-native-client.md` (12.1's `NativeClient` and
  `limits`) and `temper/11-client.md` (the view root's tree).
- The code, at the pins: jig's `crates/jig-view/src/{tree.rs:1-345,
  binding.rs:1-60, frame.rs:13-100, words.rs:38-53,
  phrases/english.rs:738-772}` (roles, states, acts, the frame's
  statuses); `crates/jig-shell-native/src/{shell.rs, files.rs:1-20,
  125-145, lib.rs}`; jig's `testing/jig-test/{src/lib.rs, src/main.rs,
  tests/process.rs, tests/support/mod.rs}` (the binary, the process tier,
  the `Person` host that reads a child's stdout); skein's
  `crates/skein-shell/src/{drive.rs, startup.rs:15-45, lib.rs:33-45}`,
  `crates/skein-io/src/layer.rs:134-215` (`adopt_read_pipe`,
  `adopt_write_pipe`, `adopt_signals`), `testing/skein-world/src/end_to_end.rs:25-140`;
  smith's `crates/smith-local-shell/src/{credentials/login.rs,
  credentials/sign_in.rs:195-235, local_shell.rs:239-246}`,
  `crates/smith-local-service/src/credentials.rs:22-40, 206, 485-620`,
  `tests/local-process/src/oauth.rs` (the `Browser` host);
  jig's `testing/jig-test-service/src/accounts.rs:120-180` and
  `testing/jig-test/src/lib.rs:117-182` (an account's registration,
  endpoint and record keeper from jig's configuration).

## What the design asks

- **Pure where it decides** (client.md, 6.8): "The renderer, tree to
  lines: from jig-view's `Tree` alone, the page's lines (6.4) and the
  last print's numbers (6.5). Its output is bounded: the most lines and
  bytes the view's largest tree can print, counted from the view's
  limits." "The command reader, line to act: from one line, the last
  print's numbers and the view's last tree, one command: acts on a node
  (press, toggle, or input then submit, as jig-view's `Act`), print,
  close, nothing, or a refusal with its line (6.5)." "Unit tests fix
  both: golden text of pages, kept as files beside the tests and changed
  only by hand".
- **How it prints** (6.4): the first line `== TITLE == LINK`; "Then every
  node in preorder, by its role" (the table of 6.4, with `cell` and a
  status with no text, which "prints its name"); states in a fixed
  order; "Nothing from the engine drives the terminal": controls and
  bytes that are not UTF-8 as U+FFFD, a tab as a space, a newline a new
  line at the same indentation.
- **Numbers and commands** (6.5): "each node a person can act on ... that
  is not disabled gets `[n]`, from 1 in preorder, at each print. A node
  of those roles that is disabled or bound to nothing prints `[-]`";
  `n`, `n WORDS`, `p`, `q`, an empty line; the six error lines.
- **The loop** (6.2): "One host on skein's loop": the native client and
  "stdin adopted as a read pipe ..., stdout as a write pipe ..., and
  SIGINT and SIGTERM from skein-shell's `open_termination_signals`";
  "stdin is left as it is, never raw"; "One line, one command."
- **When it prints** (6.3): "The whole page, always"; "At most once per
  second (a guide; the shell's print interval, in its limits)"; "At once
  after the person's own act ... and at once on `p`"; "One print at a
  time"; "The shell's own lines start with `temper: `".
- **Signing out and closing** (6.6), **the engine unreachable** (6.7):
  the lines `temper: signed out; to sign in again, run temper open
  DIRECTORY LINK with a link temper serve printed`, `temper: signing in:
  no answer from the engine; trying again`, `temper: credential for
  service PARTY, shown once: TOKEN`, `temper: cannot write the small
  store in DIRECTORY`; "Closing: `q`, the end of stdin, SIGINT or
  SIGTERM. The client does not sign out."
- **`temper open`'s arguments and exits** (6.1): DIRECTORY's parent
  opened with `open_root`, its last component to `jig-shell-native`;
  LINK_OR_ADDRESS a link or `http://HOST:PORT`, HOST an IP address on
  loopback; with an address, "the terminal reads the token file once
  with skein-shell's `read_private_file`, only to learn that there is
  one"; exit 0 "once the client closed and drained", 1 with one of six
  stderr lines.
- **`temper sign-in CONFIG.json ACCOUNT`** (credentials.md, 3): "reads
  and checks `CONFIG.json` as `temper serve` does ..., reading no secret
  file but the account's own; it opens no store and starts no service";
  refuses "a number no account has; a handed-in account; and an account
  whose registration has a client secret"; "prints the visit's URL on
  stdout, one line"; "exits 1 on any failure ..., the reason on stderr";
  "copied from smith's local shell's sign-in". Its tests (credentials.md,
  7): "against skein's fake issuer ...: the visit printed, the redirect
  taken, the record kept, exit 0; a handed-in account and a
  confidential client refused; a running engine lending the new record
  at its next grant."
- **The process tier's terminal test** (domain/testing.md, 2.6): "`temper open
  DIRECTORY LINK` with stdin and stdout piped signs in by the link
  `temper serve` printed, prints the inbox, adopts a repository as above
  by commands ..., and exits 0 on `q` after a drained close"; "In the
  order of building ...: the story's chat first, then its change landed,
  then the terminal's test."

## Choices this plan makes

- **The renderer measures, then writes.** `render::print` walks the tree
  twice with one walker: the first pass counts the bytes, the second
  writes them into a `skein_lib::Writer` of that length, so a print
  allocates once; `Bounds` is the worst case's and an assertion's, not
  the allocation's.
- **"Behind" by its words.** The renderer cannot tell the frame's second
  status (jig's identity status, "Signed in as …") from the behind
  status by position (`frame.rs:15-30`), so it takes jig's sentence for
  behind (`render::Frame { behind }`, said once at start from jig's
  phrasebook) and adds the second status to the first line only when its
  text is that sentence. The frame's link status and, when behind, the
  behind status are not printed again; its other nodes are.
- **A status with no text prints its name** (client.md, 6.4); with
  neither, `*` alone.
- **Text that is not UTF-8:** each byte outside a well-formed UTF-8
  sequence prints as one U+FFFD, and so does each character of
  U+0000-U+001F and U+007F-U+009F but a tab (a space) and a newline.
- **A number is 1 to 10 ASCII digits whose value fits `u32`;** any other
  first word is not a command (`Refusal::Unknown`). `0` is a number no
  print shows (`NoNumber`).
- **`command::Lines` splits stdin, one byte at a time,** in the step
  module, so lines cut at any byte are a unit test; the loop feeds it
  each byte it reads. A line is held up to the limit and one `\r`.
- **The loop is a `Loop` over a `Client` trait** (ordinary Rust): 12.1's
  `NativeClient` implements it in `src/terminal/native.rs`, the machine
  world's scripted client in `tests/support/mod.rs`. The machine world
  is the loop against scripted stdin chunks, a scripted stdout that takes
  writes when told, and a scripted client, on a test clock: no ring, no
  thread.
- **The terminal's own io is tagged:** its `skein_io::Io`'s submissions
  carry bit 62 of the operation's token, as jig-shell-native's files
  carry bit 63 (`files.rs:9, 132-140`), and the host routes each
  completion by it.
- **The terminal's limits** are constants in `limits.rs` (12.1's
  module): a line 4,096 bytes, a read 4,096, the print interval 1 s, 32
  shell lines waiting, the sign-in backoff 250 ms doubling to 8 s.
- **`temper open` and `temper sign-in` live in the binary**
  (`crates/temper/src/{open.rs, sign_in.rs}`), ordinary Rust after
  jig-test's `src/lib.rs`; the binary's usage line names the four
  commands (10.3 leaves it naming two).
- **`temper sign-in` drives smith's accounts driver**
  (`smith_local_service::Accounts`, as smith's `login.rs:96-135` does),
  with one `Source::SignIn` built from jig's configuration of the
  account as jig-test-service builds its driver's (`accounts.rs:152-176`):
  the record kept at `llm.records/N` under the working directory's root
  (`open_root(".")`, as jig-test's `lib.rs:179-181`), where `temper
  serve` started in the same directory finds it. It prints the visit
  on stdout (smith prints it on stderr, `login.rs:156`) and exits 0 or 1
  (smith exits 0, 2 or 3; findings/skein-smith.md, E).
- **The terminal's process test comes in two increments:** 16.4 signs in,
  prints the inbox and exits 0 on `q` as soon as 10.4's process support
  exists; 16.6 adds the adoption by commands once 15.3 has the forge
  wired and the throwaway Forgejo in the process support (domain/testing.md,
  2.6's order of building).

## Rules that bind it

- "No increment of this plan ... in `temper-shell-terminal`'s step
  modules (`route`, `render`, `command`, `limits`), uses `Vec`,
  `String`, `format!`, `std::`, a closure or a `loop`/`while`: the check
  is `grep -nE 'Vec<|String|format!|std::|\|[a-z_, ]*\||\bloop\b|\bwhile\b'
  crates/temper-shell-terminal/src/{route,render,command,limits}.rs
  crates/temper-shell-terminal/src/{route,render,command}/*.rs
  2>/dev/null`, which prints nothing." (code-organization.md, 6) Run
  it as `bash -O nullglob -c "…"`, so the brace globs match nothing
  rather than pass vacuously under `sh`.
- "Golden text of the terminal's renderer is
  `crates/temper-shell-terminal/src/tests/golden/<page>.txt`, changed
  only by hand (client.md, 6.8)." and "Scratch directories of tests are
  made under `env!("CARGO_TARGET_TMPDIR")`, never under `/tmp`"
  (code-organization.md, 5).
- "**The process and live tiers run only where a Proof line or 'Done
  when' says,** once; never in the gate."
  (README) and the terminal's machine world's share, "0.3 s and 2 s"
  (code-organization.md, 5).

## Increments

### 16.1 `render`: the tree to lines, and the print's numbers

Scope: `crates/temper-shell-terminal/src/render.rs` (step code):
`print(tree, frame, bounds) -> Print` as client.md, 6.4 and 6.5 say
(the first line, each of jig-view's 35 roles, states, numbers, text made
printable), `bounds(&jig_view::Limits)`, `worst_case`; golden pages of
jig's pages built by temper's view root from fixture states (6.8).
Needs: 11.3's `tests/client` (`jig-client-world` and its closure in
`Cargo.lock`, so this increment moves no pin), beyond the session's
start (12.1's crate `temper-shell-terminal`, its `limits` module and its
test fixtures).
Files: `crates/temper-shell-terminal/{src/lib.rs (the module),
src/render.rs, src/tests.rs (`mod render;`), src/tests/render.rs,
src/tests/fixtures.rs, src/tests/golden/{inbox, task, project,
deployment, signed_out, every_role, control_text}.txt, Cargo.toml (dev:
`jig-client-world`)}`. Shared hotspot: `src/tests/fixtures.rs` (12.1's).
Shape: as the interface sketch. Lines in preorder by role (client.md,
6.4's table): page, slot and unnamed region, figure, dialog, list and
tree print no line; a named one prints `NAME:` and indents its children
two spaces; an item's or tree item's first line starts `- ` in its
indentation's last two columns, its other lines two spaces deeper; a row
prints its cells' texts joined by ` | ` and its subtree no more; a cell
outside a row its own line; code each line four spaces deeper; quote
`> ` per line; a link with an external target `TEXT <URL>`, unnumbered.
A node's text (6.4's "its text", TEXT) is the runs of the text nodes
under it that no nearer node of another role holds, in preorder; NAME is
its name, or its text when it has none (jig's `client/domain/view.md`,
3: text "on text nodes only, as runs", "a name, where its text does not
give it"). The text nodes a line took, as its text or under a named
control, print no line of their own; a text node nobody took (under a
page, region, item, slot and the like) prints its runs as its own line;
a node of another role under a text-bearing one (a link in a paragraph)
prints its own line at that node's indentation (decisions.md, ruling
129).
Numbers: button, link, text field, text area, choice, check, tab with
`tree.bound(id)` and no `Disabled` state: `[n] `, `n` from 1 in
preorder; the same roles otherwise `[-] `. The walk, with no recursion
and no `while` (a bounded `for` over the stack's length):

```text
open = Stack<(end, indent, bullet)> with capacity limits.depth
for i in 0..tree.len():
    for _ in 0..open.len(): if open.top().end > i { break } else { open.pop() }
    node = tree.get(i); indent = open.top().indent or 0
    if node is the frame's link status, or its behind status: skip   // in the first line
    line = number(node) + mark(node) + words(node) + states(node)    // empty for silent roles
    if !line.is_empty():
        for each open entry with a pending bullet: put "- " at its indent, clear it
        put the rest of the indentation, the line split at '\n' (each part re-indented), "\n"
    if node.role == Row: put the cells joined by " | "; skip to i + node.size
    open.push((i + node.size, indent + (2 if node indents its children else 0),
               node.role is Item or TreeItem))
```

Test shape: `src/tests/render.rs`; fixtures in `src/tests/fixtures.rs`:
`page(address, documents) -> Tree` (temper's client root from
`temper_client_domain::new(&limits, SEED, FORGE)` and view root from
`temper_client_view::View::new`, started at `address`, each `JigOut::Read`
and `JigOut::Open` it asks answered with the documents
`jig_client_world::stories` gives: `frame(1)`, `task()`, `project()`,
`deployment()`, `proposal()`, `party(n)`, `stories.rs:17-402`, and an
inbox page the fixture writes with one row of each `InboxGroup`; built
once per test), `every_role() -> Tree` (built with `jig_view::Builder`:
one node of each role, each state, a disabled and an unbound control, a
check and a choice pressed and not, a status with no text), `frame()`,
`bounds()`, `golden(name) -> PathBuf` (`env!("CARGO_MANIFEST_DIR")` /
`src/tests/golden/<name>.txt`). Tests:
`each_fixture_page_prints_as_its_golden_file` (rows: inbox, task,
project, deployment, signed_out, every_role, control_text),
`numbers_go_to_enabled_bound_controls_in_preorder`,
`a_disabled_or_unbound_control_prints_a_dash`,
`the_first_line_is_the_title_and_the_link_state`,
`a_behind_frame_adds_its_sentence_to_the_first_line`,
`the_largest_fixture_tree_prints_within_its_bounds`. Each under 20 ms.
Reference test, in full:

```rust
#[test]
fn each_fixture_page_prints_as_its_golden_file() {
    let rows: [(&str, jig_view::Tree); 7] = [
        ("inbox", fixtures::inbox_page()),
        ("task", fixtures::task_page()),
        ("project", fixtures::project_page()),
        ("deployment", fixtures::deployment_page()),
        ("signed_out", fixtures::signed_out_page()),
        ("every_role", fixtures::every_role()),
        ("control_text", fixtures::control_text()),
    ];
    for (name, tree) in rows {
        let print = render::print(&tree, &fixtures::frame(), &fixtures::bounds());
        let golden = std::fs::read(fixtures::golden(name)).expect("golden file");
        assert_eq!(String::from_utf8_lossy(&print.text), String::from_utf8_lossy(&golden), "page {name}");
    }
}
```

Proof: unit, `cargo nextest run -p temper-shell-terminal -E
'test(/render::/)'`, run once: the seven pages print as their golden
files, numbers skip disabled and unbound controls, and the first line
carries the title and the link's state (the six tests, under 0.2 s).
Don't: write a golden file from the code under test (each is written by
hand, then checked); print a diff; wrap a line; read a node's style.
Size: at most 450 lines of production code.
Removes: nothing.

### 16.2 `command`: a line to a command or a refusal

Scope: `crates/temper-shell-terminal/src/command.rs` (step code):
`Lines` (stdin's bytes to lines, client.md, 6.2's "One line, one
command"), `read(line, numbers, tree) -> Command` and `refusal_line`
(6.5's table of commands and of errors).
Needs: nothing beyond 16.1.
Files: `crates/temper-shell-terminal/{src/lib.rs, src/command.rs,
src/tests.rs (`mod command;`), src/tests/command.rs,
src/tests/fixtures.rs}`.
Shape: as the interface sketch. `read`: spaces trimmed at both ends;
empty → `Nothing`; `p` → `Print`; `q` → `Close`; a number `n` then
nothing or a space and words (the words start after the spaces that
follow `n`); before the first print a number is `NoPage`; `n` not in
`numbers` → `NoNumber`; its node not in `tree`, `Disabled`, or with no
`tree.bound(id)` → `Gone`; words on a node not a text field or text area
→ `NoWords`; a text field or text area → `Write { value: words or empty,
submit: has SubmitsOnEnter }`; a check or a choice → `Act { Toggle }`;
any other bound node → `Act { Press }`; anything else → `Unknown`.
`Lines::take(byte)`: `\n` ends the line (`TooLong` if it passed the
limit, else `Whole` without a final `\r`); past `limit` bytes and one
`\r` the rest of the line is dropped until its `\n`.
Test shape: `src/tests/command.rs`; fixtures: `controls() -> (Tree,
Numbers)`, a tree built with `jig_view::Builder` and printed once by
16.1's renderer, its controls numbered in this order: 1 a button, 2 a
link, 3 a tab, 4 a check, 5 a choice, 6 a text field that submits on
enter, 7 a text area, then a disabled button (`[-]`). Tests:
`each_command_line_reads_as_its_command` (rows: `p`, `q`, ``, `   `, a
button's `n`, a check's `n`, a choice's `n`, a link's `n`, a field's `n
acme`, a field's `n` alone, an enter-sending field's `n  two words `, a
tab's `n`), `each_refused_line_reads_as_its_refusal` (rows: a number
past the print, a gone node, words for a button, a number before the
first print, `x`, `3x`, eleven digits), `each_refusal_has_its_line`,
`a_number_kept_across_a_rebuild_names_the_same_node`,
`a_line_cut_at_any_byte_makes_the_same_lines` (each split of
`3 acme\r\np\n`), `a_line_past_the_limit_is_refused_through_its_newline`.
Each under 20 ms. Reference test, in full:

```rust
#[test]
fn each_command_line_reads_as_its_command() {
    let (tree, numbers) = fixtures::controls();
    let id = |n: u32| numbers.node(n).expect("shown number");
    let rows: [(&[u8], Command); 6] = [
        (b"p", Command::Print),
        (b"  q ", Command::Close),
        (b"", Command::Nothing),
        (b"1", Command::Act { node: id(1), act: Act::Press }),
        (b"4", Command::Act { node: id(4), act: Act::Toggle }),
        (b"6  acme shop ", Command::Write { node: id(6), value: Box::from(*b"acme shop"), submit: true }),
    ];
    for (line, command) in rows {
        assert_eq!(command::read(line, Some(&numbers), &tree), command, "line {line:?}");
    }
}
```

Proof: unit, `cargo nextest run -p temper-shell-terminal -E
'test(/command::/)'`, run once: each command and each refusal of 6.5
reads as its row, and lines cut at any byte give the same commands (the
six tests, under 0.1 s).
Don't: trim or change the words beyond the spaces after `n` and at the
line's ends; decide a field's bounds (the domain's); act on a node
itself (the loop's).
Size: at most 250 lines of production code.
Removes: nothing.

### 16.3 `terminal`: the loop, and its machine world

Scope: `crates/temper-shell-terminal/src/terminal.rs` and
`src/terminal/{run.rs, native.rs}` (ordinary Rust): `Loop`, the
terminal's decisions over a `Client` (client.md, 6.2, 6.3, 6.6, 6.7):
lines into commands, acts to the client, prints when due, the shell's
own lines, closing; `Terminal`, the host on skein's loop joining the
`Loop`, 12.1's `NativeClient`, stdin, stdout and the termination
signals; the machine world in `crates/temper-shell-terminal/tests/`.
Needs: nothing beyond 16.2.
Files: `crates/temper-shell-terminal/{src/lib.rs, src/terminal.rs,
src/terminal/run.rs, src/terminal/native.rs, src/limits.rs (the
`Terminal` limits), src/client.rs (12.1's: the notices, the build count
and `act`'s result the `Client` impl reads, if 12.1 has not added them;
at most 40 lines), tests/support/mod.rs, tests/terminal.rs,
tests/fuzzy_terminal.rs}`. Shared hotspots: `src/client.rs`,
`src/limits.rs` (12.1's).
Shape (ordinary Rust, not compiled):

```rust
// temper_shell_terminal::terminal
/// The native client as the loop sees it: `NativeClient` (native.rs) and the world's script.
pub trait Client {
    /// The view's last tree.
    fn tree(&self) -> &jig_view::Tree;
    /// Builds of the view that emitted patches since the client started; a move is a change.
    fn built(&self) -> u64;
    /// Decode `act` on `node` in the last tree and step the root; false when it decodes to nothing.
    fn act(&mut self, node: jig_view::NodeId, act: jig_view::Act) -> bool;
    /// Sign in by the printed link (jig-shell-native's `Shell::sign_in`).
    fn sign_in(&mut self, link: &[u8]) -> Result<(), jig_shell_native::Unlinked>;
    /// What the client tells the shell, oldest first.
    fn notice(&mut self) -> Option<Notice>;
    /// No new exchange; the small store's last write finished; connections and files closed.
    fn close(&mut self);
}
/// What the native client tells the terminal (client.md, 6.6, 6.7).
pub enum Notice {
    /// Signed in, by a token loaded or by the link; the root has started.
    SignedIn,
    /// The sign-in by the link had no answer or was answered not ready.
    SignInAgain,
    /// The link was refused: spent, expired, or the engine's sign-ins full.
    SignInRefused,
    /// Signed out by the frame's button, or the engine said the sign-in ended.
    SignedOut,
    /// A service's credential the link handed the shell, printed once in base64url.
    Credential { party: u64, token: skein_lib::Secret },
    /// The small store could not be written.
    StoreFailed,
    /// jig-shell-native refused DIRECTORY.
    DirectoryRefused,
}
pub struct Loop { /* lines, numbers, prints, the link while signing in, closing */ }
impl Loop {
    pub fn new(limits: &limits::Terminal, bounds: render::Bounds, frame: render::Frame,
               directory: Box<[u8]>, link: Option<skein_lib::Secret>) -> Loop;
    /// Bytes read from stdin: each ended line one command, done at once.
    pub fn stdin<C: Client>(&mut self, bytes: &[u8], now: Time, client: &mut C);
    /// stdin ended, or SIGINT or SIGTERM: close, without signing out.
    pub fn close<C: Client>(&mut self, client: &mut C);
    /// After the client's iteration: notices into lines, retries, a print when due.
    pub fn iterate<C: Client>(&mut self, now: Time, client: &mut C);
    /// The next bytes for stdout, one write at a time.
    pub fn output(&mut self) -> Option<Box<[u8]>>;
    /// stdout took the last output.
    pub fn written(&mut self);
    pub fn next_deadline(&self) -> Option<Time>;
    /// Once closed and nothing waits for stdout: 0, or 1 with its stderr line.
    pub fn exit(&self) -> Option<(u8, Option<Box<[u8]>>)>;
}
/// The process: the native client and the terminal's io (bit 62) on one ring.
pub struct Terminal { /* client, loop, io, stdin, stdout, signals tokens */ }
impl Terminal {
    pub fn new(client: NativeClient, terminal: Loop, stdin: Fd, stdout: Fd, signals: Fd) -> Result<Terminal, Fd>;
}
impl skein_shell::Host for Terminal { /* iterate: reap by tag, client first, then the loop */ }
```

Prints (6.3): the first print when the first tree is built; then when
`built()` moved and a second has passed since the last print, or at
once on `p`, or at once after a command that acted, once the client's
next build is seen; while an output is in flight a due print waits and
then shows the tree as it is. A refusal's line, then the page again when
`reprints()`. The shell's lines (6.6, 6.7) queue in order with the
prints, at most `lines`. While signing in by the link: `SignInAgain`
prints its line and calls `sign_in` again after the backoff;
`SignInRefused` ends with exit 1 and the line `temper open: the link was
refused: use the fresh link temper serve printed`. The link's secret is
dropped once signed in.
Test shape: `tests/terminal.rs` and `tests/fuzzy_terminal.rs`; support
in `tests/support/mod.rs`: `Script` (a `Client` whose tree changes at
given times and which records the acts it is given), `World { terminal:
Loop, client: Script, now }` with `type_bytes(&[u8])`, `stdout_takes()`,
`run_until(Duration)`, `prints_at() -> Vec<Duration>`. Tests:
`a_change_within_the_interval_prints_once_when_it_ends`,
`an_act_prints_at_once_after_the_next_build`,
`a_due_print_waits_for_stdout_then_shows_the_tree_then`,
`a_command_cut_at_any_byte_acts_once` (rows: each split of `4\n`),
`the_end_of_stdin_closes_and_exits_zero_after_the_drain`,
`an_interrupt_closes_without_signing_out`,
`signing_in_without_an_answer_tries_again_after_its_backoff`,
`a_refused_link_exits_one_with_its_line`; fuzzy:
`random_cuts_and_stdout_delays_lose_no_command_or_change` (seeds 1 to
16). Each focused test under 20 ms; the fuzzy test under 2 s. Reference
test, in full:

```rust
#[test]
fn a_change_within_the_interval_prints_once_when_it_ends() {
    let mut world = support::World::signed_in();
    world.client.change_at(Duration::from_millis(200));
    world.client.change_at(Duration::from_millis(700));
    world.run_until(Duration::from_millis(1500));
    assert_eq!(world.prints_at(), vec![Duration::ZERO, Duration::from_secs(1)]);
}
```

Reference fuzzy test, in full (`tests/fuzzy_terminal.rs`; `World::random(seed)`
draws from the seed a list of commands typed in chunks cut at drawn
bytes, drawn times stdout takes each write, and drawn tree changes;
`typed()` is the commands it typed, `acts()` the acts the script
recorded, `shown_last()` whether the last print shows the script's last
tree):

```rust
#[test]
fn random_cuts_and_stdout_delays_lose_no_command_or_change() {
    for seed in 1..=16 {
        let mut world = support::World::random(seed);
        world.run_until(Duration::from_secs(10));
        assert_eq!(world.client.acts(), world.typed(), "seed {seed}: each command acted once, in order");
        assert!(world.shown_last(), "seed {seed}: the last change was printed");
    }
}
```

Proof: world, `cargo nextest run -p temper-shell-terminal --test
terminal`, run once: prints come at most once a second and at once after
an act, a print waits for stdout, a command cut at any byte acts once,
and the end of stdin, an interrupt and `q` close with exit 0 (the eight
tests, under 0.3 s). The process layer needs 16.4's binary.
Don't: put the terminal in raw mode; sign out on closing; print the
link's secret, a bearer token or a credential twice; start a thread in a
test; read a clock in `Loop` (time is its argument).
Size: at most 450 lines of production code.
Removes: nothing.

### 16.4 `temper open` in the binary, and the terminal's first process test

Scope: the binary's `open` command (client.md, 6.1; configuration.md,
9): the two arguments, the link or address read and checked (HOST an IP
address on loopback), DIRECTORY's parent opened, the token file read
once when an address is given, the native client and the terminal
started on skein's loop, exit 0 or 1 with 6.1's lines; the process
tier's terminal test, its first part: signing in by the printed link,
the inbox printed, `q`, exit 0.
Needs: 10.4's process support (`crates/temper/tests/support/`, the
`process` profile) and 10.3's binary.
Files: `crates/temper/{Cargo.toml (`temper-shell-terminal`), src/lib.rs
(`Command::Open`, the usage line), src/open.rs, src/tests.rs,
tests/startup.rs, tests/process.rs, tests/support/terminal.rs}`. Shared
hotspots: `crates/temper/src/lib.rs`, `tests/startup.rs` (10's),
`tests/process.rs`, `tests/support/` (10.4's).
Shape (ordinary Rust, not compiled):

```rust
// crates/temper/src/open.rs
/// The second argument, read (client.md, 6.1).
pub enum Target {
    /// `http://HOST:PORT/jig/v1/sign-in/link?secret=SECRET`: signed in by it first.
    Link { engine: std::net::SocketAddr, link: skein_lib::Secret },
    /// `http://HOST:PORT` or `http://HOST:PORT/`: the token DIRECTORY holds.
    Address { engine: std::net::SocketAddr },
}
/// Why `temper open` did not start, each with its stderr line (6.1's table).
pub enum Refused { Usage, NotALink(Box<[u8]>), NotLoopback, Directory(String), NotSignedIn, LinkRefused }
impl Refused { pub fn line(&self, directory: &Path) -> String; }
pub fn target(argument: &[u8]) -> Result<Target, Refused>;
/// Starts the terminal and drives it to its exit: 0, or 1 with the line on stderr.
pub fn run(arguments: &[OsString], clock: &skein_shell::Clock) -> u8;
```

`run`: two arguments or `Usage`; `target`; DIRECTORY's parent with
`skein_shell::open_root` (an errno → `Directory("cannot open its parent:
errno N")`); with an address, `read_private_file(DIRECTORY/token, 40,
effective_user())`: `Absent` or an empty file (jig-shell-native's
signed-out token) → `NotSignedIn`, another refusal → `Directory` with its
`Unread`; 12.1's `NativeClient::new` with DIRECTORY's last component,
the engine's address, `effective_user()` and `skein_shell::seed()`;
`Terminal::new` on fds 0 and 1 and `open_termination_signals()`;
`Kernel::open(Config { operations })`; `drive`; the `Loop`'s exit and
line.
Test shape: unit tests in `crates/temper/src/tests.rs` (no file):
`each_open_argument_reads_as_a_link_an_address_or_a_refusal` (rows: the
printed link, `http://127.0.0.1:7000`, the same with `/`, `[::1]:7000`'s
URL, `127.1.2.3`, `https://…`, a host name, no port, `10.0.0.1`, a path
of another shape), `open_without_two_arguments_prints_its_usage`; in
`crates/temper/tests/startup.rs` (focused suite; `CARGO_TARGET_TMPDIR`
is set for integration tests only):
`an_address_with_no_token_in_the_directory_is_not_signed_in` (rows:
absent, empty; scratch under `CARGO_TARGET_TMPDIR`); process test in
`tests/process.rs`: `temper_open_signs_in_prints_the_inbox_and_exits_on_q`,
on 10.4's `support::{Scratch::new(case), configuration(dir, &Forge),
serve(dir), printed_link(stdout)}` (16.4 adds `support::private(path) ->
bool`, a file of mode 0o600, and `WITHIN`, 30 s, if 10.4 has not), with
`tests/support/terminal.rs`'s `link(&mut serve, within)` (reads `temper
serve`'s stdout until `printed_link` finds the first line), `open(..)`
(the `temper open` child) and `Typist`, a host of the test's real
world writing lines to the child's stdin pipe and reading its stdout
pipe (after jig-test's `support/mod.rs` `Person`, with
`skein_world::end_to_end::Binary::start(.., Mode::Pipes, ..)` and its
`Streams::Pipes`). Unit tests under 20 ms; the process test aims at 30 s.
Reference test, in full:

```rust
#[test]
fn temper_open_signs_in_prints_the_inbox_and_exits_on_q() {
    let scratch = support::Scratch::new("temper_open_inbox");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let mut serve = support::serve(scratch.path());
    let link = support::terminal::link(&mut serve, support::WITHIN);
    let open = support::terminal::open(scratch.path(), &scratch.path().join("client"), &link);
    let mut typist = support::terminal::Typist::new(open);
    typist.wait_for(b"== Inbox == Live", support::WITHIN);
    typist.type_line(b"q");
    assert_eq!(typist.exit(support::WITHIN), 0, "stderr: {}", typist.stderr());
    assert!(support::private(&scratch.path().join("client/token")), "the bearer token is kept 0o600");
    scratch.remove();
}
```

Proof: process, `cargo nextest run -p temper --profile
process -E 'test(temper_open_signs_in_prints_the_inbox_and_exits_on_q)'`,
run once: the real `temper open` signs in by the link `temper serve`
printed, prints the inbox with the link live, and exits 0 on `q` with
its token kept private; under 60 s.
Don't: read a configuration file; accept a host name or an address off
loopback; print the link or the token; leave `temper serve` running
after the test (10.4's support ends it).
Size: at most 200 lines of production code.
Removes: nothing.

### 16.5 `temper sign-in`

Scope: the binary's `sign-in` command (credentials.md, 3): the
configuration read and checked as `temper serve` reads it, no secret
file read; the account's refusals; smith's accounts driver's sign-in on
one `Source::SignIn`, the visit's URL on stdout, the record kept at
`llm.records/N`, exit 0 or 1; tested against skein's fake issuer
(credentials.md, 7).
Needs: 10.1's startup (the document's reading and checks, the problems'
formatter); 10.4's process support; for its last test, 12.2's process
story (a chat answered by the real binary).
Files: `crates/temper/{Cargo.toml (dev: `smith-local-process-world`),
src/lib.rs (`Command::SignIn`, the usage line), src/sign_in.rs,
src/tests.rs, tests/process.rs, tests/support/sign_in.rs}`; the workspace
`Cargo.toml` (`[workspace.dependencies]` `smith-local-process-world`);
`Cargo.lock`. `smith-local-process-world` is not in the lock: this
increment runs code-organization.md, section 3's three commands, in
order (`cargo update -p jig-domain-core --precise
d91a89c45553f3f259b4d95d34a75164a9715877`, `cargo update -p
smith-host-domain --precise dfc0fe5700c628d7bce7da3d2ee918e9c1fbe46d`,
`cargo update -p skein-lib --precise
b5d3a2c3363dbbd1db8dae933cd87a643a18b3f2`), and before merging `grep -oE
'forgetic/(jig|smith|skein)\.git#[0-9a-f]+' Cargo.lock | sort -u` prints
exactly the three pinned revisions. Shared hotspots:
`crates/temper/src/lib.rs`, `tests/process.rs`, `tests/support/`, the
workspace `Cargo.toml`, `Cargo.lock`.
Shape (ordinary Rust, not compiled):

```rust
// crates/temper/src/sign_in.rs, copied from smith's credentials/login.rs
/// The account to sign in, as the configuration names it.
pub struct SignIn {
    pub number: u32,
    pub registration: skein_oauth::Registration, // jig's SignIn: client, authorization, token, redirect, scope; Form; no secret
    pub endpoint: skein_oauth_accounts::Endpoint, // resolved as jig-test's lib.rs:117-178 resolves an issuer
    pub records: Box<[u8]>,                       // engine.llm.records
}
/// Why the account is not signed in at a terminal, naming it (credentials.md, 3).
pub enum Refused { Usage, Configuration(String), NoAccount(u32), HandedIn(u32), Confidential(u32), Failed(String) }
impl Refused { pub fn line(&self) -> String; }
/// The document read and checked, the account found and refused or taken.
pub fn account(document: &[u8], number: u32) -> Result<SignIn, Refused>;
/// The driver run on `Login`, smith's host (login.rs:138-205): 0, or 1 with the reason on stderr.
pub fn run(arguments: &[OsString], clock: &skein_shell::Clock) -> u8;
```

`account`: the file with `skein_shell::read_file` within jig-test's
`CONFIG_BYTES`; 10.1's reading and checks (configuration.md, 2's steps
1 to 3; problems one line each); `jig_protocol_config::engine`'s
`llm.accounts`; then, before anything else: no account of that number,
a `Source::HandedIn`, a `SignIn` whose `secret` is set, each refused
naming the account. `run`: `Accounts::new` with one `Source::SignIn {
account, root: open_root("."), user: effective_user(), directory:
records, file: decimal(number), registration, endpoint, account_id:
empty, interactive: false }`, entropy from `skein_shell::seed()` as
smith's `oauth_entropy` (`local_shell.rs:239-246`); `sign_in(number)`;
`drive`; `CredentialRequest::Visit { url }` → one line on stdout;
`SignedIn` → exit 0; `Failed` or the driver's deadline → exit 1, the
reason on stderr; never a token printed.
Test shape: unit tests in `crates/temper/src/tests.rs`:
`each_unsigned_account_is_refused_naming_it` (rows: no such number, a
handed-in account, a client secret), `sign_in_without_two_arguments_prints_its_usage`;
process tests in `tests/process.rs`, with `tests/support/sign_in.rs`
hosting skein's fake issuer (`skein_fake_peers::oauth::Peer`) and
smith's `Browser` (`smith-local-process-world`'s `oauth::Browser`) in
the test's real world, as jig-test's support hosts its fake LLM peer:
`temper_sign_in_prints_the_visit_and_keeps_the_record`,
`a_running_engine_lends_the_record_temper_sign_in_kept`.
`tests/support/sign_in.rs`: `ACCOUNT: u32 = 1`, `AUTHORIZATION` (the
issuer's authorization URL), `configuration(dir)` (10.4's
`configuration(dir, &Forge::Unreached)`, then `engine.llm.accounts`
holding account 1 signed in at the fake issuer, `engine.llm.records`
`accounts/records`), `issuer()`, `start(dir,
account)` (the `temper sign-in` child, stdout piped), `browse(&issuer,
visit)` (the browser run in the real world until the redirect is
answered). Unit tests
under 20 ms; each process test aims at 30 s. Reference test, in full:

```rust
#[test]
fn temper_sign_in_prints_the_visit_and_keeps_the_record() {
    let scratch = support::Scratch::new("temper_sign_in_record");
    support::sign_in::configuration(scratch.path());
    let issuer = support::sign_in::issuer();
    let mut child = support::sign_in::start(scratch.path(), support::sign_in::ACCOUNT);
    let visit = child.line(support::WITHIN);
    assert!(visit.starts_with(support::sign_in::AUTHORIZATION), "the visit is the issuer's: {visit:?}");
    support::sign_in::browse(&issuer, &visit);
    assert_eq!(child.exit(support::WITHIN), 0, "stderr: {}", child.stderr());
    assert!(support::private(&scratch.path().join("accounts/records/1")), "the record is kept 0o600");
    scratch.remove();
}
```

Proof: process, `cargo nextest run -p temper --profile
process -E 'test(/sign_in/)'`, run once: the real binary prints the
issuer's visit, takes the redirect, keeps the record privately and exits
0, and a running `temper serve` lends the new record at its next grant
(the fake LLM peer sees the signed-in token); under 90 s.
Don't: open a store or start a service; read a secret file; refresh an
account; print a token; open a browser; write under `/tmp`.
Size: at most 350 lines of production code.
Removes: nothing.

### 16.6 Adopting a repository by commands

Scope: the adoption form's golden pages, each state of client.md, 4.4
and 4.7 (6.8), and the process tier's terminal test's adoption: the
form's fields written with `n words`, "It runs CI" toggled off and one
check named, `Adopt` pressed with `n`, the outcome printed, `q`, exit
0 (domain/testing.md, 2.6). The project's page its link opens is not waited
for: the process configuration's landing rule names the forge's
patterns, which the native client takes no page of at temper's pin
(client.md, 5.4); nor the repository on it (client.md, 4.3).
Needs: 15.1's form (the forge's page, bindings and navigation link);
15.3's process story (the forge component wired, the throwaway Forgejo
in the process support).
Files: `crates/temper-shell-terminal/src/tests/{render.rs, fixtures.rs,
golden/adopt_*.txt}`; `crates/temper/tests/{process.rs,
support/terminal.rs}`.
Shape: no production code: 16.1's renderer prints the forge's page as
any other; 16.2's reader acts on its fields, check and button.
Test shape: `each_adoption_form_state_prints_as_its_golden_file` (rows:
`adopt_empty`, `adopt_invalid`, `adopt_pending`, `adopt_adopted`,
`adopt_refused_not_found`, `adopt_refused_core`, `adopt_out_of_date`;
the form's states built as 15.1's view tests build them), under 20 ms;
`temper_open_adopts_a_repository_by_commands` in `tests/process.rs`,
aiming at 60 s: 15.3's `support::forgejo::Throwaway::start(&scratch)`
(`acme`, `temper-bot`, `acme/widget`) and `configuration(dir,
&throwaway.forge())`, the typist following the navigation's link,
writing owner `acme`, name `widget`, landing branch `main` and one
check, toggling CI, pressing Adopt, then waiting for "Adopted
acme/widget on forge 1 as owned.", then `q`. The project's page is not
opened: its policy names the forge's patterns, past what the native
client takes at temper's pin (client.md, 5.4), and jig's core lists no
owned resource there (client.md, 4.3).
Proof: process, `TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v16.0.5/forgejo-16.0.5-linux-amd64
scripts/proof-one-process.sh`, run once: its log names the
story and the terminal's tests passed (the README's "Done when", 8);
within the `process` profile's `global-timeout`, 10 minutes (10.4).
Don't: adopt through anything but typed commands; uncheck "It runs CI"
without a check (the forge connector refuses it); touch any Forgejo but
the one `TEMPER_FORGEJO` names.
Size: test code only.
Removes: nothing.

## Interface sketches

`s16.rs` holds these, compiled under the workspace's lints. `render`
and `command` are the terminal's decisions, step code; the loop owns
the numbers between prints and ends each print when stdout takes it.

```rust
//! Sketches of session 16: the step modules of `temper-shell-terminal` that
//! decide what the terminal prints and does (client.md, 6.8): `render`
//! (16.1), `command` (16.2), and the terminal's limits in `limits` (16.3).
//! The loop (`terminal`), `temper open` and `temper sign-in` are ordinary
//! Rust: their sketches are in the session file, not compiled here.
#![expect(unused_variables, reason = "a sketch's bodies are unreachable; its parameters are its signature")]

/// `crates/temper-shell-terminal/src/render.rs`: the tree to lines
/// (client.md, 6.4) and the print's numbers (6.5). Owner: 16.1. Users:
/// 16.2 (`Numbers`), 16.3 (the loop prints with it).
pub mod render {
    use alloc::boxed::Box;
    use jig_view::{NodeId, Tree};

    /// The most one print holds, counted from the view's tree limits
    /// (client.md, 6.8): every node on a line of its own, every newline
    /// of its text one more, each byte at most three (U+FFFD).
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Bounds {
        /// Lines, the first `== TITLE == LINK` line included.
        pub lines: u32,
        /// Bytes, every line's indentation, marks, states and `\n` included.
        pub bytes: u32,
        /// Numbered nodes: at most the tree's bindings.
        pub numbers: u32,
    }

    /// The words of jig's frame the renderer compares a status with, said
    /// once at start from jig's phrasebook: the page is behind
    /// (`JigPhrase::Behind`, "Behind: waiting for current facts").
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Frame {
        pub behind: Box<[u8]>,
    }

    /// A print's numbers (client.md, 6.5): number `n`, from 1, names the
    /// `n`th node a person can act on, in preorder, by its id; the loop
    /// keeps them until its next print.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Numbers {
        _nodes: (),
    }

    impl Numbers {
        /// The node number `n` named at the print, if `n` was shown.
        #[must_use]
        pub fn node(&self, number: u32) -> Option<NodeId> {
            unreachable!("sketch")
        }
        /// How many numbers the print showed.
        #[must_use]
        pub fn count(&self) -> u32 {
            unreachable!("sketch")
        }
    }

    /// One print: the page's lines, each ending in `\n`, and its numbers.
    #[derive(Debug)]
    pub struct Print {
        pub text: Box<[u8]>,
        pub numbers: Numbers,
    }

    /// The print's bounds for trees of `tree`'s limits; none past `u32`.
    #[must_use]
    pub fn bounds(tree: &jig_view::Limits) -> Option<Bounds> {
        unreachable!("sketch")
    }

    /// The whole page as text (client.md, 6.4): the first line, then every
    /// node in preorder by its role, states after it, numbers before it;
    /// text printable as 6.4 says. Asserts the print fits `bounds`.
    #[must_use]
    pub fn print(tree: &Tree, frame: &Frame, bounds: &Bounds) -> Print {
        unreachable!("sketch")
    }

    /// The heap one print holds at most: its text and its numbers.
    #[must_use]
    pub fn worst_case(bounds: &Bounds) -> Option<u64> {
        unreachable!("sketch")
    }
}

/// `crates/temper-shell-terminal/src/command.rs`: stdin's bytes to lines
/// (client.md, 6.2) and a line to a command or a refusal (6.5). Owner:
/// 16.2. Users: 16.3 (the loop).
pub mod command {
    use super::render::Numbers;
    use alloc::boxed::Box;
    use jig_view::{Act, NodeId, Tree};

    /// One line of stdin, made by `Lines::take` at its `\n`.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum Line {
        /// The bytes before the `\n`, a `\r` just before it dropped.
        Whole(Box<[u8]>),
        /// A line past the line limit, refused whole through its `\n`.
        TooLong,
    }

    /// stdin's bytes held until their `\n`: at most the line limit and
    /// one `\r`; past it, the rest of the line is skipped.
    #[derive(Debug)]
    pub struct Lines {
        _held: (),
    }

    impl Lines {
        /// Lines of at most `limit` bytes, a final `\r` aside.
        #[must_use]
        pub fn new(limit: u32) -> Lines {
            unreachable!("sketch")
        }
        /// One byte read from stdin; a line once it ends.
        pub fn take(&mut self, byte: u8) -> Option<Line> {
            unreachable!("sketch")
        }
        /// Whether bytes of an unended line are held (stdin's end drops them).
        #[must_use]
        pub fn holding(&self) -> bool {
            unreachable!("sketch")
        }
    }

    /// What one line asks of the loop (client.md, 6.5).
    #[derive(PartialEq, Eq, Debug)]
    pub enum Command {
        /// `n` on a button, a link, a tab or another bound node (`Press`),
        /// or on a check or a choice (`Toggle`).
        Act { node: NodeId, act: Act },
        /// `n` or `n WORDS` on a text field or a text area: `Input` with
        /// the words (none for `n` alone), then `Submit` when the field
        /// submits on enter.
        Write { node: NodeId, value: Box<[u8]>, submit: bool },
        /// `p`: print the page again, at once.
        Print,
        /// `q`: close the client (client.md, 6.6).
        Close,
        /// An empty line, or spaces only.
        Nothing,
        /// No command: the loop prints the refusal's line.
        Refused(Refusal),
    }

    /// Why a line is no command, each with its line (client.md, 6.5).
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Refusal {
        /// `temper: no [n] on the page`, then the page again.
        NoNumber { number: u32 },
        /// `temper: [n] is gone`, then the page again: its node left the
        /// tree, or is disabled or bound to nothing now.
        Gone { number: u32 },
        /// `temper: [n] takes no words`: words for a node not a field.
        NoWords { number: u32 },
        /// `temper: no page yet`: a number before the first print.
        NoPage,
        /// `temper: line too long`.
        TooLong,
        /// `temper: commands: n, n words, p, q`.
        Unknown,
    }

    impl Refusal {
        /// Whether the page is printed again after the line.
        #[must_use]
        pub fn reprints(self) -> bool {
            unreachable!("sketch")
        }
    }

    /// One line, read against the last print's numbers (none before the
    /// first) and the view's last tree; spaces around the line removed.
    #[must_use]
    pub fn read(line: &[u8], numbers: Option<&Numbers>, tree: &Tree) -> Command {
        unreachable!("sketch")
    }

    /// The refusal's line, `temper: ` first, `\n` last.
    #[must_use]
    pub fn refusal_line(refusal: Refusal) -> Box<[u8]> {
        unreachable!("sketch")
    }
}

/// `crates/temper-shell-terminal/src/limits.rs` (12.1's module): the
/// terminal's own bounds, which 16.3 adds beside the client's.
pub mod limits {
    use skein_lib::Duration;

    /// The terminal's bounds (client.md, 6.2, 6.3, 6.7), constants of
    /// the crate (the client reads no configuration).
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Terminal {
        /// A line's bytes, past which it is refused whole: 4,096.
        pub line: u32,
        /// stdin's bytes taken by one read: 4,096.
        pub read: u32,
        /// The print interval, a guide: one second.
        pub interval: Duration,
        /// The shell's own lines waiting for stdout at once: 32.
        pub lines: u32,
        /// Signing in by the link, after no answer: the first wait, 250 ms.
        pub first: Duration,
        /// ... doubling to at most this one: 8 s.
        pub most: Duration,
    }
}
```

## Questions to raise when reached

- **DIRECTORY refused by jig-shell-native,** which closes on a failed
  private open and says not why (findings/jig-client.md, Discrepancy 4;
  later.md). Decided by ruling 134: the pinned shell gives no notice,
  so the native client reads a shell that is idle before it loaded as
  `DirectoryRefused`, and `temper open` prints `temper open: DIRECTORY:
  refused (not a private directory of this user's, or its records
  unreadable)` and exits 1.
- **The native client's notices and build count** (16.3's `Client`) if
  12.1's `NativeClient` lands without them. Default: 16.3 adds them to
  `src/client.rs` within 40 lines, named in its commit body.
- **`temper sign-in`'s checks without starting:** if 10.1 has no entry
  point that runs configuration.md, 2's steps 1 to 3 alone. Default:
  `account` runs 10.1's `named` (which reads no file) and
  `jig_protocol_config::engine`, and reports their problems, one line
  each.
- **The sign-in's record root.** The engine's accounts keeper resolves
  `llm.records` under the process's working directory (jig-test's
  `lib.rs:179-181`). Default: `temper sign-in` opens `.` the same way, so
  it must run where `temper serve` runs; if 10.3 opens another root, the
  question names 10.3's line.
- **A status with neither text nor name** (client.md, 6.4 says only that
  a status with no text prints its name). Default: `*` alone.

## Done when

- The crate's tests pass: `cargo nextest run -p
  temper-shell-terminal` and `cargo nextest run -p
  temper-shell-terminal --profile fuzzy` pass.
- The binary's unit tests pass: `cargo nextest run -p temper`
  passes.
- The step modules keep the subset: `bash -O nullglob -c "grep -nE
  'Vec<|String|format!|std::|\|[a-z_, ]*\||\bloop\b|\bwhile\b'
  crates/temper-shell-terminal/src/{route,render,command,limits}.rs
  crates/temper-shell-terminal/src/{route,render,command}/*.rs"` prints
  nothing.
- The golden files changed only in this session's steps: `git log
  --format='%(trailers:key=Step,valueonly,separator=)' --
  crates/temper-shell-terminal/src/tests/golden | grep . | sort -u`
  prints `temper-16.1` and `temper-16.6` and nothing else.
- The process tier passed once: `TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v16.0.5/forgejo-16.0.5-linux-amd64
  scripts/proof-one-process.sh` exits 0, its log naming the
  story and `temper_open_adopts_a_repository_by_commands` passed.
- The terminal's world keeps its share: `cargo nextest run
  --workspace --profile measure -j 1 -E 'package(temper-shell-terminal)
  & kind(test) & not binary(/^fuzzy_/)'`, then `awk -F'[][]' '/PASS \[/
  {s += $2} END {printf "%.1f\n", s}' <its log>`, prints at most 0.3;
  the same with `--ignore-default-filter` added and `binary(/^fuzzy_/)`
  in place of `not binary(/^fuzzy_/)`, at most 2.0.
- The gate passed on this session's last landing: `git -C
  ~/src/rust/temper log -1 --grep='^Step: temper-16\.6$'
  --format='%(trailers:key=Gate-Result,valueonly)' origin/main` names
  `scripts/check.sh passed`.
