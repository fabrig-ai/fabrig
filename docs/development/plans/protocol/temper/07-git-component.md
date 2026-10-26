# Session 07: the git component and its machine world

The worker's protocol layer gets the component that runs the checkout's io:
`crates/temper-worker-protocol-git`, step code, which takes each `git::Op`
of `temper-worker-domain-checkout` with its deadline and its identity's
value, runs it as git.md's fixed sequence of git invocations (each a
process in a group of its own, spawned through io's records) or as file
operations (kernel operations it submits itself, decision 7), reads git's
machine output and stderr tail, and answers exactly one `Done`. The
session lands the crate's limits and builders first (07.1), the parsers
and the classifier against real git's transcripts (07.2, which opens the
machine world's crate for them), the component in two halves (07.3a: the
invocations, deadlines, cancel; 07.3b: the root's layout, Make, Commit's
scan, the sweep), and the git component's machine world (07.4), whose
scripted io 09.2 and 09.3 reuse.

## Read first

- `git.md` (all); `performance.md`, section 3; `channel.md`, section 6;
  `credentials.md`, section 4 ("In git").
- skein's `docs/design/io.md`, sections 5 and 6, and `lib.md`, section 7
  (a stream's contract: scans, `End`), at the pin; skein-io's
  `crates/skein-io/src/kernel.rs:361-518` (`Op`: `Open`, `Read`, `Rename
  { from_dir, from, to_dir, to }`, `Remove`, `MakeDirectory`, `List`,
  `Close`), `:519-540` (`Spawn`, `Pipe`), `:660-700` (`Entry`,
  `LONGEST_NAME`), `:781-900` (`Error`); `records.rs` (`Request::Spawn`,
  `Signal`, `Stream`, `Close`, `Abort`; `Event::Spawned`, `Exited`,
  `Stream`, `Failed`, `Closed`); `limits.rs:47-70` (`largest_read`,
  `largest_room`).
- `decisions.md`, 7; `code-organization.md`, sections 2, 3, 5 and 7.
- `findings/temper-worker.md`, sections 1 and 6, Findings 3, 4, 5, 10, 12.
- The code: `crates/temper-worker-domain-checkout/src/git.rs` (the
  vocabulary), `boundary.rs:79-87` (`Request::Io`, `Request::Cancel`),
  `limits.rs:13-38`; smith's
  `crates/smith-host-protocol/src/process.rs:122-145, 268-394, 473-505`
  at the pin (a child's spawn, pipes, exit, close and stderr tail: the
  precedent); jig's `testing/jig-test-service/src/worker.rs:571-592` at
  the pin (a kernel operation submitted under the layer's own token).
- 05.4's `tools/git-transcripts/transcripts/` (its case directories and
  masking), from 07.2 on.

## What the design asks

- **One component, one terminal** (git.md, 1): "The worker's root asks
  for a `git::Op` with a deadline (`Request::Io`) and may cancel it
  (`Request::CancelIo`); the git component runs it and answers with
  exactly one `Event::Done`." Its `owner` is echoed (section 2).
- **Fixed sequences, no retry** (section 1): "at most four git
  invocations, each its own process in a process group of its own, all
  under the operation's one deadline. Which invocation comes next
  depends only on what the last one said, never on a retry". Section 4
  gives each operation's invocations and terminals.
- **git told exactly what to do** (section 3): the program by its
  absolute path; thirteen `-c` settings, in order; the working directory
  the spawn's, beneath the worker's root, never `-C`; operands after
  `--`; ids "exactly 40 or 64 lowercase hex digits, anything else is
  `Broken`", a SHA-1 id as "its 20 bytes followed by twelve zero bytes";
  "At the worker's start the component runs `git version` once ... and
  refuses the start, naming the program, when it fails or names a
  version below 2.41".
- **The environment, whole** (section 5): `HOME`, `LC_ALL=C`, `TZ=UTC`,
  `GIT_CONFIG_NOSYSTEM`, `GIT_CONFIG_GLOBAL`, `GIT_TERMINAL_PROMPT`,
  `GIT_ASKPASS`, the committer on every invocation, the author for
  Commit, `GIT_CONFIG_COUNT`/`KEY_0`/`VALUE_0` with `Authorization:
  Basic` and the base64 of `<login>:<token>` for Clone, Fetch, Create,
  Push; "No `PATH`"; "no credential in an argument, a URL, a
  repository's configuration or a file".
- **Output** (section 6): stdout "up to `stdout_bytes` ... Past it, the
  bytes are read and dropped, counted ... an operation whose output
  passed the cap ends `Broken`"; stderr "its last `stderr_bytes`, with
  the count of bytes dropped"; the order: deadline or cancel, then a
  failed spawn or a signal (`Broken`), then exit status and stdout, then
  the stderr table, last line first; "A failure after git ran ends
  `FailedWithOutput`"; "one before any invocation ran ... ends `Failed`".
- **Processes** (section 7): "An invocation ends once git has exited,
  stdout and stderr have reached their ends, and io has closed the
  child"; a deadline or cancel signals `Kill` to the group and closes
  it, "no further invocation starts"; "An operation whose last
  invocation had already exited, its output read, when the cancel came
  ends with its own terminal".
- **Make and the sweep** (4.1, 2): Make renames `workspaces/<token>` to
  `trash/<n>` and makes the directory afresh, 0o700; the sweep empties
  `trash` "one file operation at a time"; at every start the component
  "renames `workspaces` and `home` into `trash` if they exist, and makes
  them afresh, private (0o700), with `trash` if it is absent".
- **Limits** (section 8): `operations`, `stdout_bytes`, `stderr_bytes`,
  `read_bytes`, `sweep_entries`, `sweep_bytes`; "The start is refused,
  naming the limit, when a rule does not hold"; `worst_case`; "Most
  outputs per entry point: a constant the component exports".
- **Testing** (section 9): step tests of the builders, the parsers and
  the classifier "each against transcripts and against their
  mutations", the scan's rule, the sweep's step; the machine world
  `tests/worker/protocol-git` with its list of stimuli and its referee.

## Choices this plan makes

- **File operations are kernel operations** (decision 7). The component
  submits `kernel::Submit { op, kind }` with tokens of its own prefix
  (`(6 << 60) + ((i + 1) << 48)`, code-organization.md, 7) for `Open`,
  `Read`, `Rename`, `Remove`, `MakeDirectory`, `List` and `Close`, and
  takes their `kernel::Complete` back from its parent; it spawns git
  through io's records (`io::Request::Spawn`, `Stream`, `Signal`,
  `Close`). After the start it holds two descriptors, `trash` and
  `workspaces`, opened with `OpenHow::DirectoryNoFollow`, and closes them
  at its end.
