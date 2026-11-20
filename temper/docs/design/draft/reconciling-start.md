# The engine opens even when its forge can't answer

Draft, 2026-11-20, on the user's word: the fix for the start that never opened, "done properly". On 2026-11-20 at 02:30Z dev.fabrig.ai's new engine started, bound its listeners and never opened. Its forge's eight outbox places were all held by cancelled landings, and six goals held branches that had never been made. Reading the forge afresh waited for those makings, a making waited for an outbox place, and places free only when the outbox settles, which comes after the read. The first stop was ignored until systemd killed the engine at 90 s. Making the six branches by hand let the next start open in ten seconds (P129).

That cycle is one case of a wider gap. The start waits on its forge as one barrier, with no bound, and no state of the forge's can open it but a good one: a forge that's down, an uncertain write whose find keeps failing, a read that keeps being rate-limited, or the cycle above all hold every person, chat and ctl call out, though none of them needs the forge. A stop is heard only once the start ends, and the one line the start logs names its phase, never what it waits for.

## 1. In one page

- **A start reads and doesn't write** (section 2). Reading afresh asks only what the forge holds; a goal branch found missing is made like any other write, after the outbox settles. No read waits for an outbox place, so the cycle can't form.
- **The engine opens once its own records are in** (section 3): its store loaded and linked, its runs adopted. Its forge reconciles while it serves. A decision that depends on a forge fact waits until that fact has been read afresh, and a write waits until the uncertain writes on its resource are settled. Chats, people, settings, sign-ins, reads and runs that don't touch the forge go on at once.
- **A stop is heard in every phase** (section 4): before the start's mark it ends the start at once, after it the start ends at its next phase and settles. The service manager's stop timeout is taken from the engine's own bound.
- **A wait says what it waits for** (section 5): past a bound, each start phase and each forge's reconciliation names its waiters by kind, with counts and the oldest. The health census and the forge's pages show it.
- **A rehearsal proves the opening** (section 6): a candidate opens over a copy of the deployment's store with no network, which now means something.
- **The tests replay the incident and its class** (section 7): focused worlds that start with a full outbox and a missing branch, with the forge down, with an uncertain write that never resolves, and with a stop in each phase; fuzzy sweeps that restart anywhere under a random workload with tiny tables.

## 2. A start reads and doesn't write

- **Today:**
  - The forge's read-afresh phase ends when six things are empty: the client's fresh pass, the landings' startup gathers, pending CI reads, goal-file reads, lost writers' reads and goal-branch makings (`temper/crates/temper-engine-domain-forge/src/domain.rs`, `read_afresh_complete`).
  - A making restarted for a held branch that the forge doesn't have, with no creation entry, asks the root for an entry number, reads the landing tip and keeps a creation entry, once the outbox has room (`goal_branch.rs`, `wait_for_room`). `temper/docs/design/domain/forge/branches.md`, 1, allows this case: "the restart cut its making before it committed one".
  - Restored entries count against the outbox's limit and stay paused until the settle phase (`room.rs`; `domain.rs`). The room line wakes no one before the outbox is ready, which it becomes at the settle's end. So a making that needs a place keeps read-afresh from ending, and read-afresh keeps the settle from starting.
  - The same holds in jig's own terms. Its model has a restart read afresh "before deciding anything new" (`jig/docs/design/domain/model.md`, 4), and a branch's creation is a new decision with a write.
