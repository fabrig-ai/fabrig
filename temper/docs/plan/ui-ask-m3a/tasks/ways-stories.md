+++
id = "ways-stories"
part = "stories"
kind = "feature"
needs = ["ways-derive", "stock-ways", "ways-view", "ways-family", "client-ways", "ways-delegation"]
files = ["temper/tests/engine/domain/tests/suite/ways.rs", "temper/tests/client/tests/suite/ways.rs", "temper/tests/client/tests/suite/main.rs", "temper/tests/client/src/**", "temper/docs/design/domain/forge.status.md", "temper/docs/design/protocol/configuration.status.md", "temper/docs/design/client/forge.status.md"]
effort = "high"
minutes = 150
+++

## Goal

Prove the ways end to end, as `temper/docs/design/draft/stock-ways.md`,
section 8 asks of this step: the three ways' paths over a project with CI
and both tiers, and with neither; `ways`'s current way and "Yours"; "Use
this" refused for too few approvers; "Keep it"'s change marking To begin's
step done. Then close the design's last **Not yet** entries of the plan.

Acceptance:

- The engine's domain world (`tests/suite/ways.rs`, beside `ways-read`'s
  and `ways-view`'s rows), its world given the stock's ways:
  - a project whose home repository has CI, a tier following main and a
    tier following goal branches: the `ways` document's three paths, as
    words in order, are `ways-derive`'s table's first three rows
    (Agents review and You approve ending "merge, verify, main"; Draft
    first "draft, plan, produce ×n, merge, check, repair, goal branch,
    check, update, resolve, merge, verify, main"), and `current` is Agents
    review's index;
  - a project whose repository has neither: the same less every "check"
    of CI and the tiers' stops, as `ways-derive`'s second rows;
  - "Yours": the owner's `ChangeSettings` of the project's rule 1 without
    CI makes `current` none, `own` a path with no "check";
  - `see ways` for that chat's person ends its first way block with the
    current way marked `now`, and, for the "Yours" project, its last
    block `Yours · now`.
- The joined client world (`temper/tests/client`, a new
  `tests/suite/ways.rs` beside `settings.rs`'s fixture):
  - "Keep it": an owner opens `Ask { project }` on a fresh project; the
    page's `StockWays` has Agents review current with `Keep`; the act
    commits; Home's To begin has "Tell temper how you work" done; the next
    document keeps Agents review current and offers no `Keep`;
  - "Use this" on Draft first: the act commits, the next document has
    Draft first current, and the project's kind 7 has goals on and way 4;
  - "Use this" on You approve refused: in a world where the owner is no
    known person on the repository's forge (as
    `temper/tests/engine/domain/tests/suite/forge_routes/approval_identity.rs`'s
    box has one), the refusal kept on `StockWays` names the forge, role
    0, one required and none eligible, and the document is unchanged;
  - a member sees the ways and no act.
- Status: `temper/docs/design/domain/forge.status.md`'s 8.13 **Not yet**,
  `temper/docs/design/protocol/configuration.status.md`'s 13.10 **Not
  yet** and `temper/docs/design/client/forge.status.md`'s section 9
  **Not yet** go, each where what it names is now built; anything still
  short is said in place of it, in the same words' form.

## Inputs

- `temper/docs/plan/ui-ask-m3a/notes/common.md`; `notes/contracts.md`.
- `temper/docs/design/draft/stock-ways.md`, section 8;
  `temper/docs/design/protocol/configuration.md`, 13.10;
  `temper/docs/design/client/README.md`, 5.13 and 3.5 (*To begin*);
  `temper/docs/design/domain/forge.md`, 8.13.
- The rows of `ways-derive` (`temper-engine-domain-ways`'s tests), the
  domain world's `tests/suite/ways.rs` (`ways-read`, `ways-view`), the
  client world's `tests/suite/settings.rs` and `asking_chat.rs` for their
  fixtures.

## Owns and may change

- **Owns:** the stories above; the client world's fixture lines they
  need (`temper/tests/client/src/**`), beside its siblings'.
- **Options, pre-approved:** if the client world cannot make an owner
  unknown on its forge, tell "Use this" refused in the domain world
  instead: the owner's `ChangeSettings` of the `ways` document's You
  approve items, refused with those words; say so. If a story shows a
  defect in another task's code, fix it in the lowest crate that shows it,
  name it in the report, and keep the story.

## Steps

1. The domain world's stories; then the client world's.
2. Break one rule each time (current from the second equal path; Keep
   offered after the step is done) to see a story fail, and restore.
3. The status entries.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world -p
   temper-client-world --all-targets -- -D warnings`; `heavy cargo nextest
   run -p temper-engine-domain-world --test suite -E 'test(/ways/)'` and
   `-p temper-client-world --test suite -E 'test(/ways|settings|to_begin/)'`.
   Commit and push.

## Report

As the implementer's text asks; give each story's CPU seconds.
