+++
id = "strict-checker"
part = "checker"
kind = "feature"
needs = []
files = ["tools/citations/citations.py", "tools/citations/test_citations.py", "tools/citations/README.md", "tools/citations/allow.txt"]
effort = "high"
minutes = 50
+++
# The citation checker excuses hearth's documents alone

## Goal

`tools/citations/citations.py` excuses citations of another repository's documents by phrase
("hearth's", and two phrasings of a second repository), and so hides broken citations. Make it
excuse hearth's alone and refuse a citation that names any other repository, whatever local
file it would resolve to. Make it read the root `DESIGN.md` and the Markdown under `tools/`, and
leave out every file under a tree's `docs/plan/`. Its README says so; `allow.txt` lists the
README's own examples and drops the records kept outside the repository.

Acceptance:

- `python3 -I -B tools/citations/test_citations.py` passes, with the tests of step C5; every
  other existing test is unchanged and green.
- After your last commit, `python3 -I -B tools/citations/citations.py --all` on your branch
  prints none of the 13 `strict-checker` findings in `notes/findings.md`, and none that the
  file lacks: at most the 16 `outward-wording` findings, fewer where that task has merged into
  your base, and 0 when it has.
- The command line, the output's form and the exit codes stay as they are; `--base` compares
  as it does (fab's merge queue runs it, and counts `--all`).
- No line of the checker, its tests, its README or `allow.txt` names the second repository:
  the examples use made-up names (`the probe repository's`).

## Inputs

- `temper/docs/plan/outward-citations/notes/common.md`, then `notes/findings.md`.
- `temper/docs/plan/outward-citations/inputs/request.md`, items 1, 2 and 4: the request.
- `tools/citations/README.md` (whole) and in `citations.py`: `EXTERNAL`, `walk`, `resolve` and
  `suffix`, the span loop of `check_file`, `allowed` and `keys`.
- Measured at `4c830578a7` (the plan's base): with C1 to C3 as sketched, `--all` reads 23 (the
  13 README findings and the first 10 `outward-wording` ones); with the evidence block also out
  of `allow.txt`, 29. C2's form fires on three lines only (`notes/findings.md`, last section).

## Owns and may change

- Owns `tools/citations/citations.py`, `test_citations.py`, `README.md` and `allow.txt`.
- Not yours: every file outside `tools/citations/`, and in it the text of each
  `outward-wording` finding: leave it as it is, even where `--all` prints it.
- Inside the sketches the code is yours: helper names, where a check sits, how `walk` tests a
  path.
- Options, pre-approved:
  - **A README example the checker reads** (C6): by default an `allow.txt` entry for it. If an
    entry would have to excuse more than the example (a section-self target that names a real
    document beside the README, say), rephrase that example instead so the checker does not
    read it as a citation, keeping what it teaches, and say which in the report.
  - **A finding the list lacks**, at your base, in a file C3 now reads: inside
    `tools/citations/`, it is yours to fix; elsewhere, leave the file, and list the finding
    under `## Left` with its line and target.
  - **C2 fires on a phrase that is not another repository's** at your base ("a repository's
    (`x.md`, 4)" and the like): add its first word to the excluded words, with a test for it.

## Steps

1. **C1, hearth's alone.** `EXTERNAL` keeps only its hearth branch:

   ```python
   # hearth's documents are the one other repository's left unchecked ("hearth's `domain/store.md`").
   EXTERNAL = re.compile(r"(?i)\bhearth['’]s(?:,?\s+[\w-]+){0,2}\s*[`\[(]*$")
   ```

2. **C2, any other repository's refused.** A path straight after "<name> repository's" (an
   opening backtick, bracket or parenthesis between at most), where the name is no article or
   determiner and not hearth, is a finding of kind `path` with detail `outside`, never
   resolved: `resolve` falls back by suffix (`suffix`) and would otherwise accept a local file
   of the same name. Say so in the comment:

   ```python
   # A citation of any other repository's document ("the probe repository's `docs/a.md`") is
   # refused: never resolved, since resolve() would find a same-named file here by suffix.
   OUTSIDE = re.compile(
       r"(?i)\b(?!(?:the|a|an|this|that|its|their|each|every|no|one|our|whose|which|any|same|hearth)\b)"
       r"[\w-]+\s+repository['’]s\s*[`\[(]*$")
   ```

   In `check_file`'s span loop, right after the `EXTERNAL` check:

   ```python
           if OUTSIDE.search(before[-50:]):
               out.append(("path", rel, ln, tok, "outside", ctx))
               continue
   ```

3. **C3, what is read.** In `walk`, when not `every_md`: a file under a tree's `docs/plan/` is
   left out whatever its extension (today its `.md` files are, while `plan.toml`'s comments are
   read); the root `DESIGN.md` and every `.md` under `tools/` are read. The Markdown index
   (`every_md`, the targets `suffix` resolves against) is unchanged.
4. **`allow.txt`.** Remove the block headed `# Evidence outside the repository.` (its heading
   and its six entries: the ladder walks' reports, the QA brief, the two surveys and the
   investigation); `outward-wording` drops the citations they excused. Keep every other line.
5. **C5, tests,** in `test_citations.py`'s small repositories:

   ```python
   # replaces test_other_repositories_dates_and_rfcs_are_not_citations, which asserts that a
   # second repository's citation passes
   def test_hearths_documents_dates_and_rfcs_are_not_citations(self): ...
       # hearth's `domain/store.md`, 5.2; `a.md`, 2026-10-05; RFC 6749, section 5.1: exit 0
   def test_another_repositorys_document_is_refused(self): ...
       # "As the probe repository's `docs/design/a.md`, 2.1, says." in c.md: exit 1 with
       # f"{DESIGN}/c.md:5: path docs/design/a.md (outside)", though skein/docs/design/a.md
       # exists and resolves by suffix
   def test_a_possessive_is_no_exception(self): ...
       # "As the probe's `docs/design/gone.md`, 4, says.": exit 1, path docs/design/gone.md
   def test_a_determiners_repository_is_this_one(self): ...
       # "A gate is a repository's (`a.md`, 2.1); the repository's `a.md`, 9, is gone.":
       # exit 1 with section {DESIGN}/a.md 9, and no "(outside)" in the output
   def test_design_md_and_tools_markdown_are_read(self): ...
       # the root DESIGN.md and tools/t/README.md, each "See `gone.md`.": exit 1, both named
   # test_plans_are_not_read also commits skein/docs/plan/p/plan.toml with the comment
   # "# Deletes `../../design/gone.md`." and still exits 0
   ```

6. **README.** Its "What is read" bullet names the root `DESIGN.md` and the Markdown under
   `tools/`, and leaves out "everything under a tree's `docs/plan/`, whatever its extension"
   (fab's records of a change). Its "Not citations" bullet loses its "another repository's
   documents" clause, with the example that names the second repository; a new bullet before
   it says:

   > - **Another repository's documents:** hearth's ("hearth's `domain/store.md`") are left
   >   unchecked. A citation of any other repository's document ("the probe repository's
   >   `docs/a.md`") is refused: a finding (`outside`) even where a file of that name exists
   >   here.

   Keep the rest of the README's text, its rules and their examples as they are.
7. **C6, the README's examples.** Commit, then run `--all`: the README's examples of the forms
   it reads (`x.md`, `../../a.md`, the sections they quote, `docs/a.md`) are its findings now.
   List each in a block appended after the last line of `allow.txt`, scoped to the README and
   with the target exactly as the checker prints it, never `*`:

   ```text
   # The citation README's examples of the forms it describes.
   tools/citations/README.md x.md # an example of a citation form, not a document
   tools/citations/README.md docs/a.md # an example of a refused citation
   ...
   ```

8. Run the tests, commit, run `--all` (acceptance), push.

## Report

`notes/common.md`'s report; each new test's name; the `allow.txt` entries you added, and any
example you rephrased instead; the findings `--all` printed after your change, by owner task;
anything under `## Left`.
