+++
id = "checker-readings"
part = "checker"
kind = "feature"
needs = []
files = ["tools/citations/citations.py", "tools/citations/test_citations.py", "tools/citations/README.md"]
effort = "high"
minutes = 60
+++
# The citation checker reads nine forms right

## Goal

`tools/citations/citations.py` misreads 37 of today's findings: the cited sections exist, in the
file the sentence names, or the text is no section at all. Teach it the nine forms below (R1 to
R9), each as narrow as its examples, each with a test in `test_citations.py`. A form the checker
stops reporting must be a misreading, never a real break: each rule that names a file still
fails when that file lacks the section.

Acceptance:

- `python3 -I -B tools/citations/test_citations.py` passes, with a test for each rule below
  (the existing tests unchanged and green).
- After your commit, `python3 -I -B tools/citations/citations.py --all` on your branch prints
  none of the 37 findings `notes/findings.md` gives this task, and none that the list lacks:
  the 30 findings of the other tasks, at most, and fewer if their branches have merged into
  your base.
- The command line, the output's form and the exit codes stay as they are: fab's merge queue
  runs `--base` and counts `--all` after landing.

## Inputs

- `temper/docs/plan/citation-burndown/notes/common.md`, then `notes/findings.md`, whose
  "checker-readings" list groups the 37 by rule.
- `tools/citations/README.md` and `citations.py` (`MD_TOKEN`, `SECTION_AFTER`, `OWNER`,
  `BARE_SECTION` and its `pre` check, `status_labels`, `extract`, `Tree.exists`, `resolve`).
