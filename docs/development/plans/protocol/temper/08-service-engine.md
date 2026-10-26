# Session 08: `temper-service`'s engine

`crates/temper-service`, step code: the engine's service of protocol.md,
section 6, copied from jig's reference service (`testing/jig-test-service`
at temper's jig pin) with temper's parts where the reference leaves its
own. The engine root (`temper-engine-domain`) with jig's components
around it: the store with temper's families, the LLM accounts, the
workers' channel with temper's parts and the identities' table, host
tools through `temper-engine-smith`, and the clients' component with the
forge family's records; every forge call answered unavailable at cost 0
until 14.1 wires the forge component. 08.1 lays the crate's typed
settings, limits and token prefixes; 08.2a to 08.2d the engine's loop and
store; 08.3 the workers' side; 08.4 host tools; 08.5a and 08.5b the
clients' side. 09 adds the worker's layer and the one process; 10.1 the
startup that builds the settings from configuration's bytes.

## Read first

- `docs/design/protocol.md`, sections 4 to 10; `configuration.md`,
  sections 2, 3, 7 and 8; `channel.md`, sections 3 to 8;
  `credentials.md`, sections 3 to 5; `llm.md`, sections 4 and 5;
  `client.md`, section 8; `store/README.md`, sections 3 and 5.
- jig's `docs/design/protocol/service.md` at the pin, sections 2 to 5;
  jig's `protocol/store.md`, 7.1 and 7.2.
- `decisions.md`, 5, 8, 9, 15 and 21; `code-organization.md`, sections 1
  to 8 (section 7: the token prefixes).
- `findings/jig-service.md`, whole; `findings/jig-components.md`, whole;
  `findings/temper-engine.md`, section 1, 2(c), 2(d) and Findings 6, 10, 11.
- jig's reference at the pin (`~/.cargo/git/checkouts/jig-aba39083bf77fefe/d91a89c/testing/jig-test-service/src/`):
  `lib.rs` (`Service`, `iterate` :393, `route_events` :492), `engine.rs`
  (`Engine::new` :40, `iterate` :279, `release` :373, `fire` :410,
  `wall_deadline` :267), `route.rs`, `stop.rs`, `store.rs` (`limits` :39),
  `accounts.rs`, `values.rs`, `workers.rs`, `clients.rs`,
  `clients/translation.rs`, `family.rs` (`to_root` :127), `tools.rs`,
  `limits.rs`, `startup.rs:304-411`, `boundary.rs`; its tests
  `src/tests/{fixtures.rs, engine.rs, clients.rs, first_period.rs,
  stop.rs}`; and `testing/jig-test-store/src/route.rs` (the store's
  application routing).
- temper's code: `crates/temper-engine-domain/src/{boundary.rs, store.rs,
  decision.rs, domain.rs:60-365, admission.rs, route/host_route.rs,
  translate.rs}`; `tests/engine/domain/src/protocol.rs` (the world's
  stand-in for this routing); `tests/system/src/composed/link.rs` (the
  translation this session's replaces, 18.3 moves the world onto it);
  `crates/temper-engine-smith/src/{lib.rs, start.rs, calls.rs, answers.rs,
  finish.rs}`.

## What the design asks

- **jig's shape** (protocol.md, 6): "Starting, ending, `iterate`'s passes,
  room and timers are jig's (jig's `protocol/service.md`, sections 3 to
  5)": io's events by token, one root event per decision, "A store's
  answer is always taken", released outputs to components, deadlines
  reported and fired, a failed commit stops the engine, a shutdown drains.
- **The forge, until the component** (protocol.md, 12, step 2): "Until
  step 6 the routing answers every forge call unavailable, since it
  provably never left." (decision 15: at cost 0; 14.1 removes the arm.)
- **temper's parts** (protocol.md, 6; channel.md, 7): "the routing
  encodes the root's `ForgeWorkspace` as temper's workspace items ...
  making each repository's remote from its forge's `git` base and the
  repository's owner and name"; "it decodes the work and the preparation
  into the root's `Parts`"; "A part that does not decode, on the engine:
  the root gets the answer with no parts (`Parts::none()`) ...; the
  routing counts it."
- **temper's families** (protocol.md, 6; store/README.md, 5): "The
  routing maps `Range::Forge` to one interval ... encodes the root's
  cursor with temper's key encoder, decodes each row with temper's codecs
  before the root sees it, and keeps the longest prefix of the page
  within the root's rows and decoded bytes, its cursor the last key kept."
- **The identities' table** (credentials.md, 4): "encoded once at start
  ... held as one secret by number and generation"; "The engine's routing
  gives the root `Event::Credential { number, generation: 1, valid }` for
  each identity as its first events, before its restart";
  "`jig-protocol-engine-workers` fills the value in from the identities'
  table, through the routing".
- **`PAYLOAD_VERSION`** (channel.md, 3): "Both channel components are
  given it when the services make them".
- **Host tools and results** (llm.md, 4 and 5): a call decoded with
  `smith_channel::CallName` "whose activation must be the attempt" and
  `temper_engine_smith::call`; "A call that does not decode is answered
  at once as an error that names what is wrong ... the routing renders the
  error as smith's text and hands it to the root as
  `WorkerInput::RenderedHostCall { to, answer }`"; answers rendered with
  `temper_engine_smith::answer`; "A result that does not decode fails its
  attempt as invalid"; the charter "resumes when its policy allows and the
  task has kept turns".
- **The clients' side** (client.md, 8): jig's clients' component; "The
  versions it serves are jig's wire's and, for the family `FORGE`,
  `CLIENT_VERSION` alone"; "An application request of the family `FORGE`
  at `CLIENT_VERSION` is decoded ...; one that does not decode, or of
  another family or version, is answered undecodable"; "The routing fills
  the adoption's host from that forge's configured `git` base: its host,
  with `:port` when it names one ... It leaves the host empty for a forge
  number not configured, and hands the root the party's keyed request";
  "Every read and watch of the forge's family is unknown."
- **Periods** (configuration.md, 8): "before the core admits a party's
  request, the engine's routing gives the root one period for each
  configured project, its budget the project's spend per period, with no
  next start"; "a later start gives no period".
- **Limits** (configuration.md, 7): "One profile, `small` ... Its numbers
  are constants in `temper-service`'s limits module, each documented with
  what it bounds"; "Overrides by dotted name: the path of the field in its
  owner's `Limits` type, a duration's last segment with `_ms`".

## Choices this plan makes

- **Module by module from jig's reference:**

  | temper-service | From the reference | What changes |
  |---|---|---|
  | `tokens.rs` | the prefixes in `workers.rs:11-12`, `accounts.rs:17`, `clients.rs:13`, `store.rs:384` | named constants, temper's 5 and 6 added (code-organization.md, 7) |
  | `limits.rs` | `limits.rs` | temper's profile and overrides; the root's limits from jig's and temper's |
  | `settings.rs` | `boundary.rs` (`Files`, `Seeds`) | temper's typed members (decision 9) |
  | `store/route.rs` | `jig-test-store/src/route.rs` | temper's ranges and families |
  | `store.rs` | `store.rs` | sizes from `temper_protocol_store::application` |
  | `accounts.rs`, `values.rs` | `accounts.rs`, `values.rs` | temper's root events; the identities' table |
  | `lib.rs`, `engine.rs`, `route.rs`, `stop.rs` | the same | temper's root's `Request`, `Delivery` and inputs; credentials; the forge unavailable |
  | `workers.rs`, `translate.rs`, `assignment.rs` | `workers.rs` | temper's parts, `PAYLOAD_VERSION`, the assignment from temper's root |
  | `tools.rs` | `tools.rs` | `temper-engine-smith` in place of jig's charter and tools crates |
  | `clients.rs`, `clients/translation.rs` | the same | temper's party vocabulary |
  | `family.rs`, `family/refusals.rs` | `family.rs`; jig-protocol-engine-clients' `src/translate/refusals.rs` | the forge family |

- **Pre-split by code-organization.md, section 8:** the reference's
  engine layer is about 4,300 lines (`lib.rs`, `engine.rs`, `route.rs`,
  `stop.rs`, `store.rs`, `accounts.rs`, `values.rs`, `workers.rs`,
  `tools.rs`, `clients.rs` and its translation, `family.rs`;
  findings/jig-service.md, section 1); 08.2 is 08.2a to 08.2d and 08.5 is
  08.5a and 08.5b, each under about 600 lines. "08.2's engine layer"
  (09's Needs) means 08.2d landed; "08.2's store routing" (18.1) means
  08.2a's `store::route` and 08.2b's `store::limits`.
- **Typed settings** (decision 9): `Service::engine(settings, seeds,
  accounts)` builds the engine's service; 10.1's `startup` builds the
  settings. The tests' fixtures parse jig's part with
  `jig_protocol_config::document` and `engine` from a JSON constant
  (jig's reference fixtures' `ENGINE`, `src/tests/fixtures.rs:8-78`,
  with `connectors: [1]` and one project), and write temper's members
  as Rust values: one forge (number 1, `git` `https://forge.example`,
  user 42, identity 1) and one identity (number 1, token
  `forge-token-1234`, login `temper`, author `temper <temper@forge.example>`).
