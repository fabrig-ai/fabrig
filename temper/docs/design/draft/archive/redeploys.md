# redeploys: a new build reaches a deployment without stopping its work

Draft, 2026-11-12, with 5.2 and 5.3 amended on 2026-11-15, and its versions and bridges (sections 1, 3, 4, 7, 8, 10, 11 and 12) revised on 2026-11-25 to carry `layouts-both-ways.md`, sections 3 to 7: frozen versions, layouts by fingerprint, and one conversion, in the engine, either way. How a deployment takes a new build of temper while its work goes on: the engine restarted alone while the workers keep their runs, a worker restarted with its agents resuming where they left off, and each part speaking the previous version of the wires between them. It comes from a session with the user on 2026-11-12, after redeploys of dev.fabrig.ai that each cost minutes of failed preparations and held goals, and from a read of main at `ee03a1159c`. The user's word that day: wire versions are no longer changed in place; the store and the wires upgrade from now on, heading to v0.1.0, and dev.fabrig.ai exercises both. The store's half is `../upgrades.md` and `../staged-upgrades.md`. This draft is the processes' half, and replaces `../upgrades.md`, section 9.

## 1. In one page

- **Versions frozen, layouts by fingerprint** (section 3). Every wire between the engine, its workers and their agents keeps its version; a change of layout is made in its current text, with no old text kept. smith's channel is per-build (4.4).
- **Each side states what it speaks, by layout** (3.2): at the opening, the worker states each wire's fingerprint and sends its text. A worker whose layouts give the engine no plan is refused there, named, rather than met by a task's first assignment.
- **A mismatch is never a task's failure** (3.4): what a worker cannot keep waits for one that can, spending no try.
- **The engine converts** (section 4), whichever side is newer: what it receives by Read plans to its own layouts, what it sends by Keep plans to the worker's. What the worker's layout cannot keep is never sent to it.
- **An agent is always its worker's build** (4.4): the worker pins its agent's program and configuration at its start, so the channel between them never spans two builds.
- **The engine alone, by default** (section 5). A check says when the workers may stay; the stop settles and tells the workers it restarts, they keep their runs through a restart wait of 20 minutes, and ride it.
- **A worker drains, then starts warm** (section 6): it takes no new work, stops each agent at the end of its turn within a bound, saves the work, and ends each attempt interrupted with no try spent. It keeps its stores and admits preparations a few at a time.
- **Workers in turn** (section 7): a redeploy restarts the engine alone, then the workers one at a time when asked or required, and every part together only across a change of the opening itself.
- **Clients are refused and reload** (section 8): a page or a client of another layout is told so, and reloads or upgrades. Converting for them is later.
- **dev.fabrig.ai exercises it** (7.5): every redeploy is a drill, with a referee.

## 2. What a redeploy costs today

From main at `ee03a1159c` and dev.fabrig.ai's redeploys of 2026-11-11 and 11-12, each of which swapped one binary into the engine and its three workers and restarted all four within seconds.

| Part restarted | What it costs |
|---|---|
| The engine, its workers kept | Little. The workers keep their runs through their grace (60 s as shipped), the restarted engine adopts each attempt at its worker's hello, and turns not yet acknowledged are sent again (jig's `domain/engine/restart.md`, step 3). Due work is released all at once at the opening. |
| The engine, of another build | Nothing checks that the workers can read it: wire versions did not move before v0.1.0, and nothing compares layouts. On 2026-11-11, `67507fd70c` appended a field to smith's charter with its version unmoved. Workers of the build before refused every new charter (`Invalid(Start(Other)): charter`), a permanent failure, and goals were held after four tries. |
| A worker | Its runs are cancelled at once, losing the turn in flight with the LLM and any check in flight. Work is saved and the attempt ends interrupted with no try spent, but a job's attempt ends lost, spending one. The stop bound the worker declares is not enforced over its saves. Then a cold start: its stores and workspaces are set aside, and every preparation admitted at once clones the whole repository, up to 36 against one forge, timing out and spending tries. |
| An agent | Spawned from the path and configuration the worker was given, read again at each spawn, so an old worker that starts an agent after the swap starts a new one. |
| A client | A page or a client of another build is refused (`version not served`), or meets a record it cannot decode. |

## 3. Versions frozen, layouts by fingerprint

### 3.1 No version moves

