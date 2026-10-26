# What setup builds on: the Forgejo crate, the throwaway, the conformance, the process tier

Read-only audit, 2026-10-16, temper `main` at `0ff45022e`; paths are temper's; every line is from a direct read (the graph is stale here).

## 1. `crates/temper-forge-forgejo`

- **Modules** (`lib.rs:5-17`; `no_std` outside tests; deps skein-lib and skein-json only, no serde): `request`, `client`, `server`
  (each a file per family under `client/`, `server/`), `decode` (+`walk.rs`), `document`, `failure`, `header`, `id`, `limits`, `path`, `time`.
- **A document, both sides.** Shared: `request::Request` (`request.rs:37-223`, 35 variants, no credential in it) and `Method`
  (`:4-9`: Get, Post, Patch, Delete; **no Put**, which `skein_http::Method` has). Client: `client::encode(&Request,&Limits)->
  Result<Encoded{method,target,body},Refusal>` (`client.rs:34`), `client::decoder(&Request,&Limits)->Option<Decoder>` (`:93`),
  `client::step` (`:155`) giving `decode::Step::{Demand,Row,Done(Document),Malformed}` (`decode.rs:13`). Server:
  `server::route(Method,&[u8],&Limits)->Result<Route::{Ready(Request),Body(BodyDecoder)},Unrouted>` (`server.rs:39`, NotFound
  fall-through `:45-62`), `server::body` (`server/writes.rs:218`), `server::Answer` (`server.rs:64`), `server::encode` (`:111`).
- **One route by its types:** `POST /api/v1/repos/{o}/{n}/issues/{number}/comments`, `{"body":..}`. `Request::PostComment{at:Repo,
  number:u64,body:Box<[u8]>}` (`request.rs:169`). Client target `client/writes.rs:48-51`, body `:184` through `Encoder::measure`
  then `Encoder::write` (`:21-32`). Server arm `server/writes.rs:131-135` yields `Route::Body(BodyDecoder)` (`:200`) from the field
  table `WriteKind::Comment => [field(b"body",Read::Body,true)]` (`:50`); `BodyDecoder::scalar` fills it (`:76`). Answer:
  `client.rs:109` maps it to `Kind::Item(Item::Comment)`, table `decode/walk.rs:90-100`, to `Document::Comment(Comment{id,user,
  created_at,updated_at,body})` (`document.rs:128`); fake side `Answer::Comment(Comment)` (`server.rs:131`).
- **Codecs and bounds:** `skein_json::tokenizer` (pull events) reads, `skein_json::writer::Encoder` writes, `skein_lib::{Writer,
  Decimal,List,Wall,Stack}`. `limits::Limits{json,writer,string_bytes,body_bytes,inner,target_bytes}` (`limits.rs:4`, `check`
  `:23`); `Body::{Kept,Long(len)}` (`document.rs:6`); `Malformed` (`decode.rs:7`); unknown fields skipped (`tests.rs:116`).
  `tests/codecs` replays every conformance transcript through both sides (`tests/codecs/src/forgejo.rs:379`); a 2xx JSON capture
  on an unrouted route panics at `:96`, so a new capture lands with its server route.
- **The pin is not here** (prose only: `lib.rs:1`, `failure.rs:1`). It is `testing/temper-throwaway-forgejo/src/binary.rs`:
  `VERSION="17.0.0"` `:9`, `RUNNER_VERSION="13.2.0"` `:11`, `names_forgejo` `:14-16` (first line `forgejo version 17.0.0`, then end,
  `+` or space), `names_runner` `:19-21` (exact `forgejo-runner version v13.2.0`); re-exported `lib.rs:5`; used at `:51`,
  conformance `runner.rs:203`, tests `src/tests.rs:33-68`. No digest is checked (computed `:54`, recorded in `facts.txt:1`).
  Production cannot depend on that std test crate; setup.md 11 puts "the pins" in `temper-forge-forgejo`: move the consts, the two
  pure fns (`core`-only) and their tests there, keep the throwaway's re-export, fix `tools/forgejo-conformance/README.md:5-7`.
