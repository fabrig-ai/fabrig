# Session 07: the setup's crate, Forgejo's files, the configuration, the preparation's machine

`temper forge prepare` (temper 08.1) rests on what no host runs. The session creates `temper-forge-setup` (step code: `Setup`, its
checks, the `setup.` limits, `Step`, `steps`; Forgejo's files; the preparation's machine and the processes' specs: 07.1, 07.2, 07.4)
and changes the configuration (07.3): `forges[].user` as a private file, `application.setup` required and handed on undecoded with the
`setup.`/`box.` overrides, `temper serve` and `temper sign-in` calling the crate's checks. No host, no command.

## Read first

- Design: setup.md 3, 6, 8, 9, 11, 13.1; configuration.md 3 (`setup`, `user`), 6, 7, 8; protocol.md 5; the designer's answers
  1-4, 18, 32, 33, 36 (temper `b05af1f7`). Plan: decisions.md 5-9; findings `configuration-and-binary.md` 1, 2, 4-6,
  `forge-and-tiers.md` 2, 5. skein: `programming-model.md` 3, 5.4, 10.2-10.3; `testing-strategy.md` 2.1, 8. temper: `workflow.md` 2.
- Code: `temper-forge-forgejo/src/lib.rs:1-4` (a step crate's shape); `temper-worker-protocol-git/src/invocation.rs:26,74,146`
  (typed commands); the throwaway's `instance.rs:261` (`ini`), `binary.rs:78-85` (environment), `tests.rs:5-12` (its golden).

## What the design asks

Each increment's Scope names its design; the new decisions are the designer's answers 1-4, 18, 32, 33, 36 and the Choices below.

## Choices this plan makes

- **A set-up forge's `user`** is `check::User::File(<directory>/user)`; a number is refused naming `application.forges[i].user`.
  `webhook_secret` and `webhooks` join the check in temper 05.3, in place.
- **`setup` is required** (`[]` for none): temper-service reads its presence alone. Overrides named `setup.…` or `box.…` are kept
  with their values (`limits::handed_on`) and handed on; `checked` applies `setup.` and refuses every other name, so a `box.` name
  is unknown until temper 13.1 (decisions.md 24). The crate has its own problem form (`problem::{Problems, What, lines}`, worded as temper-service's, whose `What` is closed: G1).
- **Bounds the design leaves loose (G5):** logins and org names 1-40 bytes of `[A-Za-z0-9._-]`, the first alphanumeric; email 3-254
  bytes, one `@`, no space, control byte, `<` or `>`; at most 8 labels of at most 128 bytes; `capacity` 1-64; paths at most 4096
  bytes without NUL; `directory` without a trailing `/`.
- **Step order:** each owned forge's `prepare`, `forgejo`, `runner` (if any) in `setup` order, then each entry's `reconcile`, then
  `serve`. **Machines are step functions** `step(&mut Prepare, Event, &mut Queue<Request>)`, one request at a time; entropy, files,
  locks and commands are the host's. `Printed(Box<str>)` is a version line, which 03.1's `names_forgejo`, `names_runner` take as it is.
- **Beyond the design (07.4):** "Forgejo's files" are what a cut-short first preparation leaves and `Clear` removes: `forgejo/`,
  `runner/`, `admin_token`, `person_password`. The admin token is `temper-admin-<16 hex digits>` from a `Draw`, its three scopes one
  comma-joined `--scopes` (taken by 17.0.0: `findings/forgejo-tokens.md`, its header). The connection's `token_url` names `runner/secret` (the token is the secret: `runner-token-is-the-secret`);
  its uuid is `Register`'s stdout. Locks are `forgejo.lock`, `runner.lock` in the directory. A command's streams are kept to 64 KiB.
  Environment, cleared: Forgejo's `HOME=<directory>/home`, `PATH=/usr/bin:/bin`, `TZ=UTC`, `GIT_CONFIG_NOSYSTEM=1`,
  `GIT_CONFIG_GLOBAL=<directory>/home/gitconfig`; the runner's `HOME=<directory>/runner`, `PATH` the setup's.

## Rules that bind it

- `temper-forge-setup` is step code (`no_std`, workspace lints, root `clippy.toml`): text is `Box<[u8]>`; no `String`, `Vec`,
  `format!`, closures or host. No secret in an argument, a URL, a refusal or a `Debug`: `Secret` holds each.
- **Owners:** `Setup`, `Step`, `steps` (07.1); `Place`, `File`, `State` (07.2); `user` as a file, `Named`'s new members (07.3);
  `Spec`, `Command`, `Which`, `prepare` (07.4). **Hotspots (client-api):** temper-service `startup.rs`, `startup/application.rs`;
  `crates/temper/src/lib.rs`, `tests/support/*`; root `Cargo.toml`; `Cargo.lock` is origin/main's, built again.
  `TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64`,
  `TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64`.
- Forbidden in code: `actual|genuine|legacy|compat|workaround`. Files 800 lines, tests 40.

## Increments

### 07.1 The crate `temper-forge-setup`: `Setup`, its checks, `Limits`, `Step`, `steps`
Scope: setup.md 8 (decode, bounds, the forge cross-checks), 7.1, 7.3; configuration.md 7 (`setup.` limits).
Needs: nothing.
Files: new `crates/temper-forge-setup/` (`Cargo.toml`: `skein-lib`, `skein-json`, `jig-protocol-config`, `temper-forge-forgejo` (named
from the start, setup.md 11: 07.4 and temper 09 use it, and 09.1 adds no edge), `[lints] workspace = true`; `src/{lib,setup,check,limits,problem,step}.rs`; `src/tests.rs`;
`src/tests/{fixtures,read,members,limits,steps}.rs`); root `Cargo.toml` (members :5-46, dependencies :54-163), `Cargo.lock`.
`lib.rs` makes each module `pub` and re-exports `Setup`, `Step`, `steps` (temper 09 and 12 import them at the root).
Shape: sketches 1-3 and `problem::{Problems, lines}` (opaque: `Problems::empty()`, `is_empty()`; `lines(&Problems) -> Box<[u8]>`, a line
per problem, `application.setup[0].mode.owned.listen: <reason>`). `Setup::read` uses jig's readers (`read::Object`, `read::text`,
`config::address::read`), wrapped as `application/read.rs:8-66` and `application.rs:175-233` use them. `limits::Limits { orgs: u32, deadline, backoff, backoff_max: Duration }` is
`setup.orgs`, `setup.deadline_ms`, `setup.backoff_ms`, `setup.backoff_max_ms`; `limits::small()` is decisions.md 5 (8; 120 s; 100 ms;
5 s); `limits::apply(&mut Limits, name, value) -> Result<(), Refused>` (`Refused::{Unknown, Range}`; `limits/names.rs:11` is the
model); `limits::check(&Limits) -> Result<(), Unfit>` refuses `backoff > backoff_max` and `backoff_max >= deadline`, each naming both
limits under `application.limits.overrides.<name>`. `check::members` makes setup.md 8's refusals: one entry per forge and per
directory, `forge` a `number`; `api_token` is `<directory>/api_token`, `identity_token` `<directory>/git_token`, `identity_login` the
bot's, `user` `User::File(<directory>/user)`; owned: `api` is `listen`, plaintext, `git` is `http://<listen>`, `listen` on loopback
and unique among `listeners` and the other `listen`s, the person not the bot and not `temper-admin`. `Step` hides once-or-long:
temper-box names `Step` and `steps` alone (setup.md 7.3), and 12.1's ready check counts `new` and `is_long`.
Test shape: new `src/tests/fixtures.rs` (the crate's one fixtures module, extended by 07.2, 07.4, 08, 09, 15): `document(&[u8]) -> Document` as
`temper-service/src/tests/fixtures/startup.rs:31-37`; `TWO` (setup.md 8's two entries: managed forge 1, owned forge 2 with a runner); `members()` matching it;
`changed(from, to) -> Box<[u8]>`, `TWO` with one text replaced (a row of a refusal test). Limits are `limits::small()`, this crate's builder of the shipped profile
(temper-service's is `limits.rs:247-257`); the bound test names `orgs: 1` beside its 8. Unit < 20 ms. Tests: the reference;
`a_managed_entry_reads_its_admin_token_file`; `an_owned_entry_reads_a_null_runner_as_none_and_refuses_a_missing_one`; `an_entry_past_the_orgs_limit_is_refused`;
`a_member_outside_its_bounds_is_refused_naming_its_path` (rows of `changed`: empty `orgs`, `capacity` 0, empty `labels`, a relative `forgejo`, a relative runner
`binary`); `a_forge_whose_api_token_names_another_file_is_refused_naming_its_path`; `a_set_up_forge_whose_user_is_a_number_is_refused`;
`an_owned_forge_whose_api_is_not_its_listen_is_refused`; `a_listen_taken_by_another_listener_is_refused`;
`an_entry_at_odds_with_itself_or_its_forge_is_refused_naming_its_path` (rows: a repeated `forge` or `directory`, the person named as the bot or `temper-admin`;
`members()` with the identity's `token` or `login`, `git`, or a TLS `transport` changed); `the_small_profile_passes_the_setup_limit_checks`;
`a_backoff_past_its_maximum_names_both_limits`; `a_managed_and_an_owned_forge_run_the_six_steps_of_the_design_tree`;
`an_owned_forge_without_a_runner_has_no_runner_step`; `a_managed_forge_alone_runs_reconcile_then_serve`; `an_empty_setup_runs_serve_alone`.
```rust
// crates/temper-forge-setup/src/tests/read.rs
use super::fixtures;
use crate::{limits, problem::Problems, setup::{Mode, Setup}};
use core::net::{Ipv4Addr, SocketAddr};
#[test]
fn an_owned_entry_with_a_runner_reads_into_its_members() {
    let document = fixtures::document(fixtures::TWO);
    let mut problems = Problems::empty();
    let setup = Setup::read(&document, 0, &limits::small(), &mut problems).expect("the design's example reads");
    assert!(problems.is_empty(), "the example reads clean");
    let entry = setup.entry(2).expect("forge 2 is set up");
    assert_eq!(&*entry.directory, b"forge-2");
    let Mode::Owned(owned) = &entry.mode else { panic!("forge 2 is owned") };
    assert_eq!(owned.listen, SocketAddr::from((Ipv4Addr::LOCALHOST, 3000_u16)));
    assert_eq!(owned.runner.as_ref().expect("a runner").capacity, 1);
}
```
Proof: unit, `cargo nextest run -p temper-forge-setup`: the cases above pass and `TWO` gives the six steps in the design tree's
order (setup.md 7.1), under 2 s. Unit: nothing calls the crate until 07.3 (`checked`) and temper 13.1 (`steps`).
Don't: check `webhooks` or `webhook_secret` (05.3); add a second problem form or a `Default`; read a file; expose a `Kind`.
Size: 300. Removes: nothing.

### 07.2 Forgejo's files: `app.ini`, the runner's configuration, `state`, the layout
Scope: setup.md 6.2, 6.3 (the runner's connection), 6.4 (its configuration), 3; decisions.md 8; answer 3.
Needs: nothing (07.1's crate exists at the session's start).
Files: `crates/temper-forge-setup/src/{place,ini,runner,state}.rs`, `src/tests/{golden,files}.rs`, `src/tests/fixtures.rs` (+ `owned()`, `place()`, `secrets()`).
Shape: sketches 4, 5 and `ini::text(owned: &Owned, place: &Place, secrets: &Secrets, allowed_host: Option<IpAddr>) -> Secret`, setup.md
6.2 key by key, values where the design is silent from the throwaway's `ini()` (less `APP_NAME`, `[api]`, `[git]`), `[log]` as files in
the work path, `[webhook] ALLOWED_HOST_LIST` only when `allowed_host` is some (temper 11.1 fills it), `[actions]` on with a runner.
`runner::configuration(runner: &Runner, owned: &Owned, place: &Place, uuid: &[u8]) -> Box<[u8]>`: YAML after 03.3's golden
(`registration::configuration`): `runner.{capacity, timeout: 3h, shutdown_timeout: 0s}`, `cache.enabled: false`, `host.workdir_parent`
(`<directory>/runner/jobs`), `server.connections.forgejo` with `url`, `uuid`, `token_url: file:<RunnerSecret>`, `labels`. No
`runner.file`: a connection writes no registration file (answer 3; `runner-connection-with-a-token-file-comes-online`). `state` is
`{"version":1,"forgejo":..,"runner":..|null,"prepared":..,"secret_key":..,"internal_token":..,"jwt_secret":..}`; another `version` is refused.
Test shape: goldens are literals in `src/tests/golden.rs`, written by hand from setup.md 6.2 and the throwaway's text, never produced
by the code. Tests: the reference; `app_ini_without_a_runner_turns_actions_off`; `app_ini_allows_the_webhooks_listener_when_given_one`;
`the_runner_configuration_matches_the_golden_text`; `a_state_encodes_and_decodes_to_itself`;
`a_state_of_another_version_or_with_an_unknown_member_is_refused`; `a_state_debug_shows_no_secret`.
```rust
// crates/temper-forge-setup/src/tests/golden.rs
use super::fixtures;
use crate::ini;
#[test]
fn app_ini_for_an_owned_forge_with_a_runner_matches_the_golden_text() {
    let text = ini::text(&fixtures::owned(), &fixtures::place(), &fixtures::secrets(), None);
    assert_eq!(text.expose(), APP_INI_WITH_RUNNER, "golden text written by hand from setup.md 6.2");
}
```
Proof: unit, `cargo nextest run -p temper-forge-setup -E 'test(~tests::golden) | test(~tests::files)'`: each file's text equals its
golden, under 1 s. No higher layer yet: Forgejo reads `app.ini` in temper 08.1's proof, the runner its file in 08.2's.
Don't: run or read anything; compute a golden from the code; hold a secret outside `Secret`.
Size: 220 (codec). Removes: nothing.

### 07.3 The configuration: `forges[].user` as a file, `application.setup` handed on and checked
Scope: configuration.md 3 (`setup`, `user`), 8; setup.md 8; answers 4, 33. `temper serve` and `temper sign-in` check the document with
07.1's `checked`; `temper forge` and `temper box` join in temper 08 and 13.
Needs: 07.1.
Files: temper-service: `startup.rs` (`Named` :14-20 and `named` :142-151 gain `forges`, `identities`, `clients`, `overrides`; re-exports
`application::{Forge, Identity, Transport, User}`), `startup/application.rs` (`Members` :11-19; `Forge.user` :28; `configured_limits` :48-84;
`forge` :175-233; `parse` :327; `files` :417-463; `resolve` :482-538), `application/read.rs` (beside `integer` :51), `limits.rs`
(`handed_on`, beside `apply` :259), `src/tests/startup.rs`, `src/tests/fixtures/startup.rs` (`APPLICATION` :9-14, `files()` :38).
`crates/temper`: `Cargo.toml` (`temper-forge-setup`); new `src/setup.rs`; `lib.rs` (`mod setup;`; `prepare` :310-351) and `sign_in.rs:39-43` call
`setup::check` in place of `service::named`; `tests/startup.rs`; `tests/support/{configuration.rs:37-45, forgejo.rs:114-121, mod.rs:32-33}`
(hotspots); `tests/one-process/src/fixtures.rs:98-104`; root `Cargo.toml`, `Cargo.lock`.
Shape: `application::User::{Id(u64), File(Box<[u8]>)}` (`USER_BYTES: u32 = 21`: 20 digits and a LF; re-exported). `Forge.user: User`; `user()` after
`transport`: a number is an id above 0; an object has `file` alone, a non-empty path. `files()` lists a file-form `user` (`private: true`, `max:
USER_BYTES`; capacity `forges * 3 + identities`); `resolve` reads it with `contents` and `decimal` (one final LF dropped; 1 to 20 digits, the first
not 0, overflow refused); a refusal is `What::Jig(config::What::Form)` at `...user.file`, its bytes never in a message. `parse` requires `setup`, naming `application.setup`
when missing; `configured_limits` keeps a `setup.`/`box.` name with its value instead of `Refused::Unknown`. `Named.clients` is the
clients' listener where main holds it (`engine.clients.listener`, `checks.rs:39-42`, until client-api's temper 05.2 moves it to
`application.clients`). `setup::check(bytes) -> Result<(Named, Checked), String>` runs `service::named`, parses the document (`lib.rs:313-315`), fills `Members` (`api` by
`Address::Addr(a).socket()`, each forge's identity joined by `identity`, `listeners` from `clients`), finds `application.setup` by `member`
(`lib.rs:79`) and calls `checked`; its lines are `path: reason`, the caller prefixes them. The documents of `tests/support` and
`one-process` gain `"setup": []`; the startup and process configurations name `{"file": "secrets/forge-1.user"}` and write that file.
Test shape: reuse `application` (:6), `fixtures::{engine, APPLICATION, files}` (`files()` gains `secrets/forge-1.user` = `42\n`, capacity 8); unit < 20 ms, the
binary's tests through `prepare_in` < 0.5 s. Tests: temper-service `a_user_file_member_reads_as_a_private_file_of_21_bytes`;
`a_user_file_resolves_to_the_decimal_id_it_holds`; `a_user_file_holding_no_positive_decimal_is_refused_by_path` (rows: empty, `0`, `042`, `-1`, `4 2`, `42\n\n`, 21
digits); `a_user_object_with_a_stray_or_missing_member_is_refused`; `a_missing_setup_is_refused_naming_its_path`;
`a_setup_or_box_override_is_handed_on_with_its_value`. `crates/temper/tests/startup.rs`: `a_user_file_readable_by_others_refuses_the_start_naming_it`;
`a_setup_naming_another_api_token_file_refuses_temper_serve_naming_its_path`; `a_setup_limit_past_its_check_refuses_temper_serve_naming_both_limits`;
`a_managed_entrys_absent_admin_token_file_does_not_stop_temper_serve` (`prepare_in` is `Ok`);
`a_setup_naming_another_api_token_file_refuses_temper_sign_in_naming_its_path` (`sign_in::account`, `sign_in.rs:39`). A local helper sets `config.json`'s `setup` to one
managed entry (`directory` `forge-1`, `admin_token` `secrets/forgejo-admin`, never written) and, if asked, moves the forge's `api_token`, identity `token` and `user` under `forge-1/`.
Proof: process, with the tree committed:
`export TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64`, then
`sh -c 'NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p temper --profile process -E "test(=temper_open_adopts_a_repository_by_commands)"'`
(`process.rs:34`): `temper serve` passes its start check on the id read from `secrets/forge-1.user` and the person adopts `acme/widget`; at
most 30 s (13.1 s at the base). The unit cases run in the default suite: `cargo nextest run -p temper-service -p temper`.
Don't: decode `setup` in temper-service, or name `temper-forge-setup` or `temper-box` there; read a file that only `setup` names; keep a second reader of `user`;
let a message carry a file's bytes; handle `box.` limits (temper 13.1).
Size: 200. Removes: nothing.

### 07.4 The preparation's machine and the specs
Scope: setup.md 6.3 (items 1 and 2; the upgrade is temper 15.1's), 6.4 (the processes' specs), 9, 13.1; answers 1, 2, 4, 18, 32, 36.
Needs: 07.2; temper 03.3 (`names_forgejo`, `names_runner`, `VERSION`, `RUNNER_VERSION`; facts `cli-…`, `runner-…` in
`tools/forgejo-conformance/facts.txt`: each spec's arguments are theirs).
Files: `crates/temper-forge-setup/src/{prepare,spec}.rs`, `src/tests/{prepare,spec}.rs`, `src/tests/fixtures.rs` (+ `Drive`, `first()`, `later()`).
Shape: sketches 6, 7. `spec::command` builds each spec as `invocation.rs:74,146` does for git: `Migrate`, `CreateAdmin` (`admin user create
--username temper-admin --email temper-admin@example.invalid --admin --random-password --must-change-password=false`), `CreatePerson`
(`--admin --random-password`), `Register` (`forgejo-cli actions register --secret-file <RunnerSecret> --name temper-runner --labels ..`),
each Forgejo command after `--work-path <work path> --config <app.ini>` and run in the work path; `Web`, `Daemon` (`--config
<RunnerConfiguration> daemon`) are temper 08.2's to exec. `spec::admin_token` is `admin user generate-access-token --username temper-admin
--token-name <name> --raw --scopes write:admin,write:organization,write:user` (answer 18). After the version lines (Forgejo's, then the
runner's) the machine decides by `Found`: Forgejo's files and no `state`, `Failure::Unrecorded`; no `state`, a first preparation;
`prepared: false`, a first preparation that begins with `Clear`; `prepared: true` with the pins' versions and the setup's runner presence,
a later one; any other, `Failure::Recorded` (temper 15.1 splits it). A first preparation: `Lock(File::RunnerLock)` (with a runner), `Clear`
(if begun), four `Draw`s (five with a runner), `State` with `prepared: false`, `Layout`, `app.ini`, `Migrate`, `CreateAdmin`, the admin token (to
`AdminToken`), `CreatePerson` (the password, stdout's first line, to `PersonPassword`); with a runner `RunnerSecret` (40 hex digits), `Register`,
`RunnerConfiguration`; `State` with `prepared: true` last. A later one asks no lock: `Layout`, `app.ini` again from `state`'s secrets,
and if `admin_token` is absent a `Draw` and the token for the admin that exists, nothing else: no `CreateAdmin`, which Forgejo refuses a second
time, so an admin a person deleted fails the token's command, `Failure::Token`, which the host words naming `spec::ADMIN` (setup.md 6.3 item 2, 6.5;
answer 36). A failed command ends with its status and a tail of its output, every secret the machine holds masked.
**The locks:** Forgejo's is the host's, taken before `Found` is read; the runner's the machine asks, `Request::Lock(File::RunnerLock)`, and the host
takes it without waiting and answers `Done`, a held one ending the run naming the forge, and keeps it to the process's end (temper 08.1 and 15.1 follow).
Test shape: new `src/tests/prepare.rs`, `spec.rs`; `Drive` (fixtures) holds a `Prepare` and a `Queue::with_capacity(MAX_OUT)`, and `send(Event) -> Request` (model:
the `Started` fixture, `temper-worker-protocol-git/src/tests/fixtures.rs:151`); pin lines are built from `VERSION`, `RUNNER_VERSION` with a `Writer`. Tests: the
reference; `another_runner_binary_is_refused_naming_its_path_and_line` (the Forgejo line is the pin's, the runner's is not);
`a_first_preparation_runs_its_commands_in_order`; `a_first_preparation_records_its_start_first_and_its_end_last`;
`a_first_preparation_with_a_runner_asks_for_the_runners_lock`; `a_later_preparation_at_the_same_versions_writes_app_ini_and_nothing_else` (it asks no lock);
`a_missing_admin_token_is_made_again_under_a_fresh_name`; `a_deleted_admin_ends_a_later_preparation_in_a_token_failure`;
`a_preparation_cut_short_is_removed_and_begun_again_from_an_empty_layout`; `forgejos_files_with_no_state_are_refused`;
`a_recorded_version_other_than_the_pin_is_refused`; `a_failed_command_names_its_status_and_none_of_the_secrets`; in `spec.rs`
`no_argument_of_any_command_holds_a_secret`, `the_environment_is_cleared_to_home_path_and_isolated_git`,
`the_admin_token_asks_for_three_scopes_under_a_suffixed_name`.
```rust
// crates/temper-forge-setup/src/tests/prepare.rs
use super::fixtures;
use crate::{prepare::{Event, Failure, Request}, spec::Which};
use alloc::boxed::Box;
#[test]
fn another_forgejo_binary_is_refused_naming_its_path_and_line() {
    let mut drive = fixtures::Drive::new(fixtures::owned());
    let Request::Version { which: Which::Forgejo, spec } = drive.send(Event::Found(fixtures::first())) else {
        panic!("the Forgejo binary is checked first");
    };
    assert_eq!(&*spec.program, b"/opt/forgejo/forgejo");
    let line: Box<str> = "forgejo version 16.0.5+gitea-1.22.0".into();
    let program = Box::from(&b"/opt/forgejo/forgejo"[..]);
    let ended = drive.send(Event::Printed(line.clone()));
    assert_eq!(ended, Request::End(Err(Failure::Binary { which: Which::Forgejo, program, printed: line })));
}
```
Proof: unit, `cargo nextest run -p temper-forge-setup -E 'test(~tests::prepare) | test(~tests::spec)'`: the requests for each case of
setup.md 13.1 equal the literals, under 1 s. Unit: the machine has no caller until temper 08.1, where the real binaries meet its commands.
Don't: build the upgrade; put a secret in a `Spec`'s arguments; take a lock; call `std` or a clock; write a version number but through `VERSION`.
Size: 300. Removes: nothing.

## Interface sketches

Used by name by temper 08, 09, 10, 12, 13. Types and variants carry docs; fields, trailing comments.

```rust
// crates/temper-forge-setup/src/setup.rs (07.1), sketch 1
use crate::{limits::Limits, problem::Problems};
use alloc::boxed::Box;
use core::net::SocketAddr;
use skein_json::Document;
/// A Forgejo account temper names.
#[derive(Clone, PartialEq, Eq, Debug)] pub struct Account { pub login: Box<[u8]>, pub email: Box<[u8]> }
/// An owned forge's runner: `binary` absolute, `labels` at least one, `capacity` at least 1, `path` the PATH of the runner and its jobs.
#[derive(Clone, PartialEq, Eq, Debug)] pub struct Runner { pub binary: Box<[u8]>, pub labels: Box<[Box<[u8]>]>, pub capacity: u32, pub path: Box<[u8]> }
/// What temper runs for an owned forge: `forgejo` absolute, `listen` on loopback.
#[derive(Clone, PartialEq, Eq, Debug)] pub struct Owned { pub forgejo: Box<[u8]>, pub runner: Option<Runner>, pub listen: SocketAddr, pub person: Account }
/// How a forge is set up: a person runs Forgejo (the site admin's token file), or temper does.
#[derive(Clone, PartialEq, Eq, Debug)] pub enum Mode { Managed { admin_token: Box<[u8]> }, Owned(Owned) }
/// One entry of `application.setup`: `directory` as written, `orgs` at least one and at most `Limits::orgs`.
#[derive(Clone, PartialEq, Eq, Debug)] pub struct Entry { pub forge: u16, pub directory: Box<[u8]>, pub bot: Account, pub orgs: Box<[Box<[u8]>]>, pub mode: Mode }
/// `application.setup`, at most one entry per forge.
#[derive(Clone, PartialEq, Eq, Debug)] pub struct Setup { pub entries: Box<[Entry]> }
impl Setup {
    /// Reads the list at token `at`; each problem is pushed under `application.setup`.
    pub fn read(document: &Document, at: u32, limits: &Limits, problems: &mut Problems) -> Option<Setup> { todo!() }
    /// The entry that sets up forge `number`.
    #[must_use] pub fn entry(&self, number: u16) -> Option<&Entry> { todo!() }
}
```
```rust
// crates/temper-forge-setup/src/check.rs (07.1), sketch 2
use crate::{limits::Limits, problem::Problems, setup::Setup};
use alloc::boxed::Box;
use core::net::SocketAddr;
use skein_json::Document;
/// How a forge's `user` is written.
#[derive(Clone, PartialEq, Eq, Debug)] pub enum User { Id(u64), File(Box<[u8]>) } // a number; `{"file": path}`
/// What `application.forges` and its identity say of one forge; the binary fills it from temper-service's `Named`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Forge {
    pub number: u16, pub api: Option<SocketAddr>, pub plaintext: bool, pub git: Box<[u8]>, pub user: User, // `api` when a socket address; `transport`; whole
    pub api_token: Box<[u8]>, pub identity_token: Box<[u8]>, pub identity_login: Box<[u8]>,                 // paths; its identity's `token` and `login`
}
/// The members the setup is checked against; `listeners` are the other loopback listeners (the clients'; 05.3 adds the webhooks').
#[derive(Clone, PartialEq, Eq, Debug)] pub struct Members { pub forges: Box<[Forge]>, pub listeners: Box<[SocketAddr]> }
/// The setup and limits a command acts on.
#[derive(Clone, PartialEq, Eq, Debug)] pub struct Checked { pub setup: Setup, pub limits: Limits }
/// Applies the `setup.` overrides (any other name is a problem), checks the limits, reads the list at token `setup`, checks it against `members`.
pub fn checked(document: &Document, setup: u32, members: &Members, overrides: &[(Box<[u8]>, u64)]) -> Result<Checked, Problems> { todo!() }
```
```rust
// crates/temper-forge-setup/src/step.rs (07.1), sketch 3
use crate::setup::Setup;
use alloc::boxed::Box;
/// One command of the `temper` binary that a box runs.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Step { name: Box<[u8]>, arguments: Box<[Box<[u8]>]>, long: bool } // `prepare 2`, `forgejo 2`, `runner 2`, `reconcile 1`, `serve`; `forge prepare /abs/config.json 2`
impl Step { // `long`: it runs until stopped (`forgejo`, `runner`, `serve`; the next step starts once it has), else it ends by itself
    #[must_use] pub fn new(name: Box<[u8]>, arguments: Box<[Box<[u8]>]>, long: bool) -> Step { todo!() }
    #[must_use] pub fn is_long(&self) -> bool { todo!() }
    #[must_use] pub fn name(&self) -> &[u8] { todo!() } #[must_use] pub fn arguments(&self) -> &[Box<[u8]>] { todo!() }
}
/// The configuration's steps in order; `configuration` is its absolute path.
#[must_use] pub fn steps(setup: &Setup, configuration: &[u8]) -> Box<[Step]> { todo!() }
```
```rust
// crates/temper-forge-setup/src/state.rs (07.2), sketch 4
use alloc::boxed::Box;
use skein_lib::Secret;
/// Forgejo's own secrets, drawn once at the first preparation.
#[derive(Clone, PartialEq, Eq, Debug)] pub struct Secrets { pub secret_key: Secret, pub internal_token: Secret, pub jwt_secret: Secret }
/// owned's record (setup.md 3): the versions prepared, and whether the first preparation ended.
#[derive(Clone, PartialEq, Eq, Debug)] pub struct State { pub forgejo: Box<[u8]>, pub runner: Option<Box<[u8]>>, pub prepared: bool, pub secrets: Secrets }
/// Why a `state` was refused.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Refused { Json, Version(u64), Member }
#[must_use] pub fn encode(state: &State) -> Secret { todo!() }
pub fn decode(bytes: &[u8]) -> Result<State, Refused> { todo!() }
```
```rust
// crates/temper-forge-setup/src/place.rs (07.2), sketch 5
use alloc::boxed::Box;
/// Where a forge lives: its directory made absolute, and the user that runs its processes.
#[derive(Clone, PartialEq, Eq, Debug)] pub struct Place { pub directory: Box<[u8]>, pub user: Box<[u8]> }
/// A regular file of the forge's directory (setup.md 3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum File {
    AppIni, State, AdminToken, PersonPassword, ApiToken, GitToken, User, WebhookSecret, // prepare's; reconcile's
    RunnerSecret, RunnerConfiguration, ForgejoLock, RunnerLock, // in `runner/` (the secret is also the connection's token file); taken by the preparation and by `run`
}
impl Place {
    #[must_use] pub fn path(&self, file: File) -> Box<[u8]> { todo!() } // the file's absolute path
    #[must_use] pub fn work_path(&self) -> Box<[u8]> { todo!() }        // Forgejo's work path, `<directory>/forgejo`
}
```
```rust
// crates/temper-forge-setup/src/spec.rs (07.4), sketch 6
use crate::{place::Place, setup::Owned};
use alloc::boxed::Box;
/// Which of the two binaries.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Which { Forgejo, Runner }
/// A command of the preparation, or a process `temper forge run` becomes (`Web`, `Daemon`: temper 08.2's).
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Command { Migrate, CreateAdmin, CreatePerson, Register, Web, Daemon }
/// What a host runs: no shell, no `PATH` search, nothing inherited. `environment` is the whole one, `NAME=value`; `output` the bytes kept of each stream.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Spec { pub program: Box<[u8]>, pub arguments: Box<[Box<[u8]>]>, pub environment: Box<[Box<[u8]>]>, pub directory: Box<[u8]>, pub output: u32 }
pub const OUTPUT: u32 = 65_536; // the bytes kept of each output stream, 64 KiB
pub const ADMIN: &[u8] = b"temper-admin"; // temper's admin's login, in its commands and in the host's words for `Failure::Token`
#[must_use] pub fn command(command: Command, owned: &Owned, place: &Place) -> Spec { todo!() }
#[must_use] pub fn admin_token(owned: &Owned, place: &Place, name: &[u8]) -> Spec { todo!() } // temper's admin's token; `name` ends in a random suffix
#[must_use] pub fn version(which: Which, owned: &Owned, place: &Place) -> Spec { todo!() }    // the binary with `--version`, in the same environment
```
```rust
// crates/temper-forge-setup/src/prepare.rs (07.4), sketch 7
use crate::{place::{File, Place}, setup::Owned, spec::{Command, Spec, Which}, state};
use alloc::boxed::Box;
use skein_lib::{Queue, Secret};
/// Output slots of one step.
pub const MAX_OUT: u32 = 1;
/// How a command ended.
#[derive(Clone, Copy, PartialEq, Eq, Debug)] pub enum Exit { Code(u8), Signal(u8) }
/// What the host found under Forgejo's lock: `state`'s bytes (none when absent), whether `admin_token` exists, whether the directory holds Forgejo's files.
#[derive(Clone, PartialEq, Eq, Debug)] pub struct Found { pub state: Option<Secret>, pub admin_token: bool, pub files: bool }
/// A command's end, each stream cut at `Spec::output`.
#[derive(Clone, PartialEq, Eq, Debug)] pub struct Ran { pub exit: Exit, pub stdout: Secret, pub stderr: Secret }
/// What the host sends in: `Found` first; a `Version`'s first stdout line as `Printed`; a `Run` or `Token` as `Ran`; a `Draw` as 32 random bytes; `Done` for a `Lock`, `Clear`, `Layout` or `Write`.
#[derive(Clone, PartialEq, Eq, Debug)] pub enum Event { Found(Found), Printed(Box<str>), Ran(Ran), Drew(Secret), Done }
/// What the machine asks of the host, one at a time.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Request {
    Version { which: Which, spec: Spec },   // run the spec; send its first stdout line as `Printed`
    Lock(File),                             // `RunnerLock`: the host takes it without waiting, answers `Done`; a held one ends the run, naming the forge
    Clear,                                  // remove `forgejo/`, `runner/`, `admin_token`, `person_password`; never `backups/` or `state`
    Layout,                                 // make setup.md 3's directories, private, with `forgejo/repositories` and `home/`
    Draw,                                   // draw 32 random bytes
    Run { command: Command, spec: Spec },   // run to its end; send `Ran`
    Token { spec: Spec },                   // the same, for temper's admin's token
    Write { file: File, contents: Secret }, // write whole and private
    End(Result<(), Failure>),               // the preparation ended; terminal
}
/// Why a preparation stopped; none holds a secret.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Failure {
    Binary { which: Which, program: Box<[u8]>, printed: Box<str> }, // a binary's first line is not the pin's
    Command { command: Command, exit: Exit, tail: Box<[u8]> },      // ended badly; its output masked and cut
    Token { exit: Exit, tail: Box<[u8]> },                          // the same, for the admin's token (a deleted admin's, in a later preparation)
    Unread { command: Command },                                    // printed nothing the preparation reads
    State(state::Refused),                                          // `state` is not a state of this version
    Unrecorded,                                                     // Forgejo's files and no `state`
    Recorded,                                                       // other versions than the pins' (temper 15.1 adds the upgrade)
}
/// The preparation of one owned forge; opaque. `allowed_host` is the webhooks' listener's address for `app.ini`, when configured.
#[derive(Debug)] pub struct Prepare(());
impl Prepare { #[must_use] pub fn new(owned: Owned, place: Place, allowed_host: Option<core::net::IpAddr>) -> Prepare { todo!() } }
pub fn step(machine: &mut Prepare, event: Event, out: &mut Queue<Request>) { todo!() }
```

## Questions to raise when reached

1. **A runner added to or removed from a prepared forge** (`state.runner` and the setup disagree): `Recorded`, as any mismatch; answered
   after 07.4 landed (setup.md 6.3 item 4): a later preparation registers a runner added and records one removed as none, and
   temper 15.1 builds it, `Recorded` going. **The order of several
   owned forges' steps** is pinned by no test. **A bot named `temper-admin`, or an org named like the person or the bot** (Forgejo shares user and org
   names): only the person's login is refused (setup.md 8). **The runner's `timeout` 3 h and `shutdown_timeout` 0 s** are the plan's until temper 08.2.

## Done when

1. The tests pass: `cargo nextest run -p temper-forge-setup -p temper-service -p temper` ends with every test passed.
2. The process proof: README Done when 4's run.
3. Every document carries `setup`: `git grep -L '"setup"' origin/main -- crates/temper-service/src/tests/fixtures/startup.rs crates/temper/tests/support/configuration.rs tests/one-process/src/fixtures.rs` prints nothing.
4. temper-service names neither crate and the step crate no host: `git grep -c -E 'temper-forge-setup|temper-box' origin/main -- crates/temper-service/Cargo.toml` and `git grep -c -E 'skein-(io|http|tls|shell)' origin/main -- crates/temper-forge-setup/Cargo.toml` print nothing.
5. `Step` hides its kind: `git grep -c -E 'pub (const )?fn (new|is_long)' origin/main -- crates/temper-forge-setup/src/step.rs` prints `origin/main:crates/temper-forge-setup/src/step.rs:2`.
6. No forbidden word and no file past 800 lines: `git grep -n -i -E 'actual|genuine|legacy|compat|workaround' origin/main -- crates/temper-forge-setup` and `git grep -c '' origin/main -- 'crates/temper-forge-setup/*.rs' | awk -F: '$NF > 800'` print nothing.
