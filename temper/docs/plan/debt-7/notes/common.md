# debt-7: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of your task's tree. Your task stays in its tree, unless its
  `Owns and may change` lists a file of another.
- **The request** is `temper/docs/plan/debt-7/inputs/request.md`.
- **Check the item first.** Read the debt item your spec names, whole, and the design text it cites.
  Line numbers in an item or a spec are at this plan's base: find the code by name.
- **Paying an item.** The branch that pays it makes its **Paid when** hold; a **Measure** reads 0, or
  the item's target, after. It deletes the item's file and its row in that tree's
  `docs/development/debt/README.md`, updates any status entry the item names as closed by it, and
  fixes any citation the deletion breaks. This departs from the implementer's text only for those
  files.
- **Behaviour stays unchanged.** If a test's visible events or outcome change, stop and ask. Don't
  retime a test or rewrite its assertion to make a refactor pass: debt-5 found that a "timing" change
  split an atomic commit in two. A test may change only where it names a moved item (a path, an import,
  a constructor).
- **Stop and ask** on a budget, a stored format, a wire format or a breaking public interface. An
  addition to a public interface is fine. Your spec names what else it pre-approves.
- **A test that pins a rule must be able to fail on that rule.** Break the rule by hand in your slot,
  watch the test go red, put it back: the break is never committed. Your report's table says what was
  broken, which test failed, and what the failure read. A test that only moved needs no break.
- **Keep new code in proportion.** These items exist to remove duplication: each part ends with fewer
  lines than it started. Your report gives the branch's net lines (`git diff --shortstat` against your
  base). Your base has your needs in it: a need's lines are its own task's, so measure from the last
  commit that merged a need into your branch. A task's size cap is a limit: measure it, and stop and
  ask if it's passed. A task that expects to add lines says so in its spec.
- **Acceptance searches are narrow:** code that legitimately uses the same word for something else
  is exempted, not changed; your report names any exemption the spec does not.
- **Not yours:** `docs/design/`, status files and drafts, unless your spec lists one. This departs
  from the implementer's text only for a file the spec lists, and only in the way it lists. A status
  file's form is `DESIGN.md`, 2: its date line is today's (`date -u +%F`), and it holds only the gaps
  as they stand after your change.
- **Citations.** `python3 -I -B tools/citations/citations.py --all` reads 0 at this plan's base and
  stays at 0; run it before you push.
- **Comments say what is true now,** in the design's words: no "no longer", "was", "formerly" for
  something gone, and no debt item's number in code. A sentence about something removed is deleted,
  not reworded into its history.
- **Old-form counts only fall.** If your change lowers a count of your tree's `scripts/old-forms.txt`,
  run `python3 -I -B scripts/old_forms.py --write` from the tree's root and commit the lowered file;
  never raise a count.
- **Goldens** do not move in this plan. If one would, stop and ask.
- One commit per coherent step, each saying why. Push your branch.
