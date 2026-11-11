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
- **The worker is smith's host** (section 5; ../worker.md): it prepares the
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
  as its agent child (../worker.md, section 3). `temper-engine-smith`, the
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

smith's `deliver` is the worker's push (../worker.md, section 5): the
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

A goal's design delivery to its goal branch (`../forge/branches.md`, 3; ../worker.md,
section 5) has outcomes of its own, and is never stale:

| The design push | smith's delivery |
|---|---|
| landed, the goal branch at the head it started from | delivered, `pushed <head> to branch <branch>; your checkout stays detached, as expected` |
| landed, the goal branch's moved tip merged first | delivered, `pushed <head> to branch <branch>, merging its tip <tip> first; your checkout stays detached, as expected` |
| a path that isn't Markdown | refused, naming the path: `a delivery to the goal branch carries Markdown only; code goes through a change` |
| the moved tip in conflict | refused, naming the files in conflict, the merge left in progress to settle and deliver again |
| the branch moving at each of three tries | refused: `the goal branch kept moving; deliver again` |
| a `save` | refused: `the goal branch takes checked deliveries alone; deliver instead` |

What it delivers is the tree, not the run's history: a checkout is detached
(../worker.md, section 5), and the worker commits on its own record of the head,
never on HEAD (`../../protocol/git.md`, 4.7). So a run's own commits, resets and
rebases keep their files and lose their history, and a saved tree that is
delivered unchanged lands the save's commit, the delivery's title and body
unused (../worker.md, section 5). The head delivered names what is on the branch,
and the coding charter's words say so (`../../protocol/configuration/stock.md`'s
charter 2 row), since a run can't otherwise tell.

