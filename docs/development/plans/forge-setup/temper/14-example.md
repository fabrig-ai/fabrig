# Session 14: the owned example, and the setup guide

The first milestone's setup is a document a person copies, its secret files, an LLM account handed in, `temper box`, and
a repository made on Forgejo's web and adopted (setup.md 12). 14.1 puts the owned example in `docs/examples/` as the
whole document `temper box` reads, with the files it names that hold no secret, and makes the process tier take its
owned `application` from it, so that the example, the tier and the guide cannot drift. 14.2 writes `docs/guide/box.md`
and follows it on x1 by a script, up to an adopted repository: no person's login, no live service (decisions.md 16, 21).

## Read first

- Design: setup.md 8 (last bullet), 12, 13.4, 6.5, 16 (`temper box init`: default none); configuration.md 10, 3
  (`webhooks`, `setup`), 5, 6, 7, 8; credentials.md 3; draft/mvp.md 5.1 and 7; domain/testing.md 2.6, 2.7. decisions.md 5, 14,
  16, 20, 21; `findings/configuration-and-binary.md` 3, G6; `findings/forgejo-tokens.md` row 10. skein's
  `testing-strategy.md`: 2.9 and 8.
- Code: `crates/temper/tests/support/{mod.rs:15-18, 19-41, 28-31, 52; configuration.rs:4-46; live.rs:46-69}`,
  `crates/temper/tests/{startup.rs:21-35, configuration.rs:3-24, process.rs:33-68}`, `crates/temper/src/lib.rs:146-147`,
  `crates/temper-service/src/{startup.rs:13-20, 142-151; limits.rs:245-257; tests/fixtures.rs:738;
  tests/fixtures/startup.rs:9}`; smith's `crates/smith-agent-shell/src/config.rs:883-886` and
  `benchmarks/agents/models.toml:1-2`; skein's `crates/skein-llm/src/openai/identity.rs:22-24`.
- The owned `application` block as the design held it before it became a file: `git show
  e71918bc3:docs/design/configuration.md`, under "The owned example".
- Read by name at the ready check: 08.1's `support/owned.rs` with 10.2's `owned::tier_configuration(directory) ->
  SocketAddr` (the owned configuration's writer, apart from its start of Forgejo), its `binaries()` (reads
  `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER`) and the workflow text; 13.1's and 13.2's `temper_box_*` tests.

## What the design asks

- configuration.md 10 and setup.md 8: the owned example is a file, `docs/examples/owned.json`, a whole document; the
  files it names that hold no secret are in `docs/examples/owned/`; the worker's secret and digest and the LLM account's
  file are never shipped, and the setup document says how each is made. 14.1; domain/testing.md 2.6 and setup.md 13.4 have the
  tier's configuration "written from" it and the process tier's test of `temper box` run it.
- setup.md 12, mvp.md 5.1: the walk 14.2 writes and follows; the dogfood run is mvp.md 5.1's second bullet, not ours.

## Choices this plan makes

- **The file is the whole document** (`engine`, `workers`, `application`), because `temper box CONFIG` reads one. Its
  non-secret files are smith's agent configuration (configuration.md 5) and the two charters' texts, short texts a
  person edits (setup.md 12); they widen decisions.md 14's one file (the designer's answers 12 and 45).
- **The account is handed in** (mvp.md 7's default), the live tier's provider (Codex, `chatgpt.com`) and model
  (`gpt-6-luna`); this plan's tests and script hand in an account for skein's fake peer (`support/mod.rs:28-31`).
- **The tier takes the `application` member from the file** and keeps jig's reference `engine` and `workers`
  (`support/configuration.rs:14-36`: shrunk limits, `llm` the fake peer). The box's two tests run the whole file, with
  the provider's name made a loopback socket, so that no test resolves a name.

## Rules that bind it

- **No live service, no sign-in, no LLM** (decisions.md, 16 and 21); only the binaries `TEMPER_FORGEJO` and
  `TEMPER_FORGEJO_RUNNER` name. The one outside effect is the DNS lookup of the provider's name that `temper serve`
  makes as it reads the agent's configuration (`lib.rs:146-147`): 14.2's script alone.
- **One source for the owned `application`:** the file; no test builds a copy (Done when, items 3 and 4); an increment
  of 11 that lands after 14.1 uses `owned::application`, and 14.1 converts those landed before it.


## Increments

### 14.1 The owned example as a tested file

**Corrections after the designer's answer of 2026-10-18T16:47:23Z (the question of 2026-10-18T16:33:07Z; configuration.md
7, "The owned example's `memory_bytes`", and 10), for the step that follows it and each landing after it, in this run
and in client-api's. On `memory_bytes` they take precedence over Shape 1's "`memory_bytes` and each other number are the
least the start's checks accept", over the 12:25:44Z block's "Ruling 143's rule … stays for `memory_bytes`" and over
ruling 143's rule; for each other number ruling 143 stays:**
- `engine.memory_bytes` and `workers[0].memory_bytes` are each the least T3 accepts as it lays the example below, with
  the strings the worst cases count at their bounds; never the least at one checkout's directory. The designer's probe
  on temper 0601176ab: 78,669,036,604,974 and 1,989,738,112,203 start; one less each is refused "the worst case passes
  memory_bytes by 1 bytes", from scratch directories of 78 and 83 bytes. `main` states 78,669,036,604,496 and
  1,989,738,096,213: 478 and 15,990 short.
