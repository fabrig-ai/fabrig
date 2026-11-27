# Pair: one agent in its own copy

Draft, 2026-11-27, from sessions with the user on 2026-11-26 and 2026-11-27. The try's draft sketched a person working "quite like a coding agent", with one agent that "has its own repo checkout/workspace and works directly in it", and "Land this" as the person's act (its section 4.8, on branch draft/trying-temper), leaving the build to "a draft of its own", this one. On 2026-11-26 the user and the terminal's designer, drawing Pair's frames, worked out how "Land this" works underneath, the flow this draft takes, the user's "preferred flow, no extra turn needed for /land". On 2026-11-27 the user: a pair lands its own edits alone; one change at a time; "basically it should be like a 'connected implementer'"; "'pair' might just be a name we give to a way, not the mechanism itself, which is generic"; and "'land this' is entirely mechanical": "when he thinks he's done he calls 'deliver'", which prepares "the changes themselves, the metadata", the client offers it, "accepting the proposal triggers a mechanical 'finish'", and "the agent is not given the tools to land, so there's nothing to refuse"; the duplicate checks between them "tbd". A first version, reviewed once, made the pair the change's producer; this one keeps the review's concerns and follows the flow.

## 1. In one page

- **A pair is an agent in a conversation with its person, holding a copy of the repository of its own** (section 3): a branch and a warm checkout, kept between turns, saved at each turn's end. It edits and runs there, as an implementer does, and talks as a chat does.
- **When it thinks it's done, it delivers** (3.3): the `deliver` hook checks the copy within its turn, so that it fixes what fails before its person hears of it, and a passing delivery pushes the checked head with the change's title and description. That delivery is its offer: "Land this?". It holds no tool that lands or opens a change.
- **"Land this" is the person's act, done by code, with no agent turn** (3.4): a change opened from the delivered head into the base, with the way's landing and the delivery's words, and the queue lands it. The delivery's check stands for the finish's where the two hooks are one; otherwise code runs the `finish` hook. Whether something lands never depends on the agent calling a tool.
- **Failures come back as feedback** (3.5): a red CI or a review goes to the agent, which fixes and delivers again; while the person is away, code asks them instead.
- **One change in flight at a time, and the pair goes on** (3.6): while one lands, new edits wait in the copy for the next; after it lands, the copy moves to the new base.
- **Pair is a way** (section 4): an agent recipe with `holds = ["copy"]` and `conversation = true`, delegating a change whose producer is `handed`.

## 2. Today

