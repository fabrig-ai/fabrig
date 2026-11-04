# Every own-rules task

- Read `AGENTS.md` (1, 2) and `DESIGN.md`, 4. This plan renumbers the root `AGENTS.md`'s
  sections (`agents-rules`); `DESIGN.md`, 4's rule that surviving sections keep their numbers
  is for design documents, and `AGENTS.md` is not one: the request (approved by the user)
  renumbers it, and its task fixes every citation of it.
- **Wording, never behaviour.** A changed line is prose, a comment, a test's fixture text or a
  script's header text; no check, count, profile or script changes what it does. Each line
  says what is true now in fabrig's own words (`AGENTS.md`, 2) and names none of the outward
  check's names (the list is the module doc of `tests/boundary/tests/outward.rs`), nor the
  lab's roles: no "director", "lead", "implementer", "the queue", "the pipeline".
- **The exact text is in your spec.** Apply it as given; where a line moved or changed at your
  base, find it by its text and apply the same edit to its current words, and say so. Rewrap
  an edited paragraph or comment to its neighbours' width, nothing outside it.
- **The citation checker** (`tools/citations/citations.py`) reads committed trees through
  `git archive`: commit first, then run `python3 -I -B tools/citations/citations.py --all` and
  `python3 -I -B tools/citations/citations.py --base origin/main` from the slot's root. Both
  build nothing: run them directly. Both read 0 on your branch.
- **Departure from your implementer's text, for both tasks:** you edit the design and
  foundation lines your spec names (`docs/design/`, `docs/foundation/`, one draft's notes),
  those lines only, because the request asks for exactly these edits and the renumbering's
  citations are fixed by whoever renumbers (`DESIGN.md`, 4).
- **Not here:** `.config/`, `tools/tiers/`, every status file (`*.status.md`, `STATUS.md`),
  `tools/citations/citations.py`, `tools/citations/allow.txt`, `tools/citations/test_citations.py`
  (parallel plans own them). A finding or a lab name in one of them is not yours: list it under
  `## Left`.
- **Your report** lists each line you changed (`file:line`, old words, new words, in short),
  the `citations.py --all` count before your change and after it, and each check you ran with
  its count.
