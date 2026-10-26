# Session 03: the pin, and the facts of Forgejo's command line and the runner

temper's one pinned Forgejo and runner move from the throwaway test crate into
`temper-forge-forgejo`, where owned mode can name them (03.1). The conformance
then learns what the preparation will run: Forgejo's command line on a work
path (03.2, `cli-` facts), and the runner's registration, connection and jobs
(03.3, `runner-` facts). Temper 07.4, 10.2 and 15.1 build on them. The first twelve `cli-`
facts' values and the first nine `runner-` facts' were measured by hand on the pinned binaries
on 2026-10-16; the rest (admin scopes, person, three dump facts, clone) were not: the recording
decides them.

## Read first

- Design: setup.md 6.1 to 6.4 and 13.3 as landed; protocol.md 5; forge.md 10.
  `findings/forgejo-tokens.md`, its header. `decisions.md` 4, 11, 13, 25;
  `findings/forge-and-tiers.md` 1, 2, 3, 5.
- skein `programming-model.md` 10.2, 10.3 (the pin is step code; root `clippy.toml`);
  `testing-strategy.md` 2.1, 8. temper `workflow.md` 2.

## What the design asks

- setup.md 6.1, protocol.md 5: one pin, in `temper-forge-forgejo` (03.1).
- setup.md 6.3, 13.3: what `migrate`, `admin user create`, `generate-access-token` (the
  admin's three scopes, a suffixed name) and `dump` (03.2), `forgejo-cli actions register`
  and `create-runner-file` (03.3) do.
- setup.md 6.4, 13.3: the runner's connection (`uuid` and token from the secret, no
  registration file), a runner on it alone taking jobs, a job cloning its private repository (03.3).

## Choices this plan makes

- **Facts only, no CLI transcript:** transcripts are HTTP bytes that `tests/codecs`
  replays, and a 2xx capture needs its server route (`tests/codecs/src/forgejo.rs:95-99`).
  The admin runners list is read unrecorded, through `Fixture::get` (`fixture.rs:92`).
- **One work-path type** serves `Instance` and the facts; `ini()` keeps its text.
- **Two throwaway literals** (40 hexadecimal digits, scanned for like every secret) serve
  the runner facts, so their expected `uuid`s are literals.
- **The pin's functions are rewritten, not moved:** `Option::and_then` and `format!` are
  disallowed by the root `clippy.toml` this crate follows. The tool reads the pin through
  the throwaway's re-export, so it gains no edge to `temper-forge-forgejo`.
- **The person's Basic `GET` is 01.2's** (`Instance::exchange_basic`); here the person's
  token comes from the command line, read with `Fixture::exchange`.

## Rules that bind it

- **The recording** runs only where a Proof says, as one command from a clean tree
  (commit first: the source must be committed), once, and once more after a fix aimed
  at a cause its output names; a third failure with one symptom is a question
  (development.md 5.5). The tool alone writes `facts.txt` and the transcripts. A `fails`
  line still exits 0: read `facts.txt`; a fact is a measurement, never adjusted to.
  ```sh
  export TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
  export TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
  sh -c 'cargo run -p forgejo-conformance -- --scratch target/forgejo-conformance --out tools/forgejo-conformance && cargo run -p forgejo-conformance -- --scratch target/forgejo-component --out tools/forgejo-conformance --component'
  ```
- **Only these two binaries.** No secret reaches a fact's text, a URL or a log,
  or an argument but the offline `create-runner-file` fact's (03.3).
- **A version is named once,** in `pin.rs`. `actual|genuine|legacy|compat|workaround`
  stay out of code; files stay under 800 lines.
- Shared with temper 01 to 04: the conformance's `lib.rs` flow, `facts.txt`, `README.md`;
  the throwaway's `instance.rs:116` (`token`) and `lib.rs`. client-api touches none.

## Increments

