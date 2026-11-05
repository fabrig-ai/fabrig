# debt-16: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of your task's tree. Your task stays in its tree, unless its
  `Owns and may change` lists a file of another.
- **The request** is `temper/docs/plan/debt-16/inputs/request.md`; the findings are
  `temper/docs/plan/debt-16/inputs/debt13-review.md`, read and not run: its line numbers are at an older merge, and your
  spec is what binds. Read the debt item your task pays or rewrites whole, and the design text it cites.
- **The shapes that cross tasks** are `temper/docs/plan/debt-16/notes/contracts.md` where your spec cites it, binding.
  Inside them the choices are yours.
- **Behaviour stays unchanged.** If a test's visible events or outcome change, stop and ask. Don't retime a test or
  rewrite an assertion to make a refactor pass. A test may change how it reaches the code (a call's spelling, a fixture,
  a `#[cfg(test)]` helper, its file); what it asserts stays, and a task names any exception.
- **Stop and ask** on a budget, a timeout, a stored format, a wire format or a breaking public interface beyond what your
  spec says. An addition is fine.
- **A test that pins a rule must be able to fail on that rule.** Break the rule by hand, watch the test go red, restore
  it, and report what you broke and what failed.
- **Keep new code in proportion.** A size cap in a task is the cut's estimate, not a user budget. Your report gives the
  branch's net lines (`git diff --shortstat` against your base), production and tests apart, counting inline
  `#[cfg(test)]` items as tests: a test moved out of a production file is not a production saving. Compact first. If a
  cap still can't be met without losing wanted behaviour, stop and ask, stating the measured size and why: an answer call
  may raise it by up to 25%. Beyond that, the plan's owner (fabrig/debt) decides. Budgets and timeouts stay the user's.
- **Acceptance searches are narrow:** code that legitimately uses the same word for something else is exempted, not
  changed; your report names any exemption the spec does not.
- **Never pin an exact HEAD:** fab merges the trunk in at each run's start. Line numbers and counts in a spec are at this
  plan's base: find the code by name.
- **Not yours:** `docs/design/`, status files and drafts, unless your spec lists one. This departs from the implementer's
  text only for a file the spec lists, and only in the way it lists. The sections your task builds are already written.
- **A debt item you pay is deleted in your branch.** An item your spec only rewrites keeps its file.
- **Citations.** `python3 -I -B tools/citations/citations.py --all` reads 0 at this plan's base and stays at 0; it reads
  committed trees, so run it after you commit and before you push.
- **Public files name nothing of who runs fabrig's development** (`tests/boundary/tests/outward.rs`): no lab tool,
  machine, session or repository in code, a README or a comment.
- **Comments say what is true now,** in the design's words: no "no longer", "was", "formerly" for something gone, and no
  debt item's or review finding's number in code.
- **Old-form counts only fall.** If your change lowers a count of your tree's `scripts/old-forms.txt`, run
  `python3 -I -B scripts/old_forms.py --write` from the tree's root and commit the lowered file.
- One commit per coherent step, each saying why. Push your branch.
