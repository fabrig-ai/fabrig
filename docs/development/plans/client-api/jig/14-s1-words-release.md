# Supplement jig-14.s1: words release nothing; the words' release leaves the engine

Added by ruling 210, from the design answer of 2026-10-17T02:41Z (jig
`5a9a97485`: `domain/people.md`, 5.1; `domain/tasks.md`, 7;
`client/domain/contract.md`, 3.4). Read first: jig's `AGENTS.md`,
`docs/development/workflow.md`; this plan's README, "Rules of this plan"
and "The run's steers"; `jig/14-wire.md`, "Rules that bind it" and 14.3.

Scope: people.md, 5.1: "Words release nothing: a `Message` adds words to
the inbox, and words to a task the party stopped come with a `Release` in
one compound request (below), naming the hold's revision as it would
alone."
Needs: jig 14.3 (its `Message` translates to `release: false`). It runs
in a slot of its own, beside 14.4.
Files, Shape, Tests, Proof, Size and Don't: ruling 210's Decision, point
4, as written (Files at jig `5a9a97485`; follow each by its name). Size:
at most 150 lines of production code.
temper: ruling 210's point 5 names temper's sites; the temper session
that repins jig past this step edits them.

## Corrections at jig `b6a63f8c1` (ruling 249, the ready check of 2026-10-17)

Ruling 210's point 4 stands; these join its Files, Shape and Tests:
- Production: `crates/jig-domain-core/src/routing/runs.rs:465`, the
  agent Send route's `tasks::Event::Message { release: false }` (the field
  goes); `crates/jig-domain-core/src/sibling_routes.rs` 443-496 (was
  443-476: `Outcome::Said { .., released }` is built at :489);
  `crates/jig-domain-tasks/src/procedure.rs:105-113` (`inbox::message(..)`
  loses its release argument); `crates/jig-domain-tasks/src/compound/staged.rs:152-155`
  (`Plan::Message`'s `plan.release` projection goes; :355, Amend's, stays);
  `tasks::Operation::Message { release }` (`compound.rs:15`),
  `MessagePlan.released`, and `apply_message`'s `(u64, Word, bool)` return.
- Generated: `crates/jig-protocol-store/src/generated/people_v3.rs` and
  12 goldens, `golden/people/v3/{enum_ask_say,enum_operation_message,enum_outcome_said,record_ask_say,record_operation_message,record_outcome_said}_{full,smallest}.bin`:
  `SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-store-codecs-world --test drift`,
  from a checkout where the schema is edited and the codec users are not
  yet (the lib does not compile between them).
- Derived value: `crates/jig-protocol-config/src/tests/domain.rs:197`,
  `What::Memory { over: 17_861_883_295 }` becomes the value the test
  computes after the change (the check saw `17_861_883_231`, 64 bytes
  less): take it from the test's own failure, once.
- Worlds and fakes: `tests/people/src/world.rs` 106, 150, 690-698,
  922-926 (the `sayings` tuple and the hold removal) and
  `simulation.rs:512` (an assertion changes, not only the field);
  `testing/jig-fake-parties/src/lib.rs:184-188` (`Reply::Said { released }`
  and its doc's "possibly with the stop lifted"), with its translators
  `tests/core/src/peers.rs:403` and `tests/channel/src/world.rs:301`.
- Downstream (for the repinning session of the repository downstream of
  jig, beside point 5's sites): `tests/system/src/composed.rs:622-623`;
  point 5's `tests/engine/domain/src/direct.rs:347` is now `:355` and
  `tests/engine/domain/tests/forge_routes.rs:2068` is now `:2069`.
- jig tests use `match`, not `matches!` (disallowed in jig's tests).
Any further user of the removed fields is refreshed the same way and
listed in the commit body.

## Corrections at jig `a37b5fe12` (ruling 265, the re-check after 14.3)

A compile of the change in a scratch worktree found these users of the
removed fields; each joins the Files where ruling 210's point 4 does not
already name it:
- Production: `jig-domain-people` `boundary.rs:272` (`Ask::Say`), :443
  (`Outcome::Said`), `compound.rs:13, 27`, `domain.rs:1290, 1332, 1385`
  (`Expected::Say`); `jig-protocol-store/src/codec/people/` `asks.rs:490-492,
  667-669`, `compound.rs:35-38, 78-79`, `outcomes.rs:85-86, 197-199,
  315-321`; `jig-protocol-engine-clients/src/translate/` `asks.rs:18`
  and `operations.rs:129` (14.3's `release: false`); `jig-domain-tasks`
  `boundary.rs:1378, 1380, 1651`, `domain.rs:619-623`, `inbox.rs:214-251`,
  `compound.rs:297-305`; `jig-domain-core` `sibling_routes.rs:59, 68, 99,
  114`, `routing/tasks.rs:153-155`, `routing/people.rs:410`,
  `routing/party_controls.rs:273-277, 306` (`say_request`),
  `compound.rs:204-217`.
- The fake parties' `Ask::Say { release: Option<u64> }`
  (`testing/jig-fake-parties/src/lib.rs:82-83`) stays, as the fixture's
  instruction (ruling 266): its translators into the engine's asks
  (`tests/core/src/peers.rs:216`, `tests/channel/src/world.rs:284`) send
  `Some(revision)` as one `Compound` of the words and a `Release` naming
  that revision, and `None` as the words alone; its doc says so. Its
  other users (`tests/core/tests/peers.rs:215`,
  `tests/channel/src/referee.rs`, `tests/channel/src/fixtures.rs:414`,
  `tests/conformance/src/actions.rs:41`, `tests/core/src/faults.rs:56`)
  change only where the engine's removed fields reach them.
- Tests: `tests/core/src/observations.rs:734`, `tests/client/src/joint/truth.rs:197`,
  `tests/client/src/joint/wire/{answers.rs, outcomes.rs}` (:116, :170),
  the engine-clients component's `src/tests/fixtures.rs:271`,
  `tests/fixtures/answers/outcomes.rs:95`, `tests/compound.rs:24`; the
  `Event::Message { release }` tests of `jig-domain-core` and
  `jig-domain-tasks` (`tests/revisions.rs:595, 611, 673` among them) and
  `tests/tasks/tests/`. `simulation.rs:512` is
  `tests/people/tests/simulation.rs`: its test
  `words_to_a_stopped_chat_release_it_in_one_request` asserts the rule
  this step removes, and becomes `words_to_a_stopped_chat_release_nothing`.
- `inbox::message`'s `#[expect(clippy::too_many_arguments)]` stays (8
  arguments after); its reason loses "atomic release choice".
- Downstream: `tests/engine/domain/tests/forge_routes.rs`'s `Ask::Say`
  is at :2070; temper's `tests/conformance/src/stories.rs:220, 237`,
  `tests/system/src/world.rs:244`, `tests/system/src/composed.rs:391,
  441-442`, `tests/system/tests/composed.rs:39` join point 5's sites.
Checked: with these edits, `jig-protocol-config`, `jig-protocol-store`,
`jig-store-codecs-world`, `jig-domain-people`: 473 of 473 (default),
14 of 14 (fuzzy).
