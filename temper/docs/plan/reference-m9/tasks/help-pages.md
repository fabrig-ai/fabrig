+++
id = "help-pages"
part = "stories"
kind = "docs"
needs = ["goal-tier-config", "repairs-call", "repairs-verdict"]
files = ["temper/docs/help/tier.md", "temper/docs/help/change.md"]
effort = "medium"
minutes = 40
+++

# Help: a tier of goal branches, a red's news, and `repairs`

## Goal

`temper help tier` and `temper help change` say what this plan builds: a tier following the
goals' own branches with a span, a red's news to its goal, and a change's `repairs`. The text
is given whole below; place it as given. This task departs from the implementer's text in one
point: `docs/help/` is yours here, for these edits alone.

Acceptance (`temper/docs/design/domain/after-landing.md`, 2, 5.4, 7; `agent.md`, 4.3.1):

In `temper/docs/help/tier.md`:

1. In the first bullet of `## What it is`, replace `the repository's landing branch, its main,
   unless it names another. A landing on a branch no tier follows, a goal's own branch for one,
   is cleared as it lands.` with: `the repository's landing branch, its main, unless it names
   another, or `goal`, the goals' own branches. A landing on a branch no tier follows, or one
   only a tier of goal branches follows, is cleared as it lands.`
2. After the bullet that starts `- **A run** covers every landing`, a new bullet:

   ```
   - **A tier of goal branches,** `"branch": "goal"`, checks each goal's
     own branch along the way: at most one run per its span on each
     branch, one at a time across them, at the branch's newest tip. It
     judges no landing. Its red runs the failed tests once more, alone,
     and what fails again is news to the goal holding the branch, each
     test marked new or still failing; it wakes the goal when a test is
     new. A green run says nothing.
   ```
3. After the bullet that starts `- **A landing's verdict is its own:**`, a new bullet:

   ```
   - **A red goes to its goal** as news: the tier, the run, the
     landing's commit, the failing tests blamed on it and the end of
     each failed job's log, waking the goal. The goal repairs it with a
     change naming the red landing (`repairs`, temper help change);
     when a later run passes the failures, the red landing is cleared,
     repaired by that change's landing, and its change's result ends
     `repaired by COMMIT.`
   ```
4. In `## Acting on it`, replace `set `branch` to another branch's name to follow it instead.`
   with `set `branch` to another branch's name to follow it instead, or to `goal` to follow the
   goals' own branches. Set `span_ms` to keep a tier's runs on one branch at least that many
   milliseconds apart; none when absent.`
5. In `## Limits`, replace `- temper neither tells a goal of a red after landing nor posts a
   verdict to the forge.` with `- temper posts no verdict to the forge, and a red whose goal has
   ended, or of a landing temper did not make, reaches no one.`

In `temper/docs/help/change.md`:

6. In the first bullet, replace `` `after` (what it waits for: task numbers, or names from the
   same call) and `changes` (several at once, all or none).`` with `` `after` (what it waits
   for: task numbers, or names from the same call), `repairs` (the red landing it repairs, by
   its merge commit's digits as the red's news names them: temper help tier) and `changes`
   (several at once, all or none).``
7. In the bullet that starts `- **Its panel,**`, after `` `Cleared by TIER run N.`, `` insert
   `` for a landing once red, `Cleared by TIER run N, which passed the failures blamed on it;
   repaired by COMMIT.` where a change naming it repaired it, ``.

Each paragraph keeps the file's wrapping (about 72 columns, two-space continuation in a
bullet). If reference-m3, m5 or m7b changed the same sentence first, apply the edit's meaning to
their sentence, keeping theirs, and say so.

- Tests pass: `heavy cargo nextest run -p temper-docs`; `-p temper --lib -E 'test(/help/)'`;
  `-p temper-client-tools --lib -E 'test(/help/)'`.

## Inputs

- `temper/docs/plan/reference-m9/notes/common.md`.
- `temper/docs/design/domain/after-landing.md`, sections 2, 5.4 and 7;
  `temper/docs/design/domain/agent.md`, 4.3.1 (`repairs`).

## Owns and may change

- Owns the two help pages for the edits listed; a golden file a help test compares with these
  pages (`temper/crates/temper-client-tools/src/tests/golden/**`) is regenerated as its test
  says, and named in the report.

## Steps

1. The edits; then the tests as acceptance says.
2. Commit and push.

## Report

As the implementer's text asks.

## Answers

2026-11-03. **change.md is over its concept bound (8,463 of 8,192 bytes) with the
given text: shorten other prose, or change the wording or the bound?** Shorten
`temper/docs/help/change.md`'s existing prose until the page is within 8,192 bytes; the
bound (`temper-docs`, `Kind::bound`) stays as it is and the supplied additions stay word for
word. Cut only what the same page already says elsewhere, so no fact is lost, and keep the
sentences `temper/crates/temper/src/tests/help.rs` asserts. Places that repeat themselves:
the `producing` step's `a coder that explicitly concludes no change applies ends the change
successfully` (said under `Its result`); the two `Held at producing / repairing ... or holds
again if its authority can't cover one` sentences, which can share one ending. Name each cut
in the report. tier.md needs nothing more.
