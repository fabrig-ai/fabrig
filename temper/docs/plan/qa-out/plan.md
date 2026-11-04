# qa-out: plan

Request (`inputs/request.md`, step 10 of 12 of a user-approved plan): fabrig keeps what temper
promises, what an agent driving it gets and v0.1.0's acceptance stories, and nothing of who runs
its development. The lab's runbook, kits, briefs and backlogs leave. Documents only.

## Parts and tasks

All in slice 1, released together in the first push. The request's sections 1 and 2 are one task,
section 3 another, section 4 the third.

- **qa-page:** `tasks/qa-page.md`, no needs, ~40 min. `temper/docs/development/qa.md` rewritten by
  section, its eight headings kept, section 3 in generic words; `temper/docs/design/qa.md` a
  one-line pointer; `qa.md`'s line out of `tests/boundary/outward.txt`.
- **acceptance:** `tasks/acceptance.md`, no needs, ~90 min. `temper/docs/development/acceptance.md`,
  twelve sections, eight stories; the v0.1.0 draft and the design's other readers of "the brief"
  cite it.
- **plans-out:** `tasks/plans-out.md`, needs both, ~15 min, medium. The two backlogs and two plan
  directories deleted, their inventory lines dropped, `qa.md` links `acceptance.md`.

**Critical path:** acceptance (~90 min) → plans-out (~15), about 1.75 hours of implementer time plus
fab's merges; qa-page runs beside acceptance.

## Decisions

- **Three tasks, the first two side by side.** Neither builds on the other: qa.md's section numbers
  stay, so the stories and the draft can cite them now (`notes/contracts.md`, C1). The one thing
  that needs both, qa.md's link to `acceptance.md`, goes in plans-out; qa-page cannot name a page
  not yet on the trunk without breaking the citations check.
- **The specs give the replacement text.** Most of this is words the user will read, not choices
  an implementer should make: qa.md's section 3, the draft's passages and the design's readers
  are quoted whole, and the brief-to-page translation is a table.
- **qa.md's section 3 drops every figure of the lab's, keeping the mechanism:** the model's efforts
  and the first-start rule, the room's two override names and why, the budget's `settings apply`
  and `project fund` order with `PERIOD` and `POOL` for the figures. The rungs stay as pairs of
  backends, because `agent-docs.md` cites section 3 for them. Their gating goes. A kit outside the
  repository that took figures from section 3 now keeps its own; see the report.
- **Beyond the request's list: four readers of "the brief" move to the page.**
  `temper/docs/design/domain/testing.md` (2.4 and 2.7), `temper/docs/design/release.md` (6) and
  `temper/docs/design/draft/reference-case.execution.md` (1, 2, 6) lean on a rung's brief with no
  path, so no check finds them, and each would be left pointing at nothing. testing.md's "Its Musts
  are the brief's" is a contract: its target has to stand in the repository. These are one-phrase
  edits with the meaning unchanged, inside the request's "any citation of them".
- **The implementer's "not yours" departs for named passages only** (`notes/common.md`): the
  request asks for edits to a draft and to `temper/docs/design/qa.md`, and the readers above must
  cite a page that does not exist until the same task writes it, so a lead's own design edit
  ahead of the specs would break the citations check.
- **`rung.rs` stays.** Its comment cites qa.md, 3 for the model and room recipe, which section 3 keeps,
  and fabrig-scale is changing the file.
- **Stories hold what a user does and sees.** The briefs' sitting (wall cap, evidence, report's
  shape) stays with the lab. Their phases stay, in a sentence, since testing.md 2.7 takes a live
  story's bounds from them. The GitHub and Claude mapping and the GitHub gate come from the kits,
  because `v0.1.0.md`, 9 holds every rung on those pairs too.
- **`temper/docs/plan/` records stay.** `retired-plans-out`, `tiers-wrapper` and
  `live-smith-reds-3` name removed paths: pushed specs and plans never change, and `docs/plan/` is
  outside both checks.

## Overlaps with drafts beside this one

- **v0.1.0-ladder (queue):** plans-out deletes that plan's own directory and backlog, as the request
  says. It runs last. It could not be avoided, and the source settles it. This plan's other work
  touches none of its files.
- **reference-walks (queue, a plan though not listed beside this one):** the same.
- **fabrig-scale (integration):** its `init-standard` changes `rung.rs`, which this plan leaves alone.
  No file overlaps: `git diff` from the merge base shows nothing in `temper/docs/development/qa.md`,
  the draft, testing.md or release.md.
- **debt-4 (integration):** it removes files under `temper/docs/development/debt/`; none of ours.
- **sign-in-wait (queue):** nothing shared.
