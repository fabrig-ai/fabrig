# Session 02: locate, and the adoption as the forge's own keyed request, kept durably

A repository is adopted by the forge's own keyed request, as jig asks of
an application's request about its own objects, instead of a party's
`people::Ask::Adopt` built by hand with the repository's id in it. The
forge client gains `Read::Locate`, which turns an owner and a name into
the forge's id, and the fake forge answers it. The root takes the
party's typed adoption (`PartyInput::Application`), keeps it for the
core's admission, and routes jig's application flow to the forge
connector and back; the connector refuses a forge it does not configure,
locates the repository, makes jig's adoption on the party's behalf as
the application's part, reads what temper may do there as it does
today, and keeps the outcome under the people child's token, durably,
to give again when the same key comes again. The old path, where a
party's own `Ask::Adopt` reaches the forge, goes, and the worlds adopt
through the keyed request. Four increments, each proven in the forge's
or the engine's worlds.

## Read first

- `docs/design/domain/forge.md`, section 4 (adoption, its request, the
  forge's host) and section 17 ("Locate, new").
- `docs/design/client.md`, 4.4 (the form's fields), 4.6 (the records,
  the outcome), 4.7 (the outcome in words, the reasons) and section 8
  (the routing, "The forge's errors as reasons").
- `docs/design/forge.md`, 3.1 ("Repositories by id": a locate "is
  addressed with the forge's number and repository 0"), 3.2's
  `Read::Locate` row, section 12 ("The forge client").
- `docs/design/store/README.md`, section 2 (byte 15, `Application`).
- jig at the pin (`~/.cargo/git/checkouts/jig-aba39083bf77fefe/d91a89c`):
  `docs/design/domain/connectors.md`, 5.5 ("its keyed requests") and
  section 12; `docs/design/client/domain/contract.md`, 3.3;
  `docs/design/domain/people.md`, 5.1 (the last two bullets).
- `findings/temper-engine.md`, 2(a), 2(b), Findings 2, 3, 4, 7, 8, 12;
  `findings/jig-components.md`, section 5 and Discrepancies 1, 2.
- `decisions.md`, 3 and 23.
- jig's reference application flow at the pin:
  `testing/jig-test-domain/src/lib.rs:182-191` (the root's input) and
  :621-637 (the owner minted, the core stepped); `routes.rs:41-46`
  (`Held::Applied`), :134-153 (the asks), :540-570 (the connector's
  requests back); `crates/jig-domain-core/src/routing/vocabulary.rs:41-53,
  354-366` and `routing/application_routes.rs:15-143` (admission, part,
  finish: `DropApplication` follows every admission in the same step);
  `crates/jig-domain-core/src/routing/people.rs:41-49` (a part's
  `Decided` becomes `ApplicationPartAnswered`);
  `crates/jig-domain-people/src/domain.rs:1764-1836`
  (`application_part`); `testing/jig-test-connector/src/application.rs`
  (outcomes kept with their retention's end, one expired per decision).
- temper's code: `crates/temper-engine-domain-forge/src/{boundary.rs:60-150,
  359-606, domain.rs:16-35, 1058-1322, 3580-3640, lib.rs:185-207,
  limits.rs:9-60, tests.rs:1-110, 239-400}`;
  `crates/temper-engine-domain/src/route/{forge_route.rs:14-145,
  280-298, 2039-2120, asks.rs:16-90, held.rs:110-132}`;
  `crates/temper-engine-domain/src/{boundary.rs:300-372, decision.rs:53-140,
  route.rs:90-120}`; `crates/temper-engine-domain-forge-client/src/{api.rs:40-200,
  405-560, bounds.rs:26-80, 150-195, tests.rs:58-146}`;
  `testing/temper-fake-forge-domain/src/{api.rs:30-60, 330-350, 670-690,
  scenario.rs:17-45, domain.rs:500-531, reads.rs:38-90, store.rs:20-60}`;
  `tests/engine/forge/client/src/{world.rs:95-140, 195-215, 420-440,
  610-625, translate.rs:70-80, 180-260}`; `tests/engine/forge/src/world.rs:170-200`;
  `tests/engine/domain/src/direct.rs`; `tests/engine/domain/tests/forge_routes.rs:255-312,
  392-470, 592-636, 2003-2134`.

## What the design asks

- **Locate** (domain/forge.md, 4): "The adoption names the forge, the
  repository's owner and its name; the connector first locates it, a
  read whose answer is the forge's own id for the repository, and names
  the repository by that id from then on ... A repository the forge does
  not have refuses the adoption as not found." forge.md, 12:
  "`Read::Locate { owner, name }`, answered `Answer::Located { repository
  }`, addressed with the forge's number and repository 0 (3.2); its
  answer's shape checked as every other's, and its owner and name
  counted in the call's bytes."
- **The forge's own request** (domain/forge.md, 4): "the root takes it
  from the protocol layer typed, and the connector refuses a forge the
  deployment does not configure; then locates the repository ...; then
  the root makes jig's adoption request on the person's behalf (people.md,
  5.1), in which the connector reads what temper may do there (below)
  before the core commits the adoption ... Its outcome is the adoption's:
  adopted, refused by the core with jig's reasons, or refused by the
  forge with the connector's. The connector keeps it durably under the
  people child's token until its retention ends, to give again when the
  request is sent again."
- **The request's fields** (client.md, 4.4): "`forge::Adoption`'s ... but
  the repository's id, which the forge connector locates by owner and
  name, and the forge's host, which the engine fills from the forge's
  configuration (section 8)."
- **The outcome** (client.md, 4.6): "*adopted,* with the repository as
  the forge connector recorded it: its forge's number and its id; its
  owner and name; its role; whether it is the project's home; what temper
  may do there (the forge's `Kinds` ...); and whether branch protection
  was read (unknown, absent, or a rule); *refused by the core,* ... jig's
  refusal facts ...; *refused by the forge,* with the forge connector's
  reason: no such forge; not found; no access; refused; in flight; too
  large; unreachable; invalid answer." "A refusal at admission ... is not
  an outcome: it is jig's answer, which never reaches the connector."
- **The reasons** (client.md, 8): "`Missing` is not found; `Forbidden` is
  no access; `Refused` is refused; `Busy` is in flight; `TooLarge` and
  `Full` are too large; `Unavailable`, `Timeout` and `RateLimited` are
  unreachable; any other is an invalid answer."
- **Kept durably** (store/README.md, 2): "15 | `Application` |
  `Application(u64)`: the people child's token for the request | a keyed
  adoption's outcome, kept until its retention ends ... |
  `applications`".

## Choices this plan makes

- **The root keeps the typed value for one step.** jig's core admits one
  application at a time, synchronously, and follows every admission with
  `DropApplication { owner }` in the same step
  (`application_routes.rs:15-41`). The root mints `owner` from a counter
  of its own, keeps `(owner, adoption)` in one slot, hands the value to
  the forge with the core's `Ask::Application`, and clears the slot at
  `DropApplication`; nothing of it outlives the step.
- **The forge connector decides the application** in a new module
  `crates/temper-engine-domain-forge/src/adoption.rs`; `domain.rs` (a
  large file) gains only the dispatch arms. Its outcome is kept with the
  wall time its retention ends (`until`), as jig's reference connector
  keeps its own (`application.rs:118-196`): at most `limits.applications`
  kept and `limits.adoptions` pending, the next answered
  `ApplicationAgain { why: Busy }`; one outcome past `until` is dropped
  at each new application.
- **The forge's own refusal wins.** While an application waits for its
  part, the connector records the reason of any error its adoption reads
  meet (`reason`, the interface sketch) and the row it adopted; when the
  core answers the part, a recorded reason is the outcome
  (`ForgeRefused`), else the core's refusal (`CoreRefused`), else the
  adopted row (`Adopted`). The root keeps mapping the forge's error to
  jig's facts for the core's `Decided` (forge_route.rs:2083-2117), so the
  core's part ends as today.
- **`temper-engine-domain-forge` links `jig-domain-people`,** for
  `people::Refusal`, `people::Again` and `people::Outcome` in its
  vocabulary and its kept outcome (jig's reference connector mirrors
  them in a crate of its own instead; temper's connector already meets
  jig's contract). `jig-domain-people` is in `Cargo.lock` at the pin, so
  no pin moves.
- **The outcome holds what client.md, 4.6 names** (`AdoptedRepository`),
  not the whole `forge::Repository` row, so 03.2's record and 04.4's
  family mirror it field for field.
- **`Delivery::Application.outcome` is an `Option`:** none when the
  outcome's retention has ended though the people child still answers
  `Applied` for the key; the client says "unknown past the retention"
  (client.md, 4.7).
- **02.2 runs the flow with today's `forge::Adoption`** (its
  `provider` still in it), so it lands alone; **02.3** removes the id,
  adds the locate stage, removes the party's own `Ask::Adopt` path and
  moves every world's adoption onto the keyed request, which the old
  path's removal needs (the skeleton listed the worlds' move under
  02.4). Between 02.3 and 02.4, an adoption sent again after a restart is
  answered unknown; 02.4 keeps the outcome durably.
- **The engine world gets a forge stand and fixtures** for these stories
  (`tests/engine/domain/src/{forge_stand.rs, fixtures.rs}`, 02.2), on
  `direct::Driver`, since `forge_routes.rs`'s harness lives in that large
  test file and moving it is `later.md`'s.
- **The fake forge's repositories get the ids a scenario gives them**
  (`api::Setup.id`), and a locate is a call on a repository named in
  full, answered as any read is: a user who may not read it gets
  `Forbidden` (forge.md, 13 leaves Forgejo's own answer to check).

## Rules that bind it

- "**Large files.** ... An increment adds at most 40 lines to each ...;
  its new code goes in a new module beside it" (README, "Rules"). Here:
  `crates/temper-engine-domain-forge/src/domain.rs`,
  `crates/temper-engine-domain/src/route/forge_route.rs`,
  `crates/temper-engine-domain-forge-client/src/{api.rs, keep.rs,
  outbox.rs}`, `tests/engine/domain/tests/forge_routes.rs`,
  `testing/temper-fake-forge-domain/src/tests.rs`,
  `tests/engine/forge/tests/simulation.rs`,
  `tests/engine/forge/client/tests/simulation.rs`,
  `tests/engine/domain/src/{escalation.rs, walking.rs}`.
- "**A durable record's codec follows its record.** Once 04 has landed a
  family's codec, an increment that changes a `Stored` variant or a value
  it holds changes the schema, the generated code and the translation in
  the same increment" (README, "Rules"): 02.3 changes `forge::Role`'s
  neighbours not at all, but 02.4 adds `Stored::Application`.
- "no type beside an existing one for the same thing: change the type in
  place, and its users in the same increment" (code-organization.md,
  section 6): `forge::Adoption` changes in place in 02.3; no second
  adoption type.

## Increments

### 02.1 Locate, in the forge client and the fake forge

Scope: domain/forge.md, 4 and 17; forge.md, 3.1 and 12.
`api::Read::Locate { owner, name }` and `api::Answer::Located { repository
}` (the interface sketch). The client: `bounds::read` counts `owner` and
`name` in the call's bytes (`bounds.rs:44-75`); `bounds::answer` prices
`Located` at 0 (:186); `api::accepts` expects the shape `Located` for a
locate and refuses `Located { repository: 0 }` (`api.rs:410-560`, the
checks in a new module `crates/temper-engine-domain-forge-client/src/locate.rs`,
called from api.rs and bounds.rs); the exhaustive matches gain their arm.
A locate is addressed with `Repository { forge, repository: 0 }`; the
client does not read the repository of a `Read` (`domain.rs:165-180`), so
nothing else changes. The root's `read_name` (`forge_route.rs:280-298`)
gives a locate no resource name (`None`: no tool reads it);
`temper_engine_smith::answers::forge_answer` (`answers.rs:419`) renders
`Located` as `Repository N`. The fake forge: `api::Setup` gains `id: u32`
(nonzero, unique: `scenario::repository` asserts both), the store's
`Repository` keeps it, `api::Read::Locate` on a repository named in full
answers `api::Answer::Located { id }` for a user with read permission
(`domain.rs:510-531`'s permission arm, `reads.rs:38-90`). The forge
client's world translates both ways (`translate.rs`: `read_op` maps
`Locate` to the fake's `Locate`, `answer` maps `Located { id }` to
`Located { repository: id }`) and names a locate's call by `owner/name`
(`world.rs:430-433, 615-618`, where every other call names `org/repo`).
Needs: nothing beyond the session's start.
Files: `crates/temper-engine-domain-forge-client/src/{api.rs (2
variants; 12 matches gain an arm: 451-453, 479-481, 521-523, 550-552,
581-583, 628-630, 664-666, 703-705, 730-732, 757-759, 786-788, 818-820;
at most 40 lines), locate.rs (new), bounds.rs, keep.rs (:796-798,
:892-894), outbox.rs (:496-498), lib.rs, tests.rs (a `mod locate;` line,
the helper below), tests/locate.rs (new)}`;
`crates/temper-engine-domain-forge/src/domain.rs` (:3249-3267, one arm);
`crates/temper-engine-domain/src/route/forge_route.rs` (:298, one arm);
`crates/temper-engine-smith/src/answers.rs` (one arm);
`testing/temper-fake-forge-domain/src/{api.rs, scenario.rs, store.rs,
domain.rs, reads.rs, tests.rs (a `mod locate;` line), tests/locate.rs
(new)}`; the users of `api::Setup` (each gains `id`): 
`tests/fake-forge/tests/memory.rs:97`, `testing/temper-fake-forge-domain/src/tests.rs:146`,
`tests/worker/checkout/src/forge.rs:153`, `tests/system/src/forge.rs:218`,
`tests/engine/forge/src/world.rs:115`, `tests/engine/domain/tests/referee.rs:117`,
`tests/engine/domain/tests/forge_routes.rs:266, 2142`,
`tests/engine/forge/client/src/world.rs:109`, `tests/conformance/src/systems.rs:60`
(the repository `org/repo` gets id 2 everywhere, `forge_routes.rs`'s
`org/extra` id 3, as `forge_routes.rs:425-431` maps them today); the
matches of the client's and the fake's `Read` and `Answer` in the worlds:
`tests/engine/forge/client/src/{translate.rs, world.rs:725, referee.rs:57}`,
`tests/engine/forge/client/tests/{simulation.rs:543, 607, 661, 772,
memory.rs:183}`, `tests/fake-forge/tests/memory.rs:294-311`;
`tests/engine/forge/client/tests/locate.rs` (new). Shared hotspots:
`api.rs`, `forge_routes.rs`, the worlds' `Setup` literals.
Shape: as the interface sketch (`s02.rs`, modules `api` and `fake`).
Test shape: the client's unit tests in `src/tests/locate.rs`, on
`Harness` (`tests.rs:63-142`) and `terminal` (:143), with a helper
`Harness::sent(&mut self) -> (Sent, Repository)` beside `send` (whose
assert names `REPO`, :101): `a_locate_past_the_call_bound_is_refused_too_large`
(owner 200 bytes and name 57, past `LIMITS.op_bytes` 256),
`a_located_repository_zero_is_an_invalid_answer`. The fake's in
`testing/temper-fake-forge-domain/src/tests/locate.rs`:
`a_locate_answers_the_id_the_scenario_gave_the_repository`,
`a_locate_by_a_user_who_may_not_read_is_forbidden`. The client's world,
`tests/engine/forge/client/tests/locate.rs`:
`a_locate_answers_the_forges_id_for_a_repository_by_name`,
`a_locate_of_a_repository_the_forge_lacks_answers_missing`. Unit tests
under 20 ms, world tests under 0.5 s. Reference test, in full:

```rust
//! Locating a repository by owner and name (domain/forge.md, 4; forge.md, 12).
use temper_engine_domain_forge_client::api;
use temper_engine_forge_client_world::{Settings, World};

#[test]
fn a_locate_answers_the_forges_id_for_a_repository_by_name() {
    let mut world = World::new(Settings::calm(5));
    world.read(1, api::Read::Locate { owner: Box::from(&b"org"[..]), name: Box::from(&b"repo"[..]) });
    world.run_for(1);
    assert_eq!(world.read_result(1), Some(&Ok(api::Answer::Located { repository: 2 })));
    world.finish();
}
```

Proof: world, `cargo nextest run -p temper-engine-forge-client-world
-p temper-engine-domain-forge-client -p temper-fake-forge-domain -E
'binary(locate) | test(/locate::/)'`, run once: a locate by owner and name
comes back as the id the fake forge gave the repository, a missing one as
`Missing`, a zero id as an invalid answer, an oversized one refused
before any call (the six tests above); under 3 s.
Don't: cache a located id in the client; give a locate a resource name or
a tool; change what any other read's call names.
Size: at most 200 lines.
Removes: nothing.

### 02.2 The root's keyed application, and the outcome the forge keeps

Scope: jig's `client/domain/contract.md`, 3.3; jig's
`domain/connectors.md`, 5.5; domain/forge.md, 4 (the request's flow,
with today's adoption); client.md, 4.6 and 4.7 (the outcome).
The root: `PartyInput::Application { reply_to, sign_in, key, digest,
request: forge::Adoption }` (the sketch); `route.rs` dispatches it to
`route/applications.rs` (new), which mints `owner` (a `u64` counter in
`Domain`, from 1), keeps `(owner, request)` in one slot
(`Option<(Token, forge::Adoption)>`; it is empty at each step's start:
assert so), and steps `core::Event::Application { reply_to, sign_in,
key, project: request.project, connector: config.forge_connector,
digest, owner }`. `asks.rs:37-40`'s arms become:
`Ask::Application { request, token, owner, retention, .. }` takes the
slot's value (the owner must match) into `forge::Event::Application {
request, token, retention, adoption }`; `Ask::DropApplication { owner }`
clears the slot if it still holds `owner`; `Ask::ApplicationPartAnswered
{ request, outcome }` becomes `forge::Event::ApplicationAnswered`. The
forge's new requests (forge_route.rs's exhaustive `outputs`, three arms
calling `applications.rs`): `ApplicationPart { request, adoption }` →
`adopt_repository_ask(adoption, connector)` → `core::Event::ApplicationPart
{ request, ask }` (a `None` ask answers the forge
`ApplicationAnswered { outcome: Refused(Unknown) }`); `ApplicationDone {
request, token }` → `core::Event::ApplicationFinished { request, outcome:
Ok(token) }`; `ApplicationAgain { request, why }` → `ApplicationFinished
{ outcome: Err(why) }`. `held.rs:125`: `Held::Applied { to, token, .. }`
emits `Delivery::Application { to, outcome:
domain.forge.application_outcome(token).cloned() }`;
`Held::ApplicationDelivery` stays unreachable (the forge serves no
application subject: client.md, 8 refuses every watch below the root).
`Delivery::Application` leaves as `PartyRequest::Deliver`
(`boundary.rs:590-600`). The forge connector (`adoption.rs`, new):
`Event::Application`: one kept outcome past `until` dropped; a full
table → `ApplicationAgain { why: Busy }`; no writer for
`adoption.provider.forge` (`d.writers`) → keep `ForgeRefused(NoSuchForge)`
and `ApplicationDone`; else pending, and `ApplicationPart { request,
adoption }`. While pending, `Request::Adopted { reply_to: request, .. }`
from the existing reads also records the outcome-to-be: `Err(error)` →
`reason(error)`, `Ok(adopted)` → its row (domain.rs gains these two
lines in `emit`'s callers, or one `adoption::adopted` call where it
emits `Adopted`). `Event::ApplicationAnswered { request, outcome }`: a
recorded reason → `ForgeRefused(reason)`; else `Refused(why)` →
`CoreRefused(why)`; else `Adopted { .. }` → `Adopted(AdoptedRepository)`
from the recorded row; `Again(why)` with no reason →
`ApplicationAgain { why }` and nothing kept; a kept outcome →
`ApplicationDone { request, token }`. `Domain::application_outcome(token)`.
`forge::Limits.applications` (worst case: the kept table and each
outcome's owner, name and protection contexts at `name_bytes`).
Needs: nothing beyond 02.1.
Files: `crates/temper-engine-domain/src/{boundary.rs (`PartyInput`,
`delivery_output` :590-600), decision.rs (`Delivery::Application`; its
arm at :344-353), domain.rs (the slot, the counter; :666-674's arm),
route.rs (dispatch), route/applications.rs (new), route/asks.rs
(:38-40), route/held.rs (:125), route/forge_route.rs (three arms in
`outputs`, at most 40 lines), limits.rs (worst case)}`;
`crates/temper-engine-domain-forge/{Cargo.toml (jig-domain-people),
src/{adoption.rs (new), boundary.rs (the sketch's types and variants),
domain.rs (dispatch arms, the pending and kept tables, the recording at
`Adopted`: at most 40 lines), limits.rs (`applications`, worst case),
lib.rs (`mod adoption`, exports), tests.rs (`LIMITS.applications` at
:32-72; the three exhaustive matches of `Request` at :257-283 and two
more gain three arms each; `mod adoption;`), tests/adoption.rs (new)}}`;
`tests/engine/forge/src/world.rs:47` (`applications` in its limits);
the exhaustive matches of `forge::Request`:
`tests/engine/forge/tests/simulation.rs:787-813` (two matches, three arms
each); of `Delivery`: `tests/system/src/composed.rs:595`,
`tests/engine/domain/src/{escalation.rs:781, walking.rs:737}`; the
engine world: `tests/engine/domain/src/{lib.rs, direct.rs, fixtures.rs
(new), forge_stand.rs (new)}`, `tests/engine/domain/tests/applications.rs`
(new). Shared hotspots: `forge_route.rs`, the forge's `domain.rs` and
`boundary.rs`, `decision.rs`.
Shape: as the interface sketch (`s02.rs`, modules `forge` and `engine`),
but `forge::Adoption` keeps its `provider` field (02.3 removes it),
`Request::ApplicationPart` is `{ request, adoption }`, and
`forge::Event::Adopt { reply_to, adoption }` and `adopt_repository_ask(adoption,
connector)` are today's. The engine world (ordinary Rust):
`ForgeStand::new() -> ForgeStand` in `forge_stand.rs`: the fake forge with
`org/repo` (id 2; users 1 admin, 2 write, 7 read, as forge_routes.rs:289-291)
and `org/private` (id 3; user 1 no permission), its config
`forge_world::fake_config()` with zero latency; `ForgeStand::call(&mut
self, call: Token, repository: api::Repository, op: api::Op, limits:
&client::Limits)` names the repository by its id (`{1, 2}` →
`org/repo`, `{1, 3}` → `org/private`); `ForgeStand::replies(&mut self)
-> Vec<engine::Event>` the fake's replies as `Event::Forge(Client(Answered))`;
`pub calls: u32`. `Driver` gains `pub forge: Option<ForgeStand>` and
`Driver::forge() -> Driver` (the limits `fixtures::forge_limits()`:
`walking::limits()` with `people.people` 4, `people.holdings` 4 and the
journal's writes grown as forge_routes.rs:88-93 grows them;
`walking::config(71)`); `collect`'s `Request::Forge` arm calls the stand
when there is one (and panics as today otherwise); `advance` sends its
replies. `fixtures::adoption(forge: u16) -> forge::Adoption`
(project 1, home, `forge.example`, `org`/`repo`, `temper/`, owned,
`main`, CI, no checks; `provider { forge, repository: 2 }` until 02.3).
Test shape: the engine world's `tests/engine/domain/tests/applications.rs`
(new; on `Driver::forge`, its sign-in `Driver::sign_in` at direct.rs:194
and `Driver::session` at :272), and the forge's unit tests in
`crates/temper-engine-domain-forge/src/tests/adoption.rs` (new; on
`domain()`, `outputs`, `ready` at tests.rs:76-105). Tests:
`an_adoption_naming_a_forge_not_configured_is_refused_as_no_such_forge`,
`an_adoption_on_a_configured_forge_is_adopted_with_what_temper_may_do`
(its `AdoptedRepository`: `{1, 2}`, `org`, `repo`, owned, home, every
kind true, protection `Absent`, as the fake's admin and no rule give),
`an_adoption_sent_again_under_its_key_is_answered_its_kept_outcome`
(the same outcome twice, the stand's `calls` unchanged by the second),
`an_adoption_the_forge_user_may_not_read_is_refused_as_no_access`
(`fixtures::adoption` with `org/private`, id 3); in adoption.rs:
`a_core_refusal_is_kept_as_jigs_refusal`,
`a_full_outcome_table_answers_the_application_again_busy`, with the
forge crate's fixtures module `src/tests/fixtures.rs` (new: `adoption()`,
tests.rs:241-253's values). World tests
under 0.5 s, unit tests under 20 ms. Reference test, in full:

```rust
//! The forge's keyed adoption through the root (domain/forge.md, 4; jig's
//! client/domain/contract.md, 3.3).
use skein_lib::{ReplyTo, Token};
use temper_engine_domain as engine;
use temper_engine_domain::Delivery;
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::direct::Driver;
use temper_engine_domain_world::fixtures;

#[test]
fn an_adoption_naming_a_forge_not_configured_is_refused_as_no_such_forge() {
    let mut driver = Driver::forge();
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver.send(engine::Event::Party(engine::PartyInput::Application {
        reply_to: ReplyTo::new(Token::new(95)),
        sign_in: driver.session(),
        key: [95; 16],
        digest: [5; 32],
        request: fixtures::adoption(2),
    }));
    driver.settle();
    let outcomes: Vec<&Option<forge::ApplicationOutcome>> = driver
        .delivered
        .iter()
        .filter_map(|delivery| if let Delivery::Application { outcome, .. } = delivery { Some(outcome) } else { None })
        .collect();
    let refused = forge::ApplicationOutcome::ForgeRefused(forge::AdoptionRefusal::NoSuchForge);
    assert_eq!(outcomes, [&Some(refused)]);
    assert_eq!(driver.forge.as_ref().expect("a forge stand").calls, 0);
}
```

Proof: world, `cargo nextest run -p temper-engine-domain-world -p
temper-engine-domain-forge -E 'binary(applications) |
test(/tests::adoption::/)'`, run once: a party's keyed adoption is
admitted by jig's people child, decided by the forge connector, and
answered once durable with the connector's outcome: adopted with what
temper may do, refused as no such forge or no access, jig's refusal
kept, again when full, and the same key answered from the kept outcome
without the forge (the six tests above); under 3 s.
Don't: send a party's request to the forge before the core admitted it;
keep the value past the step; count the forge's reads twice; save the
outcome yet (02.4); remove the party's own `Ask::Adopt` path yet (02.3).
Size: at most 400 lines.
Removes: `asks.rs:38-40`'s and `held.rs:125`'s unreachable arms for
`Ask::Application`, `Ask::ApplicationPartAnswered` and `Held::Applied`.

### 02.3 The adoption by owner and name; the party's own `Ask::Adopt` goes

Scope: domain/forge.md, 4 (locate, then jig's adoption as the part);
client.md, 4.4 and 8. `forge::Adoption` loses `provider` and gains
`forge: u16` (the sketch). The connector's application: after the
no-such-forge check, the field bounds of `adopt` (domain.rs:1059-1068:
empty or past `name_bytes` → `TooLarge`) refuse before any call; then a
locate (`client::Event::Read { owner: request, repository: { forge,
repository: 0 }, read: Read::Locate { owner, name } }`, a stage
`Locate` first in the pending adoption); `Located { repository }`
records the id and emits `ApplicationPart { request, adoption,
repository }`; an error keeps `ForgeRefused(reason(error))`. The root:
`adopt_repository_ask(adoption, repository, connector)` (the located id
in the resource's path, `forge_route.rs:39-73`); `core::Ask::Adopt {
request, .. }` (asks.rs:69-90) for a request the forge is applying
(`domain.forge.applying(request)` is the located repository) becomes
`forge::Event::Adopt { request }`, `adoption_restore` keyed by `request`
holding `domain.forge.repository(located).cloned()`; for any other
request it is decided `people::Outcome::Refused(people::Refusal::NotOffered)`
at once. `parse_adoption` (forge_route.rs:101-145) and `adoption_flag`
(:74-80) go. The forge's `Event::Adopt { request }` runs `adopt`'s
checks against the pending application's adoption and located id, then
the reads as today (Permission → Branches → Settings → Protection →
Collaborators); the unconfigured-forge `Forbidden` at domain.rs:1102
goes (the application refused it as no such forge before). The adoption
section (`AdoptionStage`, `PendingAdoption` at domain.rs:16-35; `adopt`,
`owns_branch`, `adoption_read`, `adoption_collaborators`,
`finish_adoption` at :1058-1322) moves to `adoption.rs`. The worlds
adopt through `PartyInput::Application` with `fixtures::adoption`-like
values and read `Delivery::Application`: `forge_routes.rs`'s
`adopt_with_checks` (:596-635) and its `adopted` (:30, :475-480);
`tests/system/src/composed.rs:727-746` and `tests/system/src/forge.rs:470-500`;
`tests/conformance/src/lib.rs:530-560`; `tests/engine/forge/src/world.rs:175-200`
(the connector world plays the core: it sends `Event::Application`,
answers `ApplicationPart` with `Event::Adopt { request }`, and `Adopted`
with `Event::ApplicationAnswered { outcome: Adopted { project } }`); and
each world's forge stand names a locate's call by `owner/name`
(forge_routes.rs:425-431, composed.rs's `forge()` at :548,
tests/system/src/forge.rs:310, `forge_stand.rs`).
`forge_routes.rs`'s adoption tests at :2003-2087 move to
`applications.rs` on the keyed request; :2088-2134 (`an_adoption_sent_twice_across_a_restart_is_made_once`)
goes, and 02.4 writes its keyed counterpart.
Needs: 02.1's locate, 02.2's application, and 02.4's outcome kept
durably: 02.4 lands before 02.3 (decisions.md, ruling 119).
Files: `crates/temper-engine-domain-forge/src/{boundary.rs, adoption.rs,
domain.rs (the moved section out; at most 40 lines added), tests.rs (the
`Adoption` literals at :241-253, :385-397; `Event::Adopt` at :254, :397),
tests/adoption.rs}`; `crates/temper-engine-domain/src/route/{forge_route.rs
(at most 40 lines added), asks.rs, applications.rs}`,
`crates/temper-engine-domain/src/lib.rs` (the export of
`adopt_repository_ask` stays); the worlds above, and
`tests/engine/domain/src/{fixtures.rs, forge_stand.rs}`,
`tests/engine/domain/tests/{forge_routes.rs (at most 40 lines added),
applications.rs}`, `tests/engine/forge/tests/simulation.rs` (its
`top::Request::Adopted` checks at :16-68 read the same requests; at most
40 lines added). Shared hotspots: `forge_route.rs`, the forge's
`domain.rs`, `forge_routes.rs`, `composed.rs` (789 lines: the adoption
helper's lines are replaced, not added to; it stays at most 800).
Shape: as the interface sketch (`s02.rs`): `forge::Adoption`,
`Event::Adopt { request }`, `Request::ApplicationPart { request,
adoption, repository }`, `Domain::applying`, `adopt_repository_ask`.
Test shape: `tests/engine/domain/tests/applications.rs` grows:
`an_adoption_of_a_repository_the_forge_lacks_is_refused_as_not_found`
(`fixtures::adoption` naming `org/none`),
`a_partys_own_adopt_ask_is_refused_as_not_offered` (a
`PartyInput::Ask { ask: people::Ask::Adopt { .. } }`: `Delivery::WebReply`
with `Outcome::Refused(Refusal::NotOffered)`, the stand's `calls` 0),
`a_repository_adopted_seeds_its_collaborators_into_roles` and
`policy_maps_connector_permissions_to_seeded_roles_after_restart` (moved
from forge_routes.rs:2003-2087, their assertions unchanged); and the
forge's unit test `an_adoption_locates_its_repository_before_any_other_read`
(the first call is `Op::Read(Read::Locate { owner: b"org", name: b"repo" })`
on `{ forge: 1, repository: 0 }`). Under 0.5 s each. Reference test, in
full (adoption.rs, on tests.rs's `domain`, `outputs` and `ready`):

```rust
#[test]
fn an_adoption_locates_its_repository_before_any_other_read() {
    let mut d = domain();
    let adoption = fixtures::adoption();
    let application = Event::Application {
        request: Token::new(4),
        token: Token::new(9),
        retention: Duration::from_secs(60),
        adoption,
    };
    assert!(outputs(&mut d, application).is_empty());
    let sent = ready(&mut d);
    let [Request::Call { repository, op, .. }] = sent.as_ref() else { panic!("one locate call") };
    assert_eq!(*repository, client::api::Repository { forge: 1, repository: 0 });
    let locate = client::api::Read::Locate { owner: Box::from(&b"org"[..]), name: Box::from(&b"repo"[..]) };
    assert_eq!(*op, client::api::Op::Read(locate));
}
```

(`fixtures::adoption()` is the forge crate's test fixture in
`src/tests/fixtures.rs`, made by 02.2 from tests.rs:241-253's values,
`forge: 1` and no id since this increment.)
Proof: world, `cargo nextest run -p temper-engine-domain-world -p
temper-engine-domain-forge -p temper-engine-forge-world -p
temper-system-world -p temper-conformance-world -E 'binary(applications)
| test(/tests::adoption::/) | binary(simulation) | binary(composed)'`,
run once: an adoption names its repository by owner and name and is
located before any other read; a repository the forge lacks is not found;
a party's own `Adopt` is not offered; the collaborators still seed roles;
the forge connector's, the system's and the conformance worlds' adoptions
pass through the keyed request; under 30 s.
Don't: ask the forge for the repository's id anywhere but the locate;
keep a second adoption type; leave `parse_adoption` or a direct
`Event::Adopt { reply_to, adoption }`.
Size: at most 300 lines, the moved adoption section (about 290 lines of
domain.rs) not counted.
Removes: `parse_adoption`, `adoption_flag`, `forge::Adoption.provider`,
the forge's unconfigured-forge `Forbidden`, and the party's own
`Ask::Adopt` reaching the forge.

### 02.4 The outcome kept durably

Scope: store/README.md, 2 (byte 15); domain/forge.md, 4 ("keeps it
durably"). `Stored::Application { token, until, outcome }` and
`Key::Application(u64)`, the last variants of both enums. The forge saves
the row in the decision that keeps the outcome (`Request::Save { record:
Stored::Application { .. }, release: None, read_afresh: None }`) and
erases it (`Request::Erase { key: Key::Application(token) }`) when it
drops the outcome past `until`; `restore` (domain.rs:3580-3640) puts it
back in the kept table. `stored_key` (lib.rs:185-207) and `stored_bytes`
(lib.rs:209-406) gain their arms. The root's `Write::Save(Record::Forge
{ row })` and `Write::Erase(Key::Forge(key))` route it as every forge
row (forge_route.rs:2009-2025), and `Range::Forge` holds it. If 04.1
has landed, 02.4 adds `Key::Application`'s encoder and decoder in
`crates/temper-protocol-store/src/keys.rs`; if 04.4 has landed, also the
family's schema, generated code and records (README, "A durable
record's codec follows its record"; 04.4 owns them otherwise).
Needs: 02.2's kept outcomes.
Files: `crates/temper-engine-domain-forge/src/{boundary.rs, adoption.rs,
domain.rs (the restore arm: at most 40 lines), lib.rs, tests/adoption.rs}`;
the users of `Stored` and `Key`: `tests/engine/forge/src/world.rs:536`
(its key arm); `tests/engine/domain/tests/applications.rs`; if 04.1 has
landed, `crates/temper-protocol-store/src/{keys.rs, tests/keys.rs}`; if
04.4 has landed, also `crates/temper-protocol-store/{schema/store-v1.schema,
src/{records.rs, generated/v1.rs, tests/records.rs}, golden/v1/}` and
`tests/codecs/tests/drift.rs`.
Shape: as the interface sketch (`s02.rs`, `forge::Stored`, `forge::Key`).
Test shape: `applications.rs`:
`an_adoption_sent_again_after_a_restart_is_answered_its_kept_outcome`
(adopt, then a new `Driver` over a `Store` with the old rows, as
root_routes.rs:4150-4156 builds one, signed in again, the same key and
request: the same `Delivery::Application` outcome, the stand's `calls`
unchanged); adoption.rs: `a_kept_outcome_is_saved_with_the_end_of_its_retention`
(the `Save` row's `until` is the step's wall plus the retention),
`an_outcome_past_its_retention_is_erased_at_the_next_application`.
Under 0.5 s each. Reference test, in full (adoption.rs):

```rust
#[test]
fn a_kept_outcome_is_saved_with_the_end_of_its_retention() {
    let mut d = domain();
    let mut adoption = fixtures::adoption();
    adoption.forge = 9;
    let application = Event::Application {
        request: Token::new(4),
        token: Token::new(9),
        retention: Duration::from_secs(60),
        adoption,
    };
    let requests = outputs(&mut d, application);
    let outcome = ApplicationOutcome::ForgeRefused(AdoptionRefusal::NoSuchForge);
    let row = Stored::Application { token: 9, until: Wall::from_nanos(160_000_000_000), outcome };
    assert!(requests.contains(&Request::Save { record: row, release: None, read_afresh: None }));
    assert!(requests.contains(&Request::ApplicationDone { request: Token::new(4), token: Token::new(9) }));
}
```

(tests.rs's `env()` at :73-75 has `wall` at 100 s, so the row's `until` is
160 s.)
Proof: world, `cargo nextest run -p temper-engine-domain-world -p
temper-engine-domain-forge -E 'binary(applications) |
test(/tests::adoption::/)'`, run once: an adoption sent again with its
key after the engine restarted is answered the outcome it had, with no
forge call; a kept outcome is saved with its retention's end and erased
past it; under 3 s.
Don't: move `STORE_VERSION` (README); keep an outcome past `until` once a
new application comes; restore anything but into the kept table.
Size: at most 380 lines of production code, the family's codec included
(about 150 without it: the forge's family; about 30 for the key's
encoder and decoder; about 200 for the schema's records and their
translations). Past 600, it splits before it starts: 02.4a the forge's
family, 02.4b its codec. The landing allowance for its family part is
the README's.
Removes: nothing.

## Interface sketches

The same text as `scratch/plan-sketches/src/s02.rs`, which compiles
(its first lines, `//! Sketches of session 02.` and an
`#![expect(unused_variables, ..)]` for the sketch's bodies, aside). Who
owns what: the client's two variants are 02.1's; the forge's application
vocabulary, the outcome and its reasons are 02.2's, the adoption without
its id and `Adopt { request }` 02.3's, the family 02.4's; the root's
input and delivery 02.2's. What ends what: a `PartyInput::Application`
ends in one reply (jig's at admission, or one `Delivery::Application`);
an `Event::Application` in one `ApplicationDone` or `ApplicationAgain`;
an `Event::Adopt { request }` in one `Request::Adopted`; a kept outcome
at its retention's end.

```rust
// temper_engine_domain_forge_client::api (a new module beside api.rs, its
// variants added to `api::Read` and `api::Answer`). Owner: 02.1. Users: 06.1
// (the documents), 13.3 (the component's plan), 14.2 (the fake's routes).
pub mod api {
    use alloc::boxed::Box;

    /// The read 02.1 adds to `api::Read` (forge.md, 3.2 and 12): the forge's
    /// id for a repository from its owner and name, addressed with the
    /// forge's number and repository 0; sent by the forge connector's
    /// adoption, ended by one `Answer::Located` or one `api::Error`.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub enum Read {
        Locate { owner: Box<[u8]>, name: Box<[u8]> },
    }

    /// The answer 02.1 adds to `api::Answer`: the repository's id on the
    /// forge, never 0 (a 0 is refused as `Error::InvalidAnswer`).
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub enum Answer {
        Located { repository: u32 },
    }
}

// temper_fake_forge_domain::api. Owner: 02.1. Users: the worlds that answer
// `Request::Forge` with the fake forge, 14.2.
pub mod fake {
    /// The read 02.1 adds to the fake's `api::Read`: a call on a repository
    /// named in full, answered `Answer::Located` for a user who may read it,
    /// `Error::Missing(What::Repository)` otherwise, as any read is.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub enum Read {
        Locate,
    }

    /// The answer 02.1 adds to the fake's `api::Answer`: the id the
    /// scenario's `Setup` gave the repository.
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub enum Answer {
        Located { id: u32 },
    }

    // `api::Setup` gains `pub id: u32`: Forgejo's id for the repository,
    // nonzero and unique in the forge; `scenario::repository` asserts both.
}

// temper_engine_domain_forge (boundary.rs, and the new adoption.rs beside
// domain.rs). Owners: 02.2 (the application's vocabulary, the outcome, its
// reasons), 02.3 (`Adoption` without the repository's id, `Adopt` by
// request), 02.4 (`Stored::Application`, `Key::Application`). Users: 03.2
// (field for field), 04.4 (the `Application` family), 08.5, 15.1.
pub mod forge {
    use alloc::boxed::Box;
    use jig_domain_people as people;
    use skein_lib::{Duration, Token, Wall};
    use temper_engine_domain_forge::{Kinds, Protection, Role};
    use temper_engine_domain_forge_client::api;

    /// A party's request to adopt a repository (client.md, 4.4; domain/forge.md,
    /// 4), as 02.3 leaves it: the form's fields, and the forge's host, which
    /// the engine's routing fills from the forge's configured `git` base
    /// (empty for a forge not configured). Sent in `PartyInput::Application`;
    /// ended by one `ApplicationOutcome` kept under the people child's token.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Adoption {
        pub project: u32,
        /// Make it the project's home, if the project has none yet.
        pub home: bool,
        /// The forge's number, as configured.
        pub forge: u16,
        pub host: Box<[u8]>,
        pub owner: Box<[u8]>,
        pub name: Box<[u8]>,
        /// temper's branch prefix there, `temper/` by default.
        pub prefix: Box<[u8]>,
        pub role: Role,
        /// The branch changes land on.
        pub landing: Box<[u8]>,
        /// Whether it runs CI; without, `checks` names at least one gate.
        pub ci: bool,
        pub checks: Box<[u32]>,
    }

    /// The adoption's outcome (client.md, 4.6), kept by the connector under
    /// the people child's token until its retention ends, and given again
    /// to the same key sent again; delivered as `Delivery::Application`.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum ApplicationOutcome {
        /// The core committed the adoption; the repository as recorded.
        Adopted(AdoptedRepository),
        /// The core refused the adoption the root made on the party's
        /// behalf, with jig's facts.
        CoreRefused(people::Refusal),
        /// The forge connector refused it, before or after the core's part.
        ForgeRefused(AdoptionRefusal),
    }

    /// The repository as the forge connector recorded it at adoption.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct AdoptedRepository {
        /// Its forge's number and the forge's id for it.
        pub repository: api::Repository,
        pub owner: Box<[u8]>,
        pub name: Box<[u8]>,
        pub role: Role,
        /// Whether it is the project's home.
        pub home: bool,
        /// What temper may do there.
        pub kinds: Kinds,
        /// Branch protection: unknown, absent, or the rule read.
        pub protection: Protection,
    }

    /// Why the forge connector refused an adoption (client.md, 4.7 and 8).
    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum AdoptionRefusal {
        /// The deployment configures no forge of that number.
        NoSuchForge,
        /// The forge has no such repository that temper's user can see.
        NotFound,
        /// temper's forge user may not read it.
        NoAccess,
        /// Another project writes it, the project has a home already, or
        /// branches under the prefix are not this deployment's.
        Refused,
        /// The same repository is being adopted already.
        InFlight,
        /// A field past the forge's bounds, or its repositories full.
        TooLarge,
        /// The forge did not answer; nothing was adopted.
        Unreachable,
        /// The forge answered what temper did not expect.
        InvalidAnswer,
    }

    /// A forge call's error as the adoption's reason (client.md, 8).
    #[must_use]
    pub const fn reason(error: api::Error) -> AdoptionRefusal {
        match error {
            api::Error::Missing => AdoptionRefusal::NotFound,
            api::Error::Forbidden => AdoptionRefusal::NoAccess,
            api::Error::Refused => AdoptionRefusal::Refused,
            api::Error::Busy => AdoptionRefusal::InFlight,
            api::Error::TooLarge | api::Error::Full => AdoptionRefusal::TooLarge,
            api::Error::Unavailable | api::Error::Timeout | api::Error::RateLimited { .. } => {
                AdoptionRefusal::Unreachable
            }
            api::Error::MissingJob
            | api::Error::Empty
            | api::Error::Exists
            | api::Error::NothingToMerge
            | api::Error::Closed
            | api::Error::Stale
            | api::Error::Conflict
            | api::Error::Protected
            | api::Error::InvalidAnswer => AdoptionRefusal::InvalidAnswer,
        }
    }

    /// The events 02.2 and 02.3 give `forge::Event` (the root sends each).
    #[derive(PartialEq, Eq, Debug)]
    pub enum Event {
        /// The core admitted the party's application `request` (jig's
        /// `core::Ask::Application`): decide it, keep its outcome under
        /// `token` until `retention` has passed. Ends in one
        /// `Request::ApplicationDone` or `Request::ApplicationAgain`.
        Application { request: Token, token: Token, retention: Duration, adoption: Adoption },
        /// The core answered the application's part (jig's
        /// `core::Ask::ApplicationPartAnswered`).
        ApplicationAnswered { request: Token, outcome: people::Outcome },
        /// 02.3: the core routed the part's adoption back (jig's
        /// `core::Ask::Adopt` for an application's request): read what
        /// temper may do there; ends in one `Request::Adopted`. It replaces
        /// `Adopt { reply_to, adoption }`.
        Adopt { request: Token },
    }

    /// The requests 02.2 and 02.3 give `forge::Request` (the root takes each).
    #[derive(PartialEq, Eq, Debug)]
    pub enum Request {
        /// Make jig's adoption as the application's part, on the party's
        /// behalf (`adopt_repository_ask`): 02.3's shape, with the located
        /// repository (02.2's carries `adoption` alone, its id inside).
        ApplicationPart { request: Token, adoption: Adoption, repository: api::Repository },
        /// The outcome is kept under `token`: the core finishes the request.
        ApplicationDone { request: Token, token: Token },
        /// Nothing was decided or kept: the party sends it again.
        ApplicationAgain { request: Token, why: people::Again },
    }

    /// 02.4: the family the connector keeps an outcome in (store/README.md,
    /// 2, byte 15), saved in the decision that kept it, erased once past
    /// `until`, restored at the forge's restart step.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum Stored {
        /// The people child's token, the end of its retention, the outcome.
        Application { token: u64, until: Wall, outcome: ApplicationOutcome },
    }

    /// 02.4: its key, the last variant of `forge::Key`.
    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
    pub enum Key {
        Application(u64),
    }

    // `forge::Limits` gains `pub applications: u32` (02.2): outcomes kept at
    // once; with `adoptions` pending, a full table answers `ApplicationAgain`.
    //
    // impl temper_engine_domain_forge::Domain {
    //     /// The outcome kept under `token`, while its retention lasts (02.2).
    //     pub fn application_outcome(&self, token: Token) -> Option<&ApplicationOutcome>;
    //     /// The repository the pending application `request` located (02.3).
    //     pub fn applying(&self, request: Token) -> Option<api::Repository>;
    // }
}

// temper_engine_domain (boundary.rs, decision.rs, route/applications.rs).
// Owner: 02.2. Users: 08.5 (the clients' side hands the decoded adoption in
// and encodes the delivered outcome), 11.3's joint front, 15.1.
pub mod engine {
    use super::forge;
    use jig_domain_people as people;
    use skein_lib::ReplyTo;
    use temper_engine_domain_forge_client::api;

    /// The variant 02.2 adds to `PartyInput`: a party's keyed adoption, as
    /// the clients' routing decoded it (client.md, 8). Admitted by jig's
    /// people child by key; ended by one reply on `reply_to`: jig's refusal
    /// or answer at admission, or `Delivery::Application` once durable.
    #[derive(Debug)]
    pub enum PartyInput {
        Application {
            reply_to: ReplyTo,
            /// The party's live sign-in.
            sign_in: u64,
            key: [u8; 16],
            /// The digest of the request's encoding, which tells apart a key
            /// sent again with another request.
            digest: [u8; 32],
            request: forge::Adoption,
        },
    }

    /// The variant 02.2 adds to `Delivery`: an application's outcome, held
    /// until the decision that answered it is durable, then the request's
    /// terminal; none when the outcome's retention has ended ("unknown past
    /// the retention", client.md, 4.7).
    #[derive(PartialEq, Eq, Debug)]
    pub enum Delivery {
        Application { to: ReplyTo, outcome: Option<forge::ApplicationOutcome> },
    }

    /// jig's adoption on the party's behalf, as the application's part
    /// (`forge_route.rs`, line 39): 02.3 adds `repository`, the located id, which
    /// becomes the resource's path. None for a role the form never offers.
    #[must_use]
    pub fn adopt_repository_ask(
        adoption: forge::Adoption,
        repository: api::Repository,
        connector: u16,
    ) -> Option<people::Ask> {
        unreachable!("sketch")
    }
}
```

The engine world's stand (ordinary Rust, not compiled), in
`tests/engine/domain/src/forge_stand.rs` and `direct.rs`:

```rust
/// The fake forge behind the direct driver: `org/repo` (id 2) and
/// `org/private` (id 3), answering the root's forge calls at once.
pub struct ForgeStand { /* the fake domain, its env and queue, the calls in flight */ pub calls: u32 }
impl ForgeStand {
    pub fn new() -> ForgeStand;
    /// One root call: by the repository's id, or, for a locate (02.3), by
    /// its owner and name.
    pub fn call(&mut self, call: Token, repository: api::Repository, op: api::Op, limits: &client::Limits);
    /// The fake's replies so far, as the root's forge events.
    pub fn replies(&mut self) -> Vec<engine::Event>;
}
impl Driver {
    /// A driver whose forge calls the stand answers (limits `fixtures::forge_limits()`).
    pub fn forge() -> Driver;
}
```

## Questions to raise when reached

- **What Forgejo answers a locate for a repository temper's user cannot
  read** (forge.md, 13: "it decides whether a locate reports such a
  repository not found or no access"). The connector maps whatever the
  forge client says (`Missing` → not found, `Forbidden` → no access).
  Default: as the error comes; 05.3's facts settle it.
- **A role the form never offers** (`forge::Role::Adopted`,
  `findings/temper-engine.md`, Finding 8): `adopt_repository_ask` gives
  none for it. Default: the root answers the part
  `Refused(Unknown)`, kept as `CoreRefused(Unknown)`; `later.md` keeps
  the variant.

## Done when

- The four increments' tests pass: `cargo nextest run -p
  temper-engine-domain-world -p temper-engine-domain-forge -p
  temper-engine-domain-forge-client -p temper-engine-forge-client-world
  -p temper-fake-forge-domain -E 'binary(applications) | binary(locate) |
  test(/adoption::|locate::/)'` passes.
- The old path is gone: `git grep -n -e parse_adoption -e
  'Event::Adopt { reply_to' -- crates tests testing` prints nothing.
- The large files grew within their bound: `for c in $(git log
  --format=%h --grep='^Step: temper-02\.' origin/main); do git show
  --numstat --format= $c -- crates/temper-engine-domain-forge/src/domain.rs
  crates/temper-engine-domain/src/route/forge_route.rs
  crates/temper-engine-domain-forge-client/src/api.rs
  crates/temper-engine-domain-forge-client/src/keep.rs
  crates/temper-engine-domain-forge-client/src/outbox.rs
  tests/engine/domain/tests/forge_routes.rs
  testing/temper-fake-forge-domain/src/tests.rs
  tests/engine/forge/tests/simulation.rs
  tests/engine/forge/client/tests/simulation.rs
  tests/engine/domain/src/escalation.rs tests/engine/domain/src/walking.rs;
  done | awk '$1 > 40'` prints nothing.
- The gate passed on this session's last landing: `git -C ~/src/rust/temper
  log --format='%(trailers:key=Gate-Result,valueonly)'
  --grep='^Step: temper-02\.' origin/main | head -1` names
  `scripts/check.sh passed`.