- Measured at 17273de9e2 (the plan's base), for R1 and R2: reading paths up any number of
  directories, with R2's possessive held to a comma or "section", drops exactly R1 and R2's
  four findings and reveals none. A looser possessive (any "`x.md`'s N") reveals one false
  finding: `skein/docs/design/forge-github.md:131`, "forge.md's 32 bytes", a count. R4 as
  sketched drops its one finding and reveals none.

## Owns and may change

- Owns `tools/citations/citations.py`, `test_citations.py` and `README.md`.
- Not yours: `tools/citations/allow.txt` (the `allow-and-zero` task's), and every finding of
  another task: leave the text it cites as it is.
- Inside the sketches the code is yours: helper names, where a rule sits, how `Tree` learns the
  commit's tracked paths.
- Options, pre-approved:
  - If a rule, at your base, reveals a finding the list lacks: if it is a misreading, narrow the
    rule until it does not; if it is a real break (the named file lacks the section), keep the
    rule and list it under `## Left` with its file, line and the section it should cite (the
    last task, `allow-and-zero`, takes it).
  - R6 may name a few other plural count nouns beside "places" ("bytes", "times", "lines"), if
    each is tested.

## Steps

1. **R1, paths up the tree.** `MD_TOKEN` reads one leading `../` today, so `../../README.md` is
   never a citation. Read any number of leading `../`.
2. **R2, the possessive and the parenthesis.** Today "`x.md`, 15" and "`x.md` 15" are x.md's.
   Also: "`x.md`'s, section 15", "`x.md`'s, 6.2" and "`x.md`'s section 15" (a possessive
   followed by a comma or by "section"), and "`x.md` (section 10 is ...)" (an opening
   parenthesis then "section"). A possessive followed by a bare number, "forge.md's 32 bytes",
   stays unread. Drops jig `client/domain/README.md:8`, `contract.md:1368`, `testing.md:96`,
   `:421`.
3. **R3, "Its" with a capital.** `BARE_SECTION`'s `pre` check skips "its", "their", "that",
   "this": skip them whatever their case ("Its sections 2, 3, 7, 10, 11 and 12 are stale" opens
   a sentence about the link before it). Drops jig `quality-review.md:21` (3).
4. **R4, "its" after another tree's path.** In "jig's `domain/engine.md`, section 17, and its
   `domain/testing.md`, sections 4 and 9", the second path is jig's: an "its" just before a path
   takes the owner of the nearest earlier "<tree>'s `path`" in the same sentence (back to the
   last ". "). Drops temper `domain/testing.md:2183`.
5. **R5, an RFC's parenthesised section.** "RFC 9500's `testRSA2048` (section 2.1)" is the
   RFC's: beside today's "RFC 6749, section 5.1", a bare section whose `pre` ends in "RFC N's",
   then a code span or at most two words, then "(", is the RFC's. Drops skein
   `oauth-issuer.md:421`.
6. **R6, a count.** "jig's `protocol/store.md`, 21 places" counts: a number after a path that
   is followed by "places" is no section. Drops temper `draft/upgrades.status.md:7`.
7. **R7, step labels.** `reference-case.status.md` says "By step (sections 3 to 7):" before its
   labels: they are the case's steps, not sections. A status file whose text before its first
   label says "By step" has its labels left unchecked; every other status file is checked as
   today. Drops temper `draft/reference-case.status.md`'s 21.
8. **R8, a list item headed by a path.** "- **`../protocol/forge.md`:** 3.2, ...; section 9,
   ...; section 13, ..." and "6. **view.md:** ... the reference UI's state (section 18)": the
   item's sections are its heading path's, through the whole item, past "; " and ". ". When a
   list item opens with a bold path (the path in backticks or bare, then ":"), add that path to
   the targets of every bare section in the item. Drops jig `client/domain/README.md:38` and
   temper `draft/ci-on-forgejo-17.md:140`, `:142`, `:145`.
9. **R9, links to tracked pages.** `extract` archives only `.md` and code, so
   `[first-run.html](first-run.html)` is dead though the page is tracked. A link resolves when
   its path is any tracked file of the commit (`git ls-tree`, which `extract` already lists).
   A link's `?query` is cut as its `#anchor` is. Drops temper `client/mockups/STATUS.md:84`,
   `:119`.
10. **Tests,** in `test_citations.py`'s `Citations`, with its helpers (`write`, `commit`,
    `since_base`, `run_check`; `DESIGN` and `a.md`'s sections 1, 2, 2.1), one or two per rule,
    each with the failing case beside the passing one where the rule names a file:

    ```python
    def test_a_path_two_directories_up_is_read(self):
        self.write(f"{DESIGN}/x/y/c.md", "# c\n\n## 1. One\n\nSee `../../a.md`, 2.1, and `../../a.md`, 7.\n")
        self.commit("head")
        result = self.since_base()
        self.assertEqual(result.returncode, 1)
        self.assertIn(f"section {DESIGN}/a.md 7", result.stdout)
        self.assertNotIn(f"{DESIGN}/a.md 2.1", result.stdout)

    def test_a_possessive_or_parenthesised_section_is_the_cited_files(self): ...
        # "`a.md`'s, section 2", "`a.md` (section 2.1 holds it)", "a.md's 32 bytes" pass;
        # "`a.md`'s, section 9" fails as {DESIGN}/a.md 9
    def test_its_with_a_capital_is_the_cited_files(self): ...
    def test_its_after_another_trees_path_is_that_trees(self): ...
        # jig/docs/design/domain/t.md with sections 4 and 9; temper's c.md cites
        # "jig's `domain/e.md`, section 1, and its `domain/t.md`, sections 4 and 9" (passes),
        # then "sections 4 and 7" (fails as jig/docs/design/domain/t.md 7)
    def test_an_rfcs_parenthesised_section_is_the_rfcs(self): ...
    def test_a_count_after_a_path_is_not_a_section(self): ...
    def test_a_status_file_by_step_labels_steps(self): ...
        # a status file "By step" with "- **14, 22:** ..." passes; one without, "**Not yet:** 14: ...", fails
    def test_a_list_items_heading_path_owns_its_bare_sections(self): ...
        # "- **`a.md`:** 1, the first; section 2, the second." passes; "section 5" fails as a.md's
    def test_a_link_to_a_tracked_page_resolves(self): ...
        # page.html committed: [p](page.html) and [q](page.html?world=x&step=5#part) pass;
        # [g](gone.html) fails as path gone.html
    ```

11. **README:** "What is read" and "Not citations" say what the checker reads now: paths up any
    number of directories, links to any tracked file, counts ("`x.md`, 21 places"), RFC
    sections in either form, a status file "By step".
12. Run the tests, commit, run `citations.py --all` (acceptance), push.

## Report

`notes/common.md`'s report; each rule's test name; the findings `--all` printed after your
change, by owner task; anything under `## Left`.
