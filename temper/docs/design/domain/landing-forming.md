# A landing's forming

Provisional, 2026-11-19. How the decision that forms a landing
(`landing.md`, section 9) is made whole: its task made first, and the
landing's row, its members' links and its branches' holds saved only
once jig holds that task; a make jig refuses forming nothing, its train
waiting a queue window, the refusal a warning in the engine's log; and
a start over a store that already holds a landing whose task was never
made ending that landing, so its members wait again. Which units form a
landing, and when forming is judged, is `landing.md`, section 9; the
landing task is `landing.md`, 4.2; the restart is `landing.md`, section
15.

## 1. In one page

- **The task first** (section 2): the forming decision asks jig to make
  the landing's task before it saves anything of the landing, and jig
  answers the make within that decision.
- **Saved only once made:** the landing's row, its members' links to it,
  its branches' holds and its members' `joined` news are made in that
  same decision only where jig then holds the task. No stored row and no
  hold names a landing task that was never made.
- **A refused make forms nothing** (section 3): its members stay ready
  and in no landing, keeping their places; the train stays free; the
  refusal is a warning in the engine's log naming jig's reason; and the
  train forms again no sooner than a queue window later.
- **A store that already holds one starts** (section 4): the start's
  link pass ends each live landing whose task jig doesn't hold, its
  members waiting again, and drops each hold whose task jig doesn't hold
  and under which no task jig holds writes, logging each. Anything a
  task jig holds names, it still refuses.

## 2. The forming decision

### 2.1 Its order

The decision that forms a free train's landing (`landing.md`, section 9)
does, in this order:

1. **Its names reported** to jig: each member's branch in the landing
   (`landing.md`, 5.1), as before.
2. **Its task's make:** jig's `Event::ConnectorRepair` with the
   project's procedure seed, the landing's spec, contract and holdings
   (`landing.md`, 4.2), as before.
3. **Its admission:** where jig holds a task by the seed's number, the
   landing's row (`landing.md`, section 12), which links each member's
   change to it and tells each member's goal it joined (`landing.md`,
   4.2), then a
   hold of each member's branch by the landing's task. Where jig holds
   none, the refusal (section 3), and none of these.

- **Answered in the decision:** jig's make drains within the decision
  that asks it, made or refused, its names reported before it, as the
  status procedure's admission already reads (`temper-engine-domain`'s
  `status_route::admission`). The admission reads jig's tasks after the
  make; it never assumes either answer.
- **Activation after:** the made task's activation and first step come
  after the admission in the decision's order, so its first step finds
  its row.
- **One commit:** the decision commits whole, so a store holds the
  landing's row and its holds with its task, or none of the three.

### 2.2 What jig answers

- **A refused make** is told to every connector as `Ask::RepairRefused`
  (jig's `domain/authority.md`, section 6), which carries:
  - `repair: Option<u64>`, the task jig's refusal names where it names
    one, as before;
  - `number: u64`, the refused make's own number, its seed's;
  - `why: tasks::Refusal`, jig's reason.
- **temper keeps it for the admission:** temper's handler of
  `RepairRefused` (`route/asks.rs`) keeps `why` against `number`,
  unstored, where a forming awaits its admission by that number, so the
  admission names it. Its status and queue repairs read `repair`, as
  before.
- **A made task** is read as jig's tasks holding it: jig sends no
  acknowledgement of a make.

## 3. A refused make

- **Nothing of the landing is saved:** no row, no link, no hold, no
  news and no history event. The seed's number stays consumed: jig never
  reuses one.
- **Its members wait,** ready, in no landing, each keeping the time it
  first became ready, so the train's next landing takes them in their
  order (`landing.md`, section 9). No member is held, repaired or told:
  the refusal is the deployment's room, configuration or funding, never
  a member's fault.
- **Its train waits a queue window:** the running engine remembers,
  unstored, each train whose forming jig refused, with the wall time and
  the reason. Forming is judged for that train again at the first
  judgement at least the deployment's queue window (`forge.queue_window_ms`,
  `../protocol/configuration/limits.md`) after the refusal; other trains
  form as ever. The train's next forming made forgets it, and a restart
  forgets every one, judging forming for every train (`landing.md`,
  section 15). So a refusal neither stops its train for good nor forms
  it again in its own decision.
