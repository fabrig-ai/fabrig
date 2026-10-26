# Session 10: the process tier

`temper serve` and `temper ctl` as processes on their own scratch directory:
argv through `main`, printed output, OS exits and signals, stdin sign-in,
and one change landed on the owned Forgejo with real git, CI and the agent
executable (client/tools.md, 11.4; domain/testing.md, 2.6). The harness signs the person
in on Forgejo's web, changes `person_password` and makes a read token,
replacing its `admin_token` read handle. The real loop carries the lock,
DIRECTORY's files, concurrent waits, the lost answer and fake-git landings.

Needs at start: nothing. Steps: `temper-10.1` through `temper-10.4`.

## Read first

- temper's `docs/design/client/tools.md`, at fe0ec316: 5.3, 5.4, 5.6, 6.1 to
  6.3, 7, 8.1 to 8.3, 9, 10, 11.3 to 11.5 (calls, output, recovery,
  the person on Forgejo and the split between testing tiers).
- temper's `docs/design/domain/testing.md`: 2.4 (agent launch) and 2.6
  (credentials, referee, story form, 3-minute aim and process command).
- temper's `docs/design/protocol/setup.md`: 6.3, 6.5, 9, 13.3, 13.4
  and 16 (the person's account and web token; reconcile's admin token).
