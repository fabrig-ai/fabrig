+++
id = "citations-docs"
part = "citations"
kind = "docs"
needs = []
files = ["hearth/docs/design/**", "hearth/AGENTS.md", "skein/docs/design/io.md", "skein/docs/design/json.md", "skein/docs/design/kernel.md"]
effort = "medium"
minutes = 50
+++

# hearth's documents cite by the path from the root

## Goal

hearth's documents cite one another by their path from `hearth/docs/design/` and skein's foundation by bare name, and
`hearth/docs/design/README.md`, 6 makes that the convention. A script moves them to the repository's rule, the path from the root,
and the convention goes. Every later task of this plan writes on what this one leaves.

Acceptance:

- Over `hearth/docs/design` but `draft/archive/factory.md`, and `hearth/AGENTS.md`, this prints only code fences, Markdown link targets
  and section 2.2's keeps:

  ```sh
  git grep -n -E '(^|[^/[:alnum:]_.-])((domain|protocol|client|draft)/[[:alnum:]_.-]+\.md|\.\./README\.md|README\.md|STATUS\.md|programming-model\.md|testing-strategy\.md|(channel|io|json|kernel|kv|lib|llm|shell|simulator|testing|connectors|authority|examples)\.md)' -- hearth/docs/design hearth/AGENTS.md ':!hearth/docs/design/draft/archive'
  ```

- `hearth/docs/design/README.md`, 6 and `hearth/docs/design/client/README.md`'s opening say the new rule, as below.
- `hearth/AGENTS.md` no longer states the convention.
- skein's three documents cite hearth by the root path, and no other line of theirs changes.
- Each citation names the document it named: step 3's comparison is clean.
- `python3 -I -B tools/citations/citations.py --all` reads 0 and `outward` passes.

## Inputs

- `hearth/docs/plan/hearth-docs/notes/common.md`
- `hearth/docs/plan/hearth-docs/notes/contracts.md`, 2
- `hearth/docs/plan/hearth-docs/inputs/request.md`, 3
- `tools/citations/README.md`, and `resolve` in `tools/citations/citations.py`
- `DESIGN.md`, 4

## Owns and may change

- **Owns:** the citations of every Markdown file under `hearth/docs/design/` but `draft/archive/factory.md`, `STATUS.md` among them;
  `hearth/docs/design/README.md`, 6; `hearth/AGENTS.md`. Writing `docs/design/` departs from the implementer's text, as
  `notes/common.md` says.
- **May change, and only so:** in `skein/docs/design/io.md`, near 1258, `json.md`, near 237, and `kernel.md`, near 1004, the citation of
  hearth's document. "hearth's domain/store.md, 5.2" becomes `hearth/docs/design/domain/store.md, 5.2`, and the same for
  `domain/render.md, section 7` and `` `domain/testing.md`, 3.3 ``. `kernel.md`, 536's "(hearth's)" names no document and stays.
- **Not to touch:** `hearth/docs/design/draft/archive/factory.md`, `hearth/docs/development/**`, `hearth/README.md` (the `readmes` task),
  a section's text beyond its citations, and any private name (a later task's).
- **Options:**
  - Markdown lines aren't rewrapped: a line grows, and the diff stays a line.
  - A section number that turns out wrong once you read the citation isn't yours: leave it and report it.
  - If `citations.py` reads more than 0 because of `draft/archive/factory.md`'s own text, make the smallest change that clears it and
    say so.

## Steps

1. Write an untracked script over the owned files, mapping by `notes/contracts.md`, 2.1, and skipping code fences, link targets and
   section 2.2's keeps. Bare `README.md` in `hearth/docs/design/client/README.md` and `../README.md` everywhere name the top document.
   Owners that end one line and paths that start the next are joined for the match, and the rewrite keeps the line break.
2. List what the acceptance's `git grep` still prints, and settle each by reading: rewrite it or say why it stays.
3. Compare: for each citation changed, the target `resolve` gives on the base must be the target of the new path, but for a bare
   `README.md` in `client/README.md`, which the checker read as itself and hearth's convention as the top document.
4. Rewrite by hand, in `hearth/docs/design/README.md`, 6, the first two bullets and the one on code citations:

   ```markdown
   - **A document is cited by its path from the repository root:**
     `hearth/docs/design/domain/store.md`; with a section,
     `hearth/docs/design/domain/store.md, section 4` or
     `(hearth/docs/design/protocol/wire.md, 4.2)`. Every document names the
     others this way, whatever directory it is in.
   - **Other trees' documents** are cited the same way:
     `skein/docs/foundation/programming-model.md`,
     `skein/docs/design/channel.md`, `jig/docs/design/domain/connectors.md`.
   ```

   ```markdown
   - **Code cites a section** at module or type level, as
     `hearth/docs/design/domain/slots.md, 5.2`, never on every field.
   ```

   The other bullets stay.
5. `hearth/docs/design/client/README.md`'s opening sentence becomes "Documents are cited by their path from the repository root, as
   everywhere in hearth's design (`hearth/docs/design/README.md`, section 6)."
6. In `hearth/AGENTS.md`, remove the bullet "**The design:**" whole, since its first paragraph names the directory, and rewrite the
   other citations.
7. Edit the three skein lines. Delete the script, commit (`hearth: documents cite by their path from the root`), run the checks in
   `notes/common.md`, push.

## Report

Citations rewritten per document; what the acceptance's `git grep` still prints, grouped by why; step 3's result; any wrong section seen.
