# Session 10: `temper forge reconcile`, and the process tier on owned with CI

The last of owned's three commands, and the process tier's move onto them. 10.1 hosts temper 09's reconcile machine on
skein-http over skein-io, as `temper forge reconcile CONFIG.json FORGE`. 10.2 replaces the process tier's provisioning
with the commands: the owned Forgejo, the bot, its tokens, id and the org's team by reconcile, the runner by `run`, the
person's token on the admin's routes, CI on. Webhooks and the hinted story are temper 11, the box 12 and 13; all here is
on polling. `forges[].user` as a file is temper 07.3's. The session Needs, at its start, temper 08.3 (the tier's Forgejo,
`Instance::attach`) and 09.3 (the machine through orgs, teams, members and hooks): its proofs read Forgejo through
`attach` (decisions.md 23).

## Read first

- **Design:** setup.md 5, 6.5, 9, 13.2 (the host's part), 13.4; configuration.md 3 (rows `api`, `transport`), 6, 7 (the
  `setup.` limits); credentials.md 5; domain/testing.md 2.6. `decisions.md` 5, 12 (180 s), 20 (the person's token), 22 (support
  Sizes), 23 (`attach`), 25 (the job's token).
- **Findings:** `configuration-and-binary.md` 1, 4, 6; `skein-io.md` 7; `forge-and-tiers.md` 4; `client-api.md` 2;
  `measurements.md` (the tier at the base: 51 s, the story 31 s). **skein:** programming-model.md 2.1, 9, 10.3;
  testing-strategy.md 2.8, 2.9.
- **This plan:** temper 03.3 (the fact `runner-a-jobs-token-clones-its-private-repository`), 07.1, 07.3 (`Setup`, `Entry`,
  `setup::check`, `Named`), 08.1 (the shell crate: `command::run_one`, `files::write_private`, `tests/support/mod.rs`;
  `forge.rs`; `tests/support/owned.rs`), 08.2 (`temper forge run`, `tests/forge.rs`), 08.3 (`Instance::attach`), 09.1-09.3
  (the machine), 01-02 (the setup's requests).

## What the design asks

- **setup.md 5, 6.5, 9:** reconcile writes only where it differs, waits for the forge within `setup.deadline_ms`, stops
  with status 1 naming the step and leaving every file whole, speaks plaintext to loopback alone, sends every request
  under the admin's token, alone reads `admin_token`; no secret in an argument, a URL or a message (10.1). **13.4,
  domain/testing.md 2.6:** the process tier runs the commands in the box's order; the person, a site admin, makes the repository
  in the org with Actions and a workflow running its check on the owned runner (10.2).

## Choices this plan makes

1. **The wait ends at the first answer that is not a 5xx.** A refused or reset connection or a 5xx on the first request
   is made again after `setup.backoff_ms`, doubling to `setup.backoff_max_ms` (as credentials.md 5 treats the engine's
   start). No later request is made again. One connection per exchange, `Connection: close`. The deadline covers the whole
   run: each exchange gets the time left, and one outliving it ends the first request as `Unanswered`, a later one as
   `Reply::Down` (the host's `next_deadline` closes the connection).
2. **One credential** (designer's answer 14): every request is under the admin's token, which the machine names. The host
   holds no other, no basic auth and no `skein_lib::base64`.
3. **Plaintext only** (answer 6): the command refuses a TLS `transport` before it reads a file, `temper forge reconcile:
   forge N: its transport is TLS, and reconcile speaks plaintext to loopback alone`. A plaintext forge's `api` is a
   loopback socket address already (configuration.md 3).
4. **Limits from the shipped profile** (development.md 5.6; a test naming another says why): documents and HTTP are
   `temper_engine_protocol_forge::limits::{documents, http}` (limits.rs:281, :299) of `temper_service::limits::
   profile(Small).forge_component` (limits.rs:247; forge.rs:110); io is jig's `profile(Small).io` (profile.rs:39), as
   `sign_in.rs:94` takes Small (its `.driver`). Each file is read within its bound: the three tokens' `token_bytes`, `webhook_secret`'s
   `hook_secret_bytes`, `user`'s 21 (20 digits and the final newline, configuration.md 6). The `setup.` limits are 07.1's,
   with the document's overrides. The binary hands the host `Host`'s value (`conventions::host`, conventions.rs:9) and
   `USER_AGENT`, so the shell crate does not depend on the engine's.
5. **Silent on success** (the box's stdout is the sign-in link alone, setup.md 7.2); a failure is one stderr line
   `temper forge reconcile: forge N: <words>`, exit 1, a machine's stop worded `<stage> <subject>: <why>`, a forge the
   setup has no entry for 08.1's `the setup has no entry for it`. A deleted `temper-admin` (setup.md 6.5) leaves a token
   Forgejo refuses (401): the run stops at `admin`, naming it. Relative paths resolve against the working directory, as
   `serve` does (lib.rs:369).
6. **A listing's page is one reply:** the exchange gathers its rows into `Reply::Rows`; any other document is
   `Reply::Document`, a 2xx with no decoder `Empty`, another status `Status`, a failed connection or a body that does not
   decode `Down` (09.1).
7. **The process tier acts as the person by a token** (decisions.md 20): made once on the admin's routes under
   `admin_token`, which only reconcile and the tier read (the test stands for the person's sign-in), scopes
   `write:repository` and `write:organization`, kept in the handle, never in a file. The org is private (answer 19), so
   is the repository, and the job clones with its own token (03.3's fact; decisions.md 25).
8. **`application.webhooks` reaches reconcile when it exists:** `Settings.listener` is `None` while `Named` lacks the
   member; whichever of 05.3 (which adds it) and 10.1 lands second wires it, and 11.1 proves it. `Settings.hook_secret_bytes`
   comes from `forge_component.hook_secret_bytes` (`temper-engine-protocol-forge/src/limits.rs:50`, 128 shipped,
   `temper-service/src/forge.rs:139`), which landed with a1a734f4d; 10.1 landed with `token_bytes` (`forge.rs:255`), and 05.3 sets it, beside `listener`
   (ruling 135).

## Rules that bind it

- **Hotspots** (client-api.md 2): rebase on origin/main before touching `crates/temper/src/lib.rs`,
  `crates/temper/tests/{process.rs,support/*}`; `Cargo.lock` is regenerated after a rebase, never merged by hand.
- **The shell crate is ordinary Rust, the machine is step code:** the host names no route, path or Forgejo word, and
  holds no `std::process` or file lock (08.1's boundary test scans `crates/*/src`; `crates/temper/tests` is outside it).
  No secret in an argument, a URL, a log line or a message: the token is a `Secret` from the file to the header.
- **The binaries,** exported on a line of their own before any process-tier command below: `export
  TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
  TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64`.
- **10.2's Size** counts the process tier's support (code-organization.md 6).

## Increments

### 10.1 Reconcile's host, and `temper forge reconcile`
Scope: setup.md 5, 6.5, 9, 13.2 (the host's side); answers 6 and 14. The host runs 09's machine to its end on skein-http
over skein-io within `setup.deadline_ms`: it does what the machine needs (a Forgejo request, a file of the forge's
directory read or written whole, entropy) and hands back the outcome.
Needs: 08.3 (`Instance::attach`; through 08.2, `temper forge run`, and 08.1's shell crate and `forge.rs`) and 09.3. Not
05.3: Choice 8 says how `listener` and `hook_secret_bytes` reach `Settings` before it lands.
Files: new `crates/temper-forge-setup-shell/src/{reconcile,exchange}.rs`, `src/lib.rs`, `Cargo.toml` (dependency
`skein-http`; `temper-forge-forgejo` and `skein-json` are 08.1's, and 10.1 adds either if 08.1 did not; dev-dependencies
`temper-service` and `temper-engine-protocol-forge`, for the profile's limits), `tests/{reconcile,exchange}.rs`,
`tests/support/mod.rs` (08.1's), `Cargo.lock`; `crates/temper/src/forge.rs` (08.1's: the `reconcile` arm), `src/lib.rs`
(client-api 05.2 hotspot: `USAGE` :23 alone), `src/tests.rs` (the two literals, :7, :14), `tests/forge.rs` (08.2's),
`tests/process.rs`, `tests/support/owned.rs` (hotspots).
Shape: sketches A-C.
**Corrections against main (ruling 134, at temper 4764383ae; a reference patch, with a stub host:
`~/src/rust/tmp/temper/forge-setup/logs/supervisor/10-1-check-4764383.patch`):**
- `reconcile_makes_a_missing_forge_directory_private` overrides `setup.backoff_max_ms` (at most 299, say 100) beside
  `setup.deadline_ms` 300 (`check.rs:125-155` refuses one without the other), and writes a private `admin_token`
  (the machine's first need, `reconcile.rs:147`).
- `USAGE` (`lib.rs:24`) and its copies: `src/tests.rs:7, :14, :129`, and `forge.rs:146`'s `usage` string.
- `tests/support/owned.rs` has `run(directory, program) -> Command` (:180, nine users) and `Running::start` (:192):
  the tier's runner is `owned::run_forge` (or another free name), `Running` logs to `server.log` (`instance.rs:121`
  reads it), holds `child: Option<Child>` so `into_child` can take it past its `Drop`, and `start` returns the port
  `prepared_processes` (:253) gives.
- The fixture's profile is `temper_service::limits::Profile`, not jig's; times are `skein_shell::Clock`, never
  `std::time::Instant` (clippy); `Peer::serve_after`'s accepts are nonblocking within about 1 s, so a run that never
  connects ends; `Stage` and `Why` are worded by exhaustive matches.
- Lines: `address()` `lib.rs:203`, `current_dir` :375; `limits::{documents, http}` `limits.rs:325, :343` (`http` takes
  `(l, tls, &io)`); `profile` `temper-service/src/limits.rs:249`; `Instance::exchange` `instance.rs:216`; `client.rs`
  `encode` :38 (its `Err(Refusal)` is a `Reply` of its own), `decoder` :117, `step` :195.
- **The exchange** (`exchange.rs`): one `skein_http::client::Client` over one socket per request, as `connection.rs:158-259`
  does without TLS, with `close: true`: `io::Request::Connect`; the call with `Host`, `Authorization`, `User-Agent`,
  `Content-Type`; `temper_forge_forgejo::client::encode` (client.rs:36) gives the method (01.1's `Put` mapped too), target and body; the answer's
  body goes into a `tokenizer::Tokenizer` whose events go to `client::step` (:160; the loop whole is
  `drive`, `temper-forge-forgejo/src/tests/fixtures.rs:67`) and `Gather`. A request with no decoder (`client::decoder` :98) is read
  for its status alone.
- **The loop** (`reconcile.rs`): a `skein_shell::Host` as sign_in.rs:164-217, with its own `Kernel::open` and `drive`
  (sign_in.rs:102, :139) as 08.1's `command::run_one`; its `next_deadline` is the earlier of the exchange's end (Choice 1)
  and the pause's. It refuses at start (`Refused::Limits`) unless `io.intake >= http::largest_read` and `io.output >=
  http::largest_room` (skein-http client.rs:101, :109). Files: read by `skein_shell::read_private_file(path, bound(..),
  effective_user())`, written by 08.1's whole-file writer; the machine's `File` maps to `<directory>/<name>`, but a managed
  `AdminToken` to the entry's `admin_token`. A draw is `skein_shell::crypto_seed()` (32 bytes). An absent file is
  `Event::Read(None)`; an unsafe, oversize or unreadable one ends the run as `Refused::File`.
- **The command:** an arm beside `prepare` and `run`, the same arguments, in `forge.rs`; it checks the document as
  `prepare` does and builds `Settings`: `address()` (lib.rs:197) gives `api`, `setup.*` come from 07.1's `Limits` with the
  document's overrides. Order: the TLS refusal (Choice 3), the forge's entry (a managed one too; absent: Choice 5's words),
  its directory made 0o700 and checked as `prepare` does (reconcile is the first command to write in a managed forge's
  directory, configuration.md 6; a managed `admin_token` is the person's, wherever it is), the run. `USAGE` gains `temper
  forge reconcile CONFIG.json FORGE`. Status 0 or 1.
- **The support** (`support/owned.rs`, 08.1's): `run(directory, which) -> Running` (`temper forge run config.json 1
  forgejo|runner` as a child, its output in `<directory>/server.log` for Forgejo, where 08.3's `attach` reads it, and
  `<directory>/runner.log` for the runner; killed on drop; `Running::into_child` hands the child over, disarming the
  drop), `start(directory) -> Running` (`prepare`, then `run` Forgejo; it does not wait: reconcile does) and
  `reconcile(directory, forge) -> Output`. 08.3's `Throwaway::start` spawns Forgejo inline, for this lift. No HTTP helper
  (decisions.md 23): the proof reads through `attach`.
Test shape: `tests/{reconcile,exchange}.rs` of the shell crate. Fixtures in `tests/support/mod.rs`: `settings(address:
SocketAddr) -> Settings` (a scratch directory holding a private `admin_token`, the profile's limits (Choice 4; io is
08.1's `io()`), an owned `Entry`, `deadline` 2 s, `backoff` 50 ms, `backoff_max` 100 ms: named here because the profile's
120 s, 100 ms and 5 s make a test slow), `ADMIN` (the admin's user document, JSON) and `Peer`, a loopback `TcpListener`
answered inside `std::thread::scope` (clippy.toml bans `thread::spawn`): `Peer::reserve() -> Peer` (a free port: bind,
drop), `Peer::address(&self) -> SocketAddr`, `Peer::serve_after(self, ms: u64, replies: &[(u16, &[u8])]) -> Heard` (binds
after `ms`, answers one request per reply on its own connection, then holds one more connection unanswered for at most
1 s), `Heard { requests: Vec<Received> }`, `Received { line: String, headers: Vec<(String, String)> }` (`line` as `GET
/api/v1/user HTTP/1.1`, header names lower-case). Focused < 0.5 s; `exchange.rs` is unit, < 20 ms. Run `cargo
nextest run -p temper-forge-setup-shell` and `cargo nextest run -p temper --test forge`.
- `a_first_request_is_made_again_until_the_api_answers` (reference, sketch E); `the_admins_token_is_sent_as_a_token_header`
  (the peer hears `authorization: token admin-token-value`);
- `a_run_ends_at_its_deadline_when_nothing_answers` (deadline 300 ms: `temper forge reconcile: forge 1: Forgejo did not
  answer within 300 ms\n`); `a_forge_that_goes_silent_after_its_answer_ends_the_run_at_the_deadline` (`ADMIN`, then
  nothing; deadline 200 ms: the line starts `...: forge 1: bot`, the run took under 400 ms);
  `a_later_request_that_fails_is_not_made_again` (`ADMIN`, then a 503: two requests heard, the line starts `...: forge 1: bot`);
- `a_refused_request_names_its_step_and_holds_no_secret` (rows: a 403 whose body repeats the token; a 401, a deleted
  `temper-admin` (setup.md 6.5): each line starts `...: forge 1: admin` and holds neither the token nor the body);
- `an_admin_token_readable_by_others_is_refused_naming_it` (`...: forge 1: admin_token: readable by others or reached
  through a link\n`); `each_file_is_read_within_its_own_bound` (`bound` rows: the three tokens `token_bytes`; `User` 21;
  `WebhookSecret` `hook_secret_bytes`); `io_short_of_the_http_reads_is_refused_before_any_request` (`intake` or `output`
  one below: `...: forge 1: io.intake: below the http client's largest read\n`, nothing heard);
- `exchange.rs`: `pause_doubles_from_the_backoff_to_its_cap` (reference, sketch E), `pause_ends_at_the_deadline` (40 ms left
  gives 40 ms; none left, `None`), `a_listing_ends_as_the_rows_it_gathered_in_order` (two `Row::Commit` steps, then
  `Document::Listed`: `Reply::Rows`);
- `crates/temper/tests/forge.rs`: `reconcile_refuses_a_tls_forge_naming_it` (a managed entry over a TLS transport),
  `reconcile_makes_a_missing_forge_directory_private` (a managed entry, nothing listening, `setup.deadline_ms` overridden
  to 300: the run fails at the deadline, `forge-1` is 0o700), `reconcile_refuses_a_forge_directory_open_to_others` (08.1's
  words, no request made), `reconcile_refuses_a_forge_the_setup_has_no_entry_for`.
Proof: process, `reconcile_converges_an_owned_forgejo_and_a_second_run_writes_nothing` (new, process.rs, on `owned::start`;
it runs the commands from the scratch with the document's relative paths, so where the files land shows Choice 5's
resolution): `temper forge reconcile config.json 1` exits 0 although Forgejo was started a moment before it; `forge-1/{api_token,
git_token,user}` are 0o600. Then 08.3's `Instance::attach` over `start`'s child (`into_child`; `Names`'s port, the admin
`Token::new` of `admin_token`) reads Forgejo by `instance.exchange(&Outgoing::get(..))` (`Outgoing` and its `get`, throwaway `lib.rs:84, :98`; instance.rs:195): the
API token answers `GET /api/v1/user` as `temper-bot` with the id `user` holds; as the admin, the bot is no site admin and
its full name holds `temper:bot`, org `acme` is private and has its team `temper-bot`. A second run exits 0 with the
three files' inode and mtime unchanged. Command: `sh -c 'export NEXTEST_DOUBLE_SPAWN=0; cargo build -p temper
--bin temper && cargo nextest run -p temper --profile process -E
"test(=reconcile_converges_an_owned_forgejo_and_a_second_run_writes_nothing)"'` ends `1 test run: 1 passed`, 60 s at most.
Don't: keep a connection across requests; make a later request again; name a route, a path or a Forgejo word in the host;
print on success; read `admin_token` in production code but here; hold a credential of its own, the bot's password, or
TLS; decide a step.
Size: 250 lines.
Removes: nothing.

### 10.2 The process tier on the owned Forgejo, with CI
Scope: setup.md 13.4, domain/testing.md 2.6, decisions 12 and 20. The throwaway's provisioning leaves the process tier: after
10.1's `owned::start`, the test starts `run runner`, runs `temper forge reconcile`, makes the person's token, and as the
person makes `acme/widget` in the org with Actions, `check.sh` and a workflow. The adoption leaves "It runs CI" checked,
the landing rule asks for it, and the change lands once the owned runner has passed its job.
Needs: 10.1 (`temper forge reconcile`, `owned::{start, run, reconcile}`); 08.3's `attach` is the session's.
Files: `crates/temper/tests/support/{forgejo,mod,person,owned}.rs` and `process.rs` (client-api hotspots, to about
10-18 20Z); no production file.
Shape:
1. `Throwaway::start` (forgejo.rs:16, one function as 08.3 left it) stops calling `create_user`, `create_repository`
   and `add_collaborator` (:21-24 at the base) and writing the bot's token and id files; `Throwaway` holds the attached
   instance, the runner's `Running`, the person's `Token` and the repository's id. In order: `owned::tier_configuration`
   (sketch D; its `owned::binaries()` panics naming a missing variable); 10.1's `owned::start` in place of 08.3's prepare
   and run, then 08.3's `attach` over its `into_child`; `owned::run(.., runner)`, killed in `stop` (:105) before Forgejo,
   and by its drop when a test panics; `owned::reconcile` (exit 0 within `WITHIN`, mod.rs:5). Each stage ends with
   `eprintln!("stage {name}: {ms} ms")` on a `Clock` (as live.rs:15 prints), which Question 1 reads.
2. The person, `ada` (the entry's): prepare made her with a random password, so Forgejo refuses her every request, under
   any token (403 `You must change your password`), until the admin clears it. Through the attached instance,
   `Instance::exchange(&Outgoing::json(..))` as `get` does (:36), under `instance.admin().token`, which is `admin_token`:
   `PATCH /api/v1/admin/users/ada` `{"login_name":"ada","source_id":0,"must_change_password":false}`, then `POST
   /api/v1/admin/users/ada/tokens` (`{"name":"person","scopes":["write:repository","write:organization"]}`; the answer's
   `sha1` is the token). As the person: `POST /api/v1/orgs/acme/repos` (`auto_init`, branch `main`, private), `PATCH
   /api/v1/repos/acme/widget` `{"has_actions":true}`, then `check.sh` (`test -s README.md`) and the workflow (sketch D) by
   `POST .../contents/<path>` on `main` (content in `skein_lib::base64`); 11.1's `push` is the same call. The workflow
   runs on pushes to `temper/**` alone, so these commits on `main` start no job (14's guide adapts). The repository's id
   is the creation's.
3. `Forge::Throwaway` (mod.rs:11) loses `token` and `user`. `forgejo::configuration` (:114) leaves the members setup.md 8
   derives to `owned::document` (`forge-1/api_token`, `{"file": "forge-1/user"}`, identity token `forge-1/git_token`,
   login `temper-bot`), changes the entry's `orgs` to `["acme"]` (08.3 names an org nobody makes), and flips the rule's
   `"ci":false` to `true` (:132), the rest as it is (`up_to_date` is required, `landing.rs:74`). `Throwaway::get` (:36) reads as the person (the repository is
   private, so every read is); `job_state(pull)` (new) reads the pull's head, then `commits/{head}/status`.
4. `person::adopt_and_change` (person.rs:233) drops its steps that press "It runs CI" (:244) and type the check
   (:245-248); the other process tests keep theirs.
5. `owned::tier_configuration(directory) -> SocketAddr` (sketch D) is the document of `Throwaway::start` apart from
   any process: forge 1 owned on a free loopback address, with a runner (`Names`' labels `temper-host:host` and its `path`,
   which holds git, base64 and sh: the workflow runs them), `acme`, `ada`, `temper-bot` and the rule above, the engine's
   side as `forgejo::configuration` writes it; `support::configuration` may run again over its directory
   (`create_dir_all`, as 08.3 has it). temper 13.1's proof calls it.
Test shape: no new file or kind; the existing tests, on the new start (`temper_open_adopts_a_repository_by_commands`,
process.rs:34, keeps its own unchecking of CI, :60). Reuse `Scratch` (mod.rs:43), `serve` (:83), `run_until_landed`
(:463), `landing_scripts` (forgejo.rs:218; its review script stays until the story proves it uncued); `configuration()`
keeps its profile overrides (:133-175). Ceiling: the story's referee, 180 s less its 5 s.
Proof: process, `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart` (process.rs:71), which now also
asserts `assert_eq!(forgejo.job_state(merged), "success")`: the merged pull request's head holds a `success` job, `main`
holds the edit, the restart finds the work; within 175 s on the story alone (31 s without CI at the base); and
`temper_open_adopts_a_repository_by_commands` (process.rs:34) on the same Forgejo, aiming at 30 s (13.1 s at the base):
a miss is noted in the landing report, read from nextest's `PASS [ t ]` line in the test log. Command: `sh -c
'export NEXTEST_DOUBLE_SPAWN=0; cargo build -p temper --bin temper && cargo nextest run -p temper --profile process -E
"test(=a_persons_change_lands_on_the_forge_and_is_found_after_a_restart) + test(=temper_open_adopts_a_repository_by_commands)"'`
ends `2 tests run: 2 passed`. The whole tier runs once, in README Done when 4's run.
Don't: keep any provisioning by `Instance` in `Throwaway::start`; read `admin_token` but for `attach`; sign in with a
password; start Forgejo or the runner but by the commands; turn webhooks on (temper 11); loosen the 180 s.
Size: 120 lines (the support's, decisions.md 22).
Removes: the throwaway's provisioning from the process tier, `token` and `user` of `Forge::Throwaway`, and `"ci":false`.

## Interface sketches

A and B are 10.1's, in the shell crate; `Entry` is 07.1's and `Reply`, `File`, `Failure` 09.1's. C is 10.1's, in the
binary; E is its reference tests; D is 10.2's.

```rust
// crates/temper-forge-setup-shell/src/exchange.rs
use skein_json::tokenizer;
use skein_lib::{Duration, Secret, Time};
use temper_forge_forgejo::{decode::Step, document::Row};
use temper_forge_setup::reconcile::Reply;

/// The `Authorization` value of every request, `token <t>` (setup.md 5); it holds a secret.
#[must_use]
pub fn authorization(token: &Secret) -> Secret { todo!() }
/// What the exchange does after a decoder's step.
#[expect(clippy::large_enum_variant, reason = "the reply moves whole, as the store's records do")]
pub enum Next {
    /// Feed the tokenizer this demand.
    Demand(tokenizer::Request),
    /// The exchange is over: the machine's reply.
    Reply(Reply),
}
/// Gathers one answer's rows, kept in `rows`.
#[derive(Default)]
pub struct Gather { pub rows: Vec<Row> }
impl Gather {
    /// A demand goes on, a row is kept; `Done(Listed)` ends as the rows, another document as itself, a malformed one as `Down`.
    pub fn take(&mut self, step: Step) -> Next { todo!() }
}
/// The wait before the first request is made again (setup.md 5): doubling from `backoff` to `backoff_max`,
/// never past `deadline`.
#[must_use]
pub fn pause(
    backoff: Duration, backoff_max: Duration, deadline: Duration, attempt: u32, began: Time, now: Time,
) -> Option<Duration> {
    let doubled = backoff.saturating_mul(1_u64.checked_shl(attempt).unwrap_or(u64::MAX));
    let left = deadline.as_nanos().checked_sub(now.saturating_since(began).as_nanos()).filter(|left| *left > 0)?;
    Some(Duration::from_nanos(doubled.as_nanos().min(backoff_max.as_nanos()).min(left)))
}
```
```rust
// crates/temper-forge-setup-shell/src/reconcile.rs
use skein_lib::Duration;
use skein_shell::Clock;
use std::{net::SocketAddr, path::PathBuf};
use temper_forge_setup::{place::File, reconcile::Failure, setup::Entry};

/// What `temper forge reconcile` hands the host, from the document and the profile.
pub struct Settings {
    /// The forge's number (every message), and where Forgejo answers: loopback, plaintext.
    pub forge: u16, pub address: SocketAddr,
    /// `Host`'s value (`conventions::host`) and the `User-Agent`.
    pub host: Box<[u8]>, pub user_agent: &'static str,
    /// The forge's directory (absolute), the admin's token file, and the bounds of the tokens and of the webhooks' secret.
    pub directory: PathBuf, pub admin_token: PathBuf, pub token_bytes: u32, pub hook_secret_bytes: u32,
    /// `setup.deadline_ms`, `setup.backoff_ms`, `setup.backoff_max_ms`.
    pub deadline: Duration, pub backoff: Duration, pub backoff_max: Duration,
    /// The profile's `limits::http`, `limits::documents` of `forge_component`, and jig's `io`.
    pub http: skein_http::client::Limits, pub documents: temper_forge_forgejo::limits::Limits, pub io: skein_io::Limits,
    /// The forge's entry (the machine's input, 09.1), and `application.webhooks`' listener when there is one.
    pub entry: Entry, pub listener: Option<SocketAddr>,
}
/// Why a run did not converge.
#[derive(Debug, PartialEq, Eq)]
pub enum Refused {
    /// The first request got nothing but a failure, a 5xx or no answer within `deadline`.
    Unanswered { forge: u16, deadline: Duration },
    /// The io limits are short of the http client's reads (`name` is `intake` or `output`); nothing was requested.
    Limits { forge: u16, name: &'static str },
    /// A file of the forge's directory unread, unsafe, oversize or not written whole.
    File { forge: u16, file: File, why: String },
    /// The machine stopped at a step.
    Stopped(Failure),
}
impl Refused {
    /// The one stderr line, `temper forge reconcile: forge N: <words>\n`, with no secret; a stop's words are
    /// `<stage> <subject>: <why>`.
    #[must_use]
    pub fn line(&self) -> String { todo!() }
}
/// The bound a file is read within: the tokens' `token_bytes`, `User` 21, `WebhookSecret` `hook_secret_bytes`.
#[must_use]
pub fn bound(settings: &Settings, file: File) -> u32 { todo!() }
/// Runs the machine to its end on a `Kernel` and `drive`; a success writes nothing.
pub fn reconcile(settings: Settings, clock: &Clock) -> Result<(), Refused> { todo!() }
```
```rust
// crates/temper/src/forge.rs (08.1's; 10.1 adds the arm)
use temper_forge_setup::check::Checked;
use temper_forge_setup_shell::reconcile::Settings;
use temper_service::startup::Named;

/// `reconcile`'s settings for forge `number` from the checked document; `Err` is the refusal's words (TLS among them).
pub fn reconcile_settings(named: &Named, checked: &Checked, number: u16) -> Result<Settings, String> { todo!() }
```
D. 10.2's, in the process tier's support, and the workflow the test puts on `main` (YAML, `run:` steps only: no `uses:`, no
network; it runs on a push to `temper/**` alone, so a commit on `main` starts nothing; the clone is 03.3's fact
`runner-a-jobs-token-clones-its-private-repository`: the sketch's `http.extraHeader` is `actions/checkout`'s, and the
fact's wins if it differs).
```rust
// crates/temper/tests/support/owned.rs (08.1's file; 10.2 adds both)
use std::{net::SocketAddr, path::{Path, PathBuf}};

/// The pinned Forgejo's and runner's paths, from `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER`; panics naming the one
/// that is missing, before anything starts.
#[must_use] pub fn binaries() -> (PathBuf, PathBuf) { todo!() }
/// Writes the process tier's whole owned document into `directory` (the engine's side as `forgejo::configuration`
/// writes it, the binaries from `binaries()`) on a free loopback address, which it returns; starts nothing.
#[must_use] pub fn tier_configuration(directory: &Path) -> SocketAddr { todo!() }
```
```rust
// crates/temper/tests/support/forgejo.rs
impl Throwaway {
    /// The state the pull request's head holds for its job (`success` when the owned runner passed it).
    #[must_use] pub fn job_state(&self, pull: u64) -> String { todo!() }
}
```
```yaml
# .forgejo/workflows/check.yaml
on: { push: { branches: ['temper/**'] } }
jobs:
  check:
    runs-on: temper-host
    steps:
      - run: |
          auth=$(printf 'x-access-token:%s' "${{ github.token }}" | base64 -w0)
          git -c "http.extraHeader=Authorization: basic $auth" clone "${{ github.server_url }}/${{ github.repository }}.git" .
          git checkout "${{ github.sha }}" && sh check.sh
```
E. 10.1's reference tests, in the shell crate.
```rust
// crates/temper-forge-setup-shell/tests/reconcile.rs
pub mod support;
use skein_shell::Clock;
use support::Peer;
use temper_forge_setup_shell::reconcile::reconcile;
#[test]
fn a_first_request_is_made_again_until_the_api_answers() {
    let peer = Peer::reserve();
    let settings = support::settings(peer.address());
    let heard = std::thread::scope(|scope| {
        let server = scope.spawn(|| peer.serve_after(120, &[(403, br#"{"message":"no"}"#)]));
        let refused = reconcile(settings, &Clock::new()).expect_err("a 403 stops the run");
        assert!(refused.line().starts_with("temper forge reconcile: forge 1: admin"), "{}", refused.line());
        server.join().expect("peer thread")
    });
    assert_eq!(heard.requests.len(), 1, "refused connections are not requests");
    assert_eq!(heard.requests[0].line, "GET /api/v1/user HTTP/1.1");
}
```
```rust
// crates/temper-forge-setup-shell/tests/exchange.rs
use skein_lib::{Duration, Time};
use temper_forge_setup_shell::exchange::pause;
#[test]
fn pause_doubles_from_the_backoff_to_its_cap() {
    let (backoff, cap, deadline) = (Duration::from_millis(100), Duration::from_secs(5), Duration::from_secs(120));
    for (attempt, wanted) in [(0, 100), (3, 800), (6, 5000), (200, 5000)] {
        let waited = pause(backoff, cap, deadline, attempt, Time::from_nanos(0), Time::from_nanos(0));
        assert_eq!(waited, Some(Duration::from_millis(wanted)), "attempt {attempt}");
    }
}
```

## Questions to raise when reached

1. **The story past 180 s** (decisions 12): report each stage's time (prepare, start, reconcile, the job) from the
   `stage <name>: <ms> ms` lines of the story run with `--no-capture` (10.2's command with `--no-capture` after `--profile
   process` and the story alone in `-E`) and its `PASS [ t ]` line; the CI story becomes a test of its own by a ruling; no
   bound is raised.

## Done when

- `scripts/check.sh` exits 0.
- README Done when 4's run (the whole tier, once) passes this session's three process tests; with `$LOG` the path the command
  ends that run with, `grep -c -E 'PASS .*(reconcile_converges_an_owned_forgejo_and_a_second_run_writes_nothing|a_persons_change_lands_on_the_forge_and_is_found_after_a_restart|temper_open_adopts_a_repository_by_commands)' "$LOG"`
  prints 3.
- `grep -c -e create_user -e create_repository -e add_collaborator crates/temper/tests/support/forgejo.rs`,
  `grep -c 'It runs CI' crates/temper/tests/support/person.rs` and `grep -c "token: &" crates/temper/tests/support/mod.rs`
  print 0; `grep -c '"ci":true' crates/temper/tests/support/forgejo.rs` prints 1.
- `! git grep -n -e base64 -e Basic -- crates/temper-forge-setup-shell/src/reconcile.rs
  crates/temper-forge-setup-shell/src/exchange.rs` exits 0 (no basic auth in the host).
- `grep -c 'temper forge reconcile CONFIG.json FORGE' crates/temper/src/lib.rs` prints 1 (the usage line).
