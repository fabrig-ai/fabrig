+++
id = "ways-read"
part = "read"
kind = "feature"
needs = ["ways-surface", "ask-words"]
files = ["temper/crates/temper-engine-domain/src/route/ways.rs", "temper/crates/temper-engine-domain/src/route/ways/**", "temper/crates/temper-engine-domain/src/route/panels.rs", "temper/crates/temper-engine-domain/src/route/see.rs", "temper/crates/temper-engine-domain/src/route/kinds.rs", "temper/crates/temper-engine-domain/src/route.rs", "temper/crates/temper-engine-domain/src/tests/see.rs", "temper/crates/temper-engine-smith/src/see.rs", "temper/crates/temper-engine-smith/src/tests/see.rs", "temper/tests/engine/domain/tests/suite/ways.rs", "temper/tests/engine/domain/tests/suite/main.rs", "temper/tests/engine/domain/tests/suite/configuring.rs", "temper/docs/design/domain/agent.status.md", "temper/docs/design/domain/landing.status.md"]
effort = "high"
minutes = 260
+++

## Goal

Make the root answer a project's `ways`: gather what a path is derived
from, out of the forge and the core, lay each stock way's items over it,
and make the `ways` document for its subject, watched and marked; and
answer `see ways` from the same document for the chat's person, rendered
as text (`temper/docs/design/domain/forge.md`, 8.13;
`temper/docs/design/protocol/configuration.md`, 13.10;
`temper/docs/design/domain/agent.md`, 8.4.1's `ways` row).

Acceptance:

- `route/ways.rs` has, as `pub(crate)` functions the view read
  (`ways-view`) reuses:
  - `inputs(domain, project, party, items: &[ways::Item]) -> ways::Inputs`,
    the project's path's inputs with `items` laid over them: the
    project's home repository, else its first adopted, else none (its
    host, owner, name, `ci`, `checks`, its test gates' names and its tiers
    from the forge's landing state); its landing branch, `main` with none;
    `ways::laid` over `Domain::landing_rules(project)` and the forge's
    given every-project rules; kind 3's producer and reviewer
    (`Domain::charters()`); the start from kind 7 (`ways::laid_ways`) and
    the project's templates (`core.project_templates(project)`: `way` 0
    the lowest-numbered offered template with `tracked` set, as the
    client's `default_way` reads it), `Start::Goal { charter }` where that
    template's resources hold its own goal branch and its executor is a
    charter, else `Start::Chat`; `by` (`Domain::landing_way`); the
    policy's roles by number, named "owner", "maintainer", "member" and
    "observer" for 0 to 3 and "role N" past them, `yours` where the party
    holds that role in the project; an `Agent` for each charter the path
    names (the producer, the reviewer, the goal template's): its model the
    first of the charter's order the party's offer holds, as the settings
    inventory's offer reads it (`route/host_route.rs`, `settings_offer`),
    by its shown name, at its default effort, `usable` where one is held,
    else the charter's first and `usable` false; `ways::can` of its
    policy's families; `ways::first_sentence` of its instructions (its
    shared instructions where it has parts);
  - `document(domain, project, party) -> ways::WaysDocument`: each of
    `config.ways_stock` in order, its `items` (`ways::items` over the
    project's kind 7), its path (`ways::derive` over `inputs` with those
    items), its `delegation` where the way's is set and the project's
    policy's ceiling or role 0's delegation lacks charter 5, role 0 or job
    3; `own`, the path over no items; `current` (`ways::current`);
    `may_change`, whether the party is an owner of the project, as jig's
    `ChangeSettings` asks.
- The subject `ways::subject(project)` of the project's scope is made by
  the root, as `approvals` is (`route/panels.rs`), for a party with a role
  in the project, and delivered as `Delivery::Ways`.
- Marked (`core::Event::Marked` of the forge's connector, as the forge's
  `Request::Marked` becomes one) for each affected project's subject: in
  the decision that commits a change of temper's kinds (where the root
  hands committed items to the forge, `route/kinds.rs`), at a repository
  adopted into a project or removed from it, and, where the root hears
  them, at a committed change of jig's own settings and at a project's
  role change.
- `see ways` (`route/see.rs`): `ways` leaves the not-yet addresses and
  answers `SeeAnswer::Ways` of the document for the chat's person (the
  chat task's requester), rendered by `temper-engine-smith`'s `see` as
  `agent.md`, 8.4.1's row says: a block a way, `NAME` or `NAME · now`,
  then its line, then its stops joined by ` → `, each `WORDS (ACTOR)`
  with the actor "a person", "an agent", "code" or "a branch"; a blank
  line between blocks; where `current` is none, a last block `Yours ·
  now` and the own path; then "You may change this project's settings."
  or "You may not change this project's settings; an owner may."; its
  `see:` line `settings rules · settings landing · settings project-ways ·
  help settings`.