- **Every wire's version is frozen** at its header's, as the store's families' are (`layouts-both-ways.md`, 3; `../../protocol/wires.md`, 3.1). A change of layout is made in the wire's current text alone, with no old text kept and no patch; what it costs a deployment is the engine's plans' (4.1).
- **The wires,** each a schema of its own:
  - jig's worker channel, its kinds' table included: each kind's number, its direction and the record it carries;
  - temper's payloads in it, a git identity's value among them (`../../protocol/channel.md`, section 6);
  - smith's charter with its result, and smith's turns, which the channel carries as bytes;
  - the pieces of smith's channel the engine keeps or sees: a call's name, and a delivery's evidence, which the engine keeps and hands to the run's next worker (`../../protocol/channel.md`, section 2);
  - smith's channel between a worker and its agents, per-build (4.4).

  Versioned otherwise: the host tools' contract, each tool's name, its input and what it means, versioned whole by its author's hand (4.2; `../../protocol/wires.md`, 7); and each job kind's request and result, files the worker carries unread, which are no wire between temper's builds (`../../protocol/wires.md`, section 8). Clients' wires are section 8's.
- **Nothing old in the repository:** no kept text, no patch, no golden file of another version and no list of layouts (`layouts-both-ways.md`, 5 and 9). `smith-transcript`'s version 7 text stays as the store's seed alone (`../../protocol/wires.md`, 3.1).
- **Frozen, not judged by a gate:** each worker wire's crate holds its version equal to its frozen number (`../../protocol/wires.md`, 3.1), and `fabrig-boundary`'s `client_layouts` holds the clients' (jig's `protocol/client/versions.md`, 1). A layout changed in place is the ordinary change, met by the engine's plans.
- **The churn it meets** (main, 2026-10-15 to 11-12): 29 in-place changes to the schemas of jig's channel (14), temper's payloads (4), smith's charter (4), its turns (2) and its channel (5); 14 of them a field, an option or a variant appended, each of which a plan derives.

### 3.2 Layouts and texts at the opening

- **A worker states each wire's layout,** its family fingerprint (skein's `data/codec/fingerprints.md`, section 1), and sends each wire's text in its credential, once per connection (jig's `protocol/channel/wires.md`, 2). jig's channel agrees its framing version as skein's opening does (skein's `net/channel.md`, 5.2), one frozen version on each side.
- **A layout equal to the engine's** converts nothing, and its text is not parsed: every part of one build meets this case (jig's `protocol/channel/layouts.md`, 2.3).
- **A layout that differs** has a plan derived for each value type the wire carries, each way, kept by the worker's text, so a fleet on one build derives once (jig's `protocol/channel/layouts.md`, 3).
- **Refused by name:** a text that does not parse, a value type with no plan and no written converter, or a `smith-pieces` layout other than the engine's refuses the opening, naming the wire, and the worker redials as after any refusal (jig's `protocol/channel/wires.md`, 3). The worker is drained and restarted on the new build (6.1).
- **Terms are the worker's layout's.** The engine accepts with the worker's kinds' table, so frames cross at the worker's layout and their bounds are that table's (skein's `net/channel.md`, 5.2; jig's `protocol/channel/wires.md`, 4 and 5).
- **`versions` prints each wire's layout,** its fingerprint, in place of a range and a floor (`../../protocol/wires.md`, section 10).

### 3.3 The floor

- **No wire keeps a floor.** A build speaks its own layouts alone, and the engine converts for any worker whose layouts its plans derive from, older or newer.
- **What no plan derives is kept to the floor of what dev.fabrig.ai runs** (`layouts-both-ways.md`, 7.4): a tombstone for a dropped field, a written converter for a retype (jig's `protocol/channel/layouts.md`, 6). Each goes once dev.fabrig.ai and every part it runs are at or past the layout that made it needed; a worker still at that layout then has no plan, and is refused at its opening by name.
- **The host tools' contract keeps its floor** (`../../protocol/wires.md`, 7): the engine answers every version from it, so an adopted run's calls are answered as before.
- **Workers keep up with it:** dev.fabrig.ai's workers are restarted in turn (7.2) before a landing removes what their layouts need, which the check (5.1) says.

### 3.4 A mismatch costs no task

- **What a worker's layout cannot keep is refused before it is sent** (4.3), as a typed answer to the domain that sent it: an assignment or a job whose charter, items or transcript the worker can't keep ends its claim refused, *untranslated*, spending no try (jig's `domain/tasks/lifecycle.md`, 2), and the fleet keeps that task off that channel while it stands (jig's `protocol/channel/layouts.md`, 5.1).
- **It is told as a mismatch,** naming the wire, the declaration and the part, in the engine's log and the task's retry notice, never as a bare `charter`.
- **A worker's own refusal for a version** (jig's `protocol/channel/wires.md`, 6) is a conversion's bug past the opening, and spends no try either.
- **What the engine cannot read,** its Read plan refusing it, is a body that does not decode (jig's `protocol/channel/layouts.md`, 5.3). A result fails its run as invalid. A turn is kept as its bytes and read later; one that cannot be read when its window is resumed closes the window (jig's `domain/engine/executors-transcripts.md`, 1).