- **Adding the setup's documents:** (a) `Method::Put` for team members (mapped at engine-protocol-forge `connection.rs:204-207`,
  fake `server.rs:391`); (b) `BodyDecoder::scalar` refuses Number and Boolean (`server/writes.rs:54`), so `active`,
  `includes_all_repositories`, `must_change_password` and the hook's nested `config` need arms (`walk.rs` has `Read::Boolean/
  Object`); (c) `Request` has no credential, the host sets `Authorization: token` (`connection.rs:165-172`), so basic auth is the
  setup host's header; (d) `document::User` is `{id,login}` (`document.rs:35`): admin and the description marker need a new
  `Document` or fields; (e) each variant sits in ~10 exhaustive lists (`client.rs` 2, `client/start` 2, `writes` 2, `files`,
  `reviews`, `items`, `server/writes.rs` 4; wildcard arms are denied) plus fake `routes.rs:143` and `tally.rs`; precedent is a file
  pair per family (`client.rs:12-22`, `server.rs:12-18`), so a `setup` pair with its own `Kind`.

## 2. `testing/temper-throwaway-forgejo`

- **Deps:** skein-io (`digest`), serde_json; its HTTP is its own blocking `std::net::TcpStream` client (`http.rs:28-114`).
- **Provisioning** (`Instance::start`, `instance.rs:27-83`): scratch absolute, not under `/tmp`, empty (`:245`); free port by bind
  and drop (`:29`); 32 random bytes hex from `/dev/urandom` (`:229`) for the admin password, SECRET_KEY, INTERNAL_TOKEN; `home/`
  0700 and an empty `gitconfig`; `app.ini` (`ini()` `:261`, golden `src/tests.rs:6`); `forgejo migrate`; `admin user create
  --username fixture --password <argv> --email .. --admin --must-change-password=false`; `forgejo web` into `server.log`; ready
  when `GET /api/v1/version` is 200 (200 polls x 50 ms); token `POST /users/fixture/tokens` under basic auth, scopes `["all"]`
  (`:116`); `GET /user` for the id. Later: `create_user` (`POST /admin/users` + token, `:160`), `create_repository` (`:172`),
  `add_collaborator` (`PUT`, `:182`), `exchange` (`token` auth only, `:196`).
- **Environment** (`binary.rs:78-85`): cleared, `HOME=<scratch>/home`, `PATH=/usr/bin:/bin`, `TZ=Asia/Kolkata`, `GIT_CONFIG_NOSYSTEM=1`,
  `GIT_CONFIG_GLOBAL=<scratch>/gitconfig`; the runner's: cleared, `HOME`, `PATH` (`runner.rs:42-46`). **Masking:** `Instance.secrets`
  (`:19`: password, both keys, Basic headers, tokens); `mask()` (`:107`) covers command stderr, the log's last 2000 bytes and failed
  bodies; `Token` Debug shows its length (`lib.rs:132`); recorded requests say `token MASKED` (`http.rs:46`, a Basic header too);
  the conformance rescans outputs (`write.rs:169-233`; registration token `runner.rs:210`).
- **Setup 6.2/6.3 re-specifies:** `ini()`'s keys (RUN_USER, WORK_PATH, server, sqlite, repository, security, DISABLE_REGISTRATION,
  actions), adding `[oauth2] JWT_SECRET`, `[mailer]` off, `[webhook] ALLOWED_HOST_LIST`, file logs (it logs to console), and
  dropping test-only `[api] MAX_RESPONSE_ITEMS` (`lib.rs:105`), `APP_NAME`, `[git] PATH`; migrate and `admin user create` (with
  `--random-password`, not argv); tokens by `generate-access-token` (today the API); the runner by `forgejo-cli actions register
  --secret-file` + `server.connections` (today an API registration token and `register --no-interactive`, `runner.rs:206-226`).
  **In no code today:** `dump`, `forgejo-cli`, `generate-access-token`, `--random-password`, orgs, teams, hooks, `[webhook]`
  (forge.md:940 says loopback webhooks are allowed; `ini()` does not).
- **Not public:** running a CLI command on a work path (`command`/`run` `:84,94`), Basic auth (`token()` `:116`; `Account.password`
  is `pub(crate)`, `lib.rs:113`), extra ini keys (`ini()` takes `Settings` only).

## 3. `tools/forgejo-conformance`

- **Flow** (`lib.rs:17-44`): `Fixture::new` (`fixture.rs:20`: admin `fixture` is `owner`; `writer`, `stranger`; 5 repos), then
  `cases::record` (`cases.rs:43`: start, writes, reads, failures), `facts::evaluate` (`facts.rs:59`), `runner::run` (`runner.rs:47`),
  `cases::flags`, `facts::flags` (`:530`), `webhook-deliveries` pushed Unchecked (`lib.rs:28-31`), `facts.txt` written. Now: 109
  holds, 4 fails (the base's), 1 unchecked; 91 transcript directories; 40 `component-` facts.
- **A fact** (`facts.rs:156-161`): `add(facts, "writer-protection-reads-are-403", case(out,"writer-branch-protection")?.status ==
  403 && .., "writer protection read was not 403")`; `add` `:35`, `case(out,name)` `:44` reads `transcripts/<case>/response`. A capture
  is `Recorder::send(name, method, target, body, token, range)` (`cases.rs:17-32`: scan, mask, `write::case`).
- **Record command** (README; forgejo-17 `findings/tools.md:41-44`): `TEMPER_FORGEJO=<v17> TEMPER_FORGEJO_RUNNER=<13.2> cargo run -p forgejo-conformance -- --scratch target/forgejo-conformance --out tools/forgejo-conformance`: **38 s**; with
  `--component` (replaces only `component-` lines, `component.rs:19`, so second) **24 s**; both in one `sh -c` (clean
  tree). Without the runner, 6 Actions and 2 component facts stay unchecked.
- **A new group** (admin routes, orgs, teams, org hooks, deliveries): append `setup(r)` after `failures` (`cases.rs:48`; masking
  numbers ids and times by first appearance, so an earlier capture renumbers all later ones); admin calls use `send(..,
  Some(&f.owner.token), ..)` (`r.w` is the writer, `:33`); extra provisioning via `Fixture::api` (`fixture.rs:84`), appended last in
  `Fixture::new`; a `fn setup(f,out,facts)` in `facts.rs` called from `evaluate`. Each 2xx capture needs its server route too.
  **Missing for 13.3:** CLI facts (migrate, dump, generate-access-token, `forgejo-cli`, `create-runner-file`) need a public
  throwaway call that runs a command on a work path (a second instance has precedent: `facts::initial`, `facts.rs:95-108`);
  basic-auth facts need a Basic exchange; the runner's start is private to `runner.rs:198-236`.
- **Webhook delivery: not yet.** No `[webhook]` in `ini()`, no listener (the only `TcpListener::bind` are port reservations,
  `instance.rs:29`, `git-transcripts/src/cases.rs:225`). It would be a blocking `std` listener thread in the tool (as `http.rs` is
  blocking), the hook secret added to `Secrets`, the raw request kept by `write::case`, `ALLOWED_HOST_LIST` added to `ini()`;
  "mergeable settled" and "job delivery vs status" are timing facts, polled as `Fixture::ready` (`fixture.rs:113`) does.

## 4. The process tier

- It is `crates/temper/tests/process.rs`: `scripts/proof-one-process.sh` builds the binary, runs `cargo nextest run -p temper
  --profile process` (`.config/nextest.toml`: `binary(=process)`, 10 min, 1 thread). `tests/one-process` is the simulated world.
- **Provisioning today:** `support::forgejo::Throwaway::start` (`support/forgejo.rs:16-26`): `Forgejo::from_env` (`TEMPER_FORGEJO`,
  `binary.rs:37`), `Settings{actions:false,..small()}`, users `acme` and `temper-bot`, repo `acme/widget`, bot a write collaborator.
  `configuration` (`:114`) writes the bot token as `secrets/forge-1.{api,git}-token`, sets `api`, `git`, `user`; landing rule
  `"ci":false` (`:132`). No reconcile, org, hook or runner; `TEMPER_FORGEJO_RUNNER` is read only by the conformance (`runner.rs:55`,
  `component/reads.rs:57,93`). So its repository runs **no Actions**.
- **Serve and story:** `support::serve` (`support/mod.rs`) runs `temper serve config.json`, stdio piped.
  `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart` (`process.rs:71`): `person::adopt_and_change`, fake LLM
  `landing_scripts` (`forgejo.rs:218`), `run_until_landed` (180 s; merged pull polled every 2 s, then Kill), restart on the same
  directory, `person::found`. Six tests (`process.rs:7,19,34,71,88,101`), **42.4 s** in all (`tmp/temper/forgejo-17/logs/
  20261016T160430-0.log`): the story 29.2 s, adopt-by-commands 11.5 s, the rest under 1.2 s; Forgejo's start is not timed apart.

## 5. Binaries here

`~/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64` (122,053,192 bytes; sha256 `5a3b07e1..1042ca` = `facts.txt:1`) and
`runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64` (22,290,594 bytes); both print their pinned line (old `v16.0.5/`, `runner-v12.13.2/` remain).
Forgejo's source notes: `~/src/rust/tmp/forgejo17-notes/` (`hook.go`, `deliver.go`, `services_webhook_*.go`).
