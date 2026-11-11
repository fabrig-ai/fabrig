# The agent

Provisional, 2026-10-05, revised 2026-11-05. temper's agent is smith's:
the standard agent built from smith, the kit for LLM agents, on skein
like temper. smith's design is its own (smith's `domain/run.md`,
`smith/docs/design/domain/session.md`, `smith/docs/design/domain/tools.md`, `smith/docs/design/domain/host.md`), and how a
run's charter, tools, contract, messages and delivery become smith's is
jig's (jig's `domain/hosts/charters.md` and `jig/docs/design/domain/hosts/README.md`, section 7). This document says
what temper fills in: the charters its engine writes, the host tools its
engine serves (`tools.md`, the one table of them), its result contracts and
conventions, delivery as a push, and the agents temper runs. What is
still open is listed in `README.md`, section 8.

## 1. In one page

- **temper runs smith's stock agent,** on workers, one process per run:
  temper's own binary run as `temper-box internal agent`, smith's agent shell
  (`../../protocol/binaries.md`). temper builds no code into it. A
  run's charter and its host's answers are all that make it temper's.
- **The engine writes the charter** (`charter.md`): the task's instructions,
  its brief rendered, every host tool, a result contract with temper's
  verdicts and a change's `title` and `body`, `.temper/pre-pr` and
  `AGENTS.md` as conventions, and a budget in the deployment's unit with
  its models' prices.
- **The host tools are temper's** (`tools.md`): jig's core's tools under temper's
  own names, and the forge's reads, effect and topics.
- **A standing role** (`standing.md`) is jig's task that stands under a
  charter of temper's: an agent that owns some of a project's work for
  the project's life, its Now its memory, its requester its person.
- **The worker is smith's host** (section 5; ../worker/README.md): it prepares the
  workspace from the forge, supervises the agent's process with smith's
  host domain, delivers a change by pushing it, and relays the rest to
  the engine.
- **The engine keeps what smith leaves to hosts:** transcripts, turn by
  turn, in its store; host tools' calls, decided once; what follows an
  answer (jig's `domain/engine/executors.md`).
- **One translation.** `temper-engine-smith`, which the
  engine's routing calls, is the only place where temper's charter,
  tools and results become smith's, and smith's back. It uses
  `jig-protocol-charter` to assemble and translate the charter, supplying
  temper's host tools, workspace delivery fact and conventions (`charter.md`;
  jig's `domain/hosts/charters.md`). Its `Tool` vocabulary is its own,
  rather than `jig-protocol-tools`' (`../../protocol/llm.md`, sections 4
  and 5). smith interprets no temper vocabulary, and the worker none of
  either.

## 2. The agent in the system

```
engine   jig's core and the forge, the store; writes each run's charter and serves its tools
worker   smith's host: workspaces, agent processes, pushing their changes; relays the rest
agent    smith: LLM work, one run per process, reporting to its worker
```

- **The engine is the only client** of the store and of the forge. It
  answers the host tools a run calls, and commits the run's turns and
  result. The worker's one write is pushing a run's branch, over git.
- **The engine never reads a repository.** It works from what the forge
  connector reads; what a workspace holds, such as a repository's
  `AGENTS.md` or its checks, is for the run to find.
- **The agent acts on the world only through the worker:** a delivery,
  which the worker makes as a push, and host tools, which the worker
  relays to the engine. It never pushes, merges or writes to a system on
  its own authority.
- **Retrying a failed run is decided above the run,** by the engine,
  across attempts. A run answers once and is done.

## 3. What temper takes from smith

- **In the domains:** the engine's root links no smith crate; charters,
  transcripts and results are bytes to jig's core (jig's
  `domain/hosts/hub.md`, 8). The worker's root composes `smith-host-domain`
  as its agent child (../worker/README.md, section 3). `temper-engine-smith`, the
  translation of `charter.md`, is step code over `smith-domain`,
  `smith-domain-run` and `smith-domain-session`.
