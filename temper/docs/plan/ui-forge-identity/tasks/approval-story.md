+++
id = "approval-story"
part = "approval"
kind = "feature"
needs = ["box-story", "approval-read"]
files = [
  "temper/tests/engine/domain/tests/suite/forge_routes/approval_identity.rs",
  "temper/tests/engine/domain/tests/suite/configuring.rs",
  "temper/tests/engine/domain/src/forge_routes.rs",
  "temper/docs/design/domain/testing.status.md",
]
effort = "high"
minutes = 60
+++

# The engine's world: a box's person's change waits under Needs you, is approved and lands

## Goal

`temper/docs/design/draft/forge-identity.md`, section 2, steps 3 to 5, through the root and the core on jig's store: on a box whose forge links its person, with their rule accepted (steps 1 and 2, `box-story`'s), a change stepped to its merge waits; the person's `approvals` document holds its row, theirs to give, while a party of another project reads none; the linked user's review approving it at its head is posted on the fake forge; the row goes as the merge is admitted, and the change lands (`temper/docs/design/domain/testing.md`, 6.4, the row *a box's approval waiting*).

Acceptance:

1. One story, `a_box_persons_change_waits_under_needs_you_until_their_review_lands_it`, on the world named below, asserting in order:
   - the change waits: no merge, after the step budget;
   - the person's `approvals` document, watched `Own` as the client watches it (`Watched::Application { scope: ApplicationScope::Own, connector: the forge's, subject: Token::new(engine::APPROVALS_SUBJECT) }`), holds one row: the change's task, its project, forge, owner, name and pull, `required` 1, `counted` 0, `yours` `Counts`, `whole` true;
   - a second party, signed in with a role in another project only, watching the same subject, reads no row;
   - the linked user's review approving at the head is posted and hinted, as `post_approval` does; the merge lands (`world.merges` 1);
   - the person's last document has no row.
2. On each forge the world offers a link on: the owned Forgejo's always; a GitHub forge's, as a second row of a table over the forge, if `box-story`'s world offered one.
3. The entry *6.4, the row *a box's approval waiting*: no engine story...* is gone from `temper/docs/design/domain/testing.status.md`; the 6.3 entry beside it is `approval-joined`'s.
4. `heavy cargo nextest run -p temper-engine-domain-world` passes; name the story's seconds in the report, within the domain world's focused budget.

This task departs from the implementer's text in one way: one status entry is yours to remove (acceptance 3), because the design already says what this task builds.

## Inputs

- `temper/docs/plan/ui-forge-identity/notes/common.md`.
- `temper/docs/plan/ui-forge-identity/notes/contracts.md`, *The link* (`Event::PersonOf`, `PersonAnswer::Is`) and *The approval read* (*The root's part*).
- `temper/docs/design/domain/testing.md`, 6.4, the rows *a box's approval identities* and *a box's approval waiting*: the second binds.
- `temper/docs/design/domain/forge.md`, 8.12, *The root's part*, *Marked*; 8.7, *The panel's approval evidence*.

## Owns and may change

**Owned:** the `files` above: in `approval_identity.rs` the new story and helpers it needs; in `configuring.rs` nothing unless the option below takes it; in `src/forge_routes.rs` the world's collection of the approvals deliveries.

**Readers and neighbours:** `approval_identity.rs`' other stories and their helpers stay as they are; a helper this story shares (`stock_waiting_change`, `post_approval`, `settle`, `configuring_proposal`) keeps its behaviour for them.

**Options, each with its default:**

- **The world.** `box-story` carries steps 1 and 2 in `configuring.rs`, whose world (the configuring fixture's, on a `ForgeStand`) drives scripted chats and has no helper to produce, check and merge a change. `approval_identity.rs`' world has them (`stock_waiting_change`, `stock_produce`, `post_approval`, `approval_panel`) and a box (`box_world`). Default: the story in `approval_identity.rs` on `box_world()`, linked as `box-story` links it (`engine::Event::Forge(Box::new(forge::Event::PersonOf { forge, answer: PersonAnswer::Is(user) }))` for the person's user on the world's forge), the person's rule (`approval_rule(0, 1)`) accepted through `configuring_proposal` and the decision `a_box_refuses_an_unfulfillable_person_approval_rule` sends, then `stock_waiting_change`'s flow. If `box-story` made `configuring.rs`' world produce changes, extend its story there instead and say so.
- **The forge's mode.** If `link-decide` keeps a link only on an owned forge, set the world's forge `Mode::Owned` in this story's world, as `configuring.rs`' `project_people_setup` sets it, one line; say so.
- **The deliveries.** `src/forge_routes.rs`' `World` keeps `pub approvals: Vec<(Token, engine::Approvals)>` beside `panels`, pushed from `Delivery::Approvals { watcher, document }` where `Delivery::Panel` is. A watch helper beside `approval_panel`, `approvals_of(world, watcher) -> engine::Approvals`, the last delivered to that watcher.
- **The row's going.** Default: assert the document after the merge holds no row. If a document between admission and landing is cheaply caught (the merge's commit and its mark in one tick), assert that too.
- **GitHub.** If `box-story`'s table has no GitHub row, leave it out here too and say so.

## Steps

1. The deliveries' collection and the watch helper.
2. The story, its asserts in acceptance 1's order; the second party signed in as `approval_identity.rs`' stories sign one in, with a role in a second project. Bite: drop `APPROVALS_SUBJECT` from `retain_review_wait`'s mark (`temper-engine-domain/src/route/forge_route.rs`), see the row assertion fail; restore.
3. The status entry.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-engine-domain-world`. Commit ("temper: the engine's world has a box's person's change wait under Needs you until their review lands it"), push.

## Report

As the implementer's text says, with the world and forge mode taken, the story's seconds, and any helper shared with another story.

## Answers

2026-11-07 (temper/ui). Yes: add the box setup builder in forge_routes.rs, setting under_box=true and Mode::Owned before bootstrap, and rebuilding bootstrap and root consistently. temper/ui widens your files for it. It's the smaller of the two, and a box's person is what the story is about. Don't extend configuring.rs's world. Keep the existing configurations as they are for the other stories, and add the builder as one more, so no other story's world changes.

2026-11-07 (temper/ui). Yes: extend this task to repair the merge-wait rollback, and temper/ui widens your files to temper-engine-domain-forge's domain.rs and its change tests. Your reading is right, and better than mine: veto_change must restore the state the change was in when the merge was proposed, so that a merge waiting on an approval stays First and owns its turn. It shouldn't jump back to the prior captured before decide_step's progression. A failing regression first, in the forge's existing change tests, as a row: a change that advances Queued to First and proposes its merge in one step, vetoed for an approval, stays First. Don't broaden approval_waits to Queued: I'm withdrawing that answer from approval-joined. Then your story, unchanged.
