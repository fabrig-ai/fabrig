# debt-6: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of your task's tree. Your task stays in its tree.
- **The request** is `temper/docs/plan/debt-6/inputs/request.md`; **the review** is
  `temper/docs/plan/debt-6/inputs/kv-scripted-review.md`. Its line numbers are at `8317051e57`.
  Your spec names the findings you take, and what the cut read in them.
- **Check each finding first.** The review ran nothing. Re-read the code at your base before you
  act on a finding: one that is wrong or has moved on is skipped, and your report says why.
- **A test that pins a rule must be able to fail on that rule.** For each test or story you add or
  change, break the rule by hand in your slot, watch it go red, and put it back: the break is never
  committed. Your report's table says what was broken, which test failed, on which front or in which
  binary, and what the failure read. "It passes" is not the check. Where a spec says a test must
  fail on today's code, write the test first and run it on today's code.
- **Stop and ask** if a change needs a design change, a breaking public interface change or a
  stored-format change. Your spec names what it pre-approves; anything else of that kind is a
  `Question:`.
- **Keep new code in proportion.** A branch that adds more lines than it removes says why in its
  report, unless its spec expects it. Behaviour stays unchanged unless the spec says otherwise.
- **Acceptance searches are narrow:** code that legitimately uses the same word for something else
  is exempted, not changed; your report names any exemption you meet that the spec does not.
- **Not yours:** `docs/design/`, status files and drafts, unless your spec lists one. This departs
  from the implementer's text only for a file the spec lists, and only in the way it lists. A status
  file's form is `DESIGN.md`, 2: its date line is today's (`date -u +%F`), and it holds only the gaps
  as they stand after your change.
- **Citations.** `python3 -I -B tools/citations/citations.py --all` reads 0 at this plan's base and
  stays at 0; run it before you push.
- **Comments say what is true now,** in the design's words: no "no longer", "was", "formerly" for
  something gone, and no review finding's number in code. A sentence about something removed is
  deleted, not reworded into its history.
- **Old-form counts only fall.** If your change lowers a count of your tree's `scripts/old-forms.txt`,
  run `python3 -I -B scripts/old_forms.py --write` from the tree's root and commit the lowered file;
  never raise a count.
- **Goldens** do not move in this plan. If one would, stop and ask.
- One commit per coherent step, each saying why. Push your branch.