- temper's `docs/design/client/README.md`, 4.7 (adopt's words).
- This plan's `temper/{05-ctl,08-real-loop,09-real-loop-stories,
  11-tools-forge,12-tools-help}.md`; `decisions.md`, "For the user", C
  (process exceptions), H (read handle); round-2 answers 25, corrected 29, 30, 33;
  `findings/temper-area-1.md`, 5; the client-api plan's process-tier command.
- The code: `crates/temper/tests/process.rs`; `crates/temper/tests/support/`
  (`mod.rs`: `serve`, `Ended`, `read_line`, `printed_link`, `ChatJudge`,
  `LANDING`; `terminal.rs`: `link`, `Typist`; `owned.rs`: `attach`,
  `assert_converged`, `Running`, `await_answer`; `forgejo.rs`:
  `Throwaway`, `person_token`, `repository`, `landing_scripts`;
  `boxed.rs`, its `/usr/bin/kill`); `testing/temper-throwaway-forgejo/src/`
  (`instance.rs`: `Instance::attach`, `Instance::token`, `exchange_basic`;
  `http.rs`: `exchange`, `parse`); skein-world's `end_to_end::Binary`
  and `real::World`.
- skein's `programming-model.md`: 2.1, 9, 10.2; skein's
  `testing-strategy.md`: 2.8, 2.9, 7, 8, 9, 10 (by number only).

## What the design asks

- **11.4, the paths through `main`:** `temper ctl`'s argv, its printed
  output and its exit statuses reach the OS. 10.3.
- **11.4 and 5.6, `sign-in -`:** the link serve prints crosses to ctl's
  stdin, never an argument (9). 10.3.
- **11.4 and 8.2, a termination signal:** the call stops, what it sent
  stays in the small store, and the next call that sends sends it again
  under its key; the same name gets the first answer (7). No signal is
  caught; the OS's status is not 0 (6.3). 10.4.
- **11.4; domain/testing.md 2.4, 2.6:** one landing against real git and CI,
  the worker launching `temper agent`, the forge's state read by API. 10.4.
- **10, the person on Forgejo:** signed in on the web with
  `person_password`, changed; the repository made in the org; a token
  with `read:repository`, `read:issue`, `read:organization`, `read:user`
  made as a person makes one (setup.md, 16); never `admin_token`, which
  reconcile alone reads (setup.md, 9; domain/testing.md, 2.6). 10.1, 10.2.

## Choices this plan makes

- **Web sign-in belongs to the throwaway Forgejo's kit:** `Web` lives
  beside its HTTP client; 10.1 alone changes the kit, a forge-kit hotspot.
- **The person's writes use `Instance::exchange_basic`:** acme/widget
  and the hook story's push use the changed password. The read token
  serves `Instance`, `merged_pull`, `job_state`, `main_readme` and
  `assert_converged`; `person_token`'s admin calls go (domain/testing.md, 2.6).
- **One host runs the calls in one `real::World` with serve:** each
  `Binary` starts when the previous step settles; its descriptors are
  admitted as in `run_story`, stdin written then closed, stdout read to
  EOF, `Ended` kept. Serve gets `Terminate` last and its drain is kept.
- **The adopt is held by pausing Forgejo** with `/usr/bin/kill -STOP`,
  then `-CONT`, as `boxed.rs` kills Forgejo: skein's `Signal` has only
  `Terminate` and `Kill`. The store's record is written before the send
  (8.2), so `pending` shows the cut call's record; that the engine holds
  it is the real loop's (temper 09, the lost answer).
- **Exit statuses (6.3):** success 0; usage and the client's own answers
  1; the engine's refusal alone 2; wait timeout 3 with the last document
  and digest. Client limits, Held, not kept and unknown past retention
  are 1; unconfirmed and client key conflict are tools bugs, also 1.
  Root/host step tests cover the refusal matrix; process tests add no copy.
- **The landing reuses `landing_scripts` unchanged:** the chat starts
  with `@forge_chat`, as the terminal's landing does; no fake LLM change.
- **Each process exception, for decisions.md's "For the user", C:**
  - Web password/token (10.1): the real loop's fake forge has no Forgejo web forms or token authorization.
  - Reconcile's convergence (10.2, existing): the real loop has no owned Forgejo setup executables.
  - Signed push delivery (10.2, existing): the real loop's fake forge cannot prove Forgejo's own signed delivery.
  - Landing and restart (10.2, existing): the real loop lacks real git/CI; this only reruns its existing credential consumer.
  - `help` (10.3): the hosted cli bypasses the executable's `main` and OS argv-to-stdout path.
  - Usage error (10.3): the hosted exit does not prove `main` converts failure to OS exit 1.
  - `sign-in -`, then `me --json` (10.3): the real loop does not join the executables' stdout and stdin or observe their printed JSON bytes.
  - Wait timeout (10.3): the hosted exit does not prove the executable returns OS exit 3 with the last document and digest.
  - Interrupted adopt (10.4): the real loop kills a hosted root; it cannot prove uncaught SIGTERM ends the executable, then its rerun succeeds once.
  - Landing (10.4): real git, Forgejo CI and the worker's spawn of `temper agent` are absent from the real loop (domain/testing.md, 2.4).

## Rules that bind it

- **No `admin_token` as a read handle:** no file under
  `crates/temper/tests/support/` reads `forge-1/admin_token` once 10.2
  lands. `process.rs`'s preparation tests read it as the preparation's
  own file, a check of prepare, and keep it.
- **Credentials are never arguments:** the link reaches ctl on stdin;
  the person's password and tokens go in request bodies and headers, and
  the kit masks them in its failures, as `Instance` masks its secrets.
- **Existing interfaces and fixtures:** no new forge client; no changes
  to the fake LLM, providers or `landing_scripts`. Process configuration
  starts from the shipped profile via `support::configuration`; keep
  the existing landing's overrides and their reasons. No STATUS or
  `[profile.*]` edits, and no optimized test builds.
- **No sleeps for order:** each poll has a named pace (100 ms for
  `pending`, 2 s for the forge, as `ChatJudge`) and the story's deadline.
- **The binaries are the machine's:** `TEMPER_FORGEJO` and
  `TEMPER_FORGEJO_RUNNER` name them; nothing downloads them.
- **Process is the proofs' layer:** each exception above needs the same
  executable or Forgejo boundary as its tests; no live LLM proves more.

## Increments

### 10.1 The throwaway Forgejo's web sign-in and a person's own token

Scope: web sign-in, mandatory password change and a scoped token from
the applications page (setup.md, 6.5 and 16); `Instance::attach` names
its account from `/api/v1/user`, not "temper-admin".
Needs: nothing beyond the session's start.
Files (forge-kit hotspot, the throwaway Forgejo; each named exactly):
`testing/temper-throwaway-forgejo/src/web.rs` (new);
`testing/temper-throwaway-forgejo/src/{http.rs, lib.rs, instance.rs,
tests.rs}`; `testing/temper-throwaway-forgejo/tests/throwaway.rs`.
Shape:
- `http.rs`: the socket's send factored out of `exchange`; a form POST
  (`application/x-www-form-urlencoded`) with a `Cookie` header; an
  answer's `Set-Cookie` values and `Location` read beside `parse`.
- `web.rs`: `Web::sign_in(port, login, password)` GETs `/user/login`,
  keeps its cookies and hidden fields (`_csrf` where given), POSTs
  `user_name` and `password` with them, and follows the redirect by
  hand; `change_password` POSTs `password` and `retype` to
  `/user/settings/change_password`; `token` POSTs `name` and each scope
  to `/user/settings/applications` and reads the 40-hex token from the
  page the redirect answers (its flash). Failures carry the status and
  the page's first bytes, with the password and token masked.
- `instance.rs`: `attach` reads `login` and `id` from `/api/v1/user`.
Test shape: `src/tests.rs` uses canned forms, cookies, hidden fields and
token flashes. The ignored reference in `tests/throwaway.rs` uses a
started throwaway; new `must_change`, `FIRST` and `SECOND` belong there.
Its own admin makes `ada` through `POST /api/v1/admin/users` with
`must_change_password: true` (the kit's admin, in the kit's own test):
```rust
#[test]
#[ignore = "needs TEMPER_FORGEJO"]
fn a_person_changes_a_password_on_the_web_and_makes_a_read_token() {
    let forgejo = Forgejo::from_env().expect("TEMPER_FORGEJO names the pinned Forgejo");
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("web-person");
    let mut instance = forgejo.start(&scratch, &Settings::small()).expect("started");
    must_change(&mut instance, "ada", FIRST); // POST /api/v1/admin/users by the kit's own admin
    let user = Outgoing::get("/api/v1/user", None);
    assert_ne!(instance.exchange_basic(&user, "ada", FIRST).expect("answered").status, 200, "must change first");
    let mut web = Web::sign_in(instance.port(), "ada", FIRST).expect("signed in");
    web.change_password(SECOND).expect("changed");
    let token = web.token("person-read", &["read:repository", "read:user"]).expect("token");
    assert_eq!(instance.exchange(&Outgoing::get("/api/v1/user", Some(&token))).expect("user").status, 200);
    let write = Outgoing::json("POST", "/api/v1/user/repos", Some(&token), br#"{"name":"w"}"#);
    assert_eq!(instance.exchange(&write).expect("refusal").status, 403, "a read token writes nothing");
    instance.stop().expect("stopped");
}
```
Tests: unit, each under 10 ms: `a_form_is_url_encoded`,
`cookies_and_hidden_fields_are_read_from_a_login_page`,
`a_token_is_read_from_the_flash`; process, under 30 s:
`a_person_changes_a_password_on_the_web_and_makes_a_read_token`.
Proof: process, the ignored test once: `sh -c 'cargo nextest run
-p temper-throwaway-forgejo --run-ignored only -E
"test(a_person_changes_a_password_on_the_web_and_makes_a_read_token)"'`
with `TEMPER_FORGEJO` exported (about 20 s).
Don't: use `Instance::token` (the API's basic-auth route) for the
person's token: setup.md, 16 says the web; change `Outgoing`'s fields
or `Failure`'s variants; name a temper engine or test item in the kit.
Size: at most 220 production lines.
Removes: `Instance::attach`'s "temper-admin".

