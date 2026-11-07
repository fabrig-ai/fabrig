+++
id = "approval-joined"
part = "approval"
kind = "feature"
needs = ["approval-read", "approval-card"]
files = [
  "temper/tests/client/tests/suite/change_panel.rs",
  "temper/tests/client/src/fixtures.rs",
  "temper/tests/client/src/front.rs",
  "temper/tests/engine/domain/src/forge_stand.rs",
  "temper/docs/design/domain/testing.status.md",
]
effort = "high"
minutes = 60
+++

# The joint world: an approval's card and its link, from the engine's read to the person's page

## Goal

The meeting point of the approval part: temper's client joined to temper's engine root, a change waiting at its merge on a review on the forge shows on the person's *Needs you* as its card, with "Review on Forgejo ↗" at the pull request's files page, and goes once their review is posted and the merge admitted (`temper/docs/design/domain/testing.md`, 6.3, *A change waiting on a review on the forge*). This is where the card and its link are checked whole, in place of a browser step: no browser story reaches such a change (same).

Acceptance:

1. The story `a_change_handed_its_pushed_branch_lands_after_ci_and_its_gate` (`temper/tests/client/tests/suite/change_panel.rs`) runs with a landing rule that also asks for one approval by the person's role in the project: after its gate's verdict, the change waits (the pull not merged after the step budget); then, on the person's `Address::Inbox { filter: Filter::ALL }`:
   - the card: the heading "Approve TITLE on Forgejo", TITLE the change's; "an approval" on its mono line; the rule's sentence; and the link "Review on Forgejo ↗" whose `external` is the pull request's files page (`ROOT/OWNER/NAME/pulls/N/files`, N the opened pull's number), as `drawn(..).external` reads "View on forge ↗"'s; then "Open" and "Later";
   - a review by the person's forge user (7 on forge 1, as `front.rs` binds them) approving at the head is posted on the forge stand and hinted;
   - the change lands as the story already asserts, and the card is gone from *Needs you* (`Step::Gone` within the world's wait).
2. The story's other assertions hold, a panel's approval line among them where the rule adds one.
3. The entry *6.3, *A change waiting on a review on the forge*: the joint world draws no approval's card or its link.* is gone from `temper/docs/design/domain/testing.status.md`.
4. `heavy cargo nextest run -p temper-client-world` passes, within temper's focused budget: name the story's seconds before and after in the report.

This task departs from the implementer's text in one way: one status entry is yours to remove (acceptance 3), because the design already says what this task builds.

## Inputs

- `temper/docs/plan/ui-forge-identity/notes/common.md`: focused alone; tests stay cheap.
- `temper/docs/design/domain/testing.md`, 6.3: *The joint world is the client's acceptance* and *A change waiting on a review on the forge*, which binds.
- `temper/docs/design/client/forge.md`, 4.1: the card's words, acts and address.
- `temper/docs/design/domain/forge.md`, 8.7 and 8.12.

## Owns and may change

**Owned:** the `files` above: in `change_panel.rs` this story and its helpers; in `fixtures.rs` and `front.rs` only what puts the rule in place; in `forge_stand.rs` a review by a user, nothing else (`approval-story` writes `src/forge_routes.rs` in that crate, a file apart).

**Readers and neighbours:** the other stories of `change_panel.rs` and of the joint world that share `Joint::with_change_landing`, `adopt_change_repository` or `goal_with_pushed_change` keep their behaviour: a rule added for this story is this story's alone (a parameter, or a second constructor), named in the report.

**Options, each with its default:**

- **The rule:** default, the deployment's landing rule of the story's world gains the approval (`approvals: [RuleApproval { role, people: 1, freshness: Clean }]`, `enforce` as the stock rule has it), in its configuration before the first start, through a second constructor beside `Joint::with_change_landing`. If the world takes settings only through a change, send it as `tests/suite/settings.rs`' stories do, before the change, and say so. The role is the person's (`fixtures::PARTY`'s) in the project.
- **The review:** `ForgeStand::review(&mut self, repository: u32, number: u64, user: u64, verdict: api::Verdict, commit: api::Commit)` posting through its fake forge's domain as a person, not as temper's writer, and the hint the engine takes for a pull's change, as the engine world's `post_approval` sends it.
- **Extend or beside:** default, extend the story. If the approval's wait changes `panels_landed`' or `quiet`'s expectations past the panel's approval line and its counted review (an order of panels, a callback count), make it a story of its own beside it, sharing the fixture and the helpers, and give both stories' seconds.
- **The second page:** the person's client opens `Address::Inbox` for the card and goes back to the task page for the story's remaining assertions, or a second client of the same party watches *Needs you*; default the first, if `world.person` steps make it one `Step::Go` each way.

## Steps

1. The rule's constructor, the stand's review.
2. The story's steps after the gate's verdict: the wait; *Needs you*'s card and its link's `external`; the review posted and hinted; the steps to landing; the card gone.
3. Bite: have `temper-service`'s `family::approvals` leave each row's `address` empty, see the link assertion fail (no primary drawn); restore. Then drop `APPROVALS_SUBJECT` from the forge's drop at `Committed` (`approval-waits`), see the card-gone step fail, or say why it still passes (a later mark); restore.
4. The status entry.
5. `cargo fmt`; `heavy cargo clippy -p temper-client-world -p temper-engine-domain-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-world`. Commit ("temper: the joint world shows an approval's card and its link until the review lands the change"), push.

## Report

As the implementer's text says, with the rule broken for the bite and the failure seen, the options taken, and the story's seconds.
