# A landing's prepare survives a restart

Draft, 2026-11-16, on the user's word. On 11-16, a landing formed its round at 15:57Z. Two minutes later the engine lost its workers and came back as a new process. The round then sat at "preparing, gate no key yet" for nearly two hours, with no prepare or gate event, until a person cancelled the landing. A stop and release changed nothing, and nothing landed into main meanwhile (P68).

## 1. A round at preparing knows whether its prepare lives

- **Today:**
  - The landing policy marks each unprepared member `asked = Some(preparation)` and returns `Decision::Prepare`. The forge saves that before jig delegates the prepare job as a child task (`temper/crates/temper-engine-domain-forge-landing/src/domain.rs`; `temper/crates/temper-engine-domain-forge/src/landing.rs`; `temper/crates/temper-engine-domain/src/route/forge_route/landing_route.rs`).
  - An admitted job is durable, and jig re-adopts or retries it across a restart and a lost worker.
  - Two places can still leave the round waiting for good:
    - At start, `landing::refresh` gives the round a synthetic `Preparation::Idle`.
    - `admitted_prepare` treats a matching earlier round and suffix as admitted, without looking at its outcome or checking that work is still recoverable.
  - Either one feeds the policy's wait, `Running` or `Idle` with the round already asked, which returns `Wait(None)` and stays there.
  - A stop and release reset jig's task, but not the round's `asked` marks. temper passes `released` only when the landing phase itself is held, so a release at preparing changes nothing.
- **Wanted:**
  - `preparation` and `admitted_prepare` tell three cases apart, for the current round and suffix:
    - live work that jig still runs or will retry;
    - an ended child whose answer is retained, which is read before anything else is decided;
    - proven absence, where no child was admitted or it ended with no answer.
    - An earlier round's admission alone never counts as live.
  - At start, the forge reconciles each round at preparing through jig's restored context, instead of a synthetic `Idle`.
  - Proven absence asks the prepare once again, with no person. Admission is fenced, so concurrent wakes make one replacement. An admitted attempt that was lost is retried as its own child through jig, after its writer slots settle, never as an overlapping second child. Prefixes already recovered are kept.
  - A release of a landing at preparing reconciles the same way.
  - Revise `temper/docs/design/domain/landing.md` (the round's preparing and its recovery) and `temper/docs/design/domain/landing-history.md`, with their status files, to say so.

## 2. Out

- Why the engine lost its workers at that moment: the deployment's own matter.

## 3. Tests

Cheap: focused and fuzzy suites only; walk = false.
- In `temper/tests/engine/domain/tests/suite/forge_routes/landing_restarts.rs`, beside the cut that keeps the same worker job:
  1. A restart after the round's intent is saved but before the prepare is admitted: the prepare is asked once, and the round gates and lands.
  2. An admitted prepare whose worker is lost, without a restart and with one: jig retries it once, with no overlapping writer, and the round lands.
  3. A prepare that ended with its answer retained before the restart: the answer is read, and nothing is asked again.
  4. A stop and release at preparing with the prepare absent: it is asked once.
- In each, no person acts and recovered prefixes stay.

Run only the new and affected tests, by name.