- T3 `serve_prepares_once_on_the_owned_example` (`crates/temper/tests/startup.rs:314`) lays the example at those bounds
  in place of its `owned_example` call: `Owned { forgejo: "127.255.255.254:65535".parse()…, ..Owned::example() }` for
  `lay` and `example_document`; in the parsed document, `workers[0].agent.program` and `workers[0].agent.arguments[1]`
  each `"/"` and 4,095 `p`s (`prepare_in` neither runs the program nor opens the argument; `configuration` stays the
  laid file); `forge-1/api_token` 512 bytes, the other three files as they are. One comment line cites configuration.md
  7. T2, T4 and the box's tests are unchanged. Size: about +10 lines.
- The step sets the two numbers from T3's refusal: with them as `main` states them, T3 is refused "the worst case passes
  memory_bytes by N bytes" for each; raise each by its N, and no more. Nothing else in `owned.json` changes.
- From then on, a landing that raises either worst case raises its number by the N T3's refusal names; a landing that
  lowers one leaves the number, and nothing checks a lowering. No other file copies either number (`git grep -n
  '78669036604\|1989738096'` names `owned.json` and client-api's ruling 363 alone).
- Don't: price these strings at their bounds in `worker_worst_case` or `workers::worst_case`, which count them as
  written and stay; add room past the bounds; change T3's ceiling (1 s; the probe's start took 43 ms).

**Corrections after the designer's answer of 2026-10-18T12:25:44Z (the question of ruling 145; configuration.md 7 and
10). They take precedence over the text below, over the corrections below and over ruling 145's provisional eight:**
- `application.limits` is `{"profile": "small", "overrides": {}}`; `workers[0].limits` is `{"profile": "small",
  "overrides": {"hub.accounts": 4, "hub.event_bytes": 598}}`, as configuration.md 10's skeleton writes them. No other
  override. This replaces the Test shape's "`small` with the fewest overrides T3 finds". The tier keeps its own
  (`support/configuration.rs:24-25, 43-45`), unchanged.
- `small` changes in `crates/temper-service/src/limits.rs` (configuration.md 7, "`small` starts as it stands"):
  `CHECKOUT.repositories` 3 to 2, `CHECKOUT.message_bytes` 256 to 4096, `FORGE.resources_per_task` 4 to 2,
  `FORGE.name_bytes` 256 to 64, each comment saying in a line what it is held to. `CHECKOUT.name_bytes` 256 and
  `GIT.stdout_bytes` 1024 stay; no other constant moves. Files gain `limits.rs`, `src/tests/checks.rs` and
  `src/tests/fixtures/startup.rs`.
- Two temper-service tests rest on small's old forge values. Each override list gains
  `"forge.name_bytes":256,"forge.resources_per_task":4`: `tests/checks.rs:43`
  (`a_remote_past_the_checkouts_names_refuses_the_start`) and `tests/fixtures/startup.rs:29`, which
  `startup_makes_the_one_process_from_the_example` reads. With 2 resources against that fixture's 4 repositories, its
  hello never reaches the engine; this is `later.md`'s, not 14.1's.
- Shape 2: `owned/agent.json`'s `profile` is the tier's (`support/configuration.rs:5-7`), written out as
  configuration.md 10's "Its agent's configuration" lists it. Its endpoint is as Shape 2 says. `hub.event_bytes` 598
  rests on its `message_bytes`, 600.
