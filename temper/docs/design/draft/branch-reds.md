# branch-reds: temper keeps the reds a goal's branch carries, and the tree keeps none

A delivery's check builds and tests nothing at its base, so it can't tell a failure the delivery made from one its branch already had. The reference project lets it tell them apart by a list in the tree, `.temper/known-reds.md` (`../../../../tools/pre-pr/design.md`, 5): a goal's coordinator and implementers write there the reds its branch carries, and the check passes a failure the list names. A list every goal edits, in a file every goal's branch carries, conflicts at nearly every merge of the base into a goal's branch, needs an agent to settle each time, and grows past its cap; and nothing prunes it. This draft has temper keep a goal's reds as records, recorded by the same judgement as today, handed to the check at each run and ended by what the checks observe. It also has the check tell a failure under load from a real one, by running its process again with nothing else running, and file what then passes as a flake. temper/dev builds it, with the reference project's check.

## 1. In one page

- **A goal's reds are temper's records** (section 3): failures its branch carries that no live change owns, each keyed as the project's check keys it, with the head it was recorded at, why, by whom and when.
- **Recorded by the coordinator's judgement,** by a tool, where it now edits a file (section 4).
- **Handed to each check as it starts** (section 5): when the run tells its host its checks are starting, the worker reads the goal's reds from the engine, keeps those whose head the check's base contains, and writes them, in the check's own entry form, to a file in the repository's git directory. The check passes a failure an entry names, as it does today.
- **Ended by what lands** (section 6): an ordinary pass of a red in a change's check ends it once that change lands into the goal's branch; the coordinator may end one; a goal's reds end with it.
- **Under load, its process again** (section 7): a failing test's process runs once more after the run, the same tests in the same order, with nothing else of the check running. A test that passes then failed under load: it refuses nothing and is filed as a flake. One that fails again is judged as today, and one-process's own diagnostic still tells a test that depends on an earlier one.
- **The passed-tree cache keys on the reds too** (section 5), so ending a red is never hidden by an earlier pass.
- **The list goes** (section 8), once no goal started before the records still carries it.
- **No base build,** as the reference's check decided (`../../../../tools/pre-pr/design.md`, 8).

## 2. Where it starts

- **The list in the tree:** entries `- test <binary id> <name>` and `- lint <package> <code> <file>`, anything else prose, at most 64 entries and 8,192 bytes; past the cap, no entry counts and the check refuses (`../../../../tools/pre-pr/design.md`, 5). The passed-tree cache keys a pass by tree and base alone (7).
- **Its writers:** a goal's coordinator, confirming that a red a change inherited has no live owner, adds it to the goal's notes and to the goal branch's list and delivers; the change's implementer adds the line it is given (`stock-charters.md`, 10.1, 10.2, 10.4 and 10.5). Each goal writes its own prose at the top. Nothing removes an entry but a person or an agent, and nothing checks that one still fails.
- **What it costs:** in one week of the reference repository's history the list changed on main 36 times and on branches reachable from it hundreds of times; it twice passed its cap, once in a resolution that stopped to ask what to keep. In one eight-hour window it was in 10 of 11 conflicts between main and a goal's branch, and the only file in 5. Of its test entries on main that day, 11 of 25 named tests ignored in their source, which never run.
- **How a check reaches temper today:** the run executes the project's check itself; a refused check goes back to the agent and never reaches a delivery, and the run tells its host only that its checks started and ended (smith's `domain/run.md`, 8.1; `../domain/agent/README.md`, 5).
- **What temper already keeps:** after-landing's tiers record failing tests by their own ids and count flakes; a flake is a finding, filed and matched by its signature (`../domain/after-landing.md`, 4 and 5.2; `../domain/findings.md`, 2, 4 and 5).
- **The merge job's rule** for the list (`known-reds-merge.md`), a stopgap until this draft is built, settles it without an agent.

## 3. A goal's reds

