+++
id = "temper-citations"
part = "temper"
kind = "docs"
needs = []
files = ["temper/docs/design/client/mockups/STATUS.md", "temper/docs/design/draft/archive/first-run.md", "temper/docs/design/draft/reference-code-production.status.md", "temper/docs/design/protocol/configuration.md", "temper/docs/design/protocol/credentials.md", "temper/docs/design/protocol/channel.md", "temper/docs/plan/me-retention/plan.md"]
effort = "medium"
minutes = 20
+++
# temper's citations name their file and section

## Goal

Six temper citations give a section with no file, so the reader (and the checker) takes it as
the citing document's own, where it has no such section; and a retired plan names the ladder's
backlog at its old path. Name the file each one means. Text only, one citation at a time.

Acceptance:

- After your commit, `python3 -I -B tools/citations/citations.py --all` prints none of the six
  findings `notes/findings.md` gives this task, and none that the list lacks.
- No test reads these files (`.config/gate/read-by-tests.txt` has none of them, and no code
  includes them): no test runs.

## Inputs

- `temper/docs/plan/citation-burndown/notes/common.md`, then `notes/findings.md`.
- `DESIGN.md`, 4. Each target below was checked at 17273de9e2: the file holds the section and
  the section holds what the sentence says.

## Owns and may change

- Owns the lines below in the files of this spec's `files`, nothing else in them.
- Departs from your text: this task edits `docs/design/` and drafts (text only;
  `notes/common.md`). It changes no Rust, so no fmt or clippy.
- Not yours: `temper/docs/design/qa.md` stays (the ladder's kit cites it); the mockups'
  `first-run.html` and `connect-model.html` links in `client/mockups/STATUS.md` are right as
  they are (the checker task reads them).
- Option, pre-approved: if a line has moved on your base, find it by its words; if its words
  changed so that the fix below no longer fits, leave it and list it under `## Left`.

## Steps

1. `temper/docs/design/client/mockups/STATUS.md`:
   - The **provider:** sentence (line 76): "a managed Forgejo that signs in through temper
     (section 9)" becomes "(`../../protocol/sign-in.md`, section 9)", sign-in.md's "temper as
     Forgejo's OpenID provider".
   - Line 134: "The client leaves each out (client README, section 11)" becomes
     "(`../README.md`, section 11)", the client README's open questions, which hold "Reset
     demo" and the placeholders.
2. `temper/docs/design/draft/reference-code-production.status.md`, line 50, in the item
   "**6, 14: the light rules (N4).** `../domain/forge.md`, 8.2 ...": "(section 6 and 8.2)"
   becomes "(`../domain/forge.md`, section 6 and 8.2)".
3. `temper/docs/design/draft/archive/first-run.md`, line 393, in 3.9: "Its policy is section
   7.2's, unchanged." becomes "Its policy is that of setup.md, 7.2, unchanged." (the box's
   "What it does", `temper/docs/design/protocol/setup.md`, 7.2; the draft cites it as
   `setup.md` elsewhere, as at line 380).
4. `temper/docs/design/protocol/`: every "(README, N)" names the file, `README.md`, which beside
   these files is `temper/docs/design/protocol/README.md` (its 14 is the open questions, its 15
   the cluster):
   - `configuration.md`: line 81 "(README, section 15)" becomes "(`README.md`, section 15)";
     lines 150, 484, 787 and 1283, "(README, 15.N)" becomes "(`README.md`, 15.N)".
   - `credentials.md`: line 694 "(README, section 14)" becomes "(`README.md`, section 14)";
     lines 537 and 692, "(README, 15.4" becomes "(`README.md`, 15.4".
   - `channel.md`: line 156, "(README, 15.2)" becomes "(`README.md`, 15.2)".
5. `temper/docs/plan/me-retention/plan.md`, line 4: `temper/docs/design/debt/v0.1.0-ladder.md`
   becomes `temper/docs/development/v0.1.0-ladder.md` (plans are not checked; the file moved).
6. Commit; run `citations.py --all` (acceptance); push.

## Report

`notes/common.md`'s report, and each edit as `file:line: before → after`.