- **The record,** a warning in the engine's log for each refusal whose
  reason differs from the last one logged for its train since it last
  formed:

  ```
  BINARY serve: warning: landing L into BRANCH of repository F/R not formed: jig refused its task: WHY; its changes C1, C2 wait
  ```

  `L` the refused make's number, `BRANCH` the train's branch, `F/R` its
  repository's forge and repository numbers, `WHY` jig's
  `tasks::Refusal` as Rust's `Debug` prints it, and the changes in the
  order they would have stacked. A refusal repeated for the same reason
  logs nothing more; the changes view still shows them waiting
  (`landing.md`, section 14).
- **Every refusal is the same here:** the live-task or project-task
  room (`Live`, `Project`), the holdings (`Holds`, `HoldTaken`), the
  funder (`Funding`), a busy or unavailable resource. The log names
  which.

## 4. A store that already holds one

A store written before section 2 was built may hold a landing whose task
jig never made: its row live (`waiting`, round 0), its members linked to
it, and its branches held by its number. Its train is never free, and
the start's link pass refused that store, a hold linking a missing task.

- **In the link pass,** after the restored names' and tiers' links and
  before `missing_link` judges the restored rows (`temper-engine-domain`'s
  `Work::ForgeRestored`):
  - **A live landing jig holds no task for,** its phase neither landed
    nor cancelled, its outbox holding no entry, and jig's tasks holding
    no task by its number, is ended: its row erased, as a refused make
    leaves none (section 3). Each member's change linked to it is left
    as a cancelled landing leaves it (`landing.md`, 4.2): ready, in no
    landing, keeping its place, unless it is recorded landed. Nothing is
    written to the landing history: the landing had no step.
  - **A hold whose task jig doesn't hold,** on a forge not ended, so
    that the link pass would refuse it for its holder, is dropped, its
    row erased, where no task jig holds writes under it: neither its
    writer's run's task nor any of its design writers. A hold such a
    task writes under is left, and the link pass refuses it as before.
  - **Each is logged,** a warning in the engine's log, `L`, `BRANCH`,
    `F/R` and the changes as section 3 names them, `T` the hold's task:

    ```
    BINARY serve: warning: landing L into BRANCH of repository F/R has no task: ended; its changes C1, C2 wait again
    BINARY serve: warning: hold on BRANCH of repository F/R by task T, which has no task: dropped
    ```
- **What it still refuses:** every other link the pass judges, as
  before: anything a task jig holds names, a hold a live writer names, a
  live landing with an entry in its outbox, and every other family's
  links.
- **Its writes are the start's,** in the link pass's commit, before the
  sweep and the switch, as jig's start repairs its projections there
  (jig's `domain/engine/restart.md`), so a second start finds nothing to
  repair.
- **Its members after:** the restart judges forming for every train
  (`landing.md`, section 15), and the members form the train's next
  landing in their order, through section 2.

## 5. Testing

- **Focused,** `temper-engine-domain-world`'s `suite`:
  - a forming whose make is refused, the live-task room full: no
    landing row, no hold, each member ready in no landing, the warning
    naming `Live`, no second forming in that decision; with room freed
    and a queue window passed, the train forms and lands. Red before
    section 2 is built, green after;
  - a store holding section 4's landing starts: the row erased, its
    members' links cleared, its holds dropped, a warning each; its
    members then form a landing and land;
  - the same store with a hold a live task writes under still refuses,
    naming that hold's link.
- **Focused,** `temper-engine-shell`'s lines: the warnings' words; and
  `jig-domain-core`: `RepairRefused`'s `number` and `why`.
- **Fuzzy:** the landing and walking suites stay green, unchanged.

## 6. Later

- A refused forming as a landing-history event per member, readable by
  `landing history` (a kind of `landing-history.md`, 2.2), and as news
  to its members' goals.
- A dropped hold's branch, where it was ever pushed, stays on the forge:
  no task releases it.
- A forming refused for good (a landing's holdings past jig's bound)
  logs once and waits each queue window; nothing holds its train for a
  person.
