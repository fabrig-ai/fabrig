# debt-14: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of your task's tree. Your task stays in its tree, unless its
  `Owns and may change` lists a file of another.
- **The request** is `temper/docs/plan/debt-14/inputs/request.md`. Read the debt item your task pays whole, and
  the design text it cites.
- **Paying an item:** when its **Paid when** holds, and its **Measure** reads 0 or its target, the branch deletes
  the item's file, unless your spec says the item stays.
- **Behaviour stays unchanged.** If a test's visible events or outcome change, stop and ask. Don't retime a
  test or rewrite an assertion to make a refactor pass. A test may change how it reaches the code (a call's
  spelling, a fixture, a `#[cfg(test)]` helper); what it asserts stays, and a task names any exception.
- **Stop and ask** on a budget, a timeout, a stored format, a wire format or a breaking public interface. An
  addition is fine.
- **hearth's memory migration runs beside this plan and owns D001 to D003.** Leave `hearth/tests/*/tests/suite/memory.rs`,
  `hearth/crates/hearth-service-builder/src/lib.rs` and the machines' `MAX_OUT` forms alone. If your task
  needs one of those, stop and ask, unless your spec says what to do instead.
- **A test that pins a rule must be able to fail on that rule.** Break the rule by hand, watch the test go
  red, restore it, and report what you broke and what failed.
- **Keep new code in proportion.** A size cap in a task is the cut's estimate, not a user budget. Your report
  gives the branch's net lines (`git diff --shortstat` against your base, production and tests apart).
  Compact first. If a cap still can't be met without losing wanted behaviour, stop and ask, stating the
  measured size and why: an answer call may raise it by up to 25%. Beyond that the user decides.
- **Acceptance searches are narrow:** code that legitimately uses the same word for something else is
  exempted, not changed; your report names any exemption the spec does not.
- **Never pin an exact HEAD:** fab merges the trunk in at each run's start. Line numbers and counts in a spec
  are at this plan's base: find the code by name, and take a count from the file as you find it.
- **Not yours:** `docs/design/`, status files and drafts, unless your spec lists one. This departs from the
  implementer's text only for a file the spec lists, and only in the way it lists.
- **Citations.** `python3 -I -B tools/citations/citations.py --all` reads 0 at this plan's base and stays at
  0; it reads committed trees, so run it after you commit and before you push.
- **Public files name nothing of who runs fabrig's development** (`tests/boundary/tests/outward.rs`): no lab
  tool, machine, session or repository in code, a README or a comment.
- **Comments say what is true now,** in the design's words: no "no longer", "was", "formerly" for something
  gone, and no debt item's or review finding's number in code.
- **Old-form counts only fall.** If your change lowers a count of your tree's `scripts/old-forms.txt`, run
  `python3 -I -B scripts/old_forms.py --write` from the tree's root and commit the lowered file.
- One commit per coherent step, each saying why. Push your branch.
