# Session 02: the tools' table

A new crate, `temper-client-tools` (client/tools.md, 4), holds area 1's table, each tool
of 5.2, 5.3, 5.4 and 5.6 a record. It parses argv into a call and prints it back,
checks usage and TEXT `-`, derives the named key (7), and supplies section 12's
bounds and the tools' preset of jig-client's `Limits`. Its module `jig` names no
temper item, kept so by a test. temper 03 runs its calls; temper 12 derives their
JSON, help and guide; the next plan's host, `temper ctl`, feeds it argv. Steps:
`temper-02.1` to `temper-02.3`.

Needs at start: temper on skein's, smith's and jig's main.
Capability: the tools' table.

## Read first

- temper's `docs/design/client/tools.md`: 4, 5 (5.5: area 1 only), 6.2, 6.3,
  7, 8.1, 11.1, 12 and 13; `docs/design/client/README.md`, 4.4.
- jig's `docs/design/client/domain/contract.md`: 3.1, 6.1 and 7.1;
  `client/domain/client.md`: 14 and 20; skein's `docs/design/lib.md`, 6.2.
- `temper/01-repin.md`: the start capability, with skein `b4c6241`,
  smith `a277e922` and jig 01's landing; no repin here.
- `temper/03-tools-root.md`, `temper/04-waits.md` and
  `temper/12-tools-help.md`: the call's users and the derivations.
- `decisions.md`, design questions 21 to 34 (21 to 32 carried by temper
  `4e14c2b5` and jig `62bcf71e`).
- temper's `docs/development/workflow.md`, 2; the client-api plan's
  `code-organization.md`: 2, 5 and 7; `findings/temper-area-1.md`: 2, 7.
- Code, under `crates/`: `temper-client-shell-terminal/src/limits.rs`
  `native`; `temper-box/src/tests/boundary.rs`;
  `temper-client-domain/src/forge/form.rs` `form` and `broken`.
- skein's `programming-model.md`: 3, 6.3 and 10; its
  `testing-strategy.md`: 2.1, 8 and 10.

## What the design asks

- **4:** the crate, its dependencies and the step-code subset; the module
  `jig`, naming no temper item, kept so by a boundary test.
- **5.1:** a tool is a record; argv and its checks come from it here;
  JSON, help and the guide's names are temper 12's derivations.
- **5.2 to 5.4, 5.6:** area 1's rows: the eight reads, the acts, `wait`
  and the host's four tools; TEXT `-` once. temper 12 checks it against
  the source of a `--call` document.
- **6.3:** usage errors and the client's own answers exit 1; only the
  engine's refusal exits 2. A sized too-large turn stays 0 (5.2, 12).
- **7:** `--key NAME` and its engine key; a named request is built from
  the call and `me` alone, never the clock. temper 03 consumes jig 05's spans.
- **11.1:** step tests of argv, usage, the boundary and the bounds.
- **12:** `limits`' constants and the tools' preset. **13:** the key is
  the name jig's named send carries; this session only derives it.

## Choices this plan makes

- **Argument kinds:** the design's five (number, word, text, file,
  duration), and a count, a project, a turn `A.T`, a typed proposal,
  a cursor, a digest, a condition, a subject and a tool. jig's
  `TurnName`, `ProposalKey`, `Proposer`, `Phase`, `InboxGroup` and
  `QueryOrder` are matched both ways with no `_` arm.
- **The table is const data,** `&'static` slices as in jig's
  `DomainLimits.owners`, with no function pointers; one parser walks it.
- **jig's part** holds the reads, the acts but adopt, `wait` and the
  host's tools (sign-in's line says "the link the engine's host
  printed"). **temper's** holds adopt's row, `TOOLS` and the key's salt.
- **adopt's checks:** syntax here; the form's checks in temper 11's
  "the tools' adopt", through `forge::form(project)` and
  `forge::broken`. `Field.typed` is private and `write` is
  `pub(super)`, in a hotspot. Both stages exit 1 before anything is sent.
