+++
id = "public-negative"
part = "public-checks"
kind = "feature"
needs = ["forge-public", "public-http", "public-ctl"]
files = ["temper/tests/one-process/src/public.rs", "temper/tests/one-process/src/public/**", "temper/tests/one-process/tests/suite/public.rs", "temper/tests/one-process/tests/suite/public/**"]
effort = "high"
minutes = 210
+++

# The negative walk: what a visitor can try, through temper's service and a caching proxy

## Goal

Every bullet of `temper/docs/design/protocol/sign-in.md`, 2.5 is a test in the one-process
world (`temper/tests/one-process`, `temper/docs/design/domain/testing.md`, 2.5): temper's real
service, opened to public reading, behind a caching proxy that is a program of the world and
follows sign-in.md 2.4; each answer checked against what 2.4, jig's
`jig/docs/design/protocol/client.md`, 17.5 and section 19, and jig's
`jig/docs/design/domain/people.md`, section 13 allow. A test that would pass with one of
`notes/common.md`'s "rules that are the point" broken is not done.

Acceptance:

- **The proxy** (`src/public/proxy.rs`), a world program on the machine as the world's other
  programs are (`machine.rs`, `host_programs`), speaking HTTP/1.1 with skein-http's parsers:

  ```rust
  pub(crate) struct Proxy { .. }
  pub(crate) struct ProxyConfig { pub upstreams: Vec<Upstream>, pub per_address: PerAddress,
                                  pub public_in_flight: u32, pub public_pool: u32,
                                  pub other_pool: u32 }
  pub(crate) struct Upstream { pub names: Vec<Box<[u8]>>, pub address: SocketAddr }
  pub(crate) struct PerAddress { pub reads_per_second: u32, pub reads_in_flight: u32,
                                 pub anonymous_per_second: u32 }
  pub(crate) struct ProxyCounts { pub upstream_public: u32, pub upstream_other: u32,
                                  pub served_from_cache: u32, pub refused_host: u32,
                                  pub busy: u32 }
  ```
  It follows 2.4 exactly: the `Host` checked against each upstream's names before the cache
  is looked in (a refused `Host` is `421`, never a copy); the key the upstream, the validated
  `Host` and the path exactly as received (never decoded or normalized); only `GET` under
  `/jig/v1/public/` answered `200` or `404` with `public, max-age=N, must-revalidate` is kept,
  never a `503`, a `no-store` or anything else; a copy fresh while its age (the `Age` it came
  with plus the time kept) is below its `max-age`, served with `Age` set to that age, and
  nothing served past it: no stale copy on an upstream's failure or while refreshing, no
  background refresh; cookies, `Authorization`, `Forwarded` and `X-Forwarded-*` dropped from
  a public read before it goes on; per client address, public reads per second and in
  flight, and public reads in flight in all, answered `503` by the proxy itself before it
  takes an upstream connection; public reads on a pool of upstream connections apart from the
  other routes'; requests with no credential to the other routes limited per address; no
  `Access-Control-` header added. `ProxyCounts` let the tests see what reached the upstream.
- **The visitor** (`src/public/visitor.rs`): a world program that sends exact request bytes
  (method, raw path, headers) to the proxy or to the service and keeps each answer's status,
  headers in order and body, so a test compares two answers byte for byte.