- Tests:
  - the engine's domain world, a new `tests/suite/ways.rs` beside
    `panels.rs`, its world's `ways_stock` the stock's
    (`temper_service::stock::ways()`): a project's `ways` watch delivers
    its document after the commit barrier, with three ways by the stock's
    names, each way's items as 13.10 has them for that project, and
    `may_change` true for its owner and false for a member; a direct
    `ChangeSettings` of the project's rule 1 marks the subject and the next
    document's `own` path's inputs follow it (assert on `inputs` or on the
    stops' `opens`, not on words `ways-derive` writes); a repository
    adopted marks it;
  - `inputs`' table, in the same file or the root's own tests: a project
    with no repository; with a home repository with CI, tiers and test
    gates; kind 7 with a goal template (`Start::Goal`); a member reader's
    roles;
  - `temper-engine-smith/src/tests/see.rs`: rows of `see ways`'s text from
    hand-built documents (a current way; `Yours`; may and may not change);
  - `temper/tests/engine/domain/tests/suite/configuring.rs`,
    `an_asking_chat_reads_each_address`: `ways` moves from the not-yet
    rows to a row answering its first line.
- `temper/docs/design/domain/agent.status.md`: the 8.4.1 sentence on
  `ways` leaves its **Not yet**; `temper/docs/design/domain/landing.status.md`:
  its 13.1 **Not yet** on a branch's path goes.

## Inputs

- `temper/docs/plan/ui-ask-m3a/notes/common.md`; `notes/contracts.md`,
  sections 1 and 4 (binding).
- `temper/docs/design/domain/forge.md`, 8.13 and 8.12 (a subject the root
  makes, `approvals`: its pattern to follow), 11.1 (a goal's own branch);
  `temper/docs/design/protocol/configuration.md`, 13.10, 13.2 (*Read from
  the forge*), 13.4 (the stock's rule 1 given per project);
  `temper/docs/design/domain/agent.md`, 8.4.1;
  `temper/docs/design/client/README.md`, 4.6 (*A project's ways*).
- The pattern to follow: `route/panels.rs` (`approvals`), `route/approvals.rs`,
  and the domain world's `tests/suite/panels.rs`
  (`a_project_changes_watch_delivers_its_document_after_the_commit_barrier`).

## Owns and may change

- **Owns:** `route/ways.rs` and `route/ways/**` (new); the `ways` arms of
  `route/panels.rs` and `route/see.rs`; the marks in `route/kinds.rs`;
  `temper-engine-smith/src/see.rs`'s `SeeAnswer::Ways` arm, its text and
  its `following` arm; the tests above.
- **Neighbour edits, pre-approved:** `route.rs`'s `mod` line; a public
  read of the forge's given every-project rules in
  `temper-engine-domain-forge/src/kinds.rs` (`pub fn every_project_rules(&self)
  -> &[settings::Held]`), its only change there; one line marking the
  subject where the root routes an adoption's or a removal's outcome, in
  `route/forge_route.rs` if that is where it is, named in the report for
  the owner of coding runs; the domain world's fixture giving its config
  `ways_stock` (`temper/tests/engine/domain/src/*.rs`).
- **Options, pre-approved:** where the root hears no committed change of
  jig's own settings or of a role, leave that mark out and say so under
  `## Left`: the page reads afresh at its next open. If the party's offer
  can't be read in the document's decision, take the deployment's offer
  and say so. `derive` is a stub until `ways-derive` lands: test what the
  root gathers, never a path's words.

## Steps

1. Read `approvals`' making, delivery and marks end to end.
2. Write `inputs` and `document`; the subject's arm in `panels.rs`; the
   marks.
3. `see ways`: the address, the answer for the chat's person, the text.
4. The tests; break one rule each (`may_change` for a member; the mark
   after a change) to see them fail, and restore.
5. The status sentence.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-engine-domain -p temper-engine-smith -p
   temper-engine-domain-forge -p temper-engine-domain-world --all-targets
   -- -D warnings`; `heavy cargo nextest run -p temper-engine-domain
   --lib`, `-p temper-engine-smith --lib`, `-p temper-engine-domain-forge
   --lib`, and `-p temper-engine-domain-world --test suite -E
   'test(/ways|panels|configuring/)'`. Commit and push.

## Report

As the implementer's text asks; name each place a mark went, and any
change outside `route/` for the owner of coding runs.