- **Argv forms:** `--kind` takes `questions`, `proposals`,
  `escalations`, `person-tasks`, `results` and `replies`; `--order` takes
  `activity|priority`, and a `tasks` call without it reads by
  `QueryOrder::Number`, ascending, `--ended` changing no order
  (decisions.md, J); a duration is whole seconds, or a number ending in
  `s`, `m` or `h`; a cursor is lower-case hex, as temper 03 prints it;
  a word after `--` is positional.
- **Proposal names:** `tN/K` is `Proposer::Task(N)`, `pN/K` is
  `Proposer::Party(N)`, with proposal K; no bare `N/K`. Parse and print
  alike, including temper 03's reads (client/tools.md, 5.3 and 6.1).
- **Close's reason:** `close N [--move D]... [--reason TEXT]`, empty by default (5.3).
- **Wait's terminal phases:** `phase:done|failed|cancelled|ended` give
  `Usage::Condition`, exit 1, naming `--until ended` (client/tools.md, 5.4). All
  14 phase words round-trip; `--until` admits only the ten live phases.
- **Watch arguments:** `inbox` takes no subject flag; `tasks` takes its
  read arguments except `--after` and `--ended`. Other arguments are
  usage errors, exit 1. `tree N` and `run N` take `ended` alone as a
  condition, `tasks` none; `waiting-on-me` is `task N`'s alone (client/tools.md, 5.4).
- **Bounded inputs:** TEXT and temper 12's JSON take at most `ask_bytes`; past it is a usage error.
- **The preset is written out,** and a test compares it with the
  terminal's `native().client`, `rows` and `text` aside, through a dev-dependency.

## Rules that bind it

- **The boundary:** nothing under `src/jig/` names `temper`, or a
  `crate::` path but `crate::jig`, so no test lies there: the tests of
  `src/jig/` are in `src/tests/`, temper 03 and 04's in `src/tests/jig/`.
- **The step-code subset** (programming-model.md, 10.3), outside the
  tests: `#![cfg_attr(not(test), no_std)]`, `#![forbid(unsafe_code)]`,
  `extern crate alloc;`; the subset's bans and checked arithmetic.
- **Fixtures** are one module, `src/tests/fixtures.rs`, made in 02.1 and
  extended by temper 03, 04, 11 and 12, never redefined; its `call` is the
  parser's, so temper 03's harness `submit`s. A unit test takes under 20
  ms, unoptimized (workflow.md, 2); no `[profile.*]` changes or STATUS edits.

## Increments

### 02.1 The crate, its bounds and the named key
Scope: the crate and its module `jig` (4); `limits` (12), with the
constants, the tools' preset and jig's part's bounds; the named key (7);
11.1's boundary and bounds tests.
Needs: nothing beyond the session's start.
Files: new `crates/temper-client-tools/{Cargo.toml, src/lib.rs,
src/jig.rs, src/jig/{key,table}.rs, src/limits.rs, src/key.rs, src/tests.rs,
src/tests/{fixtures,boundary,limits,key}.rs}`; the workspace
`Cargo.toml` (the member, and `temper-client-tools = { path =
"crates/temper-client-tools" }`); `Cargo.lock`.
Shape:
- Dependencies, each `.workspace = true`: `jig-client`, `temper-client-domain`,
  `temper-client-protocol-wire`, `skein-lib`, `skein-json`;
  `temper-client-shell-terminal` as a dev-dependency only (temper 03.2
  adds `skein-ui`). `[lints] workspace = true`.
- `limits`: `ROWS: u32 = 32`, `TEXT: u32 = 32_768`, `PART: u32 = 16_384`
  and `DOCUMENT: u64 = 1_048_576`; the `skein_lib::Duration`s `DEADLINE`
  (30 s), `LOCK` (60 s), `LOCK_TRY` (100 ms), `WAIT` (300 s) and
  `WAIT_MOST` (3600 s). `client()` writes out `native`'s numbers, with
  `rows: ROWS`, `text: TEXT` and `document_bytes: DOCUMENT` (client/tools.md, 12: a result's text, 32 KiB, is the client's); `tools()` reads from it, with
  `TEXT`, `PART`, `WAIT` and `WAIT_MOST`, into `jig::table::Limits`, defined
  here and re-exported as `jig::Limits`: all that argv, 03's runner and 04's digest read.
