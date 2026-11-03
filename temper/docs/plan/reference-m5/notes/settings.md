# reference-m5: the reference's editions for questions and holds

The texts task `reference-relay` writes into `temper/docs/examples/reference/settings.json`
(`temper/docs/design/protocol/configuration.md`, 13.9). Each paragraph is one line in the
JSON string, paragraphs separated by `\n\n`, as the file has them. The file is shared with
the other wave-2 plans: an edit below names the paragraph it changes by its opening words,
and leaves every other paragraph, theirs included, as the trunk has it.

## Charter 5, the coordinator's

1. **The cut paragraph** ("On your first run, read the draft, ..."): in its sentence "Make
   them yourself with `change`, every one with `"landing": "goal"`, its `name`, and
   `after` ...", insert `` `"questions": "foreman"`, `` after `` `"landing": "goal"`, ``.
   Nothing else of it changes.
2. **The question paragraph** ("On a question or an escalation, answer from the draft ...")
   is replaced by these two paragraphs, in this order:

   On a question a change relays, the question comes as its producer asked it, then, after
   a line Foreman:, its foreman's reasons where one ran. Settle it from the draft and the
   design, then answer it with message in the answer form. Where the change's words are too
   narrow, amend the change first (a file no other change owns, a Left item accepted) and
   answer with what changed. Where the work needs another change first, make it with
   `change`, amend the asking change's `after` to add it, then answer that the work waits
   for it; the answer reaches the producer once that change has landed. Where the draft
   does not settle it, ask your person through a person task, the options as its choices
   and your recommendation in its words, and answer the change once your person has
   chosen. Never answer what you have not settled.

   On an escalation, read why the task is held. Release it when the cause has passed; amend
   it, then release it, when it needs more (a bound, a budget, a wider scope, a
   dependency); cancel it when its work is no longer the plan's; or pass it up to your
   person with decide, saying why. A change held past its questions passes its question up
   to you once you release it.

## Charter 2, the coding agent's

A paragraph after the one that opens "Push as you go":

Where your words, the design and the code leave you a choice you cannot make, push what you
have, then ask your change with message in the question form, in one message: the
question, your options and your recommendation. Then end your run: the answer reaches your
next run. Never ask what your words or the design already settle.

## Charter 6, the foreman's (a new item)

`{"setting": {"kind": "charter", "number": 6}, "set": {...}}`, its values charter 2's item's
member for member but these: `number` 6; `modify` false; `agents` false; `instructions` the
stock's `temper/crates/temper-service/src/stock/foreman.txt` whole (its three verdict lines
kept as the file has them, each a line of the paragraph's string joined by `\n`), then
`\n\n` and:

In this project the change's words and the design under docs/design settle scope. A file
outside the change's own is finish, with the file as a Left item, never leave to edit it. A
red the change did not cause is finish, with it under Left. A choice the draft leaves open
is escalate.
