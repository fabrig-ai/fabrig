+++
id = "skein-forge-readme"
part = "skein"
kind = "docs"
needs = []
files = ["skein/docs/design/forge-github.md"]
effort = "medium"
minutes = 10
+++
# forge-github.md 16 names the live fixtures' README from the root

## Goal

`skein/docs/design/forge-github.md`, section 16, cites `tests/forge/github/README.md`; the file
is `skein/tests/forge/github/README.md`. Write the path from the repository's root.

Acceptance:

- After your commit, `python3 -I -B tools/citations/citations.py --all` no longer prints
  `skein/docs/design/forge-github.md:927: path tests/forge/github/README.md`, and prints none
  that `notes/findings.md` lacks.
- The test that reads `forge-github.md` passes, one test selected:
  `heavy cargo nextest run -p github-conformance --lib -E 'test(=tests::github_conformance_names_every_fact_of_forge_github_13)'`.

## Inputs

- `temper/docs/plan/citation-burndown/notes/common.md`, then `notes/findings.md`.
- `skein/docs/design/forge-github.md`, 16: "The live fixture format is described in
  fake-forge.md, section 11, and `tests/forge/github/README.md`."

## Owns and may change

- Owns that one line of `skein/docs/design/forge-github.md`.
- Departs from your text: this task edits `docs/design/` (text only; `notes/common.md`). It
  changes no Rust, so no fmt or clippy.

## Steps

1. Replace `` `tests/forge/github/README.md` `` with `` `skein/tests/forge/github/README.md` ``.
2. Commit; run `citations.py --all` and the test above (check it selected one test); push.

## Report

`notes/common.md`'s report, and the test's count.