- `jig::key::named` feeds the salt, a zero byte and the name to
  `skein_lib::sha256::Sha256`, and keeps the first 16 bytes; temper's
  `key::TOOLS` and `key::engine` use it.
Test shape: `src/tests/{boundary,limits,key}.rs`; fixtures `name(&[u8])`
and `sources()`, reading `src/jig.rs` and `src/jig/**` under
relative paths, as temper-box's boundary test walks a relative `src`: cargo runs a test from its package's root, and
clippy.toml bans `core::env`, so no `env!` and no lint exception. Reference tests:
```rust
#[test]
fn each_ceiling_is_within_the_client_limit_it_reads_from() {
    let client = limits::client().client.domain;
    assert_eq!(limits::ROWS, client.rows, "a page's rows are the client's rows per window");
    assert_eq!(limits::TEXT, client.text, "a result's text is the client's text");
    assert!(u64::from(limits::TEXT) <= client.document_bytes, "a result's text fits one document");
    assert!(limits::PART <= limits::TEXT && limits::PART <= client.turn_bytes, "a part fits the text and a turn");
    assert!(limits::WAIT <= limits::WAIT_MOST && limits::LOCK_TRY <= limits::LOCK, "each default within its most");
}
#[test]
fn no_file_under_jig_names_temper_or_an_item_outside_jig() {
    for (path, text) in fixtures::sources() {
        assert!(!text.to_ascii_lowercase().contains("temper"), "{path} names temper");
        for (at, _) in text.match_indices("crate::") {
            assert!(text[at..].starts_with("crate::jig"), "{path} names an item outside jig");
        }
    }
}
#[test]
fn a_named_key_is_the_first_16_bytes_of_the_salted_digest() {
    let digest = skein_lib::sha256::digest(b"temper-tools\0chat-to-shop");
    assert_eq!(key::engine(&fixtures::name(b"chat-to-shop"))[..], digest[..16]);
}
```
The others, each under 20 ms: `the_tools_preset_is_the_terminals_but_its_rows`,
`the_tools_preset_has_a_worst_case` (`temper_client_domain::worst_case`),
`a_key_name_is_1_to_64_bytes_of_lower_case_digits_and_hyphens`.
Proof: unit, `cargo nextest run -p temper-client-tools` (under 1
s): the preset is the terminal's but its rows, each ceiling is within
its client limit, and `src/jig/` names no temper item. Nothing calls it
yet: temper 03's root takes the preset and the key, the next plan's host
the deadline and the lock.
Don't: depend on the terminal outside `[dev-dependencies]`; put the salt
under `src/jig/`.
Size: at most 180 lines.
Removes: nothing.

### 02.2 The table, a call and argv: the reads and the host's tools
Scope: the record and the call (5.1); `jig::argv::parse`, from the words
after DIRECTORY to a call or a `--call` source, with `--json`;
`jig::argv::back`, from a call to its words; the usage errors; TEXT or
FILE `-` once; jig's rows for 5.2's reads and 5.6's host tools; temper's
`table::{TOOLS, parse}`.
Needs: the crate, its bounds and the named key (temper 02.1).
Files: new `src/jig/{call,argv,rows}.rs`, `src/table.rs` and
`src/tests/table.rs`; `src/jig/table.rs`, `src/jig.rs`, `src/lib.rs`,
`src/tests.rs`, `src/tests/fixtures.rs`.
Shape:
- Rows are const, built with `positional`, `flag`, `switch` and
  `repeated`, one argument a line. Add each call variant with its row;
  the sketches show 02.3's completed types. `argv::check` owns the
  checks used by argv here and JSON in temper 12.
