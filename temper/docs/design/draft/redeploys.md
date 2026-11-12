# redeploys: a new build reaches a deployment without stopping its work

Draft, 2026-11-12. How a deployment takes a new build of temper while its work goes on: the engine restarted alone while the workers keep their runs, a worker restarted with its agents resuming where they left off, and each part speaking the previous version of the wires between them. It comes from a session with the user on 2026-11-12, after redeploys of dev.fabrig.ai that each cost minutes of failed preparations and held goals, and from a read of main at `ee03a1159c`. The user's word that day: versions are no longer changed in place; the store and the wires upgrade from now on, heading to v0.1.0, and dev.fabrig.ai exercises both. The store's half is `upgrades.md` and `staged-upgrades.md`. This draft is the processes' half, and replaces `upgrades.md`, section 9.

## 1. In one page

- **Versions move with their layouts, from now** (section 3). Every wire between the engine, its workers and their agents takes a new version at the landing that changes its layout and keeps the old version's text. A wire that spans builds brings its bridge, as a stored family brings its patch (`staged-upgrades.md`, section 3); smith's channel is declared per-build (4.4). The gate refuses a layout changed in place.
- **Each side states what it speaks, by layout** (3.2): at the opening, each wire's range of versions and each version's fingerprint. An in-place change the gate missed is refused there, named, rather than met by a task's first assignment.
- **A version mismatch is never a task's failure** (3.4): what a worker cannot read waits for one that can, spending no try.
- **The newer side bridges** (section 4). It speaks every version of a wire from its floor up, translating up by the forward patch and down by its reverse. What the older side cannot hold is never sent to it. A change no bridge can carry is declared breaking, and raises the floor.
- **An agent is always its worker's build** (4.4): the worker pins its agent's program and configuration at its start, so the channel between them never spans two builds.
- **The engine alone, by default** (section 5). A check says when the workers may stay; the stop settles, the restart fits the workers' grace, and they ride it with their runs.
- **A worker drains, then starts warm** (section 6): it takes no new work, stops each agent at the end of its turn within a bound, saves the work, and ends each attempt interrupted with no try spent. It keeps its stores and admits preparations a few at a time.
- **Workers in turn** (section 7): a redeploy restarts the engine alone, then the workers one at a time when asked or required, and every part together only across a breaking change.
- **Clients are refused and reload** (section 8): a page or a client of another version is told so, and reloads or upgrades. Bridging them is later.
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

## 3. Versions that move

### 3.1 With every layout change

- **The rule reversed** (the user, 2026-11-12). Until now, a change a reader would notice was made in place, its version unmoved, until v0.1.0 (jig's `protocol/README.md`, section 7; skein's `data/codec/README.md`, section 5). From now, the wires this draft names take a new version at the landing that changes their layout, as stored families do (`staged-upgrades.md`, section 3).
- **The wires,** each a schema with its own version:
  - jig's worker channel, its kinds' table included: each kind's number, its direction and the record it carries;
  - temper's payloads in it, a git identity's value among them (`../protocol/channel.md`, section 6);
  - smith's charter with its result, and smith's turns, which the channel carries as bytes;
  - the pieces of smith's channel the engine keeps or sees: a call's name, and a delivery's evidence, which the engine keeps and hands to the run's next worker (`../protocol/channel.md`, section 2);
  - the host tools' contract: each tool's name, its input and what it means (4.2);
  - each job kind's request and result (`../domain/worker/jobs.md`);
  - smith's channel between a worker and its agents, which has no bridge (4.4).

  Clients' wires are section 8's.
