# Pair: one agent in its own copy

Draft, 2026-11-27, from sessions with the user on 2026-11-26 and 2026-11-27. The try's draft sketched a person working "quite like a coding agent", with one agent that "has its own repo checkout/workspace and works directly in it", and "Land this" as the person's act (its section 4.8, on branch draft/trying-temper), leaving the build to "a draft of its own", this one. On 2026-11-26 the user and the terminal's designer, drawing Pair's frames, worked out how "Land this" works underneath, the flow this draft takes, the user's "preferred flow, no extra turn needed for /land". On 2026-11-27 the user: a pair lands its own edits alone; one change at a time; "basically it should be like a 'connected implementer'"; "'pair' might just be a name we give to a way, not the mechanism itself, which is generic"; and "'land this' is entirely mechanical": "when he thinks he's done he calls 'deliver'", which prepares "the changes themselves, the metadata", the client offers it, "accepting the proposal triggers a mechanical 'finish'", and "the agent is not given the tools to land, so there's nothing to refuse"; the duplicate checks between them "tbd". A first version, reviewed once, made the pair the change's producer; this one keeps the review's concerns and follows the flow. A second review, on the user's word (2026-11-27), found what code must hold: the agent's write bound, the offer it names, the open change's repairs, the copy's move, funds and upgrades. The idle bound of three days is the user's (2026-11-27).

## 1. In one page

- **A pair is an agent in a conversation with its person, holding a copy of the repository of its own** (3.1, 3.2): a branch and a warm checkout, kept between turns, saved at each turn's end. It edits and runs there, as an implementer does, and talks as a chat does.
- **It writes only there** (3.3): its tools are an implementer's, no wider; the host pushes only to the pair's own refs; nothing it holds opens, lands or hands in a change.
- **When it thinks it's done, it delivers** (3.4): the `deliver` hook checks the copy within its turn, so that it fixes what fails before its person hears of it, and a passing delivery moves the pair's offer to the checked head, with the change's title and description. That delivery is its offer: "Land this?".
- **"Land this" is the person's act, done by code, with no agent turn** (3.5): the person accepts one offer by its number; code opens a change of the person's own from exactly that commit into the base, and the queue lands it. Whether something lands never depends on the agent calling a tool.
- **The open change's repairs are the pair's while the pair is open** (3.6): a red CI or a review asking for changes goes to the agent, or, while it waits for its person, to them as one ask.
- **One change in flight, and the pair goes on** (3.7): after it lands, the work done since the offer moves onto the new base.
- **Bounded** (3.8): no worker held between turns, a turn budget with a cap, an idle ask at three days, a cap on live pairs.
- **Pair is a way** (section 4): an agent recipe with `holds = ["copy"]` and `conversation = true`, offering its deliveries as a change whose producer is `handed`.

## 2. Today