- **No `service.components` member:** temper's engine always makes the
  accounts', workers' and clients' components (findings/jig-service.md,
  Discrepancy 4).
- **The store's routing maps the root's ranges** (08.2a; store/README.md,
  5, "The core's ranges are jig's, filtered" and "The census is jig's
  index walk"): `Range::Forge` to `Restore::Application([interval()])`;
  `CoreRange::{Deployment, People}` to
  `Restore::Jig([RestoreRange::Deployment])`, `[People]`;
  `CoreRange::Tasks` to `Restore::Jig([RestoreRange::Core,
  RestoreRange::Tasks])` (its projections are core rows) and
  `CoreRange::{Calls, RunProofs}` to `Restore::Jig([RestoreRange::Core])`,
  each keeping the rows `Range::contains` admits; `Range::Census(census)`
  to `Restore::Jig([Census(census)])`, its rows in jig's walk order,
  none dropped; `CoreRange::{TaskResult,
  ProposalDecision, EscalationDecision}` to `Get::Jig` of their one key;
  `CoreRange::TaskTranscript { task, from }` to
  `TranscriptRead::Window { task, attempt, turn, after }`;
  `CoreRange::Turns { task, attempt }` to `TranscriptRead::Page { task,
  backwards: false, after }`, keeping the attempt's turns;
  `CoreRange::Notes(range)` to `Load::Notes(range)`; `StoreRequest::Read`
  to `Request::Read`. A root cursor (`after: Some(key)`) becomes jig's
  `Cursor { interval, after }` with the tagged key: jig's key encoder
  for a core key, `jig_protocol_store::keys::application` of temper's for
  a forge key (findings/jig-components.md, Discrepancy 4); `interval` is
  the place, in the request's list of jig's intervals, of the one that
  holds the key (for `[Core, Tasks]`, 0 for a `Key::Core`, 1 for a
  `Key::Tasks`; 0 for every other one-interval range), and for the
  census `temper_engine_domain::census_interval(census, key)`. A page
  keeps the longest prefix within the root's `most` rows and `bytes`
  decoded bytes, its `next` the last key kept; a filtered page that
  keeps no row while jig's page has a cursor is asked again from that
  cursor, with the same bounds, before the root hears of it. A census
  page's `next` is jig's cursor's `after` decoded by
  `jig_protocol_store::keys::decode`: the last index row's key, which
  the root checks (08.2a's root part).
- **The root's deadlines.** temper's root reports none
  (`temper_engine_domain` has `fire` but no `next_deadline`): 08.2d adds
  `Domain::next_deadline(&self, env) -> Option<Time>` in a new
  `crates/temper-engine-domain/src/deadline.rs`, the earliest of its
  children's own `next_deadline`s and its wall deadlines mapped onto the
  monotonic clock as the reference's `wall_deadline` (`engine.rs:267`)
  maps them; the service fires the root once it is due.
- **Credentials first:** each identity's `Event::Credential { number,
  generation: 1, valid: Duration::from_nanos(u64::MAX) }` is stepped
  before `Event::Restart` (credentials.md, 4), where the reference steps
  its restart first (findings/jig-service.md, Discrepancy 5).
- **The forge's calls until 14.1:** `Request::Forge { call, .. }` becomes
  `Event::Forge(Box::new(forge::Event::Client(client::Event::Answered {
  call, cost: 0, result: Err(api::Error::Unavailable) })))` in the same
  pass; a `Request::Busy` forge event is offered again on the next pass,
  and dropped when the drain closes the engine.
- **An output for a component not made yet** (the workers' before 08.3,
  the clients' before 08.5a) stops the engine as the reference's
  `stop::unavailable` does, naming the component; it is routed, never
  `unreachable!`.
- **The charter moves into `temper-engine-smith`** (Finding 11): 08.3
  adds `temper_engine_smith::charter(&Assignment) -> smith_domain_run::Charter`
  with the resume rule `tests/system/src/composed/link.rs:154-160` keeps;
  the routing encodes it with `smith_protocol_channel::encode_charter`
  against the engine's endpoints. code-organization.md, section 4 names
  08.3 its owner; it lands with the assignment that needs it.
- **No saved-work branch** (decision 21): the routing sends neither
  jig's `save` nor temper's `Items.save`.
- **The core's refusal of an adoption** (08.5b) is encoded with jig's
  client wire after jig-protocol-engine-clients' `translate/refusals.rs`
  (230 lines, `pub(crate)` at the pin), copied into
  `family/refusals.rs`.
- **Proof layer:** unit, the service's own step tests in
  `crates/temper-service/src/tests/`, driving `iterate` over skein-kv's
  memory store as the reference's fixtures do; the world is 17's
  one-process world and the process tier 10.4's.

## Rules that bind it

- "No increment of this plan ... adds an `unreachable!` arm for a variant
  the service can reach: an arm the service reaches is routed"; "links
  `jig-protocol-charter` or `jig-protocol-tools` into `temper-service`"
  (code-organization.md, section 6). Every root output is matched without
  a wildcard.
- "No increment ... puts a secret's value in a domain type, a fact, a
  store record, a log or a `Debug` output but skein-lib's `Secret`'s
  length" (code-organization.md, section 6). The identities' and the API
  tokens' values move between components through the routing, never
  through the root.
- "**Keeping the pins.** Adding to any `Cargo.toml` a crate of jig, smith
  or skein that `Cargo.lock` does not hold yet ... The increment that adds
  one then runs, in this order," the three `cargo update --precise`
  commands, and the check prints the three pinned revisions
  (code-organization.md, section 3). At the base the lock lacks
  `jig-protocol-config`, `jig-protocol-accounts`,
  `jig-protocol-engine-workers`, `jig-protocol-worker-wire`,
  `jig-protocol-engine-clients` and `skein-oauth-accounts`.

## Increments

### 08.1 The crate, its tokens, its limits and its settings

Scope: `crates/temper-service` (step code; `#![cfg_attr(not(test),
no_std)]`, `#![forbid(unsafe_code)]`, the workspace's lints) with
`tokens`, `limits` and `settings` (code-organization.md, 7;
configuration.md, 3, 4 and 7; decision 9).
Needs: 04.5's store codecs (the session's start).
Files: `crates/temper-service/{Cargo.toml, src/{lib.rs, tokens.rs,
limits.rs, limits/names.rs, settings.rs, tests.rs, tests/fixtures.rs,
tests/limits.rs}}`; the workspace `Cargo.toml` (member, entries
`temper-service`, `jig-protocol-config`); `Cargo.lock` (the pins' three
commands). No existing type changes Shape.
Shape: the interface sketch's `tokens`, `limits` and `settings`.
`settings::Agent` (the worker's agent profile, as jig's reference's
`Agent`, `lib.rs:36-41`) and the final `settings::Worker { jig, root,
git }` are defined here, for 09; the checkout's limits are
`limits::Limits.checkout`, and 09.2 adds `limits::Limits.git_component`.
`profile(Small)`'s constants are the values `tests/engine/domain`'s and
`tests/engine/forge`'s fixtures run with (`tests/engine/forge/src/world.rs:8-72`
for the forge's), each with a doc line on what it bounds, `name_bytes`
256 and the client's `answer_bytes` 65,536; a quantity jig's small
profile also sets cites it. Where those values fail a domain's worst case
beside jig's small children, the constant is raised to the value the
worst case needs, and its doc line names that check (ruling 113). `apply` matches the dotted name byte for
byte in `limits/names.rs`, one arm per field of `Root`, `forge::Limits`
(with `client.`, `change_policy.`, `issue_policy.`) and
`checkout::Limits`, durations by `_ms`. `limits::engine` builds
`temper_engine_domain::Limits` field by field: `journal`, `loads` and
`brief` from `Root`; `tasks`, `authority`, `people`, `fleet`, `accounts`,
`notes` and `call_records` from `jig.core`; `forge` from `limits.forge`.
Test shape: `src/tests/limits.rs`; fixtures `src/tests/fixtures.rs`
(made here, extended by 08.2a to 08.5b). Tests:
`no_two_component_prefixes_meet`,
`an_override_sets_the_field_its_dotted_name_names` (rows:
`forge.repositories`, `forge.client.calls`, `checkout.remote_timeout_ms`,
`root.loads.rows`), `an_unknown_name_or_a_value_past_its_type_is_refused`,
`the_small_profile_passes_each_domains_worst_case` (`forge::worst_case`,
`checkout::worst_case`, `temper_engine_domain::worst_case` are `Some`),
`the_root_limits_take_jigs_children_and_temper_own`. Ceilings: each
under 20 ms. Reference test, in full:

```rust
#[test]
fn an_override_sets_the_field_its_dotted_name_names() {
    let mut limits = limits::profile(limits::Profile::Small);
    let rows: [(&[u8], u64); 4] = [
        (b"forge.repositories", 3),
        (b"forge.client.calls", 5),
        (b"checkout.remote_timeout_ms", 7_000),
        (b"root.loads.rows", 9),
    ];
    for (name, value) in rows {
        assert_eq!(limits::apply(&mut limits, name, value), Ok(()), "{name:?}");
    }
    assert_eq!(limits.forge.repositories, 3);
    assert_eq!(limits.forge.client.calls, 5);
    assert_eq!(limits.checkout.remote_timeout, skein_lib::Duration::from_millis(7_000));
    assert_eq!(limits.root.loads.rows, 9);
}
```

Proof: unit, `cargo nextest run -p temper-service`, run once: the
prefixes are disjoint, each override sets its field, an unknown name is
refused, and the profile passes every domain's worst case; under 1 s.
Don't: parse configuration (10.1's); define a component's limits the
crate does not link yet (`forge_component.` with 14.1, `git_component.`
with 09.2).
Size: at most 500 lines.
Removes: nothing.

### 08.2a The store's routing

Scope: `store::route` (public, for 18.1's world driver): the root's
`StoreRequest`s into jig's store component's `Request`s, and its
`Event`s into root events, with temper's families encoded and decoded by
`temper-protocol-store` (store/README.md, 5; the choices' range table).
The root's part, for the census (store/README.md, 5, "The census is
jig's index walk"): `temper_engine_domain::census_interval(census:
&jig_domain_core::Census, key: &jig_domain_core::Key) -> Option<u32>`
in `store.rs` (0 for a `ConversationOldest` key; 1 and the party's place
in `census.results` for a `Result` key `census.admits`; none for any
other key), and `Range::Census`'s doc saying the walk's order, the
repeats and the index-key cursor; `loads.rs` for `Range::Census` only:
`begin` takes an `after` that `census_interval` places (not
`Range::contains`), and `check` keeps its row check (each row
`Range::contains`, an ended task), drops the order check among rows,
and takes a `next` that `census_interval` places, its `(interval, key)`
above `after`'s, with or without rows; every other range's checks stay
as they are. The engine domain world's fake census
(`tests/engine/domain/src/store.rs:101-134`) walks as jig's does: the
`ConversationOldest` rows by key, then each party's `Result` rows past
its `after`, parties in the census's order, from the cursor's interval
and key; each joined to its `Ended` row, at most `most` index rows, its
`next` the last index row's key while rows remain.
Needs: nothing beyond 08.1.
Files: `crates/temper-service/{Cargo.toml, src/{lib.rs, store.rs,
store/route.rs, store/ranges.rs, tests/store_route.rs,
tests/fixtures.rs}}` (`store.rs` declares the module only); the workspace
`Cargo.toml` (entries `temper-protocol-store`, `jig-protocol-store`,
`skein-kv`); the root's part: `crates/temper-engine-domain/src/{store.rs,
loads.rs, lib.rs}` (`lib.rs`'s `pub use store::{..}` adds `census_interval`),
`tests/engine/domain/src/store.rs` and `tests/engine/domain/tests/loads.rs`.
Shape: the sketch's `store_route` (`temper_service::store::route`). A
commit's `Write::Save(Record::Core(row))` is `Write::Save(row)`,
`Write::Erase(Key::Core(key))` `Write::Erase(key)`,
`Write::Save(Record::Forge { row })` `Write::Put { key:
keys::encode(&stored_key(&row), limit), value: records::encode(&row) }`,
`Write::Erase(Key::Forge(key))` `Write::Remove { key }`. A restored
application row goes through `records::row`; one that does not decode,
a `Corrupt` and an `OpenRefused` end the store's use as the reference's
`stop` does (`Uncommitted { number }` of the last commit). Core rows come
up as `Record::Core`; transcripts' turns as `Record::Core` turn rows.
Test shape: `src/tests/store_route.rs`. Tests:
`each_root_range_asks_the_store_as_the_table_maps_it` (one row per
`CoreRange`, `Census` and `Forge`; `Tasks` as `[Core, Tasks]`),
`a_task_page_keeps_projections_then_live_tasks_and_drops_calls`,
`a_filtered_page_that_keeps_nothing_asks_again_from_jigs_cursor`,
`a_census_cursor_is_its_index_rows_key_and_interval` (a root `after` of
a party's `Result` key asks jig from `Cursor { interval: 1 + its place,
after }`; jig's cursor comes back to the root as that key, rows in jig's
order with a repeated owner),
`a_forge_commit_puts_its_encoded_key_and_row`,
`a_forge_page_reaches_the_root_decoded_with_its_last_key`,
`a_continued_forge_load_asks_from_its_tagged_cursor`,
`a_page_keeps_the_longest_prefix_within_the_roots_bounds`,
`a_row_that_does_not_decode_stops_the_engine`. The root's part, in
`tests/engine/domain/tests/loads.rs`:
`a_census_page_takes_repeated_owners_and_an_index_cursor` (rows of the
`Ended` tasks 5, 5 and 3, repeated and descending, and a `Result` key as
`next`: `Loaded`, the three rows kept) and
`a_census_cursor_behind_its_start_is_out_of_order` (a `next` of an
earlier interval than `after`'s: `Failure::Order`); the engine domain
world's suite passes with the fake's census walk. Ceilings: each under
20 ms. Reference test, in full:

```rust
#[test]
fn a_continued_forge_load_asks_from_its_tagged_cursor() {
    let mut routes = fixtures::routes();
    let last = forge::Key::Landed([9; 32]);
    let load = engine::StoreRequest::Load {
        owner: fixtures::OWNER,
        range: engine::Range::Forge,
        after: Some(engine::Key::Forge(last.clone())),
        most: 4,
        bytes: 4096,
    };
    let Some(jig_protocol_store::Request::Restore { range, after: Some(cursor), rows: 4, .. }) = route::down(&mut routes, load)
    else {
        panic!("a forge load restores from a cursor");
    };
    let jig_protocol_store::Restore::Application(intervals) = range else { panic!("the application's interval") };
    assert_eq!(intervals.as_ref(), [temper_protocol_store::interval()]);
    let relative = temper_protocol_store::keys::encode(&last, u32::MAX).expect("a fixed key fits");
    let tagged = jig_protocol_store::keys::application(&relative, u32::MAX).expect("tagged");
    assert_eq!(cursor, jig_protocol_store::Cursor { interval: 0, after: tagged });
}
```
Proof: unit, `cargo nextest run -p temper-service -E
'test(/store_route/)'`, run once: each root range asks the store as the
table maps it, a forge commit puts its encoded row, a page reaches the
root decoded within its bounds, a filtered page goes on, the census's
cursor is its index row's, and a bad row stops the engine; and `cargo nextest run -p temper-engine-domain -p temper-engine-domain-world`,
run once: the root takes a census page as jig walks it, and the engine
domain world's stories pass; under 1 s and the world's budget.
Don't: decode a row without checking its key against its record;
restore a core range as an application interval; order or dedupe the
census's rows, in the routing or the root (jig's core keeps its own
marks); change a load check of any range but `Range::Census`.
Size: at most 570 lines: 450 for the routing, 120 for the root's part
and the fake's census.
Removes: nothing.

### 08.2b The store's driver

Scope: `store::Store`, jig's store component and its three file drivers
as the reference's `store.rs`: made on the deployment's directory (or,
in tests, over skein-kv's memory stores), adopted root, commits and
loads through 08.2a's routing, its sizes checked with `sizes::check`
and copied into skein-kv's limits (findings/jig-components.md,
Discrepancy 3), its close.
Needs: nothing beyond 08.2a.
Files: `crates/temper-service/src/{store.rs, tests/store.rs,
tests/fixtures.rs}`.
Shape: `pub fn store::limits(root: &temper_engine_domain::Limits, core:
&jig_domain_core::Limits, forge: &forge::Limits, profile:
&jig_protocol_store::Limits) -> Option<jig_protocol_store::Limits>`
(public: 18.1's world driver sizes jig's component with it) as
the reference's `store.rs:39-69`, with
`temper_protocol_store::application(forge)?` and
`temper_engine_domain::journal_limits(&root.journal)`; `Store::new`,
`adopt`, `ask`, `down`, `room`, `next_deadline`, `work_pending`,
`closed`, `close`, `complete`, `iterate`, as the reference's, its kernel
operations under `tokens::STORE`.
Test shape: `src/tests/store.rs`. Tests:
`the_store_opens_over_memory_and_answers_a_commit`,
`sizes_past_skein_kvs_limits_refuse_the_store`,
`a_forge_row_committed_is_restored_by_the_forge_interval`,
`the_store_closes_after_its_drivers`. Ceilings: each under 20 ms.
Reference test, in full:

```rust
#[test]
fn a_forge_row_committed_is_restored_by_the_forge_interval() {
    let mut store = fixtures::memory_store();
    let row = fixtures::landed_row();
    fixtures::commit(&mut store, 1, [engine::Write::Save(engine::Record::Forge { row: Box::new(row.clone()) })]);
    let rows = fixtures::load(&mut store, engine::Range::Forge);
    assert_eq!(rows, [engine::Record::Forge { row: Box::new(row) }]);
}
```

Proof: unit, `cargo nextest run -p temper-service -E
'test(/store/)'`, run once: a forge row committed over the memory store
comes back from the forge's interval, and sizes past skein-kv's limits
refuse the store; under 1 s.
Don't: compute the sizes but with `sizes::check`; open files outside the
deployment's directory.
Size: at most 450 lines.
Removes: nothing.

### 08.2c The accounts

Scope: `accounts.rs` and the accounts' part of `values.rs`: skein's
accounts driver on jig's `jig-protocol-accounts` translation, its
private keeper's file operations, its tokens lent into jig's accounts'
table, the root's `Request::Account` in and `Event::Account` out, as the
reference's `accounts.rs` and `values.rs` (credentials.md, 3).
Needs: nothing beyond 08.2b.
Files: `crates/temper-service/{Cargo.toml, src/{accounts.rs, values.rs,
tests/accounts.rs, tests/fixtures.rs}}`; the workspace `Cargo.toml`
(entries `jig-protocol-accounts`, `skein-oauth-accounts`, `skein-oauth`);
`Cargo.lock` (the pins' three commands).
Shape: the sketch's `service::AccountsRuntime`; `Accounts::{new,
request, up, complete, iterate, abort, is_empty, next_deadline,
work_pending}` as the reference's, taking temper's root's
`accounts::Request` and pushing `temper_engine_domain::Event::Account`;
its io under `tokens::ACCOUNTS`.
Test shape: `src/tests/accounts.rs`. Tests:
`a_handed_in_account_is_granted_from_its_file`,
`a_granted_token_is_lent_and_kept_out_of_the_root`,
`an_account_whose_file_is_missing_is_refused_by_number`. Ceilings: each
under 20 ms. Reference test, in full, after the reference's
`src/tests/engine.rs:176-199`:

```rust
#[test]
fn a_granted_token_is_lent_and_kept_out_of_the_root() {
    let mut accounts = fixtures::accounts_with(fixtures::ACCOUNT);
    let events = fixtures::want(&mut accounts, 1);
    // The root's event names the account and its generation; its type holds no token.
    let [engine::Event::Account(accounts::Event::Component(accounts::FromComponent::Granted {
        account: 1,
        generation: 1,
        ..
    }))] = events.as_ref()
    else {
        panic!("one grant of account 1 at generation 1");
    };
    assert_eq!(fixtures::lent(&accounts, 1, 1).expose(), b"test-token");
}
```
Proof: unit, `cargo nextest run -p temper-service -E
'test(/accounts/)'`, run once: a handed-in account is granted, its token
lent from the table and absent from the root's events; under 1 s.
Don't: refresh an account, keep its record in the store, or put a token
in a root event.
Size: at most 550 lines.
Removes: nothing.

### 08.2d The engine's loop

Scope: `Service` (the engine's part; 09 adds the worker's), `iterate`
and io's routing by token (`lib.rs`), `engine.rs` (one root decision
per input, store answers always taken, room, the restart, the periods,
the credentials), `route.rs` (every root output, exhaustively), `stop.rs`
(the drain, the abort, a failed commit), the root's
`Domain::next_deadline`, and the forge's calls answered unavailable
(protocol.md, sections 6 and 12, step 2; jig's service.md, 3 to 5).
Needs: nothing beyond 08.2c.
Files: `crates/temper-service/src/{lib.rs, engine.rs, route.rs,
stop.rs, values.rs, tests/engine.rs, tests/stop.rs, tests/first_period.rs,
tests/fixtures.rs}`; `crates/temper-engine-domain/src/{lib.rs,
deadline.rs}` (`Domain::next_deadline`; `domain.rs` declares the module,
at most 3 lines).
Shape: the sketch's `service` module. `Engine` keeps the root, its
`Env`, the store, the accounts, the identities (an empty table until
08.3), its queues and the first periods (one per configured project, as
the reference's `startup.rs:405-408`, cleared when the core's deployment
row is restored, as `store.rs:468`). Root outputs, by `route::request`:
`Request::Store` to 08.2a/b; `Request::Account` to the accounts;
`Request::Forge` answered as the choices say; `Request::ToChild(released)`
back as `Event::Released` on the next pass; `Request::Busy` held and
offered again; `Request::Drained` to `stop::drained`; `Request::Stop` to
`stop::invalid_restore`; `Request::Worker` and `Request::Party` to
`stop::unavailable` naming `workers` and `clients` until 08.3 and 08.5a
route them.
Test shape: `src/tests/engine.rs`, `src/tests/stop.rs`,
`src/tests/first_period.rs`, after the reference's. Tests: in
`engine.rs`, `the_restart_asks_one_load_at_a_time`,
`a_store_answer_is_taken_with_the_journal_full`,
`a_failed_commit_releases_nothing_after_it`,
`a_forge_call_is_answered_unavailable_at_no_cost`; in `first_period.rs`,
`a_first_start_gives_each_project_one_period`; in `stop.rs`,
`a_signal_drains_and_a_second_aborts`. Ceilings: each under 20 ms; under
0.5 s together. Reference tests, in full:

```rust
// src/tests/stop.rs, after the reference's src/tests/stop.rs
#[test]
fn a_signal_drains_and_a_second_aborts() {
    let mut service = fixtures::ready();
    service.shutdown();
    assert_eq!(service.end(), None, "the first signal drains: nothing has closed yet");
    service.shutdown();
    assert_eq!(service.end(), Some(&End::Aborted));
}
```

```rust
// src/tests/first_period.rs, after the reference's src/tests/first_period.rs
#[test]
fn a_first_start_gives_each_project_one_period() {
    let mut service = fixtures::started_with_projects(&[(1, 100), (2, 73)]);
    fixtures::restart(&mut service);
    let periods = fixtures::committed_periods(&mut service);
    assert_eq!(periods, [(2, 1, 73), (1, 1, 100)]);
}
```

(`committed_periods` reads `(project, period, budget)` from the funders'
period rows the memory store committed, in commit order.)


```rust
#[test]
fn a_forge_call_is_answered_unavailable_at_no_cost() {
    let mut service = fixtures::started();
    let call = skein_lib::Token::new(7);
    let request = engine::Request::Forge {
        call,
        repository: client::api::Repository { forge: 1, repository: 2 },
        op: client::api::Op::Read(client::api::Read::Settings),
    };
    fixtures::route(&mut service, request);
    let answered = fixtures::next_forge_event(&mut service);
    assert_eq!(
        answered,
        forge::Event::Client(client::Event::Answered { call, cost: 0, result: Err(client::api::Error::Unavailable) })
    );
}
```

Proof: unit, `cargo nextest run -p temper-service`, run once: the
engine restarts over the memory store one load at a time, gives each
configured project one period at its first start, answers a forge call
unavailable at cost 0, stops on a failed commit, and drains on a signal;
under 2 s.
Don't: retry a forge call; step a party's request before the periods;
fire the root before its `next_deadline`.
Size: at most 600 lines (the root's `next_deadline` included).
Removes: nothing (14.1 removes the forge's unavailable arm).

### 08.3 The workers' side

Scope: `workers.rs` (jig-protocol-engine-workers made with
`PAYLOAD_VERSION`, its listeners and connections under
`tokens::WORKER_LISTENERS` and `WORKER_CONNECTIONS`); `values.rs`'s
identities' table (each identity encoded once with
`temper_protocol_worker_wire::identity`, held by number at generation 1)
and grants' values (jig's accounts' table for `Accounts`, the identities'
for `Application`); `Event::Credential` per identity before the restart;
`assignment.rs` (the root's `Assignment` as the hub's `LinkDown::Assign`:
messages, grants, the identity, the workspace with its items' token, the
answered calls, the charter); `translate.rs`'s `items`, `parts`,
`identity`; parts decoded beside an answer into `Parts`, a part that
does not decode counted and `Parts::none()` (channel.md, sections 3 to
8; credentials.md, 4); `temper_engine_smith::charter`.
Needs: 01.2's `Parts.preparation`; 03.1's records and `PAYLOAD_VERSION`.
Files: `crates/temper-service/{Cargo.toml, src/{workers.rs, values.rs,
assignment.rs, translate.rs, engine.rs, route.rs, tests/workers.rs,
tests/translate.rs, tests/fixtures.rs}}`;
`crates/temper-engine-smith/src/{lib.rs, start.rs, tests.rs}` (the
charter, its test); the workspace `Cargo.toml` (entries
`jig-protocol-engine-workers`, `jig-protocol-worker-wire`,
`temper-protocol-worker-wire`, `smith-protocol-channel`,
`smith-charter`); `Cargo.lock` (the pins' three commands).
`tests/system/src/composed/link.rs` keeps its own translation until 18.3.
Shape: the sketch's `translate` and `engine_smith`. `items` returns
`Ok(None)` for an empty workspace; each repository's remote is
`<git>/<owner>/<name>.git` from the forge whose `number` is
`provider.forge`; `Access::writable { push, expected: none }` for
`push: Some`, `read_only` otherwise; `save` none. `parts` maps `Left` to
`pushed` (`forge::Pushed { tag, commit }`), `Saved` to `saved` (the tags
whose landing is `landed`, and those with `from_saved` whose landing
left the branch as it was: every landing but `landed`, as
`composed/link.rs:93-105`'s `saved`), `Preparation` to `preparation`
(01.2's shape). Parts come with an answer as jig's `Part { token, bytes
}` named by the answer's `Work.left`, `Work.saved` and `Preparation::Permanent
{ resource }`; the routing decodes each with the engine's limits
(`temper_protocol_worker_wire::limits` of the forge's and the checkout's
numbers) before `translate::parts`. The assignment's grants' values are
`values.grants`, in grant order; an assignment `items` cannot encode is
answered `Failed(Invalid)`, as the reference's `workers.rs:110-122`.
After each root step, the engine drains the root's facts with
`Domain::pop_fact` (01.2's bounded queue: "08 drains"), counts each in
`Engine.facts` and drops it, until skein's trace streams exist.
Test shape: `src/tests/translate.rs`, `src/tests/workers.rs`. Tests: in
`translate.rs`, `a_workspace_reaches_items_with_each_remote_made_from_its_base`,
`the_heads_a_run_landed_reach_the_roots_pushed_heads`,
`a_save_names_each_repository_with_saved_work_after_it` (the two cases
`composed/link.rs`'s tests hold),
`each_preparation_reaches_the_roots_parts`; in `workers.rs`,
`a_part_that_does_not_decode_gives_no_parts_and_is_counted`,
`each_repositorys_grant_carries_its_identitys_value`,
`identities_are_announced_before_the_restart`,
`the_roots_facts_are_drained_and_counted_after_each_step` (one row: a
failed preparation's fact). In `crates/temper-engine-smith`:
`a_charter_resumes_when_its_policy_does_and_turns_are_kept`. Ceilings:
each under 20 ms. Reference tests, in full:

```rust
// src/tests/workers.rs
#[test]
fn each_repositorys_grant_carries_its_identitys_value() {
    let values = fixtures::values_with_identity(1, b"forge-token-1234");
    let grants = [fixtures::application_grant(1, 1)];
    let filled = values.grants(&grants, skein_lib::Time::ZERO);
    let [Some(value)] = filled.as_ref() else { panic!("one value for one grant") };
    let identity = fixtures::decoded_identity(&value.value);
    assert_eq!(identity.token().expose(), b"forge-token-1234");
    assert_eq!(value.account_id.expose(), b"");
}
```

```rust
#[test]
fn a_workspace_reaches_items_with_each_remote_made_from_its_base() {
    let forges = [fixtures::forge(1, b"https://forge.example")];
    let assignment = fixtures::assignment_with(fixtures::repository(7, b"acme", b"shop"));
    let items = translate::items(&assignment, &forges, &fixtures::wire_limits())
        .expect("within the engine's limits")
        .expect("a run with a repository has items");
    let repository = fixtures::first_repository(items);
    assert_eq!(repository.tag, 7);
    assert_eq!(repository.remote.as_ref(), b"https://forge.example/acme/shop.git");
}
```

(`first_repository` takes the generated record apart with `into_parts`.)
Proof: unit, `cargo nextest run -p temper-service -p
temper-engine-smith`, run once: an assignment leaves with its items,
remotes made from the forge's base, its charter encoded and its grant
filled with its identity's value; an answer's parts reach the root's
`Parts`, a bad part gives none and is counted; identities are announced
before the restart; under 1 s.
Don't: keep a part beyond the step that decodes it; send `save`;
interpret the remote on the engine beyond making it.
Size: at most 600 lines.
Removes: nothing.

### 08.4 Host tools and results

Scope: `tools.rs`: a relayed call decoded (`smith_channel::CallName`,
its activation the attempt; `temper_engine_smith::call`) into
`WorkerInput::DecodedHostCall`; one that does not decode answered at
once as `WorkerInput::RenderedHostCall` with smith's error text naming
its `Problem`, which 01.1's root relays with no commit; a decided call's
answer rendered with `temper_engine_smith::answer`; a delivery's settled
answer (`HostDelivery::Answer`) to the workers' component as the hub's
relayed answer; a result read with `decode_declared` and
`temper_engine_smith::result`, one that does not decode `tasks::End::Failed(Class::Invalid)`
(llm.md, 4 and 5).
Needs: 01.1's relay of an undecodable call.
Files: `crates/temper-service/src/{tools.rs, route.rs, tests/tools.rs,
tests/fixtures.rs}`; the workspace `Cargo.toml` (entry `smith-channel`
if not yet there).
Shape: `tools::decode(to, task, attempt, call: &fleet::Call) ->
engine::Event`, `tools::render(to, answer: &CallAnswer) -> engine::Event`,
`tools::result(payload, outcome: &[u8]) -> engine::Event`, each total,
after `composed/link.rs:109-150`; `HostRequest::Dropped` forgets
nothing (the routing keeps no state).
Test shape: `src/tests/tools.rs`. Tests:
`a_call_is_decoded_into_the_roots_tool`,
`a_call_of_another_activation_is_answered_an_error`,
`an_unknown_tool_is_answered_an_error_with_no_commit`,
`a_decided_answer_is_rendered_as_smiths_text`,
`a_result_that_does_not_decode_fails_its_attempt_invalid`. Ceilings:
each under 20 ms. Reference test, in full, after the reference's
`src/tests/engine.rs:200-231`:

```rust
#[test]
fn an_unknown_tool_is_answered_an_error_with_no_commit() {
    let mut service = fixtures::started_with_a_run();
    let commits = fixtures::commits(&mut service);
    fixtures::relay(&mut service, fixtures::call(b"no_such_tool", b"{}"));
    let answer = fixtures::relayed_answer(&mut service);
    let host::Relayed::Answer { error: true, text } = answer else { panic!("an error answer") };
    assert!(fixtures::contains(&text, b"unknown tool"));
    assert_eq!(fixtures::commits(&mut service), commits);
}
```

(`host::Relayed::Answer` is jig-domain-host's `link.rs:58` at the pin.)
Proof: unit, `cargo nextest run -p temper-service -E
'test(/tools/)'`, run once: a call decodes into the root's tool, an
unknown tool is answered an error with no commit, an answer is rendered,
and a bad result fails its attempt invalid; under 1 s.
Don't: link `jig-protocol-tools` or `jig-protocol-charter`; keep a call
beyond its step.
Size: at most 300 lines.
Removes: nothing.

### 08.5a The clients' component

Scope: `clients.rs` and `clients/translation.rs`: jig-protocol-engine-clients
on the configured loopback listener under `tokens::CLIENTS`, serving
jig's wire; the printed link into the service's `links`; sign-ins'
secrets through the store's secrets; jig's core events into temper's
root's `PartyInput` and its party outputs into the component's events,
exhaustive both ways; every application request answered `Undecodable`,
every read unknown, every watch refused unknown (client.md, 8).
Needs: nothing beyond 08.4.
Files: `crates/temper-service/{Cargo.toml, src/{clients.rs,
clients/translation.rs, engine.rs, route.rs, lib.rs, tests/clients.rs,
tests/fixtures.rs}}`; `crates/temper-engine-domain/src/{boundary.rs,
route.rs}` (`PartyInput::SignOut` and its routing land in 11.3, ruling
116; 08.5a translates jig's event to it); the
workspace `Cargo.toml` (entries `jig-protocol-engine-clients`,
`jig-protocol-client-wire`); `Cargo.lock` (the pins' three commands).
Shape: `Clients` as the reference's `clients.rs`; `translation::to_root(
core::Event) -> Result<engine::PartyInput, kit::Event>` (`SignIn` to
`SignedIn`, `People(Ask { .. })` to `Ask`, `Read`, `Watch`, `WatchRun`,
`Unwatch`, `Delivered`, `StreamDelivered` to their namesakes) and
`translation::to_component` of `PartyRequest` and the party
`Delivery`s (`WebReply`, `Read`, `Document`, `WatchEnded`, `Result`,
the refusals and run text) as the reference's `clients/translation.rs`.
Test shape: `src/tests/clients.rs`, after the reference's. Tests:
`the_engine_prints_one_sign_in_link_at_start`,
`a_link_sign_in_reaches_the_root_and_answers_a_token`,
`a_chat_ask_reaches_the_root_as_a_party_ask`,
`a_request_before_decisions_open_is_answered_not_ready`. Ceilings: each
under 20 ms. Reference test, in full:

```rust
#[test]
fn a_chat_ask_reaches_the_root_as_a_party_ask() {
    let ask = core::Event::People(people::Event::Ask {
        reply_to: fixtures::REPLY,
        sign_in: 3,
        key: [5; 16],
        ask: fixtures::chat_ask(),
    });
    let Ok(engine::PartyInput::Ask { sign_in: 3, key, .. }) = clients::translation::to_root(ask) else {
        panic!("a people ask is the root's party ask");
    };
    assert_eq!(key, [5; 16]);
}
```
Proof: unit, `cargo nextest run -p temper-service -E
'test(/clients/)'`, run once: the engine prints one link, a link's
sign-in reaches the root and answers a bearer token, and a chat's ask reaches the
root; under 1 s.
Don't: add a sign-in provider; decode the forge family's records (08.5b).
Size: at most 600 lines.
Removes: nothing.

### 08.5b The forge family

Scope: `family.rs`: `Served { family: Some(FORGE), first: CLIENT_VERSION,
last: CLIENT_VERSION }` beside jig's; `family::to_root` (the sketch);
the adoption's host filled from its forge's `git` base, empty for a
number not configured; the root's application request (02.2) made from
it; the outcome (02.3's) encoded as `AdoptAnswer` and answered as
`FromApplication::Answered { to, family: FORGE, record }`; a core
refusal through `family/refusals.rs` (client.md, 8). A
`Delivery::Application { to, outcome: None }` (02.2: the outcome's
retention ended) is answered with the family's record `AdoptAnswer {
outcome: unknown }` (03.2), as jig's reference application answers a
keyed request past its retention with its family's `unknown` outcome
(`testing/jig-test-protocol-client-wire/schema/test-client-v1.schema`,
`ApplicationOutcome`): jig's clients' component has no unknown answer of
its own (`FromApplication` is `Answered`, `Read`, `Delivery` or
`Undecodable`).
Needs: 02.3's adoption and outcome; 03.2's records.
Files: `crates/temper-service/{Cargo.toml, src/{family.rs,
family/refusals.rs, clients.rs, route.rs, tests/family.rs,
tests/fixtures.rs}}`; the workspace `Cargo.toml` (entry
`temper-protocol-client-wire`).
Shape: the sketch's `family`. `host(git)` is the URL's host, with
`:port` when `git` names one. The forge reasons map as client.md, 8
says: `Missing` not found, `Forbidden` no access, `Refused` refused,
`Busy` in flight, `TooLarge` and `Full` too large, `Unavailable`,
`Timeout` and `RateLimited` unreachable, any other an invalid answer,
and 02.3's no such forge.
Test shape: `src/tests/family.rs`. Tests:
`an_adopt_reaches_the_root_with_its_forges_host`,
`an_adopt_naming_no_configured_forge_reaches_the_root_with_no_host`,
`an_adopt_of_another_version_is_answered_undecodable`,
`a_forge_family_read_is_unknown_and_its_watch_refused`,
`each_outcome_is_encoded_as_the_forge_familys_answer` (rows: adopted,
core refused, each forge reason, and no outcome encoded as `unknown`),
`a_git_base_with_a_port_keeps_it_in_the_host`.
Ceilings: each under 20 ms. Reference test, in full:

```rust
#[test]
fn an_adopt_reaches_the_root_with_its_forges_host() {
    let forges = [fixtures::forge(1, b"https://forge.example:3000/git")];
    let exchange = fixtures::adopt_exchange(fixtures::adopt(1, b"acme", b"shop"));
    let event = family::to_root(exchange, &forges, &fixtures::client_limits()).expect("decodes");
    let adoption = fixtures::application_adoption(event);
    assert_eq!(adoption.host.as_ref(), b"forge.example:3000");
    assert_eq!((adoption.owner.as_ref(), adoption.name.as_ref()), (b"acme".as_slice(), b"shop".as_slice()));
}
```

(`application_adoption` takes 02.2's `PartyInput::Application` apart;
landed code wins on its shape.)
Proof: unit, `cargo nextest run -p temper-service -E
'test(/family/)'`, run once: an adoption reaches the root with its
forge's host, one of another version is answered undecodable, the
family's reads are unknown and its watches refused, and every outcome
is encoded as the family's answer; under 1 s.
Don't: decode a read or a watch of the family; read the adoption's
fields beyond filling its host.
Size: at most 550 lines (`family/refusals.rs` copied, 230).
Removes: the clients' `Undecodable` answer to every application request
(08.5a), for the family `FORGE`.

## Interface sketches

`scratch/plan-sketches/src/s08.rs`, compiled under the workspace's lints;
the same text. `crate::s03` and `crate::s04` are 03's and 04's sketches.

```rust
//! Sketches of session 08: `temper-service`'s engine (protocol.md, sections
//! 5 and 6), step code copied from jig's reference service
//! (`testing/jig-test-service` at temper's jig pin) with temper's parts
//! where the reference leaves its own.
//!
//! Who owns what: the service owns io, its kernel's queues, the engine's
//! root and its protocol layer (the store, the accounts, the workers' and
//! the clients' components, the identities' table); 09 adds the worker's
//! layer and the one process. What ends what: a failed commit or a refused
//! start ends the service (`End::Stopped`); a first signal drains it
//! (`End::Drained`), a second aborts it (`End::Aborted`).

/// `crates/temper-service/src/tokens.rs` (08.1): each component's tokens carry a
/// prefix in their top four bits (code-organization.md, section 7).
pub mod tokens {
    use skein_lib::Token;

    /// The termination signals' source.
    pub const SIGNALS: u64 = 1;
    /// The workers' listeners: jig's reference's.
    pub const WORKER_LISTENERS: u64 = 1_u64 << 60;
    /// The workers' accepted connections: jig's reference's.
    pub const WORKER_CONNECTIONS: u64 = 2_u64 << 60;
    /// The accounts' driver; each worker's layer shares the prefix (`worker`).
    pub const ACCOUNTS: u64 = 3_u64 << 60;
    /// The clients' component.
    pub const CLIENTS: u64 = 4_u64 << 60;
    /// The forge component (14.1 wires it).
    pub const FORGE: u64 = 5_u64 << 60;
    /// Each worker's git component (`git`, 09.2 wires it).
    pub const GIT: u64 = 6_u64 << 60;
    /// The store's three file drivers' kernel operations: jig's reference's.
    pub const STORE: [u64; 3] = [8_u64 << 60, 9_u64 << 60, 10_u64 << 60];

    /// Worker `index`'s tokens: `(3 << 60) + ((index + 1) << 48)`.
    #[must_use]
    pub fn worker(_index: u32) -> Option<u64> {
        unreachable!("sketch")
    }

    /// Worker `index`'s git component's tokens: `(6 << 60) + ((index + 1) << 48)`.
    #[must_use]
    pub fn git(_index: u32) -> Option<u64> {
        unreachable!("sketch")
    }

    /// A token's prefix: its top four bits, shifted down.
    #[must_use]
    pub fn prefix(_token: Token) -> u8 {
        unreachable!("sketch")
    }
}

/// `crates/temper-service/src/limits.rs` (08.1): temper's profile `small` and its
/// overrides by dotted name (configuration.md, section 7).
pub mod limits {
    use temper_engine_domain as engine;
    use temper_engine_domain_forge as forge;
    use temper_worker_domain_checkout as checkout;

    /// temper's one profile; another name refuses the start.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Profile {
        Small,
    }

    /// The engine root's own limits beyond jig's `engine.limits`: `root.`.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Root {
        pub journal: engine::JournalLimits,
        pub loads: engine::loads::Limits,
        pub brief: engine::BriefLimits,
    }

    /// Every limit of temper's own, by prefix: `root.`, `forge.`,
    /// `checkout.`; `forge_component.` joins with 14.1, `git_component.`
    /// with 09.2, each in the increment that wires the component.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Limits {
        pub root: Root,
        pub forge: forge::Limits,
        pub checkout: checkout::Limits,
    }

    /// Why an override was refused, its name kept by the caller's path.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Refused {
        /// No limit of temper's has this dotted name.
        Unknown,
        /// The value is outside the field's type.
        Range,
    }

    /// The profile's constants, each documented with what it bounds.
    #[must_use]
    pub fn profile(_profile: Profile) -> Limits {
        unreachable!("sketch")
    }

    /// Sets the limit named `name` (its field path in its owner's `Limits`, a
    /// duration's last segment with `_ms`) to `value`.
    pub fn apply(_limits: &mut Limits, _name: &[u8], _value: u64) -> Result<(), Refused> {
        unreachable!("sketch")
    }

    /// The engine root's limits: jig's children's from jig's profile
    /// (`jig`'s `core`), temper's own from `limits`; none if they disagree.
    #[must_use]
    pub fn engine(_jig: &jig_protocol_config::Limits, _limits: &Limits) -> Option<engine::Limits> {
        unreachable!("sketch")
    }
}