- The first word names the tool; `--call FILE|-` stands alone but for
  `--json`. A flag is given at most once, and each required argument is
  present. A value is of its kind: digits within its integer, a word of
  its set, a count within `rows`, a text within `ask_bytes`. `-` is taken
  once a call.
- A project of all digits is a number, otherwise a name, which the root
  resolves through `me` (5.1). `--kind` and `--order` parse into
  `InboxGroup` and `QueryOrder`; a `tasks` call without `--order` carries
  `QueryOrder::Number`.
- `back` writes the positionals, then the flags, in the row's order, so
  `parse(back(call)) == call` over every call `parse` makes. It writes no
  `--order` for `QueryOrder::Number`, and refuses a call whose order is
  `Changed` or `Ended` with `Unwritten::NoForm`: the tools never send
  either, and no argv gives it (decisions.md, J); a refusal, never a guess.
  Test shape's tables gain a `tasks` call without `--order`, which parses
  to `Number` and prints back as it was given, and a case each for
  `Changed` and `Ended`, whose `back` is `Err(Unwritten::NoForm)`.
Test shape: `src/tests/table.rs`, its tables in `const`s; the fixtures
`call(&[&str]) -> Call`, `parse`, `back` and `owned`; `EVERY_TOOL`
holds one argv per row and form, extended in 02.3 for temper 12. Reference:
```rust
const READS: &[&[&str]] = &[
    &["me"],
    &["inbox", "--project", "shop", "--kind", "replies", "--root", "41"],
    &["tasks", "--project", "3", "--label", "chat", "--tracked", "--order", "priority"],
    &["task", "41", "--tree", "--activity", "5", "--run"],
    &["chat", "41", "--before", "0a1b"],
    &["activity", "41", "--limit", "32"],
    &["turn", "41", "3.7", "--part", "2", "--from", "16384"],
    &["run", "41"],
];
#[test]
fn each_read_parses_into_its_call_and_prints_back() {
    for words in READS {
        assert_eq!(fixtures::back(&fixtures::call(words)), fixtures::owned(words), "{words:?}");
    }
}
```
The others, each under 20 ms: `each_host_tool_parses_into_its_call_and_prints_back`,
`an_unknown_tool_or_argument_is_a_usage_error`, `a_missing_or_repeated_argument_is_a_usage_error`,
`a_value_outside_its_kind_or_rows_is_a_usage_error` (`--limit 33`), `a_project_is_a_number_or_a_name`,
`a_second_stdin_is_a_usage_error`, `call_and_json_are_the_hosts_options`.
Proof: unit, `cargo nextest run -p temper-client-tools -E
'test(each_read_parses_into_its_call_and_prints_back)'` (under 20 ms):
each read's argv gives its call and prints back unchanged. Nothing calls
`table::parse` yet: the next plan's host passes it argv; temper 03's
root takes its calls.
Don't: resolve a project's name; read stdin or a file (the host does);
write the `temper ctl: ` prefix.
Size: at most 300 lines.
Removes: nothing.

### 02.3 The acts, adopt's row and `wait`
Scope: jig's rows for 5.3's acts but adopt, with `--key NAME` by each
row's `Keyed` (7); temper's row for `adopt` (5.3); jig's row for `wait`
(5.4), with its subject, `--since`, `--until` checked against the
subject, and `--timeout`; the bounds test over the rows' repeats.
Needs: the table, a call and argv (temper 02.2).
Files: new `src/jig/subject.rs`, `src/adopt.rs` and `src/tests/subject.rs`;
`src/jig/{table,call,rows,argv}.rs`, `src/jig.rs`, `src/lib.rs`, `src/table.rs`, `src/tests.rs`,
`src/tests/{fixtures,table,limits}.rs`.
Shape:
- A `Required` row without `--key` is `Usage::Key`; an `Optional` row's
  key goes in `call.key`; on a `Never` row (reads, `wait`, the host's
  tools) `--key` is unknown. `Name::new` refuses a bad name.
