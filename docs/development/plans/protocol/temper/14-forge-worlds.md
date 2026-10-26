# Session 14: the forge component wired, the fake forge's protocol layer, the protocol world, the conformance run through the component

The forge component of session 13 joins `temper-service`'s engine:
the root's `Request::Forge` goes to it and its answers come back to the
root, held and offered again when the root's admission cannot take them,
kept through the drain, its start check stopping the engine (exit 1,
naming the forge), its limits checked among startup's; 08.2's
`Unavailable` answers go. The fake forge then becomes a service
(domain/testing.md, 4.2): its domain grows the reads its routes need (Actions'
runs and jobs, an item's thread, the reviewer routes, a pull's merge
base, Forgejo's default order), and `testing/temper-fake-forge-protocol`
serves Forgejo's API over it on skein's HTTP server, from session 06's
server side, with Forgejo's quirks, the faults moved down to the wire,
and the requests counted by route. The protocol world
(`tests/engine/protocol-forge`) runs the forge connector world's stories
through bytes, streams cut at random, with the cost checks counted by
HTTP request. Last, `tools/forgejo-conformance` gains a mode that drives
the component against the throwaway Forgejo and records forge.md 13's
facts as the component sees them.

## Read first

- forge.md, sections 2 ("Room, and the drain"), 3.5, 9, 10 and 13;
  protocol.md, section 6 (the forge component, "A forge's start check");
  credentials.md, section 5; configuration.md, section 7; domain/testing.md,
  2.2 and 4.2; findings/temper-engine.md, 6.
- `temper/13-forge-component.md`: "Choices this plan makes" (the profile's
  values), the interface sketches, 13.5's machine world.
- `temper/06-forgejo-documents.md`: the interface sketches' `server`,
  `request`, `document`, `path` and `failure` modules.
- `temper/05-forgejo-tools.md`: "Choices this plan makes", 05.2 (the
  fixture, the cases) and 05.3 (`facts.txt`'s lines, the facts' names).
- `temper/08-service-engine.md` and `scratch/plan-sketches/src/s08.rs`:
  `temper_service::{tokens, limits, settings, service}` (`tokens::FORGE`,
  `limits::Limits`, `settings::Forge`, `service::{End, Stop,
  AccountsRuntime, Service::engine}`), 08.2's engine layer and its
  `Unavailable` answers (decision 15).
- `temper/02-adoption.md`, 02.1 (`api::Setup.id`, the fake's `Locate`).
- jig at temper's pin (`~/.cargo/git/checkouts/jig-aba39083bf77fefe/d91a89c`):
  `testing/jig-test-service/src/engine.rs:96-127` (an input the root
  refuses held, `room`), `stop.rs:1-80` (the drain and the components'
  close), `boundary.rs` (`End`, `Stop`), `src/tests/fixtures.rs:163-216`;
  `testing/jig-test/src/lib.rs:117-170` (an endpoint resolved, its trust
  roots read, its TLS configuration made).
- skein at temper's pin (`~/.cargo/git/checkouts/skein-336d5ae5eecfa0f9/b5d3a2c`):
  `testing/skein-fake-llm-protocol/src/provider.rs:1-500` (the
  precedent: a fake service's HTTP side over its domain, `Service`,
  `Server`, `start`, `up`, `down`, `resume`, `close`, `closed`);
  `crates/skein-http/src/server.rs:1-260`; `tests/llm-connection/tests/fake_protocol.rs`
  (a component joined to a fake protocol layer through bytes);
  `testing/skein-world/src/stream.rs`.
- temper: `testing/temper-fake-forge-domain/src/{lib.rs, boundary.rs,
  api.rs, domain.rs:44-130, 499-531, reads.rs:38-186, next.rs:121-228,
  ci.rs, pulls.rs:60-100, git.rs:360-375}` and `src/tests/next.rs:150-195`;
  `tests/engine/forge/src/world.rs` (the forge connector world: `submit`
  :475, `take_forge` :495), `tests/engine/forge/tests/simulation.rs` (its
  stories), `tests/engine/forge/client/src/{translate.rs, referee.rs}`;
  `crates/temper-engine-domain/src/admission.rs:13-58` (`try_step`) and
  `boundary.rs:229, 393-406`.
- decisions.md 12, 15 and 21; code-organization.md, sections 2 to 7.

## What the design asks

- **The forge component in the engine** (protocol.md, 6): "The root's
  `Request::Forge { call, repository, op }` goes to it; it answers each
  call with exactly one `client::Event::Answered { call, cost, result }`,
  which the routing hands the root as a forge event. An event the root
  cannot take yet comes back to the component, which holds it and offers
  it again ...; one the root returns during its drain stays with the
  component until it closes." "**A forge's start check.** ... another
  user, or a refusal of the token, stops the engine: the service drains
  as at a shutdown, and `temper serve` exits 1, naming the forge."
- **Its limits** (configuration.md, 7): the prefix `forge_component.`;
  "Limits checked against each other at start, each failure naming both
  limits: the forge component's: its connections equal the forge
  client's calls in flight; its deadline is at most the forge client's
  lifetime; its body bound holds the forge client's largest write and a
  key's marker; every listing's page size is at least 1".
- **Decision 15:** "Until then the engine's routing answers every
  `Request::Forge` with `Error::Unavailable` at cost 0 ...; 14.1 removes
  that arm."
- **The fake forge's protocol layer** (forge.md, 9): "The routes of
  section 3.2, and those of a forge's start and of resolution ... Each
  request is decoded, translated to the fake's `Read` or `Write` ..., and
  its answer encoded as Forgejo encodes it. Where a route needs what the
  fake's vocabulary does not read, the fake's domain grows that read:
  Actions' runs ... each job with its id and attempt; and the fake's CI
  statuses link their jobs in Forgejo's shape, relative to the origin,
  `<the path of html_url>/actions/runs/<run number>/jobs/<job index>`,
  with no `creator`, as Forgejo writes a status of its Actions." "its
  protocol layer gives each the id the
  scenario names, and answers `/repositories/{id}` with its path, `sha1`
  as its object format and `squash` ... as its merge style. A commit the
  fake counts is a 40-digit id, its count big-endian in its first eight
  bytes. The user a token stands for is the caller, from a table the
  scenario gives; an unknown token is answered `401`." "Forgejo's quirks,
  kept: the default order is newest created first; `sort=leastupdate`
  gives least recently updated first; a page is capped at the configured
  `max_response_items`; a size without a page is ignored where Forgejo
  ignores it; an issue's comments, a review's comments and a comparison
  come unpaged; collaborators come without their permissions; a merge
  answers with no body; the lookup `pulls/{base}/{head}` names the
  oldest of several pull requests, open or not; a pull request whose
  head adds nothing to its base is opened (`201`); deleting a branch
  that is not there answers `500` with an empty body." The fake's domain
  grows "the oldest pull request for a head and base, open or not, and
  the open pull requests, oldest first, for `Read::PullFor`'s requests
  (3.2)". "Faults move down ...: a call refused for the
  rate is a `429` with `Retry-After`; one failed before it is made is a
  refused connect or a reset before the request is read; one made and
  then timed out is a `5xx` or a dropped connection after the request was
  read; a late answer is a slow one." "Requests are counted, by route,
  by user and in total."
- **The protocol world** (forge.md, 10; domain/testing.md, 2.2): "the engine's
  forge component and the fake forge's protocol layer, joined by
  in-memory streams cut at random, running the forge connector world's
  stories ... through bytes: what the engine's domain asked is what the
  fake's domain did. A focused test of it aims at 0.5 s at most"; the
  cost checks "in the protocol world counting HTTP requests: idle ...;
  history-free ...; busy ...; honest costs: in the protocol world, the
  requests the fake served equal the sum of the costs the answers
  reported, the forges' starts' reads aside."
- **The real Forgejo check** (forge.md, 10): `tools/forgejo-conformance`
  "drives the facts of section 13 through plain requests and the forge
  component, says which hold, and records what it sent and received as
  transcripts, masked."
- **Secrets in tests** (code-organization.md, 5): "every world with a
  protocol layer checks that no value its fakes issued (... an API token)
  appears in a root's events or requests, a fact, a store record or a
  log (credentials.md, section 7)."

## Choices this plan makes

- **The routing's module** is `temper_service::forge` (the sketch's
  `service_forge`): the settings' and limits' translations and the
  answers' wrapping; the engine layer owns the component, its two output
  queues and its tokens, as jig's reference owns its components
  (`engine.rs`). An io event whose token's prefix is `tokens::FORGE`
  goes to `Component::up` with the prefix taken off; the component's io
  requests leave with it put on.
- **What the binary resolves** comes as `ForgeRuntime` (with
  `trust_roots` and `trust_root_bytes`, as `AccountsRuntime`; ruling 133), beside
  `AccountsRuntime`, wherever that is passed (`Service::engine` and its
  callers): the address resolved and the TLS configuration made, as jig's
  binary makes an issuer's (`testing/jig-test/src/lib.rs:117-170`).
  `forge::settings` pairs each `settings::Forge` with its runtime by
  number and makes `Host` with 13's `conventions::host`.
- **Answers to the root:** `forge::event(answered)` is stepped through
  `try_step`; its `Err(Event::Forge(..))` goes back as `Request::Refused`;
  the root's `Request::Busy(event)` as `Request::Returned`; one
  `Request::Offer` per `iterate` pass, after the root's release. A
  forge event `back` does not take is impossible from the routing (only
  the component's answers reach `Event::Forge`): its arm is
  `expect("the routing hands the root only the component's answers")`.
- **The drain** closes the component after the root's `Drained`, one more
  component in the drain's count of those waiting to close (jig's
  `stop.rs`); `Event::Stopped` starts the drain as a shutdown does and
  the service then ends `End::Stopped(Stop::Forge { number, why })`; the
  binary's exit line names the forge.
- **Limits:** `temper_service::limits::Limits` gains `forge_component`,
  its overrides `forge_component.<field>`, a duration's with `_ms`,
  TLS's `forge_component.tls.<field>`; the profile `small`'s values are
  13's table, made by `forge::small(&forge)` from the forge's own. A
  `limits::check` refusal becomes one problem per limit it names, at
  `application.limits.overrides.<dotted name>` (one member segment) with
  jig's `What::Bound`; a forge's own at `application.forges[i].<member>`
  (`api_token` `Bound`, `user` `Bound`, `api` `Plaintext`, `number`
  `Unique`).
- **The fake's domain grows in place** (code-organization.md, 6: "no
  type beside an existing one"): `Read::Job` names the job by its id, as
  Forgejo's log route does; `Read::Items` gains its `order`; `Pull`
  gains `merge_base`; new reads `Runs`, `Run`, `Jobs`, `Comments`; new
  writes `RequestReviewers`, `RemoveReviewers` (Forgejo's two routes;
  `SetReviewers` stays for the worlds that translate the client
  directly). For pull requests by head and base (forge.md, 3.2, "A pull
  request for a head and base"): `Read::PullFor` answers the open one,
  else the oldest (changed in place, as the forge client's vocabulary
  now says); new reads `FirstPull { head, base }`, the oldest open or
  not (Forgejo's lookup), and `OpenPulls { page, limit }`, the open pull
  requests oldest first; `Write::OpenPull` gains `opens_empty: bool`,
  true when a head that adds nothing to its base is opened as Forgejo
  opens it (the protocol layer's), false where `NothingToMerge` is
  answered as the vocabulary says (the direct worlds' translation). Runs and jobs live in a new `src/actions.rs` (`api.rs` is at
  728 lines and stays under 800). A CI status's link is
  `/<full name>/actions/runs/<number>/jobs/<index>`, which the protocol
  layer writes as it is, relative to the origin as Forgejo v16.0.5
  writes it (`Config.base` has no path), and a status the fake's CI
  posted is written with `"creator":null`. The ids are 02.1's (`api::Setup.id`); the
  token table is the protocol layer's, not the domain's.
- **The fake's protocol layer is shaped after skein's fake LLM provider:**
  a `Service` (the scenario's tables, the calls across connections, the
  tally, the faults' generator) and a `Server` per connection (skein-http's
  server, a tokenizer for a write's body with 06's `BodyDecoder`), free
  functions `start`, `up`, `down`, `resume`, `close`, `closed`. One
  request is one domain call (a write whose Forgejo answer is a
  document: its write, then one read of that document; ruling 137),
  but `/user`, `/settings/api` and
  `/users/search`, which its tables answer. The scenario gives each
  repository's protected branch (`Setup.protection` names it, and
  nothing changes it at run time), so `/branches/{b}` needs one call.
- **Faults move down:** the layer draws them itself, per request, from
  its `seed` (`Faults`), and the protocol world sets the domain's
  `unavailable` and `timeouts` chances to 0; the domain's rate limit
  stays the domain's (`RateLimited` → `429` with `Retry-After`), and its
  latency is the slow answer. A domain `Timeout` is written as a `500` or
  a dropped connection, drawn evenly.
- **The protocol world** is a library beside 13.5's machine world in
  `tests/engine/protocol-forge` (`src/protocol.rs`, `src/pipe.rs`): the
  forge connector's top (`temper-engine-domain-forge`) driven as
  `tests/engine/forge/src/world.rs` drives it, with the component and the
  fake's protocol layer between it and the fake's domain, joined by
  `pipe::Pipe`s. Its `Protocol` has the forge connector world's verbs,
  same names and arguments, so the chosen stories are copied with
  `World` replaced by `Protocol` and their expected values kept.
- **The conformance mode** is `forgejo-conformance --component --scratch
  <dir> --out <dir>`: the same fixture (05.2's), the component hosted on
  skein-shell's real ring, one call per line of forge.md 3.2, its facts
  written as a block of `facts.txt` named `component-...`, in place of
  that block, the other lines kept.

## Rules that bind it

- No increment "adds an `unreachable!` arm for a variant the service can
  reach: an arm the service reaches is routed" (code-organization.md,
  6): each component event and each root output the forge layer handles
  is matched without a wildcard.
- "**Large files.** ... An increment adds at most 40 lines to each"
  (README, "Rules"): here `testing/temper-fake-forge-domain/src/tests.rs`
  (14.2: a `mod actions;` line and the `order` field in its `Read::Items`
  literals), `tests/engine/domain/tests/forge_routes.rs` (14.2: one
  field), `tests/engine/forge/tests/simulation.rs` and
  `tests/engine/forge/client/tests/simulation.rs` (untouched; 14.4
  copies stories, it does not move them).
- "**The new worlds' share of the suites** ...: `tests/engine/protocol-forge`
  1.0 s and 8 s" (code-organization.md, 5), 13.5's machine world
  included: 14.4's stories aim at 0.7 s focused and 5 s fuzzy, summed;
  "A world past its share is made cheaper before more stories land in
  it; past twice it, a question."

## Increments

### 14.1 The forge component in the engine

Scope: protocol.md 6 and forge.md 2 in `temper-service`'s engine layer:
the component made from the settings, the binary's `ForgeRuntime`s and
`limits.forge_component` in `Service::engine` (its `limits::check`
refusals as start problems), `begin` once the engine's components start
(jig's start order: after the store, before the listeners); the root's
`Request::Forge` to `Component::down(Call)`; `Event::Answered` to the
root through `try_step`, a refused one back as `Refused`, `Offer` once
per pass, `Request::Busy` back as `Returned`; `Event::Stopped` to a
shutdown and `End::Stopped(Stop::Forge { number, why })`; the drain
closing the component after the root's `Drained`, `Event::Closed`
counted; io routed by `tokens::FORGE`; the component's deadline in the
engine's `next_deadline` and its `fire`; its worst case in the engine's
sum; `limits::Limits.forge_component` with its profile values and
overrides; the binary's exit line for `Stop::Forge` (`temper serve:
forge <number>: the API token belongs to user <id>, not <user>`, or
`temper serve: forge <number>: GET /api/v1/<user|settings/api> answered
<status>`), if the binary is on `origin/main`.
Needs: nothing beyond the session's start (08.5b's engine and 13.5's
component).
Files: `crates/temper-service/src/{forge.rs (new), engine.rs, route.rs,
stop.rs, limits.rs, lib.rs, tests.rs, tests/forge.rs (new),
tests/fixtures.rs}` (08's modules of these names, after jig's
reference's; where 08 named them otherwise, followed to its file);
`crates/temper-service/Cargo.toml` (`temper-engine-protocol-forge`,
`skein-tls`); the callers of `Service::engine`, each building
`ForgeRuntime`s beside `AccountsRuntime` (`git grep -n AccountsRuntime
-- crates tests` on `origin/main` when 14.1 starts: 10.1's `startup`,
09.3's `system`, 10.3's `crates/temper/src/*.rs`, 17.2's world, those
that exist); the users of `Stop` (`git grep -n 'Stop::' -- crates/temper
crates/temper-service tests`). Shared hotspots: `crates/temper-service/src/{engine.rs,
route.rs, lib.rs}`.
Shape: the sketch's `service_forge`, as `temper_service::forge`
(`ForgeRuntime`, `settings`, `shared`, `problems`, `event`, `back`,
`small`), the variant `Stop::Forge { number, why }`, the field
`Limits.forge_component`; `Service::engine(settings, seeds, accounts,
forges: List<ForgeRuntime>)`.
Test shape: unit tests in `src/tests/forge.rs` (`mod forge;` in
`tests.rs`), on 08's fixtures (`started`, `turn`, `engine`, as jig's
reference's `src/tests/fixtures.rs:163-216`), which gain `ForgeFace`:
the forge component's io answered by the test (`connected`, `answer(bytes)`,
`refuse`, each on a `skein_world::stream::Wire`; `skein-world` a
dev-dependency) and `forge_runtime()` (forge 1, plaintext,
`127.0.0.1:3000`). Tests:
`a_refused_start_check_stops_the_engine_naming_the_forge`,
`a_forge_call_waiting_past_its_deadline_reaches_the_root_unavailable`,
`an_answer_the_root_cannot_take_is_offered_again_next_pass`,
`an_answer_returned_in_the_drain_is_never_offered_again`,
`forge_component_limits_that_disagree_are_problems_naming_both`,
`forge_component_overrides_take_dotted_names_and_ms`. Each under 20 ms.
Reference test, in full:

```rust
#[test]
fn a_refused_start_check_stops_the_engine_naming_the_forge() {
    let mut service = fixtures::started(&fixtures::engine_document());
    let mut forge = fixtures::ForgeFace::new();
    forge.connected(&mut service);
    forge.answer(&mut service, b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\n\r\n");
    for _ in 0..64 {
        fixtures::turn(&mut service);
    }
    assert_eq!(
        service.end(),
        Some(&End::Stopped(Stop::Forge {
            number: 1,
            why: Stopped::Refused { read: StartRead::User, status: 401 },
        }))
    );
}
```

Proof: unit, `cargo nextest run -p temper-service -E
'test(/forge::/)'`, run once: a refused token stops the engine naming
forge 1, a call reaches the root once, an answer the root refused is
offered again and one returned in the drain is not, and disagreeing
limits are problems naming both (the six tests); under 0.3 s. The joint
layer for the component is 14.4's world; the process tier's story
through the binary is 15.3's.
Don't: let the API token reach the root, a fact, the store or a log;
answer a `Request::Forge` in the routing; keep a second `Stop` for the
forge.
Size: at most 350 lines.
Removes: 08.2's arm that answers `Request::Forge` with
`Error::Unavailable` at cost 0, and its test (decision 15).

### 14.2 The fake forge domain grows its routes' reads

Scope: forge.md 9 and domain/testing.md 4.2 in `testing/temper-fake-forge-domain`:
Actions runs and jobs (`src/actions.rs`): a run per commit CI starts on
in a repository (`ci::start`), its id from a counter across the forge
and its number from its repository's counter, both from 1; a job per
check context, its id from a counter across the forge, its index the
context's position in `Checks.contexts` from 0, its attempt 2 once its
context ran again (`ci::rerun`); `CheckSummary.link` of a CI status
`/<full name>/actions/runs/<number>/jobs/<index>`, of another user's
status empty; `JobRef { run, job, attempt }` the run's and job's ids;
reads `Runs { commit, number }`, `Run { run }`, `Jobs { run }`, and
`Job { job, attempt, max_bytes }` (changed in place: by the job's id);
`Comments { number }` (the thread, oldest first, unpaged, within the
fake's limits); writes `RequestReviewers`, `RemoveReviewers` (each a
user who may read, not the pull's author; needs write permission, as
`SetReviewers`); `Pull.merge_base` (the newest commit head and base
share, `None` when they share none); `Read::Items.order`
(`Order::LeastUpdated`, the order today, or `Order::NewestCreated`,
Forgejo's default); `Read::PullFor` the open pull for head and base,
else the oldest (`store.rs:267`'s `newest_pull` becomes the oldest, and
the open one is asked first; its doc in `api.rs` says so); `Read::FirstPull
{ head, base }` (the oldest, open or not: `Answer::Pull`, or
`Missing(Pull)`); `Read::OpenPulls { page, limit }` (open pull requests
by number, the `page`th `limit` of them: `Answer::Pulls(Box<[Pull]>)`,
each as `pulls::view` makes it); `Write::OpenPull.opens_empty` (true: a
head with no commit its base lacks is opened, not `NothingToMerge`; a
head branch that is the base branch stays `NothingToMerge`).
Each new read and write needs the permission
`domain.rs:499-531` gives its kind (reads: read; writes: write).
Needs: nothing beyond 14.1.
Files: `testing/temper-fake-forge-domain/src/{actions.rs (new), api.rs
(at most 800 lines after), lib.rs, reads.rs, next.rs, ci.rs, pulls.rs,
store.rs, limits.rs, domain.rs, tests.rs (a `mod actions;` line; the
`order` field in its 9 `Read::Items` literals, at most 40 lines),
tests/next.rs (the 4 `Read::Job` calls), tests/actions.rs (new)}`; the
users of the changed shapes: `tests/engine/forge/client/src/translate.rs`
(`:83` `order: LeastUpdated`; `:109` `Job { job: attempt.job, attempt:
attempt.attempt, max_bytes }`; its `OpenPull` with `opens_empty: false`), `tests/engine/forge/client/src/world.rs:169`,
`tests/fake-forge/tests/memory.rs:423`,
`tests/engine/domain/tests/forge_routes.rs:2811` (one field each), the
exhaustive matches of the fake's `Read`, `Write` and `Answer`
(`tests/engine/forge/client/src/{world.rs:721, referee.rs:53}`,
`tests/fake-forge/tests/memory.rs:308`, `testing/temper-fake-forge-domain/src/domain.rs:511-519`).
Shape: the sketch's `fake_actions` (`Run`, `Job`, the variants), and the
comments' changes there. `Answer::Comments(Box<[Comment]>)`.
Test shape: unit tests in `src/tests/actions.rs`, on the crate's
`Harness`, `setup()`, `CALM`, `ENGINE`, `PERSON`, `FIRST`, `read`
(`src/tests.rs:44-160`): `a_failing_ci_status_links_its_run_number_and_job_index`,
`a_commits_run_is_found_by_its_number_and_its_jobs_by_run`,
`a_job_log_is_read_by_job_id_and_attempt`,
`a_rerun_context_answers_its_second_attempt`,
`a_thread_is_one_read_oldest_first`,
`reviewers_are_requested_and_removed_by_their_own_writes`,
`items_come_newest_created_first_in_forgejos_default_order`,
`pull_for_answers_the_open_pull_before_an_older_closed_one` (pull 1 of
`work` into `main` closed, pull 2 of the same opened: `PullFor` answers
2, `FirstPull` answers 1, `OpenPulls { page: 1, limit: 2 }` answers 2
alone), `an_empty_pull_opens_only_when_opens_empty` (a branch at
`main`'s tip: `OpenPull` with `opens_empty: false` is `NothingToMerge`,
with `true` is `Created`). Each under 20 ms. In `setup()`'s repository CI starts first on the pushed `work`
(`ci::start` runs from `git.rs:374`, `pulls.rs:99` and `issues.rs:439`
only), so `work`'s run is number 1. Reference test, in full:

```rust
#[test]
fn a_failing_ci_status_links_its_run_number_and_job_index() {
    let mut configured = setup();
    configured.checks.passes = 0;
    let mut h = Harness::with(CALM, configured);
    let work = h.commit(FIRST, &[(b"src", b"one")]);
    h.push(ENGINE, b"work", work).unwrap();
    h.settle();
    let Answer::Checks(checks) = h.ok(PERSON, read(Read::Checks { commit: work })) else { unreachable!("checks") };
    let check = checks.first().unwrap();
    assert_eq!(check.status.state, Check::Failed);
    assert_eq!(&*check.link, b"/ai/temper/actions/runs/1/jobs/0");
    let Some(job) = check.job else { panic!("a failing CI status names its job") };
    let Answer::Runs(runs) = h.ok(PERSON, read(Read::Runs { commit: work, number: 1 })) else { unreachable!("runs") };
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].commit, work);
    assert_eq!(runs[0].id, job.run);
}
```

Proof: unit, `cargo nextest run -p temper-fake-forge-domain -p
temper-engine-forge-client-world -p temper-engine-forge-world`, run
once: a failing CI status links its run's number and its job's index,
a run and its jobs are found as Forgejo's routes find them, a log is read
by its job's id, a pull is found for its branches open first, and the
forge connector worlds' stories pass with the changed `Job` and
`PullFor` reads (the nine new tests and those worlds' suites); under
3 s.
Don't: let a link carry the protocol layer's base; draw runs' or jobs'
ids from the seed (counters, so a seed replays); change a fake decision
the direct worlds rely on beyond the shapes above.
Size: at most 350 lines.
Removes: the fake's `/job/<commit>` links.

### 14.3 `testing/temper-fake-forge-protocol`

Scope: forge.md 9's protocol layer (step code): per connection
skein-http's server; each request routed by 06's `server::route`
(`404` or `405` for one it does not route), its `Authorization: token`
looked up in `Config.tokens` (`401` for none or another, counted
`unauthorized`), a write's body decoded by 06's `BodyDecoder` (`422` for
a malformed one); the request counted in the `Tally` (total, by route,
by user); a fault drawn (`reset`: the connection closed before the
request is read; then, after the domain made the call, `drop` or
`server_error`); `/user`, `/settings/api` and `/users/search` answered
from the tables; each other route as one domain `Call` by the token's
user on the repository's full name (by id from `Config.repositories`,
or by path); a write whose Forgejo answer is a document, its write
`Call`, then one read `Call` of that document, answered with what the
read returns, the fault drawn after the write (ruling 137); its reply
encoded by 06's `server::encode` with Forgejo's
statuses (the table below); Forgejo's quirks: the default order
(`sort` other than `leastupdate`: `Order::NewestCreated`), the lookup
`pulls/{base}/{head}` by `FirstPull` (the oldest, open or not), the
open pulls' listing by `OpenPulls`, `POST R/pulls` by `OpenPull` with
`opens_empty: true` (a head that adds nothing to its base opened,
`201`), a `DeleteBranch` its domain answers `Missing(Branch)` written
`500` with an empty body, the page cap
(`limit` at most `max_response_items`; a `limit` without a `page`
ignored, `default_paging_num` in its place), the unpaged thread, review
comments and comparison, collaborators without permissions, a merge
answered `200` with no body, `X-HasMore` on a pull's files, `Range` on
raw reads (`206` with `Content-Range`); a `Do` other than `squash`
answered `405` with `failure::answer(MergeStyle)`; commit ids as 40
digits with the count big-endian in the first eight bytes (an id with
another byte set: `404`); user objects with logins from `Config.users`
(`u<id>` for an id not there); `html_url` behind `Config.base`; a
status's job link as the domain makes it, relative to the origin, and
a status by the fake's CI user (`ci` of the domain's limits) with
`"creator":null`, as Forgejo writes Actions' statuses.
Needs: nothing beyond 14.2.
Files: `testing/temper-fake-forge-protocol/{Cargo.toml, src/{lib.rs,
service.rs, server.rs, routes.rs, answers.rs, errors.rs, tally.rs,
tests.rs, tests/fixtures.rs}}` (`Cargo.toml` after
`testing/temper-fake-forge-domain/Cargo.toml`, linking `skein-lib`,
`skein-http`, `skein-json`, `temper-forge-forgejo`,
`temper-fake-forge-domain`); the workspace `Cargo.toml` (member,
`[workspace.dependencies]`); `tests/engine/protocol-forge/{Cargo.toml,
src/{pipe.rs (new), joint.rs (new)}, tests/fake_protocol.rs (new)}`. Shared hotspot: the
workspace `Cargo.toml`.
Shape: the sketch's `temper_fake_forge_protocol`. The domain's errors as
statuses:

| Domain `Error` | Status, message |
|---|---|
| `Forbidden` | `403` |
| `Missing(_)` | `404`; on `DeleteBranch`, `500` with an empty body |
| `RateLimited { reset }` | `429`, `Retry-After` the seconds to `reset`, rounded up |
| `Exists` | `409` (on `CreateBranch` and `OpenPull`) |
| `Stale` | `409`, `failure::answer(HeadOutOfDate)` |
| `Conflict` | `405` `MergeConflict` on a merge; `409` `UpdateConflict` on an update |
| `Closed` | `405`, `MergeClosed` |
| `Protected` | `405` `MergeProtected` on a merge; `403` on a deletion |
| `NothingToMerge` | `422`, no message (only a merge reaches it: an opening passes `opens_empty: true`) |
| `Empty` | `422`, `Empty` |
| `Full`, `TooLarge` | `413` |
| `Refused`, `Circular` | `422` |
| `Unavailable`, `Unreachable` | `503` |
| `Timeout` | `500`, or the connection dropped, drawn evenly |

Test shape: unit tests in `src/tests.rs`, fixtures in
`src/tests/fixtures.rs` (`config()`: base `http://127.0.0.1:3000`,
tokens `TOKEN` for user 1, users 1 `engine` and 2 `person`, repository
2 `org/repo` with no protection, `max_response_items` 2,
`default_paging_num` 30, no faults; `Harness`: a service, one server, the
fake's domain behind it with `fake_config()` of
`tests/engine/forge/src/world.rs:542`, `request(bytes)`, `run()`,
`status() -> u16`, `head() -> Vec<u8>`, `body() -> Vec<u8>`). Tests:
`an_unknown_token_is_answered_401_and_counted`,
`the_issue_listing_pages_leastupdate_capped_at_max_response_items`,
`a_limit_without_a_page_takes_the_default_paging_num`,
`a_merge_answers_200_with_no_body`,
`requests_are_counted_by_route_user_and_total`,
`a_reset_closes_before_reading_and_a_timeout_drops_or_answers_500`,
`deleting_a_missing_branch_answers_500_with_an_empty_body`,
`a_pull_with_nothing_to_merge_is_opened_201`,
`a_ci_status_has_no_creator_and_a_link_relative_to_the_origin` (a
failing CI status on a pushed commit, read by `GET
R/commits/<sha>/status`: `"creator":null`, `"target_url":"/org/repo/actions/runs/1/jobs/0"`).
Each under 20 ms. Joint, in `tests/engine/protocol-forge/tests/fake_protocol.rs`
(13.5's world crate, with `src/pipe.rs`, 14.4's pipe, made here, and
`src/joint.rs`: `Joint`, the component and this layer with the fake's
domain over one `Pipe`, its `new(seed)`, `issue(title, body)` (an issue
opened by user 2 through the domain's call), `call`, `run_for`,
`answered`, `tally`):
`the_component_starts_and_lists_items_through_the_fakes_bytes` (13's
component, its forge's user 1, and this layer over one `Pipe`, no cut:
the start reaches Ready with cap 2, a `Read::Items` answers the fake's
issue at cost 2 (the repository resolved by id, then listed), the
tally's total 4),
`pull_for_answers_the_open_pull_when_the_lookup_names_a_closed_one`
(user 2's pull request of `work` into `main` closed, then a second
opened for them, through the domain's calls; `Read::PullFor { head:
work, base: main }` answers the second, open, at cost 4: the repository
resolved, the lookup naming the first, one page of `pulls?state=open`,
the status read), and
`opening_with_nothing_to_merge_writes_no_pull_request` (a branch `same`
at `main`'s tip; `Write::OpenPull { head: same, base: main }` answers
`Err(NothingToMerge)` at cost 4: the repository resolved, two branch
reads, the comparison; the tally's `Route::OpenPull` 0, the fake holds
no pull request), each under 0.1 s. Joint's helpers grow `branch(name,
at)`, `pull(head, base)` and `close(number)`, each the domain's call by
user 2. Reference tests, in full:

```rust
#[test]
fn an_unknown_token_is_answered_401_and_counted() {
    let mut h = fixtures::Harness::new(fixtures::config());
    h.request(b"GET /api/v1/user HTTP/1.1\r\nHost: 127.0.0.1:3000\r\nAuthorization: token 0000000000000000000000000000000000000000\r\n\r\n");
    h.run();
    assert_eq!(h.status(), 401);
    assert_eq!(h.service().tally().unauthorized, 1);
    assert_eq!(h.service().tally().total, 1);
    assert_eq!(h.domain_calls(), 0);
}
```

```rust
// tests/engine/protocol-forge/tests/fake_protocol.rs
#[test]
fn the_component_starts_and_lists_items_through_the_fakes_bytes() {
    let mut joint = Joint::new(3);
    joint.issue(b"first", b"body");
    joint.run_for(Duration::from_secs(1));
    joint.call(1, REPO, Op::Read(Read::Items { since: Time::ZERO, page: 1, kind: None }));
    joint.run_for(Duration::from_secs(1));
    let answered = joint.answered(1).expect("one answer");
    assert_eq!(answered.cost, 2, "the repository resolved by id, then listed");
    let Ok(Answer::Items { items, more: false, .. }) = &answered.result else {
        panic!("one page: {:?}", answered.result)
    };
    assert_eq!(items.len(), 1);
    assert_eq!(&*items[0].title, b"first");
    assert_eq!(joint.tally().total, 4, "the start's two reads, then the call's two");
}
```

Proof: world, `cargo nextest run -p temper-fake-forge-protocol -p
temper-engine-protocol-forge-world -E 'binary(fake_protocol) |
package(temper-fake-forge-protocol)'`, run once: the component starts
against the fake's bytes and lists an issue the fake holds, an unknown
token is a `401`, a page is capped as Forgejo caps it, and the tally
counts by route and user, a missing branch's deletion is a `500`, a
pull with nothing to merge is opened, and through the component
`PullFor` finds the open pull and `OpenPull` with nothing to merge
writes nothing (the nine unit tests and the three joint ones); under
0.5 s.
Don't: keep state of a call past its reply; answer from the domain's
store but through a call (a world's `inspect` is a world's); write a
field 06's documents do not carry.
Size: at most 600 lines; its allowance 2400, its parts summed
(decisions.md, ruling 139).
Removes: nothing.

### 14.4 The protocol world

Scope: forge.md 10's protocol world in `tests/engine/protocol-forge`:
`Protocol` (`src/protocol.rs`): the forge connector's top with the
forge world's `LIMITS`, the forge component (13's fixtures' limits with
the forge world's client limits), the fake's protocol layer and domain
(`fake_config()` with `unavailable` and `timeouts` 0; `org/repo`, id 2,
no protection, users 1 (admin, the forge writer, the API token's user)
and 2 (write), as `tests/engine/forge/src/world.rs:112-140` sets it up),
joined by `pipe::Pipe`s that deliver bytes in drawn fragments and cut a
connection at a drawn byte when the world cuts; the forge connector
world's verbs (`tests/engine/forge/src/world.rs:168-360`) with the same
names and arguments; its stories through bytes; the cost checks counted
by HTTP request; the API token's check; a fuzzy sweep with cuts and the
layer's faults.
Needs: nothing beyond 14.3.
Files: `tests/engine/protocol-forge/{Cargo.toml, src/{lib.rs,
protocol.rs (new), pipe.rs, fixtures.rs}, tests/{protocol.rs (new),
fuzzy_protocol.rs (new)}}`; `src/fixtures.rs` gains `goal_view()`, the
`issues::GoalView` of `tests/engine/forge/tests/fuzzy_simulation.rs:13-22`.
Shape: ordinary Rust, sketched below ("The protocol world").
Test shape: focused tests in `tests/protocol.rs`, the stories copied
from `tests/engine/forge/tests/simulation.rs` with `World` as
`Protocol`, names and expected values kept:
`a_change_produced_opened_checked_queued_and_landed` (:708),
`a_created_issue_with_its_answer_lost_is_found_after_restart` (:906),
`a_merge_with_its_answer_lost_is_found_after_restart_without_merging_twice`
(:984), `a_failed_ci_head_is_repaired_then_checked_again` (:674),
`an_external_move_of_a_held_branch_is_reported_as_drift` (:1320),
`idle_forge_call_cost_does_not_follow_old_issue_history` (:1395, its
costs as the tally's requests); and the world's own:
`honest_costs_equal_the_requests_the_fake_served`,
`a_busy_pass_lists_at_most_changed_items_over_the_page_plus_one`,
`the_api_token_reaches_no_record_request_or_fact`. Fuzzy in
`tests/fuzzy_protocol.rs`:
`stories_through_cut_streams_answer_each_call_once` (seeds 0 to 15, cut
chance 100 in 1,000 connections, the layer's faults `reset` 20, `drop`
20, `server_error` 20: each call answered once, by the forge client
world's referee rule (`tests/engine/forge/client/src/referee.rs`); the
keyed issue made once; the requests the fake served at most the
reported costs, the starts' reads aside: a reset before the read leaves
a cost the fake never served). Ceilings: each focused test under 0.08 s,
the fuzzy test about 5 s. Reference tests, in full:

```rust
#[test]
fn honest_costs_equal_the_requests_the_fake_served() {
    let mut world = Protocol::new(11);
    world.adopt();
    world.push(b"work", b"one");
    world.open_pull(b"work", b"main");
    world.run_for(60);
    let tally = world.tally();
    let starts = tally.routes.get(&Route::User).copied().unwrap_or(0)
        + tally.routes.get(&Route::ApiSettings).copied().unwrap_or(0);
    assert!(world.costs() > 0, "the story made calls");
    assert_eq!(world.costs(), tally.total - starts);
}
```

```rust
// tests/engine/protocol-forge/tests/fuzzy_protocol.rs
#[test]
fn stories_through_cut_streams_answer_each_call_once() {
    let faults = Faults { reset: 20, drop: 20, server_error: 20 };
    for seed in 0..16 {
        let mut world = Protocol::cutting(seed, 100, faults);
        world.adopt();
        world.event(top::Event::Project { entry: 1, repository: REPO, view: fixtures::goal_view() });
        world.event(top::Event::Committed { entry: 1 });
        world.run_for(seed % 3);
        world.restart();
        world.run_for(60);
        assert_eq!(world.issues_made(), 1, "seed {seed}: the keyed issue made once");
        assert!(world.stored().get(&top::Key::Entry(1)).is_none(), "seed {seed}: the entry settled");
        assert!(world.tally().total - world.start_reads() <= world.costs(), "seed {seed}: no request uncounted");
    }
}
```

Proof: world, `cargo nextest run -p temper-engine-protocol-forge-world`
and `cargo nextest run -p temper-engine-protocol-forge-world
--profile fuzzy -E 'binary(fuzzy_protocol)'`, each run once: the forge
connector world's six stories end as they end in the direct world, the
idle cost does not follow history, each request the fake served is in
some answer's cost, and no API token reaches the top's records,
requests or facts (the nine focused tests and the fuzzy one); focused
summed under 1.0 s with 13.5's, fuzzy under 8 s with 13.5's. The joint
layer with the engine's root is 17's one-process world.
Don't: change a copied story's expected values (a difference through
bytes is a bug, or a question with the story's name); move the stories
out of `tests/engine/forge`; spawn a thread.
Size: world code only (production 0).
Removes: nothing.

### 14.5 The conformance run through the component

Scope: forge.md 10 and 13: `tools/forgejo-conformance --component`
(`src/component.rs`): the throwaway Forgejo started and provisioned as
05.2 does (`fixture.rs`), the component made for one forge (plaintext,
the instance's loopback port, `writer`'s token and id), hosted with
`skein_io::Io` on skein-shell's ring (`skein_shell::Kernel::open`,
`skein_shell::drive`, a `Host` of the tool's own that routes io to the
component); the start, then one call per row of forge.md 3.2 (and
`Locate`) against the fixture's repository, each answer checked against
what the fixture holds; a second component given `stranger`'s token (another user) and a
40-zero token for the start's stops; the requests the component sent
counted from its sockets' sends (a send beginning with a method and
` /api/v1/`); the facts below written as the `component-` block of
`<out>/facts.txt`, the block replaced, the other lines kept.
Needs: 05.3's `facts.txt` and its line format (the README's table).
Files: `tools/forgejo-conformance/{Cargo.toml (`temper-engine-protocol-forge`,
`temper-engine-domain-forge-client`, `skein-shell`, `skein-io`),
src/{component.rs (new), facts.rs, main.rs, tests.rs}, facts.txt,
README.md (at most 10 more lines)}`.
Shape: `forgejo-conformance --component --scratch <dir> --out <dir>`;
`component::run(forgejo: &Forgejo, scratch: &Path) -> Result<Vec<Fact>,
Failure>`; `facts::replace_block(text: &str, prefix: &str, facts:
&[Fact]) -> String`. The facts, in this order:
`component-start-ready`, `component-start-other-user-stops`,
`component-start-wrong-token-stops`, `component-read-items`,
`component-read-item`, `component-read-pull`, `component-read-pull-for`,
`component-read-reviews`, `component-read-statuses`,
`component-read-remarks`, `component-read-branch`,
`component-read-branches`, `component-read-pull-files`,
`component-read-compare`, `component-read-checks`,
`component-read-job`, `component-read-file`,
`component-read-protection`, `component-read-settings`,
`component-read-collaborators`, `component-read-permission`,
`component-read-locate`, `component-write-create-issue`,
`component-write-open-pull`, `component-write-post`,
`component-write-review`, `component-write-edit`,
`component-write-set-reviewers`, `component-write-close`,
`component-write-reopen`, `component-write-merge`,
`component-write-update`, `component-write-status`,
`component-write-create-branch`, `component-write-delete-branch`,
`component-keys-found-on-read-back`,
`component-costs-equal-requests-sent`,
`component-read-pull-for-open-after-closed` (a pull request of a branch
into `main` closed, then a second opened for them, by plain requests:
`PullFor` answers the second, open),
`component-write-open-pull-no-changes` (a branch at `main`'s tip:
`OpenPull` answers `NothingToMerge`, and a plain read of the
repository's pull requests shows none of that branch) (39). A call's fact `holds` when
it answers `Ok` with what the fixture holds (a write: and a read after it
shows the effect), `fails <name>: <the Error or the answer seen>`
otherwise; without `TEMPER_FORGEJO_RUNNER`, `component-read-checks` and
`component-read-job` are `unchecked ...: TEMPER_FORGEJO_RUNNER not set`.
Test shape: unit tests in `src/tests.rs`:
`the_component_block_is_replaced_and_other_lines_kept`,
`a_sent_request_is_counted_by_its_request_line` (a table: a head, a body
piece, two heads in one send). Each under 20 ms. Reference test, in
full:

```rust
#[test]
fn the_component_block_is_replaced_and_other_lines_kept() {
    let before = "# forgejo forgejo version 16.0.5 sha256 2c54\n# runner none\nholds merge-answers-200-with-no-body\nholds component-read-items\n";
    let facts = [Fact { name: "component-read-items", outcome: Outcome::Fails("TooLarge".to_owned()) }];
    let after = facts::replace_block(before, "component-", &facts);
    assert_eq!(
        after,
        "# forgejo forgejo version 16.0.5 sha256 2c54\n# runner none\nholds merge-answers-200-with-no-body\nfails component-read-items: TooLarge\n"
    );
}
```

Proof: process (a real Forgejo), after `sha256sum -c` in both of the
README's directories prints `OK`:
`TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v16.0.5/forgejo-16.0.5-linux-amd64
TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v12.13.2/forgejo-runner-12.13.2-linux-amd64
cargo run -p forgejo-conformance -- --component --scratch
target/forgejo-conformance --out tools/forgejo-conformance`, run once:
exit 0, and `grep -cE '^(holds|fails|unchecked) component-'
tools/forgejo-conformance/facts.txt` prints 39; under 3 minutes. Each
`fails` line is reported to the coordinator as a block, kind design
gap, quoting the line and forge.md's text it contradicts, unless the
design names its fallback (README, "A design gap is a question").
`facts.txt` is committed.
Don't: write a token, a password or the API token's `Debug` to a file or
stdout; change a transcript (this mode records none); register the runner
with another instance; touch any Forgejo but the one `TEMPER_FORGEJO`
names.
Size: at most 300 lines.
Removes: nothing.

## Interface sketches

The same text as `scratch/plan-sketches/src/s14.rs`, which compiles
under the workspace's lints with `crate::s06` standing for
`temper_forge_forgejo`, `crate::s08` for `temper_service`'s 08 parts and
`crate::s13` for `temper_engine_protocol_forge`.

```rust
//! Sketches of session 14: the forge component wired into `temper-service`'s engine (14.1), the
//! fake forge domain's growth (14.2) and the fake forge's protocol layer (14.3). The protocol
//! world (14.4) and the conformance tool's mode (14.5) are ordinary Rust, in the session file.
//!
//! Who owns what: the engine's routing owns the forge component's token prefix (`tokens::FORGE`)
//! and carries its answers to the root and back; the component owns its connections. The fake
//! forge's protocol layer owns its connections and the tally of requests; the fake's domain owns
//! what the forge holds. What ends what: as session 13's; a fake connection ends with
//! `Event::Closed`, the last event of its server.
#![expect(unused_variables, reason = "a sketch's bodies are unreachable; its parameters are its signature")]

// temper_service::forge (14.1, a new module `crates/temper-service/src/forge.rs`): the forge
// component in the engine's layer. Step code; keeps nothing but what each function is given.
pub mod service_forge {
    use crate::s08::{limits, settings};
    use crate::s13::temper_engine_protocol_forge as component;
    use alloc::boxed::Box;
    use component::boundary::{Answered, Stopped};
    use component::limits::{Shared, Unusable};
    use jig_protocol_config::Problems;
    use skein_lib::List;
    use temper_engine_domain as engine;
    use temper_engine_domain_forge as forge;

    /// What the binary resolved for one forge, as jig's reference binary resolves an issuer
    /// (`testing/jig-test/src/lib.rs:117-170` at the pin): handed to `Service::engine` beside
    /// `AccountsRuntime`.
    #[derive(Debug)]
    pub struct ForgeRuntime {
        /// The forge's number, as `settings::Forge.number`.
        pub number: u16,
        /// `api` resolved: the first address a host name gives.
        pub address: core::net::SocketAddr,
        /// The transport made from `settings::Transport`: the server name and trust roots read.
        pub transport: component::settings::Transport,
    }

    /// The component's settings, one forge per configured forge, in order; a forge with no
    /// runtime, or a runtime with no forge, is a problem at `application.forges[i].api`.
    pub fn settings(
        forges: &[settings::Forge],
        runtime: List<ForgeRuntime>,
        problems: &mut Problems,
    ) -> Option<component::settings::Settings> {
        unreachable!("sketch")
    }

    /// What the component is given of the forge's and io's limits.
    #[must_use]
    pub fn shared(limits: &limits::Limits, io: &skein_io::Limits) -> Shared {
        unreachable!("sketch")
    }

    /// One refusal as start problems, one per limit it names (configuration.md, 7: "each
    /// failure naming both limits"), at `application.limits.overrides.<dotted name>`.
    pub fn problems(unusable: Unusable, problems: &mut Problems) {
        unreachable!("sketch")
    }

    /// The root's event for an answer: `Event::Forge(Client(Answered { call, cost, result }))`.
    #[must_use]
    pub fn event(answered: Answered) -> engine::Event {
        unreachable!("sketch")
    }

    /// An answer handed back by the root (`try_step`'s `Err(Event::Forge(..))`, or
    /// `Request::Busy`); another forge event comes back unchanged.
    pub fn back(event: Box<forge::Event>) -> Result<Answered, Box<forge::Event>> {
        unreachable!("sketch")
    }

    /// The variant 14.1 adds to `temper_service::Stop` (08.2's, `crate::s08::service::Stop`).
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Stop {
        /// A forge's start check refused: the drain runs as at a shutdown, then the service ends
        /// `End::Stopped(Stop::Forge { .. })` and `temper serve` exits 1, naming the forge.
        Forge { number: u16, why: Stopped },
    }

    /// The field 14.1 adds to `temper_service::limits::Limits`: `forge_component`, its
    /// overrides `forge_component.<field>` (`_ms` for a duration, `tls.<field>` for TLS's).
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Limits {
        pub forge_component: component::limits::Limits,
    }

    /// The profile `small`'s values for the component, derived where configuration.md 7
    /// ties them to the forge's: `connections` the client's `calls`, `deadline` its
    /// `lifetime`, `repositories` the forge's, `body_bytes` at least `2 × op_bytes + 21`.
    #[must_use]
    pub fn small(forge: &forge::Limits) -> component::limits::Limits {
        unreachable!("sketch")
    }
}

// temper_fake_forge_domain (14.2): what its routes need that the fake's vocabulary lacks.
// Session 02.1 gave repositories their ids (`api::Setup.id`).
pub mod fake_actions {
    use alloc::boxed::Box;

    /// An Actions run (`src/actions.rs`, new, re-exported by `api`): one per commit CI starts on
    /// in a repository.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct Run {
        /// The forge's run id, from one counter across the forge, from 1.
        pub id: u64,
        /// Its number in its repository (Forgejo's `index_in_repo`), from 1.
        pub number: u64,
        /// The commit it ran on.
        pub commit: u64,
    }

    /// A job of a run: one per check context, in `Checks.contexts`' order.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct Job {
        /// The forge's job id, from one counter across the forge, from 1.
        pub id: u64,
        pub run: u64,
        /// Its context's position in `Checks.contexts`, from 0: a link's job index.
        pub index: u32,
        /// 1, or 2 once its context ran again.
        pub attempt: u32,
    }

    /// The reads 14.2 adds to the fake's `api::Read` (`Job` changes in place: it names the job
    /// by its id, as Forgejo's log route does, and no longer its commit and run).
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub enum Read {
        /// The run of `commit` whose number is `number`: `Answer::Runs`, none or one.
        Runs { commit: u64, number: u64 },
        /// One run: `Answer::Run`, or `Missing(Job)`.
        Run { run: u64 },
        /// A run's jobs, in index order: `Answer::Jobs`.
        Jobs { run: u64 },
        /// A job attempt's log, cut to `max_bytes` and one more: `Answer::File`.
        Job { job: u64, attempt: u32, max_bytes: u32 },
        /// The item's whole thread, oldest first, unpaged as Forgejo's: `Answer::Comments`.
        Comments { number: u64 },
        /// The oldest pull request of `head` into `base`, open or not, as Forgejo's lookup
        /// names it: `Answer::Pull`, or `Missing(Pull)`.
        FirstPull { head: Box<[u8]>, base: Box<[u8]> },
        /// The open pull requests by number, the `page`th `limit` of them: `Answer::Pulls`.
        OpenPulls { page: u32, limit: u32 },
    }

    /// The answers 14.2 adds (`Comments(Box<[api::Comment]>)` and `OpenPulls`'
    /// `Pulls(Box<[api::Pull]>)` beside these).
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub enum Answer {
        Runs(Box<[Run]>),
        Run(Run),
        Jobs(Box<[Job]>),
    }

    /// The writes 14.2 adds, Forgejo's two reviewer routes; `SetReviewers` stays for the
    /// worlds that translate the client directly.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub enum Write {
        /// Adds each of `reviewers` not asked yet; answered `Done`.
        RequestReviewers { number: u64, reviewers: Box<[u64]> },
        /// Removes each of `reviewers` asked; answered `Done`.
        RemoveReviewers { number: u64, reviewers: Box<[u64]> },
    }

    // `api::Pull` gains `pub merge_base: Option<u64>`: the newest commit head and base share.
    // `api::Read::PullFor` answers the open pull of `head` into `base`, else the oldest.
    // `api::Write::OpenPull` gains `pub opens_empty: bool`: true opens a head with no commit its
    // base lacks, as Forgejo does (the protocol layer's calls); false answers `NothingToMerge`.
    // `api::CheckSummary.link` is, for a CI status, `/<full name>/actions/runs/<number>/jobs/<index>`
    // (the protocol layer writes it as it is, relative to the origin, as Forgejo v16.0.5
    // does), and empty for another user's status.
}

// temper_fake_forge_protocol (14.3, `testing/temper-fake-forge-protocol`, step code): Forgejo's
// API over the fake forge's domain, on skein's HTTP server, from `temper-forge-forgejo`'s server
// side (`crate::s06::server`), after skein's `testing/skein-fake-llm-protocol` at the pin.
pub mod temper_fake_forge_protocol {
    use crate::s06::limits as documents;
    use crate::s06::request;
    use alloc::boxed::Box;
    use skein_lib::stream::{Down, Up};
    use skein_lib::{Env, List, Map, Queue, Secret, Token};

    /// Stand-ins for `temper_fake_forge_domain`'s boundary (not a dependency of the sketch
    /// crate): `Event::Call { reply_to, user, repository, op }` and `Request::Reply { to, result }`.
    pub mod fake_domain {
        use alloc::boxed::Box;
        use skein_lib::ReplyTo;

        /// `temper_fake_forge_domain::api::Op`.
        #[derive(Clone, PartialEq, Eq, Hash, Debug)]
        pub struct Op;
        /// `temper_fake_forge_domain::api::Answer`.
        #[derive(Clone, PartialEq, Eq, Hash, Debug)]
        pub struct Answer;
        /// `temper_fake_forge_domain::api::Error`.
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct Error;

        /// `temper_fake_forge_domain::Event`.
        #[derive(PartialEq, Eq, Debug)]
        pub enum Event {
            Call { reply_to: ReplyTo, user: u64, repository: Box<[u8]>, op: Op },
        }

        /// `temper_fake_forge_domain::Request` (its `Hook` never reaches the protocol layer
        /// before webhooks are built).
        #[derive(PartialEq, Eq, Debug)]
        pub enum Request {
            Reply { to: ReplyTo, result: Result<Answer, Error> },
        }
    }

    /// The most one connection's entry point emits upward, and downward.
    pub const MAX_UP: u32 = 3;
    pub const MAX_DOWN: u32 = 18;

    /// The layer's limits.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct Limits {
        /// Domain calls held at once, over every connection.
        pub calls: u32,
        /// Connections at once.
        pub connections: u32,
        pub http: skein_http::server::Limits,
        pub documents: documents::Limits,
        /// The largest raw answer (a file, a log) the layer writes.
        pub raw_bytes: u32,
    }

    /// One repository as the scenario names it.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Repository {
        /// The id the scenario gave it (`api::Setup.id`).
        pub id: u32,
        /// `owner/name`, as the fake's calls name it.
        pub full: Box<[u8]>,
        /// The branch its protection covers, as the scenario's `Setup.protection` names it.
        pub protected: Option<Box<[u8]>>,
    }

    /// Faults moved down (forge.md, 9; domain/testing.md, 4.2), each a chance in 1,000 per request,
    /// drawn by the layer; the fake's own `unavailable` and `timeouts` are 0 under it.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct Faults {
        /// The connection closed before the request is read: the component's `Unavailable`.
        pub reset: u32,
        /// After the domain made the call: the connection dropped with no answer.
        pub drop: u32,
        /// After the domain made the call: a `500` with no body.
        pub server_error: u32,
    }

    /// The scenario's side of the protocol layer.
    #[derive(Debug)]
    pub struct Config {
        /// `http://127.0.0.1:3000`: before each `html_url` and job link.
        pub base: Box<[u8]>,
        /// API tokens and the users they stand for; any other token, or none: `401`.
        pub tokens: List<(Secret, u64)>,
        /// Users' logins by id; an id not here is written `u<id>`.
        pub users: List<(u64, Box<[u8]>)>,
        pub repositories: List<Repository>,
        /// `/settings/api`'s; also the page cap of every listing.
        pub max_response_items: u32,
        pub default_paging_num: u32,
        pub faults: Faults,
    }

    /// A route, counted in the tally: one per request kind of 06's `request::Request`.
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub enum Route {
        User,
        ApiSettings,
        RepositoryById,
        Repository,
        UserSearch,
        Issues,
        Issue,
        Comments,
        Pull,
        PullFor,
        Pulls,
        CombinedStatus,
        Reviews,
        ReviewComments,
        Branch,
        Branches,
        Protection,
        Protections,
        Collaborators,
        Permission,
        PullFiles,
        Compare,
        Raw,
        Runs,
        Run,
        Jobs,
        JobLog,
        CreateIssue,
        OpenPull,
        PostComment,
        PostReview,
        EditIssue,
        RequestReviewers,
        RemoveReviewers,
        Merge,
        Update,
        PostStatus,
        CreateBranch,
        DeleteBranch,
    }

    /// The route of a decoded request.
    #[must_use]
    pub fn route(request: &request::Request) -> Route {
        unreachable!("sketch")
    }

    /// The requests the layer read, counted as each is read (forge.md, 9: "by route, by user and
    /// in total"); a world compares `total` with the costs the component reported.
    #[derive(Debug)]
    pub struct Tally {
        pub total: u64,
        pub routes: Map<Route, u64>,
        pub users: Map<u64, u64>,
        /// Requests no route took (`404` or `405`), and those a token did not admit (`401`).
        pub unrouted: u64,
        pub unauthorized: u64,
    }

    /// Why the layer could not be made.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum Error {
        /// A limit is 0, or a table passes it.
        Limits,
    }

    /// To the world.
    #[derive(PartialEq, Eq, Debug)]
    pub enum Event {
        /// A call for the fake's domain; its reply comes back through `down`.
        Domain(fake_domain::Event),
        /// The layer closes this connection (a fault, or `Connection: close`); `closed` follows.
        Close,
        /// The connection is closed. Terminal for its server.
        Closed,
    }

    /// The routing and calls across connections, and the tally.
    #[derive(Debug)]
    pub struct Service {
        _owned: (),
    }

    impl Service {
        /// Checks `config` against `limits`; `seed` draws the faults.
        pub fn new(config: Config, limits: &Limits, seed: u64) -> Result<Service, Error> {
            unreachable!("sketch")
        }
        /// The server a domain reply is for, or the reply back when its connection is gone.
        pub fn target(
            &mut self,
            request: fake_domain::Request,
        ) -> Result<(Token, fake_domain::Request), fake_domain::Request> {
            unreachable!("sketch")
        }
        /// What it read.
        #[must_use]
        pub fn tally(&self) -> &Tally {
            unreachable!("sketch")
        }
        /// The owner's reclaim point.
        pub fn reclaim(&mut self) {
            unreachable!("sketch")
        }
    }

    /// One connection's HTTP state.
    #[derive(Debug)]
    pub struct Server {
        _owned: (),
    }

    impl Server {
        /// A connection of `owner`, the world's token for it.
        pub fn new(owner: Token, limits: &Limits) -> Result<Server, Error> {
            unreachable!("sketch")
        }
        /// Whether `resume` has work.
        #[must_use]
        pub fn has_work(&self) -> bool {
            unreachable!("sketch")
        }
    }

    /// Starts reading the first request. Reserve `MAX_UP` and `MAX_DOWN` first, at each entry point.
    pub fn start(
        server: &mut Server,
        service: &mut Service,
        env: &Env<Limits>,
        above: &mut Queue<Event>,
        below: &mut Queue<Down>,
    ) {
        unreachable!("sketch")
    }

    /// The stream below's event.
    pub fn up(
        server: &mut Server,
        service: &mut Service,
        env: &Env<Limits>,
        event: Up,
        above: &mut Queue<Event>,
        below: &mut Queue<Down>,
    ) {
        unreachable!("sketch")
    }

    /// The domain's reply to this connection's call: its answer encoded as Forgejo's.
    pub fn down(
        server: &mut Server,
        service: &mut Service,
        env: &Env<Limits>,
        reply: fake_domain::Request,
        above: &mut Queue<Event>,
        below: &mut Queue<Down>,
    ) {
        unreachable!("sketch")
    }

    /// Pending writes of the answer, as room comes.
    pub fn resume(
        server: &mut Server,
        service: &mut Service,
        env: &Env<Limits>,
        above: &mut Queue<Event>,
        below: &mut Queue<Down>,
    ) {
        unreachable!("sketch")
    }

    /// The world closes the connection; its call, if any, is forgotten when its reply comes.
    pub fn close(
        server: &mut Server,
        service: &mut Service,
        env: &Env<Limits>,
        above: &mut Queue<Event>,
        below: &mut Queue<Down>,
    ) {
        unreachable!("sketch")
    }

    /// io's `Closed` for the connection: `Event::Closed`.
    pub fn closed(
        server: &mut Server,
        service: &mut Service,
        env: &Env<Limits>,
        above: &mut Queue<Event>,
        below: &mut Queue<Down>,
    ) {
        unreachable!("sketch")
    }

    /// The layer's heap at once: its tables, each connection's HTTP state and its answer.
    #[must_use]
    pub fn worst_case(limits: &Limits, config: &Config) -> Option<u64> {
        unreachable!("sketch")
    }
}
```

The protocol world and the conformance mode, ordinary Rust, not
compiled:

```rust
// temper_engine_protocol_forge_world::pipe (tests/engine/protocol-forge/src/pipe.rs, 14.3),
// ordinary Rust, not compiled.

/// One in-memory connection: each end keeps its side's demand and room, takes the other's
/// sends, delivers them in fragments drawn from the seed, and the world cuts it at a byte.
pub struct Pipe { /* the two ends (each an intake, a demand, a granted room), the rng, the cut */ }

/// How a pipe ends early.
pub enum Cut {
    /// It never does.
    Never,
    /// After this many bytes in all, both ends hear `Failed(Fault::Reset)`.
    At(u64),
}

impl Pipe {
    pub fn new(seed: u64, cut: Cut) -> Pipe;
    /// The component's stream request.
    pub fn client_down(&mut self, down: Down);
    /// The fake's server's.
    pub fn server_down(&mut self, down: Down);
    /// What the component hears next, if anything is due.
    pub fn client_up(&mut self) -> Option<Up>;
    /// What the fake's server hears next.
    pub fn server_up(&mut self) -> Option<Up>;
    /// Bytes carried so far, both ways.
    pub fn carried(&self) -> u64;
}

// temper_engine_protocol_forge_world::protocol (tests/engine/protocol-forge/src/protocol.rs, 14.4),
// ordinary Rust, not compiled.

/// The forge connector's top, the forge component, the fake's protocol layer and domain,
/// joined by pipes; the forge connector world's verbs. A second answer for one call fails
/// the test where it comes.
pub struct Protocol { /* the top and its env; the component and its env, events and io;
                        the fake's protocol service, its servers by connection, the pipes;
                        the fake's domain and env; stored rows, seen requests, counts */ }

impl Protocol {
    /// `org/repo` (id 2) adopted by nothing yet; no cuts, no faults.
    pub fn new(seed: u64) -> Protocol;
    /// Cuts `chance` in 1,000 connections at a drawn byte, and the layer's faults.
    pub fn cutting(seed: u64, chance: u32, faults: Faults) -> Protocol;
    // The forge connector world's verbs, same names and arguments
    // (tests/engine/forge/src/world.rs:168-360):
    pub fn event(&mut self, event: top::Event);
    pub fn adopt(&mut self);
    pub fn adopt_as(&mut self, role: top::Role);
    /// The top and the component made afresh, as a new process; the fake runs on.
    pub fn restart(&mut self);
    pub fn permission(&mut self, permission: raw::Permission);
    pub fn produce(&mut self, branch: &[u8]);
    pub fn push(&mut self, branch: &[u8], content: &[u8]);
    pub fn status(&mut self, branch: &[u8], check: raw::Check);
    pub fn open_pull(&mut self, head: &[u8], base: &[u8]) -> u64;
    pub fn close(&mut self, number: u64);
    pub fn resolve(&mut self, branch: &[u8], content: &[u8]);
    pub fn preload_history(&mut self, count: u32);
    pub fn run_for(&mut self, seconds: u64);
    pub fn seen(&self) -> &[top::Request];
    pub fn take_seen(&mut self) -> Box<[top::Request]>;
    pub fn stored(&self) -> &Map<top::Key, top::Stored>;
    pub fn writes(&self) -> u32;
    pub fn calls(&self) -> u32;
    // The protocol world's own:
    /// The sum of the costs the component's answers reported.
    pub fn costs(&self) -> u64;
    /// The fake protocol layer's tally.
    pub fn tally(&self) -> &Tally;
    /// Whether the API token's bytes appear in no top request, stored row or fact seen.
    pub fn token_kept(&self) -> bool;
    /// The `/user` and `/settings/api` requests the fake served.
    pub fn start_reads(&self) -> u64;
    /// The issues the fake's domain made (its observations).
    pub fn issues_made(&self) -> u32;
}

// forgejo_conformance::component (tools/forgejo-conformance/src/component.rs, 14.5),
// ordinary Rust, not compiled.

/// The component's facts against a fresh throwaway Forgejo provisioned as 05.2's fixture.
pub fn run(forgejo: &Forgejo, scratch: &Path) -> Result<Vec<Fact>, Failure>;

/// The io layer and the component on skein-shell's ring, routed as temper-service routes them.
struct Hosted { /* skein_io::Io, the component, its queues, the requests counted */ }
impl skein_shell::Host for Hosted { /* iterate, completions, submissions, deadlines, ... */ }

// forgejo_conformance::facts (14.5 adds):
/// `text` with the lines of facts named `prefix...` replaced by `facts`, in place of the
/// first such line, or at the end; every other line kept as it was.
pub fn replace_block(text: &str, prefix: &str, facts: &[Fact]) -> String;
```

## Questions to raise when reached

- **The fake's `TooLarge`, `Circular` and `Unreachable`** have no
  Forgejo counterpart: the table above writes `413`, `422` and `503`, so
  the component reads `Full`, `Refused` and `Timeout` where the direct
  worlds' translation reads `TooLarge`, `Refused` and `Unavailable`
  (`tests/engine/forge/client/src/translate.rs:150-172`). Default: the
  table; a copied story whose outcome differs through bytes for this
  reason is raised with its name.
- **A user the scenario's table lacks:** forge.md 9 names users by the
  token table only. Default: login `u<id>`.
- **The binary's exit line for `Stop::Forge`:** credentials.md 5 says
  "the reason on stderr", not its words. Default: the two lines in
  14.1's Scope.
- **A forge component start that never ends** (a forge unreachable
  for good): credentials.md 5 waits with backoff, and the forge's calls
  are answered `Unavailable` at their deadlines; nothing stops the
  engine. Default: so, as the design says; no deadline on a start.
- **A host name that resolves to several addresses:** configuration.md
  3 says "a host's name and port". Default: the first address, as jig's
  binary takes an issuer's.
- **The conformance mode's runner:** the Actions facts need a run whose
  job fails (05.3's workflow). Default: 05.3's workflow and runner,
  pushed again in this mode's fixture; without the runner, the two facts
  are `unchecked`.

## Done when

- The session's crates and worlds pass: `cargo nextest run -p
  temper-service -p temper-fake-forge-domain -p temper-fake-forge-protocol
  -p temper-engine-protocol-forge-world -p temper-engine-forge-world -p
  temper-engine-forge-client-world` passes, and `cargo nextest
  run -p temper-engine-protocol-forge-world --profile fuzzy -E
  'binary(/^fuzzy_/)'` passes.
- The routing answers no forge call itself: `git grep -nE
  'api::Error::Unavailable|Error::Unavailable.*cost: 0' --
  crates/temper-service/src ':!*tests*'` prints nothing.
- The component's facts are recorded: `grep -cE '^(holds|fails|unchecked)
  component-' tools/forgejo-conformance/facts.txt` prints 39, and each
  `fails` line among them is named in a ruling of `decisions.md`.
- The world keeps its share: `cargo nextest run -p
  temper-engine-protocol-forge-world --profile measure -j 1 -E 'not
  binary(/^fuzzy_/)'`, then `awk -F'[][]' '/PASS \[/ {s += $2} END
  {printf "%.1f\n", s}' <its log>`, prints at most 1.0; the same with
  `--ignore-default-filter` and `binary(/^fuzzy_/)` in place of `not
  binary(/^fuzzy_/)` prints at most 8.0.
- The pins held: `grep -oE 'forgetic/(jig|smith|skein)\.git#[0-9a-f]+'
  Cargo.lock | sort -u` prints the three pinned revisions and no other line.
- The gate passed on this session's last landing: `git -C
  ~/src/rust/temper log --format='%(trailers:key=Gate-Result,valueonly)'
  --grep='^Step: temper-14\.' origin/main | head -1` names
  `scripts/check.sh passed`.