- **A red** is a failure a goal's branch carries that no live change of the goal owns: its key, as the project's check names a failure (opaque to temper, within `red_key_bytes`); the goal; the goal branch's head it was recorded at; why, in a line; who recorded it, a task or a party; when; and its generation, the goal's count of records, so that a later record of the same key is told from an earlier one.
- **Applies where its cause is:** a red counts for a check whose base contains the head it was recorded at. A change based before it doesn't carry its cause, so a failure there is the change's own.
- **Kept** while the goal lives, at most `branch_reds` at once; a record past it is refused, naming the bound, and the goal is held, since a branch carrying that many is a goal to look at, not a list to grow. An ended red goes at once: what it was stays in the goal's activity.

## 4. Who records one

- **The goal's coordinator, by judgement,** when a change asks about a failure its check refused and the coordinator finds it inherited and owned by no live change: by a host tool, `red`, that records it with its key and why at the goal branch's head the change was based on, or ends one (`../domain/agent/tools.md`, 2). The change's next check passes it, without the change delivering anything for it.
- **Nobody else:** an implementer neither records nor ends one; it asks, as today.

## 5. How the check reads them

- **At each check's start:** when the run tells its host that its checks are starting (smith's `domain/run.md`, 8.1), the worker reads the goal's reds from the engine, keeps those whose recorded head is an ancestor of the check's base (the merge base of the landing pin and the head, `../domain/worker/checkouts.md`), and writes them to `temper/reds` under the repository's git directory (`git rev-parse --git-path temper/reds`): a first line `temper-reds 1`, then one entry a line in the check's own form, each key once, written to a temporary file and renamed. It removes the check's results file (section 6) first. A read that fails or doesn't answer within `reds_read_ms` writes no file.
- **A run of every check:** a refused check, a retried one and the check before a `finish` each start afresh, so each reads the reds as they are then: a red recorded mid-hold counts at the next check.
- **What the check does with it:** a failure an entry names is inherited, named in the summary, and refuses nothing; a build failure never is. A missing, malformed or oversized file is no reds, and the summary says which.
- **The passed-tree cache** keys a pass by tree, base and the reds file's digest, so a pass under a red that has since ended is not reused (`../../../../tools/pre-pr/design.md`, 7). A cache hit observes nothing (section 6).
- **The reference's check** reads this file beside the tree's list while the list stays (section 8), the union counting, with no cap of its own: the bound is temper's (section 3).

## 6. How one ends

- **Observed, then landed:** the check writes `temper/check-results` under the git directory, `temper-check 1` and its run's identity first, then each listed red its run selected and passed outright, `passed KEY`; a test ignored, starved, unobserved, or passing only when its process ran again (section 7) is no such pass. When the run tells its host its checks ended, passed or refused, the worker sends the results, identified by task, attempt and check number, and the engine keeps them with the change at that tree, once per identity. When that change lands into the goal's branch, each red it passed, recorded before the pass was observed (by generation), ends.
- **By the coordinator:** the same tool ends one, with why.
- **With the goal:** a goal's reds end when it ends.
- **Never by a stale file:** the worker removes the results file before each check, so a file left by an earlier check or a timeout ends nothing.

## 7. A failure under load

- **Its process again:** after its run, the project's check runs each failing test's process once more, the same binary's selected tests in the same order up to that test, with nothing else of the check running, before it judges. A test that passes then failed under load, since its predecessors were the same: it refuses nothing, and the summary names it, with both logs' tails within its bound. A test that fails again is judged as today: inherited when an entry names it, otherwise the delivery's. One-process's own alone diagnostic still labels a test that depends on an earlier one, and that failure stays the delivery's (`../../../../tools/tiers/design.md`, 11.6).
- **Bounded:** at most `rerun_processes` processes; past it, none is run again and every failure is judged as today, since that many is no flake.
- **Filed as a flake:** each test that passed when its process ran again is written to the results file, `flake KEY`, with its logs' tails; the engine files or counts a `flake` finding on the repository for it, its source the delivery's check, its signature the repository and the key, routed to the area owning the test's package or, with none, to the project's triage (`../domain/findings.md`, 2 and 4; `../domain/areas.md`), once per check identity.
- **The landing's gate stays the guard:** a delivery passed on a flake still lands only through its landing's gate, by the gate's own rules (`../domain/landing.md`, 7.3; `../domain/findings.md`, 5); a change that made a test flaky meets it there, or in the findings that grow after it lands.