/// `crates/temper-service/src/settings.rs` (08.1): the typed values startup (10.1)
/// builds from configuration's bytes and the files it names (decision 9).
/// Secrets' values are `Secret`s; nothing here is printed but their lengths.
pub mod settings {
    use alloc::boxed::Box;
    use skein_lib::{List, Secret};

    /// A forge temper works on (configuration.md, section 3, `forges`).
    #[derive(Debug)]
    pub struct Forge {
        pub number: u16,
        pub kind: Kind,
        /// Where the forge component connects, never an in-process name.
        pub api: jig_protocol_config::Address,
        pub transport: Transport,
        /// The base of each remote and the host of the forge's resource paths.
        pub git: Box<[u8]>,
        /// The API token, read from its private file: the forge component's alone.
        pub api_token: Secret,
        /// The forge user the API token belongs to: the forge writer.
        pub user: u64,
        /// The number of this forge's git identity.
        pub identity: u32,
    }

    /// The documents a forge speaks.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Kind {
        Forgejo,
    }

    /// A forge component's connections.
    #[derive(Debug)]
    pub enum Transport {
        /// Only to a socket address on loopback.
        Plaintext,
        /// The trust file's bytes, or none for the machine's roots.
        Tls { server_name: Box<[u8]>, trust: Option<Box<[u8]>> },
    }

    /// A git identity (configuration.md, section 3, `identities`).
    #[derive(Debug)]
    pub struct Identity {
        pub number: u32,
        pub token: Secret,
        pub login: Box<[u8]>,
        pub author: Author,
    }

    /// The author and committer of the commits made with an identity.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Author {
        pub name: Box<[u8]>,
        pub email: Box<[u8]>,
    }

    /// The private files jig's parts name, by path, as jig's reference takes them.
    #[derive(Debug)]
    pub struct Files {
        pub entries: List<(Box<[u8]>, Secret)>,
    }

    /// The seeds the shell draws at start, as jig's reference takes them.
    #[derive(Debug)]
    pub struct Seeds {
        pub deployment: [u8; 16],
        pub seed: u64,
        pub crypto: Secret,
    }

    /// The engine's settings: jig's part, typed by jig-protocol-config, and
    /// temper's members (configuration.md, sections 2 and 3).
    #[derive(Debug)]
    pub struct Engine {
        pub jig: jig_protocol_config::Engine,
        pub files: Files,
        pub forges: Box<[Forge]>,
        pub identities: Box<[Identity]>,
        pub landing: temper_engine_domain::LandingPolicy,
        pub charter: u32,
        pub limits: super::limits::Limits,
    }

    /// The worker's agent profile, which the binary reads from smith's
    /// agent configuration, as jig's reference service's `Agent`
    /// (`lib.rs:36-41`); 09 builds the worker's host half from it.
    #[derive(Debug)]
    pub struct Agent {
        pub endpoints: smith_protocol_channel::Endpoints,
        pub limits: smith_host_protocol::Limits,
        pub bounds: jig_domain_host::Bounds,
        pub host_limits: smith_host_domain::Limits,
    }

    /// A worker's settings: jig's part and temper's (configuration.md,
    /// section 4); 09 builds the worker from it.
    #[derive(Debug)]
    pub struct Worker {
        pub jig: jig_protocol_config::Worker,
        /// The worker's root directory: workspaces, trash and home under it.
        pub root: Box<[u8]>,
        /// The absolute path of the git program.
        pub git: Box<[u8]>,
    }
}