### 10.2 The harness plays the person; the read handle off `admin_token`

Scope: `owned::attach` signs in, changes the password and attaches with
the person's read token; repository and push use their password;
`person_token` goes (client/tools.md, 10; domain/testing.md, 2.6).
Needs: the throwaway Forgejo's web sign-in and person's token (10.1).
Files (forge-kit support hotspot): `crates/temper/tests/support/{owned.rs, forgejo.rs}`;
`crates/temper/tests/process.rs` (the reconcile test's call of `attach`).
Shape:
- `owned::attach(directory, port, running) -> Person` waits on
  `/api/v1/version` (`await_answer`), reads `forge-1/person_password`,
  calls `Web::sign_in(port, "ada", ..)`, `change_password` with a fresh
  32-hex secret, `token("person-read", READ)` with `READ` the four
  scopes of client/tools.md, 10, then `Instance::attach` with that token.
- `Throwaway` keeps the `Person`; `repository` and `push` send by
  `exchange_basic` with the new password; `get` and the referee's reads
  by the instance's token; `person_token` goes.
- `assert_converged` reads the bot, `acme` and its teams with the
  instance's token, as before.
Test shape: no new test: the existing stories run on the person's
handle. The reference: `reconcile_converges_an_owned_forgejo_and_a_second_run_writes_nothing`,
unchanged but for `attach`'s answer.
Tests, each as today: that one, under 60 s;
`a_push_is_delivered_signed_with_the_secret_reconcile_drew`, under 60 s;
`a_persons_change_lands_on_the_forge_and_is_found_after_a_restart`,
within `LANDING` and its restart.
Proof: process, three stories once, with `TEMPER_FORGEJO` and
`TEMPER_FORGEJO_RUNNER` exported: `sh -c 'cargo build -p temper
--bin temper && NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p temper
--profile process -E "test(reconcile_converges_an_owned_forgejo_and_a_second_run_writes_nothing)
| test(a_push_is_delivered_signed_with_the_secret_reconcile_drew) |
test(a_persons_change_lands_on_the_forge_and_is_found_after_a_restart)"'`
(about 5 minutes).
Don't: change the preparation tests' reads of their own files; hand
the read token to temper's configuration (the engine's credentials are
reconcile's); widen `READ`.
Size: at most 80 production lines.
Removes: `forgejo.rs`'s `person_token`; `owned.rs`'s `forge-1/admin_token` read.

### 10.3 `temper ctl`'s fixture and its paths through `main`

Scope: `support::ctl`, a host that runs a story's ctl calls in sequence
beside serve; the help, a usage error, `sign-in -` with `me --json`, and
a wait past its timeout, each as the OS reports it (client/tools.md, 5.6, 6.3,
8.1, 8.2, 11.4).
Needs: `temper ctl`; the tools' help.
Files: `crates/temper/tests/support/ctl.rs` (new);
`crates/temper/tests/support/mod.rs` (`pub mod ctl;`);
`crates/temper/tests/process.rs`.
Shape:
- `Call` supplies arguments after `ctl DIRECTORY` and stdin (Interface
  sketches); `Story` owns directory, steps and deadline; `Seen` keeps ends.
- `run(story)` starts `support::serve`, reads its link as
  `terminal::link` does (`read_line`), then runs one `real::World` of a
  host owning serve's `Binary` and streams and the current call's.
  DIRECTORY is the scratch's `ctl/`, made by nobody but ctl.
- `alone(directory, call)` runs one call with no serve.
  The host checks settled I/O and serve's drain as `run_story` does.
Test shape: four tests in `process.rs`; `Forge::Unreached` for serve.
```rust
#[test]
fn temper_ctl_signs_in_by_the_link_on_stdin_and_reads_me() {
    use support::ctl::{Call, Step, Story};
    let scratch = support::Scratch::new("ctl_sign_in");
    support::configuration(scratch.path(), &support::Forge::Unreached);
    let seen = support::ctl::run(Story::new(scratch.path(), vec![Step::Run(Call::sign_in()), Step::Run(Call::new(&["me", "--json"]))]));
    assert_eq!(seen.calls[0].exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&seen.calls[0].stderr));
    let me: serde_json::Value = serde_json::from_slice(&seen.calls[1].stdout).expect("one JSON object");
    assert_eq!((seen.calls[1].exit, me["digest"].is_string()), (Some(Exit::Code(0)), true), "{me}");
    scratch.remove();
}
```
Tests: `temper_ctl_help_prints_the_guide_and_exits_0` (under 1 s, names `sign-in`);
`temper_ctl_without_a_tool_exits_1_and_opens_nothing` (under 1 s, one
stderr line starting `temper ctl: `, DIRECTORY absent); the reference (under 10 s);
`a_wait_past_its_timeout_exits_3_with_its_digest` (under 10 s: read `me --json`'s
digest D, then `wait me --since D --timeout 1` exits 3 with that document and D).
Proof: process, one story: `sh -c 'cargo build -p temper --bin
temper && NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p temper --profile
process -E "test(temper_ctl_signs_in_by_the_link_on_stdin_and_reads_me)"'`
(about 1 minute with the build).
Don't: test a tool's behaviour here (temper 05's step tests, temper
06's client stories, temper 08's real loop); pass the link as an
argument; read DIRECTORY's files.
Size: at most 200 production lines (the fixture).
Removes: nothing.

### 10.4 An adopt cut by a termination signal, made once, then landed

Scope: one story on the owned Forgejo: `adopt` held by a paused
Forgejo, cut by uncaught `SIGTERM`, kept, rerun under its name with its
first answer and one adoption; then a chat landing with real git and CI, the agent a
`temper agent` process (client/tools.md, 7, 8.2, 11.4; domain/testing.md, 2.4, 2.6).
Needs: the person's read handle (10.2); the ctl fixture (10.3);
the tools' adopt.
Files: `crates/temper/tests/support/ctl.rs`;
`crates/temper/tests/support/owned.rs` (`Running::id`, forge-kit support hotspot);
`crates/temper/tests/support/forgejo.rs` (`Throwaway::pid`, kept before
`attach` takes the child; forge-kit support hotspot); `crates/temper/tests/process.rs`.
Shape: `Step::Start(Call)` (started, not awaited); `Step::Until(Call,
bytes)` (run every 100 ms until stdout holds the bytes); `Step::Cut`
(`Signal::Terminate` to the started call, then its end); `Step::Pause`
and `Step::Resume` (`/usr/bin/kill -STOP|-CONT` on Forgejo's pid, as
`boxed.rs`); `Step::Merged` (`merged_pull` every 2 s, as `ChatJudge`);
`Story` gains the existing fake LLM peer and the `Throwaway`; `Until`
keeps only its last call. `Start`'s answer occupies its step's position.
Poll `pending` for `adopt`; a row is operation, `party N`, and first sending
in temper's wall-time words or `not sent`, no name or key (5.6). Check its
JSON: `pending` rows of `operation`, `party`, `first_sent` (integer or null), and `unread`.
The rerun's kept JSON is only `operation`, `outcome`; text: `kept: ` + operation + outcome (6.1/6.2).
Test shape: one test, the story form of domain/testing.md, 2.6; the fake LLM
`landing_scripts(ANSWER, &forgejo)`.
```rust
#[test]
fn an_adopt_cut_by_a_signal_is_made_once_and_its_change_lands() {
    use support::ctl::{Call, Step, Story};
    let scratch = support::Scratch::new("ctl_landing");
    let forgejo = support::forgejo::Throwaway::start(&scratch);
    support::configuration(scratch.path(), &forgejo.forge());
    let adopt = &["adopt", "--project", "1", "--owner", "acme", "--name", "widget", "--landing", "main", "--key", "widget", "--json"];
    let story = Story::landing(scratch.path(), &forgejo, vec![
        Step::Run(Call::sign_in()), Step::Run(Call::new(&["me", "--json"])), Step::Pause, Step::Start(Call::new(adopt)),
        Step::Until(Call::new(&["pending"]), b"adopt"), Step::Cut,
        Step::Run(Call::new(&["pending", "--json"])), Step::Resume, Step::Run(Call::new(adopt)),
        Step::Run(Call::new(&["pending"])),
        Step::Run(Call::new(&["chat", "--project", "1", "@forge_chat Land a small change", "--key", "change"])),
        Step::Merged,
    ]);
    let seen = support::ctl::run(story);
    let [_, me, cut, _, pending, again, empty, chat] = seen.calls.as_slice() else { panic!("eight ctl results") };
    assert!(cut.exit.is_some() && cut.exit != Some(Exit::Code(0)), "{:?}", cut.exit);
    assert!(cut.stdout.is_empty(), "no answer before the cut");
    assert_eq!(pending.exit, Some(Exit::Code(0)));
    let me: serde_json::Value = serde_json::from_slice(&me.stdout).expect("me");
    let party = me["party"]["number"].as_u64().expect("signed-in party");
    let listed: serde_json::Value = serde_json::from_slice(&pending.stdout).expect("pending object");
    assert_eq!(listed, serde_json::json!({"pending": [{"operation": "adopt", "party": party, "first_sent": listed["pending"][0]["first_sent"].as_u64()}], "unread": 0}), "one cut record; integer or null first sending; no name or key");
    let adopted: serde_json::Value = serde_json::from_slice(&again.stdout).expect("one result");
    assert_eq!((again.exit, adopted["adopted"].is_object()), (Some(Exit::Code(0)), true));
    let kept: serde_json::Value = serde_json::from_slice(&again.stderr).expect("one kept object, no prefix");
    assert_eq!(String::from_utf8_lossy(&again.stderr).lines().count(), 1);
    assert_eq!(kept, serde_json::json!({"operation": "adopt", "outcome": adopted}), "same first answer; no name, key or second outcome");
    assert_eq!((empty.exit, empty.stdout.is_empty()), (Some(Exit::Code(0)), true));
    assert_eq!((chat.exit, chat.stdout.starts_with(b"done: task ")), (Some(Exit::Code(0)), true));
    let merged = forgejo.merged_pull().expect("merged");
    assert_eq!(forgejo.job_state(merged), "success");
    assert!(forgejo.main_readme().ends_with(support::EDIT));
    forgejo.stop();
    scratch.remove();
}
```
Tests: that one, within `LANDING` (180 s) from Forgejo's start, its
adopt steps within 30 s (`WITHIN`).
Proof: process, one story, both binaries exported: `sh -c 'cargo build
-p temper --bin temper && NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p temper --profile process -E
"test(an_adopt_cut_by_a_signal_is_made_once_and_its_change_lands)"'`
(about 3 minutes); then `scripts/proof-one-process.sh`
once on `origin/main`, with both exports; report the tier's time.
Don't: change `landing_scripts`, the fake LLM or the terminal's landing
stories; add a test for agent launch alone; assert an exit status for
the cut call beyond "not 0" (client/tools.md, 6.3 names none).
Size: at most 130 production lines.
Removes: nothing.

## Interface sketches

```rust
// testing/temper-throwaway-forgejo/src/web.rs (10.1)
/// A person signed in on Forgejo's web: its cookies, never printed.
pub struct Web { /* port, cookies, secrets to mask */ }
impl Web {
    /// GET /user/login, then POST its form; a password that must change is answered with the change page.
    pub fn sign_in(port: u16, login: &str, password: &str) -> Result<Self, Failure>;
    pub fn change_password(&mut self, new: &str) -> Result<(), Failure>;
    /// The applications page's form; the token as the answered page shows it once.
    pub fn token(&mut self, name: &str, scopes: &[&str]) -> Result<Token, Failure>;
}

// crates/temper/tests/support/owned.rs (10.2)
pub const READ: [&str; 4] = ["read:repository", "read:issue", "read:organization", "read:user"];
/// The person as the harness plays them (client/tools.md, 10).
pub struct Person { pub instance: temper_throwaway_forgejo::Instance, pub login: String, pub(super) password: String }
pub fn attach(directory: &Path, port: u16, running: Running) -> Person;

// crates/temper/tests/support/ctl.rs (10.3, 10.4)
pub enum Stdin { Nothing, Link, Bytes(Vec<u8>) }
pub struct Call { pub arguments: Vec<Vec<u8>>, pub stdin: Stdin }
impl Call {
    pub fn new(arguments: &[&str]) -> Self;
    /// `sign-in -`, fed the link serve printed.
    pub fn sign_in() -> Self;
}
pub enum Step { Run(Call), Start(Call), Until(Call, &'static [u8]), Cut, Pause, Resume, Merged }
pub struct Story<'a> { /* directory, steps, deadline, peer, forge */ }
impl<'a> Story<'a> {
    pub fn new(directory: &'a Path, steps: Vec<Step>) -> Self;
    pub fn landing(directory: &'a Path, forge: &'a super::forgejo::Throwaway, steps: Vec<Step>) -> Self;
}
/// Each Run/Start/Until's end in step order (Until's last), and serve's drain.
pub struct Seen { pub calls: Vec<super::Ended>, pub serve: super::Ended }
pub fn run(story: Story<'_>) -> Seen;
pub fn alone(directory: &Path, call: Call) -> super::Ended;
```

## Questions to raise when reached

- If web forms differ, raise their status and fields, secrets masked;
  `Instance::token` as a substitute needs a design answer (setup.md, 16).
- If the changed password is refused by basic authentication, raise it
  before making a writing token.
- If the pause fails to hold adoption, raise the steps' times before
  choosing another act to cut; if idle `me` changes within a second,
  raise both documents before choosing another timeout subject.
- If `TEMPER_FORGEJO` or `TEMPER_FORGEJO_RUNNER` is absent, raise it
  before its process Proof; no download. Report the whole tier's time
  at 10.4: the existing landings already exceed domain/testing.md, 2.6's 3-minute aim.

## Done when

- `git grep -n -e '^mod web' origin/main -- testing/temper-throwaway-forgejo/src/lib.rs` prints a line.
- `git grep -n -F 'temper-admin' origin/main -- testing/temper-throwaway-forgejo/src/instance.rs` prints nothing.
- `git grep -n -F -e 'forge-1/admin_token' -e 'admin/users/' origin/main -- crates/temper/tests/support` prints nothing.
- `git grep -n -F 'read:organization' origin/main -- crates/temper/tests/support` prints a line.
- `git grep -n -F -e '"ctl"' -e '"pending"' -e '-STOP' origin/main -- crates/temper/tests/support` prints a line for each.
