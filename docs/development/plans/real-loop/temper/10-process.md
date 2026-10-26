# Session 10: the process tier

`temper ctl` as the shipped executable beside `temper serve`, on a
scratch directory of its own: argv through `main`, printed output, the
OS's exit statuses, the printed link read on stdin, and an act cut by a
termination signal, asked again under its key and made once on the owned
Forgejo (tools.md, 11.4). Today's stories already hold the landing and
agent launch; this session adds neither. The deployment's key stays a
`handed_in` file (decisions.md, design question 1), and the person is
the one `support::forgejo` makes today (decisions.md, 6).

Needs at start: temper 05 (05.6's executable: `Command::Ctl` and
`ctl::run` in `crates/temper/src/ctl.rs`, temper 05's names), with its
acts and `pending` (05.4, 05.5). Part 1's tools (help, `me`'s
digest, `wait`, `adopt`) are on `origin/main`; `pending` is temper 05.5's.
Steps: `temper-10.1`, `temper-10.2`.

## Read first

- temper's `docs/design/client/tools.md`: 5.4, 5.6, 6.1 to 6.3, 7, 8.1 to 8.3,
  11.3, 11.4.
- temper's `docs/design/domain/testing.md`: 2.6 (the tier, its credentials and
  "Outside the suites"), 6.1; `docs/design/protocol/setup.md`, 13.4.
- skein's `docs/design/io.md`, 7 and `shell.md`, 6 (termination signals
  at startup); `kernel.md`, 6.2 (spawn, wait and signal by pidfd).
- This plan's `decisions.md` (design question 1, decision 6) and
  `temper/05-ctl.md`, 05.6.