- The designer's probe: temper f6cb29f5c, the tier's document (`support::configuration`) with the example's 13 engine
  overrides, `git` `http://127.0.0.1:3000` and the tier's agent. With the four constants changed, `{}` and the worker's
  two overrides, `temper::prepare_in` is `Ok`. One past each is refused: `hub.event_bytes` 597 and 599, `hub.accounts`
  5, `checkout.message_bytes` 4095, `checkout.repositories` 3 and `checkout.name_bytes` 257 (`workers[0].limits: past
  its bound`); `forge.resources_per_task` 3 (`must agree with checkout.repositories and checkout.name_bytes`);
  `forge.name_bytes` 115 (`must agree with the longest remote`; 114 starts). Then `cargo nextest run -p
  temper-service -p temper` ran 260 tests, 260 passed. T3 checks the example's start; no test is added for it.
- Ruling 143's rule (each number the least its check accepts) stays for `memory_bytes` and each number the design does
  not give. The numbers above are given. Ruling 145's "Change no `startup.rs`, `worker_settings.rs` or smith code"
  stays. Size: about +8 lines (the four constants and the two fixture lines).

**Corrections after the designer's answer of 2026-10-18T11:40:51Z (the question of ruling 139; configuration.md 7 and
10). They take precedence over the text below and over ruling 139's T4 line:**
- Shape 1, `engine.limits`: `small` with configuration.md 10's 13 overrides, as its skeleton writes them: the three
  segment bounds 8; `documents.rows` 2, `documents.lines` 2, `documents.text_bytes` 128; `views.watchers` and
  `views.watchers_per_party` 16; `authority.grants` and `tasks.authority_grants` 12, `notes.scopes` 15;
  `tasks.authority_bytes` 512, `tasks.batch` 1. No other `engine` override. "If T4 finds the documents past the
  client's, …" goes: the file names them.
- The grants stay the 10 `support/forgejo.rs:156-165` writes (kinds 1, 2, 3, 4, 7, 8, 10, 11, 12, 13), each on
  `["forge", "127.0.0.1:3000"]` open `acme`, in `rules.ceiling`, `projects[0].policy.ceiling`, its `roles[0].authority`
  and `projections`; the spends as the tier writes them.
- The designer's probe (temper dbc5b4da7; no code changed to d09271b57), the tier's engine with these limits and the 10
  grants: `temper::prepare_in` is `Ok`. Without `tasks.batch` 1 it is refused `engine.limits: past its bound` (jig's
  `sizes::check`: `Value { needs: 4758343, has: 1303199 }`); without `notes.scopes` each core override is `invalid
  override`. T3 is the example's own check of it.
- T4 `the_owned_example_fits_the_native_client`, in place of the text below, over `owned::EXAMPLE`'s `engine` member
  parsed as `tests/configuration.rs:9-15` do: (1) the four assertions of `:16-33` (rows, lines, paths, text); (2)
  `assert_eq!` of the JSON's `engine.limits` with configuration.md 10's literal, a `json!`; (3) the authority's entries:
  if `main` at 14.1's start holds client-api's temper-04.s1 (`native()`'s `authority` bound and its three assertions
  after the paths'), the same three over the example's engine; else `authority.grants` and `tasks.authority_grants`
  each at most 12 (client.md 5.4), a literal. The commit body says which. Size: +10.
- T3 (ruling 142): `example_document` keeps the file's `charter` 2, so T3 starts the example's own charter; only the tier's `owned::application` sets 1.

**Corrections against main (ruling 139, at temper 64f610ae6, before 05.3 landed; they take precedence over the text
below; the example's authority limits: the designer's answer, above):**
- T2's second line reads `application.forges[0].user.file: forge-1/user: absent` (`application.rs:514`,
  `tests/startup.rs:90`).
- The DNS lookup (Rules that bind it) is `address()`, `lib.rs:210-219` (`to_socket_addrs` at :214), not :146-147.
- Shape 3: the engine's `llm` address is `{"host":"chatgpt.com","port":443}`, which no text replacement reaches:
  `example_document` sets that member itself.
- `support/forgejo.rs` holds no `"owned"`: its owned members come through `owned::document` and `owned::Names`
  (`forgejo.rs:145-155`) and the landing rule (:166); the org is already `acme` (`owned.rs:119` and :289 say `workshop`);
  only `bot: "temper-bot"` (`forgejo.rs:151`) changes.
