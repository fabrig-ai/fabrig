+++
id = "readmes"
part = "family"
kind = "docs"
needs = []
files = ["hearth/README.md", "skein/README.md", "smith/README.md", "jig/README.md", "temper/README.md"]
effort = "medium"
minutes = 15
+++

# hearth is a tree in every README

## Goal

hearth joined the monorepo, but every README still says otherwise: its own "The family" names separate repositories under an
old organization, and the other four trees link a repository that is about to go.

Acceptance:

- `git grep -n -E 'fabrig-ai/hearth|fabrica-works' -- skein/README.md smith/README.md jig/README.md temper/README.md hearth/README.md`
  prints nothing.
- No README of the five says hearth is in a repository of its own.
- Each of the five READMEs' "The family" has the same five rows, `hearth` among them as a relative link.
- `python3 -I -B tools/citations/citations.py --all` reads 0 and `outward` passes.

## Inputs

- `hearth/docs/plan/hearth-docs/notes/common.md`
- `hearth/docs/plan/hearth-docs/inputs/request.md`, 4
- `smith/README.md`, "The family": the form to match

## Owns and may change

- **Owns:** the five READMEs, in the sections below alone.
- **Not to touch:** `temper/docs/design/draft/v0.1.0.md`; hearth's own `AGENTS.md`.
- **Options:** if a link in hearth's "Start here" doesn't resolve, fix it and say so. They resolve on this plan's base, `AGENTS.md`
  among them, so expect no change there.

## Steps

1. In `skein/README.md`, `smith/README.md`, `jig/README.md` and `temper/README.md`, remove the two lines
   `[hearth](https://github.com/fabrig-ai/hearth), immutable machines built` /
   `from a revision, builds on skein in a repository of its own.`, with the blank line before them, and add below the table's
   temper row:

   ```markdown
   | [hearth](../hearth) | Immutable machines, built from a revision | skein |
   ```

   In `temper/README.md` the paragraph after those lines stays, a blank line from the table.
2. In `hearth/README.md`, "The family" becomes:

   ```markdown
   hearth is a tree of [fabrig](https://github.com/fabrig-ai/fabrig), an
   agent-driven software factory: write, prove, run.

   | Tree | | Builds on |
   |---|---|---|
   | [skein](../skein) | One thread, one loop, no async | |
   | [smith](../smith) | Protocol-driven agent loop for AI applications | skein |
   | [jig](../jig) | Engine for agentic applications | skein, smith |
   | [temper](../temper) | Agent-driven software development on a forge | skein, smith, jig |
   | [hearth](../hearth) | Immutable machines, built from a revision | skein |
   ```

3. In `hearth/README.md`, "Start here": the bullet on skein's foundation becomes the same links `skein/README.md` gives, relative
   to `hearth/`:

   ```markdown
   - skein's foundation, which hearth follows:
     [`programming-model.md`](../skein/docs/foundation/programming-model.md),
     [`testing-strategy.md`](../skein/docs/foundation/testing-strategy.md) and
     [`notes.md`](../skein/docs/foundation/notes.md).
   ```

4. Commit, run the checks in `notes/common.md`, push.

## Report

The rows you changed in each README, and the check's count.
