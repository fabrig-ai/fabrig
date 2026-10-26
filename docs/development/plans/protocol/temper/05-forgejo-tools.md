# Session 05: the throwaway Forgejo, its transcripts and facts, git's transcripts

temper gets its throwaway Forgejo and the two tools that record what a
real Forgejo v16.0.5 and real git 2.53 say. `testing/temper-throwaway-forgejo`
starts the binary `TEMPER_FORGEJO` names, refusing any other version, on
loopback with SQLite in a scratch directory, makes its users, tokens and
repositories, and removes the instance and its scratch directory at the end; the tools and the process
tier (10.4, 15.3) use it. `tools/forgejo-conformance`, in Rust, replaces
the Python tool: it sends the requests of forge.md 3.2 and 3.5 and the
failures of 3.4 as plain HTTP/1.1, keeps each exchange as a masked
transcript, and writes forge.md 13's facts as `holds`, `fails` or
`unchecked` lines, the Actions ones with a forgejo-runner. `tools/git-transcripts`
runs each invocation of git.md 4 and each failure of git.md 6 against
the throwaway Forgejo, with git.md 3's settings and 5's environment, and
keeps argv, exit, stdout and stderr, masked. Everything here is ordinary
Rust outside the suites' timed tests; each tool runs once, by its Proof,
and its output is committed. Sessions 06 (Forgejo's documents), 07 (the
git component), 14 (the fake forge) and 17 (the fake git program) read
the transcripts as files.

## Read first

- forge.md, sections 3 (all), 10 ("The real Forgejo check") and 13.
- git.md, sections 3 to 6 and 9.
- domain/testing.md, 2.6 (the throwaway Forgejo, "the test's support").
- findings/temper-misc.md, section 3 (what `run.py` and `next_domain.py`
  do: app.ini, users, tokens, the runner's registration and
  configuration; where they and the design disagree).
- README.md, "Rules of this plan", the throwaway Forgejo's rule;
  code-organization.md, sections 1 (the environment variables), 2 (the
  three crates' rows), 5 ("Transcripts are read as files", "Scratch
  directories") and 6.
- The code: `tools/forgejo-conformance/run.py` (212 lines) and
  `next_domain.py` (200), which 05.2 removes; `tests/clippy.toml` and
  `tests/engine/forge/client/Cargo.toml` (the `[lints]` table), the
  precedents of the three crates' lints.

## What the design asks

- **The binary, and only it** (forge.md, 10): "Its binary is the one
  `TEMPER_FORGEJO` names, as the process tier's is (domain/testing.md, 2.6): the
  tool names no path of its own, refuses to run without the variable,
  and refuses a binary whose `--version` is not v16.0.5, naming it; it
  records the binary's digest with what it captured."
- **The instance** (forge.md, 10; domain/testing.md, 2.6): "SQLite in a scratch
  directory, loopback, a small `MAX_RESPONSE_ITEMS` so paging shows ...;
  the tool makes its admin user and tokens; never an existing Forgejo."
  The process tier's support "starts it, makes the admin user, temper's
  forge user, their tokens and a repository, and stops and removes the
  instance after."
- **Transcripts** (forge.md, 10): "format-faithful, content-irrelevant.
  Ids, times, counts and text vary; keys, nesting, types and status lines
  must match. Volatile fields are masked before comparing; tokens and
  secrets never reach a file." decisions.md, 10: "raw bytes, one
  directory per case: Forgejo's as `request` and `response` (HTTP/1.1 as
  sent and received, masked), git's as `argv`, `exit`, `stdout` and
  `stderr`."
- **Facts** (forge.md, 10 and 13): the tool "drives the facts of section
  13 through plain requests ..., says which hold, and records what it
  sent and received as transcripts ... Actions facts need a runner:
  `forgejo-runner`, the program `TEMPER_FORGEJO_RUNNER` names, registered
  with the throwaway instance alone; without it the tool reports those
  facts unchecked." (Driving them through the forge component is 14.5's.)
- **git's transcripts** (git.md, 9): "every invocation of section 4, and
  every failure of section 6 (a missing repository, branch and commit, a
  wrong token, no permission, a protected branch, a refused connection),
  against the throwaway Forgejo ..., which it starts as
  `tools/forgejo-conformance` does. Each keeps the argument vector, exit
  status, stdout and stderr, masked: object ids, ports, paths and times.
  No credential is ever written." And two checks of forge.md 13: whether
  Forgejo redirects git for a renamed repository (git.md, 10), and the
  line git prints for a push a protected branch refuses (git.md, 4.8).

## Choices this plan makes

- **One support crate, two tools, each a library and a binary.**
  `temper-throwaway-forgejo` owns the binary's check, the instance's
  life, provisioning, and a plain HTTP/1.1 client (one connection per
  request, `Connection: close`), so the tools and the process tier share
  them. Each tool is `src/lib.rs` with its modules and a `src/main.rs`
  of at most 30 lines, so a world (17.1) can call its masking.
- **The instance's environment is given whole:** `HOME` (`<scratch>/home`),
  `PATH=/usr/bin:/bin`, `TZ=Asia/Kolkata`, `GIT_CONFIG_NOSYSTEM=1`,
  `GIT_CONFIG_GLOBAL=<scratch>/gitconfig`; nothing of the caller's. The
  zone is fixed at +05:30, all year, so Forgejo's times carry the same
  offset on every machine, and it is not zero. git for Forgejo is the
  path `Settings::git` names, written as app.ini's `[git] PATH`.
