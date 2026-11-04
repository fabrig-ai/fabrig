# Request: pay four debt items, one per tree (fifth round)

From fabrig/debt, 2026-11-04 (the user's continuous tech-debt process; source of truth:
each tree's `docs/development/debt/`). Four independent parts (temper's pays four small items, smith's pays a review's findings), one per tree, sized S to M. Each
part pays its item, so that the item's **Paid when** holds, and nothing else. A part
that needs a design change, a breaking public interface change or a stored-format change stops
and asks (additions to a public interface are fine).

Every part, on its own branch:

- reads the item file whole, and the design text it cites;
- makes the change so the item's **Paid when** holds; if the item has a **Measure**, the
  command reads 0 (or the item's target) after;
- keeps behaviour unchanged unless the item says otherwise; tests that pinned removed code go
  with it, and the rest pass unchanged;
- **deletes the item's file and its row** in that tree's `docs/development/debt/README.md`, in
  the same branch (the README's rule: git keeps the history);
- updates any status entry the item names as closed by it, and any citation the deletion breaks
  (`tools/citations/citations.py --all` stays at 0);
- writes acceptance searches narrowly: code that legitimately uses the same word for something
  else (for example a test that checks an old field is refused) is exempted, not changed.

**All four parts must be in the plan when the cut ends.** debt-2 lost a part left to a designer
still in flight when its cut ended, and the plan landed without it. If a part needs design, finish
that design inside the cut, or ask a question and leave the part in the plan as waiting on it.

Keep new code in proportion. debt-3's state machines added more lines than they removed; a state
enum replaces its flags, and the transitions it needs, not a layer around them.

## 1. jig D001: every root echoes the core's own steps back to it

`jig/docs/development/debt/D001-roots-sequence-core-steps.md`. A root handles only the core's `Now`
values that need the root (a peer to call, a store to load, a payload to keep); none is answered by
pushing a fixed core event straight back. This goes through `examples/ops/domain/src/route.rs`,
`testing/jig-test-domain/src/routes.rs`, and temper's `temper-engine-domain/src/route/now.rs`. This
part should remove more than it adds.

## 2. skein D010: GitHub's dialect stands on Forgejo's modules

`skein/docs/development/debt/D010-github-on-forgejo-modules.md`. No non-test code under
`skein-forge/src/github` names `forgejo::`; what both dialects share sits at skein-forge's root, and
forge-connection no longer translates twin types field by field. This part should remove more than
it adds. Don't touch the fake forges: `bugs-skein` is working in them.

## 3. smith: the spot review of debt-3

`inputs/smith-lifecycles-review.md`, findings 1 to 10. These are follow-ups to debt-3's lifecycle
machines: about 200 lines go, half in tests, with no machine's behaviour changed. Finding 2's
helpers come first. Apply each finding unless the code has since moved on; if one is wrong, skip it
and say why in the report. No debt item closes here.

## 4. temper D007, D008, D009 and D010: four small items

`temper/docs/development/debt/D007-docs-guide-kind.md`, `D008-pipeline-names-in-code.md`,
`D009-interim-account-counts.md` and `D010-path-shared-fixtures.md`, one task each. Each is
independent and sized S.