- **In the protocol layers:** smith's codecs, its host half and its agent
  shell (`../../protocol/components.md`).
- **In the worlds:** smith's agent domain over skein's fake LLM, its
  agent processes, and its codecs (section 7).
- **LLM accounts are jig's,** made from the client, on skein's accounts
  driver. A run's grants reach it through its worker and smith's host
  half: for each LLM provider it uses, a credential with its scheme, or
  a program source's name, whose login the host checks and the program
  reads (`../../protocol/credentials.md`, section 3; jig's
  `protocol/llm.md`, section 6). The agent runs the loop itself, or
  through the program (smith's `protocol/programs.md`, section 2).

## 4. Messages

What a task hears while its run is live reaches smith as messages, as
jig's `domain/hosts/README.md`, section 7 says: each named, with its sender's
label, its kind and its words whole, the start carrying those waiting
in the task's inbox. Words read `2: …`; a question reads `2 asks
(question 4): …`, its number the one `message`'s answer form takes;
an answer reads `1 answers question 4: …`. A person's inbox answer
notifies the asked task with `Person answered task 2's question 4 in their inbox; task 2 has the answer and does the work:\n…` when task 2 asked it.
A question the receiving task asked itself keeps `Person answered question 4 in their inbox:\n…`,
including the answer words within 256 bytes, cut at a UTF-8 boundary and
marked when cut; an empty stored notice keeps the line without words.
So a goal hears its person's answer to question 4 by the number it asked,
and a chat, which answers no question waiting in its person's inbox (`agents.md`, 2), hears that
question 4 was answered, with the person's words and the asker that owns the work. Proposals and
escalations waiting for the task's decision are relayed the same way, each naming what `decide` or
`decide_escalation` takes.

## 5. Delivery is a push

smith's `deliver` is the worker's push (../worker/checkouts.md): the
worker commits exactly the tree the run checked, with the result's
`title` and `body`, and pushes it as a fast-forward, never forced, to the
branch the assignment names; a merge in progress is committed as the
merge, with both parents. Its outcomes, in smith's terms (smith's
`domain/run.md`, 8.2):

| The push | smith's delivery |
|---|---|
| landed in every repository with a change, or already landed by this hold's last push | delivered, each receipt `pushed <head> to branch <branch>; your checkout stays detached, as expected` |
| nothing to push | nothing |
| a branch moved, as a fast-forward cannot take it or its head is not the one expected | stale: the run ends, and the engine decides afresh |
| a conflicted file still holds a marker | refused, naming the file |
| refused by the forge, unreachable, timed out, broken, too large, missing the repository, a branch or a commit | failed, with the reason and the last 512 bytes of git's diagnostic output |

A goal's design delivery to its goal branch (`../forge/branches.md`, 3; ../worker/checkouts.md
) has outcomes of its own, and is never stale:

| The design push | smith's delivery |
|---|---|
| landed, the goal branch at the head it started from | delivered, `pushed <head> to branch <branch>; your checkout stays detached, as expected` |
| landed, the goal branch's moved tip merged first | delivered, `pushed <head> to branch <branch>, merging its tip <tip> first; your checkout stays detached, as expected` |
| a path that isn't Markdown | refused, naming the path: `a delivery to the goal branch carries Markdown only; code goes through a change` |
| the moved tip in conflict | refused, naming the files in conflict, the merge left in progress to settle and deliver again |
| the branch moving at each of three tries | refused: `the goal branch kept moving; deliver again` |
| a `save` | refused: `the goal branch takes checked deliveries alone; deliver instead` |

What it delivers is the tree, not the run's history: a checkout is detached
(../worker/checkouts.md), and the worker commits on its own record of the head,
never on HEAD (`../../protocol/git.md`, 4.7). So a run's own commits, resets and
rebases keep their files and lose their history, and a saved tree that is
delivered unchanged lands the save's commit, the delivery's title and body
unused (../worker/checkouts.md). The head delivered names what is on the branch,
and the coding charter's words say so (`../../protocol/configuration/stock.md`'s
charter 2 row), since a run can't otherwise tell.