- **The old version's text is kept,** with its golden bytes, as a stored family's is (skein's `data/codec/command.md`, section 7), and a wire that spans builds brings its bridge (section 4). The bridge inventory (4.2) also names per-build wires, whose exemption is from translation, never from version moves or kept text.
- **The gate refuses** a wire whose layout differs from its latest kept text with its version unmoved, as it refuses a stored family's (`upgrades.md`, section 12).
- **The churn it meets** (main, 2026-10-15 to 11-12): 29 in-place changes to the schemas of jig's channel (14), temper's payloads (4), smith's charter (4), its turns (2) and its channel (5); 14 of them a field, an option or a variant appended; and 19 version moves beside them.

### 3.2 Fingerprints at the opening

- **Each side states, for each wire, the versions it speaks** and each one's layout fingerprint (skein's `data/codec/README.md`, section 9). jig's channel agrees its own version from both sides' ranges, as skein's opening does (skein's `net/channel.md`, 5.2); the others are agreed beside it.
- **Agreed per wire:** the highest version both speak. The engine keeps each worker's agreed versions with its channel, and the fleet places by them (4.3).
- **Terms and limits are the agreed versions'.** Each side states the largest body of each kind as the agreed versions encode it, after translation, and checks the other's against the same; a configured limit both sides must share is stated beside them. So an appended field fails no terms that its translated value fits.
- **A version both sides name with different fingerprints** is an in-place change past the gate. The opening refuses it, naming the wire, the version and both fingerprints, and the worker redials as after any refusal.
- **No version in common** refuses the opening likewise, naming the wire and both ranges.
- **`versions` prints them** (`../protocol/binaries.md`): each wire's range, each version's fingerprint, and its floor (3.3).

### 3.3 The floor

- **Each build speaks every version of a wire from its floor to its current.** The floor is kept in the repository, one per wire.
- **It bounds what is spoken, not what is read.** A kept text and its forward patch stay for good, as a stored family's do (skein's `data/codec/command.md`, section 7), since stored turns, calls' names and deliveries' evidence keep the versions they were written in. Below the floor, a build no longer negotiates the version or translates down to it.
- **It moves only by a landing that says so:** one that raises it once no deployment speaks the versions below, or a breaking change (4.5), which raises it to its own version.
- **At least the previous version, for a wire that spans builds.** A landing that moves its version leaves its floor at or below the previous one, unless the change is breaking. A wire declared per-build in the bridge inventory (4.2) speaks only its build's version and needs no breaking declaration for that move.
- **Until bridges are built,** each build's floor is its current version: versions move and are compared, and nothing is translated. A worker of another layout is then refused at the opening by name, and the check (5.1) says it may not stay.
- **Workers keep up with it:** dev.fabrig.ai's workers are restarted in turn (7.2) before a new build's floor passes the versions they speak, which the check says.

### 3.4 A mismatch costs no task

