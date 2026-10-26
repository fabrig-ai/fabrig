# Session 09: `temper-service`'s worker, and one process

`temper-service` gets the worker's layer and the one-process composition
(protocol.md, sections 7 and 8). The layer is jig's reference worker
(`testing/jig-test-service/src/worker.rs` and `worker_route.rs` at the
pin), copied and filled where the reference leaves application parts
empty: temper's parts decoded before the root sees an assignment and
encoded beside its answer, the link's credential naming
`PAYLOAD_VERSION`, agents run as `temper agent`, LLM grants to smith's
host half, git identities' values into the worker's identities' table,
and the root's git and file operations through the git component (07),
each with its identity's newest value. Then `system.rs` joins 08's
engine layer and this worker in one service and one loop over the
in-process name `workers`: the engine starts first, a signal drains it,
and the worker ends at once once it has drained. 09.1 is split in two
(09.1a: the payloads, parts and identities as functions; 09.1b: the
layer), so the README's 09.2 and 09.3 keep their numbers.

## Read first

- `protocol.md`, sections 7, 8 and 10; `channel.md`, sections 2 to 8;
  `credentials.md`, section 4; `git.md`, sections 2, 5 and 7;
  `domain/worker.md`, section 9 (the protocol layer's boundary).
- jig's `docs/design/protocol/service.md`, sections 3 to 6 and 10, at
  the pin.
- `decisions.md`, 7, 8, 9 and 21; `code-organization.md`, sections 3, 4,
  5 and 7.
- `findings/jig-service.md`, section 1 ("Worker routing", "Where
  application-specific parts plug in", "Payload version"), Discrepancies
  1 and 6; `findings/temper-worker.md`, section 2, Findings 9 and 13.
- The code, at the pin: jig's `testing/jig-test-service/src/worker.rs`
  (682 lines; `:95` the credential's `payload: 1`, `:277-288` parts
  dropped and values into the tables, `:361-388` an agent's spawn,
  `:571-592` the root's close), `worker_route.rs` (266 lines),
  `lib.rs:42-391` (`Service`), `:393-543` (`iterate`, `route_events`),
  `src/tests/worker.rs` and `src/tests/fixtures.rs:540-600`; jig's
  `crates/jig-protocol-accounts/src/table.rs` (`Table`, two generations
  by number and generation), `crates/jig-protocol-worker-link/src/boundary.rs`,
  `crates/jig-domain-host/src/link.rs:7-55` (`LinkDown`, `LinkUp`) and
  `boundary.rs:470-625` (`Workspace.items`, `Work.left`, `Work.saved`,
  `Preparation::Permanent { resource }`).
- temper's `crates/temper-worker-domain/src/{boundary.rs, wire.rs,
  route.rs:40-41, assignment.rs, workspace.rs:143-180}`;
  `tests/worker/domain/src/{protocol.rs, evidence.rs}` (the world's own
  payload routing and evidence codec, which `worker_route` follows).
- 08's `crates/temper-service/src/{lib.rs, settings.rs, limits.rs,
  tokens.rs, translate.rs, tests/fixtures.rs}`, as 08 landed them; 07's
  `crates/temper-worker-protocol-git` and
  `tests/worker/protocol-git/src/io.rs` (09.2 on); the session files
  `temper/08-service-engine.md`, `temper/03-wire-codecs.md`,
  `temper/01-domain-parts.md` (01.3) and `temper/07-git-component.md`.

## What the design asks

- **The worker's service** (protocol.md, 7): "No store; starting, ending
  and smith's payloads are jig's". temper adds: "Agents run as `temper
  agent` ... its environment holding no credential"; "The routing
  decodes the workspace items into the root's `wire::Workspace` before
  the root sees the assignment, and encodes the root's `wire::Work` and
  `wire::Preparation` beside an answer"; "LLM accounts' go to smith's
  host half ...; git identities' go into the identities' table, from
  which the routing hands the git component an identity's value with
  each operation that names it"; "Each `git::Op` the root asks for goes
  to the git component"; "Starting: configuration, the components, the
  worker's root directory opened, then the link dialled at once".
- **The payload version** (channel.md, 3): "Both channel components are
  given it when the services make them: `jig-protocol-worker-link` puts
  it in the worker's credential".
- **By tag, never by place** (channel.md, 5): each part names its
  repositories by tag; "The worker's routing sends [the preparation] only
  for a permanent failure".
- **Translation** (channel.md, 7): the worker's `workspace`, `work`,
  `preparation` and `value`, "Total and exhaustive both ways ... No
  function reads io, keeps state or decides"; "items that do not decode
  reach the root as such, and the root refuses the start as invalid,
  naming the workspace items"; and `domain/worker.md`, 9: "Items that do
  not decode reach the root as an assignment it refuses as an invalid
  start, naming them (4.1)".
- **A git identity's value** (channel.md, 6): "the routing decodes it and
  keeps it in the worker's identities' table, at most the two newest
  generations of each, and hands it to the git component with each
  operation that names it ... A value that does not decode, or breaks a
  field's rule, is dropped and counted; an operation that needs it fails
  `Broken`."
- **A delivery's evidence** (channel.md, 2): "encoded and decoded with
  smith's codec by the worker's routing, and decoded on the worker
  against the outcome the engine committed: one that does not decode or
  disagrees refuses the start as invalid".
- **One process** (protocol.md, 8): "`temper-service`'s system module
  owns the engine's protocol layer and the worker's as children, each
  with its own root and journal, sharing io and nothing else"; "Joined
  by the in-process name `workers`"; "configuration for both, then the
  engine's start, then the worker's. A signal drains the engine; once it
  has drained, the service ends the worker at once, its agent processes'
  and git invocations' groups killed as io closes them."

## Choices this plan makes

- **09.1 is two increments:** jig's `worker.rs` (682 lines) and
  `worker_route.rs` (266) with temper's parts and the identities' table
  pass 600 lines (code-organization.md, 8). 09.1a lands the functions
  (copied payload routing, temper's parts, the table) and the root's
  `Undecodable`; 09.1b lands the layer that calls them. 18.3's "09.1" is
  09.1a's functions and 09.1b's layer.
- **The functions are public:** `temper_service::{translate,
  worker_route, identities}`, so none is dead code before 09.1b, 18.3
  and the worlds can call them (code-organization.md, 4 names
  `translate::{workspace, work, preparation, value}`), and the worker
  layer takes them by name.
- **Items that do not decode reach the root typed:** the root's
  `Event::Assign { items, .. }` becomes `Result<wire::Workspace,
  wire::Undecodable>`, and the root's entrance check refuses `Err` naming
  "workspace items". The design asks it (`domain/worker.md`, 9); the root
  has no way to hear it today (`boundary.rs:61` takes a decoded
  `wire::Workspace`).
- **The worker's identities' table is temper's own** (`IdentityTable`):
  jig's `Table` finds a value by number and generation
  (`table.rs:64-72`), while the routing hands "the identity's newest";
  it holds the git component's `Identity` (07.1), decoded and checked
  once as the value arrives. It has room for numbers `1..=` jig's worker
  part's `limits.core.accounts.credentials`, as the reference sizes its
  application table (`worker.rs:125-128`). LLM grants stay in jig's
  `Table`, as the reference has them.
- **Until 09.2, the root's `Request::Io` is answered at once** with
  `Event::Done { done: Failed { fault: Broken } }`, which the checkout
  takes as transient; `Request::CancelIo` needs nothing, as its `Done`
  has come. A chat with no repository never asks (protocol.md, 12, step
  5). 09.2 removes the arm.
- **The link dials once the git component has started** (09.2): the
  component's start (the version check, the root's layout) is among "the
  components", which come before "the link dialled at once".
- **A worker whose git is unusable stops the service as a forge's start
  check does** (protocol.md, 6): the engine drains, and the end is
  `Stop::Start` with a problem naming `application.workers[0].git`.
- **Tokens:** the worker's own from 08.1's `tokens` (prefix 3 with
  `(i + 1) << 48`), its git component's prefix 6 with `(i + 1) << 48`
  (code-organization.md, 7), `i = 0`. The git component's kernel
  operations leave through the worker's `submitted` queue and come back
  by prefix, as the reference's root close does.
- **The composition is jig's reference `lib.rs`,** moved into
  `system.rs`: one io, the kernel's router, the engine's layer, one
  worker; the worker iterates once the engine's store is open and its
  workers' listener open (`lib.rs:435-442`'s `started`); an ended engine
  aborts the worker (`lib.rs:446-452`); a second signal aborts both
  (`lib.rs:204-225`).
- **Tests drive the layers without a simulator:** the store in memory
  (08's fixture after jig's `tests/fixtures.rs:176-193`), io's joined
  sockets in process, 07.4's scripted io (`temper_worker_protocol_git_world::io::Below`)
  below the git component in 09.2, and in 09.3 a fixture kernel that
  answers what the git component's start submits.

## Rules that bind it

- "No increment of this plan: ... adds an `unreachable!` arm for a
  variant the service can reach: an arm the service reaches is routed"
  (code-organization.md, 6). Each root request and each link request is
  matched without a wildcard.
- "puts a secret's value in a domain type, a fact, a store record, a log
  or a `Debug` output but skein-lib's `Secret`'s length"
  (code-organization.md, 6); and "every world with a protocol layer
  checks that no value its fakes issued (an LLM access token, a git
  identity's token, an API token) appears in a root's events or
  requests, a fact, a store record or a log" (code-organization.md, 5).
- "**`temper-service`** links what protocol.md, section 5's table says.
  It never links `jig-protocol-charter` or `jig-protocol-tools`"
  (code-organization.md, 3).

## Increments

### 09.1a Smith's payloads, temper's parts and the identities' table, as functions

Scope: `worker_route.rs`, jig's reference `worker_route.rs` copied with
temper's root's types (`temper_worker_domain::{Encodable, Encoded,
Invalid, Event}`): `decode`, `encode`, `activity`, `process`, and
`evidence` (a resumed delivery's evidence decoded with `smith_channel`
against its outcome, as `tests/worker/domain/src/evidence.rs:17-27`
does, `Invalid::Evidence` otherwise) in place of the reference's
`delivery`. temper's parts as bytes: `items` (the part under the
assignment's `workspace.items`, decoded against the worker's limits with
03.1's codec, then `translate::workspace`), `answer_parts` (`Left`
under `work.left`, `Saved` under `work.saved`, `Preparation` under a
permanent failure's `resource`, each only where the answer names its
token), `identity` (a grant's value decoded as 03.1's `Identity`, then
`translate::value`). `translate`'s worker functions beside 08.3's engine
ones: `workspace`, `work`, `preparation`, `value`, each a `match` with
no wildcard arm on what it reads. `identities.rs`: `IdentityTable`, `Unfit`,
`identities_worst_case`. The root: `wire::Undecodable`; `Event::Assign`
takes `items: Result<wire::Workspace, wire::Undecodable>`; the entrance
check (`assignment.rs`) refuses `Err` with the words "workspace items".
Needs: 07.1's `temper_worker_protocol_git::Identity`; at the session's
start: 08.3's translations and fixtures, 03.1's records
(`temper_protocol_worker_wire::{PAYLOAD_VERSION, v1::{Items, Left,
Saved, Preparation, Identity, Limits}}`), 01.3's `wire::Work.saved` and
`wire::Preparation` by tag.
Files: `crates/temper-service/{Cargo.toml, src/{lib.rs, worker_route.rs,
translate.rs, identities.rs, tests.rs, tests/fixtures.rs,
tests/worker_parts.rs}}`; `crates/temper-worker-domain/src/{wire.rs,
boundary.rs, route.rs, assignment.rs, tests/entrance.rs,
tests/fixtures.rs, tests/lifecycle.rs}`; the users of `Event::Assign`,
each wrapping its items in `Ok`: `crates/temper-worker-domain/src/tests/fixtures.rs:268`,
`tests/entrance.rs:14,22`, `tests/lifecycle.rs:24,34`,
`tests/system/src/composed.rs:583`, `tests/worker/domain/src/runtime.rs:489`,
`tests/worker/domain/src/whole.rs:310`. Shared hotspots:
`crates/temper-worker-domain/src/wire.rs` (01.3 changed it),
`crates/temper-service/src/translate.rs` (08.3's),
`crates/temper-service/Cargo.toml`. `temper-service` gains
`smith-channel`, `smith-protocol-channel`, `smith-host-protocol`,
`smith-host-domain`, `smith-domain-run`, `jig-domain-host`,
`temper-worker-domain`, `temper-worker-protocol-git` where 08 has not
added them; `smith-host-protocol` is not in `Cargo.lock` at the base:
adding it runs code-organization.md, 3's three `cargo update --precise`
commands, and the check prints the three pins.
Shape: as the sketch's `translate`, `worker_route` and `identities`
sections. `IdentityTable::granted` keeps a number's newest and one older
generation; a generation older than both is dropped; a number past the
table's room is `Unfit`. `value` returns `None` for a login holding `:`
or a control byte, or a name or email empty or holding NUL, newline, `<`
or `>` (channel.md, 6).
Test shape: step tests in `crates/temper-service/src/tests/worker_parts.rs`;
fixtures gain `agent()` (copied from jig's reference fixtures'
`agent()`), `one_repository()` (a `wire::Workspace` of one repository,
tag 3, name `b"app"`), `identity(token)` (login `b"temper-bot"`, author
`b"Temper Bot"`, `b"bot@example.test"`). Tests:
`an_engine_s_items_reach_the_worker_s_workspace_field_for_field` (08.3's
`items` encoded, then this `items`, with the remote made from the base,
owner and name), `left_and_saved_go_up_under_the_tokens_the_answer_names`,
`a_permanent_preparation_goes_up_by_tag_and_a_transient_one_not`,
`items_whose_bytes_do_not_decode_are_undecodable`,
`an_identity_breaking_a_field_rule_is_dropped`,
`a_late_older_generation_leaves_the_newest_in_place`,
`delivery_evidence_disagreeing_with_its_outcome_is_invalid`. In the
root's `src/tests/entrance.rs`:
`workspace_items_that_did_not_decode_are_refused_naming_them`. Under
20 ms each. Reference tests, in full:

```rust
// crates/temper-worker-domain/src/tests/entrance.rs
#[test]
fn workspace_items_that_did_not_decode_are_refused_naming_them() {
    let mut h = Harness::new();
    let assign =
        Event::Assign { assignment: assignment(true, Box::new([])), items: Err(wire::Undecodable), save: None };
    assert_eq!(&*h.step(assign), [refused(b"workspace items")]);
    assert_eq!(h.domain.host().hosted(), 0);
}
```

```rust
// crates/temper-service/src/tests/worker_parts.rs
#[test]
fn a_late_older_generation_leaves_the_newest_in_place() {
    let mut table = IdentityTable::new(2);
    table.granted(1, 2, fixtures::identity(b"tok-2")).expect("number 1 has room");
    table.granted(1, 1, fixtures::identity(b"tok-1")).expect("an older generation is kept");
    assert_eq!(table.newest(1), Some(&fixtures::identity(b"tok-2")));
    assert_eq!(table.newest(2), None);
    assert_eq!(table.granted(3, 1, fixtures::identity(b"tok-3")), Err(Unfit));
}
```

Proof: unit, `cargo nextest run -p temper-service -p
temper-worker-domain -E 'test(/worker_parts|did_not_decode/)'`, run once:
an engine's items reach the worker's workspace field for field, parts go
up under their answer's tokens by tag, items that do not decode are
refused naming them, and the table hands the newest value (the eight
tests above); under 0.2 s. The layer that calls them is 09.1b; the joint
layer opens at 17.2.
Don't: decode or encode a part in a root; keep state in `worker_route`
or `translate`; put an identity's token in a fact or a `Debug`.
Size: at most 600 lines of production code.
Removes: nothing.

### 09.1b The worker's layer

Scope: `worker.rs`, jig's reference `worker.rs` copied: the worker's
root (`temper_worker_domain`) for jig's test worker; built from 08.1's
typed settings (decision 9: `settings::Worker { jig, root, git }`,
`settings::Agent`, the checkout's limits from `limits::Limits.checkout`)
where the reference parses jig's configuration, its checks returning `Problems` with the reference's
paths; the link's credential with `temper_protocol_worker_wire::PAYLOAD_VERSION`
in place of `payload: 1` (`worker.rs:95`), made by a crate-visible
`credential(settings, secret) -> link::Credential`, the secret
resolved from the `files: &settings::Files` `Worker::new` takes (08.1's
type), by `settings.jig.secret`, with the reference's two checks
(`worker.rs:66-68`: not there `Unreadable`, past
`settings.jig.limits.link.secret` `Bound`, each naming `secret`; decisions.md,
ruling 127); agents spawned as
the reference spawns them, the program and arguments from jig's worker
part's `agent` (temper's binary, `agent <AGENT.json>`), environment
empty; each `link::Request` through a crate-visible `Worker::link(now,
request)`, which the link's drain calls and the tests call with a record:
`Down` for an assignment whose `workspace` is some becomes `Event::Assign
{ assignment, items: worker_route::items(..), save }`, any other `Down`
`Event::Link`; `Value` for `Accounts` into jig's `Table`, for
`Application` through `worker_route::identity` into the `IdentityTable`,
a value that does not decode dropped and counted (`undecodable_values`);
the root's
`Request::Answer` up as `link::Event::Up { up: LinkUp::Answer { run,
attempt, answer }, parts: worker_route::answer_parts(..) }`; `Decode`,
`Encode`, `Fact` and process events through 09.1a's `worker_route`; the
root's `Request::Io` answered `Failed { fault: Broken }` at once,
`CancelIo` nothing (Choices). `lib.rs` exports `Worker`,
`WorkerRuntime` and `worker_worst_case`.
Needs: nothing beyond 09.1a; 08.1's `settings::{Worker, Agent}`,
`limits::Limits` and `tokens` (the sketch's stand-ins).
Files: `crates/temper-service/{Cargo.toml, src/{lib.rs, worker.rs,
tests/worker.rs, tests/fixtures.rs}}`. Shared hotspot:
`crates/temper-service/Cargo.toml` (it gains `jig-protocol-worker-link`
and `jig-protocol-accounts` where 08 has not: the pins' three commands
then run, as 09.1a says).
Shape: as the sketch's `worker` section, without `git`'s parts (09.2):
`WorkerRuntime { root }` only, `refused()` always `None`.
Test shape: step tests in `src/tests/worker.rs`, after jig's reference
`src/tests/worker.rs`; fixtures gain `worker()` (a `Worker` from 08's
settings fixture and a `Files` holding its secret's path, root
`Fd::new(9)`), `spawn(&mut Worker) -> (Token,
kernel::Spawn)` (copied from the reference's `:13-22`), `TEMPER =
b"/opt/temper/bin/temper"`, `AGENT_JSON = b"/opt/temper/agent.json"`,
`NOW`, `chat_assignment()` and `repository_assignment()` (each a
`link::Request::Down`, the second with its items' part),
`application_value(number, generation, token)` (a `link::Request::Value`
for the application's table). Tests:
`the_link_s_credential_names_temper_s_payload_version`,
`an_assignment_with_items_reaches_the_root_decoded_with_its_save`,
`undecodable_items_refuse_the_start_as_invalid_naming_them`,
`an_answer_goes_up_with_its_left_and_saved_parts`,
`a_git_identity_s_value_lands_in_the_identities_table_only`,
`the_agent_runs_as_temper_agent_with_no_credential_in_its_environment`,
`a_git_operation_fails_broken_until_the_git_component_is_wired`. Under
20 ms each. Reference test, in full:

```rust
#[test]
fn the_agent_runs_as_temper_agent_with_no_credential_in_its_environment() {
    let mut worker = fixtures::worker();
    worker.link(fixtures::NOW, fixtures::chat_assignment());
    let (_, launched) = fixtures::spawn(&mut worker);
    assert_eq!(launched.program.as_ref(), fixtures::TEMPER);
    assert_eq!(launched.args.as_ref(), [Box::from(&b"agent"[..]), Box::from(fixtures::AGENT_JSON)]);
    assert!(launched.env.is_empty(), "no credential reaches an agent");
}
```

Proof: unit, `cargo nextest run -p temper-service -E
'test(/tests::worker::/)'`, run once: the credential names
`PAYLOAD_VERSION`, parts cross both ways under their tokens, an identity
lands in the identities' table, and an agent starts as `temper agent`
with no credential (the seven tests above); under 0.3 s. The joint layer
needs 09.3's composition and 17.2's world.
Don't: hard-code a payload version; drop a part the hub names; hand an
identity's value to smith's host half or the hub; add an `unreachable!`
arm for a root or link request.
Size: at most 600 lines of production code.
Removes: nothing.

### 09.2 The git component wired

Scope: 08.1's `limits::Limits` gains `git_component:
temper_worker_protocol_git::Limits` (its profile `small` constants, its
`git_component.` overrides in `limits/names.rs`, configuration.md, 7).
The worker's layer owns a `temper_worker_protocol_git::Component` made
from `limits::Limits.git_component`, `limits::Limits.checkout` and
`Settings { git: settings::Worker.git, home: <WorkerRuntime::path>/home, root, prefix: the
git prefix for worker 0, serial: drawn from the worker's seed,
largest_read, largest_room: io's }`;
`WorkerRuntime` gains `path`; the component's `start` at the worker's
first iterate, the link fired only after `Event::Started`; `Unusable`
sets `refused()` to a problem naming `application.workers[0].git`; the
root's `Request::Io { owner, op, deadline }` becomes the component's
`Request::Io` with `identity`: the op's `identity` number (Clone, Fetch,
Create, Commit, Push) looked up with `IdentityTable::newest` and cloned,
`None` for Make, CheckOut, Merge or a number with no value;
`Request::CancelIo` passed on; `Event::Done` stepped into the root as its
`Event::Done`; io events by the component's `accepts` to `up`; its
`kernel` outputs into the worker's `submitted`; completions by `owns` to
`complete`; deadlines by `next_deadline` and `is_due`/`fire`; `stop()`
sends `Request::Shutdown`; `abort()` calls the component's `abort`;
`work_pending`, `is_empty`, `operations` and `worker_worst_case` count
the component.
Needs: 07.3's component (07.3b's: the layout, Make and the scan). Its
tests take 07.4's `temper_worker_protocol_git_world::io::Below` as a
dev-dependency; if 07.4 has not landed when 09.2 starts, its fixtures
answer the component's start and one clone themselves (at most 80
lines of test code), and the report says so.
Files: `crates/temper-service/{Cargo.toml, src/{worker.rs, worker_git.rs
(new: the component's routing, so `worker.rs` stays within 800 lines),
limits.rs, limits/names.rs, lib.rs, tests/worker_git.rs,
tests/limits.rs, tests/fixtures.rs}}`.
Shape: as the sketch's `WorkerRuntime`, `Worker::refused`,
`Worker::complete` and `Worker::operations`.
Test shape: step tests in `src/tests/worker_git.rs`; fixtures gain
`started_worker()` (a worker whose git component has started, its start
answered by `Below`), `IDENTITY = 1` (the forge's git identity's
number), `answer_until_spawn(&mut Worker, command) -> kernel::Spawn`
(answers the operations before the first spawn whose first argument
after the settings is `command`, with `Below`).
Tests: `a_clone_reaches_git_with_its_identity_s_newest_value`,
`an_operation_whose_identity_has_no_value_fails_broken_without_git`,
`a_cancel_reaches_the_component_and_its_done_reaches_the_root`,
`the_link_dials_only_once_the_git_component_has_started`,
`an_unusable_git_refuses_the_worker_naming_its_program`,
`the_component_s_file_operations_leave_through_the_worker`. Under 20 ms
each. Reference test, in full:

```rust
#[test]
fn a_clone_reaches_git_with_its_identity_s_newest_value() {
    let mut worker = fixtures::started_worker();
    worker.link(fixtures::NOW, fixtures::application_value(fixtures::IDENTITY, 1, b"tok-old"));
    worker.link(fixtures::NOW, fixtures::application_value(fixtures::IDENTITY, 2, b"tok-new"));
    worker.link(fixtures::NOW, fixtures::repository_assignment());
    let clone = fixtures::answer_until_spawn(&mut worker, b"clone");
    // base64 of b"temper-bot:tok-new"
    let newest = &b"GIT_CONFIG_VALUE_0=Authorization: Basic dGVtcGVyLWJvdDp0b2stbmV3"[..];
    assert!(clone.env.contains(&Box::from(newest)), "the newest generation reaches git");
    for argument in clone.args.iter() {
        assert_eq!(skein_lib::bytes::find(argument, b"tok-"), None, "no token in an argument");
    }
}
```

Proof: unit, `cargo nextest run -p temper-service -E
'test(/tests::worker_git::/)'`, run once: a clone reaches git with the
newest identity's header and no token in its arguments, a missing value
fails `Broken` unspawned, a cancel and a `Done` cross, and the link waits
for git's start (the six tests above); under 0.3 s. The joint layer is
17.2's.
Don't: let the component read the table; keep an identity's value past
the call that hands it over; dial before `Started`.
Size: at most 300 lines of production code.
Removes: 09.1b's `Failed { fault: Broken }` answer to the root's
`Request::Io`.