- decide's two forms, tried in order: `proposal` (`tN/K` or `pN/K`) with an `outcome`
  of `accept|reject|pass-up`, or `task` and `--hold R` with one of
  `release|leave|pass-up`; each may end with a `reason` text.
- `close N [--move D]... [--reason TEXT]`: at most 8 moves, the
  preset's `delegates`; the optional reason defaults to empty in the
  runner. `cancel` takes `--grace D`, then REASON.
- `adopt::ROW` is keyed: `--project P`, `--forge F` within `u16`, the
  texts `--owner`, `--name`, `--landing` and `--prefix`, `--role
  owned|fork|context`, the switches `--no-home` and `--no-ci`, and
  `--check G` at most 8 times, the forge's `checks`. An absent flag stays
  absent: the defaults are the form's.
- `wait`'s syntax is owned here: `argv::parse` builds only watch subjects
  (Choices), `subject::condition` parses `--until`, and `takes` checks it;
  temper 04 evaluates the checked value.
- `--since` takes 16 lower-case hex digits into `[u8; 8]`;
  `--timeout` defaults to `limits::WAIT`, is capped by `WAIT_MOST`.
  `--since` and `--until` may be given together; `check` applies the
  same subject, condition and duration rules to temper 12's JSON calls.
Test shape: the acts' tables in `src/tests/table.rs`, wait's in
`src/tests/subject.rs`. The reference test:
```rust
const UNTIL: &[(&[&str], bool)] = &[
    (&["wait", "me", "--until", "waiting"], true),
    (&["wait", "me", "--until", "ended"], false),
    (&["wait", "task", "41", "--until", "phase:backing-off"], true),
    (&["wait", "tree", "41", "--until", "replied"], false),
    (&["wait", "tree", "41", "--until", "waiting-on-me"], false),
    (&["wait", "tree", "41", "--until", "ended"], true),
    (&["wait", "task", "41", "--until", "waiting-on-me"], true),
    (&["wait", "task", "41", "--until", "phase:done"], false),
    (&["wait", "task", "41", "--until", "phase:failed"], false),
    (&["wait", "task", "41", "--until", "phase:cancelled"], false),
    (&["wait", "task", "41", "--until", "phase:ended"], false),
    (&["wait", "task", "41", "--until", "ended"], true),
    (&["wait", "tasks", "--project", "shop", "--until", "ended"], false),
];
#[test]
fn a_condition_is_taken_only_where_its_subject_names_it() {
    for (words, taken) in UNTIL {
        assert_eq!(fixtures::parse(words).is_ok(), *taken, "{words:?}");
    }
}
```
The others, each under 20 ms:
`each_act_parses_into_its_call_and_prints_back`, `a_keyed_act_without_its_key_is_a_usage_error`,
`a_key_on_a_read_wait_or_host_tool_is_a_usage_error`, `close_takes_an_optional_reason`,
`decides_two_forms_parse_by_their_arguments` (both proposers, no bare `3/17`),
`adopts_flags_parse_and_its_absent_flags_stay_absent`,
`each_phase_word_names_one_of_jigs_phases_and_back` (14 words; only ten in `--until`),
`a_timeout_defaults_to_300_seconds_and_stops_at_3600`, `wait_tasks_takes_neither_a_cursor_nor_ended`,
`wait_inbox_takes_no_subject_flag` (`--project`, `--kind`, `--root`, `--after`),
`other_read_flags_are_not_watch_arguments`, `ended_phase_errors_name_until_ended`,
`since_and_until_parse_together`, `the_rows_repeats_are_within_the_client_limits`.
Proof: unit, `cargo nextest run -p temper-client-tools -E
'test(a_condition_is_taken_only_where_its_subject_names_it)'` (under 20
ms): each subject takes 5.4's conditions and no others. Nothing sends an
act yet: temper 03's root sends it under `key::engine`; temper 04 gives
the conditions their meaning.
Don't: run the form's checks, or touch
`crates/temper-client-domain/src/forge.rs` or `forge/form.rs` (temper
11's adopt increment, a hotspot); decide whether a condition is met;
compute a digest.
Size: at most 300 lines.
Removes: nothing.

## Interface sketches

temper 03 owns how a call runs and how its result renders; the next
plan's host, argv's source, stdin, the `temper ctl: ` line and the exit
status; temper 04, what a condition means and the digest's value. temper 11
runs adopt's form checks; temper 12 needs only **the tools' table**, using
these records, `argv::check` and `src/tests/fixtures.rs::EVERY_TOOL`.

02.3 owns `jig::subject::{Subject, Condition, condition, takes, phase,
phase_word}`. temper 04 imports them; it adds no condition parser or phase
words. It evaluates `ended` on a delivery or the subject's watch ending,
then reads an ended task to say how it ended (5.4). It takes
`limits::{WAIT, WAIT_MOST}` through `jig::Limits`' `wait` and
`wait_most`: 300 s and 3600 s; no `WAIT_TIMEOUT` or `WAIT_TIMEOUT_MOST`.

temper 03 supplies `Owner::Runner` = 3: a call's number in each key's and
handle's high 32 bits, allocated above the highest call among Start's kept
keys (client/tools.md, 4). A kept line prints no name and no key: `kept: ` + the
operation + the outcome; its JSON has exactly `operation` and `outcome`
(6.1, 6.2). Unconfirmed and a client key conflict are tools bugs, exit 1,
reported as findings by temper 12's guide; client-own answers exit 1 (6.3).

```rust
// crates/temper-client-tools/src/jig/table.rs
/// A row, const data (5.1); `forms` is one form, or decide's two, tried in order.
pub struct Tool { pub name: &'static [u8], pub line: &'static [u8], pub result: &'static [u8],
    pub forms: &'static [&'static [Argument]], pub keyed: Keyed, pub area: u8 }
/// `name` in JSON (snake case, 6.2); `shown` in a usage line (`N`, `A.T`).
pub struct Argument { pub name: &'static [u8], pub shown: &'static [u8], pub place: Place,
    pub kind: Kind, pub occurs: Occurs, pub line: &'static [u8] }
pub enum Place { Positional, Flag, Switch }
pub enum Occurs { Required, Optional, Repeated { most: u32 } } // `most` within its client limit
pub enum Keyed { Never, Optional, Required }
pub enum Kind { Number, Count, Word { set: &'static [&'static [u8]] }, Text, File, Duration,
    Project, Turn, Proposal, Cursor, Digest, Group, Order, Condition, Subject, Tool }
/// jig's part's bounds, `Copy`, from the host's preset.
pub struct Limits { pub rows: u32, pub ask_bytes: u64, pub text: u32, pub part: u32,
    pub wait: Duration, pub wait_most: Duration }
// crates/temper-client-tools/src/jig/call.rs
/// A call (5.1): its row and form, one slot per argument of the form, its key's name.
pub struct Call { pub tool: u8, pub form: u8, pub values: Box<[Option<Values>]>, pub key: Option<Name> }
pub enum Values { One(Value), Many(Box<[Value]>) }
pub enum Value { Number(u64), Word(u8), Text(Box<[u8]>), Stdin, File(Box<[u8]>), Duration(Duration),
    Switch, Project(Project), Turn(TurnName), Proposal(ProposalKey), Cursor(Box<[u8]>),
    Digest([u8; 8]), Group(InboxGroup), Order(QueryOrder), Condition(Condition),
    Subject(Subject), Tool(u8) }
pub enum Project { Number(u32), Name(Box<[u8]>) }
/// What argv gives the host: a call, or where `--call` reads one.
pub enum Parsed { Call { call: Call, json: bool }, File { file: Source, json: bool } }
pub enum Source { Path(Box<[u8]>), Stdin }
// crates/temper-client-tools/src/jig/argv.rs
/// A usage error (6.3: exit 1), naming its word or argument; the host writes its line.
pub enum Usage { Tool, Unknown { at: u16 }, Extra { at: u16 }, Missing { argument: u8 },
    Twice { argument: u8 }, Kind { argument: u8 }, Rows { argument: u8 }, Long { argument: u8 },
    Many { argument: u8 }, Stdin, Key, Condition, Json }
pub fn parse(table: &[&Tool], words: &[&[u8]], limits: &Limits) -> Result<Parsed, Usage>;
/// The table's checks for argv and JSON; a failure ends parsing, exit 1.
pub fn check(table: &[&Tool], call: &Call, limits: &Limits) -> Result<(), Usage>;
/// Why `back` wrote no argv: the list's bound, or a call no argv gives (decisions.md, J).
pub enum Unwritten { Overflow(Overflow), NoForm }
pub fn back(table: &[&Tool], call: &Call, out: &mut List<Box<[u8]>>) -> Result<(), Unwritten>;
// crates/temper-client-tools/src/jig/subject.rs
pub enum Subject { Me, Inbox, Task(u64), Tree(u64), Run(u64), Tasks(Box<Call>) }
pub enum Condition { Waiting, Updates, Ended, WaitingOnMe, Held, Replied, Phase(Phase) }
/// Parse 5.4's condition for its subject; terminal phases name `--until ended`.
pub fn condition(subject: &Subject, word: &[u8]) -> Result<Condition, Usage>;
/// tree/run: `Ended` only; `WaitingOnMe`: `Task` only; `Phase`: ten live phases only.
pub fn takes(subject: &Subject, condition: &Condition) -> bool;
pub fn phase(word: &[u8]) -> Option<Phase>; pub fn phase_word(phase: Phase) -> &'static [u8];
// crates/temper-client-tools/src/jig/key.rs
/// 1 to 64 bytes of `a-z`, `0-9` and `-`; `Name::new(&[u8]) -> Option<Name>`, `bytes()`.
pub struct Name { bytes: Box<[u8]> }
pub fn named(salt: &[u8], name: &Name) -> skein_lib::RequestKey;
// crates/temper-client-tools/src/jig.rs: re-export table::Limits as jig::Limits.
// crates/temper-client-tools/src/{key,limits,table}.rs: temper's part.
pub const TOOLS: &[u8] = b"temper-tools"; // key.rs
/// The engine key jig's named send carries (jig 02); temper 03 sends under it.
pub fn engine(name: &Name) -> skein_lib::RequestKey;
pub fn client() -> temper_client_domain::Limits; pub fn tools() -> jig::Limits;
pub const TOOLS: &[&Tool]; // table.rs: jig's rows, then adopt::ROW
pub fn parse(words: &[&[u8]]) -> Result<Parsed, Usage>;
```

## Questions to raise when reached

None.

## Done when

Each in `~/src/rust/temper`, after `git fetch`:

- `git show origin/main:Cargo.toml | grep -qx '    "crates/temper-client-tools",'`
- `git show origin/main:crates/temper-client-tools/Cargo.toml | grep -qx 'skein-json.workspace = true'`
- `test -z "$(git grep -n -i temper origin/main -- crates/temper-client-tools/src/jig.rs crates/temper-client-tools/src/jig)"`
- `test -z "$(git grep -h -o -E 'crate::[a-z_]+' origin/main -- crates/temper-client-tools/src/jig.rs crates/temper-client-tools/src/jig | grep -vx 'crate::jig')"`
- `git show origin/main:crates/temper-client-tools/src/limits.rs | grep -c -E 'pub const (ROWS: u32 = 32|TEXT: u32 = 32_768|PART: u32 = 16_384|DOCUMENT: u64 = 1_048_576);' | grep -qx 4`
- `git grep -q -F 'b"temper-tools"' origin/main -- crates/temper-client-tools/src`
- `! for t in me inbox tasks task chat activity turn run message close adopt answer decide release stop cancel wait sign-in sign-out pending help; do git grep -q -F "name: b\"$t\"" origin/main -- crates/temper-client-tools/src/jig/rows.rs crates/temper-client-tools/src/adopt.rs || echo "missing $t"; done | grep -q missing`
