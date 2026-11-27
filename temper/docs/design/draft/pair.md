# Pair: a connected implementer

Draft, 2026-11-27, from a session with the user that day. The try's draft sketched a person working "quite like a coding agent", with one agent that "has its own repo checkout/workspace and works directly in it", and "Land this" as the person's act (its section 4.8, on branch draft/trying-temper); it left the build to "a draft of its own", this one. The user's answers: a pair lands its own edits alone, never handing over to another way ("yes, its own edits"); one change at a time; "basically it should be like a 'connected implementer'"; and "'pair' might just be a name we give to a way, not the mechanism itself, which is generic". So this draft adds one mechanism, a change whose producer talks with its requester, and Pair is the way that uses it ([ways-recipes.md](ways-recipes.md)). How it looks, in the pages and the terminal, stays the try's and the UX pass's.

## 1. In one page

- **A connected implementer** (section 3): a change's producer, an implementer in its own checkout, that works in conversation with the person who asked for the change, instead of from a spec, unattended. Its replies reach the person; the person's words reach it; it edits, runs and checks as any producer does.
- **Its copy is the change's own:** the producer's checkout on the change's branch, kept between turns by an unchecked save each time it stops to wait (3.3). No worker place is held while it waits.
- **"Land this"** is the person's word that asks it to finish (3.4): it runs the `finish` hook and pushes, and from there it is any change: its pull request, CI, the way's gates and its landing.
- **One change, then done:** the producer task ends with its change; the next change is a new start. Repairs after it are the change's own, as today (3.5).
- **Pair is a way, nothing more** (section 4): a change whose producer is connected. Any way may connect a producer, and nothing else in a way changes. The ways format loses `copy` and `handed`, which this replaces.

## 2. Today

- **A producer runs unattended:** a change's producing task is an agent under charter 2, Finite, that works from its spec, may `save`, ask its requester a question and `wait`, `deliver` and `finish` with a change (`../domain/agent/tools.md`; smith's `domain/run-delivery.md`, section 1). A message to its requester is a question alone (the stock implementer's words).
- **A chat converses, and writes nothing:** an Open run whose text replies wait for the person and whose activation parks; Open requires `wait` and forbids `deliver` (smith's `domain/run.md`, section 1). A change chat delegates its change to a producer.
- **A message may ask a run to finish:** `finish_requested` puts an Open run into its wind-down; a Finite run ignores the bit (smith's `domain/run-messages.md`, section 1).
- **Only saved or delivered work reaches a producer's next run:** its checkout is prepared afresh at the branch's head each time (`../domain/agent/README.md`, section 5).

## 3. A connected implementer

### 3.1 A change whose producer talks

- **The change comes first:** a start in a way whose change names a connected producer (section 4) makes the change at once, with the person's words as its spec, and its producer task with `conversation` set. Its requester is the person.
- **The person's words reach the producer** as its requester's do today, waking it (jig's `domain/tasks/messages.md`, section 3); **its replies reach the person** as a conversation's do, on the producer task.
- **One producer for the change's life until it finishes:** its runs are that task's, each resumed from its transcript, as a parked run resumes today (smith's `domain/run-messages.md`, section 1).

### 3.2 A Finite run in conversation

- **smith gains a Finite run with a conversation,** a charter's flag: its text replies reach its host as replies and park the run until words come, as Open's do, and it keeps `save`, `deliver` and `finish` with its change contract, as Finite's do. It never stops without a reply or a result: the nudge to finish that a Finite run gets when it stops without one is, here, the park.
- **`finish_requested`** asks it to finish: the next turn is offered `finish` and `deliver` alone, as a wind-down's last turns are, and its text reply says what still stops it if it can't.
- **Its budget is the change's,** spent across its runs, with its reserve kept for finishing (smith's `domain/budget.md`).

### 3.3 Its copy, kept between turns

- **The change's branch is the copy:** the producer's checkout, made at the change's branch as any producer's is.
- **Saved when it parks:** a connected run that parks with edits not yet delivered saves them first, unchecked (smith's `domain/run-delivery.md`, 1.5), so that its next run starts from them. A park with nothing new saves nothing.
- **Warm where it can be:** a worker that still holds the checkout reuses it; one that doesn't prepares it afresh at the branch's head, which holds the save.
- **No place held while waiting:** between runs nothing is live, as for any parked run, so a person's pause costs no worker.

### 3.4 "Land this"

- **The person's word,** sent with `finish_requested`: in the pages a press, in the terminal `temper ctl task message TASK --finish "WORDS"`.
- **The producer finishes as any does:** formatters as its charter says, then `finish` with the change's title and body, which runs the `finish` hook ([hooks.md](hooks.md)) and pushes. A refused delivery comes back to it, and to the person, as words: the run goes on until it lands or the person stops it.
- **From there it is any change:** its pull request opens, CI and the way's gates judge it, and it lands by the way's landing, under the floor (`ways.md`, section 5). Pair's stock way asks no review: its person was there (`ways-gallery.md`, 1.1).

### 3.5 After it finishes

- **The producer task ends with its change,** and the conversation with it. The next change is a new start, in the same way or another.
- **Repairs are the change's own,** as today: a CI failure or a review asking for changes makes a repair, unattended, under the producer's charter, and its requester hears of it (`../domain/forge/change/repairs.md`).

### 3.6 Its charter

- **`pair`, a stock charter:** the implementer's tools, `shell` and `modify` in its copy, `save`, `deliver` and `finish`, with the conversation's flag; no tool that pushes or merges beyond its own delivery, as no producer has.
- **Its words:** work with the person, show what you change and why, ask when unsure, and finish when they say "Land this"; a sub-agent is the way to a review in place when they ask for one, so its settings list a model for sub-agents.
- **Like any charter,** a project's own in `.temper/charters/pair.md` once it runs its own ways (`ways-charters.md`).

## 4. As a way

- **A change recipe whose producer is connected:** the producer's recipe sets `conversation = true` (`ways-recipes.md`, section 2). Nothing else in the format is Pair's.

  ```toml
  # .temper/ways/pair.toml
  format = 1
  headline = "You and one agent, in its own copy"
  line = "You work with one agent on the change; when it's right, you say Land this."
  start = "change"

  [tasks.change]
  executor = "change"
  into = "base"
  producer = "partner"
  gates = []
  landing = { ci = true, up_to_date = true }

  [tasks.partner]
  executor = { agent = "pair", model = "claude-opus-5-5", effort = "high" }
  contract = "change"
  conversation = true

  [reaches]
  ```

- **Generic:** any way may connect a change's producer, a milestone's goals' changes among them, where a person wants to work with one; and a way that doesn't, doesn't.
- **What the ways format drops:** `holds = ["copy"]` and `producer = "handed"`, which the ways draft kept for Pair: the connected producer's checkout is the copy, and its finish the handover.

## 5. Upgrades

- **The charter's flag** is a field an older agent's layout lacks, so a connected charter is refused to an older worker, and its run waits for one that has it (`../protocol/wires.md`, sections 5 and 8). A charter without it reaches an older worker as today.
- **`temper ctl task message --finish`** is a new flag on a client's wire; an older client is refused by name and reloads.

## 6. Later

- **Handing over from a pair** to another way (the user, 2026-11-26: later, with moves between ways).
- **Several changes in one pair,** one after another without a new start.
- **Repairs back into the conversation** while the person is there.
- **The pages and the terminal:** "Land this", the diff, the chat's look, the playground's showcase of it (sketched in the try's draft on branch draft/trying-temper, and the UX pass's).