A `save` (`tools.md`) is the same push with no check run before it, and marked
unchecked: its commit's title is `Unchecked: work in progress, saved before a
question`, with no body, and a landed save's receipt reads `saved <head> to
branch <branch>, unchecked: no check ran; your checkout stays detached, as
expected`. Its other outcomes are the table's. A save's push moves the
change's own branch as a delivery's does, so the engine hears of it as of
any push the run left (../worker.md, 4.2). When it accepts the head the
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
the tree unchanged (../worker.md, section 5), so the head a finish delivers
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
answer carries what the run left on the forge (../worker.md, 4.2), and the
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
- **The worker's and the system worlds** (../worker.md, section 10): smith's
  scripted agent for the agent child, and smith's real agent domain over
  skein's fake LLM, its tools working in the trees the worker prepared.
- **Stories that are temper's:** a change produced, checked and pushed,
  and its push found moved; a coordinator that delegates a batch, hears
  results, and proposes what is beyond its authority; a chat that waits,
  hears its person's words, parks, and resumes from its transcript; a
  host tool answered busy by the engine, asked again and decided once.

## 8. Open questions

- **Ask temper before M3b** (`ask-temper.md`): M3b supplies
  the other views, the Showing part's paragraph on sketches and the actors'
  marks (`../../draft/ask-temper.execution.md`, section 3). The
  ChatGPT and Claude Code walk, a real model's use of `see ways`, of the person's
  addresses, of the `view` block and of `remember`, and the user's judgement
  of `ask.txt`, its Remembering part among it, follow landing.

- **A real chat choosing `change`** (`tools.md`): the owned-forge
  landing story requires the chat's own successful `change`, rather than
  a goal's change. Observe whether `goal`'s declaration steers a plain
  repository change there, and whether the branch-read guidance gives a
  real run its next read. Default: declarations and bounded refusal
  guidance, without a new admission restriction; the existing live story
  runs after landing (`../../testing/live-tier.md`). A scripted call establishes
  neither model choice nor recovery.

- **Forge-read fallout with real agents** (`agents.md`): the existing
  ChatGPT and Claude Code owned-forge landing stories check that a real
  run completes without losing `temper-box serve`. Refusal-rendering checks
  alone do not establish either outcome. Default: those live stories
  run after landing, with no change required by the refusal contract;
  `../forge/README.md`, section 11 keeps the routing proof's limits.

- **A direct change result's rendering room** (`charter.md`, 3): the landing procedure
  reserves 59 bytes for its result's label and sender before fitting its
  report. A task made directly by `delegate` with a Change contract has
  no such reserve, so its result at the contract's bound can be refused
  by a live requester's receiving bound. Default: keep the durable result
  whole and read it through `read_task`; the rendering allowance remains
  jig's `domain/hosts/README.md`, section 10's question.

- **Ask temper changing the catalogue** (`agents.md`, 1): observe a real model
  reading and proposing a model setting, including estimated prices, and
  keeping its suggestions within the offer, which its instructions do not
  name. Default: scripted stories of `see` and `propose_settings`; the
  real-provider observation is the user's, by trying it (`ask-temper.md`).

- **A real model asking its person** (`agents.md`, 2; the request `ui-goal-asks`):
  the client world's sheet-to-answer story supplies a typed `delegate`
  call, and the stock and host-answer tests hold the instructions. They
  do not establish a real model choosing that call from the sheet or
  Goals, with Away on or off or with always-ask words. Default: those
  instructions and focused stories; real-model observation follows landing.

- **A permission for its own requester alone** (`agents.md`, 2): a goal a person
  started may address anyone holding that person's role, since jig's
  executor permission is a role's, and a requester who has since changed
  role can no longer be addressed. A permission naming the requester would
  be jig's, a new executor kind. And `shapes::set_agreed_goal_from`, were it
  sent, would owe a question no person task answers. Default: the role, the
  charter's words, and that shape unsent.

- **Proposal-form diagnostics** (`tools.md`): `settings` reads name the kind
  and its missing member; `propose_settings` keeps the shared settings-file
  parser's errors. Default: those words, until a proposal needs the same
  kind-specific guidance. Both tools' schemas name the numbered kinds.

- **A real model revising a proposal** (`agents.md`, 1; the request `ui-proposal-2`
  of 2026-11-06): after landing, check that a request from "Change
  something" leads to a fresh proposal without a trip to Needs you, and
  that a request for a model no account runs leads to browser Settings,
  Accounts. Default: the stock instructions and scripted stories; the
  real-model observation follows landing.

- **A goal as the designer** (`tools.md`;
  `../../draft/goal-as-designer.md`, sections 3, 5, 6, 7 and 9): the delivery
  path is independent of the stock and reference instructions, their
  wakes and their location. Those words and settings are folded into the
  reference workflow together. Calls, briefs and reports take the room
  of `../../protocol/configuration/settings.md`, 7, with the reference's fallback.
  Default: the existing stock and reference words and wakes; test charters
  exercise goal delivery.

- **Constraining models in settings reads and proposals** (`agents.md`, 1): whole
  charter values can name models outside the current offer. Restricting what
  a read exposes while still round-tripping that value, or rejecting a new
  unoffered choice without rejecting an unchanged charter, needs a settings
  contract. Default: the inventory is filtered; whole charter reads and
  configured-model validation stay. Ask temper's instructions impose no
  restriction to the current offer (`../../draft/ask-temper.md`, section 8).

- **What a delivery keeps, under a deployment's own charter** (6): the stock's
  coding charter and the reference's tell a run its own history isn't kept. A
  deployment's own charter is its author's, so a run under one isn't told.
  Default: so.
- **A charter where the process has no local agent.** The engine does not
  know its remote worker's charter allowance, so it fits a charter
  to the smaller of `hub.charter_bytes` and smith's 65,536 (`tools.md`), and a
  remote worker that takes more, the reference's 127,480 among them,
  receives its brief cut at 65,536. The worker's hello on jig's link, which
  names its reply bound (`tools.md`), could carry the allowance beside it, or the
  engine's document could name it in `hosts.workers`, and the request
  `landing-behind-main` of 2026-11-08 asks it be sized as that bound is.
  Default: the smaller of the two defaults.
- **A `see` answer by its worker's reply bound** (`tools.md`). The protocol layer
  renders a `see` answer within the local agent's bound alone, since its
  hand-out (`HostDelivery::See`) names no call key, so a run on a remote
  worker whose bound is smaller may meet smith's `TooLarge`. Default: the
  local agent's bound.
- **The person's forge identity in a chat's brief,** rather than in the
  settings inventory (`agents.md`, 1), which Ask temper reads first.
  A brief section would spend every chat's room for what one charter
  uses. Default: the inventory.
- **A collaborator's login in `read_collaborators`** (`tools.md`). Default: ids,
  with `read_permission` naming each.
- **`read_permission`'s login held to the settings family** (`tools.md`). It
  reaches every run with the forge reads, as the forge's own answer about
  a user of a repository that run may already read; holding it would be a
  condition in `temper-engine-domain`'s render. Default: every such run.
- **A person known by a login alone** (`agents.md`, 1), where no link vouches for them:
  `temper-box serve` with several members, a managed Forgejo, a pasted App. Often
  no collaborator, adoption finds no person for them and the inventory's forge
  person has no forge user id; `read_permission` takes an id, so the agent
  cannot match them, and `propose_settings` refuses at the call, in
  acceptance's words, an approval only they could meet (`../forge/change/gates.md`, 3;
  `ask-temper.md`, 2), saying who to add on the forge before adopting again, in the browser the adoption
  form typed afresh (`../../client/forge/family.md`, 4). A family of links waits for
  `temper-box serve`'s members (`../../protocol/sign-in.md`, section 9).
  Default: adoption's seeds and the refusal above; a configured login alone
  supplies no verified forge identity.
- **Which host tools a charter offers.** jig's charter carries the host
  families its charter's `tools` names and its task's authority holds
  (jig's `domain/engine/executors.md`, 1), each stock charter's named in
  `../../protocol/configuration/stock.md`; temper's declares by them the
  `settings` and delegate families alone, an asking charter its own tools
  alone, and every other host tool in every other charter (`tools.md`), and the
  authority refuses what it does not allow. Whether to declare only what the
  charter names for the other families too, which the host part's budget (`tools.md`)
  may force, and how a brief teaches an LLM when to delegate a task rather
  than a sub-agent. `withdraw`, offered to a chat and refused by its
  authority, since policy 1 gives no decide family
  (`../../protocol/configuration/stock.md`), is one such tool.
- **What sizes temper's own way** (`agents.md`, 1): a read of the deployment's worker slots,
  an address of `see`'s, or an answer from the person.
  Default: the person's, as nothing reads
  them.
- **QA in temper's own way** (`agents.md`, 1): `../../draft/own-way.md`, 6, has it proposed once
  a tester is set; QA after landing isn't built (`../findings.md`, section 8).
  Default: the agent says so, and leaves it out.
- **Ask temper woken by a landing** of a document a charter
  names, where the landing touches structure, to say what it means for
  the settings ("the process document changed; here is what it means
  for your setup": `../../draft/configuring.md`, section 6). Default: a
  person starts it (`agents.md`, 1); a landed document reaches the next runs of the
  charters that name it (`../../protocol/configuration/settings.md`, 6).
- **Who may name a plan's members of `change`:** the control family
  says it (`tools.md`, 1), since a plan's maker amends, cancels and releases what
  it makes. A family of its own would let a policy grant control without
  plans, or plans without control.
- **A goal's question kept with its chat** (jig's `domain/tasks/messages.md`,
  5): the chat, woken by it, would answer from what it knows (its
  person's priorities, its notes), or ask its person a question of its
  own and answer the goal once it has the answer, in place of `agents.md`, 2's
  route through the person's inbox. Default: `agents.md`, 2's route.
- **Who else writes a question's line form** (`agents.md`, 2): the stock goal alone
  is taught it, by one sentence of charter 1, as the request of 2026-11-05
  for *Needs you* asks. A change's producers, its repairs and resolutions, and
  the reference's charters are not, so their questions read as words,
  answered by a reply. Default: as it is; a line in each charter that asks.
- **A change's foremen's LLM choice** (../forge/change/questions.md, 1): `llm` (`tools.md`, 1)
  names the producer's and the reviews' roles, and a change's foremen
  run at their charter's default, where the reference's draft gives
  each change one for its foremen too
  (`../../draft/reference-code-production.md`, sections 5 and 14). Default:
  the foreman's charter's, the reference setting its model there
  (`../../protocol/configuration/reference.md`); a `foreman` role once a plan
  needs foremen of different weights.
- **A larger whole compare** (`tools.md`). Nine files, nine commits or one line
  changed in a long file can exceed the stock's forge bounds. Raising the
  row bound alone cannot fit both whole sides of a long file; a paths-only
  comparison would change what the settled answer holds. Default: keep
  the bounds and direct the run to pull-file pages and selected file reads,
  until a real landing shows that recovery cannot serve the change.
- **A cancel's grace:** jig's `cancel` takes one, if the run gives it
  (jig's `domain/engine/executors.md`, 3); temper's takes none.
- **A code graph** for a workspace (../worker.md, section 12), as an MCP
  tool source once smith has one.
- **A change produced by a job of any kind.** `producer` names a
  charter, and `merge` the merge job, a catch-up's producer (`tools.md`, 1):
  every producer a reference change needs
  (`../../draft/reference-code-production.md`, section 3). Default: no
  other job kind produces a change; a project whose tooling makes
  changes with no model would name its kind as a member of `change`.
- **A document moved within one call** (`ask-temper.md`, 3): jig decides and records a
  write refused as moved, so the root cannot read the newest and write again
  under the same call, as `../../draft/ask-temper.md`, 5.3 has it. Default: the
  call is refused naming its section, and the run's next call applies it to
  the newest; a retry within the call once jig's notes take a section's splice
  at the write, or leave an application's moved write unrecorded.
- **Parts of a settings proposal across a restart** (`ask-temper.md`, 2): the engine holds
  them in its memory, so a restart loses them, and a run that was running
  across it is refused its next proposing call once, even with no part given.
  Default: as it is; a durable store for them if a real chat meets a restart
  mid-set often enough to matter.
- **How the settings' addresses read** (`ask-temper.md`, 1): `../../draft/ask-temper.md`,
  5.1's rules for how an answer reads hold the addresses read as the person; the
  settings, their history and proposals still read as the settings' renderers
  write them for ctl and the run's `settings` read, the history's "read on" line
  among it. Default: as they are, until a walk shows a run misled by one; then
  each takes the rules in its own read.
- **A revision's values of temper's kinds through `see`** (`ask-temper.md`, 1): `revision N`
  reads as the run's settings read does, which carries no value of temper's kinds
  at that revision, where ctl's `revision show` reads a set landing rule or forge
  charter from the forge at it, and the one it replaced. Default: as the run's
  read is, until a person asks for one; then the root reads the forge's values at
  the revision, as ctl does.
- **A project's own policy through `see`** (`ask-temper.md`, 1): the draft's addresses name
  no `settings project-policy`, which holds the forge's permission mapping
  an approval rule's role follows (`agents.md`, 1). Default: none, since
  `propose_settings` refuses at the call what too few eligible approvers would
  fail (`ask-temper.md`, 2) and the inventory says who is eligible; M2 adds it if a person's
  question needs the mapping.
