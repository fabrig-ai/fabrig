# debt-11: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of your task's tree. Your task stays in its tree, unless its
  `Owns and may change` lists a file of another.
- **The request** is `temper/docs/plan/debt-11/inputs/request.md`. **The design** is
  `tools/lints/design.md`. **The contracts** are `temper/docs/plan/debt-11/notes/contracts.md`: the
  definitions, which package takes which profile, which `clippy.toml` is which, and each level that changes.
- **Behaviour stays unchanged.** Lints move, and code only as far as a lint asks. If a test's visible
  events or outcome change, stop and ask. Don't retime a test or rewrite an assertion to make a refactor
  pass. A panic message may change where `unwrap()` becomes `expect("...")`.
- **Stop and ask** on a budget, a stored format, a wire format or a breaking public interface. An addition
  is fine.
- **No lint moves by accident.** A level or a ban moves only as the contracts' sections 3 and 4 say. Your
  report lists each one you saw move, and anything those sections lack. `tools/lints/profiles/` and
  `tools/lints/clippy/` are `lint-check`'s: no later task changes them. A crate that can't take its profile
  or its variant takes scoped exceptions; past 40 in one crate, stop and ask, with the count and the lints.
- **What a move turns red, in this order:**
  1. An `#[expect]` of a lint now allowed, or of a ban now gone, is unfulfilled: delete it, or that lint from
     its list. A reason that spoke of the lint goes with it.
  2. What a lint can fix by itself with no change of meaning: fix it (`unwrap()` to `expect("why")`, a
     hidden lifetime, a doc's backticks, a pedantic rewrite clippy suggests).
  3. `#[allow]` becomes `#[expect(..., reason = "...")]`. A `disallowed_*` lint is never allowed in a manifest.
  4. Anything else is a real exception: a scoped `#[expect(lint, reason = "...")]` on the smallest item that
     holds it, saying why in the design's words (`tools/lints/design.md`, 6). This is the one kind of
     exception there is; a task never edits a block by hand.
- **A wildcard arm stays.** Deleting an `#[expect(clippy::wildcard_enum_match_arm, ...)]` leaves the match
  as it is. Don't rewrite a test's exhaustive match into `_`: this plan moves no test code for its own sake.
- **Keep new code in proportion.** A task's size cap is the cut's estimate, not a user budget. Your report
  gives the branch's net lines (`git diff --shortstat` against your base). Compact first. If a cap still
  can't be met without losing wanted behaviour, stop and ask, stating the measured size and why: an answer
  call may raise it by up to 25%. Beyond that the user decides.
- **Acceptance searches are narrow:** code that legitimately uses the same word for something else is
  exempted, not changed; your report names any exemption the spec does not.
- **Never pin an exact HEAD:** fab merges the trunk in at each run's start. Line numbers and counts in a
  spec are at this plan's base: find the code by name.
- **Not yours:** `docs/design/`, status files and drafts, unless your spec lists one. This departs from the
  implementer's text only for a file the spec lists, and only in the way it lists. The debt items' files and
  README rows are `lint-gate`'s: every item cites the others, so one branch deletes all four.
- **Citations.** `python3 -I -B tools/citations/citations.py --all` reads 0 at this plan's base and stays at
  0; run it before you push. A manifest's block and a `clippy.toml` cite `tools/lints/design.md`.
- **Comments say what is true now,** in the design's words: no "no longer", "was", "formerly" for something
  gone, and no debt item's number in code.
- **Old-form counts only fall.** If your change lowers a count of your tree's `scripts/old-forms.txt`, run
  `python3 -I -B scripts/old_forms.py --write` from the tree's root and commit the lowered file.
- **The wasm32 clippy.** A package in `.config/gate/wasm32.txt` that you change also runs
  `heavy cargo clippy --target wasm32-unknown-unknown -p PKG -- -D warnings`.
- One commit per coherent step, each saying why. Push your branch.