- **Its vocabulary mirrors the root's:** `Request::Io { owner, op,
  deadline, identity }`, `Request::CancelIo { owner }`, `Event::Done {
  owner, done }`; and `start` with its terminals `Event::Started` and
  `Event::Unusable`, `Request::Shutdown` for the sweep. A cancel ends as
  the checkout's `Failed`/`FailedWithOutput { fault: Cancelled }`; there
  is no `Done::Cancelled` (findings/temper-worker.md, Finding 10).
- **The identity's value is the component's `Identity`** (token as
  skein-lib's `Secret`, login, name, email). The component links no wire
  crate (code-organization.md, 3); the worker's routing makes it from
  03.1's decoded record (09.1a's `translate::value`). It is the value an
  environment is built from, not a second form of the record.
- **A missing value fails at the entrance:** an op that names an
  identity and comes with `identity: None` ends `Failed { fault: Broken
  }` with no spawn (git.md, 2). So does an op past `operations` in
  flight, or an `owner` already in flight.
- **stdin is a pipe for `commit-tree` only;** other invocations get io's
  unassigned descriptor, `/dev/null`, which git reads as an empty stdin,
  as a pipe written with nothing and closed would be. One entity less
  per invocation.
- **Pipes are read by scans:** stdout by `Read::Scan` to the
  invocation's delimiter (LF; NUL after `-z`), each at most io's
  `largest_read`, appended to a capped buffer; stderr by `Read::Line`
  into a ring of `stderr_bytes`. Bytes after the last delimiter at the
  stream's end are never delivered (lib.md, 7): an output whose last
  record lacks its delimiter reads as malformed (`Broken`).
- **The sequence is a pure function** (`sequence::first`,
  `sequence::after`), step-tested without io; the component's process
  machine calls it after each invocation has closed.
- **`trash/<n>` starts at a drawn serial:** `Settings::serial`, drawn by
  the worker's service from its seed at each start, counts up from there
  in decimal (git.md, 4.1: "counted from a number the component draws
  from its seed at each start"); a Make whose rename meets a name anyway
  ends `Broken`, which the domain retries as transient: the `Rename` of
  a directory onto a `trash/<n>` that holds anything fails
  (`ENOTEMPTY`), and one onto an empty directory replaces it, losing
  nothing; Make takes git.md's two operations, with no probe before the
  rename (ruling 114).
- **Commit's scan** opens `workspaces/<token>/<repository>/<path>`
  beneath the root with `OpenHow::ReadNoFollow` and reads it at offsets in
  pieces of `read_bytes` until a read returns 0. `NotFound`,
  `TooManyLinks` (a link, refused) or `NotAFile` at the open, and
  `IsADirectory` at the first read, mean no marker ("A path that is gone,
  a directory or a link holds no marker"); any other error ends the
  operation `Broken`.
- **The sweep** opens `trash` afresh for each pass (`Open`
  `DirectoryNoFollow`, `List` one page, `Close`), takes the first entry,
  and does as git.md, 4.1 says, one file operation at a time; a failure
  ends the pass, counted, and the next pass starts at the next Make or
  the next start (git.md, 4.1).
- **`sweep_bytes` is at least `kernel::LONGEST_NAME` (255):** io's `List`
  demands it (kernel.rs:468-475), and git.md, 8 says so ("`sweep_bytes`
  at least 255, the longest name io lists"); the check refuses a smaller
  value naming `git_component.sweep_bytes`.
- **The two containers of performance.md, 3** are modules of the
  component: `capped` (stdout) and `ring` (`ByteRing`, the stderr tail).
- **The version check comes first in the start,** then the layout
  (07.3b); `Started` after both.
- **07.3 is split** into 07.3a (about 600 lines: the operations' process
  machine) and 07.3b (about 400: the file operations), as
  code-organization.md, 8 asks of an increment above about 600 lines.
- **The machine world's crate opens in 07.2,** for the transcripts: step
  code may not use `env!` (`clippy.toml`'s `core::env`), and
  code-organization.md, 5 reads transcripts by a path from
  `env!("CARGO_MANIFEST_DIR")`, so the transcript tests live in the world
  (ordinary Rust), and the step tests use literals.
- **The machine world plays io directly,** with no simulator: a driver
  answers the component's io requests and kernel submissions from
  scripts made of the transcripts, and is a library module (`io`) that
  09's tests reuse.

## Rules that bind it

- "**`temper-worker-protocol-git`** links `skein-lib`, `skein-io` and
  `temper-worker-domain-checkout`. It decodes no channel record and reads
  no configuration (git.md, section 2)." (code-organization.md, 3)
- "No increment of this plan: ... adds a retry below the domain
  (protocol.md, section 5); puts a secret's value in a domain type, a
  fact, a store record, a log or a `Debug` output but skein-lib's
  `Secret`'s length" (code-organization.md, 6).
- "**Transcripts are read as files** from the tools' directories by path
  from `env!("CARGO_MANIFEST_DIR")`, raw bytes, never re-recorded by a
  test." And the share: "`tests/worker/protocol-git` 0.5 s and 3 s"
  (code-organization.md, 5).

## Increments

### 07.1 The crate, its limits, the builders

Scope: the crate `crates/temper-worker-protocol-git` (step code, the
workspace's lints, `#![cfg_attr(not(test), no_std)]`,
`#![forbid(unsafe_code)]`), linking `skein-lib`, `skein-io` and
`temper-worker-domain-checkout`. `limits.rs`: `Limits`, `Unfit`, `check`
(git.md, 8's rules, and `sweep_bytes` at least `kernel::LONGEST_NAME`),
`largest_output`, `worst_case` over git.md, 8's payloads (per operation
in flight: argument vector, environment, message, `stdout_bytes`,
`stderr_bytes`, `read_bytes`, `conflicts × path_bytes`; one sweep page);
07.3a and 07.3b add the containers they make. `boundary.rs`: `Identity`,
`Settings`, `Request`, `Event`, `Unusable`, `Version`, `Outputs`.
`invocation.rs`: `SETTINGS`, `Command`, `Source`, `arguments`,
`environment`, `message`, `hex`, `object`, `place` (git.md, sections 3
to 5). The checkout's stale doc comments (findings/temper-worker.md,
Finding 12): `Op::Make`, `Op::CheckOut`, `Op::Merge`, `Fault::Missing`,
`Fault::Unreachable` and `Limits.name_bytes` say what git.md says, doc
lines only.
Needs: nothing.
Files: `crates/temper-worker-protocol-git/{Cargo.toml, src/{lib.rs,
limits.rs, boundary.rs, invocation.rs, tests.rs, tests/fixtures.rs,
tests/invocation.rs}}`; the workspace `Cargo.toml` (member and
`[workspace.dependencies]` entry); `crates/temper-worker-domain-checkout/src/{git.rs,
limits.rs}` (doc comments only). Shared hotspot: the workspace
`Cargo.toml`. skein-io is in `Cargo.lock` already: no pin moves.
Shape: as the interface sketch's `limits`, `boundary` and `invocation`
sections. `arguments` returns the vector after argv[0] (io puts the
program's path there, `kernel.rs:520-521`): `-c`, each of `SETTINGS`,
then the command. `environment(home, kind, identity)`: `None` kind for
the version check; the committer is the identity's author whenever the
operation names one with a value, else `temper` and
`temper@temper.invalid`; author variables for `Kind::Commit` only;
`GIT_CONFIG_*` for `Clone`, `Fetch`, `Create`, `Push` only; no `PATH`.
`hex` writes 40 digits when the last twelve bytes are zero, else 64;
`object` takes exactly 40 or 64 lowercase hex digits. `place` is
`workspaces/<token in decimal>/<repository>`; Clone's directory is
`workspaces/<token in decimal>`.
Test shape: step tests in `src/tests/invocation.rs`; fixtures in
`src/tests/fixtures.rs` (new): `checkout()` (a `checkout::Limits`
literal: `workspaces: 2, repositories: 2, name_bytes: 64, message_bytes:
256, conflicts: 4, path_bytes: 64`, timeouts 30 s and 5 s, `facts: 8`),
`limits()` (a `Limits` that `check` passes against it), `identity()`
(token `b"tok-1"`, login `b"temper-bot"`, name `b"Temper Bot"`, email
`b"bot@example.test"`), `HOME = b"/srv/worker/home"`, `REMOTE =
b"https://forge.example.test/acme/app.git"`, `COMMIT` (`[0xab; 20]` then
twelve zero bytes), `COMMIT_256` (`[0xcd; 32]`). Tests:
`each_command_s_arguments_are_git_md_s_vector` (a table, one row per
`Command` variant and per `Source`, each row's expected vector a
literal), `a_remote_operation_carries_the_token_only_in_its_header`,
`a_commit_is_authored_and_committed_as_the_identity`,
`an_operation_naming_no_identity_commits_as_the_placeholder`,
`the_message_is_title_blank_line_body_and_newline` (rows: body empty or
not), `ids_of_forty_and_sixty_four_digits_convert_both_ways` (rows: 40,
64, 39 digits, uppercase, a non-hex byte),
`limits_breaking_git_md_s_rules_are_refused_naming_the_limit` (one row
per `Unfit` but `Memory`), `the_identity_s_debug_shows_no_token_byte`.
Each under 20 ms. Reference test, in full:

```rust
#[test]
fn a_remote_operation_carries_the_token_only_in_its_header() {
    let identity = fixtures::identity();
    let environment = invocation::environment(fixtures::HOME, Some(git::Kind::Push), Some(&identity));
    for expected in [
        &b"GIT_CONFIG_COUNT=1"[..],
        b"GIT_CONFIG_KEY_0=http.extraHeader",
        // base64 of b"temper-bot:tok-1"
        b"GIT_CONFIG_VALUE_0=Authorization: Basic dGVtcGVyLWJvdDp0b2stMQ==",
    ] {
        assert!(environment.contains(&Box::from(expected)), "{expected:?} is set");
    }
    let push = invocation::Command::Push {
        remote: Box::from(fixtures::REMOTE),
        commit: fixtures::COMMIT,
        branch: Box::from(&b"temper/7"[..]),
        expected: None,
    };
    for argument in invocation::arguments(&push).iter() {
        assert_eq!(skein_lib::bytes::find(argument, b"tok-1"), None, "no token in an argument");
    }
}
```

Proof: unit, `cargo nextest run -p temper-worker-protocol-git`,
run once: each command's vector is git.md's, the token rides only in
`GIT_CONFIG_VALUE_0`, ids convert both ways, and limits that break a
rule are refused naming it (the eight tests above); under 0.2 s. No
higher layer exists yet.
Don't: link a crate beyond the three; read configuration; put the token
in an argument, a URL or a `Debug` output; add `PATH`.
Size: at most 450 lines of production code.
Removes: nothing.

### 07.2 The parsers and the classifier

Scope: `output.rs` (git.md, sections 4 and 6): `fetched`, `status` and
`Status`, `paths`, `version`, `classify`, `Markers` (the scan's rule,
4.7). The machine world's crate `tests/worker/protocol-git`
(`temper-worker-protocol-git-world`, ordinary Rust, `tests/clippy.toml`)
opens with `src/transcripts.rs`: `read(case) -> Transcript { argv,
exit, stdout, stderr }`, raw bytes from
`env!("CARGO_MANIFEST_DIR")/../../../tools/git-transcripts/transcripts/<case>/`,
and `mutations(bytes) -> Vec<Box<[u8]>>` (each line dropped, each line
cut at its middle, one hex digit raised, the last delimiter dropped).
Needs: 05.4's transcripts, their case directories and masking.
Files: `crates/temper-worker-protocol-git/src/{lib.rs, output.rs,
tests/output.rs}`; `tests/worker/protocol-git/{Cargo.toml (its `[lints]`
copied from `tests/worker/domain/Cargo.toml`; `tests/clippy.toml` applies
by its directory), src/lib.rs, src/transcripts.rs, tests/transcripts.rs}`;
the workspace `Cargo.toml`.
Shared hotspot: the workspace `Cargo.toml`.
Shape: as the sketch's `output` section. `status` reads `--porcelain`'s
line for `refs/heads/<branch>` (`<flag>\t<from>:<to>\t<summary>`):
space, `*`, `=`, `!` with `[rejected] (non-fast-forward)`, `(fetch
first)`, `(stale info)`, `[remote rejected] (…declined)`; anything else
`Other`. `paths` refuses more than `conflicts` paths or one longer than
`path_bytes` (git.md, 4.6). `classify` reads the tail's lines that begin
`fatal: ` or `error: `, last to first; the first matching section 6's
table decides; a local `kind` (Make, CheckOut, Merge, Commit) is
`Broken`.
Test shape: step tests in `src/tests/output.rs`, literals only:
`a_marker_cut_across_reads_is_found`,
`markers_of_six_and_eight_characters_are_no_markers`,
`one_carriage_return_before_the_newline_is_dropped`,
`a_nul_list_past_the_checkout_s_limits_is_refused`,
`stderr_lines_decide_last_to_first_the_first_match_wins`; under 20 ms
each. World tests in `tests/worker/protocol-git/tests/transcripts.rs`:
`each_fetch_transcript_parses_to_its_fetched_commit`,
`each_push_transcript_s_status_line_is_its_git_md_row`,
`each_failure_transcript_classifies_as_section_six_says`,
`the_version_transcript_parses_as_at_least_two_forty_one`,
`mutated_outputs_parse_to_none_never_to_another_value`; each row a case
directory 05.4 landed, by its name (the names below are this plan's
guess: the landed directory's names win), each row's expected value a
literal (a masked id as the transcript writes it); under 0.1 s summed.
Reference tests, in full, one per test file:

```rust
// crates/temper-worker-protocol-git/src/tests/output.rs
#[test]
fn a_marker_cut_across_reads_is_found() {
    let rows: [(&[&[u8]], bool); 4] = [
        (&[b"one\n<<<<", b"<<< ours\ntwo\n"], true),
        (&[b"=====", b"==\r\n"], true),
        (&[b">>>>>>>", b" theirs"], true),
        (&[b"<<<<<<", b"<\n"], false),
    ];
    for (reads, marked) in rows {
        let mut markers = Markers::START;
        for read in reads {
            markers.feed(read);
        }
        assert_eq!(markers.finish(), marked, "{reads:?}");
    }
}
```

```rust
// tests/worker/protocol-git/tests/transcripts.rs
#[test]
fn each_failure_transcript_classifies_as_section_six_says() {
    let rows = [
        ("clone-missing-repository", git::Kind::Clone, git::Fault::Missing { missing: git::Missing::Repository }),
        ("fetch-missing-branch", git::Kind::Fetch, git::Fault::Missing { missing: git::Missing::Branch }),
        ("fetch-missing-commit", git::Kind::Fetch, git::Fault::Missing { missing: git::Missing::Commit }),
        ("fetch-wrong-token", git::Kind::Fetch, git::Fault::Refused),
        ("push-no-permission", git::Kind::Push, git::Fault::Refused),
        ("clone-refused-connection", git::Kind::Clone, git::Fault::Unreachable),
    ];
    for (case, kind, fault) in rows {
        let transcript = transcripts::read(case);
        assert_ne!(transcript.exit, 0, "{case}: git failed");
        assert_eq!(output::classify(&transcript.stderr, kind), fault, "{case}");
    }
}
```

Proof: unit and world, `cargo nextest run -p
temper-worker-protocol-git -p temper-worker-protocol-git-world`, run
once: real git 2.53's outputs parse to their values, its failures
classify as section 6's table says, and no mutation parses to another
value (the ten tests above); under 0.3 s.
Don't: re-record or rewrite a transcript; parse stderr outside
`classify`; read a transcript from a step test.
Size: at most 400 lines of production code (the world's `transcripts.rs`
is test code).
Removes: nothing.

### 07.3a The component: invocations, deadlines, cancel

Scope: `component.rs` (`Component`: `new`, `start`'s version check,
`down`, `up`, `fire`, `abort`, `accepts`, `owns`, `next_deadline`,
`is_due`, `is_empty`, `operations`, `reclaim`), `operation.rs` (an
operation in flight: its `op`, owner, deadline, identity, the command
running, its child and pipe tokens, which ends it has seen, its stdout
`Capped` and stderr `ByteRing`, cancelled or timed out), `sequence.rs`
(`Next`, `first`, `after`, git.md, section 4), `capped.rs`, `ring.rs`
(performance.md, 3), `MAX_OUT`; `worst_case` adds the operations' table.
An invocation: `io::Request::Spawn` (program `Settings::git`,
`arguments`, `environment`, root `Settings::root`, directory `place`,
pipes 1 and 2 `Out`, and 0 `In` for `CommitTree`); on `Spawned`, a scan
demand on stdout and a line demand on stderr, and for `CommitTree` the
message sent in pieces of at most io's `largest_room`, then `Finish`;
`Exited` kept; each pipe closed after its `End`, the child after
`Exited` and both ends; the invocation ends when each `Closed` has come;
then `after` decides. A deadline (`fire`) or a cancel: `Signal { child,
Kill, Group }`, then `Close` of the child and its pipes; `TimedOut` or
`Cancelled` once the child's `Closed` has come; a cancel after the last
invocation closed, or with `after` already at `End`, ends with that
outcome. `Failed` for a spawn: `Broken` (`Failed` for the first
invocation, `FailedWithOutput` with an empty diagnostic after). Until
07.3b, `Make` and a `Commit` with `merging: Some(_)` end `Failed { fault:
Broken }` at once.
Needs: nothing beyond 07.2.
Files: `crates/temper-worker-protocol-git/src/{lib.rs, limits.rs,
component.rs, operation.rs, sequence.rs, capped.rs, ring.rs,
tests/component.rs, tests/sequence.rs, tests/fixtures.rs}`.
Shape: as the sketch's `Component`, `Outputs`, `MAX_OUT`, `sequence`,
`capped` and `ring` sections. Owners: io owner tokens and kernel
operation tokens are `Settings::prefix` plus a serial; `accepts` also
knows the child and pipe tokens `Spawned` named.
Test shape: step tests; fixtures gain `Below` (pops what the component
asked of io and answers it: `spawned(owner) -> (child, pipes)`,
`stdout(pipe, bytes)`, `end(pipe)`, `exited(child, code)`,
`closed(tokens)`), `Started` (a component past its version check),
`commit_op(merging)`, `push_op(expected)`, `PARENT`, `EXPECTED`.
Tests in `src/tests/sequence.rs`:
`each_operation_s_next_invocation_is_git_md_s` (a table over `after`,
one row per operation and exit: Fetch's 0 → `Fetched`; Create's
statuses; Merge's 0 and 1 with and without a list; Commit's `DiffIndex`
0 → `Unchanged`, 1 → `WriteTree`; Push's `IsAncestor` 1 → `Rejected`).
Tests in `src/tests/component.rs`:
`a_fetch_runs_one_invocation_and_ends_fetched`,
`a_deadline_kills_the_group_and_ends_timed_out_after_the_close`,
`a_cancel_after_the_last_exit_ends_with_the_operation_s_outcome`,
`a_cancel_between_invocations_starts_no_further_one`,
`stdout_past_its_cap_is_read_dropped_and_ends_broken`,
`an_operation_whose_identity_has_no_value_fails_broken_unspawned`,
`a_git_older_than_two_forty_one_makes_the_start_unusable`. Under 20 ms
each. Reference tests, in full, one per test file:

```rust
// crates/temper-worker-protocol-git/src/tests/sequence.rs
#[test]
fn each_operation_s_next_invocation_is_git_md_s() {
    let commit = fixtures::commit_op(None);
    let push = fixtures::push_op(Some(fixtures::EXPECTED));
    let diff = invocation::Command::DiffIndex { parent: fixtures::PARENT };
    let ancestry = invocation::Command::IsAncestor { expected: fixtures::EXPECTED, commit: fixtures::COMMIT };
    let rows = [
        (&commit, &diff, 0, Next::End(git::Done::Unchanged)),
        (&commit, &diff, 1, Next::Run(invocation::Command::WriteTree)),
        (&push, &ancestry, 1, Next::End(git::Done::Rejected)),
    ];
    for (op, ran, code, next) in rows {
        assert_eq!(sequence::after(op, ran, code, b"", b"", 0, &fixtures::checkout()), next, "{ran:?} exit {code}");
    }
}
```

```rust
// crates/temper-worker-protocol-git/src/tests/component.rs
#[test]
fn a_deadline_kills_the_group_and_ends_timed_out_after_the_close() {
    let mut component = fixtures::Started::new();
    component.down(fixtures::fetch(fixtures::OWNER, fixtures::DEADLINE));
    let (child, pipes) = component.spawned();
    component.fire(fixtures::DEADLINE);
    assert_eq!(component.signals(), [(child, kernel::Signal::Kill, kernel::Target::Group)]);
    assert_eq!(component.done(), None, "nothing ends before the child has closed");
    component.closed(child, &pipes);
    assert_eq!(
        component.done(),
        Some((fixtures::OWNER, git::Done::FailedWithOutput {
            fault: git::Fault::TimedOut,
            diagnostic: git::PushDiagnostic::empty(),
        })),
    );
}
```

Proof: unit, `cargo nextest run -p temper-worker-protocol-git`,
run once: a fetch ends fetched, a deadline and a cancel end as git.md, 7
says and only after the close, a cap breached ends `Broken`, and a git
below 2.41 refuses the start (the eight tests above); under 0.3 s. The
machine world comes at 07.4.
Don't: retry an invocation; start an invocation after a deadline or a
cancel; end an operation before its child's `Closed`; hold the
identity's value past the operation's end.
Size: at most 600 lines of production code.
Removes: nothing.

### 07.3b The component: the root's layout, Make, Commit's scan, the sweep

Scope: after the version check, the start's layout (git.md, 2): the
start's renames and directories as kernel operations, one at a time:
`MakeDirectory` `trash` (0o700; `Exists` is fine), `Open` `trash`,
`Rename` `workspaces` and `home` into `trash/<n>` (`NotFound` is nothing
to move), `MakeDirectory` `workspaces` and `home` (0o700), `Open`
`workspaces`; another failure is `Unusable { why: Layout { error } }`.
Make (4.1): `Rename { from_dir: workspaces, from: <token>, to_dir:
trash, to: <n> }` (`NotFound`: nothing to move), then `MakeDirectory {
dir: workspaces, name: <token>, mode: 0o700 }`; `Succeeded`, or `Failed {
fault: Broken }`. Commit's scan (4.7, as Choices say) between `Unmerged`
and `Add`, its `Markers` per path; paths with markers end `Conflicted`
in the list's order, nothing staged. The sweep (4.1) while `trash` is
not empty and no `Shutdown` came. At the end (`Shutdown` with nothing in
flight, or `abort`), `Close` of the two descriptors.
Needs: nothing beyond 07.3a.
Files: `crates/temper-worker-protocol-git/src/{component.rs, files.rs
(new: layout, Make, scan, sweep), limits.rs, tests/files.rs,
tests/fixtures.rs}`.
Shape: no new public item; `Component::start` now ends `Started` after
the layout; `operations()` counts one kernel operation per operation in
flight and one for the sweep.
Test shape: step tests in `src/tests/files.rs`; fixtures gain
`Started::laid_out()` (past the layout, its descriptors `TRASH` and
`WORKSPACES`, `SERIAL = 1000`), `file_operation() -> kernel::Op`,
`complete_ok()`, `complete_err(kernel::Error)`, `listing(&[(&[u8],
kernel::Kind)])`. Tests:
`the_start_moves_workspaces_and_home_into_trash_and_makes_them_afresh`,
`make_renames_the_old_workspace_aside_and_makes_it_empty`,
`a_merging_commit_with_markers_left_ends_conflicted_staging_nothing`,
`a_conflicted_path_gone_a_directory_or_a_link_holds_no_marker` (rows:
`NotFound`, `IsADirectory` at the read, `TooManyLinks`),
`the_sweep_removes_files_and_moves_subdirectories_up_into_trash`,
`no_sweep_pass_starts_after_shutdown`. Under 20 ms each. Reference test,
in full:

```rust
#[test]
fn make_renames_the_old_workspace_aside_and_makes_it_empty() {
    let mut component = fixtures::Started::laid_out();
    component.down(fixtures::make(fixtures::OWNER, fixtures::WORKSPACE));
    assert_eq!(
        component.file_operation(),
        kernel::Op::Rename {
            from_dir: fixtures::WORKSPACES,
            from: Box::from(&b"7"[..]),
            to_dir: fixtures::TRASH,
            to: Box::from(&b"1000"[..]),
        },
    );
    component.complete_ok();
    assert_eq!(
        component.file_operation(),
        kernel::Op::MakeDirectory { dir: fixtures::WORKSPACES, name: Box::from(&b"7"[..]), mode: 0o700 },
    );
    component.complete_ok();
    assert_eq!(component.done(), Some((fixtures::OWNER, git::Done::Succeeded)));
}
```

Proof: unit, `cargo nextest run -p temper-worker-protocol-git`,
run once: the start's layout, Make, the scan's outcomes and the sweep's
steps are git.md's (the six tests above); under 0.3 s.
Don't: follow a link; remove a tree in one operation; sweep after
`Shutdown`; run a second file operation of one operation, or of the
sweep, before the first completed.
Size: at most 400 lines of production code.
Removes: 07.3a's `Failed { fault: Broken }` answers to `Make` and to a
`Commit` with a second parent.

### 07.4 The git component's machine world

Scope: `tests/worker/protocol-git` grows into git.md, 9's machine world:
`src/io.rs` (the scripted io below, a public module: answers spawns
with a transcript's stdout, stderr and exit, cut into reads at drawn
places, an exit before or after the pipes' ends, an exit by a signal, a
spawn refused; kernel file operations over an in-memory tree, with
failures injected; directories listed in pages of two), `src/world.rs`
(scripted operations above: the root's `Request::Io` and
`Request::CancelIo` at scripted times, deadlines at each invocation),
`src/referee.rs` (git.md, 9: "each operation ends exactly once, with the
outcome its scripted outputs imply; no invocation starts after a
deadline or a cancel; every child is closed before its operation's
terminal; no credential in an argument; memory within `worst_case`"),
`src/fixtures.rs` (scenarios by name: each operation on its transcript
cases, a missing identity's value, file operations failing, deep and
wide trees for the sweep; `drawn(seed)`, operations and faults drawn from a seed).
Needs: nothing beyond 07.3b.
Files: `tests/worker/protocol-git/{Cargo.toml, src/{lib.rs, io.rs,
world.rs, referee.rs, fixtures.rs}, tests/protocol_git.rs,
tests/fuzzy_protocol_git.rs}`.
Shape: `temper_worker_protocol_git_world::{io::Below, world::{run,
Outcome}, fixtures}`; `Below` is the driver 09.2 and 09.3 reuse: it holds
a `Component`'s `Outputs`, answers them from a `Script`, and hands back
`Event`s.
Test shape: focused, `tests/protocol_git.rs`:
`each_operation_ends_once_with_its_transcript_s_outcome`,
`a_deadline_at_each_invocation_ends_timed_out_and_starts_no_more`,
`a_cancel_racing_the_last_exit_ends_with_whichever_won`,
`outputs_cut_at_random_and_past_each_cap_end_as_git_md_says`,
`the_sweep_empties_deep_and_wide_trees_in_pages_of_two`,
`memory_stays_within_the_worst_case_at_its_limits`; summed at most
0.4 s, each under 0.4 s, so that with 07.2's transcript tests (0.1 s)
the world keeps its 0.5 s share (code-organization.md, 5). Fuzzy,
`tests/fuzzy_protocol_git.rs`:
`drawn_operations_and_faults_keep_the_referee_s_rules`, seeds one after
another in one thread, at most 3 s. Reference tests, in full, one per
test file:

```rust
// tests/worker/protocol-git/tests/fuzzy_protocol_git.rs
#[test]
fn drawn_operations_and_faults_keep_the_referee_s_rules() {
    for seed in 1..=64 {
        let outcome = world::run(&fixtures::drawn(seed), seed);
        assert_eq!(outcome.verdict, Verdict::Passed, "seed {seed}");
    }
}
```

```rust
// tests/worker/protocol-git/tests/protocol_git.rs
#[test]
fn a_cancel_racing_the_last_exit_ends_with_whichever_won() {
    let rows = [(fixtures::Race::ExitFirst, None), (fixtures::Race::CancelFirst, Some(git::Fault::Cancelled))];
    for (race, fault) in rows {
        let outcome = world::run(&fixtures::cancel_race(race), fixtures::SEED);
        assert_eq!(outcome.verdict, Verdict::Passed, "{race:?}");
        assert_eq!(outcome.faults(), [(fixtures::OWNER, fault)], "{race:?}: one terminal");
    }
}
```

Proof: world, `cargo nextest run -p
temper-worker-protocol-git-world` and `cargo nextest run -p
temper-worker-protocol-git-world --profile fuzzy`, each run once: each
operation ends once with its transcript's outcome, deadlines and cancels
start nothing after them, and memory stays within `worst_case` (the six
focused tests, at most 0.4 s summed; the fuzzy test, at most 3 s).
Don't: spawn a thread; run real git; re-record a transcript.
Size: world code only; no production line.
Removes: nothing.

## Interface sketches

`crates/temper-worker-protocol-git` owns its vocabulary; the worker's
routing (09.2) owns the identities' table and hands one value per
operation. An operation ends at its `Done`; the start at `Started` or
`Unusable`; the component at `is_empty` after `Shutdown` or `abort`. The
sketch compiles as `plan-sketches`' `s07.rs`.

```rust
//! Sketches of session 07: `temper-worker-protocol-git`, the git component
//! (git.md). Signatures only; bodies are `unreachable!("sketch")`, but for
//! the marker scan (07.2), an algorithm that is easy to get wrong.
#![expect(unused_variables, reason = "a sketch's bodies are unreachable")]

use alloc::boxed::Box;
use skein_io::{self as io, kernel};
use skein_lib::{Queue, Secret, Time, Token};
use temper_worker_domain_checkout::{self as checkout, git};

// temper_worker_protocol_git::limits (07.1)

/// The git component's limits (git.md, section 8): set by temper's profile
/// `small`, overridable under `git_component.`, handed to `Component::new`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    /// Operations in flight at once: at least the checkout's `workspaces`.
    pub operations: u32,
    /// stdout kept per invocation: at least the largest output section 4
    /// reads under the checkout's limits (`largest_output`).
    pub stdout_bytes: u32,
    /// The stderr tail kept per invocation: at least `PushDiagnostic::CAPACITY`.
    pub stderr_bytes: u32,
    /// One read of Commit's scan: above 0.
    pub read_bytes: u32,
    /// Entries of one page of the sweep's listing: above 0.
    pub sweep_entries: u32,
    /// Bytes of names of one page of the sweep's listing: at least
    /// `kernel::LONGEST_NAME`, which io's `List` demands.
    pub sweep_bytes: u32,
}

/// The limit a start is refused for (git.md, section 8: "The start is
/// refused, naming the limit"). Made by `check`; its terminal is the
/// worker's refused start, which names `git_component.<limit>`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Unfit {
    /// Fewer than the checkout's `workspaces`.
    Operations,
    /// Below `largest_output` under the checkout's limits.
    StdoutBytes,
    /// Below `PushDiagnostic::CAPACITY`.
    StderrBytes,
    /// Zero.
    ReadBytes,
    /// Zero.
    SweepEntries,
    /// Below `kernel::LONGEST_NAME`.
    SweepBytes,
    /// `worst_case` does not fit a `u64`.
    Memory,
}

/// Checks each rule of git.md, section 8, against the checkout's limits.
pub fn check(limits: &Limits, checkout: &checkout::Limits) -> Result<(), Unfit> {
    unreachable!("sketch")
}

/// The largest stdout section 4 reads under the checkout's limits: the
/// unmerged list (`conflicts` paths of `path_bytes`, each with its NUL), a
/// push's status lines (`name_bytes` branch and remote).
#[must_use]
pub fn largest_output(checkout: &checkout::Limits) -> Option<u32> {
    unreachable!("sketch")
}

/// The most memory the component holds (git.md, section 8): per operation
/// in flight its argument vector, environment, message, stdout cap, stderr
/// ring, read buffer and parsed paths; one sweep page; the operations' table.
#[must_use]
pub fn worst_case(limits: &Limits, checkout: &checkout::Limits) -> Option<u64> {
    unreachable!("sketch")
}

/// The most each entry point emits, but `abort` (git.md, section 8: "an
/// event starts at most one invocation, or one file operation").
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MaxOut {
    /// Into `Outputs::events`.
    pub events: u32,
    /// Into `Outputs::io`.
    pub io: u32,
    /// Into `Outputs::kernel`.
    pub kernel: u32,
}

/// One `Done`; a spawn, or a kill and the closes of a child and its three
/// pipes; one file operation.
pub const MAX_OUT: MaxOut = MaxOut { events: 1, io: 4, kernel: 1 };

// temper_worker_protocol_git::boundary (07.1, 07.3a)

/// A git identity's value as the worker's routing hands it with one
/// operation (credentials.md, section 4; channel.md, section 6): the token
/// git authenticates with, its login, and the author. Held only while that
/// operation is in flight. Its `Debug` shows the token's length only.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Identity {
    pub token: Secret,
    pub login: Box<[u8]>,
    pub name: Box<[u8]>,
    pub email: Box<[u8]>,
}

/// What the worker's service gives the component when it makes it.
#[derive(Debug)]
pub struct Settings {
    /// `application.workers[0].git`: git's absolute path (configuration.md, 4).
    pub git: Box<[u8]>,
    /// The absolute path of `home` beneath the worker's root: git's `HOME`.
    pub home: Box<[u8]>,
    /// The worker's root directory, opened once at start by the service; every
    /// spawn and file operation resolves beneath it.
    pub root: kernel::Fd,
    /// The component's token prefix: `(6 << 60) + ((i + 1) << 48)`
    /// (code-organization.md, section 7), for io owners and kernel operations.
    pub prefix: u64,
    /// The first serial number of this start's `trash/<n>`, drawn per start.
    pub serial: u64,
    /// io's `Limits::largest_read`: the most each pipe's read demand asks.
    pub largest_read: u32,
    /// io's `Limits::largest_room`: the most each send to stdin carries.
    pub largest_room: u32,
}

/// The routing to the component, for the worker root's requests.
#[derive(Debug)]
pub enum Request {
    /// The root's `Request::Io`: run `op` for `owner`, giving up at
    /// `deadline`, with its identity's newest value, or `None` when the op
    /// names no identity or the table has no value (then `Broken`, no git).
    /// Its terminal is exactly one `Event::Done` echoing `owner`.
    Io { owner: Token, op: git::Op, deadline: Time, identity: Option<Identity> },
    /// The root's `Request::CancelIo`: abandon `owner`'s operation. Its
    /// `Done` still comes: `Cancelled`, or the outcome that won the race.
    CancelIo { owner: Token },
    /// The worker is shutting down: no further sweep pass starts. No terminal.
    Shutdown,
}

/// The component to the routing.
#[derive(PartialEq, Eq, Debug)]
#[expect(clippy::large_enum_variant, reason = "bounded diagnostics stay inline, as the checkout's Done")]
pub enum Event {
    /// Terminal of `start`: git's version is at least 2.41 and the worker's
    /// root is laid out (`workspaces/` and `home/` empty, `trash/`).
    Started,
    /// Terminal of `start`: the worker cannot run git; the service refuses
    /// its start, naming `application.workers[0].git`.
    Unusable { why: Unusable },
    /// Terminal of `Request::Io`: exactly one per operation.
    Done { owner: Token, done: git::Done },
}

/// Why the start failed.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Unusable {
    /// `git version` failed (`None`), or named a version below 2.41.
    Version { found: Option<Version> },
    /// A file operation of the start's layout failed.
    Layout { error: kernel::Error },
}

/// A git version, as `git version` prints it.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Version {
    /// `X` of `git version X.Y.Z`.
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

/// The queues the parent owns and drains after each entry point, each with
/// room for `MAX_OUT`.
#[derive(Debug)]
pub struct Outputs {
    pub events: Queue<Event>,
    pub io: Queue<io::Request>,
    pub kernel: Queue<kernel::Submit>,
}

/// The git component: the operations in flight, their invocations, pipes,
/// deadlines and file operations; the start; the sweep. Fields elided.
#[derive(Debug)]
pub struct Component {}

impl Component {
    /// Refused with the limit that breaks git.md, section 8's rules.
    pub fn new(limits: &Limits, checkout: &checkout::Limits, settings: Settings) -> Result<Component, Unfit> {
        unreachable!("sketch")
    }
    /// Runs `git version`, then lays the root out (07.3b); ends in `Started`
    /// or `Unusable`. Called once, before any `down`.
    pub fn start(&mut self, now: Time, out: &mut Outputs) {
        unreachable!("sketch")
    }
    /// A request of the routing.
    pub fn down(&mut self, now: Time, request: Request, out: &mut Outputs) {
        unreachable!("sketch")
    }
    /// An io event for one of its children or pipes (`accepts` said so).
    pub fn up(&mut self, now: Time, event: io::Event, out: &mut Outputs) {
        unreachable!("sketch")
    }
    /// A completion of one of its own kernel operations (`owns` said so).
    pub fn complete(&mut self, now: Time, complete: kernel::Complete, out: &mut Outputs) {
        unreachable!("sketch")
    }
    /// Ends at most one operation past its deadline: kills its invocation's
    /// group and closes it; `TimedOut` once the child has closed.
    pub fn fire(&mut self, now: Time, out: &mut Outputs) {
        unreachable!("sketch")
    }
    /// The worker's end: aborts every child and pipe it holds, at most
    /// `operations × 4` io requests; no `Done` follows.
    pub fn abort(&mut self, out: &mut Outputs) {
        unreachable!("sketch")
    }
    /// Whether `event`'s owner is one of its spawns, children or pipes.
    #[must_use]
    pub fn accepts(&self, event: &io::Event) -> bool {
        unreachable!("sketch")
    }
    /// Whether the kernel operation `operation` is its own (by its prefix).
    #[must_use]
    pub fn owns(&self, operation: Token) -> bool {
        unreachable!("sketch")
    }
    /// The earliest deadline of an operation in flight.
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        unreachable!("sketch")
    }
    #[must_use]
    pub fn is_due(&self, now: Time) -> bool {
        unreachable!("sketch")
    }
    /// Nothing in flight, no file operation, no descriptor of its own open.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        unreachable!("sketch")
    }
    /// The kernel operations it may have in flight at once:
    /// `operations + 1` (one per operation, one for the sweep).
    #[must_use]
    pub fn operations(&self) -> u32 {
        unreachable!("sketch")
    }
    /// Frees what ended operations held, at the loop's reclaim point.
    pub fn reclaim(&mut self) {
        unreachable!("sketch")
    }
}

// temper_worker_protocol_git::invocation (07.1): the builders

/// The settings of git.md, section 3, each after `-c`, in this order, on
/// every invocation.
pub const SETTINGS: [&[u8]; 13] = [
    b"core.hooksPath=/dev/null",
    b"core.fsmonitor=false",
    b"protocol.version=2",
    b"protocol.allow=never",
    b"protocol.https.allow=always",
    b"protocol.http.allow=always",
    b"credential.helper=",
    b"http.sslVerify=true",
    b"gc.auto=0",
    b"maintenance.auto=false",
    b"rerere.enabled=false",
    b"merge.conflictStyle=merge",
    b"submodule.recurse=false",
];

/// One invocation of an operation's sequence (git.md, section 4).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Command {
    /// `git version`, once at start.
    Version,
    /// `clone --no-checkout --no-tags --origin origin -- <url> <dir>` (4.2).
    Clone { remote: Box<[u8]>, directory: Box<[u8]> },
    /// `fetch --porcelain … -- <url> +<source>:refs/temper/fetched` (4.3).
    Fetch { remote: Box<[u8]>, source: Source },
    /// `push --porcelain --force-with-lease=refs/heads/<branch>: …` (4.4).
    Create { remote: Box<[u8]>, branch: Box<[u8]>, commit: git::Commit },
    /// `checkout --force --detach <commit>`, the first of `CheckOut` (4.5).
    CheckOut { commit: git::Commit },
    /// `clean -ffdxq`, the second of `CheckOut`.
    Clean,
    /// `merge --no-commit --no-ff -- <theirs>` (4.6).
    Merge { theirs: git::Commit },
    /// `diff-files --name-only -z --diff-filter=U`, Merge's and Commit's.
    Unmerged,
    /// `add --all` (4.7).
    Add,
    /// `diff-index --cached --quiet <parent> --` (4.7).
    DiffIndex { parent: git::Commit },
    /// `write-tree` (4.7).
    WriteTree,
    /// The message goes on stdin, the one invocation with a stdin pipe.
    CommitTree { tree: git::Commit, parent: git::Commit, merging: Option<git::Commit> },
    /// `merge-base --is-ancestor <expected> <commit>`, before a leased push (4.8).
    IsAncestor { expected: git::Commit, commit: git::Commit },
    /// `push --porcelain [--force-with-lease=…:<expected>] -- <url> <commit>:refs/heads/<branch>` (4.8).
    Push { remote: Box<[u8]>, commit: git::Commit, branch: Box<[u8]>, expected: Option<git::Commit> },
}

/// What a fetch names before `:refs/temper/fetched` (git.md, 4.3).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Source {
    /// `refs/heads/<branch>`.
    Branch(Box<[u8]>),
    /// The commit's full id.
    Commit(git::Commit),
    /// `HEAD`, the forge's default branch.
    Default,
}

/// The argument vector after argv[0]: `-c` and each of `SETTINGS`, then the
/// command, its options, `--` and its operands.
#[must_use]
pub fn arguments(command: &Command) -> Box<[Box<[u8]>]> {
    unreachable!("sketch")
}

/// The environment, whole (git.md, section 5): `kind` is the operation's,
/// `None` for the version check; the committer is the identity's author, or
/// `temper <temper@temper.invalid>` without one.
#[must_use]
pub fn environment(home: &[u8], kind: Option<git::Kind>, identity: Option<&Identity>) -> Box<[Box<[u8]>]> {
    unreachable!("sketch")
}

/// The commit message on stdin: `title`, then a blank line and `body` when
/// `body` is not empty, then a newline.
#[must_use]
pub fn message(title: &[u8], body: &[u8]) -> Box<[u8]> {
    unreachable!("sketch")
}

/// An id to git: 40 digits when the last twelve bytes are zero, else 64.
#[must_use]
pub fn hex(commit: git::Commit) -> Box<[u8]> {
    unreachable!("sketch")
}

/// An id from git: exactly 40 or 64 lowercase hex digits, anything else `None`.
#[must_use]
pub fn object(digits: &[u8]) -> Option<git::Commit> {
    unreachable!("sketch")
}

/// The spawn's directory beneath the worker's root:
/// `workspaces/<token in decimal>/<repository>`.
#[must_use]
pub fn place(at: &git::Place) -> Box<[u8]> {
    unreachable!("sketch")
}

// temper_worker_protocol_git::output (07.2): the parsers and the classifier

/// A push's status line for `refs/heads/<branch>` (git.md, 4.4 and 4.8).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Status {
    /// Space: a fast-forward.
    FastForward,
    /// `*` `[new branch]`.
    NewBranch,
    /// `=` `[up to date]`.
    UpToDate,
    /// `!` `[rejected] (non-fast-forward)`.
    NonFastForward,
    /// `!` `[rejected] (fetch first)`.
    FetchFirst,
    /// `!` `[rejected] (stale info)`.
    StaleInfo,
    /// `!` `[remote rejected] (…declined)`.
    Declined,
    /// Any other flag or summary, `+` included.
    Other,
}

/// Fetch's one porcelain line `<flag> <old-id> <new-id> refs/temper/fetched`.
#[must_use]
pub fn fetched(stdout: &[u8]) -> Option<git::Commit> {
    unreachable!("sketch")
}

/// The status of `refs/heads/<branch>` among push's porcelain lines, if any.
#[must_use]
pub fn status(stdout: &[u8], branch: &[u8]) -> Option<Status> {
    unreachable!("sketch")
}

/// A NUL-terminated path list, within the checkout's `conflicts` and
/// `path_bytes`; `None` past them or malformed (`Broken`).
#[must_use]
pub fn paths(stdout: &[u8], checkout: &checkout::Limits) -> Option<Box<[Box<[u8]>]>> {
    unreachable!("sketch")
}

/// `git version X.Y.Z…`'s numbers.
#[must_use]
pub fn version(stdout: &[u8]) -> Option<Version> {
    unreachable!("sketch")
}

/// The fault section 6's table gives the tail's `fatal: ` and `error: `
/// lines, last to first; `Broken` for a local operation or no match.
#[must_use]
pub fn classify(tail: &[u8], kind: git::Kind) -> git::Fault {
    unreachable!("sketch")
}

/// Whether a file's lines hold a conflict marker (git.md, 4.7), fed in
/// reads of any size: a line, split at a newline with one trailing carriage
/// return dropped, that begins with seven `<` and a space, is exactly seven
/// `=`, or begins with seven `>` and a space.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Markers {
    /// The first eight bytes of the line being read.
    head: [u8; 8],
    /// The bytes of that line so far, its newline excluded.
    seen: u32,
    /// Its last byte, to drop one trailing carriage return.
    last: u8,
    /// Whether a line ended so far held a marker.
    found: bool,
}

impl Markers {
    /// Before the file's first byte.
    pub const START: Markers = Markers { head: [0; 8], seen: 0, last: 0, found: false };

    /// One read of the file.
    pub fn feed(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            if byte == b'\n' {
                self.end_line();
            } else {
                if let Some(slot) = self.head.get_mut(usize::try_from(self.seen).unwrap_or(usize::MAX)) {
                    *slot = byte;
                }
                self.seen = self.seen.saturating_add(1);
                self.last = byte;
            }
        }
    }

    /// At the file's end: a last line with no newline counts too.
    #[must_use]
    pub fn finish(mut self) -> bool {
        if self.seen > 0 {
            self.end_line();
        }
        self.found
    }

    fn end_line(&mut self) {
        let length = if self.last == b'\r' { self.seen.saturating_sub(1) } else { self.seen };
        let kept = self.head.get(..usize::try_from(length.min(8)).expect("eight fits")).expect("within the head");
        let divides = length == 7 && kept == b"=======";
        self.found = self.found || kept.starts_with(b"<<<<<<< ") || kept.starts_with(b">>>>>>> ") || divides;
        self.seen = 0;
        self.last = 0;
    }
}

// temper_worker_protocol_git::sequence (07.3a): what comes next

/// What an operation does next (git.md, section 4): which invocation runs
/// only on what the last one said, never on a retry.
#[derive(Clone, PartialEq, Eq, Debug)]
#[expect(clippy::large_enum_variant, reason = "bounded diagnostics stay inline, as the checkout's Done")]
pub enum Next {
    /// Spawn this invocation next.
    Run(Command),
    /// Make's two file operations (07.3b).
    Make,
    /// Commit's scan of these originally conflicted paths, then `Add` (07.3b).
    Scan { files: Box<[Box<[u8]>]> },
    /// The operation's terminal, once its last child has closed.
    End(git::Done),
}

/// The first step of `op`.
#[must_use]
pub fn first(op: &git::Op) -> Next {
    unreachable!("sketch")
}

/// The step after `ran` exited with `code`: by exit status and stdout
/// first; else the stderr tail classifies it, with its diagnostic.
#[must_use]
pub fn after(op: &git::Op, ran: &Command, code: u8, stdout: &[u8], tail: &[u8], cut: u64, checkout: &checkout::Limits) -> Next {
    unreachable!("sketch")
}

// temper_worker_protocol_git::capped and ::ring (07.3a; performance.md, 3)

/// A buffer that grows by doubling up to its cap; a write past the cap is
/// refused whole. Holds an invocation's stdout.
#[derive(Debug)]
pub struct Capped {}

/// A write that would pass the cap; nothing was written.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Full;

impl Capped {
    #[must_use]
    pub fn new(cap: u32) -> Capped {
        unreachable!("sketch")
    }
    pub fn write(&mut self, bytes: &[u8]) -> Result<(), Full> {
        unreachable!("sketch")
    }
    /// A box of exactly its length.
    #[must_use]
    pub fn finish(self) -> Box<[u8]> {
        unreachable!("sketch")
    }
}

/// The last `capacity` bytes written, made once; what falls out is counted.
/// Holds an invocation's stderr tail.
#[derive(Debug)]
pub struct ByteRing {}

impl ByteRing {
    #[must_use]
    pub fn new(capacity: u32) -> ByteRing {
        unreachable!("sketch")
    }
    pub fn write(&mut self, bytes: &[u8]) {
        unreachable!("sketch")
    }
    /// The bytes dropped before the tail.
    #[must_use]
    pub fn dropped(&self) -> u64 {
        unreachable!("sketch")
    }
    /// The tail, oldest first.
    #[must_use]
    pub fn tail(&self) -> Box<[u8]> {
        unreachable!("sketch")
    }
}
```

## Questions to raise when reached

- **`HOME` from a relative root.** configuration.md, 2 resolves paths
  against the working directory, and git.md, 5 wants `HOME` absolute.
  Default: the binary hands the worker its root's absolute path (09.2's
  `WorkerRuntime::path`), and the component takes `Settings::home`.
- **Masked ids.** 07.2's expected values are the transcripts' masked
  ids. Default: 05.4 masks an id by another id of the same length in
  lowercase hex; if it does not, 07.2's tests compare shapes (`Some`)
  rather than values, and say so in the report.

## Done when

- The crate and its world pass: `cargo nextest run -p
  temper-worker-protocol-git -p temper-worker-protocol-git-world` and
  `cargo nextest run -p temper-worker-protocol-git-world --profile
  fuzzy` pass.
- The crate links three crates: `sed -n '/^\[dependencies\]/,/^\[/p'
  crates/temper-worker-protocol-git/Cargo.toml | grep -vE
  '^\[|^$|^(skein-lib|skein-io|temper-worker-domain-checkout)\b'` prints
  nothing.
- No token in the component's arguments or `Debug`: the tests
  `a_remote_operation_carries_the_token_only_in_its_header` and
  `the_identity_s_debug_shows_no_token_byte` pass in the run above.
- The gate passed on this session's last landing: `git -C
  ~/src/rust/temper log -1 --grep='^Step: temper-07\.4$'
  --format='%(trailers:key=Gate-Result,valueonly)' origin/main` names
  `scripts/check.sh passed`.