### 03.1 The pin in temper-forge-forgejo
Scope: `VERSION`, `RUNNER_VERSION`, `names_forgejo`, `names_runner` and their
four tests leave `binary.rs:8-21` and `src/tests.rs:32-68` of the throwaway for
`temper_forge_forgejo::pin` (setup.md 6.1; decisions 4); the throwaway re-exports.
Needs: nothing.
Files:
- `crates/temper-forge-forgejo`: new `src/pin.rs`; `src/lib.rs` (`pub mod pin;` after
  `pub mod path;`, :11); `src/tests.rs:1` (`mod pin;`); new `src/tests/pin.rs`.
- Throwaway: `Cargo.toml` (`temper-forge-forgejo.workspace = true`; root `Cargo.toml:93`);
  `src/binary.rs` (the definitions go; :51 uses `names_forgejo`, `pub(crate)` at :14,
  imported from the pin); `src/lib.rs:5` (`pub use temper_forge_forgejo::pin::{RUNNER_VERSION,
  VERSION, names_runner}; pub use binary::{Forgejo, Refusal};`); `src/tests.rs:1-3,32-68`.
- Users, unchanged: `tests/throwaway.rs:2,25`, `tools/forgejo-conformance/src/runner.rs:203`, and
  `Forgejo::from_env` (which calls `names_forgejo`) at `tools/forgejo-conformance/src/{lib.rs:18,
  component.rs:17}`, `tools/git-transcripts/src/lib.rs:76`, `crates/temper/tests/support/forgejo.rs:17`.
  Prose naming `binary.rs`: `tools/forgejo-conformance/README.md:6-7`, `tools/git-transcripts/README.md:10`.
- `Cargo.lock` gains the throwaway-to-forge edge (cargo writes it).
Shape: `pin` below. The four tests keep their names and move to `src/tests/pin.rs`; they build
lines with `str::replace` on templates whose `@` stands for the pin (`format!` is disallowed
here), so no version appears in a test.
Test shape: `src/tests/pin.rs`; no fixtures; no limits (temper's profile builder is
`temper-service/src/limits.rs:247-257`). Tests:
`forgejo_s_line_with_its_build_details_names_the_pin`,
`a_forgejo_version_extending_the_pin_is_refused`,
`the_runner_s_version_line_names_the_runner_pin`,
`a_runner_version_containing_the_pin_is_refused`. Ceiling: unit, 20 ms. Reference:
```rust
// crates/temper-forge-forgejo/src/tests/pin.rs
use crate::pin::{VERSION, names_forgejo};

const BUILT: &str = "forgejo version @+gitea-1.22.0 (release name @) built with GNU Make 4.4.1, go1.26.9 : bindata, timetzdata, sqlite, sqlite_unlock_notify";

#[test]
fn forgejo_s_line_with_its_build_details_names_the_pin() {
    for line in [BUILT.replace('@', VERSION), "forgejo version @".replace('@', VERSION)] {
        assert!(names_forgejo(&line), "{line}");
    }
}
```
Proof: process: the recording (Rules); then `head -2 tools/forgejo-conformance/facts.txt` prints the
forgejo header naming 17.0.0 and `# runner forgejo-runner version v13.2.0`, `grep -c '^holds component-'
…/facts.txt` prints 40, and `grep -E '^fails ' …/facts.txt | cut -d: -f1` still names the base's four `fails` lines: both binaries pass through the pin's new home. Then `git checkout -- tools/forgejo-conformance`
(this increment records nothing). Bound: 10 minutes, build included (the base's recordings took 44 s and 24 s).
Don't: keep a second definition or a copy of a version; check a digest (6.1 asks the version line
alone); change what `Forgejo::from_path` refuses (`tests/throwaway.rs:4-18`).
Size: 40.
Removes: the definitions (14 lines) and the four tests (37 lines) from the throwaway.