/// `crates/temper-service/src/store/route.rs` (08.2a): the root's store requests
/// into jig's store component's, and its answers back, with temper's
/// families encoded by temper-protocol-store. Public: 18.1's world driver
/// routes through it. Keeps only each load's continuation.
pub mod store_route {
    use crate::s04::temper_protocol_store as families;
    use alloc::boxed::Box;
    use skein_lib::Queue;
    use temper_engine_domain as engine;

    /// The bounded table of loads in flight, made with the root's limits.
    #[derive(Debug)]
    pub struct Routes {
        _owned: (),
    }

    impl Routes {
        /// A table of `table` loads, decoding within the root's limits and
        /// temper's families' bounds.
        #[must_use]
        pub fn new(_table: u32, _limits: &engine::Limits, _families: families::Bounds) -> Routes {
            unreachable!("sketch")
        }
        /// Whether every routed load has had its terminal.
        #[must_use]
        pub fn idle(&self) -> bool {
            unreachable!("sketch")
        }
    }

    /// What one of the component's events becomes.
    #[derive(Debug)]
    pub enum Up {
        /// One root event: a commit's terminal or a load's page.
        Root(Box<engine::Event>),
        /// The next page of a continued load, asked at once.
        Ask(jig_protocol_store::Request),
    }

    /// One root store request as jig's component takes it; none for one the
    /// table has no room for.
    #[must_use]
    pub fn down(_routes: &mut Routes, _request: engine::StoreRequest) -> Option<jig_protocol_store::Request> {
        unreachable!("sketch")
    }