- T4: `tests/configuration.rs`'s test spans :3-34 and has four assertions (rows, lines, paths, text).
- Users the Files miss: `support/live.rs:131` reads `worker-root` (the example's `workers[0].root` is `workspaces`),
  through `Throwaway::start` (`forgejo.rs:22`), which `process.rs:36,71` and `live.rs:8` use; copies of the `setup`
  member outside Done when 3 and 4's grep: `tests/forge-setup/reconcile/src/fixtures.rs:9`,
  `crates/temper-forge-setup/src/tests/fixtures.rs:13` and `read.rs:27,62,67`. Change each copy that the example's
  values reach, and list each in the commit body.
- The sketch's `use std::{…path::{Path, PathBuf}}` clashes with `owned.rs:6` (E0252): merge `net::SocketAddr` into the
  existing `use std::{…}`.
- Line references now: `support/forgejo.rs:122-131` is :156-165; `support/mod.rs` 15-18 is 13-16, 15 is 17, 19-41 is
  21-43, 28-31 is 30-33, 52 is 55; `support/configuration.rs` 14-36 is 14-35, 29 is 28, 31-33 is 30-32;
  `fixtures.rs:738` is :753; `process.rs:33-68` is 33-66.

Scope: `docs/examples/owned.json` and the files beside it; `support::owned`'s example functions; the tier's owned
configuration, and the box's two tests, taken from them; four tests of the file (setup.md 8; configuration.md 7).
Needs: temper 05.3 (`application.webhooks`, `forges[].webhook_secret`) and 13.1 (`temper box`), which bring 07.3 (`user`
as a file; `application.setup`), 10.2 (the owned writer, the workflow) and 12.2.
Files: `docs/examples/owned.json`, `docs/examples/owned/{agent.json, charters/chat.txt, charters/coding.txt}` (new);
`crates/temper/tests/support/owned.rs` (08.1's, with 10.2's `tier_configuration`; extended here, and
`tier_configuration` takes its `application` member from `owned::application`); `support/forgejo.rs` (hotspot:
`configuration`, as 10.2 left it, stops writing the owned members; find them, and those of 08.1's `owned::document`,
with `git grep -n '"owned"' -- crates/temper/tests`; 08.1's shell tests keep `document` over `Names`);
`crates/temper/tests/process.rs` (hotspot: the two `temper_box_*` tests take `Owned::tier()`, call `owned::lay` and
write `owned::example_document` in place of `owned::tier_configuration`); `crates/temper/tests/{startup,
configuration}.rs`; `crates/temper-service/src/tests/{startup.rs, fixtures.rs, fixtures/startup.rs}`. The tier's org and
bot (`workshop`, `temper-bot`) become the example's `acme` and `temper`: `git grep -n 'workshop\|temper-bot' --
crates/temper/tests` lists the tests that name them.
Shape:
1. `owned.json`, valid JSON with no placeholder syntax (`temper_service::named` takes it as it stands): `application` is
   the block of `e71918bc3` (Read first), letter for letter. `engine` and `workers` are jig's parts in the shape the
   tier writes (`support/configuration.rs:14-36`; the forge's grants and the rule's authority as
   `support/forgejo.rs:122-131` writes them), without the tier's limit shrinks, in configuration.md 10's skeleton:
   `hosts.workers` `secrets/worker.digest`, `workers[0].secret` `secrets/worker.secret`, `agent` `/usr/local/bin/temper`
   and `/home/me/temper/owned/agent.json`, two charters (`instructions` `owned/charters/chat.txt` and `coding.txt`) and
   two templates (`application.charter` 2), `engine.limits` `small` with its three authority overrides (8),
   `deployment.directory` `state`. `llm`: the endpoint `codex`, TLS to `{"host": "chatgpt.com", "port": 443}`, the model
   `gpt-6-luna`, one account `{"number": 1, "source": {"handed_in": {"file": "accounts/llm.json"}}}`, `records`
   `accounts/records`. `memory_bytes` and each other number are the least the start's checks accept: take each from
   the check that refuses it, and rerun the focused tests until T3's start passes (each stage of configuration.md 2's
   "Reading" reports its problems and stops, `startup.rs`'s `return Err(problems)`; ruling 143). If T4 finds the documents past the client's, the engine names `documents.rows`,
   `lines` and `text_bytes` as the tier does (`support/configuration.rs:31-33`).
2. `owned/agent.json`: the endpoint, account and model of `engine.llm` under the same names (configuration.md 5), the
   real provider's members as `support/live.rs:46-69` sets them: TLS to `chatgpt.com:443`, the machine's roots, the
   three identity headers written out. No credential. Each charter is a short text a person edits (setup.md 12): a line
   or two of what the charter does.
3. `support::owned` (Interface sketches). `application` sets `charter` 1 and, unless `review`, the rule's `gates` to
   `[]` (domain/testing.md 2.6: CI alone); addresses and binaries are replaced everywhere they occur as text, the runner's path
   before Forgejo's. `Owned::tier()` reads the binaries through 10.2's `binaries()`, which panics naming the missing
   variable (domain/testing.md 2.6), so a box test refuses at once, not at a prepare 120 s later. `tier_configuration` assigns
   `value["application"] = owned::application(&Owned::tier())` and keeps writing the engine's side of the forge. The
   box's tests call `tier`, `lay` and write `example_document` (not 08.1's `document`, which takes a `Value` and
   `Names`).
Test shape: `crates/temper/tests/{startup,configuration}.rs` (default suite, `Scratch::startup` `support/mod.rs:52`, as
`startup.rs:21-35` and `configuration.rs:3-24`; `write_private` `support/mod.rs:15`) and
`crates/temper-service/src/tests/startup.rs` (`fixtures::OWNED` beside `APPLICATION`, `fixtures/startup.rs:9`,
re-exported at `fixtures.rs:738`). Limits: the file's `application.limits` is `small` with the fewest overrides T3 finds
(builder `limits.rs:245-257`), each named in the landing's report; the tier keeps its own
(`support/configuration.rs:43-44`). Ceilings: T1 under 20 ms; T2 under 0.5 s; T3 under 1 s (a full start); T4 under 50
ms; a miss is noted in the landing report, read from nextest's `PASS [ t ]` lines in the test log. Tests:
- T1 (temper-service) `the_owned_example_is_named_without_a_problem`: `crate::named(fixtures::OWNED)` is `Ok`, with
  `agent` `/home/me/temper/owned/agent.json`, `worker_root` `workspaces`, `deployment` `state`, four files under
  `forge-1/` (reconcile's) and `accounts/llm.json` listed as private.
- T2 `serve_on_the_owned_example_asks_only_for_the_files_reconcile_makes`, written out:
  ```rust
  // crates/temper/tests/startup.rs (14.1); `support`, `fs` as the file's head has them
  #[test]
  fn serve_on_the_owned_example_asks_only_for_the_files_reconcile_makes() {
      let scratch = support::Scratch::startup("owned-example-refused");
      let owned = support::owned::Owned::example();
      support::owned::lay(scratch.path(), &owned);
      let document = support::owned::example_document(scratch.path(), &owned);
      fs::write(scratch.path().join("config.json"), document).expect("configuration");
      assert_eq!(
          temper::prepare_in(scratch.path()).err().expect("the start is refused"),
          "temper serve: application.forges[0].api_token: forge-1/api_token: absent\n\
           temper serve: application.forges[0].user: forge-1/user: absent\n\
           temper serve: application.forges[0].webhook_secret: forge-1/webhook_secret: absent\n\
           temper serve: application.identities[0].token: forge-1/git_token: absent\n"
      );
      scratch.remove();
  }
  ```
  The lines' order is `named.files`' and the `user` line's path is 07.3's: read both at the ready check.
- T3 `serve_on_the_owned_example_prepares_once_reconcile_has_made_its_files`: T2's directory plus `forge-1/` (0o700)
  with the four files, private, literal contents (`forge-1/user` holds `42`); `prepare_in` is `Ok`: the agent
  configuration, the engine and the setup agree (configuration.md, sections 5 and 8).
- T4 `the_owned_example_fits_the_native_client`, in `configuration.rs` beside `:3-24`'s test: its three assertions over
  `owned::EXAMPLE`'s `engine` member (parsed as `:9-15` do), and the three authority overrides from the JSON, each 8.
- The tier's owned tests (08.3, 10.2, 11.x) pass unchanged; none sets a member of the owned `application`.
Proof: process, the box's two tests (13.1's `temper_box_prints_the_link_and_a_stop_leaves_no_process`, 13.2's
  `temper_box_restarts_a_killed_forgejo_while_the_engine_serves`) on the whole example, once:
  ```sh
  export TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
  export TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
  sh -c 'export NEXTEST_DOUBLE_SPAWN=0; cargo build -p temper --bin temper && cargo nextest run -p temper --profile process -E "test(/^temper_box_/)"'
  ```
  The link is printed on stdout, Forgejo killed is restarted, a stop leaves no process (13.2's referee), 120 s at most
  each. 14.2 runs the guide's copy; README Done when 4's run is the rest of the tier on the file's `application`.
Don't: add a key the checks do not need (a comment, a `$schema`); read the files at run time by a path (`include_bytes!`
only); compute an expected value from the file (T1 to T4 write literals); build an owned member in a test.
Size: about 130 lines (the sketch with its bodies; all test support: the support count of code-organization.md, 6).
Removes: the owned `application` members that 08.1, 08.3, 10.2 and any 11 landed before write by hand in the tier's
  support.

### 14.2 The first milestone's setup guide, followed on x1

Scope: `docs/guide/box.md`, the walk of setup.md 12; a script that follows it to an adopted repository.
Needs: 14.1; `temper box` (13.1); the pinned binaries; `/usr/bin/git`; DNS for the provider's name.
Files: `docs/guide/box.md` (new); a fault the script finds in the example or the guide is fixed in `docs/examples/*` or
the guide, here. The script is `~/src/rust/tmp/temper/forge-setup/logs/temper-14.2/follow.sh`, not the repository's.
Shape:
1. **The guide:** eight sections headed `## Step N. <title>` (the script cuts by these), each with at most one `sh`
   block a person runs as it stands, with `$TEMPER`, `$FORGEJO` and `$RUNNER` set by step 1's to the example's own paths
   (`/usr/local/bin/temper`, `/opt/forgejo/forgejo`, `/opt/forgejo/forgejo-runner`): 1 what you need and the three
   variables (the pinned Forgejo 17.0.0 and runner 13.2.0, which temper does not download; git; free ports 3000, 7100,
   7101); 2 a private directory, `cp` of the example as `config.json` and of `owned/`, `sed` of `/home/me/temper`,
   `/opt/forgejo/forgejo`, `/opt/forgejo/forgejo-runner` and `/usr/local/bin/temper` (the runner's path first); 3 the
   files that hold secrets: `secrets/` and `accounts/records` (0o700), a random `secret` (32 bytes as hex) written with
   `printf '%s\n'` to `secrets/worker.secret`, and `printf %s "$secret" | sha256sum | cut -d' ' -f1 >
   secrets/worker.digest` (jig's digest file, `configuration.md` 10: 64 digits and a newline, of the secret without its
   final newline); 4 the LLM account handed in, `accounts/llm.json` in jig's format (`access_token`, `expires_at`,
   `account_id`), from a borrowed Codex login as `scripts/proof-live.sh` does: `TEMPER_LIVE_MODEL=gpt-6-luna live-login
   ~/.codex/auth.json DIR` (`tools/live-login`), its `live-account.json` moved into place; 5 `temper box config.json`,
   which keeps its terminal and prints the sign-in link; 6 Forgejo's web: sign in as `ada` with the password in
   `forge-1/person_password` and change it where Forgejo asks, since its API refuses her until then (setup.md 12;
   findings row 10); a repository `acme/widget` with Actions on, and `.forgejo/workflows/check.yaml` and `check.sh`
   (`test -s README.md`) committed to `main` (`runs-on: temper-host`, no `uses:`; it runs on a push to `temper/**`,
   where temper pushes its work; both texts the tier's, from 10.2); 7 `temper open client LINK`, the adoption form's
   fields (owner `acme`, name `widget`, landing branch `main`, check `1`, "It runs CI" kept as it is); 8 ask for a
   change, stop with Ctrl-C, start again.
2. **It says, in a line each:** everything is on loopback; the runner's jobs run as your user and read temper's files,
   so only your own code, never a stranger's pull request (setup.md 15); backups are yours, with the box stopped (16); a
   second box changes the three addresses together; the example's rule covers `acme/widget` alone; the box resolves the
   provider's name at its start; an account handed in is never refreshed, so a lapsed one is handed in again; the
   charters are short texts to edit; `temper sign-in` is the other way once mvp.md 7's registration is settled; steps 4
   and 8 need a real account and are the dogfood run's.
3. **The script** runs, in order, with a fresh directory under `logs/temper-14.2/` and `follow.log` holding each block's
   text before its output: blocks 1, 2, 3 as `sh -e`, block 1's three assignments first rewritten to the worktree's
   `target/debug/temper` (absolute), `$TEMPER_FORGEJO` and `$TEMPER_FORGEJO_RUNNER`; block 4 not run, the script writes
   in its place the account for skein's fake peer,
   `{"access_token":"token","expires_at":4102444800,"account_id":"acc"}`, private (`support/mod.rs:28-31`); block 5 in
   the background (`box.out`, `box.err`); the web steps as the person's HTTP requests: the sign-in and the
   password-change forms (`/user/login`; `/user/change_password`), each posted with the fields its page holds, its `_csrf` when it holds one (the pinned `/user/login` holds none; ruling 147), then the
   repository, `check.yaml`, `check.sh` and check (e)'s push as API requests under basic auth with the new password
   (routes from the pinned Forgejo's `swagger.v1.json`); block 7 with its lines fed through a pipe, each after the
   client printed the control it names (`process.rs:33-68`); then a stop (SIGTERM) and block 5 again.
4. **Its checks,** each `ok (<letter>) <what>` or exit 1 with the log's tail: (a) `box.out`'s first line starts
   `http://127.0.0.1:7100/jig/v1/sign-in/link?secret=` within 120 s; (b) `GET /api/v1/version` on 3000 answers 200 and
   `17.0.0`; (c) `forge-1/` holds `api_token`, `user`, `webhook_secret`, `git_token`, `admin_token`, `person_password`,
   each mode 600 (a `stat`); (d) `GET /api/v1/orgs/acme/hooks` as `ada`, who owns `acme` (setup.md 6.5), lists one
   active hook whose URL is `http://127.0.0.1:7101/hooks/1`; (e) a file pushed to a new branch `temper/guide` (contents
   API, `new_branch`) starts the workflow: that commit's status (`GET /api/v1/repos/acme/widget/commits/<sha>/status`)
   is `success` within 90 s, the owned runner having passed it, and the branch is then deleted (`DELETE /api/v1/repos/acme/widget/branches/temper/guide`, 204), since adoption refuses a branch under the deployment's prefix that it does not own (`temper-engine-domain-forge`'s `adoption.rs`, the `Branches` stage; ruling 151); (f) the client prints `Adopted acme/widget on forge 1`
   within 30 s and exits 0 on `q`; (g) the stop exits the box 0 within 60 s and `pgrep -f <the directory>` prints
   nothing; (h) the second start prints a link within 120 s; a stop again leaves no process.
Test shape: none; the script is the proof.
Proof: process, on x1 with the guide and the example committed, the exports of 14.1's Proof and
  ```sh
  cargo build -p temper --bin temper
  sh ~/src/rust/tmp/temper/forge-setup/logs/temper-14.2/follow.sh
  ```
  The script exits 0 after printing checks (a) to (h), and `follow.log` holds the commands and their output. Bound: 420
  s wall, the script's own timeouts as above, at most ten runs, each after the last one's cause, a new one, was found by a focused probe and handled in the script (rulings 150 and 151); a failure with a cause seen before, or a fault in temper or the example rather than the script, is a question.
Don't: run `temper sign-in`, `live-login` or anything that reaches the provider; use a binary other than the two pinned;
read a forge file but `person_password` (as the person); change the guide while a run is under way (a fix starts the
script again from step 1).
Size: 0 (a document). Landing: no gate if only Markdown changed, else the full gate.
Removes: nothing.

## Interface sketches

14.1 extends 08.1's `support::owned` (`Names`, `document(value, &Names)`, `configuration`, `prepare`, `files`,
`scripted`, `finished`; 10.1's and 10.2's) with the names below, renamed at the ready check on a clash.

```rust
// crates/temper/tests/support/owned.rs
use serde_json::Value;
use std::{net::SocketAddr, path::{Path, PathBuf}};

/// `docs/examples/owned.json`, as the repository holds it.
pub const EXAMPLE: &[u8] = include_bytes!("../../../../docs/examples/owned.json");
/// The files beside it that hold no secret, by their path in a copy: the agent's configuration, the charters.
pub const SHIPPED: [(&str, &[u8]); 3] = [
    ("owned/agent.json", include_bytes!("../../../../docs/examples/owned/agent.json")),
    ("owned/charters/chat.txt", include_bytes!("../../../../docs/examples/owned/charters/chat.txt")),
    ("owned/charters/coding.txt", include_bytes!("../../../../docs/examples/owned/charters/coding.txt")),
];
/// What a test changes in the example: its addresses, binaries and the landing rule's review.
pub struct Owned {
    /// Forgejo's `listen`, repeated by the forge's `api` and `git` and the landing rule's host segment.
    pub forgejo: SocketAddr,
    /// `application.clients.listener`.
    pub clients: SocketAddr,
    /// `application.webhooks.listener`.
    pub webhooks: SocketAddr,
    /// What the provider's name (`chatgpt.com:443`) becomes, in the document and the agent's configuration.
    pub llm: SocketAddr,
    /// `setup[0].mode.owned.forgejo`.
    pub forgejo_binary: PathBuf,
    /// `setup[0].mode.owned.runner.binary`.
    pub runner_binary: PathBuf,
    /// Whether the rule keeps the example's agent review (gate 1); the process tier asks for CI alone.
    pub review: bool,
}
impl Owned {
    /// The example's own values: ports 3000, 7100, 7101, `/opt/forgejo/forgejo{,-runner}`, the review, the fake
    /// peer's `127.0.0.1:34443` (`support/configuration.rs:29`).
    #[must_use]     pub fn example() -> Self {
        todo!()
    }
    /// The process tier's: three free loopback addresses, the binaries from 10.2's `binaries()` (both variables, a
    /// panic naming the missing one), the fake peer, no review.
    #[must_use]     pub fn tier() -> Self {
        todo!()
    }
}
/// The example's `application` member with `owned`'s values, and `charter` 1 (the tier has one charter).
#[must_use] pub fn application(owned: &Owned) -> Value {
    todo!()
}
/// The whole example for a start: `owned`'s values, `/home/me/temper` made `directory`, `/usr/local/bin/temper`
/// the binary under test.
#[must_use] pub fn example_document(directory: &Path, owned: &Owned) -> Vec<u8> {
    todo!()
}
/// What the guide's steps 2 to 4 make in `directory`: `SHIPPED` edited alike, the private `secrets/` and
/// `accounts/records`, the worker's secret and digest (literals), the account for skein's fake peer.
pub fn lay(directory: &Path, owned: &Owned) {
    todo!()
}
```

## Questions to raise when reached

None: the designer's answer 45 settled the charters' texts and the person's first password.

## Done when

1. `jq -c '.application.setup[0].mode.owned.listen, .application.webhooks.listener, .application.clients.listener'
   docs/examples/owned.json` prints `{"socket":"127.0.0.1:3000"}`, `{"socket":"127.0.0.1:7101"}`,
   `{"socket":"127.0.0.1:7100"}` in that order; `jq -e . docs/examples/owned/agent.json >/dev/null` exits 0; and `git
   ls-files docs/examples` prints the four files and no other: no secret is shipped.
2. `cargo nextest run -p temper-service -p temper -E 'test(/owned_example/)'` runs 4 tests, 4 passed.
3. `git grep -n '127.0.0.1:3000' -- crates/temper/tests ':!crates/temper/tests/support/owned.rs'` prints nothing: one
   place holds the example's addresses.
4. `git grep -n '"owned"' -- crates/temper/tests ':!crates/temper/tests/support/owned.rs'` prints nothing: no test
   builds an owned member.
5. The guide: `grep -c '^## Step ' docs/guide/box.md` prints 8; `awk '/^## Step /{n++} $0 ~ "^\140\140\140sh" {c[n]++}
   END {for (i in c) if (c[i] > 1) print "step", i}' docs/guide/box.md` prints nothing; and its lines (14.2, Shape 2)
   are there: `for w in 'on loopback' 'only your own code' 'backups are yours' 'three addresses' 'covers acme/widget
   alone' 'resolves the provider' 'handed in again' 'short texts' 'temper sign-in' 'steps 4 and 8'; do grep -q -- "$w"
   docs/guide/box.md || echo "missing: $w"; done` prints nothing.
6. 14.2's Proof ran: `grep -c '^ok (' ~/src/rust/tmp/temper/forge-setup/logs/temper-14.2/follow.log` prints 8, and `tail
   -n 2` of it prints the lines of checks (g) and (h).
7. README Done when 4's run, the whole process tier on the file's `application`, passed every test.