- **Wanted:**
  - A making at restart is two steps:
    - **Its read** belongs to read-afresh: is the branch there, at which commit, and is a creation entry of its own in the outbox.
    - **Its write** belongs to the work after it. A branch found missing with no entry becomes a making waiting for room, admitted in the room line's order once the outbox is ready, as in steady state (`branches.md`, 1).
  - Read-afresh's end counts reads only. No term of it waits for an outbox place, an entry number or anything the settle frees.
  - A change that needs the goal branch waits for the branch, bounded by the change's stall as today, never the start.
  - Every other read-afresh term is checked for the same flaw by the milestone that builds this section: no term may wait on a write, or on an effect the settle releases.
  - Revise `temper/docs/design/domain/forge/branches.md`, 1 (a making at restart) and `skein/docs/design/forge/client.md`, 8 (the restart's steps), with their status files.

## 3. The engine opens once its own records are in

- **Today:**
  - jig's start script runs load, link, sweep, switch, wires and adopt runs, then reads every connector afresh, then settles every outbox, then opens (`jig/crates/jig-domain-core/src/start/script.rs`).
  - The service opens its clients only once the root is open, so every client request is refused as not ready before then, including those that need no forge: a person's sign-in, a chat, settings, a task's read (`temper/crates/temper-engine-service-component/src/engine.rs`).
  - Due tasks don't start until the root opens (`temper/crates/temper-engine-domain/src/domain.rs`).
  - Read-afresh and the settle have no bound. A forge that stays down keeps the fresh pass and the reads the outage wrapper retries waiting forever. An uncertain write whose find keeps failing keeps the settle waiting forever. A read that is rate-limited again and again requeues forever (`skein/crates/skein-forge-client/src/keep/answer.rs`; `temper/crates/temper-engine-domain-forge/src/outage.rs`; `skein/crates/skein-forge-client/src/outbox/retry.rs`; `calls.rs`).
- **Wanted:**
  - **Opening needs the store, not the forge.** The script opens after adopt runs. Read-afresh and the settle become the forge's reconciliation, run after the opening, and the forge reports it reconciled when both end.
  - **Facts are read before a decision depends on them, one resource at a time.** That is already jig's rule for working sets (`jig/docs/design/domain/connectors.md`, 5.1). A restart's reads don't add a second, global barrier. While a forge reconciles:
    - a decision that reads a resource whose watch is still pending its first fresh read waits for that read, as a change's read session waits today;
    - a procedure that judges several resources (a landing's gather, a tier's state) waits for its own reads, as in steady state;
    - nothing judges a resource from what the store held before the restart alone.
  - **A write waits for the uncertain writes on its resource.** The outbox admits new entries and orders them in its room line as soon as it has restored its table. It sends a write to a resource only once every restored entry on that resource has had its first look. A write on another resource goes ahead.
  - **What goes on at once:** people signing in and out, chats, settings, task and census reads, worker links and adopted runs, due tasks whose work doesn't touch the forge. A run's call that needs the forge waits in that call, within the run's own deadlines, as it does when the forge is slow.
  - **What waits for the forge:** forge pages show "reconciling" and what they wait for (section 5). Landing trains, tiers, change openings and goal-branch makings start their next decision once their own reads are fresh.
  - **The audit comes first.** Before the script changes, the milestone that builds this section lists every procedure that today decides on a forge fact without its own read, relying on the start's barrier instead. Each one gains its read, or waits for the forge's reconciled report, by name. The list goes into this draft's status.
  - Revise `jig/docs/design/domain/model.md`, 4 (a restart's reads, per resource), `jig/docs/design/domain/engine/restart.md` (the script and its phases), `jig/docs/design/domain/connectors.md`, 5.1 (a connector's reconciliation after opening), `jig/docs/design/protocol/service.md`, 3 (when clients are admitted), and temper's forge design (`temper/docs/design/domain/forge/`) for what each forge procedure waits on, with their status files.

## 4. A stop is heard in every phase

- **Today:**
  - A first signal stops the listeners, but the root hears its stop only once it is ready, that is, open (`temper/crates/temper-engine-service-component/src/stop.rs`, `advance`). A start that can't open can't stop either: on 11-20 the stop waited until systemd killed it.
  - jig's service design already says a stop before the start's mark closes the components that wait for it at once (`jig/docs/design/protocol/service.md`, 5, step 4). temper's service doesn't do this.
  - The settled stop's own bound, from the forge client's settle, is twice the write lifetime plus the clock margin: 602 s at the stock figures (`skein/crates/skein-forge-client/src/outbox/stop.rs`). dev.fabrig.ai's unit stops at 90 s.
- **Wanted:**
  - **Before the mark** (the wires commit): a stop ends the start at once, as jig's service design says. Nothing outward has happened, and the next start begins again.
  - **After the mark and before the opening:** the start ends at the end of its current phase, whose work is local and finite, then stops as a settled stop would.
  - **After the opening:** the settled stop as today, the forge's reconciliation included. A forge that reconciles never holds a stop past the settled stop's bound, past which the stop ends unsettled and names what it leaves (`temper/docs/design/draft/upgrades.md`, 8).
  - **A second signal aborts,** as today.
  - **The bounds derive from one another.** The engine prints its stop bound and its start's report bound (section 5) in its `--version` details. A unit written for it, by temper's help or a deployment's description, takes its stop timeout from there, with a margin, and never sets a start timeout below the store's open.
  - Revise `temper/docs/design/domain/root.md`, 8.2 (the stop during a start), `temper/docs/help/server.md` (the units' timeouts) and the configuration's derived figures, with their status files.

## 5. A wait says what it waits for

- **Today:** the one start diagnostic is "the core is not answering: starting: <phase>", logged once when a client is refused (`temper/crates/temper-engine-service-component/src/core_wait.rs`). It names no waiter, count or age.
- **Wanted:**
  - **A start phase past its report bound** logs what it waits for, by kind, with counts and the three oldest by name, then again at each doubling of its age. The bound is a derived figure, a few times the phase's work on the deployment's store, never a deadline that fails the start.
  - **A forge reconciling past its report bound** does the same for its pending reads and its unsettled entries: "reconciling: 6 goal-branch reads, 2 uncertain writes; oldest g10084 for 14 min". Its census row and the forge pages show the same words.
  - **A table full** (outbox entries, read admission) names its holders, oldest first, in the same census. That is what the 11-20 hang needed and what no line said.
  - Revise the deployment census's design and the forge pages' (under `temper/docs/design/client/forge/`), with their status files.

## 6. A rehearsal proves the opening

- **Today:**
  - `upgrade --check` stops at the store's switch, before the wires, adopt runs and the forge (`temper/crates/temper-engine-domain/src/restart.rs`).
  - A deployment's drill that serves a copy with no network can only show that the listeners bind, since the start then waits for a forge it can't reach. That passed on 11-20, and so did the build that hung.
- **Wanted:**
  - With section 3 built, an engine over a copy with no network opens, and reports its forge reconciling. A rehearsal's pass is "opened within its bound", read from the engine, never "listeners up" or "alive for a minute".
  - `upgrade --check` gains a step that runs on to the opening in a sandbox with no network, its forge refused at once rather than awaited, and reports the opening and what the forge would have waited for. Revise `temper/docs/design/protocol/binaries.md`, 2 (the check) and `temper/docs/design/redeploy.md`, 4.1, with their status files.

## 7. Tests

Focused and fuzzy suites only, within their budgets. Each case is a world below the real loop: memory and CPU, no process, network or disk.

- **Focused,** in `temper/tests/engine/domain/` and `temper/tests/engine/forge/`:
  1. A restart with an outbox of two, both held by ended landings, and a held goal branch the forge doesn't have, with no entry: the engine opens; the making waits in the room line; when a holder is released the branch is made at the landing tip. (11-20.)
  2. The same with room: the branch is made after the settle, not during read-afresh.
  3. A restart with the forge refusing every call: the engine opens; a sign-in, a chat and a settings change answer; forge pages say "reconciling" with their waiters; when the forge answers, reconciliation ends and the waiting procedures go on.
  4. A restart with an uncertain write whose find keeps failing: the engine opens; writes on other resources go ahead; writes on that resource wait, and it is named past the report bound.
  5. A read rate-limited at every attempt: the engine opens, and the read is named past its bound.
  6. A stop at each phase of the start, as rows of one table test: before the mark, the start ends at once with nothing outward; after it, the start ends at the phase's end; after the opening, it settles. Each next start opens.
  7. A start phase and a reconciliation past their report bounds log their waiters, by kind, count and oldest.
- **Fuzzy:**
  - The engine domain's restart sweep restarts at any iteration, not only between operations (its `follows` sweep drains before each restart today), under a random workload of goals, changes, landings, cancellations and holds, with the outbox, read admission and sign-ins at two places.
  - After every restart, the referee expects the opening within a bound, and every procedure's next decision once its resource has been read.
  - The fault kinds this draft adds (a restart at any iteration, a stop in any start phase, a forge down through a start) go into the fault vocabulary the execution worlds share, so they combine with theirs in one sweep.
- **Paid for** within the suites' budgets, which this draft doesn't raise. For one, the engine domain's fuzzy module runs eleven focused tests again unchanged (`temper/tests/engine/domain/tests/suite/main.rs`).

## 8. Out

- **The outbox leak:** cancelled landings keep their gate dispatches' places. It made the cycle reachable, and it is fixed by the goal-execution work that owns it. This draft makes a full outbox harmless to a start, not rare.
- **Who stops a deployment's engine and how:** the deployment's operator. This draft gives them the bounds to use.
- **Workers' restarts:** another draft.

## 9. Later

- A connector other than the forge, when one exists: it follows section 3's rules by being a jig connector.
- A forge's reconciliation shown on the person's home page, beside the goals it holds.