- The code: `crates/temper/tests/process.rs` (`landing`,
  `a_native_chat_reaches_the_model`,
  `temper_open_adopts_a_repository_by_commands`,
  `temper_serve_prints_its_link_and_drains_on_a_signal`);
  `crates/temper/tests/support/mod.rs` (`serve`, `Ended`,
  `printed_link`, `Process`, `ChatJudge`, `run_story`, `WITHIN`);
  `support/person.rs` (`Person`: a host owning serve's pipes and reading
  its link); `support/configuration.rs` (`documents`);
  `support/owned.rs` (`Running`, `start`, `attach`);
  `support/forgejo.rs` (`Throwaway::start`); `support/boxed.rs`
  (`Judge::advance`'s `/usr/bin/kill`); skein-world's `real::World`
  (`spawn_with_fds`) and `end_to_end::Binary`; skein-io's
  `kernel::{Op, Spawn, Pipe, Way, Signal, Target, Exit}`.
- `.config/nextest.toml`, `[profile.process]`;
  `scripts/proof-one-process.sh`.

## What the design asks

- **11.4, through `main`:** argv, printed output and the OS's exit
  statuses: help (0), a usage error (1) and a wait past its timeout (3,
  with its document and digest; 5.4, 6.3). 10.1.
- **11.4 and 8.1, `sign-in -`:** serve's printed link on ctl's stdin,
  never an argument. 10.1.
- **11.4, 8.2 and 7, a termination signal:** the call exits 143, 128 +
  SIGTERM's number (decisions.md, design question 3); the next call that sends resends it under its key; the rerun gets the
  first answer; the act is made once. 10.2.
- **11.4, the key from a private file and its refusal:** not built
  (design question 1); `support::configuration` hands the credential in
  as today.
- **11.4, agent process launch:** built.
  `support::configuration::documents` makes each worker's agent
  `CARGO_BIN_EXE_temper agent agent.json`, and
  `a_native_chat_reaches_the_model` and both landing tests run it.
  Nothing is added.
- **11.4 and testing.md 2.6, one landing:** built, through `landing`:
  `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart`,
  with the hinted case `a_change_lands_by_deliveries_with_every_poll_raised`
  (setup.md, 13.4). Nothing is added.
- **11.4, the lock, DIRECTORY's files and a call's exit** belong to the
  real loop (temper 08, 09). No test here asserts any of them.
- **testing.md 2.6, "Outside the suites":** the tier aims at 3 minutes;
  "if CI takes it past, the story with CI becomes a test of its own".

## Choices this plan makes

- **No second landing.** Today's `landing` adopts and chats through the
  native client (`support::person::adopt_and_change`), as testing.md 2.6's
  story, step 3, has it. To join that story, the tools' `adopt --landing`
  would take serve's first link through `support::ctl`
  (`sign-in -`, `adopt ... --landing main`, then `chat --project 1
  "@forge_chat Land a small change"`), leaving the native person only
  the restart (`person::found`). The printed link is spent by its first
  sign-in, so the two clients cannot share the first run. That join
  would cost the native client its only landing, so it is not made. The
  tools' landing is temper 13's, in the real loop. The cut needs no CI,
  so 10.2 ends at the adoption.
- **One world per story.** `support::ctl::Calls`, a host after
  `person::Person`, runs the story's calls through io's `Op::Spawn` on
  the scratch's root, beside serve's `Binary`, in one `real::World`.
  Hosts cannot join a running world, so the calls are one host.
- **The act is held by pausing Forgejo:** `/usr/bin/kill -STOP`, then
  `-CONT`, on its pid, as `boxed.rs`'s `Judge::advance` runs `-KILL`.
  skein's `Signal` has only `Terminate` and `Kill`. `Running`'s pid is
  Forgejo's, because `temper forge run` execs it. A stopped Forgejo still
  dies by `Instance`'s stop or drop (`SIGKILL`).
- **No exit 2 story:** 2 crosses the same `main` conversion as 1 and 3.
  Which calls exit 2 is the real loop's.
- **Time.** The tier ran 19 tests in 152.9 s at the plan's base
  (`findings/base-times.md`; the hinted landing 86.4 s, the person's
  landing 23.8 s). Expected times, unmeasured:

  | Test | Expected | Ceiling |
  |---|---|---|
  | `temper_ctl_help_prints_the_guide` | 0.1 s | 1 s |
  | `temper_ctl_without_a_tool_exits_1` | 0.1 s | 1 s |
  | `temper_ctl_signs_in_by_the_link_read_on_stdin` | 0.5 s (serve's link as the drain test's 0.09 s, two calls) | 5 s |
  | `a_wait_past_its_timeout_exits_3` | 1.5 s (its 1 s timeout) | 5 s |
  | `an_adopt_cut_by_a_signal_is_made_once` | 10 s (`Throwaway::start` as the terminal adopt's 6.4 s, ten calls) | 30 s past the start (`WITHIN`) |

  The tier is then about 165 s, against 180 s. A test past its ceiling
  is understood and made cheaper, never excepted (testing.md, 6.1).
  Neither new story runs CI, so neither is "the story with CI". If the
  tier passes 180 s, 10.2 raises it with each test's time (Questions).
- **Stale text:** 10.2 touches `support/forgejo.rs`, so it documents
  `Throwaway` as testing.md 2.6's owned Forgejo, set up by temper's own
  commands and attached through skein's `Instance`. It is not skein's
  throwaway Forgejo (`fake-forge.md`, 6), and its module doc's "isolated
  Forgejo" goes. It also fixes `owned::attach`'s doc, which calls the
  admin's handle "read-only". The type keeps its name, because temper
  13.3 edits `forgejo.rs` too. temper 14.1, which touches
  `.config/nextest.toml`, rewrites its stale `[profile.live]` comment.

## Exceptions to the real loop

One line each, for "For the user":

- `temper ctl help` (10.1): the real loop hosts the cli library, never
  the executable's argv and stdout.
- A usage error's exit 1 (10.1): only the executable shows `main`'s
  conversion of the host's exit to the OS's status.
- `sign-in -`, then `me --json` (10.1): serve's printed link crosses
  process pipes to ctl's stdin, and ctl prints its bytes.
- A wait's timeout, exit 3 with its digest (10.1): the OS's status 3 and
  the document printed beside it.
- An adopt cut by `SIGTERM`, then made once (10.2): the OS delivers the
  signal to the executable, whose host reads skein's `Shutdown` and exits
  143 (design question 3). The real loop ends hosted programs itself.

## Rules that bind it

- **The executable's paths only:** the five exceptions above. Which
  status a call has, kept records, the lock and DIRECTORY's files are
  temper 05's unit tests and temper 08's and 09's real loop.
- **Credentials never as arguments:** the link reaches ctl on stdin.
- **No sleeps for order:** `Until` reruns at a named pace (100 ms); every
  story ends by the world's deadline, `WITHIN`.
- **Nothing else changes:** `landing`, `landing_scripts`, the fake LLM,
  `person`'s scripts, the preparation tests and temper's
  dev-dependencies stay as they are. No new forge client, no
  `[profile.*]` change, no optimized build (README).
- **One behaviour per test,** named for it, never with "and".
- **The binaries are the machine's:** 10.2 needs `TEMPER_FORGEJO` and
  `TEMPER_FORGEJO_RUNNER` (prerequisite 3); nothing downloads them.
  10.1 needs neither (`Forge::Unreached`).

## Increments

### 10.1 `temper ctl`'s fixture and its paths through `main`

Scope: `support::ctl` runs a story's calls one after another beside
serve, or one call alone; it proves help, a usage error, `sign-in -` and
a wait past its timeout, each as the OS reports it (tools.md, 6.3, 8.1,
11.4).
Needs: temper 05 (05.6's executable, 05.5's `sign-in -`); temper-08.s1,
which frees each ended call's stream (ruling 124).
Files: `crates/temper/tests/support/ctl.rs` (new);
`crates/temper/tests/support/mod.rs` (`pub mod ctl;`, and
`Process::Calls` with its arms in `Process::host`, `host_mut` and
`ChatJudge::observe`'s match; the comment on the landing story's bound
cites `domain/testing.md` 2.6, the path since temper `5587ae49b`);
`crates/temper/tests/process.rs`.
Shape:
- `Calls` (a `Host`, after `person::Person`) owns the scratch's root
  (`skein_shell::open_root`) and serve's pipes (`Binary::take_streams`).
  It reads serve's stdout until `printed_link` gives the link.
- A call is an `Op::Spawn`:
  - program `CARGO_BIN_EXE_temper`, args `ctl client` and the call's
    (or `Call::bare`'s alone after `ctl`);
  - an empty environment, as `serve`'s;
  - `root` the scratch's root, declared among `Calls`' descriptors,
    and `dir` empty (`Spawn.dir` is relative to `Spawn.root`); pipes 0
    `Way::In`, 1 and 2 `Way::Out`.
- Its stdin gets the link and `\n` for `Stdin::Link`, then is closed.
- Its stdout and stderr are read to EOF, each up to `OUTPUT`, a
  `const` of `ctl.rs` (65,536 bytes, serve's limit; the guide is 4,698).
- Then `Op::Wait { reap: false }`, then `Op::Wait { reap: true }` (skein-io
  reaps only after an observed exit, one `Wait` per pidfd at a time),
  then `Close` of the pidfd; its `Ended` is kept. The next
  step starts once the call's descriptors are closed.
- `run`: the world holds `Process::Binary(serve)` and `Process::Calls`,
  spawned with their descriptors as `run_story` does. A private referee
  sends serve `Signal::Terminate` once `Calls` is done. It asserts
  serve's exit is `Code(0)` with its stderr shown, and that the world
  settles within `WITHIN`.
- `alone`: `Calls` with no serve and no link.
- DIRECTORY is the scratch's `client`, made only by ctl.
Test shape: four tests in `process.rs`; each scratch is
`support::Scratch::new`, configured by `support::configuration` with
`Forge::Unreached`; `ctl::json` parses stdout, its stderr in the panic.
- `temper_ctl_help_prints_the_guide`: `alone(.., Call::bare(&["help"]))`
  exits `Code(0)`; stdout starts with `b"Agent tools\n"`; stderr is
  empty.
- `temper_ctl_without_a_tool_exits_1`: `alone(.., Call::new(&[]))` exits
  `Code(1)`; stderr is one line starting `temper ctl: `.
- `a_wait_past_its_timeout_exits_3`: sign in, `me --json`, then
  `wait me --timeout 1 --json`. The wait exits `Code(3)`, and its
  `digest` is `me`'s: nothing moved (5.4).
- The reference:
```rust
#[test]
fn temper_ctl_signs_in_by_the_link_read_on_stdin() {
    use support::ctl::{Call, Step, Story};
    let scratch = support::Scratch::new("ctl_sign_in");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let steps = vec![Step::Run(Call::sign_in()), Step::Run(Call::new(&["me", "--json"]))];
    let seen = support::ctl::run(Story::new(scratch.path(), steps));
    let [signed, me] = seen.calls.as_slice() else { panic!("two ctl ends") };
    assert_eq!(signed.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&signed.stderr));
    let document = support::ctl::json(me);
    assert_eq!((me.exit, document["party"]["number"].is_u64()), (Some(Exit::Code(0)), true), "{document}");
    scratch.remove();
}
```
Proof: process, the highest layer for `main`'s paths:
`sh -c 'cargo build -p temper --bin temper &&
NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p temper --profile process -E
"test(/^temper_ctl_/) | test(a_wait_past_its_timeout_exits_3)"'`
(about 1 minute with the build); each test's time in the commit body.
Don't: test a tool's behaviour here (temper 05, 08, 09); pass the link
as an argument; assert DIRECTORY's files or the lock; add a test for
exit 2.
Size: 0 by the plan's count, which leaves out `crates/temper/tests/`;
about 190 lines by hand (`ctl.rs` about 180).
Removes: nothing.

### 10.2 An adopt cut by a termination signal, made once

Scope: one story on the owned Forgejo. It pauses Forgejo, starts
`adopt`, and waits for `pending` to list it. Then `SIGTERM` cuts the
call, Forgejo resumes, and the rerun under the same `--key` gets the
first answer (tools.md, 5.6, 6.2, 7, 8.2, 11.4).
Needs: 10.1's fixture; temper 05's acts and `pending` (05.4,
05.5); part 1's `adopt`.
Files: `crates/temper/tests/support/ctl.rs`;
`crates/temper/tests/support/owned.rs` (`Running::id`; `attach`'s doc);
`crates/temper/tests/support/forgejo.rs` (`Throwaway::pid`, kept before
`attach` takes the child; a doc for `Throwaway`, which has none, and
the module doc);
`crates/temper/tests/process.rs`.
Shape:
- `Step::Start(Call)`: the call starts and is not awaited. Its end
  takes this step's place in `Seen::calls`.
- `Step::Until(Call, bytes)`: the call reruns every `PACE` (100 ms; a
  slower pace only lengthens the story) until its stdout holds the
  bytes. `Seen::calls` keeps the last end.
  `Calls` holds the started call and the current one at once:
  `pending` takes no lock (8.3).
- `Step::Cut`: `Op::Signal { signal: Signal::Terminate, to:
  Target::Child }` to the started call, then its end.
- `Step::Pause` and `Step::Resume`: `/usr/bin/kill -STOP` or `-CONT` on
  `Story::on`'s Forgejo pid; each must exit 0.
- `Story::on` takes the `Throwaway`; serve uses its configuration.
Test shape: one test, the reference:
```rust
#[test]
fn an_adopt_cut_by_a_signal_is_made_once() {
    use support::ctl::{Call, Step, Story, json};
    let scratch = support::Scratch::new("ctl_cut");
    let forgejo = support::forgejo::Throwaway::start(&scratch);
    support::configuration(scratch.path(), &forgejo.forge());
    let adopt = &["adopt", "--project", "1", "--owner", "acme", "--name", "widget", "--landing", "main", "--key", "widget", "--json"];
    let seen = support::ctl::run(Story::on(scratch.path(), &forgejo, vec![
        Step::Run(Call::sign_in()), Step::Run(Call::new(&["me", "--json"])), Step::Pause,
        Step::Start(Call::new(adopt)), Step::Until(Call::new(&["pending"]), b"adopt"), Step::Cut,
        Step::Run(Call::new(&["pending", "--json"])), Step::Resume, Step::Run(Call::new(adopt)),
        Step::Run(Call::new(&["pending"])),
    ]));
    let [_, me, cut, _, listed, again, empty] = seen.calls.as_slice() else { panic!("seven ctl ends") };
    assert_eq!((cut.exit, cut.stdout.as_slice()), (Some(Exit::Code(143)), b"".as_slice()), "128 + SIGTERM");
    let (party, listed) = (json(me)["party"]["number"].as_u64().expect("signed in"), json(listed));
    let first_sent = listed["pending"][0]["first_sent"].clone();
    assert!(first_sent.is_u64() || first_sent.is_null(), "{listed}");
    let row = serde_json::json!({"operation": "adopt", "party": party, "first_sent": first_sent});
    assert_eq!(listed, serde_json::json!({"pending": [row], "unread": 0}));
    assert_eq!(again.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&again.stderr));
    let adopted = json(again);
    let kept: serde_json::Value = serde_json::from_slice(&again.stderr).expect("one kept line, JSON");
    assert_eq!(kept, serde_json::json!({"operation": "adopt", "outcome": adopted}), "the first answer");
    assert_eq!((empty.exit, empty.stdout.as_slice()), (Some(Exit::Code(0)), b"".as_slice()));
    forgejo.stop();
    scratch.remove();
}
```
`from_slice` refuses a second stderr line. One answer under one key is the made-once proof (tools.md, 7).
Proof: process, with both variables exported: `sh -c 'cargo build
-p temper --bin temper && NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p
temper --profile process -E "test(an_adopt_cut_by_a_signal_is_made_once)"'`
(about 1 minute with the build). Then the tier once, `scripts/proof-one-process.sh`, with both exported. The commit body gives
the tier's time and this test's.
Don't: catch or mask the signal in the fixture; add a landing or CI to the
story; change `Throwaway`'s name or `landing_scripts`.
Size: 0 by the plan's count; about 60 lines by hand.
Removes: `owned::attach`'s "read-only proof handle" and `forgejo.rs`'s
"isolated Forgejo".

## Interface sketches

```rust
// crates/temper/tests/support/ctl.rs
use std::path::Path;
/// What a call reads on stdin.
pub enum Stdin { Nothing, Link }
pub struct Call { arguments: Vec<Box<[u8]>>, stdin: Stdin, directory: bool }
impl Call {
    /// `temper ctl client ARGUMENTS...`.
    #[must_use]
    pub fn new(arguments: &[&str]) -> Self;
    /// `temper ctl ARGUMENTS...`, with no DIRECTORY (`help`).
    #[must_use]
    pub fn bare(arguments: &[&str]) -> Self;
    /// `sign-in -`, fed the link serve printed.
    #[must_use]
    pub fn sign_in() -> Self;
}
/// `Run` in 10.1; the rest in 10.2.
pub enum Step { Run(Call), Start(Call), Until(Call, &'static [u8]), Cut, Pause, Resume }
pub struct Story<'a> { scratch: &'a Path, steps: Vec<Step>, forgejo: Option<u32> }
impl<'a> Story<'a> {
    #[must_use]
    pub fn new(scratch: &'a Path, steps: Vec<Step>) -> Self;
    /// 10.2: serve on the owned Forgejo, which `Pause` and `Resume` signal.
    #[must_use]
    pub fn on(scratch: &'a Path, forgejo: &'a super::forgejo::Throwaway, steps: Vec<Step>) -> Self;
}
/// The ends of `Run`, `Start` and `Until` in step order, and serve's.
pub struct Seen { pub calls: Vec<super::Ended>, pub serve: super::Ended }
/// The story's calls, as one host of the world.
pub struct Calls { /* root, serve's pipes, link, steps, started and current calls, ends */ }
#[must_use]
pub fn run(story: Story<'_>) -> Seen;
#[must_use]
pub fn alone(scratch: &Path, call: Call) -> super::Ended;
/// The call's stdout as one JSON object; otherwise panics with its exit and stderr.
#[must_use]
pub fn json(ended: &super::Ended) -> serde_json::Value;

// crates/temper/tests/support/mod.rs: `Process` gains `Calls(Box<ctl::Calls>)`.

// crates/temper/tests/support/owned.rs (10.2)
impl Running {
    #[must_use]
    pub fn id(&self) -> u32;
}

// crates/temper/tests/support/forgejo.rs (10.2)
impl Throwaway {
    /// Forgejo's pid, which `temper forge run` became.
    #[must_use]
    pub fn pid(&self) -> u32;
}
```

## Questions to raise when reached

- **The cut call does not exit 143 within `WITHIN`:** temper 05's host
  ends a call on skein's `Shutdown` with 128 + the signal's number
  (design question 3). Raise it with the exit seen, before any change to
  the host or the test.
- **`pending` does not list `adopt` while Forgejo is paused,** or the act
  is answered first: raise it with the steps' times before choosing
  another act to cut.
- **`adopt --project 1` is refused:** raise it with the projects
  `me --json` lists (the native person adopts in project 1,
  `person::adopt_and_change`).
- **The act's printed record has no `outcome`** to compare with the kept
  line's (6.2, "as the act's"): raise it with both records.
- **`TEMPER_FORGEJO` or `TEMPER_FORGEJO_RUNNER` is absent:** blocked
  (prerequisite 3); no download.
- **The tier passes 180 s:** raise it with each test's time. testing.md
  2.6 says only that "the story with CI becomes a test of its own".

## Done when

Each prints what it says, on `origin/main`, after `git fetch`:

- `git grep -n -F 'pub mod ctl;' origin/main -- crates/temper/tests/support/mod.rs`
  prints a line.
- `git grep -c -E 'fn (temper_ctl_help_prints_the_guide|temper_ctl_without_a_tool_exits_1|temper_ctl_signs_in_by_the_link_read_on_stdin|a_wait_past_its_timeout_exits_3|an_adopt_cut_by_a_signal_is_made_once)\(' origin/main -- crates/temper/tests/process.rs`
  prints a count of 5.
- `git grep -o -n -F -e '"-STOP"' -e '"-CONT"' origin/main -- crates/temper/tests/support/ctl.rs`
  prints two lines, one for each signal (ruling 128).
- `git grep -n -F -e 'read-only proof handle' -e 'isolated Forgejo' origin/main -- crates/temper/tests/support`
  prints nothing.