## 4. Conversion

### 4.1 The engine converts

- **In the engine's workers component,** whichever side is newer (jig's `protocol/channel/layouts.md`): what it receives runs through a Read plan from the worker's layout to its own, then its own decoder; what it sends is encoded at its own layout, then runs through a Keep plan to the worker's (skein's `data/codec/layouts.md`, 2). The worker, and the agent its build pins (4.4), speak their own layouts alone and know nothing of other builds.
- **Why the engine:** it is the one side that holds every connection, and a redeploy restarts it first (7.1). The plans need no age, so a worker newer than the engine converts the same way, and a redeploy's order costs refusals, never correctness (`layouts-both-ways.md`, 6.1).
- **Read skips, Keep refuses:** what the engine receives is used as its own, what its layout lacks skipped; what it sends must mean the same at the worker, so what the worker's layout lacks is dropped only at its default or where the engine's text marks it `droppable`, and anything else refuses the value (4.3).
- **Nothing goes up only to come down again.** Bytes kept at the layout they were written in (a stored turn, a call's name, a delivery's evidence) are converted from that layout, never through the engine's: a stored turn by the layout the store's registry names for it (jig's `protocol/store/transcripts.md`, 6), a call's name and a delivery's evidence not at all (4.2).
- **What the worker must get back is not dropped.** A field a worker sends and expects to meet again (an identity it echoes, a value it resumes from) keeps a tombstone once dropped, so the engine's Keep plan writes it back (3.3).
- **Written once, in the schema:** the common changes, a field added with a default, a field dropped with its tombstone, a variant added, a bound widened, cost their author nothing beyond the schema's line; a retype or a reshape takes a written converter (3.3).

### 4.2 What crosses

| Wire | Sent by | Engine to worker, Keep | Worker to engine, Read |
|---|---|---|---|
| jig's channel records | both | each frame as its kind's enumeration, `FromEngine` | each frame, `FromWorker` |
| temper's payloads | both | items, job specs, answers to asks | what a run left and saved, preparations, asks |
| a git identity's value | the engine | its `Grant` converted at the assignment, the job and each renewal, keeping its number, generation and validity | — |
| smith's charter | the engine | converted to the worker's layout | — |
| smith's result | the agent | — | an ended run's outcome |
| smith's turns | the agent; the engine, resuming | each turn of a window resumed, and each part's body | each turn and part as it arrives |
| a call's name, a delivery's evidence (`smith-pieces`) | the agent; the engine, resuming | never converted: kept as bytes that name no layout, they cross as written, and the opening requires the worker's layout to be the engine's | the same |
| a host tool's input | the agent | — | by the tools' contract of the run's charter (below) |
| a job's request and result files | the engine; the worker | carried unread: no wire between builds (3.1) | the same |
| smith's channel (`smith-channel`), per-build (4.4) | the worker; its spawned agent | nothing: the worker pins its agent's program and configuration at startup | nothing: a restarted worker spawns fresh agents, adopting none |

- **Each value type of each wire** is planned in each direction it crosses, and no other (jig's `protocol/channel/layouts.md`, 2.1, which lists them).
- **Stored turns are never rewritten.** The engine keeps a turn's bytes at the layout its agent wrote, which the store's registry names, and reads them by that layout's plan (jig's `protocol/store/transcripts.md`, 6). A window resumed on a worker is converted turn by turn to the worker's layout; one turn that can't be kept refuses the assignment, which is placed elsewhere (4.3).
- **A git identity's value is a secret inside a grant,** opaque to jig's records, and its `Grant` is temper's value type, converted by its Keep plan at each send (`../../protocol/wires.md`, 6). The engine keeps its two newest generations at its own layout alone.
- **The host tools' contract is versioned whole.** A run's charter names the version its tools were written in, and a run adopted across an engine restart keeps calling them. The engine answers every version from its floor: a tool removed, renamed or changed in its input or meaning keeps its old contract until the floor passes it, so an adopted run's calls are answered as before. Dropping an old contract sooner is a breaking change of that contract (`../../protocol/wires.md`, 7).
- **A job's retained answer,** kept by a worker across the engine's restart, is read by its Read plan like any message.

### 4.3 What the worker cannot keep

- **Only at an assignment may something wait.** An assignment or a job whose charter, items or window the worker's layout cannot keep is refused for that worker before anything is sent, *untranslated*, and the fleet keeps the task off that channel; another worker takes it, or it waits, spending no try and saying why; during a rolling redeploy (7.2), the next worker back takes it (jig's `protocol/channel/layouts.md`, 5.1). Whether it goes down depends on its values, not on the layouts alone: an option appended at none goes down, so trying it is the judge.
- **A running attempt cannot move.** A message to a run already on a worker (an inbox message, a call's answer, a grant's renewal, a cancel) that the worker's layout cannot keep is discarded and logged, and the channel takes no new placement while what it hosts goes on (jig's `protocol/channel/layouts.md`, 5.2). No gate holds these kinds total across layouts: a change their messages cannot carry to the previous build's workers is met by draining those workers (6.1).
- **A newer worker before an older engine** converts the same way: what it sends is read by the engine's Read plans, what the engine's layout lacks skipped, and a worker whose layouts give no plan is refused at the opening, by name (3.2).

