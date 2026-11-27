# Pair: a connected implementer

Draft, 2026-11-27, from a session with the user that day. The try's draft sketched a person working "quite like a coding agent", with one agent that "has its own repo checkout/workspace and works directly in it", and "Land this" as the person's act (its section 4.8, on branch draft/trying-temper); it left the build to "a draft of its own", this one. The user's answers: a pair lands its own edits alone, never handing over to another way ("yes, its own edits"); one change at a time; "basically it should be like a 'connected implementer'"; and "'pair' might just be a name we give to a way, not the mechanism itself, which is generic". So this draft adds one mechanism, a change whose producer talks with the person who asked for it, and Pair is the way that uses it ([ways-recipes.md](ways-recipes.md)). Its one review found the first version leaving authority, idling and repairs to the model's good will; this version puts them in code. How it looks stays the try's and the UX pass's.

## 1. In one page

- **A connected implementer** (section 3): a change's producer, an implementer in the change's own checkout, that works in conversation with the person who started the change, instead of from a spec, unattended.
- **Only the person can make it deliver** (3.3): "Land this" is a typed request of the person's, checked by the host. Until it comes, the producer edits, runs and talks, and can't push a checked head or finish with a change.
- **Its work is kept by saves the person agreed to** (3.4): when it stops to wait, what it has is saved to the change's branch, unchecked, and nothing downstream may take an unchecked head.
- **It idles without cost and without end** (3.5): waiting holds no worker, and an idle bound asks the person, never ending the work by itself.
- **After "Land this," it is any change** (3.6): its pull request, CI, the floor and the way's gates, the landing. Repairs run unattended, under the implementer's charter, from what was agreed.
- **Pair is a way, nothing more** (section 4): a change recipe whose producer sets `conversation`, for a change a person starts.

## 2. Today

