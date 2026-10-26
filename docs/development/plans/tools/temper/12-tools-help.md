# Session 12: the tools' help and JSON calls

From temper 02's table come `--call`'s JSON and a tool's record for
`help TOOL --json` (12.1), then help's usage lines, the agents' guide
and their goldens (12.2). The tests check that every derivation names
every tool. This is `temper-client-tools`' step code: it reads no file
and prints nothing. It runs alongside temper 03; the next plan's help
wiring and process tier need its capability.
Steps: `temper-12.1` and `temper-12.2`.

Needs at start: the tools' table.
Capability: the tools' help.

## Read first

- temper's `docs/design/client/tools.md`: 4, 5.1 to 5.4, 5.6, 6.1 to 6.3, 7,
  8.1, 8.2, 10, 11.1, 11.3, 11.5 and 12, including `fe0ec316`'s
  kept, pending, dropped and unsent output; `docs/design/client/README.md`, 4.4.
- `temper/02-tools-table.md`: `Tool`, `Argument`, `Call`, `Parsed`,
  `Source`, `Usage`, `table::TOOLS`, `jig::argv::check`,
  `jig::subject::{Subject, Condition, condition, takes, phase, phase_word}`,
  `limits` and `src/tests/fixtures.rs::EVERY_TOOL`.
  `temper/03-tools-root.md`: the result rendering, outside this session.
- `temper/04-waits.md`: the conditions' meaning, outside this session.
  The next plan's host wires pending, JSON and help; its process tier
  tests help through the executable.
- `decisions.md`, whole: design questions 21 to 34, especially **29,
  CORRECTED** and 33's gaps closed; the plan's own 5 to 7 and 15.
- skein's `docs/design/json.md`: 3, 4 and 5; code:
  `crates/skein-json/src/{tokenizer,document,writer}.rs`
  (`Tokenizer`, `Document`, `Encoder`).
- temper's `crates/temper-engine-smith/src/json.rs`, `parse` (only its
  tokenizer drive, not its recursive value parser);
  `crates/temper-client-shell-terminal/src/tests/fixtures.rs`, `golden`.
- temper's `docs/development/workflow.md`, 2; `findings/temper-area-1.md`,
  2 and 7; the client-api plan's `code-organization.md`, 2, 5 and 7.
- skein's `programming-model.md`: 3, 6.3 and 10; its
  `testing-strategy.md`: 2.1, 8 and 10.

## What the design asks

- **4, 5.1:** JSON and help derive from the table, in the module `jig`
  that names no temper item; the guide's text belongs to temper.
- **5.3, 5.4:** the acts' forms and wait's arguments are the same in
  argv, JSON and help; temper 02 owns their records and syntax checks.
- **5.6, 8.1:** `--call FILE|-` gives one call; a TEXT `-` uses stdin
  once, never under `--call -`. The next plan's host reads its bytes.
- **6.2:** snake-case argument names, integers and no floats;
  `help TOOL --json` prints the tool's record, not a call's result.
- **5.6, 6.1, 6.2, 8.2:** guide examples of kept and pending output
  carry no name or key; unread records are counted, and other parties'
  kept requests stay parked while the call goes on.
- **6.3, 7, 11.5:** the guide explains sign-in, the wait/act loop,
  named keys and prefixes, reading acts, exit statuses, and forge reads.
  Unconfirmed or a client-side key conflict is a tools bug, exit 1,
  reported as a finding; exit 2 is the engine's answer alone.
- **11.1, 11.5, 12:** JSON round trips; help as golden text; the guide's
  calls checked by the table; every derivation names every tool; the
  help fits the tools' text window.

## Choices this plan makes

- **JSON forms:** a duration is integer seconds; a turn is `"3.7"`;
  a switch is `true`; a repeated argument is an array; `wait`'s subject
  is its argv words joined by one space; the key is the argument `"key"`.
  A TEXT and the whole `--call` document are bounded by
  `limits::tools().ask_bytes`; exceeding it is a usage error, exit 1.
- **Proposals** are `"tN/K"` or `"pN/K"`, preserving task or party,
  as the table and temper 03's printed reads do (5.3, 6.1).
- **Close** has the optional text argument `"reason"`, shown as
  `[--reason TEXT]`, empty by default (5.3).
