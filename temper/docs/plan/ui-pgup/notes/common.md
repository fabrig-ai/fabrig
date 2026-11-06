# ui-pgup: the rules every task names

The request is `temper/docs/plan/ui-pgup/inputs/request.md`: read it, it is short, and its section *For every task of this plan* binds you. Where it and your spec differ on what runs, the request wins: only focused and fuzzy run, never a tier, a browser story or `tools/tiers/run`, and no browser story is added.

## What this plan holds to

Section 5 of `temper/docs/design/draft/chat-reading.md`: PgUp, PgDn, a wheel and "Latest ↓" move a conversation while its agent writes and after its turn ends, the focus stays in the composer with its words and caret, and only the person's own acts bring the view to the foot.

## Departures from the implementer's text

- **The driver is tracked.** `temper/docs/plan/ui-pgup/driver/` holds this plan's driver, as `temper/docs/plan/bundle-loading/driver/` holds that plan's, in place of a script gone before the push: `reproduce` writes it and `proof` runs the same steps on the fixed code, so the two reports compare. It stays in the plan's directory: it is no tool, no test and no tier, and no code or test reads it. It names no path or address of a machine, and nothing private.
- **Status lines.** The design for this plan is written, in the sections your spec cites. You change no design text, but your task removes the **Not yet** line or clause your spec names from its status file, once its code holds.
- **Failing test first,** in the lowest tier that shows the rule: run it red, and your report says what it showed.

## What `reproduced.md` changes for you

`temper/docs/plan/ui-pgup/reproduced.md` is what each step did on the code before any fix. A fix task reads it first.

- A step it reports as holding, on the path your task covers: add your task's tests, leave the production code alone, and say so in your report.
- A cause it names that your spec doesn't: fix that cause too, if it is in your files; otherwise leave it to `proof` and name it under `## Left`.

## Beside this plan

Other plans are changing the same files, and yours is one small change among them. Keep hunks small, don't reformat, and don't move code you don't need to change. If your merge meets another draft in a file your spec names, keep their change, and say so in your report.

## Limits

No budget or timeout is raised. No limit changes in this plan.
