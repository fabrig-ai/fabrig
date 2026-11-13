# A change's stall waits for its dependencies

Draft, 2026-11-13. A change whose task waits on another task to end is
held as stalled while it waits, though nothing it could do has failed.
This draft has its stall count only once it can move.

## 1. What happens today

- **A change cut `after` another** (its task's dependencies,
  jig's `domain/tasks/lifecycle.md`) sits at `producing` with no
  producer until every dependency ends: temper starts its producer
  only then.
- **Its stall counts all the same.** Past the deployment's
  `forge.change_policy.stall_ms` (30 minutes by default), the change
  and its task are held: `no progress at producing for 30 minutes:
  waiting for its producer's head and result`. The hold escalates to
  its holder, the goal above it or the project's role.
- **A release only rearms it.** Released while the dependency still
  runs, the change is held again a stall later, so whoever decides
  the escalation can do nothing useful but wait for the dependency.
- **Seen on a deployment:** a goal's second and third changes, each
  after its first, held for more than two hours while the first's
  producer worked, each hold sitting in the project role's inbox as
  an escalation.

`../domain/forge/change/repairs.md`, *Stalls*, already says a wait for
the change's agent is not a stall. A wait for its dependencies is the
same kind of wait, and the design does not say so.

## 2. Wanted, ranked

1. **A wait for its dependencies is not a stall.** While the change's
   task has a dependency that has not ended, the change's stall does
   not count. It counts from the last dependency's end, as it counts
   from a delegate's end today. *Stalls* in
   `../domain/forge/change/repairs.md` gains a sentence beside "Nor is
   a wait for its agent", and the code follows it.
2. **A hold already placed ends when the wait does.** A change held on
   a stall at `producing`, with no producer started and a dependency
   pending, is released by the deployment, with no party's release,
   when its last dependency ends, as *A result that arrives on a held
   change is kept* releases one whose delegate ends. Its stall then
   counts from that end. This clears holds placed before the repair
   reaches a deployment.
3. **A dependency that ends failed or cancelled** leaves the change as
   it does today: this draft changes only when the stall counts, not
   what a failed dependency does.

## 3. Out

- The stall's bound, and the stalls of every other step.
- jig's own task stalls (an executor's, a person's, an escalation's).
- How an escalation is routed or decided.

## 4. Tests

Cheap, focused and fuzzy suites only; walk = false.

- In temper's engine domain story `forge_routes/stall_hold.rs`, beside
  its stall tests:
  - a change after a live task, past the stall, is not held; once the
    dependency ends it is held a full stall later if its producer gives
    nothing;
  - a change held on a stall at `producing` while its dependency was
    live is released when the dependency ends, with no party's release,
    and its producer starts;
  - a change whose producer is live is still not held (today's rule,
    kept).
- Run only the new and affected tests by name.

## 5. Formats

If when the stall counts from must be stored where nothing stores it
today, the store's version moves and its patch comes with it:
upgrades are supported (`redeploys.md`).