    /// One component event as root events or the next ask; a row that does
    /// not decode ends the load and stops the engine (`Uncommitted` of the
    /// last commit, as jig's reference does).
    pub fn up(_routes: &mut Routes, _event: jig_protocol_store::Event, _into: &mut Queue<Up>) {
        unreachable!("sketch")
    }
}

/// `crates/temper-service/src/lib.rs` (08.2d): the service and its `iterate`.
pub mod service {
    use alloc::boxed::Box;
    use skein_io::kernel::{Complete, Fd, Submit};
    use skein_lib::{List, Queue, Time, Wall};

    /// How the service ended.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum End {
        /// A first signal drained it.
        Drained,
        /// A second signal turned the drain into an abort.
        Aborted,
        /// The engine stopped; 14.1 adds a forge's refused start check.
        Stopped(Stop),
    }

    /// Why the engine stopped.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum Stop {
        /// A commit failed: nothing after it is released.
        CommitFailed { number: u64 },
        /// A listener failed or closed while the engine ran.
        Listener,
    }

    /// What the binary hands the accounts' driver, as jig's reference does.
    #[derive(Debug)]
    pub struct AccountsRuntime {
        pub root: Fd,
        pub effective_user: u32,
        pub issuers: List<(u32, skein_oauth_accounts::Endpoint)>,
        pub trust_roots: u32,
        pub trust_root_bytes: u32,
    }

    /// The engine's service (09 adds the worker's layer).
    #[derive(Debug)]
    pub struct Service {
        _owned: (),
    }

    impl Service {
        /// The engine's service from typed settings, every component made
        /// and checked; the problems name their paths.
        pub fn engine(
            _settings: super::settings::Engine,
            _seeds: &super::settings::Seeds,
            _accounts: Option<AccountsRuntime>,
        ) -> Result<Service, jig_protocol_config::Problems> {
            unreachable!("sketch")
        }
        /// Adopt the deployment directory, opened by the binary.
        pub fn adopt_store(&mut self, _root: Fd, _effective_user: u32) -> Result<(), Fd> {
            unreachable!("sketch")
        }
        /// Adopt the termination signals' descriptor.
        pub fn adopt_signals(&mut self, _descriptor: Fd) -> Result<(), Fd> {
            unreachable!("sketch")
        }
        /// The kernel's completions, for the shell to fill.
        pub fn completions(&mut self) -> &mut Queue<Complete> {
            unreachable!("sketch")
        }
        /// The kernel's submissions, for the shell to take.
        pub fn submissions(&mut self) -> &mut Queue<Submit> {
            unreachable!("sketch")
        }
        /// The sign-in links the engine printed, for stdout, one per line.
        pub fn links(&mut self) -> &mut Queue<Box<[u8]>> {
            unreachable!("sketch")
        }
        /// How it ended, once it has.
        #[must_use]
        pub fn end(&self) -> Option<&End> {
            unreachable!("sketch")
        }
        /// The earliest deadline of io, the root and every component.
        #[must_use]
        pub fn next_deadline(&self) -> Option<Time> {
            unreachable!("sketch")
        }
        /// The earliest deadline but io's own.
        #[must_use]
        pub fn next_policy_deadline(&self) -> Option<Time> {
            unreachable!("sketch")
        }
        /// Whether `iterate` has work at `now`.
        #[must_use]
        pub fn work_pending(&self, _now: Time) -> bool {
            unreachable!("sketch")
        }
        /// Whether everything it owns has closed.
        #[must_use]
        pub fn is_empty(&self) -> bool {
            unreachable!("sketch")
        }
        /// A first call drains, a second aborts.
        pub fn shutdown(&mut self) {
            unreachable!("sketch")
        }
    }

    /// One pass: io's completions to components, components to the root,
    /// one root decision, released outputs to components, io's submissions.
    pub fn iterate(_service: &mut Service, _now: Time, _wall: Wall) {
        unreachable!("sketch")
    }
}

