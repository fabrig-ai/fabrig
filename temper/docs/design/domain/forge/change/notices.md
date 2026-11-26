# A relay's notices to its senders

What a change's relay (`questions.md`, 1) and a goal's closer's
(`../branches.md`, 2) tell the task that sends them an answer or words:
that an answer is held, by what and since when, and when it is
delivered; and, for an answer or words neither takes, a refusal that
names what is open. Written from `../../../draft/bounded-waits.md`, 6.1
and 6.2. A relay takes answers and words from tasks alone, so every
sender here is a task; a party's words to a change are
`questions.md`, 1, *Its requester's words and amendments*.

## 1. A held answer tells its sender

- **Held.** An answer a relay holds, a change's for a pending
  dependency or for message room (`questions.md`, 1, *Held for a
  dependency it did not start from* and *Held for message room*), a
  closer's for message room (`../branches.md`, 2, *Room and
  admission*), tells its sender so, once, in the step that takes it and
  holds it, in words:
  - for a dependency: `Your answer to question Q is held until task D
    ends; held since TIME.`;
  - for message room: `Your answer to question Q is held until this
    change has message room; held since TIME.`, a closer saying `this
    closer`;

  Q the relay's own question the answer answers, by the number jig gave
  it, the one its sender answered; D the dependency that holds it, the
  first of the change's pending dependencies that its task did not start
  from and that is no change into another repository, in the order the
  change's task keeps them; TIME that step's instant, as
  `YYYY-MM-DDTHH:MM:SSZ`.
- **Its room.** The notice is the question's notice (jig's
  `domain/tasks/messages.md`, 2, *A procedure's question keeps room for
  its notice*; `domain/connectors-procedures.md`, 1.3): it is sent in the
  room the question kept in its sender's inbox when the relay asked it,
  apart from the step's message room, so a relay short of either still
  sends it. A step that takes an answer and relays it at once sends
  none, and gives the room back. A step refused whole sends nothing; the
  step after it takes the answer again and sends the notice then.
- **Delivered.** The step that relays a held answer to its asker tells
  its sender in the same step, in words, `Your answer to question Q,
  held since TIME, is delivered to task A.`, TIME the *since* the record
  kept (2) and A the asker: a change's producer, repair or resolution;
  for a closer, the change it relays the answer to. It is an ordinary
  message of the step, sharing its message room: the answer and its
  notice go in one step, or wait together for a step with room for both,
  as *Held for message room* says. Where that change holds the answer in
  turn, its own notice goes to the closer, which passes it up to its goal
  as its open steps' words (`../branches.md`, 2).
- **Ended first.** An answer whose asker has ended tells its sender so,
  as `questions.md`, 1, *An asker that ends first*, and
  `../branches.md`, 2, *A change that ends first*, say, in place of the
  delivered notice.
- **Why in the question's room.** A relay short of message room can't
  send a notice through that same room, and a sender whose inbox is full
  would refuse it, refusing the step; a room taken when the question is
  asked is there whenever the answer comes
  (`../../../draft/bounded-waits.notes.md`).

## 2. The record names the blocker

- **What it keeps.** With a held answer, the relay's record keeps the
  number of the question it answers, its blocker, a dependency by task
  number or message room, and since when that blocker has held it. A
  step that finds another blocker than the record's (one dependency
  landed while another is pending; no dependency left while room is
  short) records the new one with that step's instant, and sends no
  second notice. A closer's held answer is held for message room alone,
  so its record keeps the *since*.
- **Stored.** Each is an added field with its default in the store's
  registered family (`temper-store`: the change's `Held`, the closer's
  asked question), so no version moves (jig's
  `protocol/store/layouts.md`). A record an earlier build kept reads question
  0 and neither blocker nor *since*: the first step after the start that
  reads it writes its blocker, since that step, so its *since* is from
  the first start of the build that read it
  (`../../../draft/bounded-waits.md`, 3.4); no notice, held or
  delivered, goes for it, since its sender was never told it was held.
- **Shown.** The change's panel carries a held question's blocker and
  *since* (`../../../client/forge/records.md`, 4), and
  `ctl`'s `change show` prints them on its line, `answered, held until
  task D ends, since TIME` or `answered, held for message room, since
  TIME`, its JSON giving the question a `held` object beside its
  `stands`, `{"blocker": {"task": D}, "since": "TIME"}` or
  `{"blocker": "room", "since": "TIME"}`, and none where the record
  keeps neither (`../../../client/ctl/areas-forges.md`, 1). The project's waits
  read names them too, once it lists a change's waits
  (`../../../draft/bounded-waits.md`, 9).

## 3. An answer or words a relay doesn't take

- **At the call.** An agent's `message` in the answer form to a change
  or a closer, naming a question other than the one that target has
  open to the agent, is refused at the call, nothing sent:
  `Question Q is not open at task C; its question to you is N: LINE.
  Answer it: message target=C, form=answer, question=N.`, or, where C
  has none open to it, `Task C has no question open to you; read_task
  on task C shows its questions.` N is the target's question asked of
  the caller (a change's, the one its task holds a credit for with the
  caller its answerer; a closer's, its standing question's number) and
  LINE that question's first line as the relay's record keeps it (a
  change's asked question's line; the first line of a closer's standing
  question's words, as `../branches.md`, 2 cuts it). For these two
  targets these words replace jig's reference refusal, which names no
  question. An answer matching the open question passes to jig, which
  admits it as before.
- **At a step.** An answer a relay finds in its inbox and doesn't take
  (a change's while its first open question isn't asked; a closer's
  that matches no standing question, or answers one already answered)
  receives the same words, as words to its sender, at the step that
  takes it, in place of the fallback feedback a change gave it and the
  silence a closer gave it.
- **A closer's other words.** Words from a task that is no open step's
  change, its goal's among them, are answered, in words, `This closer
  takes no words: it relays its open steps' questions and their answers
  alone. To answer, message form=answer with the question's number.`,
  where they were taken and dropped.
- **None dropped for room.** A reply above that finds no message room
  at its step stays on the relay's record and goes at the next step,
  due at once, as a closer's retained replies do (`../branches.md`, 2,
  *Room and admission*), so nothing a relay takes is dropped unanswered.

## 4. Open questions

- A sender's own read of a held answer's blocker, in `read_task` on the
  change, beside the durable question number it gives: the views of
  `../../../draft/bounded-waits.md`, 9 list it as a wait.