- **app.ini** is the planner's smoke check's, which started v16.0.5
  (`APP_NAME`, `WORK_PATH`, `[server]` on `127.0.0.1` with `ROOT_URL`,
  `APP_DATA_PATH`, `DISABLE_SSH`, `OFFLINE_MODE`, `[database]` sqlite3,
  `[repository]` `ROOT` and `DEFAULT_BRANCH = main`, `[security]`
  `INSTALL_LOCK` and two 32-byte random hex secrets from `/dev/urandom`,
  `[service] DISABLE_REGISTRATION`, `[log] MODE = console`, `LEVEL =
  Error`), plus `[git] PATH`, `[actions] ENABLED` as `Settings::actions`
  says, and `[api] MAX_RESPONSE_ITEMS` only when `Settings` gives one.
  `RUN_USER` is the login that `/etc/passwd` gives the uid owning
  `/proc/self`: no environment variable is read for it
  (code-organization.md, 1, "No other variable is read").
- **The `--version` check:** the first line of `<binary> --version` (run
  in the environment above) must begin `forgejo version 16.0.5+` or be
  `forgejo version 16.0.5`; v16.0.5 printed `forgejo version
  16.0.5+gitea-1.22.0 (release name 16.0.5) built with GNU Make 4.4.1,
  go1.26.8 : bindata, timetzdata, sqlite, sqlite_unlock_notify` (the
  writer's one run, 2026-10-14). The digest is SHA-256 of the binary's
  bytes, by `skein_io::digest::digest`, written as 64 lowercase hex
  digits.
- **Users:** `fixture` is made by `forgejo admin user create --admin`
  before `web` starts (as `run.py` did); the others by the admin API
  (`POST /api/v1/admin/users`) once it serves; tokens by `POST
  /api/v1/users/{login}/tokens` with Basic authentication and
  `{"name":"throwaway","scopes":["all"]}` (the planner's smoke check got
  a token this way on v16.0.5). Passwords are 32 random bytes in hex.
- **temper's identity in the recorded cases is `writer`,** a write
  collaborator who is not an admin, as a deployment's forge user is; a
  case names `fixture` (the admin and owner) or `stranger` (no access)
  where the case is about them.
- **Transcripts' bytes** (decision 10): `request` is the request as the
  tool sent it, its `Authorization` value written `token MASKED`;
  `response` is the status line and the header lines as received, but
  `Transfer-Encoding` dropped and `Content-Length` set to the masked
  body's length, then the body de-chunked and masked. Forgejo sends
  larger JSON answers chunked (`/repos/...` was, the writer's run), and
  consumers read the body after the blank line with no parser of
  chunks.
- **The masking,** on bytes, one map for the whole run, in case order,
  the same rule in both tools (each in its `mask.rs`):
  - `127.0.0.1:<port>` of the instance becomes `127.0.0.1:3000`;
  - a run of exactly 40 or 64 lowercase hex digits (an object id, an
    `Etag`) becomes its order of first appearance, from 1, in lowercase
    hex zero-padded to the same length;
  - an RFC 3339 date-time inside a JSON string whose year is 2000 to
    2999 becomes `2020-01-01T00:00:00` plus its order of first
    appearance in seconds, its offset kept as written; Forgejo's zero
    times (`0001-...`, `1970-...`) are kept;
  - the values of `Date` and `Last-Modified` become `Wed, 01 Jan 2020
    00:00:00 GMT`;
  - git's: the scratch directory's absolute path becomes `/scratch`.
  - **Ids are kept:** a fresh instance numbers users, repositories,
    issues, comments and reviews in the tool's fixed order, so they are
    the same at every run (the skeleton's "replaces ids" is dropped: a
    masked id would also have to be followed into paths).
  - **No secret is written:** before writing a file, the tool searches
    its bytes for each password, API token, registration token and git
    header value the run made (and the base64 of `<login>:<token>`); one
    found stops the run, naming the case and the secret's kind, with
    nothing more written.
- **Targets and bodies as temper's client writes them,** so session 06's
  encoders meet them byte for byte: each name in a path and each query
  value with RFC 3986's unreserved bytes kept and every other byte
  `%XX` in uppercase hex; query parameters in forge.md 3.2's order;
  request bodies compact JSON with their keys in 3.2's order (`{title,
  body}`, `{head, base, title, body}`, `{event, body}`, ...), written by
  the tool's own writer, not `serde_json` (whose maps sort their keys).
  A `since` is written `YYYY-MM-DDTHH:MM:SSZ`; the recorded cases use
  only `1970-01-01T00:00:00Z` and `2554-07-21T23:34:33Z`, so no request
  carries a time that varies.
- **One run records transcripts and facts:** `forgejo-conformance
  --scratch <dir> --out <dir>` records the cases, then evaluates the
  facts (05.3), many of them from the cases' recorded statuses and
  messages. A case directory it records is replaced; one it does not
  record (the Actions cases without a runner) is left as it is.
- **Scratch directories** are arguments, never under `/tmp`: the Proofs
  pass `target/forgejo-conformance` and `target/git-transcripts` in the
  session's worktree; `start` refuses a scratch path that exists and is
  not empty, or that begins with `/tmp`.
- **No webhooks here:** standalone polls (forge.md, 5); the webhook
  facts are `unchecked ...: webhooks are built after standalone`.

## Rules that bind it

- "No increment of this plan ... starts, stops, configures or reads the
  user's own Forgejo (`~/.local/bin/forgejo`, `~/.local/bin/forgejo-server`,
  which is Forgejo 7.0.12, `~/.local/state/forgejo`), or any Forgejo but
  the one `TEMPER_FORGEJO` names, whose `--version` must be 16.0.5"; nor
  "writes under `/tmp`"; nor "writes Python" (code-organization.md, 6).
- "**The process and live tiers run only where a Proof line or 'Done
  when' says,** once; never in the gate"
  (README, "Rules of this plan"). Every test here that starts Forgejo is
  `#[ignore = "needs TEMPER_FORGEJO"]`.
- "**Tools and test support** may link `serde_json` (decisions.md, 10);
  no step crate does. No crate adds any other dependency from outside
  skein, smith and jig", and "an increment that adds a crate of jig,
  smith or skein runs the three `cargo update --precise` commands"
  (code-organization.md, 3): 05.1 adds `skein-io` and `serde_json`.

## Increments

### 05.1 `temper-throwaway-forgejo`: the binary checked, an instance started, provisioned and removed

Scope: `testing/temper-throwaway-forgejo`, ordinary Rust (domain/testing.md,
2.6; forge.md, 10): `Forgejo::from_env()` and `Forgejo::from_path(path)`
(the version check and the digest, as "Choices" says); `Forgejo::start(scratch,
&Settings)` (app.ini, `migrate`, the admin `fixture` and its token,
`web` with stdout and stderr to `<scratch>/server.log`, ready once `GET
/api/v1/version` answers `200`, polled every 50 ms at most
`Settings::ready_polls` times, a child that exits meanwhile failing the
start with the log's last 2,000 bytes); users, tokens, repositories and
collaborators; `Instance::exchange`, the plain HTTP/1.1 client;
`Instance::stop` (the child killed and waited for, the scratch removed);
`Drop` doing the same when `stop` was not called.
Needs: nothing.
Files: `testing/temper-throwaway-forgejo/{Cargo.toml, clippy.toml,
src/{lib.rs, binary.rs, instance.rs, http.rs, tests.rs},
tests/throwaway.rs}`; the workspace `Cargo.toml` (member,
`[workspace.dependencies]` `temper-throwaway-forgejo`, `skein-io`,
`serde_json = "=1.0.154"`) and `Cargo.lock`. Shared hotspots: the
workspace `Cargo.toml`, `Cargo.lock`.
Shape: as the interface sketch (ordinary Rust). `clippy.toml` is
`tests/clippy.toml`'s; the `[lints]` table is
`tests/engine/forge/client/Cargo.toml`'s. Dependencies: `skein-io` (the
digest), `serde_json` (ids and shas read from answers). `exchange`
writes `Host`, `Authorization: token <t>` when a token is given,
`Content-Type: application/json` and `Content-Length` with a body,
`Range` when given, and `Connection: close`; it reads the answer whole,
a `Content-Length` or chunked body, within `Settings::answer_bytes`.
Test shape: unit tests in `src/tests.rs`:
`the_app_ini_names_loopback_sqlite_and_the_page_cap` (app.ini rendered
with fixed secrets and port, against a literal),
`a_chunked_answer_is_read_whole_and_unframed`,
`a_recorded_request_holds_no_token`. Integration tests in
`tests/throwaway.rs`, scratch under `env!("CARGO_TARGET_TMPDIR")`:
`a_binary_printing_another_version_is_refused_naming_it` (a `sh` script
the test writes, printing `forgejo version 15.0.0+gitea-1.22.0`),
`a_scratch_directory_that_is_not_empty_is_refused`, and, ignored,
`a_throwaway_forgejo_starts_serves_its_admin_and_is_removed`. Each
focused test under 20 ms; the ignored one under 30 s. Reference test,
in full:

```rust
#[test]
#[ignore = "needs TEMPER_FORGEJO"]
fn a_throwaway_forgejo_starts_serves_its_admin_and_is_removed() {
    let forgejo = Forgejo::from_env().expect("TEMPER_FORGEJO names Forgejo 16.0.5");
    let scratch = Path::new(env!("CARGO_TARGET_TMPDIR")).join("throwaway-start");
    let mut instance = forgejo.start(&scratch, &Settings::small()).expect("started");
    let admin = instance.admin().clone();
    let user = instance.exchange(&Outgoing::get("/api/v1/user", Some(&admin.token))).expect("answered");
    assert_eq!(user.status, 200, "the admin's token reads its user");
    let made = instance.create_repository(&admin, "specimen", Init::Readme).expect("made");
    assert_eq!(made.name, "specimen");
    instance.stop().expect("stopped");
    assert!(!scratch.exists(), "the scratch directory is removed");
}
```

Proof: process (a real Forgejo), `TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v16.0.5/forgejo-16.0.5-linux-amd64
cargo nextest run -p temper-throwaway-forgejo --profile
measure --run-ignored only` (the default profile's 15 s budget would
stop it), run once: Forgejo 16.0.5 starts on loopback in a scratch
directory, its admin's token reads `/api/v1/user`, a repository is
made, and the scratch is gone after `stop`; under 30 s. Before it, `cd
~/src/rust/tmp/temper/forgejo/v16.0.5 && sha256sum -c
forgejo-16.0.5-linux-amd64.sha256` prints `OK` (README).
Don't: name a Forgejo path in code (the variable only); read any
environment variable but `TEMPER_FORGEJO`; spawn a thread; send `SIGTERM`
through a shell (`Child::kill`, then `wait`).
Size: at most 400 lines.
Removes: nothing.

### 05.2 `tools/forgejo-conformance` in Rust: the transcripts

Scope: the tool (forge.md, 10; decision 10): provisions the fixture
below, sends each case's request through `Instance::exchange`, and
writes `<out>/transcripts/<case>/{request,response}` masked, and
`<out>/transcripts/binary.txt` (two lines: `version <the --version
line>`, `sha256 <hex>`). `run.py` and `next_domain.py` go; `README.md`
is rewritten (at most 60 lines: how to run, the layout, the masking).
Needs: 05.1's instance.
Files: `tools/forgejo-conformance/{Cargo.toml, clippy.toml, README.md,
src/{lib.rs, main.rs, fixture.rs, cases.rs, mask.rs, write.rs,
tests.rs}, transcripts/}`; the workspace `Cargo.toml` (member).
Shared hotspot: the workspace `Cargo.toml`.
Shape: `forgejo-conformance --scratch <dir> --out <dir>`;
`mask::Mask` (the run's maps) with `mask::Mask::bytes(&mut self, &[u8])
-> Vec<u8>`, `mask::response(&mut Mask, &Exchange) -> Vec<u8>`,
`mask::request(&mut Mask, &Exchange) -> Vec<u8>`; `write::check(case,
request, response, &Secrets) -> Result<(), Leak>`, which
`write::case(out, case, request, response, &Secrets)` runs before it
writes anything. The fixture,
made by `fixture.rs`, in this order: `MAX_RESPONSE_ITEMS = 2`, Actions
on; users `fixture` (admin), `writer`, `stranger`; repositories
`fixture/specimen` (initialized, `main`), `fixture/private` (private),
`fixture/archived`, `fixture/moved`, `fixture/narrow` (squash merges
refused); `writer` a write collaborator of `specimen`, `archived` and
`narrow`; on `specimen`: a label `temper / label`, three files on a
branch `feature/slash` (contents API), a pull request `feature/slash`
into `main` opened by `fixture`, a branch `conflict` whose change
conflicts with `main`'s, and a protection rule on `main` (status check
`fixture/test`, one approval, stale approvals dismissed).
The cases, each one request, recorded in this order (`R` is
`/api/v1/repos/fixture/specimen`, the token `writer`'s unless named):
- start and resolution: `user`, `user-fixture` (`fixture`'s token),
  `settings-api`, `repository-by-id`, `repository-by-path`,
  `users-search-uid` (`?uid=<fixture's id>&limit=1`);
- writes that make the reads' state: `create-issue` (body behind a key's
  marker), `post-comment` (likewise), `post-comment-plain`,
  `edit-issue-title`, `open-pull` (`writer`'s own branch), `post-review-approved`
  and `post-review-request-changes` (on `fixture`'s pull, marker
  bodies), `post-review-comment`, `request-reviewers`,
  `remove-reviewers`, `post-status` (`fixture/test` success),
  `post-status-pending`, `create-branch`, `create-branch-exists`,
  `close-issue`, `reopen-issue`, `update-pull`, `merge-pull`,
  `delete-branch`;
- reads: `issues-page-1` (`state=all&since=1970-01-01T00:00:00Z&sort=leastupdate&page=1&limit=2`),
  `issues-page-2`, `issues-type-issues`, `issues-type-pulls`,
  `issues-since-2554`, `issue`, `issue-of-pull`, `issue-comments`,
  `pull`, `pull-merged`, `pull-for-encoded-head` (`R/pulls/main/feature%2Fslash`),
  `combined-status`, `combined-status-empty`, `statuses-page-2`,
  `reviews`, `review-comments`, `branch`, `branch-with-slash`,
  `branches-page-1`, `branches-page-2`, `branch-protection` (as
  `fixture`), `branch-protections` (as `fixture`), `collaborators`,
  `collaborator-permission`, `owner-permission`, `pull-files-page-1`,
  `pull-files-page-2` (each with `X-HasMore`), `compare`,
  `compare-reverse`, `raw-file`, `raw-file-range` (`Range:
  bytes=0-3`);
- failures (forge.md, 3.4): `wrong-token` (40 zeros), `no-token`,
  `missing-issue`, `raw-file-missing`, `writer-branch-protection`,
  `writer-branch-protections`, `stranger-repository-by-path`,
  `stranger-repository-by-id`, `moved-repository` (the old path after a
  rename), `archived-write`, `open-pull-exists`, `open-pull-no-changes`,
  `create-issue-empty-title`, `merge-stale-head`, `merge-closed`,
  `merge-conflict`, `merge-protected`, `merge-style-refused`
  (`narrow`), `update-conflict`, `delete-protected-branch`,
  `delete-missing-branch`.
A case whose status is not the one its name implies is still recorded;
05.3's facts read it.
Test shape: unit tests in `src/tests.rs`, inputs as literals:
`masking_rewrites_ports_ids_times_and_dates_by_first_appearance` (a
table: each rule a row, and one run of two cases sharing an id),
`a_chunked_response_is_written_unframed_with_its_masked_length`,
`a_secret_in_a_case_stops_the_run_naming_the_case`,
`targets_encode_every_byte_but_unreserved_ones`. Each under 20 ms.
Reference test, in full:

```rust
#[test]
fn a_secret_in_a_case_stops_the_run_naming_the_case() {
    let secrets = Secrets::of(&[(SecretKind::ApiToken, "0123456789abcdef0123456789abcdef01234567")]);
    let leaked = write::check(
        "user",
        b"GET /api/v1/user HTTP/1.1\r\nAuthorization: token MASKED\r\n\r\n",
        b"HTTP/1.1 401 Unauthorized\r\n\r\n{\"message\":\"token 0123456789abcdef0123456789abcdef01234567\"}",
        &secrets,
    );
    assert_eq!(leaked, Err(Leak { case: "user".to_owned(), kind: SecretKind::ApiToken }));
}
```

Proof: process (a real Forgejo), `TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v16.0.5/forgejo-16.0.5-linux-amd64
cargo run -p forgejo-conformance -- --scratch
target/forgejo-conformance --out tools/forgejo-conformance`, run once:
exit 0; then `ls -d tools/forgejo-conformance/transcripts/*/ | wc -l`
prints 77 (the cases above), `grep -L '^HTTP/1.1 [1-5][0-9][0-9] '
tools/forgejo-conformance/transcripts/*/response` prints nothing, and
`grep -rlE 'Authorization: token [0-9a-f]{40}' tools/forgejo-conformance/transcripts`
prints nothing; under 2 minutes. The transcripts are committed.
Don't: record webhooks; mask JSON with `serde_json` (bytes only, so the
shape cannot change); write a transcript of the provisioning requests
the case list does not name.
Size: at most 550 lines.
Removes: `tools/forgejo-conformance/run.py`, `next_domain.py`.

### 05.3 Forgejo's facts, and the Actions transcripts

Scope: after the cases, the same run evaluates forge.md 13's facts and
writes `<out>/facts.txt`: a first line `# forgejo <the --version line>
sha256 <hex>`, a second `# runner <its --version line>` or `# runner
none`, then one line per fact below, in this order: `holds <fact>`,
`fails <fact>: <observed>` (the status, message or value seen, one
line) or `unchecked <fact>: <why>`. With `TEMPER_FORGEJO_RUNNER`, the
runner (its `--version` must contain `12.13.2`, else the run stops
naming it) is registered with the instance alone, a workflow pushed, and
the Actions cases recorded: `actions-combined-status`, `actions-runs`
(`?head_sha&run_number&page=1&limit=1`), `actions-run`, `actions-jobs`,
`actions-job-log`, `actions-job-log-range`.
Needs: nothing beyond 05.2.
Files: `tools/forgejo-conformance/src/{facts.rs, runner.rs, tests.rs}`,
`tools/forgejo-conformance/{facts.txt, transcripts/actions-*}`,
`README.md` (at most 10 more lines).
Shape: `facts::Fact { name: &'static str, outcome: Outcome }`, `enum
Outcome { Holds, Fails(String), Unchecked(String) }`, `facts::write(out,
&[Fact])`. The runner (`runner.rs`): its environment given whole
(`HOME=<scratch>/runner/home`, `PATH=/usr/bin:/bin`); the token from
`GET R/actions/runners/registration-token` (`200` with `{"token":...}`
on v16.0.5, the writer's run); `forgejo-runner --config <scratch>/runner/config.yaml
register --no-interactive --instance <base> --token <t> --name
temper-conformance --labels temper-host:host`, then `daemon`, killed and
waited for at the end; its configuration `next_domain.py:103-117`'s keys
(`runner.file`, `capacity: 1`, `timeout: 1m`, `fetch_interval` and
`report_interval` 100ms, `labels`, `cache.enabled: false`,
`host.workdir_parent`), checked against `forgejo-runner
generate-config` of v12.13.2 once (a key it lacks is a question). The
workflow `.forgejo/workflows/conformance.yaml`, `on: push`, two jobs on
`temper-host`: `pass` (`exit 0`) and `fail` (prints
`TEMPER_CONFORMANCE_FAILED_JOB`, `exit 1`), no checkout and no action.
The wait for both jobs' statuses: the combined status read every 500 ms,
at most 120 times. The facts (their names are the interface 14.5
updates), each checked by the case or the requests named:
- confirmed from v15: `api-default-page-cap-50-and-30` (a second instance,
  no `[api]`, read `/settings/api`), `pull-for-accepts-encoded-slash`,
  `update-conflict-is-409-merge-failed`, `merge-old-head-is-409-head-out-of-date`,
  `merge-answers-200-with-no-body`, `pull-files-page-with-x-hasmore`,
  `compare-ignores-page-and-limit`, `create-branch-at-full-commit-id`,
  `writer-protection-reads-are-403`;
- the listing: `leastupdate-orders-least-recently-updated-first`,
  `leastupdate-ties-ordered-by-number`, `since-is-inclusive-at-the-second`,
  `since-in-2554-answers-an-empty-page-with-date`;
- an item's updated time: `updated-moves-on-comment`,
  `updated-stays-on-comment-edit`, `updated-stays-on-comment-delete`,
  `updated-moves-on-review`, `updated-moves-on-head-push`,
  `updated-stays-on-head-status`, `updated-stays-on-base-push`,
  `updated-moves-on-close`, `updated-moves-on-reopen` (domain/testing.md, 4.2's
  rule, which the fake keeps);
- bodies and times: `unedited-comment-updated-equals-created`,
  `unedited-review-updated-equals-submitted`, `edit-moves-comment-updated`,
  `marker-line-feed-kept`, `body-of-65536-bytes-kept-whole`,
  `body-of-1048576-bytes-kept-whole`;
- pull requests: `patch-issue-edits-a-pull-request`,
  `pull-for-names-the-newest-of-several`, `branch-path-takes-encoded-slash`,
  `base-sha-is-the-base-tip`;
- statuses: `combined-status-without-status-counts-0`,
  `skipped-and-success-combine-as-passed`;
- order and shape: `comments-come-oldest-first`,
  `reviews-come-oldest-first`, `dismissed-review-keeps-its-body`,
  `remark-position-is-its-line`, `raw-range-answers-206`;
- forge.md 3.4's statuses and messages, from the failure cases:
  `merge-closed-is-405`, `merge-conflict-is-405`, `merge-protected-is-405`,
  `merge-style-is-405`, `open-pull-exists-is-409`,
  `open-pull-no-changes-is-422`, `empty-title-is-422`,
  `moved-repository-is-a-redirect`, `archived-write-is-423`,
  `delete-protected-branch-is-403`, `delete-missing-branch-is-404`,
  `create-branch-exists-is-409`, `unreadable-repository-by-path-is-404`,
  `unreadable-repository-by-id-is-404`;
- protection: `effective-protection-name-for-a-writer`,
  `effective-protection-name-for-an-admin`;
- rate: `no-rate-limit-headers` (no `X-RateLimit-*` field on any
  recorded answer);
- Actions (runner given, else `unchecked <fact>: TEMPER_FORGEJO_RUNNER
  not set`): `actions-report-each-job-as-a-status-on-its-head`,
  `actions-status-link-is-runs-n-jobs-i`, `actions-status-link-names-no-attempt`,
  `run-number-is-index-in-repo`, `run-jobs-come-in-index-order`,
  `job-log-range-answers-206`;
- later: `webhook-deliveries` (`unchecked webhook-deliveries: webhooks
  are built after standalone`).
Test shape: unit tests in `src/tests.rs`:
`a_fact_line_is_holds_fails_or_unchecked_with_its_reason` (a table of
the three outcomes against literal lines),
`a_fact_read_from_a_case_compares_its_status_and_message` (a recorded
`merge-stale-head` exchange as a literal). Under 20 ms each.
Proof: process (a real Forgejo and runner), `TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v16.0.5/forgejo-16.0.5-linux-amd64
TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v12.13.2/forgejo-runner-12.13.2-linux-amd64
cargo run -p forgejo-conformance -- --scratch
target/forgejo-conformance --out tools/forgejo-conformance`, run once
(after `sha256sum -c` in both directories prints `OK`): exit 0;
`grep -cE '^(holds|fails|unchecked) ' tools/forgejo-conformance/facts.txt`
prints 63 (the facts above), `grep -c '^unchecked '
tools/forgejo-conformance/facts.txt` prints 1 (the webhooks), and `ls -d
tools/forgejo-conformance/transcripts/*/ | wc -l` prints 83 (05.2's 77
and the six `actions-*` cases); under 3 minutes. Each `fails` line is
raised as a question (development.md, 8.7), quoting the line and
forge.md's text it contradicts, unless the design names its fallback;
the run goes on.
Don't: register the runner with any instance but this run's; download
an action or a container; change the design's text to fit a fact.
Size: at most 450 lines; ruling 102 lets it land as one increment at
at most 800 (decisions.md).
Removes: nothing.

### 05.4 `tools/git-transcripts`: git 2.53 against the throwaway Forgejo

Scope: the tool (git.md, 9): starts the throwaway Forgejo (Actions off),
makes `fixture` (admin), `writer` and `stranger`, the repositories
`fixture/specimen` (initialized, two commits on `main`, a branch
`topic`), `fixture/empty` (not initialized), `fixture/private`,
`fixture/hashed` (`object_format_name: sha256`) and `fixture/guarded`
(`main` protected: no push); a worker root `<scratch>/worker` with
`workspaces/1/`, `home/` (empty, 0o700); then runs each case below with
the git the caller names, git.md 3's settings in their order, the
spawn's directory as git.md 3 says, git.md 5's environment given whole
(identity `writer`, its token in `GIT_CONFIG_VALUE_0`'s Basic header for
`Clone`, `Fetch`, `Create`, `Push`), and writes
`<out>/transcripts/<case>/{argv,exit,stdout,stderr}` masked, and
`<out>/facts.txt` with the two checks.
Needs: 05.1's instance (not 05.2 or 05.3).
Files: `tools/git-transcripts/{Cargo.toml, clippy.toml, README.md,
src/{lib.rs, main.rs, invocation.rs, cases.rs, mask.rs, tests.rs},
transcripts/, facts.txt}`; the workspace `Cargo.toml` (member).
Shared hotspot: the workspace `Cargo.toml`.
Shape: `git-transcripts --git <absolute path> --scratch <dir> --out
<dir>`; it refuses a git whose `git version` is below 2.41, naming it.
`clippy.toml` is `tests/clippy.toml`'s; the `[lints]` table is 05.1's.
`mask::Mask::new(scratch: &Path, port: u16)` holds the run's map of ids;
`Mask::bytes(&mut self, &[u8]) -> Vec<u8>` masks one output.
`argv` holds one argument per line, the first `git` (the program's
path is not written); `exit` is the status in decimal, or `signal <n>`,
and a line feed; `stdout` and `stderr` as git wrote them, masked
(ports, object ids, the scratch path). The cases, one invocation each,
named `<operation>-<situation>` (`-1`, `-2` for an operation's later
invocations):
- `version`;
- Clone: `clone`, `clone-empty`, `clone-missing-repository`,
  `clone-wrong-token`, `clone-stranger-private`, `clone-refused-connection`
  (a loopback port nothing listens on);
- Fetch: `fetch-branch`, `fetch-commit`, `fetch-default`,
  `fetch-up-to-date`, `fetch-missing-branch`, `fetch-missing-commit`,
  `fetch-wrong-token`, `fetch-sha256-branch` (`hashed`);
- Create: `create-new-branch`, `create-exists-at-commit`,
  `create-exists-elsewhere`, `create-protected` (`guarded`);
- CheckOut: `checkout-1`, `checkout-2` (`clean -ffdxq`);
- Merge: `merge-clean`, `merge-up-to-date`, `merge-conflict`,
  `merge-conflict-2` (`diff-files`);
- Commit: `commit-add`, `commit-unchanged` (`diff-index` exit 0),
  `commit-changed` (exit 1), `commit-write-tree`,
  `commit-tree-one-parent`, `commit-tree-two-parents`,
  `commit-unmerged` (`diff-files` with a path still unmerged);
- Push: `push-fast-forward`, `push-new-branch`, `push-up-to-date`,
  `push-non-fast-forward`, `push-lease-stale`, `push-protected`
  (`guarded`), `push-no-permission` (`stranger`'s token on `specimen`),
  `push-ancestor-yes` and `push-ancestor-no` (`merge-base
  --is-ancestor`).
The facts, as 05.3's lines: `git-follows-a-renamed-repository-redirect`
(a fetch of `specimen`'s old URL after a rename, exit 0) and
`protected-push-is-remote-rejected-declined` (`push-protected`'s status
line is `!` and `[remote rejected]`, its reason ending `declined`).
Test shape: unit tests in `src/tests.rs`:
`the_environment_is_given_whole_and_holds_the_header_only_for_remote_operations`
(the `Command` the tool builds for `fetch-branch` and for `checkout-1`,
its environment against literals),
`masking_rewrites_ids_ports_and_the_scratch_path`,
`no_argument_holds_a_token_or_its_base64`. Under 20 ms each. Reference
test, in full:

```rust
#[test]
fn masking_rewrites_ids_ports_and_the_scratch_path() {
    let mut mask = Mask::new(Path::new("/home/someone/temper/target/git-transcripts"), 41_823);
    let first = mask.bytes(
        b"To http://127.0.0.1:41823/fixture/specimen.git\n\
          *\t9fceb02d0ae598e95dc970b74767f19372d61af8:refs/heads/topic\t[new branch]\n",
    );
    let again = mask.bytes(b"/home/someone/temper/target/git-transcripts/worker 9fceb02d0ae598e95dc970b74767f19372d61af8\n");
    assert_eq!(
        first,
        b"To http://127.0.0.1:3000/fixture/specimen.git\n\
          *\t0000000000000000000000000000000000000001:refs/heads/topic\t[new branch]\n"
            .to_vec()
    );
    assert_eq!(again, b"/scratch/worker 0000000000000000000000000000000000000001\n".to_vec(), "one map for the run");
}
```
Proof: process (real git and Forgejo), `TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v16.0.5/forgejo-16.0.5-linux-amd64
cargo run -p git-transcripts -- --git /usr/bin/git
--scratch target/git-transcripts --out tools/git-transcripts`, run
once: exit 0; `ls -d tools/git-transcripts/transcripts/*/ | wc -l`
prints 41 (the cases above); `cat tools/git-transcripts/transcripts/push-protected/stdout`
shows the `[remote rejected]` line; `grep -cE '^(holds|fails|unchecked) '
tools/git-transcripts/facts.txt` prints 2; under 2 minutes. A `fails`
line is raised as a question, as in 05.3.
Don't: use `-C`, a credential helper, a URL with a user or token, or
`PATH` in git's environment; write the environment to a file; reuse
`forgejo-conformance`'s instance.
Size: at most 450 lines.
Removes: nothing.

## Interface sketches

Ordinary Rust, not compiled (`s05.rs` holds no sketch): the support
crate's surface, which 05.2 to 05.4, 10.4, 14.5 and 15.3 use by name.

```rust
// temper_throwaway_forgejo (testing/temper-throwaway-forgejo/src/lib.rs), ordinary Rust, not compiled.

/// The one Forgejo version this plan runs (forge.md, 10).
pub const VERSION: &str = "16.0.5";

/// The binary `TEMPER_FORGEJO` names, checked: its first `--version` line and its SHA-256.
#[derive(Debug, Clone)]
pub struct Forgejo { binary: PathBuf, version: String, sha256: String }

/// Why no Forgejo: the variable unset, the binary unreadable, or another version (named).
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    Unset,
    Unreadable { path: PathBuf, error: String },
    Version { path: PathBuf, printed: String },
}

impl Forgejo {
    pub fn from_env() -> Result<Forgejo, Refusal>;
    pub fn from_path(binary: &Path) -> Result<Forgejo, Refusal>;
    pub fn binary(&self) -> &Path;
    pub fn version(&self) -> &str;
    /// 64 lowercase hex digits.
    pub fn sha256(&self) -> &str;
    /// Starts an instance in `scratch` (made here; refused if it exists and is not empty, or is under /tmp).
    pub fn start(&self, scratch: &Path, settings: &Settings) -> Result<Instance, Failure>;
}

/// How an instance runs.
#[derive(Debug, Clone)]
pub struct Settings {
    /// `[api] MAX_RESPONSE_ITEMS`; `None` keeps Forgejo's default.
    pub max_response_items: Option<u32>,
    /// `[actions] ENABLED`.
    pub actions: bool,
    /// The git Forgejo runs (`[git] PATH`).
    pub git: PathBuf,
    /// Readiness polls, 50 ms apart.
    pub ready_polls: u32,
    /// The largest answer `exchange` reads, head and body.
    pub answer_bytes: usize,
}

impl Settings {
    /// `MAX_RESPONSE_ITEMS = 2`, Actions on, `/usr/bin/git`, 200 polls, 16 MiB.
    pub fn small() -> Settings;
}

/// A running throwaway Forgejo; stopped and removed by `stop`, or by `Drop`.
#[derive(Debug)]
pub struct Instance { /* the child, the port, the scratch path, the admin */ }

/// A user the instance made, with an API token of scope `all`.
#[derive(Debug, Clone)]
pub struct Account { pub login: String, pub id: u64, pub token: Token, password: String }

/// An API token; its `Debug` prints its length only.
#[derive(Clone, PartialEq, Eq)]
pub struct Token(String);

/// How a repository starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Init { Readme, Empty, Sha256 }

/// A repository the instance made.
#[derive(Debug, Clone)]
pub struct Made { pub id: u64, pub owner: String, pub name: String }

/// One request `exchange` sends.
#[derive(Debug, Clone)]
pub struct Outgoing<'a> {
    pub method: &'a str,
    pub target: &'a str,
    pub token: Option<&'a Token>,
    pub body: Option<&'a [u8]>,
    pub range: Option<(u64, u64)>,
}