### 4.4 An agent is its worker's build

- **Pinned at the worker's start.** The worker takes its agent's program and the agent's configuration, as its own configuration names them (`../../domain/worker/agent-processes.md`, section 1; `../../protocol/binaries.md`, section 5), into its own data directory as it starts, and spawns that copy for every run until it exits. A redeploy replacing the program or the configuration changes what the worker's next start pins, never a running worker's agents.
- **So smith's channel never spans two builds.** Its version is frozen as every wire's is, and nothing converts it (`../../protocol/wires.md`, section 2). The pieces of it the engine keeps are wires of their own (3.1).
- **The boundary is local pipes,** from the worker's `smith-host-protocol` to its child agent's `smith-protocol-channel`, one channel per run (`../../domain/worker/agent-processes.md`, section 1; smith's `protocol/channel.md`, section 2). The engine is neither endpoint nor a relay for these frames. An engine-only redeploy leaves the worker and its pinned agents together on their existing build, including agents spawned after the program at its configured path is replaced. A worker restart starts new channels with its newly pinned agent; agents from its previous start are never adopted. So a change of smith's channel, adding drain among them, costs nothing across builds.
- **Pinning is a prerequisite for keeping a worker.** A worker from before pinning must be stopped before its configured agent program is replaced, then restarted on the new build. Keeping that worker across a swap could make it spawn an agent from another build; the per-build exemption does not cover that rollout. Section 7.3's restart of every part establishes the pinned workers before engine-only redeploys rely on them.
- **An agent goes with its worker.** A worker that stops or dies takes its agents' process groups with it, and a worker's start finds nothing of its previous life still writing in its workspaces (smith's `domain/host.md`, section 4, whose status records today's gap).
- **Agents outliving their worker,** adopted by its next start, are later (the user, 2026-11-12; section 12).

### 4.5 Changes no plan carries

- **No breaking declaration for a wire's schema.** A change no plan derives and no written converter carries refuses each worker whose layout needs it, by name, at its opening (3.2): those workers are drained and restarted on the new build (6.1), while the rest stay.
- **A change of the opening itself** (the credential, laid out alike in every build) is no layout any plan reads: its landing restarts every part together (7.3), as the landing that froze the versions did (`../../protocol/wires.md`, 3.2).
- **A change of meaning a worker cannot be told,** a field kept with another sense, is a new field beside the old, kept until the floor passes the old one (3.3).
- **The host tools' contract keeps its breaking declaration,** which raises that contract's floor (`../../protocol/wires.md`, 7).

## 5. The engine alone

### 5.1 When the workers may stay

- **The check** (`upgrade --check`, `../../protocol/binaries.md`), run by the new binary before the swap. It is a rehearsal of the store's registry changes and jobs over a copy, which then restores the copy as a start does, through the domain's fit and link checks, and names the first record a start would refuse, in a start's words (`../store-starts.md`, section 2); its verdict on the workers compares the wire numbers the store keeps (`../upgrades.md`, section 9, before this draft), and says unknown when it keeps none. Judging each worker by its layouts, texts and terms is this draft's, and new. A worker may stay when the new build's engine would accept its opening: for every wire, its layout is the new build's or its text gives a plan for each value type, its `smith-pieces` layout is the new build's, and its terms fit (3.2), judged by the function the opening judges by (`../../protocol/wires.md`, 10.2). It names each worker that may not, and why.
- **What it reads:** beside the wires the store keeps for the engine (jig's `protocol/store/records.md`, 3), each worker's opening as the engine last authenticated it: its name, its layouts and texts, its terms and when. So the check runs with the engine stopped.
- **A worker with no record,** or none since the engine's last start, is reported unknown and treated as one that may not stay.
- **A worker that may not stay** makes the redeploy one of every part together (7.3).

### 5.2 The stop settles and says so

- **Told to stop, the engine settles first** (`../upgrades.md`, section 8; jig's `domain/engine/restart.md`, 5): its effects in flight on the forge settle, and no run is placed meanwhile.
- **Then it says it restarts.** Once settled, the engine keeps in its store that it restarts and how long it asks its workers to wait, its restart wait; then, before it closes their links, it tells each worker so. It says it only on a link whose worker's layout of the workers' wire has the word's kind (jig's `protocol/channel/layouts.md`, 4.1): to a worker without it the word is not sent, which is no mismatch (4.3), and that worker keeps its grace. Without the word, a clean close, a reset and a silence look alike to a worker, and each meets the same grace.
- **The workers keep their runs.** Their links stay up while the engine settles, and their wait starts when the link drops.

### 5.3 The window fits the wait

- **The window** is the settled stop, the swap, the store's registry changes and jobs (jig's `protocol/store/opening.md`, section 2) and the start up to the first hello. The rehearsal measures it (`../staged-upgrades.md`, section 5). On 2026-11-15, dev.fabrig.ai's engine restarts took 2 and 6 minutes, a check over the stopped backup and a start replaying the store's log, and its workers, whose configuration kept a 10-second grace from before 5 minutes was the default, cut three runs.
- **A restart said is waited for long.** A worker that heard the word keeps its runs through the restart wait, by default 20 minutes (section 12), counted on its own clock from the link's loss, rather than its grace, whose end cancels nothing meanwhile. Past the wait it cancels and saves as at its grace's end. The user's word of 2026-11-15: runs wait a good while for the engine, rather than being cancelled.
- **A link lost without a word keeps the grace,** 5 minutes by default. It may be a partition, with the engine still up and holding the worker's runs for its own grace, which outlasts every worker's stop bound (6.1; jig's `domain/engine/fleet.md`). A longer grace would keep a worker gone for good from its runs' retry as long. An engine that crashes is ridden when it is back within the grace.
- **The engine holds the runs as long.** Every start that follows a stop which said it, until the word is discharged, holds each run it restores on a worker for the longest restart wait said and its grace after it, counted on its own clock from that start, rather than its grace alone. The hold stays with the run through its worker's hello and any loss after it, since a hello received doesn't show that the worker heard the engine back, and a later word never shortens a hold outstanding. The word is discharged once each run it covered has ended, been placed elsewhere after its hold, or had its worker hear the engine again. So no next attempt is placed while a worker may still host the last, and a worker gone for good across a restart has its runs moved elsewhere that much later. A build that doesn't know the word holds them for its grace alone, so going back past the build that brings it is every part together, each worker drained first (7.3).
- **A worker's bounds keep their meaning.** The stop bound a worker states at its hello still bounds a link lost without the word, and a stop; the hello's check of it against the engine's grace is unchanged. A word heard has a bound of its own, the restart wait and the same tail after it, which the engine's hold covers. A worker told to stop while it waits drains as ever (6.1), within its stop bound.
- **A waiting run goes on** as far as it can without its engine: its agent's turns with the LLM, its tools and checks, its saves and pushes, its jobs within their own deadlines. Its turns, its answers and its calls relayed to the engine are kept and sent at the relink (jig's `domain/hosts/hub.md`, 6). The rest is degraded as today: its reads from the engine fail at the loss, so a check judges without its inherited reds, as when their lookup is late, and its reports to the engine, a check's results among them, and its live pieces are dropped.
- **What needs the engine waits within the run's own deadlines.** A relayed call waits within its caller's and its run's deadlines; past them it fails to its agent as any slow call does, and the run goes on. A relayed attempt that its own timeout withdraws while the link is down is tried again with no uncertain effect. A completion that can't be issued for want of a valid grant waits for one, keeping its retries, as a wait its agent's supervision knows of; one already issued runs on under its provider's deadlines, and an account that must be signed in again fails as it does now.
- **The run's budgets keep counting.** The wait ends cancellation for contact, not the run's own bounds: a run whose time, turns or spend run out meanwhile ends as any run whose budget runs out.
- **A window past the wait** ends the runs interrupted, with no try spent (jig's `domain/hosts/hub.md`, 6): a slow restart costs turns, never tasks.
- **Who will not ride it is named.** At its settled stop, the engine names in its own lines each worker told it restarts and each not told, whose layout lacks the word and so keeps its grace alone (`../../protocol/wires.md`, section 4). The first redeploy onto the build that brings the word is stopped by an engine that cannot say it, so all its workers keep their grace. The referee (7.5) judges each worker by the wait it was given and its own return.

### 5.4 The opening

- **Adopted runs go on.** The due work released together at the opening is admitted by the workers a few at a time (6.4), with no ramp of its own.

## 6. A worker restarted

### 6.1 The drain

- **Told to stop, a worker drains** (jig's `domain/hosts/hub.md`, 7). It says so on its link, takes no new run or job, and asks each agent to stop at its answer: the LLM's answer in flight arrives and is committed as it comes, each call's answer is committed as it ends, and the calls still running after a short grace, the drain's grace, by default 10 seconds, are cut, the turn left open for the next attempt (`worker-restarts.md`, 3; smith's `domain/turn-parts.md`, section 4). A worker whose link does not carry parts (jig's `protocol/channel/records.md`, 5) asks its agents to stop at the end of their turns, as before.
- **Within the drain bound,** by default 3 minutes (section 12). A run still waiting for its answer at the bound is cancelled as before: that answer is lost, and the next attempt plans its turn again.
- **Then:** each agent gone; relays and deliveries settled; the work saved to its saved-work branch, and the workspace kept on the worker for the run's next attempt (`../../domain/worker/checkouts.md`, *A workspace kept for its run*); the answer `Cancelled(Shutdown)`, the attempt suspended with no try spent and its window left open (`worker-restarts.md`, 5); every turn, part and answer acknowledged before the worker exits.
- **A job the drain cancels** before its effect began ends suspended too, where today it ends lost and spends a try. One whose effect began is left for the next attempt to recover, as after any loss.
- **The stop bound is enforced, not declared.** The bound a worker states at its hello (jig's `domain/hosts/hub.md`, 6) is a deadline the worker keeps over the drain, every cancellation, delivery and save, and its agents' teardown. A save still running at it is abandoned and its agent's group killed, so nothing of the attempt writes after it, and the engine's grace is sized from it.
- **The service manager's stop timeout** sits above the stop bound. A kill before it is a crash, which takes the drain's path without its settle (`worker-restarts.md`, 7): the worker back as a new process keeps the workspace its run held, the engine ends the attempt suspended at its hello (jig's `domain/engine/executors.md`, 6.5), and the next attempt on that worker saves the workspace before its agent starts and resumes from its committed answers and calls (`../../domain/worker/checkouts.md`, *A workspace kept for its run*). What it loses is what was never committed: the answer in flight, and a cut command's output, which the model reads as interrupted after an unknown time (smith's `domain/turn-parts.md`, section 3).

### 6.2 What a run loses

- **At most the answer in flight when the drain's bound came, and the cut calls' progress.** Its next attempt resumes the cut turn from its committed answer and its calls' committed answers, without asking the LLM again: it runs a cut check again, skipping the tests that passed, answers a host call from what the engine decided, and hands any other cut call to the model as interrupted, with how long it ran and its output so far (smith's `domain/turn-parts.md`, section 5). On the same worker, while it is back within the restart wait, it takes up its kept workspace, ignored files and build outputs included; on any other worker, it starts from its saved work, which leaves those out (jig's `domain/engine/executors-transcripts.md`, 1, and `jig/docs/design/domain/engine/executors.md`, 6.4).

### 6.3 A warm start

- **Stores kept,** each made once by one creator while the other preparations wait (goal 1492, in flight), and cloned without blobs, by the user's word of 2026-11-12.
- **Workspaces not kept, but a cut run's for that run alone.** The workspace-reuse mode stays internal, since an agent can write a workspace's `.git` and so reach later workstreams. With its store kept, a workspace is a local clone and a checkout. A run a drain or stop cut keeps its workspace on its worker for its next attempt alone, never another workstream's, so that concern does not reach it (`../../domain/worker/checkouts.md`, *A workspace kept for its run*).

### 6.4 Preparations a few at a time

- **One active preparation per worker by default,** shared by runs and jobs. The others queue before the checkout in order of arrival, a preparation's deadlines count from its admission, and its permit is released only once a cancelled preparation has settled. Run slots stay as they are (`../../domain/worker/checkouts.md`, amended by its owner).
- **Three workers then make three preparations at once against a forge,** after a restart or a burst of due work alike.

## 7. The redeploy

### 7.1 The engine alone, by default

1. **A build that runs a store job** is first rehearsed on a backup of the deployment's data and drilled on staging.fabrig.ai, and a backup is taken with the engine stopped (`../staged-upgrades.md`, sections 5 to 7).
2. The new build's check (5.1) says the workers may stay.
3. The settled stop, the swap and the start, which the workers ride (5.2, 5.3).
4. A redeploy whose change the workers run goes on with 7.2.

### 7.2 Workers in turn

- **When:** on request, for a change the workers run; or when the check says a worker's layouts give the new engine no plan (3.2).
- **One at a time:** drain (6.1), swap, start, and wait for its hello and its first logins before the next. The other workers serve throughout: two of dev.fabrig.ai's three.
- **The engine first,** since it converts for every worker; a worker that goes first is converted the same way (4.3), and its opening says whether its layouts plan.

### 7.3 Every part together

- **Only across a change of the opening itself (4.5), or with a worker that may not stay:** every worker drained, then the engine's settled stop, every binary swapped, and all started. Runs end interrupted with their work saved; none is lost.

### 7.4 The configuration

- **As `../upgrades.md`, section 10, from now on:** each build reads its own members and the previous build's. A member added is optional, with a default; one removed or renamed is accepted with a warning naming its replacement until a landing drops it. An agent's configuration is pinned with its program (4.4).

### 7.5 Exercised on dev.fabrig.ai

- **Every redeploy is a drill** (the user, 2026-11-12). Its script runs the check, does what the check says, and its referee reports afterwards: each worker's layouts, the engine's or converted, by wire; no try spent and no attempt lost because of the redeploy; no goal held for its infrastructure since; each worker's return against the wait it was given, the restart wait or its grace; and no run ended for contact while the engine went alone, though such an end spends no try.
- **staging.fabrig.ai goes first** with a build that runs a store job (7.1) or changes the opening (4.5).
- **The drills run on the deployments after landing.** Plans prove with the focused and fuzzy suites alone (section 10).

## 8. Clients

- **A client of another layout is refused and told** (jig's `protocol/client/versions.md`, 1): its families' versions are frozen as every wire's are (3.1), and the stream's hello names the engine's layouts, which the client judges, so a client of another layout is refused by name rather than meeting a record it cannot decode. A page reloads to the engine's bundle, and `temper-client` and the TUI say to upgrade. Requests kept at another layout are dropped with a notice (`../upgrades.md`, 7.4).
- **No kept text and no conversion** for clients: an engine serves its one layout of each (`../../protocol/wires.md`, section 9).
- **Not converted, for now** (the user, 2026-11-12): 198 in-place changes to the clients' wires in the four weeks to 11-12, the most churn of any wire. The engine converting for them as it does for workers is later (section 12).

## 9. Infrastructure failures

- **A preparation failing for its infrastructure** (a git operation timed out, the forge unavailable) spends the same tries as a broken run, so a burst of them holds goals. Sections 6.3 and 6.4 end such bursts.
- **What remains is a policy of jig's own:** such failures pause with a backoff up to minutes and hold only past a bound of their own, naming the infrastructure, while a run's own failures keep their tries (jig's `domain/tasks/lifecycle.md`, 2). A preparation that always fails still holds, past its bound.

## 10. Proving it

Plans prove it with the focused and fuzzy suites alone; the drills come after landing (7.5).

- **Each plan:** skein's round trip, a value converted by the Keep plan and back by the Read plan coming back as itself, over `every`'s texts and the fuzzy suite's pairs; each refusal and each "no plan" a case of its own (skein's `data/codec/layouts.md`, 6).
- **The freeze:** a client family's version moved is refused by `client_layouts`, and each worker wire's crate holds its frozen number (3.1).
- **The opening:** a worker at the engine's layouts, converting nothing; one at another layout, its plans derived; one whose layout gives no plan, refused and named; one whose `smith-pieces` layout differs, refused and named (jig's `protocol/channel/wires.md`, 7).
- **A story across layouts** in the one-process world (`../../testing/tiers.md`, 1.5), with a worker made to speak other layouts of its wires: a goal through a landing, a close job, a chat resumed, a run adopted across an engine restart and sent a message, a call's answer and a renewed git identity after it, and a host tool of the previous contract called by that run.
- **The engine alone:** an engine restarted mid-run with its workers kept, the run going on with no new attempt; a window past the grace ending it interrupted.
- **A restart said:** an engine that announced its restart, back after longer than the grace and within the restart wait, the run's turns and checks going on meanwhile, its relayed call and its completion's grant waiting, and no new attempt; the same window after a link lost without a word, the run ended interrupted after the grace; a worker gone for good across an announced restart, its run placed elsewhere only once the hold has passed; a hello received whose answer never reaches the worker, then the link lost, the hold kept; two restarts in a row, a crash during the start between them, and a shorter restart wait on the second, no hold shortened; a call near its deadline, and an attempt withdrawn by its own timeout while the link is down; a run near its time budget, ending as its budget says; a worker at the version before, sent no word, keeping its grace and its link.
- **A worker drained:** mid-turn, the turn ends, the work is saved and the next attempt resumes with no try spent; a job drained before its effect ends interrupted; a save past the stop bound abandoned, with nothing written after it.
- **A mismatch refused:** an assignment its worker cannot read waits for another worker, spending no try, its notice naming the wire.
- **An agent is its worker's build:** with the configured program and configuration replaced under a running worker, its next agent is still the pinned one.

## 11. Who carries what

Each part is carried by a goal, across the trees.

| Part | Where |
|---|---|
| Frozen versions, plans between any two layouts in Read and Keep modes, tombstones (3.1, 3.3, 4.1) | skein's codec; each wire's crate |
| Layouts, texts and terms at the opening; conversion in the engine's workers component; a value the worker can't keep as a refusal; placement around it (3.2 and 3.4 to 4.3) | jig's channel and fleet; temper's payloads |
| The charter's, result's and turns' value types; calls' names and deliveries' evidence; stopping at a turn's end; agents' process groups (4.2, 4.4, 6.1) | smith |
| The host tools' contracts; jobs' requests and results; git identities for each channel (4.2) | temper's engine and worker |
| Agents pinned at the worker's start; the drain and the enforced stop bound; jobs on a drain; preparations a few at a time (4.4, section 6) | temper's worker; jig's hub |
| A settled stop that places nothing; the graces (5.2, 5.3) | jig's engine; temper's configuration |
| The restart said on each link whose version carries it, and its wait: kept by the worker, held by the engine over the runs it restores until discharged; a waiting run's relayed calls and grants; the redeploy naming who keeps its grace alone, and its referee judging each worker by its wait (5.2, 5.3, 7.5) | jig's engine, fleet, link and hub; smith's run and session; temper's configuration; the redeploy script and its referee |
| `versions` and `upgrade --check` with fingerprints, texts, terms and workers' openings (3.2, 5.1) | temper's binaries |
| The policy for infrastructure failures (section 9) | jig's tasks |
| The redeploy script, its referee and the drills (section 7) | the deployments' operator |

## 12. Later and open questions

- **Agents outliving their worker** (the user, 2026-11-12: later): a worker's restart adopting its agents still running, which needs their channel over something that outlives the worker, and the worker's state on disk.
- **Clients converted:** pages and clients of another layout served, the engine converting for them as it does for workers, once their wires settle (`layouts-both-ways.md`, 10).
- **The drain's bound.** Default: 3 minutes.
- **The workers' grace.** Default: 5 minutes, with the engine's grace above every worker's enforced stop bound.
- **The restart wait.** Default: 20 minutes, with room over every window measured so far, 6 minutes on 2026-11-15 and 11½ on 2026-11-13's store patch, not a current upgrade path; a worker gone for good across a restart holds its runs that long and the grace after it. It is the engine's to set and say, so one setting serves every worker.
- **A run's reports to the engine** kept and sent at the relink, as its turns are, rather than dropped while it waits. Later.
- **How long a tombstone or a written converter stays** (3.3). Default: until dev.fabrig.ai and every part it runs are past the layout that needed it (`layouts-both-ways.md`, 7.4).
- **The policy for infrastructure failures** (section 9). Default: a bound of its own, in jig.