/// `crates/temper-service/src/translate.rs` (engine side, 08.3): small
/// total functions between temper's records and the engine root's types
/// (channel.md, section 7), keeping no state. 18.3's composed world calls
/// them without the bytes between.
pub mod translate {
    use crate::s03::temper_protocol_worker_wire as wire;
    use skein_lib::Secret;
    use temper_engine_domain as engine;

    /// The run's workspace items, each remote made from its forge's `git`
    /// base, owner and name; none for a run with no repository.
    pub fn items(
        _assignment: &engine::Assignment,
        _forges: &[super::settings::Forge],
        _limits: &wire::v1::Limits,
    ) -> Result<Option<wire::v1::Items>, wire::v1::Problem> {
        unreachable!("sketch")
    }

    /// The root's parts from what an answer carried, each if present.
    #[must_use]
    pub fn parts(
        _left: Option<wire::v1::Left>,
        _saved: Option<wire::v1::Saved>,
        _preparation: Option<wire::v1::Preparation>,
    ) -> engine::Parts {
        unreachable!("sketch")
    }

    /// One identity's value for the identities' table.
    pub fn identity(
        _identity: &super::settings::Identity,
        _limits: &wire::v1::Limits,
    ) -> Result<Secret, wire::v1::Problem> {
        unreachable!("sketch")
    }
}