### 09.3 One process

Scope: `system.rs`: jig's reference `lib.rs` composition (`Service`,
`iterate`, `route_events`, `route_components`, the kernel's router, the
completions handed to the store, the worker or io) with one engine
layer (08's) and one worker layer: the worker iterates once the engine's
store is open and its workers' listener is open; the worker's link and
the engine's listener meet by the in-process name `workers` in the one
io; the first signal drains the engine (08's `stop`), and once the
engine has an end the worker is aborted at once; a second signal aborts
both; a worker `refused()` stops the engine as a forge's start check
does, the end `Stopped(Stop::Start(problems))`; `fmt_roots` over the
engine's root and the worker's (`lib.rs:120`); `operations` adds the
worker's. `temper_service::{Service, iterate}` are `system`'s, public,
where 08 placed an engine-only `Service` (08's `Service` moves here, its
users following).
Needs: 08.5b's forge family (the engine layer this composes, as the
README's row says).
Files: `crates/temper-service/src/{system.rs (new), lib.rs, engine.rs or
wherever 08 placed `Service` and `iterate`, tests/system.rs,
tests/fixtures.rs}`; 08's tests that construct `Service`, following it.
Shape: as the sketch's `system` section; construction stays
crate-visible, for 10.1's `startup`. 09.3 adds `Stop::Start(Problems)`
to 08's `Stop` (an unusable git, 09.2); 14.1 adds `Stop::Forge {
number, why }` separately, for a forge's refused start check.
Test shape: step tests in `src/tests/system.rs`, the store in memory;
fixtures gain `one_process()` (an engine and a worker from 08's
settings fixture), `Kernel` (answers what the git component's start
submits: `git version` spawned and printing `git version 2.53.0\n`, its
pipes read to their ends, its wait, the layout's file operations; any
other kernel record fails the test naming it), `turn(&mut Service)`,
`until_hosting(&mut Service)`. Tests:
`the_worker_dials_workers_and_its_hello_reaches_the_engine`,
`the_worker_iterates_only_once_the_engine_listens`,
`a_signal_drains_the_engine_then_ends_the_worker_at_once`,
`a_second_signal_aborts_both_layers`,
`io_events_and_completions_reach_the_layer_whose_tokens_they_carry`,
`an_unusable_git_stops_the_service_naming_its_program`. Under 0.1 s
each. Reference test, in full:

```rust
#[test]
fn a_signal_drains_the_engine_then_ends_the_worker_at_once() {
    let mut service = fixtures::one_process();
    fixtures::until_hosting(&mut service);
    service.shutdown();
    for _ in 0_u32..32 {
        fixtures::turn(&mut service);
    }
    assert_eq!(service.end(), Some(&End::Drained));
    assert!(service.is_empty(), "the worker ended with the engine");
}
```

Proof: unit, `cargo nextest run -p temper-service -E
'test(/tests::system::/)'`, run once: the worker's hello reaches the
engine over `workers`, a signal drains the engine and then ends the
worker, and an unusable git stops the service naming it (the six tests
above); under 0.5 s. The joint layer opens at 17.2 (`tests/one-process`),
the process tier at 10.4.
Don't: route an event by trying layers in turn where a token's prefix
decides; let the worker iterate before the engine listens; give the
worker a grace after the engine's end.
Size: at most 300 lines of production code.
Removes: 08's engine-only `Service` and `iterate`, moved into `system`.

## Interface sketches

`temper-service` owns the worker's layer, its routing and the
composition; the root owns what a part means; the git component owns
its operations. A part lives only while it is encoded or decoded; an
identity's value lives in the table until a newer pair pushes it out,
and in the git component while its operation is in flight. The sketch
compiles as `plan-sketches`' `s09.rs`; its stand-ins at the end are the
other sessions' types (03.1's records, 08.1's settings, 08's `End`).

```rust
//! Sketches of session 09: `temper-service`'s worker layer, its routing of
//! smith's payloads and temper's parts, the worker's identities' table, the
//! git component wired, and the one-process composition. Types other
//! sessions own are stand-ins at the end, named for their owners; their
//! sketches and landed code define them, and win where these differ.
#![expect(unused_variables, reason = "a sketch's bodies are unreachable")]

use alloc::boxed::Box;
use jig_domain_host as host;
use jig_protocol_config::Problems;
use jig_protocol_worker_wire::Part;
use skein_io::{self as io, kernel};
use skein_lib::{Queue, Secret, Time, Wall};
use temper_worker_domain::{self as root, wire};

use crate::s07 as git_component;

// temper_worker_domain::wire (09.1a adds)

/// Workspace items whose bytes did not decode (channel.md, section 7). The
/// root's `Event::Assign { items, .. }` takes `Result<wire::Workspace,
/// Undecodable>`; an `Err` is refused as an invalid start naming "workspace
/// items" (`domain/worker.md`, section 9), as items beyond its limits are.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Undecodable;

// temper_service::translate, the worker's functions (09.1a; channel.md, section 7)

/// The root's workspace items and the saved-work branch, as its
/// `Event::Assign` takes them.
pub type Assigned = (wire::Workspace, Option<Box<[u8]>>);

/// The worker's side of `Items`: the root's workspace and the saved-work
/// branch; no part is an empty workspace and no save.
#[must_use]
pub fn workspace(items: Option<&payload::Items>) -> Assigned {
    unreachable!("sketch")
}

/// What a run left (`Left`, its landed heads by tag) and saved (`Saved`, by
/// tag with `from_saved`, absent without a save), field for field.
#[must_use]
pub fn work(work: &wire::Work) -> (payload::Left, Option<payload::Saved>) {
    unreachable!("sketch")
}

/// A failed preparation by its repository's tag; none for a transient one,
/// which jig's class says.
#[must_use]
pub fn preparation(preparation: &wire::Preparation) -> Option<payload::Preparation> {
    unreachable!("sketch")
}

/// A grant's decoded `Identity` as the git component takes it; `None` when a
/// field breaks its rule (channel.md, section 6), which the routing counts.
#[must_use]
pub fn value(identity: &payload::Identity) -> Option<git_component::Identity> {
    unreachable!("sketch")
}

// temper_service::worker_route (09.1a): jig's reference `worker_route.rs`,
// copied, and temper's parts as bytes. Each keeps no state.

/// The hub's start into smith's, against the worker's agent configuration;
/// a resumed delivery's evidence through `evidence`.
pub fn decode(run: skein_lib::Token, source: host::Start, agent: &settings::Agent) -> Result<Box<root::agent::Start>, root::Invalid> {
    unreachable!("sketch")
}

/// A resumed delivery's evidence: smith's channel record of the delivery's
/// reply, which must decode and agree with `outcome` (`Invalid::Evidence`
/// otherwise; channel.md, section 2).
pub fn evidence(outcome: host::DeliveryOutcome, evidence: &[u8]) -> Result<root::agent::Delivery, root::Invalid> {
    unreachable!("sketch")
}

/// One of smith's payloads, encoded for the hub, as the reference does.
#[must_use]
pub fn encode(value: root::Encodable, agent: &settings::Agent) -> root::Encoded {
    unreachable!("sketch")
}

/// One of smith's channel facts as jig's activity, if it decodes.
#[must_use]
pub fn activity(body: &[u8], agent: &settings::Agent) -> Option<host::Activity> {
    unreachable!("sketch")
}

/// smith's host half's process event as the root's, as the reference does.
#[must_use]
pub fn process(event: smith_host_protocol::ProcessEvent) -> Option<root::Event> {
    unreachable!("sketch")
}

/// The assignment's workspace items: the part under `workspace.items`,
/// decoded against the worker's limits, then `workspace`.
pub fn items(assignment: &host::Assignment, parts: &[Part], limits: &payload::Limits) -> Result<Assigned, Undecodable> {
    unreachable!("sketch")
}

/// The parts beside an answer: `Left` under `work.left`, `Saved` under
/// `work.saved`, `Preparation` under a permanent failure's `resource`, each
/// only where the answer names its token; none for a refusal.
#[must_use]
pub fn answer_parts(answer: &host::Answer, work: &wire::Work, preparation: Option<&wire::Preparation>) -> Box<[Part]> {
    unreachable!("sketch")
}

/// A grant's value from the application's table: `Identity` decoded, then
/// `value`; `None` is dropped and counted.
#[must_use]
pub fn identity(value: &Secret, limits: &payload::Limits) -> Option<git_component::Identity> {
    unreachable!("sketch")
}

// temper_service::identities (09.1a; credentials.md, section 4)

/// The worker's identities' table: each git identity's two newest
/// generations, by the number the grants name, filled from the link's
/// values. Only the routing reads it, for each operation naming an identity.
#[derive(Debug)]
pub struct IdentityTable {}

/// A number or a value beyond the table's bounds; the value is dropped.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Unfit;

impl IdentityTable {
    /// Room for numbers `1..=identities`.
    #[must_use]
    pub fn new(identities: u32) -> IdentityTable {
        unreachable!("sketch")
    }
    /// Keeps `identity` as `number`'s `generation`; the older of three goes.
    pub fn granted(&mut self, number: u32, generation: u64, identity: git_component::Identity) -> Result<(), Unfit> {
        unreachable!("sketch")
    }
    /// `number`'s newest generation, if any.
    #[must_use]
    pub fn newest(&self, number: u32) -> Option<&git_component::Identity> {
        unreachable!("sketch")
    }
}

/// Two generations of `identities` numbers, each value within `value_bytes`.
#[must_use]
pub fn identities_worst_case(identities: u32, value_bytes: u32) -> Option<u64> {
    unreachable!("sketch")
}

// temper_service::worker (09.1b, 09.2): jig's reference `worker.rs`, copied

/// What the binary prepared for the worker: its root directory, opened (made
/// private if absent), and, from 09.2, that directory's absolute path, from
/// which the git component's `HOME` is made.
#[derive(Debug)]
pub struct WorkerRuntime {
    pub root: kernel::Fd,
    pub path: Box<[u8]>,
}

/// The worker's layer: its root, the link, smith's host half and agent
/// processes, the LLM accounts' table, the identities' table, the git
/// component (09.2). Fields elided.
#[derive(Debug)]
pub struct Worker {}

impl Worker {
    /// Refused naming the path of what does not fit (the reference's checks,
    /// and 09.2's git component's `Unfit` as `git_component.<limit>`).
    pub fn new(
        settings: &settings::Worker,
        agent: &settings::Agent,
        limits: &limits::Limits,
        // The private files startup read; the worker's secret by
        // `settings.jig.secret`, as the reference's (ruling 127).
        files: &settings::Files,
        runtime: WorkerRuntime,
        seed: u64,
    ) -> Result<Worker, Problems> {
        unreachable!("sketch")
    }
    /// One pass: link, agent processes, the git component, root events.
    pub fn iterate(&mut self, now: Time, wall: Wall) {
        unreachable!("sketch")
    }
    /// Whether `event`'s owner is its link's, an agent's or git's.
    #[must_use]
    pub fn accepts(&self, event: &io::Event) -> bool {
        unreachable!("sketch")
    }
    pub fn up(&mut self, now: Time, event: io::Event) {
        unreachable!("sketch")
    }
    /// A kernel completion of its own (its root's close, its git component's
    /// file operations), or `Err` handing back another's.
    pub fn complete(&mut self, now: Time, complete: kernel::Complete) -> Result<(), kernel::Complete> {
        unreachable!("sketch")
    }
    /// What it asks of io and of the kernel, for the service to submit.
    pub fn io(&mut self) -> &mut Queue<io::Request> {
        unreachable!("sketch")
    }
    pub fn submitted(&mut self) -> &mut Queue<kernel::Submit> {
        unreachable!("sketch")
    }
    /// Shutdown: the root hears it; the git component sweeps no more.
    pub fn stop(&mut self) {
        unreachable!("sketch")
    }
    /// The end at once: every entity aborted, the root's work dropped.
    pub fn abort(&mut self) {
        unreachable!("sketch")
    }
    /// Its start was refused once running: the git component was unusable
    /// (09.2), naming `application.workers[0].git`.
    #[must_use]
    pub fn refused(&self) -> Option<&Problems> {
        unreachable!("sketch")
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        unreachable!("sketch")
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        unreachable!("sketch")
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        unreachable!("sketch")
    }
    /// The kernel operations it may have in flight: its root's close and its
    /// git component's.
    #[must_use]
    pub fn operations(&self) -> u32 {
        unreachable!("sketch")
    }
}

/// The worker layer's memory: the reference's sum, the identities' table
/// and, from 09.2, the git component's.
#[must_use]
pub fn worker_worst_case(settings: &settings::Worker, agent: &settings::Agent, limits: &limits::Limits) -> Option<u64> {
    unreachable!("sketch")
}

// temper_service::system (09.3; protocol.md, section 8)

/// One process: io, the kernel's queues, the engine's layer and the worker's,
/// sharing io and nothing else, routing io's events by their tokens. Made by
/// 10.1's `startup`. Fields elided.
#[derive(Debug)]
pub struct Service {}

impl Service {
    /// The deployment directory the binary opened, for the store.
    pub fn adopt_store(&mut self, root: kernel::Fd, effective_user: u32) -> Result<(), kernel::Fd> {
        unreachable!("sketch")
    }
    pub fn adopt_signals(&mut self, descriptor: kernel::Fd) -> Result<(), kernel::Fd> {
        unreachable!("sketch")
    }
    pub fn completions(&mut self) -> &mut Queue<kernel::Complete> {
        unreachable!("sketch")
    }
    pub fn submissions(&mut self) -> &mut Queue<kernel::Submit> {
        unreachable!("sketch")
    }
    /// The sign-in links the engine prints, one per line on stdout.
    pub fn links(&mut self) -> &mut Queue<Box<[u8]>> {
        unreachable!("sketch")
    }
    /// The first signal drains the engine, then the worker is ended at once;
    /// the second aborts both.
    pub fn shutdown(&mut self) {
        unreachable!("sketch")
    }
    #[must_use]
    pub fn end(&self) -> Option<&stand_in::End> {
        unreachable!("sketch")
    }
    #[must_use]
    pub fn worst_case(&self) -> u64 {
        unreachable!("sketch")
    }
    /// The ring's size: io's operations, the store's, the worker's.
    #[must_use]
    pub fn operations(&self) -> u32 {
        unreachable!("sketch")
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        unreachable!("sketch")
    }
    #[must_use]
    pub fn next_policy_deadline(&self) -> Option<Time> {
        unreachable!("sketch")
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        unreachable!("sketch")
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        unreachable!("sketch")
    }
    /// The two roots alone, for a world's secrets check.
    pub fn fmt_roots(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unreachable!("sketch")
    }
}

/// One turn of the one loop: io's up pass, the engine's layer, the worker's
/// once the engine has started, the components, io's down pass.
pub fn iterate(service: &mut Service, now: Time, wall: Wall) {
    unreachable!("sketch")
}

// Stand-ins, so this sketch compiles.

/// 03.1's generated records (`temper_protocol_worker_wire::v1`), fields elided.
pub mod payload {
    /// The workspace items, down beside an assignment (channel.md, 4).
    #[derive(Debug)]
    pub struct Items {}
    /// What a run left, up beside its answer (channel.md, 5).
    #[derive(Debug)]
    pub struct Left {}
    /// What a run saved, up beside its answer (channel.md, 5).
    #[derive(Debug)]
    pub struct Saved {}
    /// A permanent failed preparation, up as the answer's resource.
    #[derive(Debug)]
    pub struct Preparation {}
    /// A git identity's value, down in a grant (channel.md, 6).
    #[derive(Debug)]
    pub struct Identity {}
    /// The family's limits, each part's decoding bound.
    #[derive(Debug)]
    pub struct Limits {}
}

/// 08.1's typed settings (`temper_service::settings`), as 09 reads them.
pub mod settings {
    use alloc::boxed::Box;

    /// The worker's agent profile, as jig's reference service's `Agent`.
    #[derive(Debug)]
    pub struct Agent {
        pub endpoints: smith_protocol_channel::Endpoints,
        pub limits: smith_host_protocol::Limits,
        pub bounds: jig_domain_host::Bounds,
        pub host_limits: smith_host_domain::Limits,
    }

    /// A worker's settings: jig's part and temper's (configuration.md, 4).
    #[derive(Debug)]
    pub struct Worker {
        /// jig's worker part.
        pub jig: jig_protocol_config::Worker,
        /// The worker's root directory: workspaces, trash and home under it.
        pub root: Box<[u8]>,
        /// `application.workers[0].git`: git's absolute path.
        pub git: Box<[u8]>,
    }
}

/// 08.1's limits (`temper_service::limits`), the fields 09 reads; `root`
/// and `forge` elided.
pub mod limits {
    /// temper's own limits by prefix; 09.2 adds `git_component`.
    #[derive(Debug)]
    pub struct Limits {
        /// `checkout.`: the worker root's checkout child.
        pub checkout: temper_worker_domain_checkout::Limits,
        /// `git_component.` (09.2): the git component's (git.md, 8).
        pub git_component: crate::s07::Limits,
    }
}

/// 08's service end (jig's reference `End` and `Stop`).
pub mod stand_in {
    /// How the service ended; `temper serve` exits 0 on `Drained` only.
    #[derive(Debug)]
    pub enum End {
        /// A signal's drain finished.
        Drained,
        /// A second signal, or a stop, cut the drain short.
        Aborted,
        /// The engine stopped itself.
        Stopped(Stop),
    }
    /// Why the engine stopped itself.
    #[derive(Debug)]
    pub enum Stop {
        /// A commit failed (jig's `protocol/service.md`, 5).
        CommitFailed { number: u64 },
        /// Added by 09.3: the worker refused once running, its git
        /// unusable (09.2); 14.1 adds `Forge { number, why }` apart.
        Start(jig_protocol_config::Problems),
        /// A listener failed.
        Listener,
    }
}
```

## Questions to raise when reached

- **One io in one process.** The reference makes one `Io` from the
  engine's limits; its entity slots must also hold the worker's link,
  `slots × 4` agent entities and `git_component.operations × 4` git
  entities. Default: one io from the engine's limits, and 10.2's checks
  across refuse a start whose entity slots cannot hold them, naming the
  io limit.
- **A value after its assignment.** If the link hands an assignment
  before its grant's value, the first Clone finds no value. Default: the
  operation fails `Broken`, the checkout's prepare fails transient and
  the engine places the run again; the report says if 09.2's tests saw
  it.
- **Counting undecodable values.** channel.md, 6 says "dropped and
  counted" and names no fact. Default: a counter on the worker's layer,
  read by tests, no fact.
- **A worker refused once running.** git.md, 3 refuses "the start" while
  the engine's layer already runs in one process. Default: the engine
  drains, as at a forge's failed start check (protocol.md, 6), and the
  end names `application.workers[0].git`.
- **The worlds' own payload routing.** `tests/worker/domain/src/protocol.rs`
  and `evidence.rs` keep their copies. Default: kept; pointing them at
  `temper_service::worker_route` is a later change.

## Done when

- The service and its users pass: `cargo nextest run -p
  temper-service -p temper-worker-domain -p temper-worker-domain-world -p
  temper-system-world` passes.
- No payload version is hard-coded: `git grep -n 'payload: 1' --
  crates/temper-service/src` prints nothing.
- jig's charter and tools are not linked: `git grep -n -E
  'jig-protocol-(charter|tools)' -- crates/temper-service/Cargo.toml`
  prints nothing.
- No `unreachable!` in the worker's routing or the composition: `git
  grep -n 'unreachable!' -- crates/temper-service/src/worker.rs
  crates/temper-service/src/worker_git.rs crates/temper-service/src/worker_route.rs
  crates/temper-service/src/system.rs` prints nothing.
- The pins held: `grep -oE 'forgetic/(jig|smith|skein)\.git#[0-9a-f]+'
  Cargo.lock | sort -u` prints the three pinned revisions and no other line.
- The gate passed on this session's last landing: `git -C
  ~/src/rust/temper log -1 --grep='^Step: temper-09\.3$'
  --format='%(trailers:key=Gate-Result,valueonly)' origin/main` names
  `scripts/check.sh passed`.