- **The deployment** (`src/public.rs`, `configure(value)` as `m15::configure` is, active only
  while a public test runs): public reading on at
  `temper/docs/design/draft/first-release.status.md`'s proposed figures, scaled to the world
  (`clients.public.reads` 4, `clients_http.public_calls` 4, `rate` 8, `rate_burst` 8,
  `bytes`/`bytes_burst` 512 KiB, `answer_bytes` 128 KiB, `max_age` 2, `core.public.*` as
  small as the stories need); its world's project 1 made public by its owner with
  `temper ctl project amend` (the world's `ctl.rs` `Command`), a public goal with a change
  that lands (the world's merge story, its delegate tracked), a conversation's delegate, a
  change outside every goal's tree, a second project kept private, and a third made then
  deleted.
- **The tests,** `tests/suite/public.rs`, each named for its bullet of 2.5, seeded, the seed in
  every failure, grouped so that one world run serves several (each world start is costly):
  - *Every owner, version and kind:* `every_owner_version_and_kind_answers_as_declared`
    (jig's owner 0 and the forge's owner, each served version `200`, a version not served
    `200` with `no-store`, a kind no owner declared `404`, an owner none declared `404`);
    `every_inaccessible_identifier_is_one_unknown_byte_for_byte` (another project's task, a
    private project, one being deleted, one never made, a conversation's delegate, a change
    outside every goal's tree: status, headers and body equal, read at one instant of the
    world's clock, through the proxy and directly).
  - *Malformed paths:* `malformed_paths_are_404_no_store_and_never_kept` (a number past its
    type, a leading zero, 15, 17 and upper-case hex digits, `%2F` and `%61`, an empty and a
    `.` segment, a query, a trailing slash; each `404` with `no-store`, and a second send
    reaches the upstream again by `ProxyCounts`).
  - *No credential anywhere else:* `no_credential_anywhere_else_is_signed_out` (every keyed
    request, read, watch, stream and sign-out, with no credential and with a cookie of
    another deployment); `a_cookie_or_bearer_on_a_public_read_changes_no_byte` (a signed-in
    person's cookie, a bearer, both: answers equal to the bare read's; the upstream received
    neither header).
  - *No widening:* `a_members_set_visibility_is_refused` (exit 2 from ctl, the project still
    private); `each_broken_relation_refuses_the_start` (configuration.md 6's relations, one at
    a time, each start refused at its path).
  - *A store written before:* `a_store_written_before_public_reading_reads_every_project_private`
    (projects made with the figures at zero, the service restarted with them on: every
    project's read unknown, *projects* empty).
  - *Revocation:* `a_private_or_deleted_project_is_unknown_at_once_and_through_the_proxy_within_the_max_age`
    (a project made private, another deleted, while reads of each are under way and cached:
    the service answers unknown at the next read, the proxy no later than the `max-age` after
    the last answer's decision, and never a copy after);
    `a_stopped_upstream_serves_copies_until_their_freshness_ends`.
  - *The proxy's key:* `two_deployments_behind_one_proxy_keep_their_own_copies` (the same
    path through two upstreams, each answered its own bytes);
    `a_refused_host_is_never_answered_from_the_cache`.
  - *Load:* `public_reads_past_the_limits_are_busy_while_signed_in_work_is_answered` (past
    the proxy's per-address and in-all limits and the engine's room, rate and bytes: `503`
    with `Retry-After`, while the world's signed-in client's reads and a request at the same
    moment are answered within the world's usual bound); `abandoned_reads_leave_the_budgets_whole`
    (reads whose connections close midway; after a quiet second, `rate_burst` reads in a row
    are all answered).
  - *No CORS:* `no_answer_carries_an_access_control_header` (a document's, a `404`'s, a
    `503`'s, a refusal's, a guard's and an `OPTIONS` preflight's, through the proxy and
    directly).
  - The proxy's own rules: `the_proxy_keeps_age_and_serves_nothing_stale`,
    `the_proxy_keeps_only_public_200_and_404`.

## Inputs

- `temper/docs/plan/public-reading-build/notes/common.md` ("The rules that are the point").
- `temper/docs/design/protocol/sign-in.md`, 2.4 and 2.5; `temper/docs/design/domain/testing.md`,
  2.5; jig's `jig/docs/design/protocol/client.md`, 17.5 and section 19;
  `jig/docs/design/domain/people.md`, 13.2, 13.5-13.10; `jig/docs/design/protocol/configuration.md`,
  section 6.
- `tasks/public-ctl.md`, `tasks/forge-public.md`, and the merged `public-http` (the route's
  answers and headers).
- As they stand: `temper/tests/one-process`'s `src/lib.rs`, `world.rs` (`run`,
  `host_programs`, `service`, `refused_restore`), `machine.rs`, `fixtures.rs`
  (`world_document`, where each module's `configure` is called), `ctl.rs` (`Command`,
  `Caller`), `m15.rs` and `m15/` (a module of the same kind: its `configure`, its run and its
  suite file), `lower.rs`; `tests/suite/main.rs`, `tests/suite/m15.rs`.

## Owns and may change

- Owns only new files: those in the header. The world is playground-temper's too (its
  trunk edits `temper/tests/one-process/**`): add exactly one module line in `src/lib.rs`
  (`pub mod public;`) and one in `tests/suite/main.rs` (`mod public;`), and, pre-approved,
  one line in `src/fixtures.rs` `world_document`, `crate::public::configure(&mut value);`
  beside `crate::m15::configure`. No other existing file of the world changes; if one must
  (a `pub(crate)` the module needs, a dependency in `Cargo.toml`), it is a question, unless
  a wrapper in your own files reaches the same.
- Nothing outside the world. A fault the walk finds in another task's code is reported with
  the failing test, its seed and log, never repaired here.
- Pre-approved:
  - If a second temper service does not fit the world, the second deployment behind the
    proxy is a scripted upstream program of your own answering the same path with other
    bytes, and say so.
  - If the world's stories offer no tracked delegate, make the public goal through the
    world's chat story with its scripted delegate given `"tracked"` in your configuration,
    as `lower::configure` edits the policy; if that is refused, use the nearest public goal
    the world can make and report what the forge's kinds were read on.
  - If deleting a project has no path in the world but the native client's request, send
    jig's `RemoveProject` as the world's stories send requests.
  - If a test's world run is over the default profile's budget, group it with another's run
    first; report the measurements, never move a test to escape the budget.

## Steps

1. The visitor and the proxy, with the proxy's own tests
   (`heavy cargo nextest run -p temper-one-process-world --test suite -E 'test(/public::the_proxy/)'`).
2. The deployment and the tests bullet by bullet, each run alone first
   (`-E 'test(=public::NAME)'`).
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-one-process-world --all-targets -- -D warnings`; the world's focused tests
   (`heavy cargo nextest run -p temper-one-process-world`), checking the profile's budget;
   the citations check.
4. Commit (why: what a visitor can try against an opened deployment, each tried); push your
   branch.

## Report

As the implementer's text says, and also: each test with the bullet of 2.5 it covers and
its world-run time, the proxy's rules each test would catch broken, and every place the
service or the proxy answered otherwise than 2.4 and 17.5 say.

## Answers

2026-11-03. **The user's decision on public titles.** User, 2026-11-03 15:40Z: public reading shows titles. A goal's and a task's title (the first line of its spec, as jig's client contract names a task's title) crosses into its public line, and a public project's name crosses into its public record. There is no author flag and no human publication capability for these. Everything else in people.md 13.3's "No text at all" list still never crosses: spec beyond its first line, instructions, inputs, results, words, questions and answers, transcripts, notes, reasons, labels' meanings, proposals, escalations, people's and parties' names, charters', procedures' and roles' names. Review finding 1 is settled by the user's decision.

For this task: Titles and the project's name are expected in public bytes now; every other text source keeps its marker and must never appear.