- **A chat converses and only inspects:** an Open run whose text replies wait and park; its charter grants no `modify` and no `shell` (`../domain/agent/tools.md`). Open requires `wait` and forbids `deliver` (smith's `domain/run.md`, section 1).
- **A goal's run delivers to its own branch and goes on:** a Finite run, "Delivering isn't finishing": its `finish` offers a report or a failure, never a change, and what it delivered is on its branch once the receipt says so (`../domain/agent/tools.md`). A design delivery carries Markdown alone; code goes through a change.
- **`deliver` checks, then the host commits and pushes;** `save` pushes unchecked and is granted only with `deliver` (smith's `domain/run-delivery.md`, 1.2 and 1.5).
- **A person opens a change from a pushed branch** with `OpenChange`, a title and a body, the person its requester; the change has no producer, and its repairs are temper's (`../domain/forge/change/opened.md`).
- **A change's words:** the pull request's title is the producer's `finish` title, its body the description, and a branch handed in with no title lands as "Update repository" (`../domain/forge/change/README.md`).

## 3. The pair

### 3.1 Its copy

- **A branch of the pair's own,** made from the base when it starts, and a checkout of it on a worker, kept warm between turns where the worker can; prepared afresh at the branch's head where it can't. Its person's own checkout is never touched.
- **Saved at each turn's end** that changed it, by the host, unchecked, as a `save` saves (smith's `domain/run-delivery.md`, 1.5), so that the copy is on the forge, a page can show its diff, and the next turn starts from it. Starting a pair agrees to these saves: the way's line says so. A save refused, for a conflict marker, or stale, is told to the agent and the person at once.
- **No one builds on it:** the pair's branch is no change and no dependency's source. What lands goes through a change opened from it (3.4).
- **No worker held between turns:** between turns nothing is live, as for any parked run.

### 3.2 Its run

- **A goal's run, in conversation:** Finite, its `finish` a report or a failure that closes the pair, never a change; it delivers to its own branch and goes on, as a goal's run does, its deliveries carrying code. Its text replies reach the person and park it, as a chat's do: smith's Finite run gains the conversation a chat has, and jig keeps the reply's park as a wait for words, so that a pair waiting on its person is never "nothing left to await", and spends no try.
- **Its tools:** `modify` and `shell` in its copy, granted explicitly, `deliver`, and `finish` with a report. No tool opens a change, lands one or hands one in.
- **Its participant** is its requester, the person who started it, alone: another party's words to it are refused, naming the person.
- **Its charter, `pair`:** work with the person, show what you change and why, ask when unsure, and deliver when you think it's done, with the change's title and description. A sub-agent, read-only, on a model the charter lists, is a review in place when the person asks for one.
- **Its history** resumes from the transcript, compacted as smith compacts (smith's `domain/session-context.md`, section 1); a fresh start recovers the copy from its branch and the delivery and its state from the host, which holds them.

### 3.3 Its delivery is its offer

- **`deliver { title, body }`,** smith's, in the agent's turn: the `deliver` hook ([hooks.md](hooks.md)) checks the copy; a refusal comes back to the agent with its output's tail, as smith gives one today, so that it fixes and delivers again before its person is asked anything. A passing delivery is committed and pushed to the pair's branch, the checked head, and the host keeps it with its words: the title, at most 60 characters for a person, and the description.
- **The offer:** the latest delivery, drawn by code as a prompt, "Land this?", its line saying what the landing waits for ("lands after CI") computed from the way's resolved landing values (`ways.md`, 5.4), never written by the agent. In the recipe it is the `land` delegation, proposed and held for the person (`ways-recipes.md`, section 2).
- **A newer delivery replaces it.** Edits after a delivery stay in the copy, saved, and the offer stands for the delivered head, showing that edits came after it.

### 3.4 "Land this"

The person accepts the offer (in the terminal, `temper ctl task land TASK`). Code then, with no agent turn:

1. **Waits for the agent's turn to end,** so that nothing writes while it lands.
2. **Takes the delivered head,** not the copy's: what lands is what was checked and offered. Edits after it wait for the next delivery.
3. **Runs the `finish` hook only where it is another hook:** where `.temper/hooks/finish` is the `deliver` hook, the same file or a link to it, as fabrig's is, the delivery's check stands for the finish's, and nothing runs twice. Otherwise code runs the `finish` hook on the delivered head, and a refusal goes back as feedback (3.5).
4. **Opens the `land` change** from that head, as `OpenChange` does: into the base, with the way's resolved landing values and the delivery's words, the person its requester. A delivery carries its words, so a pair's change never lands as "Update repository".
5. **The queue lands it,** as any change, under the floor and the way's gates.

### 3.5 Failures

- **A refused `finish` hook, a red CI, or a review asking for changes** on the open change comes back to the agent with what failed, and the agent offers the fix as an ask ("2 checks failed. Fix them?"). Its next delivery, accepted, updates the open change rather than opening another, while that change hasn't landed.
- **While the person is away,** the pair is paused: a failure becomes an ask, by code, with no turn run, and goes to the person's Needs you, as Pair's `[reaches]` asks everything.
- **A change its person closes** hands its repairs to temper's, as a change opened from a branch does today.

### 3.6 One change in flight, and after

- **While a change lands,** new edits and deliveries stay in the copy and its branch, not handed in, so that its CI doesn't restart; the next accepted delivery opens the next change once the first has landed. Adding them to the open change is the person's choice, by accepting a delivery while it is red or held (3.5), never the default.
- **After it lands,** the pair goes on: its copy moves to the base's new head, merging what it holds. A conflict there goes to the agent as feedback.
- **The pair's states,** as both clients draw them: pairing; finishing its step, then pausing; paused; asks you; landing, waiting on CI; landed; closed.

### 3.7 Idle, and its bound

- **Paused when its person leaves:** the agent finishes the step it was on, then waits; a question comes only at the end of that step, or from outside it, as a red CI.
- **An idle bound,** the project's setting, three days unless set: past it, the pair asks its person to keep it or close it, and nothing closes by itself. Closing releases the branch's hold and the pair's funds; a change it opened stays open.
- **Its funds:** the pair's budget pays its turns; each change it opens is funded by the person at "Land this", as `OpenChange`'s is.

## 4. As a way

```toml
# .temper/ways/pair.toml
format = 1
headline = "You and one agent, in its own copy"
line = "You work with one agent in its own copy, saved as it goes; when it delivers, you say Land this."
start = "pair"

[tasks.pair]
executor = { agent = "pair" }
contract = "report"
conversation = true
holds = ["copy"]
delegates = { land = {} }

[tasks.land]
executor = "change"
into = "base"
producer = "handed"
gates = []
landing = { ci = true, up_to_date = true }

[reaches]
```

- **`holds = ["copy"]`** is an agent's branch of its own and its warm checkout (3.1). **`producer = "handed"`** is a change whose head its requester handed in, as `OpenChange`'s is. Both are the ways format's, kept for this (`ways-recipes.md`, sections 2 and 3).
- **Generic:** any way may give an agent a copy and a handed change; "pair" is the stock's name for this one. Pair adds no gate of its own; a floor's review still holds.
- **From `temper ctl` alone:** `temper ctl chat start --project P --way pair "WORDS"` (`ways.md`, section 4), `temper ctl task message TASK "WORDS"` to talk, `temper ctl task show TASK` to read the pair, its delivery and its change, `temper ctl task land TASK` for "Land this", `temper ctl task cancel TASK` to close.

## 5. Upgrades

- **A Finite run in conversation** is a charter an older agent's layout can't run: it is refused to an older worker, and the run waits for one that has it (`../protocol/wires.md`, sections 5 and 8). A chat and a goal reach an older worker as today.
- **`task land` and the offer** are new on the client's wire: a browser at another layout reloads, and a terminal client at another layout says it is incompatible and asks to be upgraded (`../protocol/wires.md`, section 9).

## 6. Later

- **Handing over from a pair** to another way (the user, 2026-11-26: later, with moves between ways).
- **One pair, one change,** closed at landing, as a way of its own, if wanted.
- **The pages and the terminal's look:** the frames both clients drew on 2026-11-26, "Land this?", the diff, the states (the UX pass's).
