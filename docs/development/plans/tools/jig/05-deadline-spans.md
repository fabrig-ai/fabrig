# Session 05: deadlines given as spans

A request carries a template's deadline span unchanged; authority makes it a wall at the decision that makes the task.
A pending proposal retains the span, including across restart. Five increments, each landing alone, build
**jig's deadline spans, landed**. Needs at start: **jig on skein's and smith's main**. 05.2 also needs
**jig's read of one turn, landed**, for its v9 wire; 05.5 needs **jig's send under a named key, landed**.

## Read first

- **Design at jig 6ad8bb40:** `docs/design/client/domain/contract.md` 3.1, 3.2, 3.6, 5.3, 5.4 and 8;
  `client/domain/client.md` 11 and 19; `domain/authority.md` 7, 8.1 and 9; `domain/tasks.md` 4 and 9;
  `domain/engine.md` 4; `protocol/configuration.md` 3.1; `protocol/client.md` 3; `protocol/README.md` 7;
  `protocol/store.md` 5.1 and 5.3. Read the design-only commits `git show 62bcf71e` and `git show 6ad8bb40`.
- **Plan:** `jig/01-repin.md`, `jig/02-named-send.md`, `jig/03-turn-read.md`; `findings/jig-section-13.md`;
  the designer's round-2 answers 27 and 34: engine-owned spans and their finite-bound judgments.
  The first repin is skein b4c6241 and smith a277e922; this session moves no dependency pin.
- **Source:** `crates/jig-client/src/shapes.rs` (`chat`, `set_goal`, `fill`, `fall`);
  `jig-client-domain/src/contract/{operations,policy_edit,me}.rs`; `jig-domain-people/src/batch.rs`;
  `jig-domain-core/src/routing/{create,propose,party_decisions}.rs`, `src/goals.rs` (`goal_accept_allowed`),
  `src/{proposals,acts/objects/decisions,documents/offers}.rs`; `jig-domain-tasks/src/{boundary,person_proposals}.rs`;
  both ends' `translate/operations.rs`; `jig-protocol-store/src/codec/{people/batch,tasks/phase}.rs`.
  Paths in this list after the first are also below `crates/`.
- **Rules:** jig's `AGENTS.md`, `docs/development/workflow.md` 2; skein's foundation `development.md` 5.1 to 5.7;
  `programming-model.md` 3, 6.3 and 10.3; `testing-strategy.md` 2.1, 2.2, 6, 8 and 10.
  The graph query for `home-free-src-rust-jig` returned not indexed; the survey used primary-checkout source instead.

## What the design asks

- **contract.md 3.1 and 3.2:** the same name and ask get the first answer within retention. Build the ask from inputs
  and `me`, never a clock. A changed template or bound can change that ask and therefore produce a key conflict.
- **authority.md 7, 8.1; tasks.md 4, 9; engine.md 4:** every span/wall check uses that decision's wall plus the span:
  admission, where a proposal waits and each re-decision, acceptance and document offers. Acceptance beyond the accepter's
  authority is refused and the holder passes it up; past the ceiling it is refused. The made task keeps the checked wall.
- **contract.md 3.2 and 3.6; configuration.md 3.1:** Create and a proposed batch carry spans. A proposal does not spend
  its span while pending. Templates still grant nothing; the engine admits the expanded ordinary request.
- **contract.md 5.3; client.md 19:** `chat`, `set_goal` and `fill` read no clock. A Span is within both draft bounds,
  like AsAllowed; Given stays a wall and None stays none. Other fields still choose the four outcomes. The engine answers
  a Create beyond authority with whether it could be proposed; the application or person must propose explicitly.
- **protocol/client.md 3:** spans cross as durations; no monotonic time crosses. Documents of made tasks keep walls.
- **protocol/README.md 7; contract.md 8; store.md 5:** one served version and one store version per family;
  mismatches are refused, with no migration or parallel reader.

## Choices this plan makes

- **Wire v9 changes in place.** The protocol rules require a version move for changed plumbing and one pre-release
  served version; they do not promise compatibility between increments of this unreleased plan. development.md 5.1
  supplies the once-per-format-per-plan rule. Jig 03 owns v8 to v9. 05.2 waits for its last landing, then 05.2/05.3
  extend v9, regenerate it and its goldens; no v10, v8 reader or widened served range.
- **The other encodings change too.** `people-v3.schema` stores an Ask's Member (the saved request used for replay);
  `tasks-v3.schema` stores New through Creation/ProposedBatch (the pending proposal). Both move to v4 in 05.1.
  Core embeds task-family bytes and keeps its own v3: opening checks the family tuple before decoding rows.
  The client's table envelope stays v2; its entries already carry the request owner's wire version. Configuration's
  existing `TemplateDeadline::Span(Duration)` and JSON need no new spelling or version.
