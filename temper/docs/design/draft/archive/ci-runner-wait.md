# Waiting for a CI runner doesn't count as a stall

Draft, 2026-11-15, on the user's word. On 11-15, CI ran two at a time on one runner, each run taking 16 to 32 minutes, so a pull request's run could wait 30 to 45 minutes for a runner. A change's 30-minute "no progress at checking" stall counted that wait, so six changes were held while merely queued. One of them held main's landing turn, and the hold passed the turn on. A held change can't act on its run's result, so a person had to release each (P55).

## 1. The checking stall counts running, not waiting

- **Today:**
  - `checking` sends an unknown or pending CI verdict through `settling` to `waiting`, which holds the change at `change.since + limits.stall`, 30 minutes by default (`forge.change_policy.stall_ms`) (`temper/crates/temper-engine-domain-forge-change/src/domain.rs`; `temper/docs/design/domain/forge/change/repairs.md`, *Stalls*).
  - Polling a pending CI verdict leaves `since` alone. Queued and running look the same, because commit statuses say only `pending`.
  - The forge's Actions reads do tell `waiting`, `blocked` and `running` apart (`Read::Run`, and on main `Read::JobState`, `skein/crates/skein-forge/src/forgejo/plans/`). temper reads the job's state only after the stall, to explain the hold, and never to reconsider the clock.
- **Wanted:**
  - While a change's CI is pending, it reads the run's or job's execution state for that head, before deciding a stall.
  - The checking stall has its own execution clock. Time spent waiting or blocked for a runner counts nothing. The clock starts at the first observed running state, and repeated polls don't restart it. Counting from the first observed running state undercounts by up to one poll, and the docs say so. Persist the clock across restart: the store is registered, so use a default, not a new version.
  - A new attempt or a rerun starts the clock afresh. With several jobs, the clock runs while any of them is running.
  - Where the forge gives no execution state, a separately bounded grace for the unknown state applies, then the stall as today. The docs say which applies when.
  - A real stall still holds, and a hold still passes the landing turn on, as `temper/docs/design/domain/forge/queues.md` has it. A change merely waiting for a runner keeps its turn.
  - Revise `../../domain/forge/change/repairs.md`, *Stalls*, and the line that keeps the job state diagnostic only, with their status files.

## 2. A landing's gate timeout counts running too

- **Today:** a turn landing's gate expiry counts from its dispatch (`attempt.sent`), and its verdict treats waiting and running alike (`temper/crates/temper-engine-domain-forge/src/landing.rs`; `temper/crates/temper-engine-domain-forge-landing/src/verdict.rs`; `temper/docs/design/domain/landing-proof.md`, 2.1). A gate run queued behind others can time out without ever having run.
- **Wanted:** the run follower (`runs::Follow::answered`, `temper/crates/temper-engine-domain-forge/src/runs.rs`) keeps the run's execution timing, and the gate expiry counts from the first observed running state, as section 1 does. Revise `../../domain/landing-proof.md`, 2.1, to say so.

## 3. Out

- The 30-minute allowance itself. A run that really takes 32 minutes still stalls. The figure is the user's to set.
- An authoritative start time from the forge, which would need skein's reads extended.

## 4. Tests

Cheap: focused and fuzzy suites only; walk = false.
- The change policy's tests (`temper/crates/temper-engine-domain-forge-change/src/tests.rs`, the stall cases):
  - a run queued past 30 minutes doesn't stall;
  - the clock starts once it runs, and a run past the bound stalls;
  - completion advances;
  - polls and a restart keep the clock;
  - a rerun resets it;
  - mixed jobs;
  - no execution state follows the stated rule.
- The stall-hold story (`temper/tests/engine/domain/tests/suite/forge_routes/stall_hold.rs`) and the queue-turn tests (`temper/crates/temper-engine-domain-forge/src/tests/queue_turn.rs`): a change waiting for a runner keeps its turn.
- The landing and run-follower tests: a gate queued past its timeout doesn't expire before it runs.
- The dialect's edge tests (`skein/crates/skein-forge/src/forgejo/plans/tests/edges.rs`) if a read is added.

Run only the new and affected tests, by name.