- **A producer runs unattended:** a change's producing task is an agent under charter 2, Finite, working from its spec; it may `save`, ask its requester a question and `wait`, `deliver` and `finish` with a change (`../domain/agent/tools.md`; smith's `domain/run-delivery.md`, section 1). The change task is its requester: it delegates, funds and receives its result (`../domain/forge/change/README.md`, section 1).
- **A chat converses, and writes nothing:** an Open run whose text replies wait for the person and whose activation parks; Open requires `wait` and forbids `deliver` (smith's `domain/run.md`, section 1). jig derives a run's mode from how its task ends.
- **A message may ask a run to finish:** `finish_requested` puts an Open run into its wind-down; a Finite run ignores it (smith's `domain/run-messages.md`, section 1). temper's party words reach a change's current producer as ordinary words today.
- **A parked result-ended agent with nothing awaited** is woken again and spends a try, and is held past its tries (jig's `domain/tasks/lifecycle.md`, section 2).
- **Repairs and resolutions reuse the producer's charter and model,** with the change's spec and rulings (`../domain/forge/change/repairs.md`, section 1).
- **Early starts read a dependency's branch head,** not its last checked head, a known gap (`../domain/forge/change/README.status.md`).

## 3. A connected implementer

### 3.1 Who talks with it

- **A change a person starts:** the change's requester is that person, as for a change they open today (`../domain/forge/change/opened.md`). Connected producers are for such changes alone at first: a change an agent starts, a goal's among them, keeps an unattended producer (section 6).
- **Its participant:** the change's requester, and no one else. Their words reach the producer; its replies reach them, on the change. An owner's or another party's words to the producer are refused, naming the participant, so that the person's work is steered by the person.
- **Before the producer exists,** the start's words are the change's spec, and words sent meanwhile queue for its first run. **After it ends,** the conversation is closed, and words go to the change as today.
- **The change keeps its edges:** it delegates the producer, funds it from the change's budget, and takes its result, as today. A retried producer is the same task, so the conversation follows it.

### 3.2 A Finite run in conversation

- **jig gives the mode, from the recipe:** a producer whose recorded recipe sets `conversation` (`ways-recipes.md`, section 2) runs Finite, with a conversation: its text replies reach the participant and park the run, as Open's do, and it keeps its change contract. An older worker's agent, lacking the mode, is refused the run, which waits for a worker that has it.
- **A reply parks it in a wait for words:** an explicit wait jig keeps as the task's state, so that "nothing left to await" is never true of it, and no try is spent while the person is away.
- **Its outcomes, each distinct:** a reply parks and waits; an explicit `wait` the same; an empty or cut turn is retried as today; a budget wind-down parks with a reply saying what is left, never finishing on its own; a finish request goes as 3.3 says.

### 3.3 "Land this," enforced

- **A typed request of the participant's:** `FinishRequest`, on the client's wire, naming the change, sent by the person (`temper ctl task finish TASK`, and in the pages a press). It reaches the producer as a host act, never as words the model can be told to imagine.
- **Until one is held, delivery is refused by the host:** `deliver` and a change-producing `finish` answer refused, "the person hasn't said Land this", and the model is told so. `save` stays, as 3.4 says.
- **Once held, it lasts until it ends:** the producer may still edit, format and fix, then calls `finish`, which runs the `finish` hook ([hooks.md](hooks.md)) and pushes the checked tree, with the change's title and body. The request is spent by a delivery that lands, and ended by the person's next words, a cancel, or the change ending; a check that fails or a push refused leaves it held, so the producer can fix and finish again.
- **What the person sees:** each outcome on the change, typed, and in `temper ctl task show`: request held, checking, check failed with its tail, push refused, stale, budget spent, delivered, landed. When the model gives no reply, the host serves the reason itself, bounded.
- **A finish with nothing to land** is a verdict, `no_change`, and the change ends with no change, as today.

### 3.4 Its saves

- **Agreed at the start:** starting a change in a connected way agrees that its work in progress is saved to the change's own branch when the producer stops to wait. The way's line says so.
- **Saved when it parks** with work not yet saved or delivered: the host takes the workspace lease, settles jobs, and saves as a `save` does, the files a save takes and no others (smith's `domain/run-delivery.md`, 1.5). A refused save, for a conflict marker, or a stale one, is said in the reply, and the run doesn't park until the person has heard it.
- **Nothing downstream takes an unchecked head:** a connected change offers no early start and is no dependency's source until it has delivered checked, whatever the early-start gap above.
- **Resumed from the save:** the next run's checkout is the branch's head, the save, and `finish` checks it afresh before it pushes, even where the checkout is clean. A worker that still holds the checkout reuses it, as cache reuse does today.

### 3.5 Idle, and its bound

- **Waiting holds no worker:** between runs nothing is live, as for any parked run.
- **An idle bound,** the project's setting, three days unless set: past it, the change is held with a question to the person, keep going or cancel, and nothing is cancelled by itself. Funds stay reserved while it is held. A cancel releases the branch's hold and the funds, as a change's cancel does today.
- **Its history:** each run resumes from the transcript, compacted as smith compacts (smith's `domain/session-context.md`, section 1). A fresh start after a resume limit or a move recovers what matters from the change, not the transcript: its spec, its rulings (3.6), the saved head, and the finish request's state, which the host holds.

### 3.6 After it delivers

- **What was agreed becomes the change's:** the delivery's title and body are the change's record, and each decision the person made in the conversation that the delivery depends on is written as a ruling on the change, as an answered question's is today (`../domain/forge/change/questions.md`, section 1). The raw conversation is never a repair's brief or a pull request's words.
- **Repairs and resolutions run unattended,** under the implementer charter, not `pair`'s, with the change's spec and rulings, as today. The person hears of each as news, and the "Land this" they gave covers the repaired head's landing, as for any change.
- **The producer task ends with its delivery,** and the conversation with it. The next change is a new start.

### 3.7 Its charter and tools

- **`pair`, a stock charter:** the implementer's words for working with a person, showing what it changes and why, and asking when unsure; `modify` and `shell` in its workspace, granted explicitly, where the stock implementer has `shell` off; `save`, and `deliver` and `finish` under 3.3's rule.
- **A review in place:** a sub-agent, read-only, given only the workspace families for reading, on a model the charter's settings list. Its findings reach the person through the producer.
- **What the person is told:** the conversation is the change's, readable by whoever may read the change.

## 4. As a way

- **A change recipe whose producer is connected:** the producer's recipe sets `conversation = true` (`ways-recipes.md`, section 2), in a way whose `start` is a change a person starts. Pair adds no gate of its own: its stock file asks none, and a floor's review still holds.

  ```toml
  # .temper/ways/pair.toml
  format = 1
  headline = "You and one agent, on one change"
  line = "You work with one agent on the change, which saves as it goes; when it's right, you say Land this."
  start = "change"

  [tasks.change]
  executor = "change"
  into = "base"
  producer = "partner"
  gates = []
  landing = { ci = true, up_to_date = true }

  [tasks.partner]
  executor = { agent = "pair" }
  contract = "change"
  conversation = true

  [reaches]
  ```

- **Started from `temper ctl`:** `temper ctl change start --project P --way pair --budget B "WORDS"` (`ways.md`, section 4, built by W4), printing the change's task; `temper ctl task message TASK "WORDS"` to talk; `temper ctl task show TASK` and `task turn` to read; `temper ctl task finish TASK` for "Land this"; `temper ctl task cancel TASK` to stop.

## 5. Upgrades

- **The producer's mode** reaches a worker as a field of its assignment; an older agent's layout lacks it, so the run waits for a worker that has it (`../protocol/wires.md`, sections 5 and 8). An unconnected producer reaches an older worker as today.
- **`FinishRequest` and `temper ctl task finish`** are new on the client's wire: a browser at another layout reloads, and a terminal client at another layout says it is incompatible and asks to be upgraded (`../protocol/wires.md`, section 9).

## 6. Later

- **Connected producers for an agent's change,** a goal's among them, once who talks with them is decided.
- **Handing over from a pair** to another way (the user, 2026-11-26: later, with moves between ways).
- **Several changes in one pair,** one after another without a new start.
- **Repairs back into the conversation** while the person is there.
- **The pages and the terminal's look:** "Land this", the diff, the chat's look, the playground's showcase (sketched in the try's draft on branch draft/trying-temper, and the UX pass's).