A `save` (`tools.md`) is the same push with no check run before it, and marked
unchecked: its commit's title is `Unchecked: work in progress, saved before a
question`, with no body, and a landed save's receipt reads `saved <head> to
branch <branch>, unchecked: no check ran; your checkout stays detached, as
expected`. Its other outcomes are the table's. A save's push moves the
change's own branch as a delivery's does, so the engine hears of it as of
any push the run left (../worker/README.md, 4.2). When it accepts the head the
attempt left under its writer claim, it records that head in the change as
well as the branch, however the head reached it: reported in the run's
answer, or found by the read that frees the attempt's writer when no answer
reported it (jig's `domain/tasks/holds.md`, 3; `../forge/change/README.md`, 1).
That holds whatever the delegate: a producer, a repair or a resolution, a
goal's landing's among them, whose branch is the goal's branch held under the
goal's hold. The next attempt's pull section pins that head
(`../forge/change/repairs.md`). Recording the head completes no production, repair or
resolution: the save remains unchecked, and the forge opens no pull request
on it (`../forge/change/README.md`, 1). In its hold, the next checked delivery, or the
worker's own save of saved work, commits afresh on a save's head even with
the tree unchanged (../worker/checkouts.md), so the head a finish delivers
carries the delivery's title and body, never the unchecked mark. A later
hold that starts from that save's head and delivers it unchanged lands it as
it is: its tree is the one that finish checked, and its title still says
unchecked.

A receipt says it in words, whatever the charter, so that a run that finds its
checkout detached does not take a good delivery for a failure: its full head,
the branch it landed on, and that a detached checkout is expected. A branch name
that would take a receipt past its bound (512 bytes, smith's `Receipt`) leaves
the branch's words out and keeps the rest.

A head in words is spelled as git spells it: 40 digits on SHA-1, 64 on SHA-256.
The twelve zero bytes that fit a SHA-1 id in temper's 32 are never printed, since
a run copies what it reads. That holds for a receipt and for every other word
that prints a whole head, a cancel's reason among them.

A push that lands is the result, even if the run was winding down. The
answer carries what the run left on the forge (../worker/README.md, 4.2), and the
forge's change procedure goes on from it (../forge/change/README.md).

## 6. Turns and transcripts

Turns are smith's bytes (smith's `domain/session.md`, section 3), which
the worker relays and the engine keeps opaque, by conversation and
window (jig's `domain/engine/executors.md`, 2; jig's `domain/hosts/hub.md`, 4).
temper's client decodes them as jig's client does (jig's
`client/domain/client/conversations.md`). A run that resumes is given its
transcript's current window and the calls committed after its last turn;
past the resume limit, or when smith refuses the transcript, the next
run starts fresh, its brief carrying the tail when it is readable. If
the saved window is unread, the first load spends a transient try; its
retry starts fresh without the tail, keeping the person's pending words
(jig's `domain/engine/executors.md`, 2).

An unidentified startup fault keeps its agent-failure classification and
bounded operator diagnostic through cleanup (jig's `domain/hosts/hub.md`, 3).
It retains the transcript and pending words for the next attempt; it does
not establish a transcript refusal. A persistent fault exhausts the agent
allowance and holds the chat. An unidentified fault alone never takes
the fresh preparation above.

## 7. The world

smith's worlds test the agent itself (smith's documents, each section
"The world"). temper's test what it fills in, and the agent under its
host:

- **`temper-engine-smith`'s step tests:** the root's charter becomes
  smith's start, its brief in order and its budget exact; every declared
  tool decodes its least shape and refuses a wrong one; `amend` and
  `release` name their revision; every answer renders as smith's text
  within smith's limits, or as an error; a job's answer names its failed
  step before its log; smith's terminals keep their spend and the root's
  contract.
- **The routing's tests:** a call decoded into the root's `Tool`; one of
  another activation, or of an unknown tool, answered as an error with no
  commit; a result that does not decode failing its attempt as invalid;
  a change's result kept only with its settled resource; each settled
  delivery the hub's answer (`../../protocol/llm.md`, section 6).
- **The worker's and the system worlds** (../worker/README.md, section 9): smith's
  scripted agent for the agent child, and smith's real agent domain over
  skein's fake LLM, its tools working in the trees the worker prepared.
- **Stories that are temper's:** a change produced, checked and pushed,
  and its push found moved; a coordinator that delegates a batch, hears
  results, and proposes what is beyond its authority; a chat that waits,
  hears its person's words, parks, and resumes from its transcript; a
  host tool answered busy by the engine, asked again and decided once.

## 8. Open questions

- Ask temper before M3b: `ask-temper.md`, section 5.

- A real chat choosing `change`: `tools.md`, section 3.

- **Forge-read fallout with real agents** (`agents.md`): the existing
  ChatGPT and Claude Code owned-forge landing stories check that a real
  run completes without losing `temper-box serve`. Refusal-rendering checks
  alone do not establish either outcome. Default: those live stories
  run after landing, with no change required by the refusal contract;
  `../forge/README.md`, section 11 keeps the routing proof's limits.

- A direct change result's rendering room: `charter.md`, section 6.

- Ask temper changing the catalogue: `agents.md`, section 5.

- A real model asking its person: `agents.md`, section 5.
- A permission for its own requester alone: `agents.md`, section 5.

- Proposal-form diagnostics: `tools.md`, section 3.

- A real model revising a proposal: `agents.md`, section 5.

- A goal as the designer: `agents.md`, section 5.
- Constraining models in settings reads and proposals: `agents.md`, section 5.

- **What a delivery keeps, under a deployment's own charter** (6): the stock's
  coding charter and the reference's tell a run its own history isn't kept. A
  deployment's own charter is its author's, so a run under one isn't told.
  Default: so.
- A charter where the process has no local agent: `tools.md`, section 3.
- A `see` answer by its worker's reply bound: `ask-temper.md`, section 5.
- The person's forge identity in a chat's brief,: `agents.md`, section 5.
- A collaborator's login in `read_collaborators`: `tools.md`, section 3.
- `read_permission`'s login held to the settings family: `tools.md`, section 3.
- A person known by a login alone: `agents.md`, section 5.
- Which host tools a charter offers: `tools.md`, section 3.
- What sizes temper's own way: `agents.md`, section 5.
- QA in temper's own way: `agents.md`, section 5.
- Ask temper woken by a landing: `agents.md`, section 5.
- Who may name a plan's members of `change`: `tools.md`, section 3.
- A goal's question kept with its chat: `agents.md`, section 5.
- Who else writes a question's line form: `agents.md`, section 5.
- A change's foremen's LLM choice: `tools.md`, section 3.
- **A larger whole compare** (`tools.md`). Nine files, nine commits or one line
  changed in a long file can exceed the stock's forge bounds. Raising the
  row bound alone cannot fit both whole sides of a long file; a paths-only
  comparison would change what the settled answer holds. Default: keep
  the bounds and direct the run to pull-file pages and selected file reads,
  until a real landing shows that recovery cannot serve the change.
- A cancel's grace: `tools.md`, section 3.
- **A code graph** for a workspace (../worker/checkouts.md, section 1), as an MCP
  tool source once smith has one.
- A change produced by a job of any kind: `tools.md`, section 3.
- A document moved within one call: `ask-temper.md`, section 5.
- Parts of a settings proposal across a restart: `ask-temper.md`, section 5.
- How the settings' addresses read: `ask-temper.md`, section 5.
- A revision's values of temper's kinds through `see`: `ask-temper.md`, section 5.
- A project's own policy through `see`: `ask-temper.md`, section 5.