- **A deadline selector beside the budget selector.** Add `NewDeadline::{Named, Span(Duration)}` to NewTask, mirrored
  by `Deadline` in people::Member and tasks::New and by a selector in ProposedTask documents. Named uses the authority's
  existing optional wall. Span supersedes that wall; shapes put None there. This follows NewBudget::AsAllowed, which
  already supersedes the authority's spend. It adds no second wall, global deadline type, or changed live-task Authority.
  Preserve the selector in request equality/hash, both store codecs and client restore. Only a resolved Named reaches Make.
- **A coherent small first carrier landing.** 05.2 introduces only Named, used by every producer and reader. The Span
  variant arrives in 05.3 with its complete admission, persistence and translation paths. No intermediate release exposes
  a span that the engine ignores or refuses merely because the next increment has not landed; no compatibility path stays.
- **Resolution belongs to authority.** A pure authority helper receives each check's decision wall and duration, using
  the saturating nanosecond addition that shapes::fill uses today. Zero gives that wall, overflow the largest wall.
  The core routes its result into both the authority check and the new task. It never rewrites the people child's saved
  ask or pending proposal. Capture the admission/acceptance wall before asynchronous lookups; disk completion cannot move it.
- **Order and shared files:** 05.1 can run beside jig 02/03. Its store module/import changes meet jig 03 in
  `jig-protocol-store/src/{lib,codec,tests/codec}.rs`; preserve jig 03's core/transcript changes on rebase.
  Jig 03 also shares `jig-client-protocol-wire/schema/client-v9{,-part1}.schema`, generated v9 and goldens,
  the two protocol ends' document adapters, and `tests/client-protocol/src/{client,fixtures}.rs`.
  Jig 02 shares the client-world helpers and `tests/client-protocol/tests/{client.rs,client/goals.rs}` through Send's
  name. 05.2/05.4 do not depend on named send; use the landed Send shape. 05.5 reuses 02's `send_named`/`made_under`.
- **Downstream inventory for temper 03.1 (planning only):** its capability is **temper on jig's named send, turn read
  and spans**. Remove chat's wall argument in `crates/temper-client-domain/src/pages/chats.rs` (`Chats::start`) and
  `src/tests/pages/chats.rs`; follow Member in `crates/temper-service/src/tests/fixtures.rs` (`people_member`), and New
  in `tests/{conformance/src/scenarios,engine/domain/tests/root_routes}.rs`. ProposedTask's span reaches
  `crates/temper-client-view/src/cards/proposal.rs` and its authority words in `src/cards/words.rs`.
  No set_goal/GoalDraft caller was found in temper's `crates/`, `tests/` or `testing/`. Its people_v3 users DO follow 05.1:
  `crates/temper-protocol-store/src/records/{application,application/refusal}.rs` and `src/tests/records.rs` move to people_v4.
  These are the repin's inputs, not work assigned here; neither a tools-local clock adapter nor a changed template is needed.
- **Other jig users:** the reference UI's `page/{conversations,project/goals}.rs` and `sections/project/goals.rs`, the protocol world's
  `src/client/shapes.rs` and `tests/{client.rs,client/goals.rs}`, the client world's
  `src/{stories,stories/goals,harness/referee/offers}.rs`, the ops world's `src/client/stories.rs` and
  `tests/client_stories/creating.rs`, and jig-client's shape tests call `shapes::{chat,set_goal,fall}` today.
  `examples/ops` and `testing/` have no direct call of these two shapes; the testing service constructs people::Member,
  and the scripted client engine owns typed NewTask payloads/admission. Those carriers and the typed joint adapter follow.