- **Wait** uses temper 02's subject and condition checks, including in
  JSON; help describes the same rules (5.4). Subjects: `me`, `inbox`
  with no subject flags, `task N`, `tree N`, `run N`, and `tasks` with
  its read arguments except `--after` and `--ended`. Conditions:
  `me`: `waiting`; `inbox`: `waiting|updates`; `task N`:
  `ended|waiting-on-me|held|replied|phase:<phase>`; `tree N` and `run N`:
  `ended` only; `tasks`: none. `waiting-on-me` on a task covers its tree.
  The ten live phases are admitted; `phase:done|failed|cancelled|ended`
  is a usage error, exit 1 naming `--until ended`. An ended task's
  answer is its ended read. `--since` and `--until` together require
  both; a matching read's digest covers the watch's document, not its
  cursor. Timeouts use `limits::{WAIT, WAIT_MOST}` (300 s, 3600 s).
- **The guide's exits:** 0 for a read answered, an act done/proposed or a met wait;
  1 for usage and client failures, including limits, Held, NotKept and
  unknown past retention; 2 for an engine refusal alone; 3 for an unmet
  wait's timeout or watch end. A sized too-large turn stays 0 and is
  reported as a finding. Unconfirmed and a client-side key conflict
  cannot reach a call: `me` is live before sending and each call owns
  its keys. Either is a tools bug, exit 1: report it as a finding (6.3,
  11.5). An engine key conflict is 2. Runtime exits belong to temper 03.
- **Recovery examples:** `kept: ` plus jig's operation, or the forge's
  `stop`/`adopt`, then the outcome as an act prints it; JSON has only
  `operation` and `outcome`, one object per stderr line (6.1, 6.2).
  No name or either key: a record keeps the engine key, a named
  request's digest, not its name. Keep no runner records to retain names.
- **Pending examples:** each row has operation, `party N`, and first
  sending in temper's wall-time words or `not sent`; JSON is
  `{"pending":[{"operation":...,"party":N,"first_sent":...}],"unread":N}`,
  with jig's integer or null for `first_sent`. No name or key. Records
  of an unsupported store version or undeclared family owner/version
  are counted, never listed; malformed store files fail (5.6, 6.3).
- **One source of names:** `guide::GUIDE` is prose with checked call
  examples; the tool list printed after it comes from `table::TOOLS`.
  The coverage test checks that list as well as parsing, help and JSON.

## Rules that bind it

- **The boundary:** `src/jig/` names neither `temper` nor a crate item
  outside `crate::jig`; temper 02's boundary test continues to hold.
- **Step code:** programming-model.md, 10.3; no `std`, recursion,
  closures, `Vec`, `String`, `format!`, indexing or unchecked
  arithmetic outside tests. No file or stdin reads here.
- **Fixtures:** extend `src/tests/fixtures.rs`, made by temper 02;
  limits come from its `limits::client()` and `limits::tools()`.
  Unit tests take under 20 ms, unoptimized. No `[profile.*]` changes,
  repin or STATUS edit. No forge or LLM hotspot files change here.

## Increments

### 12.1 A call's JSON
Scope: `--call`'s JSON (8.1, 6.2): `{"tool": ..., "arguments": {...}}`
decoded into a call, checked as argv's is, and a call encoded back; a
tool's record encoded for `help TOOL --json`; a TEXT `-` refused under
`--call -`.
Needs: nothing beyond the session's start (the tools' table).
Files: new `crates/temper-client-tools/src/jig/json.rs` and
`src/tests/json.rs`; `src/jig.rs`, `src/tests.rs`, `src/tests/fixtures.rs`.
All paths below are under `crates/temper-client-tools/`.
Shape:
- `jig::json::call` drives `skein_json::tokenizer::Tokenizer` as
  `temper-engine-smith`'s `json::parse` drives its input, then builds
  `skein_json::Document::from_tokens`. Walk `Document::token` by checked
  index and container depth (at most 3), reading bytes with `text`,
  without recursion. `Compact::end` ends a text range, not a subtree.
- A member but `tool` and `arguments`, a repeated member, an argument
  the form lacks, or a value of another JSON kind is a usage error. Numbers are integers; a
  project is an integer or a string; the other forms are the Choices'.
  Decode the subject before checking its condition, regardless of JSON
  member order. Use temper 02's `jig::subject` types and parsers, then
  `jig::argv::check` for key, occurrence, kind, subject flags, condition
  and timeout; add no second check or phase vocabulary. Bad input exits 1.
- `write_call` and `write_tool` measure, then write, with
  `skein_json::writer::Encoder`. A record holds its name, line, area,
  keyed, result, and forms of arguments (name, shown, place, kind,
  occurs, line). `Encoder::measured`'s `TooLong` and `TooDeep` are
  `Refusal::Long`; its `Text` and `Number`, from argv bytes a `Value::Text`
  or `File` carries that are not UTF-8, are `Refusal::Text`: matched, since
  clippy.toml bans `map_err`.
