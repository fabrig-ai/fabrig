# debt-22: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of your task's tree. Your task stays in its tree, unless its
  `Owns and may change` lists a file of another.
- **The request** is `temper/docs/plan/debt-22/inputs/request.md`. Read the debt item or review finding your task pays
  whole, and the design text it cites.
- **Behaviour stays unchanged.** If a test's visible events or outcome change, stop and ask. Don't retime a test or
  rewrite an assertion to make a refactor pass. A test may change how it reaches the code (a call's spelling, a fixture,
  its file); what it asserts stays, and a task names any exception.
  A size pin that shrinks because of the task's own change is the one standing exception (fabrig/debt, 2026-11-05): update
  it and say so in your report. A pin that grows, or a visible event or refusal that changes, still stops and asks.
- **Stop and ask** on a budget, a timeout, a stored format, a wire format or a breaking public interface beyond what your
  spec says. An addition is fine.
- **A test that pins a rule must be able to fail on that rule.** Break the rule by hand, watch the test go red, restore
  it, and report what you broke and what failed. New tests earn their lines: a table or a helper, not copies.
- **A table keeps the rows that can still fail.** A row that copies one production arm, one line per variant, adds no
  rule; a derived column goes. Coverage of a rule goes only if your spec says so.
- **Keep new code in proportion.** A size cap in a task is the cut's estimate, not a user budget. Your report gives the
  branch's net lines (`git diff --shortstat` against your base), production and tests apart, counting inline
  `#[cfg(test)]` items as tests and generated code apart. Compact first. If a cap still can't be met without losing
  wanted behaviour, stop and ask, stating the measured size and why: an answer call may raise it by up to 25%. Beyond
  that, the plan's owner (fabrig/debt) decides. Budgets and timeouts stay the user's.
- **Acceptance searches are narrow:** code that legitimately uses the same word for something else is exempted, not
  changed; your report names any exemption the spec does not.
- **Never pin an exact HEAD:** fab merges the trunk in at each run's start. Line numbers and counts in a spec are at this
  plan's base: find the code by name, and re-measure before you start. An item already paid is deleted, saying what paid
  it. A finding that turns out wrong is skipped, saying why.
- **Not yours:** `docs/design/`, status files and drafts, unless your spec lists one. This departs from the implementer's
  text only for a file the spec lists, and only in the way it lists.
- **A debt item you pay is deleted in your branch,** with any status entry your spec names as closed by it and any
  citation that breaks.
- **Citations.** `python3 -I -B tools/citations/citations.py --all` reads 0 at this plan's base and stays at 0; it reads
  committed trees, so run it after you commit and before you push.
- **Stay off:** every temper client view, `temper.css` and `jig-client-view`: other plans are changing them.
- **Public files name nothing of who runs fabrig's development** (`tests/boundary/tests/outward.rs`): no lab tool,
  machine, session or repository in code, a README or a comment.
- **Comments say what is true now,** in the design's words: no "no longer", "was", "formerly" for something gone, and no
  debt item's or review finding's number in code.