## 8. What goes, and when

- **The charters' words** that edit the list (`stock-charters.md`, 10.1, 10.2, 10.4 and 10.5, and the reference's texts, `../../charters/fabrig/`): the coordinator records a red with the tool (section 4); the implementer's "a red you didn't cause" passes when the check names it inherited, and otherwise asks. A goal's note of known reds goes too: the records are the note. These change with the records, so no goal started after them writes the list.
- **The list,** its cap, the merge job's rule for it and its stopgap draft go once every goal started before the records has ended: until then the check reads both, and the list's existing entries keep counting for the branches that carry them.
- **`delivery-checks.md`'s inheritance** then reads from the records, still with no base build, and its status says so.

## 9. Settings and bounds

- **`branch_reds`** per goal (stock 64), **`red_key_bytes`** (stock 256), **`rerun_processes`** per check (stock 4) and **`reds_read_ms`** (stock 5,000), among the forge's limits (`../protocol/configuration/limits.md`).
- **The files:** `temper/reds` at most `branch_reds` entries of `red_key_bytes`, with its header; `temper/check-results` at most `branch_reds` passes and `rerun_processes`' tests as flakes, each flake's log tails within the summary's bound.

## 10. Upgrades

- **A new stored family,** the goal's reds and the changes' kept observations, with its version and patch (`../protocol/store.md`, 2 and 4).
- **The worker's wire** gains the read of a goal's reds and the results' report: it moves its version and the newer side bridges the previous one (`redeploys.md`, 4.1 and 4.2). A worker of the previous version reads and reports nothing: its checks have no reds file, as a tree without the list has none today. A hold of a goal with reds is placed on a worker that speaks the new version (`redeploys.md`, 4.3).
- **The host tools** gain `red`, versioned as the host tools' contract is (`../protocol/wires.md`). The run's channel to its host is unchanged: it already tells checks' start and end.

## 11. Acceptance

Focused and fuzzy suites alone, each failing first:

- the coordinator records a red; the change's next check passes that failure as inherited without any delivery for it; a red past `branch_reds` refused and the goal held;
- a red recorded at a head the check's base doesn't contain doesn't count; one it contains does;
- the worker writes the reds file fresh at each check's start, a refused check and a check before `finish` included, and removes the results file first; a slow or failed read writes none;
- a cached pass is not reused after a red ends; a cache hit reports nothing;
- a change's ordinary pass of a red ends it only when that change lands into the goal's branch, and not a later record of the same key; a change that never lands ends nothing; ignored, starved, unobserved and rerun passes end nothing;
- a test that fails under load and passes when its process runs again refuses nothing and becomes a flake finding, counted once per check; one that fails again refuses; one that depends on an earlier test stays the delivery's; past `rerun_processes`, no rerun;
- the coordinator's end and a goal's end;
- a worker of the previous wire version: no file, failures judged as the delivery's;
- the reference's check reading both the file and the tree's list, then the file alone;
- fuzzy: random record, check, land and end sequences keep each goal's reds within bound, each key once, and never pass a failure no applicable record names.

## 12. Later

- **Main's reds by code,** from after-landing's records, once a tier's test ids map to the check's keys and its records say at which commit each began; until then a red main carries reaches a goal as any other, by its coordinator.
- **A landing's gate recording reds:** the failing tests of a goal's landing at its candidate, recorded on its branch by code where now its coordinator judges.
- **A base run for a red in doubt,** within the check's bound, where no record names it and it is cheap to run at the base.