Test shape: `src/tests/json.rs`. `fixtures::EVERY_TOOL` holds one argv
per row and form, from temper 02's tables; `fixtures::from_json`
decodes. The reference test:
```rust
#[test]
fn every_call_goes_from_argv_to_json_and_back_unchanged() {
    for words in fixtures::EVERY_TOOL {
        let call = fixtures::call(words);
        let json = jig::json::write_call(table::TOOLS, &call).expect("a call's JSON within its bound");
        assert_eq!(fixtures::from_json(&json), call, "{words:?}");
    }
}
```
The others, each under 20 ms:
`decides_arguments_are_proposal_or_task_and_hold_then_outcome_and_reason`
(both `t3/17` and `p3/17`; no unprefixed proposal),
`an_unknown_member_or_argument_is_a_usage_error` (duplicates included),
`a_value_of_another_json_kind_is_a_usage_error`,
`json_uses_the_tables_close_reason` (absent and explicit),
`json_wait_arguments_and_conditions_match_argv` (Choices' subject/condition
matrix, forbidden inbox flags and tasks' cursor/ended, all four terminal
phases, since/until together and timeout bounds),
`a_text_from_stdin_is_refused_under_call_stdin`,
`a_document_past_ask_bytes_is_a_usage_error` (`Usage::Json`; the length
is checked against `ask_bytes`, a u64, before the tokenizer, whose length
is a u32),
`each_tools_record_is_written_as_json`.
Proof: unit, `cargo nextest run -p temper-client-tools -E
'test(every_call_goes_from_argv_to_json_and_back_unchanged)'` (under 20
ms): every row's call goes from argv to JSON and back unchanged. No host reads
`--call` at this session's start; the next plan's host reads the file or stdin
and passes its source here. Result JSON remains temper 03's.
Don't: write a result's JSON (temper 03 and the next plan's host); use floats; duplicate
temper 02's condition parser or temper 04's evaluation.
Size: at most 250 lines.
Removes: nothing.

### 12.2 Help and the agents' guide
Scope: `temper ctl help [TOOL]` (5.6, 11.5): without TOOL, the guide and
then each tool's usage line and line; with TOOL, its record. Usage lines
derived from the rows, reading as 5.2 to 5.6 write them; the guide,
temper's text; each derivation naming every tool (5.1); help as golden
text (11.1).
Needs: a call's JSON and a tool's JSON record (temper 12.1); the root and
the runner, with `me` and `task N` (temper 03.2), for `fixtures::golden`.
Files: new `src/jig/help.rs`, `src/guide.rs`, `src/help.rs`,
`src/tests/help.rs` and
`src/tests/golden/{help,help_decide,help_close,help_wait,help_pending,help_adopt}.txt`
(recorded by hand); `src/lib.rs`, `src/jig.rs`, `src/tests.rs`, `src/tests/fixtures.rs`,
`src/tests/limits.rs` (`the_help_fits_the_tools_window`); `src/jig/rows.rs` (5.3's
argument order) and `src/jig/subject.rs` (`live`), as the Shape says.
Shape:
- `jig::help::usage` writes the name, each positional as `shown`, each
  flag as `--name SHOWN` (in `[...]` when optional, `...` after it when
  repeated, a set as its words joined by `|`), then `--key NAME` or
  `[--key NAME]` by `keyed`. Each of decide's forms gets a line.
  Its proposal is shown as `tN/K|pN/K`; `close` shows `[--reason TEXT]`.
  `wait`'s help gives Choices' subject arguments and condition matrix,
  the ten live words from `subject::phase_word`, and `--until ended`.
  It explains the matching read's digest and combined since/until.
- Against main (ruling 117): a switch (`Place::Switch`, its `shown`
  empty) is `[--name]`, its place read before its kind; a repeated flag
  is `[--name SHOWN]...`, as close's `[--move D]...`; a positional with
  a `Kind::Word` set shows its words joined by `|`, as decide's
  `release|leave|pass-up`; `Kind::Order` shows `activity|priority`, the
  words `argv::value` reads. The rows take 5.3's order: in
  `src/jig/rows.rs`, `CHAT_CREATE`'s `project` comes before `text`,
  `CANCEL`'s `grace` before `reason`, and the escalation form of
  `DECIDE` has `hold` after `task`. Where a row's `shown` or order still
  differs from 5.2's to 5.6's line, the row changes to the line's, with
  each test whose expected text follows the rows' order; list each in
  the commit body. `table::TOOLS` names `chat` twice (`rows::CHAT`,
  `rows::CHAT_CREATE`), and `argv::tool_index` finds the first: `help
  chat`, `help chat --json` and the tool list give every row of a name,
  in the table's order, and `help::text(Some(i))` writes row `i` and
  each later row of its name. `subject::live` becomes `pub(crate)`, and
  `wait`'s help writes `phase_word(p)` for each `p` it admits, over one
  const of every `Phase` in `subject.rs`.
- `jig::help::{tools, tool}` and temper's `help::text` measure into
  `skein_lib::Writer::new(text)`, `text` being
  `usize::try_from(limits::TEXT)`, then write into
  `Writer::new(measured.written())`: a `Writer` is exact-length, and
  `finish` asserts it full. `the_help_fits_the_tools_window` asserts on
  the measured length.
- `guide::GUIDE` covers sign-in (`sign-in -`, the link on stdin), the
  loop of 5.4, key names and their prefix (7), reading acts (5.3), the
  exit statuses (6.3), and reading the forge but changing it only
  through temper. Each call line starts `temper ctl DIRECTORY `.
  Give the failures Choices names, with this instruction on one line:
  `Unconfirmed or a client-side key conflict is a tools bug (exit 1); report it as a finding.`
  A repeat keeps its name, party and ask within retention; a changed
  template/bound in `me` can make another ask, an engine key conflict.
  Held may name another party's parked request; it is a client failure.
  A signal's status is the OS's, never 0; a stopping engine's wait
  continues through reconnect and fails only past the link deadline.
  The tool list after the guide is derived from `table::TOOLS`.
- Guide and tool-record result descriptions follow Choices' output.
  Show `kept: create done: task 41, chat`, without a name or key.
  After pending's rows: `N kept records these tools cannot read; a call
  that sends drops them`, exit 0, `unread` in JSON; an invalid store:
  `DIRECTORY's store cannot be read`, exit 1. A sending call prints
  `dropped: N kept records these tools cannot read`, JSON `{"dropped":N}`;
  parked work: `kept: N for party P, unsent while it is not signed in here`,
  JSON `{"party":P,"unsent":N}`. The sending call's notices go on stderr;
  it goes on, answering and erasing only the signed-in party's kept requests.
  `pending` reads only the store, with no lock, client or send (5.6).
Test shape: `src/tests/help.rs`; goldens read through
`fixtures::golden(name)` (temper 03's). The reference test:
```rust
#[test]
fn every_call_line_of_the_guide_parses_with_the_table() {
    let calls: Vec<&[u8]> = guide::GUIDE
        .split(|byte| *byte == b'\n')
        .filter_map(|line| line.trim_ascii_start().strip_prefix(b"temper ctl DIRECTORY ".as_slice()))
        .collect();
    assert!(calls.len() >= 4, "the guide shows the loop's calls");
    for call in calls {
        fixtures::parse_line(call).expect("each call of the guide parses with the table");
    }
}
```
That is its meaning, not its form: clippy.toml bans `Vec`, `slice::split`,
`filter_map` and `collect`. Write it as a `for` loop over
`0..=guide::GUIDE.len()` with `match guide::GUIDE.get(at)`, counting the
calls, and `fixtures::parse_line` likewise, without `Option::is_none_or`.
The others, each under 20 ms: `the_help_prints_as_its_golden_files`,
`each_derivation_names_every_tool` (compare `EVERY_TOOL`'s parsed rows
and forms, help's list, the JSON records and the guide's generated tool
list with `table::TOOLS`, in both directions; prose examples name only
members of that table),
`a_tools_usage_reads_as_the_design_writes_it` (a const table of 5.2's
to 5.6's lines, with Choices' settled forms),
`the_guide_distinguishes_engine_refusals_from_client_failures`
(all four exits, both impossible client answers called tools bugs and
reported as findings, the sized-turn exception),
`the_guide_shows_kept_pending_dropped_and_parked_work`
(golden examples have Choices' fields and messages, no request name or
key; unread records counted, invalid file refused). Among the bounds tests,
`the_help_fits_the_tools_window`.
Proof: unit, `cargo nextest run -p temper-client-tools -E
'test(every_call_line_of_the_guide_parses_with_the_table)'` (under 20
ms): the guide names only calls the table parses. No host prints help
at this session's start: the next plan's host connects it, and its process
tier checks the executable.
Don't: put the guide's words under `src/jig/`; print anything; implement
recovery or pending here. The next plan's real-loop stories prove those
paths, including whether dropped bytes persist as a jig finding (33).
Size: at most 200 production lines.
Removes: nothing.

## Interface sketches

temper 02 owns the table and call types; temper 12 owns these derivations.
The next plan's host reads the source of `--call`, passes whether it was stdin,
writes help's bytes and supplies the exit status. temper 03 owns the
result JSON and recovery notices; that host prints them and owns pending. The guide
follows their designed output and needs no running root or host. The
tool record's `name` and the call's `key` argument remain table fields;
neither adds a name or key to kept/pending results. A codec error ends
decoding before any call reaches the runner.

```rust
// crates/temper-client-tools/src/jig/json.rs
use crate::jig::{argv::Usage, call::{Call, Source}, table::{Limits, Tool}};
/// An encoded call or tool record cannot fit its output bound, or cannot be JSON.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    /// Nothing is written; the caller reports the bound.
    Long,
    /// An argument's bytes are not UTF-8; nothing is written.
    Text,
}
/// Decode one call with the table's checks; Source controls stdin's single use.
pub fn call(table: &[&Tool], bytes: &[u8], from: Source, limits: &Limits) -> Result<Call, Usage>;
/// Measure and encode a checked call, preserving its row, form and arguments.
pub fn write_call(table: &[&Tool], call: &Call) -> Result<Box<[u8]>, Refusal>;
/// A tool's record for help TOOL --json, including every argument form.
pub fn write_tool(tool: &Tool) -> Result<Box<[u8]>, Refusal>;
// crates/temper-client-tools/src/jig/help.rs
use skein_lib::{Overflow, Writer};
/// Write all of a tool's forms from its table record.
pub fn usage(tool: &Tool, out: &mut Writer) -> Result<(), Overflow>;
/// The table's complete tool list, used after the guide.
pub fn tools(table: &[&Tool], out: &mut Writer) -> Result<(), Overflow>;
/// One tool's record in text.
pub fn tool(tool: &Tool, out: &mut Writer) -> Result<(), Overflow>;
// crates/temper-client-tools/src/guide.rs
/// The agents' guide; every example is checked against the table.
pub const GUIDE: &[u8];
// crates/temper-client-tools/src/help.rs
use skein_lib::{Overflow, Writer};
/// None: guide then tools; Some: that tool's record. The host writes the bytes.
pub fn text(tool: Option<u8>, out: &mut Writer) -> Result<(), Overflow>;
```

## Questions to raise when reached

- If 12.2 measures the help without TOOL past `limits::TEXT` (32 KiB),
  raise it, with the size.

## Done when

Each in `~/src/rust/temper`, after `git fetch`:

- `git grep -q -E 'pub fn call\(' origin/main -- crates/temper-client-tools/src/jig/json.rs`
- `git grep -q -E 'pub fn write_call\(' origin/main -- crates/temper-client-tools/src/jig/json.rs`
- `git grep -q -E 'pub fn write_tool\(' origin/main -- crates/temper-client-tools/src/jig/json.rs`
- `git grep -q -F 'adopt --project P [--forge F] --owner O --name N --landing B [--role owned|fork|context] [--prefix X] [--no-home] [--no-ci] [--check G]...' origin/main -- crates/temper-client-tools`
- `git grep -q -F 'decide N --hold R release|leave|pass-up [REASON]' origin/main -- crates/temper-client-tools`
- `git grep -q -F 'decide tN/K|pN/K accept|reject|pass-up [REASON]' origin/main -- crates/temper-client-tools`
- `git grep -q -F 'close N [--move D]... [--reason TEXT]' origin/main -- crates/temper-client-tools`
- `! for t in me inbox tasks task chat activity turn run message close adopt answer decide release stop cancel wait sign-in sign-out pending help; do git grep -q -E "^ *$t( |\$)" origin/main -- crates/temper-client-tools/src/tests || echo "missing $t"; done | grep -q missing`
- `git grep -q -F 'temper ctl DIRECTORY wait me --until waiting' origin/main -- crates/temper-client-tools/src`
- `git show origin/main:crates/temper-client-tools/src/guide.rs | grep -F 'Unconfirmed or a client-side key conflict' | grep -F 'exit 1' | grep -q -F 'report it as a finding'`
- `git grep -q -F 'kept: create done: task 41, chat' origin/main -- crates/temper-client-tools/src/tests/golden`
- `git grep -q -F 'kept records these tools cannot read; a call that sends drops them' origin/main -- crates/temper-client-tools/src/tests/golden`