impl<'a> Outgoing<'a> {
    /// A `GET` with no body and no range.
    pub fn get(target: &'a str, token: Option<&'a Token>) -> Outgoing<'a>;
    /// A request with a JSON body.
    pub fn json(method: &'a str, target: &'a str, token: Option<&'a Token>, body: &'a [u8]) -> Outgoing<'a>;
}

/// One exchange: what was sent (its `Authorization` value written `token MASKED`) and received.
#[derive(Debug, Clone)]
pub struct Exchange {
    pub sent: Vec<u8>,
    pub status: u16,
    /// The status line and header lines as received, CRLF-separated.
    pub head: Vec<u8>,
    /// The body, unframed (chunks joined).
    pub body: Vec<u8>,
}

impl Instance {
    pub fn port(&self) -> u16;
    /// `http://127.0.0.1:<port>`.
    pub fn base(&self) -> String;
    pub fn scratch(&self) -> &Path;
    pub fn admin(&self) -> &Account;
    pub fn create_user(&mut self, login: &str) -> Result<Account, Failure>;
    pub fn create_repository(&mut self, owner: &Account, name: &str, init: Init) -> Result<Made, Failure>;
    pub fn add_collaborator(&mut self, owner: &Account, repository: &str, user: &Account, permission: &str) -> Result<(), Failure>;
    pub fn exchange(&self, outgoing: &Outgoing<'_>) -> Result<Exchange, Failure>;
    /// Every secret the instance made (passwords, tokens), for a writer's scan.
    pub fn secrets(&self) -> Vec<String>;
    pub fn stop(self) -> Result<(), Failure>;
}

/// Why a start, a provisioning step or an exchange failed, with what was seen.
#[derive(Debug)]
pub enum Failure {
    Scratch { path: PathBuf, why: String },
    Spawn { step: &'static str, error: String },
    Exited { step: &'static str, status: String, log_tail: String },
    NotReady { polls: u32, log_tail: String },
    Io(String),
    Status { target: String, status: u16, body: String },
}
```

- **Who owns what:** the instance owns its child and its scratch
  directory; the caller owns the scratch's parent. `stop` and `Drop`
  end both; nothing of an instance outlives its owner.

## Questions to raise when reached

- **A fact fails** whose design text names no fallback: a question
  (development.md, 8.7) with the `fails` line and the design's sentence.
  Default: the run goes on; nothing in the design or the fake changes
  here.
- **`413` (Forgejo's quota)** needs a quota configuration this fixture
  does not make. Default: no case provokes it, and 3.4's row rests on
  the status alone; raise it if 13.4 needs its message.
- **Forgejo refuses to start without `RUN_USER`'s fallback,** or the
  runner refuses `next_domain.py`'s configuration keys: raise it with
  `server.log`'s or the runner's last lines. Default: none expected.
- **A case's status differs from what its name says** (say,
  `moved-repository` answers `404`, not a redirect): it is recorded as
  received and its fact `fails`; no case is renamed to fit.

## Done when

- The throwaway Forgejo starts and is removed:
  `TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v16.0.5/forgejo-16.0.5-linux-amd64
  cargo nextest run -p temper-throwaway-forgejo --profile
  measure --run-ignored only` passed once (05.1's Proof).
- No Python is left in the tool: `git ls-files 'tools/forgejo-conformance/*.py'`
  prints nothing.
- The transcripts and facts are committed: `test -s
  tools/forgejo-conformance/transcripts/binary.txt && test -s
  tools/forgejo-conformance/facts.txt && test -s tools/git-transcripts/facts.txt`
  succeeds, and `grep -q 'sha256 2c548bf1bfa09a441a3a674b1dd4959d77576816ad50534c23496908a3ea5a19'
  tools/forgejo-conformance/transcripts/binary.txt` succeeds.
- No token reached a file: `git grep -nE 'Authorization: (token|Basic) [0-9A-Za-z+/=]{20,}'
  -- tools` prints nothing.
- The gate passed on this session's last landing: `git -C
  ~/src/rust/temper log --format='%(trailers:key=Gate-Result,valueonly)'
  --grep='^Step: temper-05\.' origin/main | head -1` names
  `scripts/check.sh passed`.