- **A chat converses and only inspects:** an Open run whose text replies wait and park; its charter grants no `modify` and no `shell` (`../domain/agent/tools.md`). Open requires `wait` and forbids `deliver` (smith's `domain/run.md`, section 1). jig derives a run's mode from how its task ends: a requester-ended task's run is Open, a closer-ended one's Finite.
- **A goal's run delivers to its own branch and goes on:** a Finite run, "Delivering isn't finishing": its `finish` offers a report or a failure, never a change (`../domain/agent/tools.md`). A design delivery carries Markdown alone; code goes through a change. A Finite run's text reply ends its turn unfinished.
- **`deliver` checks, then the host commits and pushes;** `save` pushes unchecked and is granted only with `deliver`; a push with nothing new returns nothing, and a stale one ends the run (smith's `domain/run-delivery.md`, 1.2 and 1.5). The stock implementer holds no `shell`.
- **A person opens a change from a pushed branch** with `OpenChange`, a title and a body, the person its requester: the change takes the source's head, or the `at` named, failing where the head is another commit, onto a branch of its own; its repairs are temper's, automatic, and a later push to the source doesn't reach it (`../domain/forge/change/opened.md`).
- **A task holding a branch is tracked and ended by its closer** (`ways-recipes.md`, section 2, Derived).
- **Away** is each person's switch for which decisions wait for them; it pauses no task (`../domain/forge/attention.md`, section 4).
- **Hooks:** `.temper/hooks/<moment>`, a link allowed only to a bare name beside it, as fabrig's `finish` to `deliver` ([hooks.md](hooks.md), section 3).

## 3. The pair

### 3.1 Its copy and its offer

- **Two refs of the pair's own,** made from the base when it starts: its **copy**, which saves move, and its **offer**, which only a passing delivery moves. A checkout of the copy on a worker is kept warm between turns, in the workstream cache, bounded and evictable as today (`../domain/worker/checkouts.md`, section 1), and prepared afresh at the copy's head where it isn't. Its person's own checkout is never touched.
- **Saved at each turn's end** that changed it, by the host, after the turn's writers have settled: the files a `save` takes and no others (smith's `domain/run-delivery.md`, 1.5), pushed unchecked to the copy under an expected-head lease. Starting a pair agrees to these saves: the way's line says so. A save refused, for a conflict marker, or stale, keeps the last saved copy; the agent and the person are told at once that the edits since are on the worker alone.
- **Where they live:** on the project's repository, where change branches live today, readable as theirs are; a project on a public repository publishes a pair's saves as it publishes a change's.
- **No one builds on them:** a pair's refs are refused as an `OpenChange` source, a dependency's source or an early start, but by "Land this" (3.5).

### 3.2 Its run

- **Finite, in conversation:** a run whose text reply parks it, as a chat's does, and which keeps a Finite run's `deliver`. smith's Finite run gains this mode; jig gives it by the recipe's `conversation`, not by the task's ending (section 4), and keeps the park as a wait for words, so that a pair waiting on its person is never "nothing left to await" and spends no try.
- **What parks it:** a reply, or an explicit `wait`, each after the turn's save. **What wakes it:** its person's words, or their yes to a repair ask (3.6). **What ends it:** a `finish` with a report or a failure, or its person's close; either ends the pair. A budget wind-down parks with a reply saying what is left, never finishing on its own. Words that arrive during a turn reach that turn, as a chat's do.
- **No closer:** a pair holds a copy, not a branch: its copy never lands, so it is neither tracked nor ended by a closer (`ways-recipes.md`, section 2).
- **Its participant** is its requester, the person who started it, alone: another party's words to it are refused, naming the person.
- **Its history** resumes from the transcript, compacted as smith compacts (smith's `domain/session-context.md`, section 1); a fresh start recovers the copy from its ref, and the offer, its state and any open ask from the host, which holds them.

### 3.3 Its tools, and their bound

- **An implementer's, no wider:** `inspect` and `modify` in its copy, its program tools, `deliver`, and `finish` with a report. No `shell` until the worker's file and network confinement has landed (the confinement draft, `confinement.md`, sections 5.3 and 5.4); until then a pair runs what an implementer runs, through its program tools and hooks.
- **The host pushes only to the pair's copy and offer;** the agent holds no forge credential, as an implementer holds none today (`../domain/forge/writes.md`, section 2).
- **Nothing it holds starts work:** no `change`, `goal` or proposal tool. Admission refuses any change, landing or task a pair's run asks for, through a tool, a batch, a proposal or an amendment, under its charter or a project's own; "Land this" is the person's (3.5).
- **Its sub-agents inspect only:** a review in place, when the person asks for one, is a sub-agent granted `inspect` alone, on a model the charter lists; smith's sub-agent may not take its parent's `modify` (smith's `domain/run.md`, 5.3).
- **Its charter, `pair`:** work with the person, show what you change and why, ask when unsure, and deliver when you think it's done, with the change's title and description.

### 3.4 Its delivery is its offer

- **`deliver { title, body }`,** smith's, in the agent's turn: the `deliver` hook checks the copy; a refusal comes back to the agent with its output's tail, as smith gives one today, so that it fixes and delivers again before its person is asked anything.
- **A passing delivery** is committed and pushed to the copy, and the offer is moved to that commit. A delivery of a tree already saved and unchanged is a delivery too: it checks the saved commit and moves the offer to it, with no new commit.
- **The host keeps the offer** with a number of its own: the commit and its tree, the base it was checked against, the `deliver` hook's outcome and the hook's identity (3.5), and its words, the title, at most 60 characters for a person, and the description.
- **The person sees it** as a prompt drawn by code, "Land this?", its line saying what the landing waits for ("lands after CI") computed from the way's resolved landing values (`ways.md`, 5.4), never written by the agent, and how many edits came after it.
- **A newer delivery replaces it;** edits after a delivery stay in the copy, saved, and the offer stands for its commit.

### 3.5 "Land this"

The person accepts an offer by its number (in the terminal, `temper ctl task land TASK OFFER --key K`). Code then, with no agent turn:

1. **Refuses a superseded offer,** naming the newer one, and refuses while another change of the pair's is in flight (3.7) but where it is the open change's repair (3.6).
2. **Fences the pair:** no turn starts until it is done, and a running turn ends first.
3. **Runs the `finish` hook only where it is another hook,** resolved in the offered tree: where `.temper/hooks/finish` is a link to `deliver` by the hooks' link rule, as fabrig's is, or neither exists and one check stands for both, the offer's check is the finish's. Two files with the same bytes are two hooks. A distinct `finish` runs on a checkout of the offered commit, bounded as a hook is ([hooks.md](hooks.md)), and a run that changes a tracked file is refused. A refusal goes back as feedback (3.6).
4. **Opens the change** as `OpenChange` does, from the offer ref at the offer's commit, onto a branch of the change's own: into the base, with the way's resolved landing values and the offer's words, the person its requester and its root, funded by the amount the person accepts, which the prompt shows. The pair's record links it. A retry with the same key returns the same change.
5. **The queue lands it,** as any change, under the floor and the way's gates; its proof and gates hold for whatever candidate the queue makes of it.

### 3.6 The open change

- **Its repairs are the pair's while the pair is open:** a red CI or a review asking for changes holds the change, the hold naming the pair, instead of temper's automatic repair. Its other holds keep their own release.
- **The agent hears it** with what failed, tagged with the change's head, and offers the fix as an ask ("2 checks failed. Fix them?"). **While the pair waits for its person,** no turn runs: code asks the person once, deduplicated, and their yes wakes the agent with it.
- **A repair starts from the change's head:** the change's head is brought into the copy first, so that what the queue did to it is there. Its delivery is an offer for that change: accepted, it updates the change at the head the repair started from, after the change's writers and queue entry have settled; the `finish` hook runs as 3.5 says, stale evidence is dropped, and the change is gated again.
- **Closing the pair** releases the hold to temper's automatic repairs; the change stays open, its person's. **Closing the change** ends its repairs, and the pair may offer again.

### 3.7 One change in flight, and after

- **While a change lands,** new edits and deliveries stay in the copy and its offer; a new offer can be accepted once the change has landed or closed, or as its repair (3.6).
- **After it lands,** the work done since the accepted offer moves onto the landed base: under the pair's fence, the copy's commits after the accepted head are replayed onto the base's new head, not merged, so that what a repair dropped or a squash rewrote doesn't come back. The copy is pushed under its lease. A conflict keeps the old copy, goes to the agent as feedback, and survives a lost worker, since the old copy is on the forge. Offers made against the old base are retired: the next needs a fresh delivery.
- **The pair's states,** as both clients draw them: pairing; finishing its step, then pausing; paused; asks you; landing, waiting on CI; landed; closed.

### 3.8 Its bounds

- **No worker held between turns,** and no writer lease: between turns nothing is live, as for any parked run.
- **Its funds:** the pair's own budget, capped and cumulative, pays its turns; each change it opens is funded apart, by the person at "Land this" (3.5). A repair spends its change's funds; a top-up is the person's act.
- **Idle three days** (the user, 2026-11-27), the project's setting: past it the pair asks its person once to keep it or close it, and nothing closes by itself.
- **Live pairs** are capped per person, the project's setting, four unless set; a start past it is refused, naming the open ones.
- **Closing** ends its run, deletes its copy and offer refs, and releases its funds; a change it opened stays open, its person's.

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
offers = "land"

[tasks.land]
executor = "change"
into = "base"
producer = "handed"
gates = []
landing = { ci = true, up_to_date = true }

[reaches]
```

- **`holds = ["copy"]`** is an agent's copy and offer, its warm checkout (3.1); its holder has no closer (3.2). **`offers = "land"`** names the change recipe its offers open: an edge only its person crosses, by "Land this", making a change rooted at the person, not a delegate of the pair. **`producer = "handed"`** is a change whose head its requester hands in, as `OpenChange`'s is. All are the ways format's (`ways-recipes.md`, sections 2 and 3).
- **Generic:** any way may give an agent a copy and an offered change; "pair" is the stock's name for this one. Pair adds no gate of its own; a floor's review still holds.
- **Landing as the project lands:** its change takes the project's mode, queue or turn, and joins a turn as any change does; it needs no turn of its own.
- **From `temper ctl` alone:** `temper ctl chat start --project P --way pair --budget B "WORDS"` (`ways.md`, section 4), printing the pair's task; `temper ctl task message TASK "WORDS"` to talk; `temper ctl task show TASK` to read the pair, its offer (number, commit, words, edits since, what landing waits for), its change and its asks; `temper ctl task land TASK OFFER --budget B --key K` for "Land this", answering stale or busy where it can't; `temper ctl task answer` for its asks; `temper ctl task cancel TASK` to close.

## 5. Upgrades

- **New stored families:** the pair's record, its offer and its link to its change are new layouts, read by the next build by the upgrades rule (`../protocol/wires.md`, sections 6 and 10). The change a pair opens is an ordinary opened change, and its hold an ordinary hold naming the pair, so a build that doesn't know pairs reads both as today's.
- **Workers:** a pair's assignment and the acceptance's hook job require the mode; an older worker lacks it, so neither is placed there and both wait for a worker that has it (`../protocol/wires.md`, sections 5, 8 and 10). A chat and a goal reach an older worker as today.
- **Clients:** `task land` and the offer are new on the client's wire: a browser at another layout reloads, and a terminal client at another layout says it is incompatible and asks to be upgraded (`../protocol/wires.md`, section 9).
- **Main stays redeployable:** a project with no pair runs as today, and a pair starts only once its build is on the engine.

## 6. Later

- **`shell` for a pair,** once the worker's confinement has landed (3.3).
- **Handing over from a pair** to another way (the user, 2026-11-26: later, with moves between ways).
- **One pair, one change,** closed at landing, as a way of its own, if wanted.
- **The pages and the terminal's look:** the frames both clients drew on 2026-11-26, "Land this?", the diff, the states (the UX pass's).