### 03.2 Forgejo's command line: a work path, and the `cli-` facts
Scope: the throwaway runs Forgejo's command line on a work path, no web process, and on a running
instance; the conformance records 17 `cli-` facts (13.3 bullet 1's Forgejo part).
Needs: 03.1 landed (the same `lib.rs` and `tests.rs` of the throwaway).
Files:
- `testing/temper-throwaway-forgejo`: new `src/cli.rs`; `src/instance.rs` (`start` prepares
  scratch, home, gitconfig and `app.ini` through `WorkPath`: :28-47 and `command`/`run`
  :84-102 move there; `token` :116 becomes `pub`); `src/lib.rs` (`mod cli; pub use
  cli::{Output, WorkPath};`); `src/tests.rs` (+1); `tests/throwaway.rs` (+3).
- `tools/forgejo-conformance`: new `src/cli.rs`; `src/lib.rs:17-44` (`mod cli;` the group runs
  after `facts::flags` :27 and before the `webhook-deliveries` push :28; `fixture` becomes
  `let mut`); `src/tests.rs` (+2); `README.md` (a `cli-` paragraph); `facts.txt` (by the tool).
Shape: a command is `<binary> --work-path W --config W/app.ini ARGS`, run in `W` with
`binary::environment`'s variables (`binary.rs:78-85`); `W/tmp/` exists. `Output` has no derived
`Debug` (it holds passwords and tokens). `Instance::start` is unchanged outwardly;
`Instance::cli` runs the same on its work path, web running. `F` is `<instance scratch>/cli-fresh`
(`forgejo.work_path`, no web); `I` is `f.instance.cli`. `U x` is `admin user create --username x
--email x@example.invalid --admin --random-password --must-change-password=false`; `P x` is the
same without the flag (the person's form, 07.4's `CreatePerson`). A password is the text between
the first and the last quote of stdout's first line (`password_of`); a raw token is 40 lowercase
hex digits and one LF (`raw_token`). The group adds each password and token it learns, and
`f.instance.secrets()` after its Basic token, to `secrets` before `write::check` (`lib.rs:33`); a
`seen` text quotes lengths and codes, never output. `A` is the admin, `f.owner.login`.

| Fact (`cli-…`) | On | ARGS | Holds when |
|---|---|---|---|
| `migrate-runs-on-a-fresh-work-path` | F | `migrate` | exit 0, stderr empty, `F/forgejo.db` exists, `F/repositories` does not |
| `migrate-runs-again-on-a-migrated-work-path` | F | `migrate` | exit 0 |
| `user-create-prints-the-random-password-first` | F | `U temper-admin` | exit 0, stderr empty; stdout is exactly `generated random password is '<P>'` (P 12 characters) and `New user 'temper-admin' has been successfully created!` |
| `user-create-random-password-length-is-asked` | F | `U other --random-password-length 32` | the same two lines, P 32 characters |
| `user-create-again-is-refused-naming-the-user` | F | `U temper-admin` | exit 1; stderr exactly `Command error: CreateUser: user already exists [name: temper-admin]`; stdout still opens with a password line |
| `user-list-prints-a-header-and-a-row-per-user` | F | `admin user list` | exit 0; line 1 splits on whitespace into `ID Username Email IsActive IsAdmin 2FA`; two six-word rows, the name second |
| `user-create-random-password-signs-in` | I | `U cli-user`, then `f.instance.token("cli-user", P)` | exit 0; the Basic call (`POST /users/cli-user/tokens`) answers 201 |
| `token-raw-prints-the-token-alone` | I | `admin user generate-access-token --username writer --token-name cli-raw --scopes all --raw` | exit 0, stderr empty, stdout a raw token; `GET /api/v1/user` with it answers 200 for `writer` |
| `token-scopes-limit-what-the-token-reads` | I | the same, `--scopes read:repository`, then `read:user` | `GET /api/v1/repos/fixture/specimen`: 200, then 403 `token does not have at least one of required scope(s): [read:repository]` (`facts::matches`, `facts.rs:53`) |
| `token-name-used-twice-is-refused` | I | the same, name `twice`, run twice | second exit 1, stdout empty, stderr exactly `Command error: access token name has been used already` |
| `token-admin-scopes-comma-joined-are-taken-under-two-suffixes` | I | `--username A --scopes write:admin,write:organization,write:user --raw`, names `temper-admin-a` then `temper-admin-b` (setup.md 6.3: a fresh suffix) | both exit 0, stderr empty, a raw token each, the two differing; `GET /api/v1/admin/users?limit=1` under each answers 200 |
| `user-made-to-change-its-password-is-refused-the-api-until-cleared` | I | `P cli-person`; `generate-access-token --username cli-person --token-name person --raw --scopes all`; `GET /api/v1/user`; `PATCH /api/v1/admin/users/cli-person` `{"login_name":"cli-person","source_id":0,"must_change_password":false}` under `f.owner.token`; `GET /api/v1/user` again | the first `GET` 403 with a message containing `change your password`; the `PATCH` 2xx; the second `GET` 200 for `cli-person` (temper 10.2's person is made so) |
| `dump-needs-the-repositories-directory` | F | `dump --file F/dump.zip --tempdir F/tmp` | exit 1; stderr's first line `Failed to include repositories: open F/repositories: no such file or directory`; `F/dump.zip` exists, empty |
| `dump-writes-the-database-and-app-ini` | F, after `create_dir(F/repositories)` | `dump --file F/dump2.zip --tempdir F/tmp` | exit 0, stdout and stderr empty; the file starts `PK` and holds the bytes `forgejo-db.sql` and `app.ini` |
| `dump-appends-zip-to-a-file-name-without-it` | F (repositories made) | `dump --file F/x.zip.partial --tempdir F/tmp` | exit 0; `F/x.zip.partial.zip` starts `PK`; `F/x.zip.partial` does not exist (15.1's partial) |
| `dump-needs-its-tempdir-to-exist` | F (repositories made) | `dump --file F/dump3.zip --tempdir F/absent` | exit 1, stderr not empty; `F/absent` does not exist; `F/dump3.zip` exists, empty |
| `dump-over-an-existing-file-overwrites-it` | F (repositories made), `F/over.zip` holding `junk` | `dump --file F/over.zip --tempdir F/tmp` | exit 0, stdout and stderr empty; the file starts `PK` and holds `forgejo-db.sql` (15.1's restart) |

Test shape: `tests/throwaway.rs` with a fake binary (its pattern at :4-32) for the work path; the
conformance's `src/tests.rs` for the two helpers. Tests:
`a_work_path_runs_commands_in_the_scratch_with_its_environment` (below),
`a_failed_work_path_command_keeps_its_exit_code_and_both_streams`,
`a_work_path_is_refused_when_its_scratch_is_not_empty`,
`an_output_shows_its_lengths_and_no_text` (`src/tests.rs`; literal `Output { code: Some(0),
stdout_bytes: 13, stderr_bytes: 0 }`), `a_password_is_read_between_its_first_and_last_quote`
(`generated random password is 'a'b'` gives `a'b`), `a_raw_token_is_forty_lowercase_digits_and_a_line_feed`.
Ceilings: unit 20 ms, fake-binary tests 0.5 s. Limits: `Settings::small()` (`lib.rs:17-21`); no
profile limit is involved. A fact's reference (`token-name-used-twice-is-refused`) is in the sketches. Reference:
```rust
// testing/temper-throwaway-forgejo/tests/throwaway.rs
use std::{fs, os::unix::fs::PermissionsExt, path::Path};
use temper_throwaway_forgejo::{Forgejo, Settings, VERSION};

/// A Forgejo that prints the pinned line for `--version` and otherwise runs `body`.
fn forgejo_running(root: &Path, body: &str) -> Forgejo {
    fs::create_dir_all(root).expect("root");
    let binary = root.join("forgejo");
    let script = format!("#!/bin/sh\nif [ \"$1\" = --version ]; then echo 'forgejo version {VERSION}'; exit 0; fi\n{body}\n");
    fs::write(&binary, script).expect("script");
    fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).expect("executable");
    Forgejo::from_path(&binary).expect("version accepted")
}

#[test]
fn a_work_path_runs_commands_in_the_scratch_with_its_environment() {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join("work-path-run");
    let forgejo = forgejo_running(&root, "pwd; echo \"$HOME $PATH $TZ\"; echo \"$@\"");
    let work = forgejo.work_path(&root.join("work"), &Settings::small()).expect("prepared");
    let output = work.run(&["admin", "user", "list"]).expect("ran");
    let w = work.scratch().display();
    assert_eq!(
        output.stdout,
        format!("{w}\n{w}/home /usr/bin:/bin Asia/Kolkata\n--work-path {w} --config {w}/app.ini admin user list\n")
    );
    fs::remove_dir_all(root).expect("removed");
}
```
Proof: process: the recording (Rules) from the commit: `grep -c '^holds cli-'
tools/forgejo-conformance/facts.txt` prints 17, `grep -cE '^(fails|unchecked) cli-'` prints 0,
`grep -c '^holds component-'` prints 40, the base's four `fails` lines stay; `git diff --numstat
-- tools/forgejo-conformance/facts.txt` starts `17	0`; `git status --short
tools/forgejo-conformance/transcripts | grep -c '^??'` prints 0. The real Forgejo 17.0.0 answered
every command. Bound: 10 minutes; the group adds at most 15 s to the base's 44 s.
Don't: record a CLI transcript; change `Settings` or `ini()`'s text; start a web process on `F`;
keep `Instance`'s own copy of the preparation beside `WorkPath`; send a Basic `GET` (01.2's).
Size: 190.
Removes: nothing (`Instance`'s `command` and `run` move into `WorkPath`).

### 03.3 The runner: `runner-` facts, and the runner started by connection
Scope: `forgejo-cli actions register` and the runner's `create-runner-file` recorded as 10
`runner-` facts (13.3 bullets 1-2); the conformance starts its runner as setup.md 6.3-6.4 will:
a secret file, `register --secret-file`, `server.connections` with `token_url: file:`, no
registration file; a job clones its private repository with the job's token.
Needs: 03.2 landed (`Instance::cli`, `Secrets`).
Files: `tools/forgejo-conformance`: new `src/registration.rs`, new `src/job_clone.rs`;
`src/runner.rs` (`start` :198-236 rewritten, the API registration token :206-210 and `register
--no-interactive` :220-226 go; `run` :47 calls `registration::facts(f, binary, secrets) ->
Result<Vec<Fact>>`, the first seven, before `run_with` :54, which adds the connection pair and,
when `out` is `Some`, `job_clone::fact(f) -> Result<Fact>` (makes the repository, pushes the workflow, reads the status); without `TEMPER_FORGEJO_RUNNER` all ten are
`unchecked`, as `NAMES` :16 are, :55-62); `src/write.rs:9` (`RegistrationToken` becomes
`RunnerSecret`); `src/lib.rs` (`mod registration; mod job_clone;`); `src/tests.rs` (+3);
`README.md:30-36`; `facts.txt` (by the tool). `component.rs:29` reaches `run_with` too: the
component mode starts the runner the new way, without the clone, and its
`component-read-checks`/`-job` facts (`component/reads.rs:57-96`) are a second witness.
Shape: `FACT_SECRET` = `0123456789abcdef0123456789abcdef01234567` (uuid literal
`30313233-3435-3637-3839-616263646566`); `RUNNER_SECRET` = `fedcba9876543210fedcba9876543210fedcba98`
(`66656463-6261-3938-3736-353433323130`). A secret file is `scratch/runner/secret`, private, the 40
digits and no LF; the runner's `token_url` names that same file. `start` registers RUNNER_SECRET
(`--name temper-conformance --labels temper-host:host`), writes the configuration (connection
`forgejo`, `labels: [temper-host:host]`; no `runner.file`, no `runner.labels`) and spawns `daemon`.
Forgejo's commands run through `f.instance.cli` (`REG` is `forgejo-cli actions register
--secret-file S --name N --labels L`); the runner's through `command(binary, scratch)`
(`runner.rs:42`). The admin list is `GET /api/v1/admin/actions/runners` with `f.owner.token`, a
JSON array of `{id, uuid, owner_id, repo_id, name, status, version, labels, …}`; the throwaway's
page cap is 2 (`Settings::small`), which holds because these facts run before the real runner
registers. The clone fact, after every other runner fact (no earlier transcript moves), makes
`fixture/clone-check` by `f.api("POST", "/api/v1/user/repos", {"name":"clone-check","private":true,
"auto_init":true,"default_branch":"main"})` and `has_actions`, puts one workflow on `main` with
`f.put`, and reads the commit status within 60 s (`run_with`'s 500 ms poll), never a job's log.

| Fact (`runner-…`) | Command | Holds when |
|---|---|---|
| `register-secret-file-prints-the-uuid` | `REG`, FACT_SECRET, `--name fact-runner --labels fact:host` | exit 0, stderr empty, stdout exactly the literal uuid (36 characters, no LF) |
| `register-refuses-a-secret-of-another-length` | `REG` with 39 digits; with 40 digits and a LF | exit 1, stderr `Command error: the secret must be exactly 40 characters long, not 39: generate-secret can provide a secret matching the requirements`; likewise `not 41` |
| `register-refuses-a-secret-that-is-not-hexadecimal` | `REG` with forty `z` | exit 1, stderr `Command error: the secret must be an hexadecimal string: encoding/hex: invalid byte: U+007A 'z'` |
| `register-again-keeps-one-global-runner` | `REG` again, `--name renamed --labels other:host` | exit 0, same uuid; the admin list has one row of that uuid: `owner_id` 0, `repo_id` 0, `name` `renamed`, `labels` `["other:host"]` |
| `create-runner-file-takes-its-secret-only-as-an-argument` | `--config C create-runner-file --instance URL --name n`, plus `--secret-file S`; plus nothing; plus `--secret FACT_SECRET` | exit 1 `Error: unknown flag: --secret-file`; exit 1 `Error: required flag(s) "secret" not set`; exit 0, stderr holding `has been deprecated; declare connections in the runner configuration instead`, `runner.file` written, no instance contacted |
| `uuid-is-the-secrets-first-sixteen-characters-in-hex` | that file | `uuid` equals the literal |
| `token-is-the-secret` | that file | `token` equals FACT_SECRET |
| `connection-with-a-token-file-comes-online` | the runner started on `registration::configuration` with RUNNER_SECRET | within 30 s (polled every 100 ms) its row (the literal uuid) has status `idle` or `active` and `version` `v13.2.0`; no `.runner` file exists in `scratch/runner` (a connection writes no registration file) |
| `connection-alone-takes-the-jobs` | the Actions run | both jobs settle with a success and a failure status (beside `NAMES[0]`) |
| `a-jobs-token-clones-its-private-repository` | the workflow on `fixture/clone-check`: one job, `run:` steps only, no `uses:` (the runner is offline); a first step fails if `GIT_TERMINAL_PROMPT=0 git clone "${{ github.server_url }}/${{ github.repository }}.git" a` succeeds; a second runs `auth=$(printf 'x-access-token:%s' "${{ github.token }}" \| base64 -w0)`, `git -c "http.extraHeader=Authorization: basic $auth" clone <the same URL> b` and `test -s b/README.md` | the head's combined status is `success` within 60 s (temper 10.2 relies on the second step's form) |

Test shape: the conformance's `src/tests.rs`. Tests: the golden configuration (below),
`a_listed_runner_is_found_by_its_uuid` (helper `listed(body, uuid)`; the literal row
`[{"id":1,"uuid":"30313233-3435-3637-3839-616263646566","owner_id":0,"repo_id":0,"name":"r1","status":"idle","version":"v13.2.0","labels":["temper-host"],"description":"","ephemeral":false}]`)
and `the_clone_workflow_runs_steps_only_and_names_the_jobs_token` (no `uses:`; `github.token`
once). Ceiling: unit, 20 ms. Limits: none (`timeout: 1m` and the polls are the runner start's own,
`runner.rs:215`). Reference:
```rust
// tools/forgejo-conformance/src/tests.rs
use crate::registration::configuration;
use std::path::Path;

#[test]
fn the_runner_configuration_names_one_connection_by_its_token_file() {
    let text = configuration(
        "http://127.0.0.1:3000",
        "30313233-3435-3637-3839-616263646566",
        Path::new("/scratch/runner/secret"),
        Path::new("/scratch/runner/jobs"),
    );
    assert_eq!(
        text,
        "log:\n  level: info\nrunner:\n  capacity: 1\n  timeout: 1m\n  shutdown_timeout: 1s\n  fetch_interval: 100ms\n  report_interval: 100ms\ncache:\n  enabled: false\nhost:\n  workdir_parent: /scratch/runner/jobs\nserver:\n  connections:\n    forgejo:\n      url: http://127.0.0.1:3000/\n      uuid: 30313233-3435-3637-3839-616263646566\n      token_url: file:/scratch/runner/secret\n      labels:\n        - temper-host:host\n"
    );
}
```
Proof: process: the recording (Rules) with the runner: `grep -c '^holds runner-'` prints 10,
`grep -cE '^(fails|unchecked) runner-'` prints 0, `grep -c '^holds component-'`
prints 40, the base's four `fails` lines stay; the 13 Actions facts stay `holds` (a runner that took no jobs fails them or
ends the run at 60 s); `git diff --numstat -- tools/forgejo-conformance/facts.txt` starts `10	0`;
`git status --short tools/forgejo-conformance/transcripts | grep -c '^??'` prints 0. Bound: 10
minutes; the start adds at most 5 s, the clone fact at most 30 s.
Don't: run `forgejo-runner register`, or `create-runner-file` outside facts 5 to 7; touch `NAMES`;
leave `RegistrationToken` behind; use `actions/checkout` or any action; read a job's log.
Size: 180.
Removes: `runner.rs:206-210` and `:220-226`; `SecretKind::RegistrationToken`.

## Interface sketches

```rust
// crates/temper-forge-forgejo/src/pin.rs
/// The pinned Forgejo release: the one whose behaviour the conformance checked, and owned mode runs.
pub const VERSION: &str = "17.0.0";
/// The pinned forgejo-runner release.
pub const RUNNER_VERSION: &str = "13.2.0";
/// Whether Forgejo's first `--version` line names `VERSION`, ended by nothing, `+` or a space.
#[must_use]
pub fn names_forgejo(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("forgejo version ") else {
        return false;
    };
    rest.split(['+', ' ']).next() == Some(VERSION)
}
/// Whether forgejo-runner's first `--version` line is `forgejo-runner version v` and `RUNNER_VERSION`, nothing more.
#[must_use]
pub fn names_runner(line: &str) -> bool {
    line.strip_prefix("forgejo-runner version v") == Some(RUNNER_VERSION)
}
```
```rust
// testing/temper-throwaway-forgejo/src/cli.rs
use crate::{Failure, Forgejo, Instance, Settings};
use std::{fmt, path::{Path, PathBuf}};

/// What one command of Forgejo's command line did; invalid UTF-8 in either stream is replaced.
pub struct Output {
    /// The exit code; `None` when a signal ended the command.
    pub code: Option<i32>,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
}
/// Its `Debug` shows the code and the byte counts, never the text: a password or a token may be in it.
impl fmt::Debug for Output { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { todo!() } }
/// A scratch directory Forgejo's command line runs on: `app.ini`, `home/`, `gitconfig`, `tmp/`; no web process.
pub struct WorkPath { forgejo: Forgejo, scratch: PathBuf, port: u16 }
impl Forgejo {
    /// # Errors
    /// Refuses a scratch that is not absolute, is under `/tmp` or is not empty, as `Instance::start` does.
    pub fn work_path(&self, scratch: &Path, settings: &Settings) -> Result<WorkPath, Failure> { todo!() }
}
impl WorkPath {
    /// Runs `forgejo --work-path W --config W/app.ini ARGS` in `W`; the command's own failure is in its `Output`.
    /// # Errors
    /// Returns a spawn failure.
    pub fn run(&self, args: &[&str]) -> Result<Output, Failure> { todo!() }
    /// The canonical scratch directory.
    #[must_use]
    pub fn scratch(&self) -> &Path { todo!() }
}
impl Instance {
    /// Runs a command of Forgejo's command line on this instance's work path, its web process running.
    /// # Errors
    /// Returns a spawn failure.
    pub fn cli(&self, args: &[&str]) -> Result<Output, Failure> { todo!() }
}
```
```rust
// tools/forgejo-conformance/src/cli.rs
use crate::{Result, failure, facts::{Fact, add}, fixture::Fixture, write::{SecretKind, Secrets}};
use temper_throwaway_forgejo::Forgejo;

/// The `cli-` facts: the fresh work path's, then the fixture instance's.
pub fn facts(forgejo: &Forgejo, f: &mut Fixture, secrets: &mut Secrets) -> Result<Vec<Fact>> { todo!() }

pub(crate) fn password_of(stdout: &str) -> Option<&str> { todo!() }
pub(crate) fn raw_token(stdout: &str) -> Option<&str> { todo!() }

fn token_name_used_twice(f: &Fixture, secrets: &mut Secrets, facts: &mut Vec<Fact>) -> Result<()> {
    let args = ["admin", "user", "generate-access-token", "--username", "writer", "--token-name", "twice", "--raw", "--scopes", "all"];
    let first = f.instance.cli(&args).map_err(|e| failure(&e))?;
    secrets.add(SecretKind::ApiToken, first.stdout.trim_end());
    let second = f.instance.cli(&args).map_err(|e| failure(&e))?;
    let held = first.code == Some(0) && second.code == Some(1) && second.stdout.is_empty();
    let said = second.stderr == "Command error: access token name has been used already\n";
    add(facts, "cli-token-name-used-twice-is-refused", held && said, &format!("codes {:?} and {:?}", first.code, second.code));
    Ok(())
}
```
```rust
// tools/forgejo-conformance/src/registration.rs
use std::path::Path;
/// The runner's configuration: one connection to the instance, its token in a file, the jobs' directory.
pub(crate) fn configuration(base: &str, uuid: &str, token_file: &Path, jobs: &Path) -> String { todo!() }
```

## Questions to raise when reached

None open: the designer's answers settled the second `admin_token` (a random suffix), a cut-short
first preparation, the runner's registration file (none) and the scopes.

## Done when

1. `grep -rnE '"17\.0\.0"|"13\.2\.0"' crates testing tools tests --include='*.rs'` prints two
   lines, both in `crates/temper-forge-forgejo/src/pin.rs`.
2. `grep -rnE 'fn names_(forgejo|runner)' crates testing tools tests --include='*.rs'` prints two lines, both in `pin.rs`.
3. `grep -c 'binary.rs' tools/forgejo-conformance/README.md tools/git-transcripts/README.md` prints `0` twice.
4. `cargo nextest run -p temper-forge-forgejo -p temper-throwaway-forgejo -p forgejo-conformance` exits 0.
5. After the recording (Rules): `grep -c '^holds cli-' tools/forgejo-conformance/facts.txt` prints 17,
   `grep -c '^holds runner-'` 10, `grep -c '^holds component-'` 40, and
   `grep -E '^fails ' tools/forgejo-conformance/facts.txt | cut -d: -f1` still names the base's four `fails` lines (counts are by prefix: temper 01, 02 and 04 add their own).
6. `grep -rlE "generated random password is '[^<]|0123456789abcdef0123456789abcdef01234567|fedcba9876543210fedcba9876543210fedcba98" tools/forgejo-conformance/facts.txt tools/forgejo-conformance/transcripts`
   prints nothing.
7. `scripts/check.sh` exits 0 (the gate; 238 s cold at the base, 32 to 48 s warm).