- **Judgments use working copies.** 05.3 threads the step's `Env.wall` through each span-judging function, including
  `propose::allowed`, `goal_accept_allowed`, holder rechecks and `acts::objects::decisions::proposal`. The authority helper
  resolves only the copy judged there; later checks start from the retained span. 05.4 passes the filled deadline selector
  to `fall`, skipping deadline comparisons for Span alone. No check invents a clock, no new commit kind is needed,
  and no timer re-routes proposals merely because a span would now exceed a wall (designer's answer 34).

## Rules that bind it

- The plan's one-owner and boundary rules: authority resolves; the root routes; tasks keep the resolved authority.
  Change existing carriers and every translation together, including stored requests and proposal documents.
  Jig code, tests and commit bodies name no downstream application. Edit no design document or STATUS row.
- No forge, fake LLM or provider file is in this session. Reuse the existing worlds, scripted workers and fixtures.
  Keep additions out of the nearly full `tests/client-protocol/tests/client.rs` and the 817-line `tests/core/tests/parties.rs`.
- All cargo commands run unoptimized; no profile change. Units stay below 20 ms, focused world/joint stories
  below 0.5 s each; the suite budgets stay 15 s and 60 s. Regeneration changes schemas' outputs, not the generator.
  Proofs naming new code run once when built; readiness checks compile the sketches against their landed prerequisites.

## Increments

### 05.1 The stored request and proposal families at version 4
Scope: move the two affected store-family versions before changing their fields; store.md 5; step `jig-05.1`.
Needs: nothing beyond the session's start.
Files: `crates/jig-protocol-store/schema/{people,tasks}-v3.schema` renamed to v4; corresponding `src/generated/`
modules and `golden/{people,tasks}/v4/`; `src/{lib,format,codec,sizes}.rs`, `src/codec/{core,people,tasks}.rs` and their
child modules' imports; `src/tests/{format,commits,open,codec}.rs` and codec submodules;
`tests/store-codecs/tests/{drift,memory,fuzzy_decode}.rs`.
Shape: change each family declaration to 4 and `format::current` to `tasks: 4, people: 4`. Update every active
people_v3/tasks_v3 import and golden reader, including core's embedded-task codec limits. Core remains 3, notes/component 1.
Bootstrap generated filenames from the old generated modules, then run
`SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-store-codecs-world --test drift`; rerun without the variable.
Old generated modules/current golden readers go; no record fields change yet. No old-family decoder is retained.
Test shape: existing codec/drift tests; `src/tests/open.rs` reuses `planted`, `version_put`, `assert_refusal` for
`a_store_of_another_tasks_version_refuses_naming_tasks` (found 3, wanted 4), plus the equivalent people test.
Extend the fixture's format construction for people without duplicating it. Existing old-row codec checks use v3 bytes.
The version cases follow: `format.rs`'s `a_format_row_of_another_tasks_version_is_refused_naming_tasks` and `open.rs`'s
`a_store_of_another_tasks_version_refuses_naming_tasks` want 4; `another_family_is_refused_naming_the_first_difference`'s
people case finds 5, wanted 4, with an arm per family (clippy denies `wildcard_enum_match_arm`); `commits.rs`'s
`a_first_commit_puts_the_format_row_with_its_header` expects `tasks: 4, people: 4`; `codec.rs`'s
`a_people_row_of_another_version_does_not_decode` writes version 5. `memory.rs` and `fuzzy_decode.rs` say core is 3 and
tasks and people 4.
Proof: joint, `cargo nextest run -p jig-client-protocol-world --test client -E 'test(=the_engine_restarting_under_a_pending_stop_answers_it_once)'`;
the real engine/store reopens its new family tuple and answers the retained request once, under 0.5 s.
Don't: move the client wire, core/store component version or pins; change schema fields; retain a migration reader.
Size: at most 180 production lines; generated files, goldens and mechanical test imports aside.
Removes: active v3 people/tasks modules and readers, replaced by v4.

### 05.2 One deadline selector through every boundary
Scope: give existing named walls an explicit request selector, including replay and pending proposal documents;
contract.md 3.2, protocol/client.md 3; step `jig-05.2`.
Needs: jig's read of one turn, landed. 05.1's store versions precede this increment.
Files: `crates/jig-client-domain/src/contract/{operations,documents}.rs`; `crates/jig-domain-people/src/{batch,lib}.rs`;
`crates/jig-domain-tasks/src/{boundary,lib}.rs`;
`crates/jig-domain-core/src/{routing/create,routing/run_delegation,routing/tasks,documents/objects}.rs`;
`crates/jig-client/src/shapes.rs`; `crates/jig-{client-protocol-link,protocol-engine-clients}/src/translate/operations.rs`;
`crates/jig-protocol-engine-clients/src/translate/objects/proposals.rs`,
`crates/jig-client-protocol-link/src/translate/documents_part7.rs`;
`crates/jig-client-protocol-wire/schema/client-v9.schema`, generated v9 and goldens;
`crates/jig-protocol-store/schema/{people,tasks}-v4.schema`, generated modules/goldens,
`crates/jig-protocol-store/src/codec/{people/batch,tasks/phase}.rs`.
Also `testing/jig-client-scripted-engine/src/{payload,payload/operations}.rs`, `tests/client/src/joint/wire/asks/create.rs`
and proposal/history adapters; all constructors in the surveyed unit/world fixtures, notably `testing/jig-test-service/src/tests/fixtures.rs`,
`tests/{core,people,tasks,conformance,channel,client-browser,client-protocol}/` and `crates/jig-client-view/src/tests/`;
`tests/client/tests/stories.rs` (its `NewTask`); the unit, world and scripted fixtures that build these literals outside
those directories: `crates/jig-domain-core/src/{tests,tests/fixtures}.rs`, `crates/jig-domain-people/src/tests/fixtures.rs`,
`crates/jig-domain-tasks/src/tests/{last_batch,revisions,revisions/text}.rs`, `crates/jig-protocol-store/src/tests/fixtures.rs`,
`testing/jig-client-scripted-engine/src/{owner_history/tests,owner_proposal/tests,tests/fixtures,tests/recurring}.rs`,
`tests/client/src/{stories,joint/engine_stories,joint/wire/history,joint/wire/historical/tests,joint/wire/projects/tests}.rs`
and `tests/ops/src/conformance/mod.rs`; `crates/jig-protocol-store/src/sizes.rs` and `src/tests/format.rs`, whose
worst-case sizes (81_241, 81_353, 17_499, 34_998) follow the grown rows; `tests/client-wire/tests/fuzzy_client_wire.rs`, whose hand-listed wire
types gain `NewDeadline` here and `DeadlineSpan` in 05.3. The shipped store profiles (ruling 118):
`crates/jig-protocol-config/src/profile/{small,standard}.rs` (`STORE_KV`'s `value`, `chunk`, `page.bytes`) and
`testing/jig-test-service/src/tests.rs` (`both_jig_profiles_have_a_bounded_service`'s rows).
Shape: add the selector field and ONLY Named, as sketched below, to the existing types. Every producer supplies Named,
every translation matches it exhaustively; made tasks' stored Authority stays wall-only. ProposedTask carries the selector
through current and historical documents, including recurring batch documents. Share each existing batch codec between
Create and Propose. Preserve all other fields. Recompute payload/worst-case accounting where the inline carrier grows.
A shipped profile whose `value` a grown row passes (`sizes::check`'s needs over has) takes the growth, needs minus has,
in `value`, `chunk` and `page.bytes` alike, and the service test's row follows (ruling 118).
Regenerate store v4 as in 05.1; wire v9 with
`SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-client-wire-world --test drift`; rerun without the variable.
Test shape: extend existing link `src/tests/create.rs` and fixtures/create.rs, engine-client `src/tests/fixtures/create.rs`,
and store `src/tests/codec/{record,phase}.rs`: Named with no wall and with a finite wall survives Create, Propose and restore.
Reuse the drift and `tests/store-codecs/tests/profiles.rs` shipped-profile checks; assert every changed bound still fits,
reporting needs/has, not assuming unchanged memory. Units under 20 ms; no new test file or fixture family.
Proof: joint, `cargo nextest run -p jig-client-protocol-world --test client -E 'test(=a_party_creates_a_batch_whose_as_allowed_task_hears_what_it_was_given)'`;
the new wire/engine carriers still make both roots and preserve the budget answer, under 0.5 s.
Don't: add Span before its end-to-end meaning exists; change public Authority/Budget walls, amendments or template configuration.
Size: at most 280 production lines, including carrier consumers and codecs; generated code and tests aside.
Removes: implicit selection of an authority's wall at batch boundaries; no compatibility constructor.

### 05.3 A span kept until the task-making decision
Scope: add Span through 05.2's carriers, resolve at Create/acceptance, and retain the original ask/proposal;
authority.md 7, 8.1, tasks.md 4, 9, engine.md 4; step `jig-05.3`.
Needs: nothing beyond 05.2's deadline selectors.
Files: 05.2's enums, schemas and translations; `crates/jig-domain-authority/src/{lib,value}.rs` and existing tests;
`crates/jig-domain-core/src/{goals,proposals,effects,acts,acts/requests,acts/objects,acts/objects/decisions}.rs`
(`acts::request` calls `admit`), `src/routing/{create,propose,party_decisions,parties,people,tasks,runs,procedure_steps,views}.rs`,
`src/documents{,/objects,/offers,/deployment,/task_tree,/project_note}.rs`, `src/reads{,/rows}.rs` and core test callers;
new `crates/jig-domain-core/src/routing/deadlines.rs` declared in `routing/mod.rs`;
`crates/jig-domain-tasks/src/{batch,stored}.rs`; `crates/jig-protocol-store/src/sizes.rs` and `src/tests/format.rs`, as in 05.2;
the shipped profiles and the service test's rows, as in 05.2 (ruling 118);
`crates/jig-client-view/src/{cards/proposal,proposed_task,proposed_task_words}.rs` and their existing tests;
`testing/jig-client-scripted-engine/src/{requests,requests/creating,payload/operations}.rs`;
new `tests/core/tests/deadline_spans.rs`; `tests/client-protocol/src/client/operations.rs`,
new `tests/client-protocol/tests/deadline_spans.rs`; existing core, link, engine-client and store codec test fixtures.
Shape: Span carries Duration in each schema, never an epoch u64. Authority's `deadline_after` performs the one arithmetic
rule. Every judging function takes `decision: Wall` from its step's Env. Create, including set_goal's Create, resolves a
working copy before constructing/checking delegates; its authority refusal computes could-be-accepted at that same wall.
`propose::allowed` and `proposals::action_for_check` judge resolved copies for admission's ceiling, could-be-accepted and
holder selection. Propose keeps Span in tasks::New through ProposePerson, Stored::PersonProposal and restart; no resolution
is saved there. `proposal_holder` and `proposal_recheck_project` recompute from Span at every pass, re-route and stall.
`goal_accept_allowed` and `proposal_accept_allowed` check the accepter with acceptance's wall; that same resolved working
batch reaches Make and funding, through party_decisions or runs. Preserve its task numbers and the original proposal facts.
`acts::objects::decisions::proposal` receives the wall through acts::admit and documents' offers. Thread it through
documents::make/objects::proposal and other offer callers; make_with_clock forwards DocumentClock.wall, views supplies Env.wall.
An aged span outside its accepter is answered beyond authority (8.1), with no Make or funding; the holder can pass it up
under tasks.md 9. Recheck the ceiling too: past it, refuse. Add no automatic proposal, commit kind or time-driven re-routing.
Only Named with its checked optional wall reaches Make; unresolved Span there is a root invariant failure. Pending and
historical proposal rendering distinguishes Span from no deadline. No wall comes from IO, sent-at, proposal since or restart.
The scripted engine uses its supplied send/acceptance wall with the same semantics, and the typed joint adapter preserves
the selector. Extend store round trips for saved Create/Propose asks and pending/decided proposals; regenerate v9/v4.
Recompute changed inline, payload and codec bounds again now Span carries Duration; run the shipped-profile checks.
Test shape: units `zero_span_is_the_decision_wall`, `span_addition_saturates_at_the_last_wall`, and codec cases for Span.
The new core file reuses `stories::offers::world_with`, `fixtures::{ask_as,people_member,live_row}` and World's `set_clock`;
limits come from world_with's existing profile-derived builder. Its reference:

```rust
use jig_core_world::{
    fixtures::{ask_as, live_row, people_member},
    stories::offers::world_with,
};
use jig_domain_people as p;
use skein_lib::{Duration, Time, Wall};
#[test]
fn a_span_starts_at_create_admission() {
    let mut world = world_with(301, Some(p::Role::Member), |_| {});
    let sign_in = world.sign_in.expect("signed in");
    world.set_clock(Time::from_nanos(1_000_000_000), Wall::from_nanos(10_000_000_000));
    let mut member = people_member(b"work", 20);
    member.deadline = p::Deadline::Span(Duration::from_secs(5));
    ask_as(&mut world, sign_in, 31, p::Ask::Create { project: 1, batch: Box::new([member]) });
    assert_eq!(live_row(&world, 2).authority.budget.deadline, Some(Wall::from_nanos(15_000_000_000)));
}
```

Add `a_pending_span_survives_restart_until_acceptance` (stored Span, then accepted wall),
`a_delayed_commit_does_not_move_the_deadline`, and `a_span_past_the_ceiling_is_refused_when_proposed`.
`an_aged_span_is_beyond_its_accepter_and_passed_up`: it fits its holder at proposal, time advances without re-routing,
then the offered Accept and sent acceptance agree on beyond authority; no task/funding changes, the holder passes it up,
and a covering holder accepts with that later wall plus Span. Also cover aging past the ceiling with no Make.
`holder_rechecks_use_their_own_wall` covers pass, re-route and stall; `a_resolved_span_beyond_the_creator_is_refused`
checks the refusal's could-be-accepted answer at admission. Reuse `world_with_policy` for finite role/project bounds,
parties.rs's propose/decide and existing proposal-holder fixtures; extend one shared fixture, not parties.rs. Each under 0.5 s.
The new protocol file reuses `Story::live_batch`, `fixtures::create_tasks`; add `Story::task_deadline` beside task_budget,
reading the same task_record. Its reference, using only the existing fixture's first member's fixed task number:

```rust
use jig_client::{Ask, Asker, Key, contract as c};
use jig_client_protocol_world::{client::Story, fixtures};
use skein_lib::Duration;
#[test]
fn a_span_crosses_the_client_wire_to_task_admission() {
    let mut story = Story::live_batch(fixtures::SEED, true);
    let mut tasks = fixtures::create_tasks();
    tasks.get_mut(0).expect("first task").deadline = c::NewDeadline::Span(Duration::from_secs(5));
    let before = story.wall().as_nanos();
    let key = Key { asker: Asker::JIG, number: 85 };
    story.send(key, Ask::Jig(Box::new(c::Ask::Create { project: 1, tasks })));
    assert!(story.run_until(|s| !s.answered(key).is_empty(), Duration::from_secs(1)));
    let at = story.task_deadline(1).expect("made with deadline").as_nanos();
    assert!((before + 5_000_000_000..=story.wall().as_nanos() + 5_000_000_000).contains(&at));
}
```

Proof: joint, `cargo nextest run -p jig-client-protocol-world --test deadline_spans -E 'test(=a_span_crosses_the_client_wire_to_task_admission)'`;
the real engine answers a span carried as bytes, under 0.5 s; world,
`cargo nextest run -p jig-core-world --test deadline_spans -E 'test(=an_aged_span_is_beyond_its_accepter_and_passed_up)'`,
once, under 0.5 s: the aged acceptance is refused, passed up and made with the accepting decision's wall.
Don't: mutate the saved ask, reset a span on retry, start it at proposal submission, bypass finite authority, or allocate
a second batch's task numbers on acceptance. Do not change fake LLM/provider scripts to exercise ordinary task creation.
Size: at most 300 production lines; generated code and tests aside.
Removes: wall-only assumptions in admission, proposal judgments, document offers and batch codecs.

### 05.4 Shapes without a clock
Scope: clock-free spans and draft judgments, contract.md 5.3, client.md 19; step `jig-05.4`.
Needs: nothing beyond 05.3's engine judgments of spans.
Files: `crates/jig-client/src/{shapes,tests/shapes}.rs`;
`crates/jig-client-view/src/page/{conversations,project/goals}.rs`, `src/sections/project/goals.rs`, `src/tests/shapes.rs`;
`tests/client-protocol/src/client/shapes.rs`, `tests/client-protocol/tests/{client.rs,client/goals.rs,deadline_spans.rs}`;
`tests/client/src/{stories,stories/goals,harness/referee/offers}.rs`;
`tests/ops/src/client/stories.rs`, `tests/ops/tests/client_stories/creating.rs`.
Shape: `chat(me, project, words)`, `set_goal(me, project, draft, reason)` and private fill take no Wall. None/Given use
Named and the existing optional wall; a template Span produces Span and clears the authority wall. GoalDraft's user-given
deadline remains Option<Wall>. Pass the filled NewDeadline to fall: Span is within both `within.latest` and
`maximum_deadline`; Named uses the existing optional-wall comparison, including no deadline beyond a finite bound.
Budget, executor and authority still select Create, Propose, Refused or NotOffered as before. Span skips no other check.
Both UI fall callers select the template's deadline the same way as set_goal; their prediction and submitted ask agree.
Keep close_chat, its grace and explicit walls elsewhere unchanged. Remove shape-filling clocks only; UI rendering still has time.
Test shape: extend existing shape tests/fixtures, replacing the old SPAN_DEADLINE expectation with the duration itself.
`a_template_span_is_kept_in_the_ask`, `the_same_inputs_and_me_make_the_same_ask`, `a_given_deadline_remains_a_wall`,
`a_goal_span_is_within_both_finite_deadline_bounds` (unchanged inputs/me still yield Create), and
`a_goal_span_keeps_the_other_creating_judgments` (budget, executor, authority and unavailable template cases).
Reuse existing fall cases for Given and None; UI shape tests check the same prediction before sending. Each unit under 20 ms.
Reuse 05.3's protocol story in `a_shaped_chat_span_reaches_the_engine`, getting me through shapes_me and changing its
chat template's deadline in a local input copy only; the engine must not expand a template. Also
`a_shaped_goal_span_beyond_authority_is_not_automatically_proposed`: finite first bound, a Span-shaped Create sent,
engine Authority with could-be-accepted, no proposal until explicitly sent. Use the world's existing policy fixture.
Proof: joint, `cargo nextest run -p jig-client-protocol-world --test deadline_spans -E 'test(=a_shaped_chat_span_reaches_the_engine)'`;
the clock-free helper reaches the real engine and stores its wall, under 0.5 s; unit,
`cargo nextest run -p jig-client -E 'test(=a_goal_span_is_within_both_finite_deadline_bounds)'`, once, under 20 ms,
shows the pure draft judgment against both finite bounds; 05.3 and the joint story cover engine authority.
Don't: supply Wall::EPOCH to keep a signature, keep a compatibility shape, read a clock in fall, or silently drop a span.
Size: at most 180 production lines, including call sites and the selector-aware judgment; tests aside.
Removes: shapes' now arguments and client-side span addition; superseded wall-only shape expectations.

### 05.5 A name sent again after time moved
Scope: contract.md 3.1's identical ask and first answer across clocks, storage and restarts; step `jig-05.5`.
Needs: jig's send under a named key, landed. 05.4's clock-free shapes precede this increment.
Files: `tests/client-protocol/tests/deadline_spans.rs`, `tests/client-protocol/src/client/{shapes,operations}.rs` and
`tests/client-protocol/src/client.rs` only if 02's send_named/made_under need a fixture extension. No product or seam files.
Shape: reuse 02's named-send, loss and reload machinery, 05.3's deadline observation and the stored proposal reader
`Story::party_proposal`. Keep one captured me and words, build the ask afresh after advancing simulated time, and compare
the asks as well as their outcomes. Drain the first record's erase before the next application key takes the name.
Test shape: `a_named_span_replayed_after_time_moves_keeps_its_first_task`: save/send, lose its answer, reload, settle,
restart_engine with an offline duration within retention, rebuild from the same me/words, send under a new application
key and the same name; first answer, one make, identical task number and deadline. Also
`a_named_proposal_keeps_its_span_across_client_and_engine_restart`: the pending stored action still has Span, and acceptance
later starts the full span; `a_changed_template_under_the_same_name_is_a_conflict`: the changed duration is a different
ask, no second make. Use fixtures::owner_limits and its retention; no wall sleep, seed sweep or new world.
Proof: joint, `cargo nextest run -p jig-client-protocol-world --test deadline_spans -E 'test(=a_named_span_replayed_after_time_moves_keeps_its_first_task)'`;
the repeated public request crosses HTTP bytes and gets the first durable answer and wall, under 0.5 s.
Don't: compare only task counts, replace the changed clock with a fixed epoch, reuse an already-built ask as the entire
reconstruction test, or duplicate 02's unrelated Held matrix. Merge any helper copies introduced by these joint stories.
Size: at most 70 lines in world helpers; story tests aside.
Removes: duplicate fixture setup introduced by this session, if any.

## Interface sketches

05.2 lands only Named; 05.3 adds Span everywhere in the same increment. These are boundary deltas inside existing
modules, not parallel task types. The client, people and tasks each own their vocabulary, mapped exhaustively.
All fields of the changed task/request carriers are shown. Existing wall-bearing authority types do not change.

```rust
// crates/jig-client-domain/src/contract/operations.rs; re-exported by jig-client
/// The request's deadline selection, retained until admission or acceptance.
pub enum NewDeadline {
    /// Use authority.deadline, including None; terminates in the existing optional wall.
    Named,
    /// The creator's duration; authority resolves it at the task-making decision.
    Span(skein_lib::Duration),
}
/// The application's batch member; wire NewTask mirrors every field.
pub struct NewTask {
    pub executor: OwnerExecutor, pub spec: Specification, pub contract: NewContract,
    pub authority: OwnerAuthority, pub budget: NewBudget, pub deadline: NewDeadline,
    pub wake: Option<OwnerWakePolicy>, pub dependencies: skein_lib::List<TaskRef>,
    pub named: skein_lib::List<OwnerGrant>, pub tracked: Option<u32>, pub label: u32, pub flags: TaskFlags,
}
// crates/jig-domain-people/src/batch.rs; tasks has its own identical enum in boundary.rs
/// Mirrors NewDeadline in the authenticated request and its saved Ask.
pub enum Deadline {
    /// The authority's optional wall.
    Named,
    /// Unresolved until the decision that makes the task.
    Span(skein_lib::Duration),
}
/// People owns the request; store people-v4 Member mirrors all fields.
pub struct Member {
    pub executor: TaskExecutor, pub words: Box<[u8]>, pub parameters: Box<[Parameter]>, pub inputs: Box<[u64]>,
    pub contract: Contract, pub authority: Authority, pub budget: Budget, pub deadline: Deadline,
    pub wake: WakePolicy, pub dependencies: Box<[Dependency]>, pub named: Box<[Grant]>,
    pub tracked: Option<u32>, pub label: u32, pub flags: Flags,
}
// crates/jig-domain-tasks/src/boundary.rs
/// Future member kept in a proposal; Make accepts only a resolved Named.
pub struct New {
    pub label: u32, pub flags: Flags, pub number: u64, pub project: u32, pub executor: Executor,
    pub spec: Spec, pub contract: Contract, pub authority: Authority, pub deadline: Deadline,
    pub numbers: Numbers, pub funder: Funder, pub dependencies: Box<[u64]>, pub holdings: Box<[Holding]>,
    pub wake: WakePolicy, pub recurring: Option<Box<RecurringTemplate>>, pub tracked: Option<u32>,
}
// crates/jig-client-domain/src/contract/documents.rs
/// Pending/retained action facts; the selector mirrors tasks::New and the wire ProposedTask.
pub struct ProposedTask {
    pub number: u64, pub spec: SpecificationDocument, pub executor: OwnerExecutor,
    pub contract: TaskContractDocument, pub authority: OwnerAuthorityDocument, pub deadline: NewDeadline,
    pub budget: u64, pub funder: Funder, pub dependencies: skein_lib::List<u64>,
    pub resources: skein_lib::List<OwnerPart>, pub wake: OwnerWakePolicy, pub priority: Option<u32>, pub cut: bool,
}
// crates/jig-domain-authority/src/value.rs (re-exported by lib.rs)
/// Each check supplies its decision wall; only the task-making check stores the result, never back into the ask.
pub fn deadline_after(decision: skein_lib::Wall, span: skein_lib::Duration) -> skein_lib::Wall;
// crates/jig-domain-core/src/goals.rs; existing Core method, with the supplied decision wall
impl Core {
    pub fn goal_accept_allowed(&self, proposal: &tasks::PersonProposal, project: u32, person: u64,
        role: people::Role, tasks_left: u32, decision: skein_lib::Wall) -> Result<tasks::Funder, people::Refusal>;
}
// crates/jig-domain-core/src/acts.rs; the same wall reaches objects::decisions::proposal
pub fn admit(core: &Core, limits: &Limits, party: u64, act: Act, object: &documents::Subject,
    decision: skein_lib::Wall) -> Admission;
// crates/jig-client/src/shapes.rs
pub fn chat(me: &Me, project: u32, words: Text) -> Result<Ask, NotOffered>;
pub fn set_goal(me: &Me, project: u32, draft: GoalDraft, reason: Text) -> Judged;
/// Judge only Named's optional wall; Span passes both deadline bounds, leaving other judgments intact.
pub fn fall(creating: &Creating, draft: &GoalDraft, deadline: NewDeadline) -> Fall;
```

The wire mirrors NewDeadline with `named` and `span: DeadlineSpan`; `DeadlineSpan { value: duration }`.
Store people-v4 Member and tasks-v4 Member use the same two-variant shape; Creation still wraps Member plus recurring.
Core's existing documents::ProposedTask gains `deadline: tasks::Deadline`, preserving every other field in its translation.
No span is placed in TaskRecord.authority, a policy ceiling, or an amendment's authority.

## Questions to raise when reached

- A changed carrier exceeds a shipped profile's memory/codec bound: report the measured needs and available room before
  changing a limit. A store profile's `value`, `chunk` and `page.bytes` raised by the measured growth is no question
  (ruling 118); a need in `commit`, `ops`, `queued` or memory is. A proof exceeds 0.5 s: profile it; do not change optimization or wait on jig 04 to excuse the cost.
- Rebase reveals a newly landed consumer outside this inventory or a different format owner: report the exact source
  and necessary repair.

## Done when

Run these commands in `~/src/rust/jig` after `git fetch`. They inspect final `origin/main` files, never history or STATUS.
The new positive checks are expected to fail at the drafting base; they run after this session lands.

- `git show origin/main:crates/jig-client-protocol-wire/src/lib.rs | grep -qx 'pub const VERSION: u32 = 9;'`
- `git show origin/main:crates/jig-protocol-store/src/format.rs | grep -q 'tasks: 4, people: 4'`
- `git grep -q 'family jig-store-tasks 4' origin/main -- crates/jig-protocol-store/schema`
- `git grep -q 'family jig-store-people 4' origin/main -- crates/jig-protocol-store/schema`
- `test -z "$(git grep -l -E 'tasks_v3|people_v3' origin/main -- crates/jig-protocol-store/src tests/store-codecs/tests)"`
- `git show origin/main:crates/jig-client-domain/src/contract/operations.rs | grep -q 'pub enum NewDeadline'`
- `git show origin/main:crates/jig-client-protocol-wire/schema/client-v9.schema | grep -q 'enum NewDeadline.*span: DeadlineSpan'`
- `git show origin/main:crates/jig-client-protocol-wire/schema/client-v9.schema | grep -q 'record DeadlineSpan.*value: duration'`
- `git grep -q -w -F 'Deadline::Span' origin/main -- crates/jig-protocol-store/src/codec/people`
- `git grep -q -w -F 'Deadline::Span' origin/main -- crates/jig-protocol-store/src/codec/tasks`
- `git grep -q -F 'deadline_after' origin/main -- crates/jig-domain-core/src/routing`
- `git show origin/main:crates/jig-client/src/shapes.rs | grep -q -F 'pub fn chat(me: &Me, project: u32, words: Text)'`
- `git show origin/main:crates/jig-client/src/shapes.rs | grep -q -F 'pub fn set_goal(me: &Me, project: u32, draft: GoalDraft, reason: Text)'`
- `git show origin/main:crates/jig-client/src/shapes.rs | grep -q -F 'pub fn fall(creating: &Creating, draft: &GoalDraft, deadline: NewDeadline)'`
- `! git show origin/main:crates/jig-client/src/shapes.rs | grep -q -E 'now: Wall|now\.as_nanos'`
- `test -n "$(git grep -l --all-match -e 'Span' -e 'restart' origin/main -- tests/core/tests)"`
- `test -n "$(git grep -l --all-match -e 'send_named' -e 'made_under' -e 'deadline' origin/main -- tests/client-protocol/tests)"`