- **What a worker cannot read is a host's refusal, not a run's failure.** An assignment refused for the version of its charter, its items or its transcript, which the opening should have prevented, ends its claim refused: the task claims again after a pause and spends no try (jig's `domain/tasks/lifecycle.md`, 2), and the fleet places no such assignment on that worker while its channel stands.
- **It is told as a mismatch,** naming the wire and both sides' versions, in the engine's log and the task's retry notice, never as a bare `charter`.
- **What the engine cannot read once the opening agreed** is a bridge's bug, caught by each bridge's tests (section 10). A result fails its run as invalid, as today. A turn is kept as its bytes and read later; one that cannot be read when its window is resumed closes the window, as today (jig's `domain/engine/executors.md`, 2).

## 4. Bridges

### 4.1 The newer side translates

- **In the shared protocol crates,** which both sides link. The side whose current version is newer than the agreed one translates what it sends down and what it receives up. The older side runs as if nothing had changed.
- **Up, by the forward patch:** each kept version to the next by the patch the command writes (skein's `data/codec/command.md`, section 3), over generic values (skein's `data/codec/README.md`, section 8), chained from the agreed version to the current. It is total: an added field takes the value every older record takes (`upgrades.md`, 4.3).
- **Down, by its reverse.** The command derives it wherever the forward patch is derived: an added field is dropped when it holds the value the older side assumes (an option at none, a list empty); a widened integer is narrowed when it fits; a dropped field goes down with the value its author declares the older side should assume. Otherwise the author writes the reverse, or declares which values the older side cannot hold (4.3).
- **Nothing goes up only to come down again.** Bytes kept in a version the reader speaks (a stored turn, a call's name, a delivery's evidence) cross as they were written. What crosses down is what the newer side's own code made.
- **What the older side must get back is not dropped.** A field an older side sends and expects to meet again (an identity it echoes, a value it resumes from) stays in the schema until the floor passes the versions that need it; dropping it sooner is a breaking change.
- **Written once, at the landing,** with its tests (section 10). The common change, an option appended, costs its author nothing beyond running the command.

### 4.2 What crosses

| Wire | Sent by | Down, to an older side | Up, from an older side |
|---|---|---|---|
| jig's channel records | both | each record by its kind's reverse | forward |
| temper's payloads | both | items, job specs, preparations | forward |
| a git identity's value | the engine | encoded for each channel in its agreed version, at the assignment and at each renewal, keeping its number, generation and validity | — |
| smith's charter | the engine | written in the worker's agreed version | — |
| smith's result | the agent | — | in its charter's version, read forward |
| smith's turns | the agent; the engine, resuming | a window resumed on an older worker | kept as written, read forward (`upgrades.md`, 7.2) |
| a call's name, a delivery's evidence | the agent; the engine, resuming | kept as written, crossing as written to a worker that reads it | kept as written, read forward |
| a host tool's input | the agent | — | by the tools' contract of the run's charter (below) |
| a job's request and result | the engine; the worker | by the kind's reverse | forward |
| smith's channel (`smith-channel`), per-build (4.4) | the worker; its spawned agent | no bridge: the worker pins its agent's program and configuration at startup | no bridge: a restarted worker spawns fresh agents, adopting none |

- **Stored turns are never rewritten.** The engine keeps a turn's bytes in the version its agent wrote, and patches them forward as it reads them (`upgrades.md`, 7.2; smith's `protocol/README.md`, section 8). A window resumed on a worker of an older version crosses as written when that worker reads it, goes down otherwise, or is placed elsewhere (4.3).
- **A git identity's value is a secret inside a grant,** opaque to jig's records, so its translation is temper's, for each channel, where it encodes the value (`../protocol/channel.md`, section 6). The two newest generations are kept in each version a connected worker speaks.
- **The host tools' contract is versioned whole.** A run's charter names the version its tools were written in, and a run adopted across an engine restart keeps calling them. The engine answers every version from its floor: a tool removed, renamed or changed in its input or meaning keeps its old contract until the floor passes it, so an adopted run's calls are answered as before. Dropping an old contract sooner is a breaking change.
- **A job's retained answer,** kept by a worker across the engine's restart, is read forward like any message.

### 4.3 What cannot go down

- **Only at an assignment may something wait.** An assignment whose charter or items use what the worker's version lacks, or whose window holds turns it cannot read, is placed on a worker that reads it. With none, it waits, spending no try and saying why; during a rolling redeploy (7.2), the next worker back takes it.
- **A running attempt must be reachable.** Everything sent to a run already on a worker (an inbox message, a call's answer, a grant's renewal, a cancel) must go down whole to its worker's agreed version, since that run cannot move. A change whose reverse is not total for these is a breaking change (4.5).
- **A newer worker before an older engine:** everything the worker sends (turns, results, its payloads, its jobs' results) must go down whole. A worker whose wires cannot is refused at the opening, naming them, so the engine goes first (7.2).

### 4.4 An agent is its worker's build

- **Pinned at the worker's start.** The worker takes its agent's program and the agent's configuration, as its own configuration names them (`../domain/worker/README.md`, section 5; `../protocol/binaries.md`, section 5), into its own data directory as it starts, and spawns that copy for every run until it exits. A redeploy replacing the program or the configuration changes what the worker's next start pins, never a running worker's agents.
- **So smith's channel never spans two builds.** It speaks one version and has no bridge. Its version still moves with its layout, so that a misconfigured program is refused at the opening, by name (smith's `protocol/README.md`, section 8). The pieces of it the engine keeps are wires of their own (3.1).
- **The boundary is local pipes,** from the worker's `smith-host-protocol` to its child agent's `smith-protocol-channel`, one channel per run (`../domain/worker/README.md`, section 5; smith's `protocol/channel.md`, section 2). The engine is neither endpoint nor a relay for these frames. An engine-only redeploy leaves the worker and its pinned agents together on their existing build, including agents spawned after the program at its configured path is replaced. A worker restart starts new channels with its newly pinned agent; agents from its previous start are never adopted. Thus the move from channel 6 to 7, adding drain, is a per-build version move, recorded in the bridge inventory (4.2), without a bridge or a breaking declaration for the deployment's cross-build wires.
- **Pinning is a prerequisite for keeping a worker.** A worker from before pinning must be stopped before its configured agent program is replaced, then restarted on the new build. Keeping that worker across a swap could make it spawn an agent from another build; the per-build exemption does not cover that rollout. Section 7.3's restart of every part establishes the pinned workers before engine-only redeploys rely on them.
- **An agent goes with its worker.** A worker that stops or dies takes its agents' process groups with it, and a worker's start finds nothing of its previous life still writing in its workspaces (smith's `domain/host.md`, section 4, whose status records today's gap).
- **Agents outliving their worker,** adopted by its next start, are later (the user, 2026-11-12; section 12).

### 4.5 Breaking changes

- **A change no bridge carries is declared breaking** in the landing that makes it: a reverse that can be neither derived nor written, a message to a running attempt that cannot go down whole (4.3), or a change of meaning the older side cannot be told. The gate asks for the declaration, as it asks for a patch.
- **It raises the wire's floor to its own version** (3.3), so no build speaks across it, and a redeploy across it restarts every part together (7.3).
- **A fresh mark on smith's turns** (jig's `domain/engine/executors.md`, 2) is a breaking change of the turns' wire.
- **Rare by design:** for each breaking change, the command names what makes it so and the bridgeable alternative, such as a new field beside the old, kept until the floor passes.

## 5. The engine alone

### 5.1 When the workers may stay

- **The check** (`upgrade --check`, `../protocol/binaries.md`), run by the new binary before the swap. Goal 648 builds it as a rehearsal of the store's patching over a copy, whose verdict on the workers compares the wire numbers the store keeps (`upgrades.md`, section 9, before this draft), and says unknown when it keeps none. Judging each worker by its ranges, fingerprints and terms is this draft's, and new. A worker may stay when, for every wire, its stated range meets the new build's range at or above the new build's floor, its fingerprints for the versions both name agree, its terms fit the new build's for the agreed versions (3.2), and no breaking change or fresh mark lies between. It names each worker that may not, and why.
- **What it reads:** beside the wires the store keeps for the engine (jig's `protocol/store/records.md`, 3), each worker's opening as the engine last authenticated it: its name, its ranges, its fingerprints, its terms and when. So the check runs with the engine stopped.
- **A worker with no record,** or none since the engine's last start, is reported unknown and treated as one that may not stay.
- **A worker that may not stay** makes the redeploy one of every part together (7.3).

### 5.2 The stop settles

- **Told to stop, the engine settles first** (`upgrades.md`, section 8; jig's `domain/engine/restart.md`, 1): its effects in flight on the forge settle, which main does, and no run is placed meanwhile, which main does not yet hold, since it still resumes the fleet while settling.
- **The workers keep their runs.** Their links stay up while the engine settles, and their grace starts when the link drops.

### 5.3 The window fits the grace

- **The window** is the settled stop, the swap, the store's patching (`upgrades.md`, section 6) and the start up to the first hello. The rehearsal measures it (`staged-upgrades.md`, section 5).
- **The workers' grace covers it with room.** The 60 s shipped is shorter than a start that patches. Its default rises to cover a measured window, and the engine's grace over its workers rises with it, since it must outlast every worker's stop bound (6.1): a worker gone for good has its runs moved elsewhere that much later. Default: 5 minutes for the workers' grace (section 12).
- **A window past the grace** ends the runs interrupted, with no try spent (jig's `domain/hosts/hub.md`, 6): a slow restart costs turns, never tasks.

### 5.4 The opening

- **Adopted runs go on.** The due work released together at the opening is admitted by the workers a few at a time (6.4), with no ramp of its own.

## 6. A worker restarted

### 6.1 The drain

- **Told to stop, a worker drains** (jig's `domain/hosts/hub.md`, 7). It says so on its link, takes no new run or job, and asks each agent to stop at the end of its current turn, once the LLM has answered and the turn's calls have their answers.
- **Within the drain bound,** by default 3 minutes (section 12). A run still in its turn at the bound is cancelled as today: a check in flight is cut, and the next attempt runs it again.
- **Then as today:** each agent gone; relays and deliveries settled; the work saved to its saved-work branch; the answer `Cancelled(Shutdown)`, the attempt interrupted with no try spent and its window left open; every turn and answer acknowledged before the worker exits.
- **A job the drain cancels** before its effect began ends interrupted too, where today it ends lost and spends a try. One whose effect began is left for the next attempt to recover, as after any loss.
- **The stop bound is enforced, not declared.** The bound a worker states at its hello (jig's `domain/hosts/hub.md`, 6) is a deadline the worker keeps over the drain, every cancellation, delivery and save, and its agents' teardown. A save still running at it is abandoned and its agent's group killed, so nothing of the attempt writes after it, and the engine's grace is sized from it.
- **The service manager's stop timeout** sits above the stop bound. A kill before it is a crash: the workspace's changes since its last save or delivery are lost, whatever turns were committed since, and the attempt is lost.

### 6.2 What a run loses

- **At most a turn's tail that the bound cut,** and whatever its saved work leaves out: ignored files and build outputs. Its conversation, its committed calls and its saved work go on in its next attempt, on any worker (jig's `domain/engine/executors.md`, 2).

### 6.3 A warm start

- **Stores kept,** each made once by one creator while the other preparations wait (goal 1492, in flight), and cloned without blobs, by the user's word of 2026-11-12.
- **Workspaces not kept.** The workspace-reuse mode stays internal, since an agent can write a workspace's `.git` and so reach later workstreams. With its store kept, a workspace is a local clone and a checkout.

### 6.4 Preparations a few at a time

- **One active preparation per worker by default,** shared by runs and jobs. The others queue before the checkout in order of arrival, a preparation's deadlines count from its admission, and its permit is released only once a cancelled preparation has settled. Run slots stay as they are (`../domain/worker/checkouts.md`, amended by its owner).
- **Three workers then make three preparations at once against a forge,** after a restart or a burst of due work alike.

## 7. The redeploy

### 7.1 The engine alone, by default

1. **A build that patches the store** is first rehearsed on a backup of the deployment's data and drilled on staging.fabrig.ai, and a backup is taken with the engine stopped (`staged-upgrades.md`, sections 5 to 7).
2. The new build's check (5.1) says the workers may stay.
3. The settled stop, the swap and the start, which the workers ride (5.2, 5.3).
4. A redeploy whose change the workers run goes on with 7.2.

### 7.2 Workers in turn

- **When:** on request, for a change the workers run; or when the check says a coming build's floor would pass them.
- **One at a time:** drain (6.1), swap, start, and wait for its hello and its first logins before the next. The other workers serve throughout: two of dev.fabrig.ai's three.
- **The engine first** whenever a wire moved. A worker goes first only when all its wires go down whole, which its opening checks (4.3).

### 7.3 Every part together

- **Only across a breaking change or a fresh mark, or with a worker that may not stay:** every worker drained, then the engine's settled stop, every binary swapped, and all started. Runs end interrupted with their work saved; none is lost.
- **Until sections 3 and 5 are built,** every redeploy is this one, drained once 6.1 is.

### 7.4 The configuration

- **As `upgrades.md`, section 10, from now on:** each build reads its own members and the previous build's. A member added is optional, with a default; one removed or renamed is accepted with a warning naming its replacement until a landing drops it. An agent's configuration is pinned with its program (4.4).

### 7.5 Exercised on dev.fabrig.ai

- **Every redeploy is a drill** (the user, 2026-11-12). Its script runs the check, does what the check says, and its referee reports afterwards: each worker's agreed versions; no try spent and no attempt lost because of the redeploy; no goal held for its infrastructure since; the window against the grace.
- **staging.fabrig.ai goes first** with a build that patches the store (7.1) or carries a breaking change.
- **The drills run on the deployments after landing.** Plans prove with the focused and fuzzy suites alone (section 10).

## 8. Clients

- **A client of another version is refused and told** (jig's `protocol/README.md`, section 7): `version not served`, with the ranges served. A page reloads to the engine's bundle, and `temper-client` and the TUI say to upgrade. Requests kept in another version are dropped with a notice (`upgrades.md`, 7.4).
- **Their versions move with their layouts too** (3.1), with no kept text and no bridge, so that a client of another layout is always refused by name rather than meeting a record it cannot decode. An engine serves its one version of each.
- **Not bridged, for now** (the user, 2026-11-12): 198 in-place changes to the clients' wires in the four weeks to 11-12, the most churn of any wire. Bridging them is later (section 12).

## 9. Infrastructure failures

- **A preparation failing for its infrastructure** (a git operation timed out, the forge unavailable) spends the same tries as a broken run, so a burst of them holds goals. Sections 6.3 and 6.4 end such bursts.
- **What remains is a policy of jig's own:** such failures pause with a backoff up to minutes and hold only past a bound of their own, naming the infrastructure, while a run's own failures keep their tries (jig's `domain/tasks/lifecycle.md`, 2). A preparation that always fails still holds, past its bound.

## 10. Proving it

Plans prove it with the focused and fuzzy suites alone; the drills come after landing (7.5).

- **Each bridge:** the kept version's golden bytes taken up equal the current's, and the current's taken down equal the kept version's. Fuzzy: any current value that can go down comes back up as itself; one that cannot is refused, never altered; any value of an older version taken up reads under the current code.
- **The gate:** a mutation of each kind it must fail on: a layout changed in place, a kind's number or direction changed, a kept text edited, a bridge missing, a floor raised without a breaking declaration.
- **The opening:** a worker speaking the floor, one speaking the current, one naming a version with another layout (refused, named), one with no version in common (refused, named), and one whose appended field fits its terms only once translated.
- **A story across versions** in the one-process world (`../testing/README.md`, 2.5), with a worker made to speak its wires' previous versions: a goal through a landing, a close job, a chat resumed, a run adopted across an engine restart and sent a message, a call's answer and a renewed git identity after it, and a host tool of the previous contract called by that run.
- **The engine alone:** an engine restarted mid-run with its workers kept, the run going on with no new attempt; a window past the grace ending it interrupted.
- **A worker drained:** mid-turn, the turn ends, the work is saved and the next attempt resumes with no try spent; a job drained before its effect ends interrupted; a save past the stop bound abandoned, with nothing written after it.
- **A mismatch refused:** an assignment its worker cannot read waits for another worker, spending no try, its notice naming the wire.
- **An agent is its worker's build:** with the configured program and configuration replaced under a running worker, its next agent is still the pinned one.

## 11. Who carries what

temper/migration carries every part, across the trees.

| Part | Where |
|---|---|
| Versions moving with layouts; kept texts, reverses, floors, the breaking declaration and the gate's checks (3.1, 3.3, 4.1, 4.5) | skein's patch command and check; each wire's crate |
| Fingerprints, ranges and terms at the opening; translation in the channel's components; placement by agreed versions; a mismatch as a refusal (3.2, 3.4, 4.1 to 4.3) | jig's channel and fleet; temper's payloads |
| The charter's, result's and turns' bridges; calls' names and deliveries' evidence; stopping at a turn's end; agents' process groups (4.2, 4.4, 6.1) | smith |
| The host tools' contracts; jobs' requests and results; git identities for each channel (4.2) | temper's engine and worker |
| Agents pinned at the worker's start; the drain and the enforced stop bound; jobs on a drain; preparations a few at a time (4.4, section 6) | temper's worker; jig's hub |
| A settled stop that places nothing; the graces (5.2, 5.3) | jig's engine; temper's configuration |
| `versions` and `upgrade --check` with ranges, fingerprints, terms, floors and workers' openings (3.2, 5.1) | temper's binaries |
| The policy for infrastructure failures (section 9) | jig's tasks |
| The redeploy script, its referee and the drills (section 7) | the deployments' operator |

## 12. Later and open questions

- **Agents outliving their worker** (the user, 2026-11-12: later): a worker's restart adopting its agents still running, which needs their channel over something that outlives the worker, and the worker's state on disk.
- **Clients bridged:** pages and clients of the previous version served, once their wires settle.
- **The drain's bound.** Default: 3 minutes.
- **The workers' grace.** Default: 5 minutes, with the engine's grace above every worker's enforced stop bound.
- **How long a floor stays.** Default: until dev.fabrig.ai and staging.fabrig.ai have both run past it for a week.
- **The policy for infrastructure failures** (section 9). Default: a bound of its own, in jig.
- **A newer worker before its engine** (4.3). Default: only when all its wires go down whole.
