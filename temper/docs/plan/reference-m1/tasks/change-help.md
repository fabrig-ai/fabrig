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
