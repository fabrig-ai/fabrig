+++
id = "merge-help"
part = "catch-up"
kind = "docs"
needs = ["catch-up-call"]
files = ["temper/docs/help/change.md"]
effort = "medium"
minutes = 30
+++

## Goal

`temper help change` names `change`'s `merge` and says that a conflict goes to the merge job
before a resolution. The text below is the lead's: apply it verbatim (a stated departure from
the implementer's text, whose "Not yours" otherwise keeps `docs/help/` from implementers), then
make the help tests pass without changing its words; a test that fails on the words is a
question.

Acceptance: the three replacements below are in `temper/docs/help/change.md`, and
`temper-docs`'s tests and the help tests of its readers pass.

## Inputs

- `temper/docs/design/domain/agent.md, 4.3.1` (`merge`); `temper/docs/design/domain/forge.md,
  8.2` (step 6), `8.4` ("The merge job"), section 10.
- `temper/docs/plan/reference-m7b/notes/common.md`.

## Owns and may change

- `temper/docs/help/change.md`, the three replacements alone.
- A help test that pins a line this changes (`rg` the old lines under `temper/crates/temper-docs`
  and temper's tests): its expectation, to the new words only.

## Steps

1. Replace

   ```
     `changes` (several at once, all or none).
   ```

   with

   ```
     `changes` (several at once, all or none), and `merge` (a catch-up:
     a commit the merge job merges into a branch made at the landing
     branch's tip, where the project names a `merge_job`).
   ```

2. Replace

   ```
   - `updating`: a branch behind its landing branch is updated by the
     forge, which merges the base in, where its rules ask up to date;
     else a conflicting change is held, saying so;
   - `resolving`: an agent resolves a conflict with the base and pushes
     the merge commit;
   ```

   with

   ```
   - `updating`: a branch behind its landing branch is updated by the
     forge, which merges the base in, where its rules ask up to date;
   - `resolving`: a conflict with the base, whether met by the update
     or by the merge itself, goes first to the merge job, where the
     project names a `merge_job`: the project's command settles what it
     knows in the merge, and the worker pushes it; what it leaves, an
     agent resolves in the merge in progress, and the worker pushes the
     merge commit;
   ```

3. Replace

   ```
   - **Its pull request title** is its coder's,
   ```

   with

   ```
   - **A catch-up** (`merge`) has the merge job for its coder: it ends
     with no change where the landing branch already holds the commit.
   - **Its pull request title** is its coder's,
   ```

4. `heavy cargo nextest run -p temper-docs`, then the help tests that read the page
   (`rg 'change.md|help change' temper/crates temper/tests` for their packages); `cargo fmt`
   (no Rust should change); commit and push.

## Report

As the implementer's text says.

## Answers

2026-11-03. **The page is 6 bytes over its bound with the R2 text: shorter text, or trim other prose?**
Shorter text; trim no other prose. The page now starts at 8,191 bytes (it grew after R2 was
written). Keep run 1's branch, and change its first replacement so that it ends with these lines in
place of the original line ``  same call) and `changes` (several at once, all or none).``:

```
  same call), `changes` (several at once, all or none) and `merge`
  (a catch-up).
```

The second replacement stays R2's, verbatim. The page then measures 8,187 bytes. The design's
`merge` is "a catch-up (forge.md, section 10): the commit to take", so naming the catch-up is
enough for the help. Then run step 4's checks, the reader help tests among them.
