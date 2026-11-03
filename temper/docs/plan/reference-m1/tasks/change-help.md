+++
id = "change-help"
part = "change"
kind = "docs"
needs = ["change-words"]
files = ["temper/docs/help/change.md"]
effort = "medium"
minutes = 30
+++

# change-help: the change page names a plan's members, the light rules and the report

## Goal

`temper help change` says what `change` takes beside the words, that a change is checked and
updated by its landing branch's rules, and that its result carries its coder's report. The
text below is the lead's: apply it verbatim (a stated departure from the implementer's text,
whose "Not yours" otherwise keeps `docs/help/` from implementers), then make the help tests
pass without changing its words; a test that fails on the words is a question.

Acceptance: the four replacements below are in `temper/docs/help/change.md`, and
`temper-docs`'s tests and the help tests of its readers pass.

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3.1 ("What a plan decides");
  `temper/docs/design/domain/forge.md`, 8.2 steps 3, 6 and 7, and 8.7 ("What the procedure
  asks of the rules").
- `temper/docs/plan/reference-m1/notes/common.md`.

## Owns and may change

Owns `temper/docs/help/change.md`. The page is changed on plan reference-m8's trunk too
(`reference-m8/help-pages`): keep each hunk to the replacements below. No other file.

1. In `## What it is`, the bullet `- **A change task** is a delegate ...` ends, after
   `from those words alone (temper help task).`, with:

   ```
     A run whose authority holds the control family (amend, cancel,
     release) names, too, what a plan decides: `landing`, `goal` for its
     own goal branch or a branch's name, else the default branch;
     `after`, the changes it waits for, by task number or by a `name`
     given in the same call; and `changes`, several changes made at
     once, all or none.
   ```

2. In the bullet `- **Its result** is the change landed: ...`, the words `BASE as SHORT.`,
   then what the landing passed` become `BASE as SHORT.`, then `Report:` and its coder's last
   report, then what the landing passed`, rewrapped at the page's width, nothing else of the
   bullet changed.

3. The step `` - `checking`: the forge's CI runs at its head; `` becomes:

   ```
   - `checking`: the forge's CI runs at its head, where its landing
     branch's rules ask CI passed (temper help landing-rule);
   ```

4. The step `` - `updating`: a branch behind its landing branch is updated by the `` and the
   rest of its item gain, at the item's end before its `;`, the words `, where its landing
   branch's rules ask up to date; under rules that ask none, a change whose merge conflicts
   is held, saying so`, rewrapped.

## Steps

1. The replacements.
2. `heavy cargo nextest run -p temper-docs`, then `heavy cargo nextest run -p
   temper-engine-smith -E 'test(/docs/)'` and `heavy cargo nextest run -p temper -E
   'test(/help/)'` (check each selected some; if a filter selects none, run the package's
   tests whose names `rg 'help' <crate>/src` shows read the pages).
3. Commit ("Say what change takes and how it lands, in its help") and push.

## Report

As the implementer's text says.

## Answers

2026-11-02.

- **The four replacements make `change.md` 8,789 bytes, over the concept bound of 8,192:
  shorter text, or a raised bound?** Shorter text; no bound is raised (the bound is the help
  design's, and reference-m8's `help-pages` met the same bound the same way). Continue from
  the pushed branch `reference-m1/change-help` (commit `d93c8af49e`) and apply the hunks
  below to `temper/docs/help/change.md`, byte for byte. They replace the four replacements
  of the steps above with shorter words of the same meaning (the `checking` step no longer
  links `temper help landing-rule`: `gating` beside it does) and tighten four passages
  outside them: the pull request title, the result's no-change and failed-coder sentences,
  the guarantee on a reviewer that cannot read, and `Bounded work`; and they drop the
  sentence `A failed repair counts as one attempt; ...`, which "counted against its reset
  repair limit" already says. This widens Owns' "keep each hunk to the replacements" to
  these hunks; reference-m8's edits to the page are already on this trunk, untouched. The
  page is then 8,177 bytes. If a help test still refuses it, that is a question again; no
  bound is raised. Then Steps 2 and 3 as written (commit "Fit the change page within its
  bound"). Blank context lines in the diff are written empty.

  ```diff
  @@ -10,15 +10,12 @@
     `change`, giving the words and, where its project has several
     repositories it may write, which one. Its coder writes the change
     from those words alone (temper help task).
  -  A run whose authority holds the control family (amend, cancel,
  -  release) names, too, what a plan decides: `landing`, `goal` for its
  -  own goal branch or a branch's name, else the default branch;
  -  `after`, the changes it waits for, by task number or by a `name`
  -  given in the same call; and `changes`, several changes made at
  -  once, all or none.
  -- **Its pull request title** comes from the coder's change result. Context
  -  at the beginning of the brief stays in the brief. Titles keep punctuation
  -  and are shortened at a UTF-8 boundary with “…” when the forge needs it.
  +  With the control family (amend, cancel, release), a run names, too,
  +  `landing` (`goal`, a branch, or the default), `after` (the changes
  +  it waits for, by task number or a `name` from the same call) and
  +  `changes` (several at once, all or none).
  +- **Its pull request title** is its coder's, punctuation kept, cut at a
  +  UTF-8 boundary with “…” where the forge needs it.
   - **Its panel,** as `change show` prints it: `change TASK ·
     OWNER/NAME on forge FORGE · BRANCH into BASE`, its pull request and
     its address, `step:` and the step, `way: turn · landing task N`
  @@ -35,10 +32,10 @@
     follow its landing branch (temper help tier), the result waits for
     their verdicts and adds a line, `Cleared by TIER run N.`, or `Not
     cleared: no verdict within the verification bound.` Or it fails,
  -  or is cancelled, as any task. If its coder concludes the request does
  -  not apply, it ends successfully with that explanation: no pull request,
  -  merge, hold or escalation, and nothing claimed landed. A coder that fails
  -  for good holds it at `producing` (below).
  +  or is cancelled, as any task. A coder that concludes the request does
  +  not apply ends it successfully, saying why, with no pull request and
  +  nothing claimed landed; one that fails for good holds it at
  +  `producing` (below).

   ## How it changes

  @@ -50,16 +47,15 @@
     concludes no change applies ends the change successfully;
   - `opening`: temper opens its pull request;
   - `checking`: the forge's CI runs at its head, where its landing
  -  branch's rules ask CI passed (temper help landing-rule);
  +  branch's rules ask it;
   - `gating`: its blocking gates give their verdicts (temper help
     landing-rule);
   - `queued`: it waits its turn in its landing branch's queue, `queued:
     place N`;
   - `first`: at the front, temper checks CI and the blocking gates again;
   - `updating`: a branch behind its landing branch is updated by the
  -  forge, which merges the base in, where its landing branch's rules
  -  ask up to date; under rules that ask none, a change whose merge
  -  conflicts is held, saying so;
  +  forge, which merges the base in, where its rules ask up to date;
  +  else a conflicting change is held, saying so;
   - `resolving`: an agent resolves a conflict with the base and pushes
     the merge commit;
   - `repairing`: an agent repairs it, after CI failed at its head (a
  @@ -106,8 +102,7 @@
   more of its tasks, or holds again if its authority can't cover one.
   Held at `repairing` because its repair failed for good, release makes
   a fresh repair for the same reason, counted against its reset repair
  -limit, or holds again if its authority can't cover one. A failed repair
  -counts as one attempt; it does not mean all its repairs are spent.
  +limit, or holds again if its authority can't cover one.
   Held at `producing`, its hold carries the coder's own words, why it
   made no change, whole within 512 bytes, and its chat hears them:
   cancel it if they hold, or release it if the coder was wrong. Its
  @@ -127,18 +122,15 @@
     repair; a person's rejection holds it; an advisory gate never blocks.
     A reviewer that gives no verdict is asked once more, then the change
     is held.
  -  If a reviewer cannot read what it needs to judge, it fails saying why.
  -  The operator's `temper serve` stderr names refused forge reads and call
  -  admission by task, attempt, tool, layer and reason. Claude's built-in
  -  Read denials appear in the task's recorded tool results. These operator
  -  diagnostics are volatile; capture stderr when investigating a refusal.
  -
  +  A reviewer that cannot read what it needs fails saying why;
  +  `temper serve`'s stderr, unkept, names each refusal by task, tool
  +  and reason; Claude's Read denials are in the task's tool results.
   - **Gates are temper's:** a change's gates run in temper, and `change
     show` prints each, `gate 1 · review · blocking · approved`; temper
     posts none to the forge as a review or a status, so the pull
     request's reviews there may be empty.
  -- **Bounded work:** past its limit of repairs, resolutions or updates,
  -  the change holds rather than trying again.
  +- **Bounded work:** past a limit of repairs, resolutions or updates,
  +  it holds.
   - **The parent hears its result,** landed, and cleared where tiers follow its branch, or not.

   ## Acting on it
  ```
