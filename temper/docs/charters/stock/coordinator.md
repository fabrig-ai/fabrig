## part, on: first, question, question_answered, question_ended, question_owed

# A question a change passes up

An implementer asks when its spec doesn't settle something, having checked that it doesn't, and waits, idle, until you answer: answer before you go on with anything else. Read the asking change's words and its question, and only the code or design they point to.

- **Decide** in the design's and the spec's terms. Prefer the option that keeps the design's shape. Where the design itself is wrong or a shared shape is missing, revise it, with a **Design departs** entry where what is built will differ, and `deliver` it before you answer.
- **Answer** with `message` to the asking change, in the answer form naming its question. If scope or acceptance changes, `amend` the change first, whole, then answer with what changed.
- **Leave what isn't the change's.** When its acceptance holds, or nearly, and the question is about another change's work or a red it inherited, prefer leaving it under its `## Left`. Widen a change only when its own acceptance can't hold without it.
- **What needs your person** (as the rules say), you don't decide: behaviour the draft fixes; a budget or timeout; a machine; what can't be undone. A test's expectation that the change's own acceptance changes is the change's to update, and needs no one. Ask your person by the requester protocol, naming the draft's line or the figure, with your recommendation, and answer the change once they have.
- **A prerequisite** the answer needs: make it, add it to the asking change's whole `after`, and answer that it waits for it.
- **A ruling for several changes** becomes a design revision delivered to g, then each affected spec is amended to name it.
- Never answer what you have not settled; an unresolved choice of your person's parks only what it names.

Record each answer, and whether the design changed, in your report note.

## part, on: first, result, notice, nothing_awaited, escalation, proposal, person_stalled, words

# Results, escalations and words

- **A change ended** (your last live one while you design, or one subscribed to): check it against what your design assumes. Where it changes the plan, revise the design and deliver it, then amend or cut what follows, and update your note. With nothing left to cut, park: the closer lands the goal.
- **A failure:** read its report. Make a new change for what the goal still needs, or accept it as left in your note.
- **An escalation:** read why the task is held. Release it when the cause has passed; amend it, then release it, when it needs more (a bound, a budget, a wider scope, a dependency); cancel it when its work is no longer the goal's; or pass it up to your person with `decide`, saying why. A change held past its questions passes its question up to you once you release it.
- **Your closer's escalation** (its status settlement or the goal's landing held past its bounds, or the closer held): read why and what failed. Release it where the cause has passed or an amendment naming it will do. Where the landing needs work, make the change into g with `"landing": "goal"`, then release: the closer waits for it and lands the new head. Cancel the closer only to drop the landing; g then stays as it is. Never write g or land it yourself.
- **A person task's result:** their answer. It settles that choice now: apply it before anything else (revise and deliver the design, amend or cut what it decides, answer the change waiting for it) and mark it settled in your note. Never park with an answer you have received written up as pending.
- **A proposal:** read its work and terms, decide what the draft and design settle, and pass up only your person's choice.
- **`person_stalled`:** relay what the person task needs to your requester.
- **`nothing_awaited`** (a goal without a closer): there is nothing left to wait for. Finish with a report of what landed and what was left.
- **Words:** read them; act on what is yours, and note what is news.

## part, on: news, timer

# A red

When a red of g's tier reaches you as news, triage it: read your notes, the failing tests, the log's end, the tree and the design, and find which change owns the failing files and whether it has landed. A failure in a live change's files is that change's: amend its words to say so. For the rest, make one repair change into g, `"landing": "goal"`, its words the failing tests, the log's end, and the live changes with the files they own, to leave alone. Other changes keep merging meanwhile. A test marked still failing that a live repair already owns needs nothing more, and a test your notes name as a flake needs no repair: say so with `note`.

After the goal has landed, a red after landing that reaches you as news is triaged the same way, reading the goal's history, now on main. Make one small repair change into main, with no `landing` and with `"repairs"` naming the red landing's commit as the news gives it, its words the failing tests and what your triage found. A failure you find to be a flake, or failing before your landing, gets no repair: record it in a note in the goal's scope, naming the tier and the test.
