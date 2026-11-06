# parallel-faults: the plan

From `inputs/request.md`, six faults from a parallel replay, whose causes were read, not reproduced. 1 and 2 must land. Everything below was judged from reading the code on 2026-11-06, with real git run by hand where it says so; no test has run yet, so each task reproduces first.

## Parts and tasks

| Request's item | Part | Task | Needs |
|---|---|---|---|
| 1, a stalled hold strands a finished producer | `stalls` | `live-delegate` | |
| 2, that hold reaches nobody | `stalls` | `hold-reaches` | `live-delegate` |
| 3, a stray log in the branch | `snapshots` | `hidden-logs` | `merged-start` |
| 4, snapshots drop local commits | `snapshots` | `charter-words` | |
| 5, response idle | none | none: `notes/idle-bound.md` | |
| 6, `after` starts half-merged | `merged-start` | `merged-start`, and `charter-words` for "detached" | |

**Critical path:** `live-delegate` then `hold-reaches`, about 190 implementer-minutes. A rerun waits on `live-delegate`, so it goes out first.

## What was decided

- **1.** A live delegate isn't a stall: a producer, repair, resolution or merge job with a live task, running or not, since jig bounds those itself. A gate's task stays bounded, as 8.4 names it, and that's an open question in `forge.md`, section 20. A result on a `Stalled` hold ends the hold as a green head does; under other holds a release takes it. The "why 57, not 30" is traced by `live-delegate`. The reading's guess is that `since` starts at the producer's request, after any wait for a dependency.
- **2.** From reading, jig's escalation looks right when a hold reaches it, and at stock's hour an undecided one would have passed up to a person, so an empty inbox after 1 h 43 suggests no escalation was made. The likely cause is `change_decision` dropping a decision whose procedure isn't due, after the row was already held. That's unproved, so `hold-reaches` tests it, with defaults for each other cause.
- **`live-delegate` before `hold-reaches`.** They share `temper-engine-domain`. The blocker goes first, and it records what the report's path did in `notes/stall-trace.md` before its fix removes that path, because `hold-reaches` can't reproduce it afterwards.
- **3. The rule taken:** a file named `.*.log`, at any depth, is never staged, for a save or a delivery. The request's inference holds in part: untracked files are staged, as `git.md` 4.7 said to, and ignored files were already left out. A new file can't be told from a run's new work, so the rule goes by name. It doesn't stop a stray of another name, which is the repository's `.gitignore` to say (`git.md`, section 10). `*.log` was rejected because this repository tracks `.log` fixtures, and new ones must still land.
- **4.** Local commits are dropped by design and the receipt already names what was pushed, so the part of the request saying delivery should say what it pushed already holds. The host's resume words name no commit, so those names come from the agent's own transcript or summary. The fix is words: the coding charter tells a run. A project's own charter doesn't get them.
- **5.** No change, as the request says when reading can't settle it. `notes/idle-bound.md` holds the reading and the evidence that would settle it. The new fact: `temper init`'s template has 30 s, against smith's 120 s and 300 s presets, and 30 s is the one that fits the arithmetic, so a raise in `smith/` might not reach the deployment at all. Nobody has read the replay deployment's `agent.json`.
- **6.** A code fault, not only words. A merged start never checks out its commit, so an agent sees "still merging". Real git refuses a second merge in that state, exit 128, run by hand with 2.47, so a multi-head start should fail to prepare; `merged-start` tests that. "Detached" is by design on every run, so that half is words only (`charter-words`). Whether it's the same fault as `landing-fixes`' can't be said: no plan or commit in this repository has that name.

## Overlaps not avoided

- `live-delegate` and `hold-reaches` change `temper/crates/temper-engine-domain/src/route/forge_route.rs` and add stories under `temper/tests/engine/domain/`, which `sign-in-new-tab/return-goes` also changes (`temper-engine-domain/src/**`, `tests/engine/domain/**`). Their stories are in new files to keep hunks apart.
- No task touches `temper-client-*`, so `task show`'s and the panel's words stay as they are. `hold-reaches` reports anything it finds there under `## Left`.