/// `crates/temper-engine-smith/src/start.rs`, added by 08.3: the charter of
/// an assignment, its conversation resumed when the run's policy resumes
/// and the task kept turns (the rule `tests/system/src/composed/link.rs`
/// keeps today).
pub mod engine_smith {
    use temper_engine_domain as engine;

    /// smith's charter for `assignment`.
    #[must_use]
    pub fn charter(_assignment: &engine::Assignment) -> smith_domain_run::Charter {
        unreachable!("sketch")
    }
}

/// `crates/temper-service/src/family.rs` (08.5b): the forge family on jig's
/// client wire, decoded and encoded in the routing (client.md, section 8).
pub mod family {
    use crate::s03::temper_protocol_client_wire as wire;
    use alloc::boxed::Box;
    use jig_protocol_engine_clients as kit;
    use temper_engine_domain as engine;

    /// One application exchange of jig's clients' component, as jig's
    /// reference's `family::to_root` routes it: a request of the family
    /// `FORGE` at `CLIENT_VERSION` that decodes becomes the root's keyed
    /// application request (02.2), the adoption's host filled from its
    /// forge's `git` base (empty for a forge number not configured); any
    /// other request is answered `Undecodable`, every read `Read { record:
    /// None }` (unknown), every watch refused `Unknown`, without the root.
    pub fn to_root(
        _exchange: kit::ToApplication,
        _forges: &[super::settings::Forge],
        _limits: &wire::v1::Limits,
    ) -> Result<engine::Event, Box<kit::Event>> {
        unreachable!("sketch")
    }

    /// The host segment of a forge's resource paths: `git`'s host, with
    /// `:port` when it names one (configuration.md, section 3).
    #[must_use]
    pub fn host(_git: &[u8]) -> Box<[u8]> {
        unreachable!("sketch")
    }
}
```

## Questions to raise when reached

- **The root refuses a credential before its restart** (08.3). Default:
  step the identities first; if `try_step` returns one, step it after
  the restart's first step, as jig's reference does, and say so.
- **jig's clients' component sends `people::Event::SignOut`**
  (`jig-protocol-engine-clients/src/closing.rs:137` at the pin), and
  temper's root's `PartyInput` has no sign-out input (08.5a). Default:
  add `PartyInput::SignOut`, field for field, routed to the people child
  as the core takes it, in 08.5a's Files; add it and report it, do not
  wait. 11.3 adds it first (ruling 116); 08.5a then translates
  `people::Event::SignOut` to it.
- **A continued `CoreRange::Turns` load** (08.2a): jig's transcript
  pages continue by an offset, the root's by a key. Default: the routing
  keeps each continued transcript load's offset beside the root's last
  key in its bounded table, and a continuation it does not hold starts
  the page again from the attempt's first turn.
- **The profile's numbers** (08.1). Default: the domain worlds' fixture
  values, `name_bytes` 256, the client's `answer_bytes` 65,536; 10.2's
  checks across limits may move them, with the reason in its report.
- **02.3's outcome holds a refusal type other than `people::Refusal`**
  (08.5b). Default: translate it field for field into jig's client
  wire's `Refusal` and name the rows that have no counterpart.

## Done when

- The crate builds and its tests pass: `cargo nextest run -p
  temper-service -p temper-engine-smith` passes.
- It links neither jig's charter nor its tools: `grep -nE
  'jig-protocol-(charter|tools)' crates/temper-service/Cargo.toml`
  prints nothing.
- The forge's calls are answered unavailable in one place: `grep -n
  'Error::Unavailable' crates/temper-service/src/route.rs` prints one line.
- The pins held: `grep -oE 'forgetic/(jig|smith|skein)\.git#[0-9a-f]+'
  Cargo.lock | sort -u` prints exactly the three pinned revisions.
- The gate passed on the session's last landing: `git -C ~/src/rust/temper
  log -1 --grep='^Step: temper-08\.' --format='%(trailers:key=Gate-Result,valueonly)'
  origin/main` names `scripts/check.sh passed`.
