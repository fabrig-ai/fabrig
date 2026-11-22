# A round at preparing

Provisional, 2026-11-17. Whether a landing's round at preparing has a
prepare that lives, read afresh at each of the landing's steps from
jig's context and the landing's job record, and how such a round
recovers with no person across an engine's restart, a worker's loss,
and a person's stop and release. It is `../draft/prepare-restart.md`'s
design. It details `landing-prepare.md`, 1.2 (*Lost or partial*) and 15 for the
round's prepare; the prepare job itself, its branches and its outcome
stay `landing.md`'s, and its events `landing-history.md`'s.

## 1. In one page

- **Read, never assumed:** at each step of a landing whose round is at
  preparing, the root reads whether the round's prepare lives, has
  answered, or is absent (section 2), from the landing task's context as
  jig holds it (its inbox and its delegates) and from the landing row's
  job record. The policy's `asked` marks record intent, never admission:
  alone, they never make the round wait.
- **Live:** the round waits. jig runs the job, and retries a lost
  attempt as a new attempt of the same child once its writer slots
  settle (section 4). The landing never makes a second prepare child
  while one lives.
- **Answered:** the answer is read before anything else is decided
  (`landing-prepare.md`, 1.2, *Its outcome*), and nothing is asked again for
  what it prepared.
- **Absent:** the round asks the prepare once again, with no person,
  for the members above the prefixes already recovered; jig's procedure
  step fences it, so concurrent wakes make one replacement (section 5).
- **At the start,** the forge reads afresh and wakes each landing: the
  first decision after a restart is a step jig offers with its restored
  context, never one made on a fact the forge supplies itself
  (section 3). A person's release of a landing at preparing is such a
  step too (section 6).
- **No silent wait:** a round at preparing waits only on a live child,
  which its job's deadline and tries bound (`landing.md`, 4.2, `landing-prepare.md`, 1.2), or
  on the reads an answer needs, which are retried as any landing read
  is; never on a mark alone.

## 2. The prepare's cases

- **The current round and suffix:** the round is the landing's
  preparation number (`Landing.preparation`, counted from 1); the suffix
  is its members still in the landing, unlanded, with no prefix in the
  round, in landing order: the members its prepare decision marks asked.
- **The fact** is the policy's `Preparation`
  (`temper-engine-domain-forge-landing`), which the root's `preparation`
  reads (`temper-engine-domain`, `route/forge_route/landing_route.rs`)
  in this order, the first case that holds deciding:
  1. **An answer in the inbox:** the landing task's inbox holds a
     prepare child's answer. A failure, or a report the round cannot
     read, is `Failed`; a single landing's conflict is `Conflict`, as
     today; any other report is `Idle`, its outcomes heard with the step.
  2. **A live child:** one of the landing's prepare children has not
     ended. Held (past its tries, or under a person's hold) it is
     `Failed`; active, due, backing off before a retry, or closing, it is
     `Running`. Any live prepare child counts, whatever round asked it:
     two never write the members' branches at once, and a round that
     ends settles its prepare before cleanup (`landing-prepare.md`, 1.1).
  3. **An answer the round keeps:** the job record (`LandingRow.job`,
     `landing-history.md`, 3.3) names the current round and exactly the
     suffix; its child has ended; its outcome is none of those that end
     with no answer (`lost`, `cancelled`, `timeout`, `failed`); and the
     round still keeps that answer's report for a member of the suffix
     (`Round.failure`, `landing-prepare.md`, 1.2, *Its outcome*). It is `Idle`:
     the step reads the kept report and the members' branches.
  4. **Proven absence:** otherwise, `Absent`. No child was admitted for
     the current round and suffix (no record names them: none was
     claimed yet, or the record names an earlier round or another
     suffix), or the record's child ended with no answer, or the round
     no longer keeps its answer.
- **An earlier round's admission alone never counts,** as live or as
  answered: a record of an earlier round, or of another suffix, is no
  evidence, and a live child counts by being live, not by its record.
- **What the policy does with it** is unchanged (`landing-prepare.md`, 1.2):
  `Running` waits; `Idle` waits, while the suffix is marked asked in this
  round, only until the step takes a prefix, which clears the marks;
  `Absent` asks, within `landing-preparing.md`, section 5's bound; `Failed` holds where the
  suffix was asked; a conflict
  leaves. A round that ends waits on `Running` before its cleanup
  (`landing-prepare.md`, 1.1).

## 3. At the start

- **The forge's start reads, then wakes** (`landing.md`, 15): for each
  live landing it reads afresh what a step reads, and then wakes the
  landing's procedure. It decides nothing on that read: every way it
  ends, a failed read among them, is the wake, never a `Wait` or any
  other decision, which would take the step jig offered the landing.
- **It supplies no preparation of its own:** the hearing the forge
  steps itself, `LandingHeard`
  (`temper-engine-domain-forge-vocabulary`), carries `preparation:
  Option<landing::Preparation>`, none from the start's read and the
  root's reading (section 2) from every step jig offers. A hearing with
  none decides nothing about the prepare: were one to reach the policy,
  it would be heard as `Running`. At a round that ends, a hearing with
  none reads no prefix that a running prepare could still push.
- **It never replaces a real step's hearing:** where a step jig offered
  is already gathering for the landing, the start's read leaves its
  hearing and its entry as they are.
- **The first decision** is that of the step jig offers with the
  landing task's restored context, its unread inbox and its delegates'
  phases as the store kept them (jig's `domain/tasks/lifecycle.md`, 3:
  a restart offers a due step again against fresh facts), so that the
  round's prepare is read as at any step (section 2).

## 4. A lost attempt

- **jig retries it in the same child:** an attempt whose worker is lost,
  its link lost past the fleet's grace or its adoption failing after a
  restart, ends `lost` (`landing-history.md`, 2.2), and jig tries the
  same child again as a new attempt within its tries; past them it
  holds the child, and the landing holds (`landing.md`, 4.2).
- **After its writer slots settle:** the new attempt is claimed only
  once the lost attempt's writer slots on the members' branches are
  free, after the forge has read those branches afresh (jig's
  `domain/tasks/holds.md`, 3), so that what the lost attempt pushed is
  known before anyone writes again. Two attempts never write at once.
- **The landing waits meanwhile:** the child lives, so the round's
  prepare is `Running` (section 2) and no second child is made.
- **What it pushed is kept:** the retried job reads the members'
  branches and reuses the prefixes the lost attempt delivered
  (`landing-prepare.md`, 1.2, *Lost or partial*); its answer gives the round
  each prefix, recovered or new.
- **The same with a restart and without:** a lost link, and a restart
  that finds the attempt unadopted, end in the same retry.

## 5. Asked again

- **Proven absence asks once:** a step that reads `Absent` while the
  suffix is marked asked in the current round asks the prepare again,
  with no person. Before it delegates, the members' branches are read,
  and the prefixes found are taken (`landing-prepare.md`, 1.2, *Lost or
  partial*), so that the replacement prepares only the members above
  them: prefixes already recovered are kept. The same prepare, its
  member, head, main and suffix unchanged, is asked again only once,
  and no sooner than one `poll` after the step that read its absence:
  that step waits `prefixes` (`landing-waits.md`, 2) until then, and
  the members' branches are read again at the deadline. A third ask of
  the same prepare holds the landing instead, as a prepare failed past
  its tries does (`landing.md`, 4.2, *No silent wait*), until a
  person's release, which asks again from a fresh count. A different
  prepare, after a head move, main moving, a cut or prefixes taken, is
  asked at once. The landing keeps the prepare it last asked across
  rounds, since a stale round re-forms its round
  (`Landing.last_prepare`, a field of the landing's row with a
  default).
- **Fenced by jig's step:** the forge's step that saves the marks asks
  the root, within the same root decision, to answer the step jig
  offered with the prepare child in its batch; jig accepts one decision
  per offered step and makes the child in that acceptance (jig's
  `domain/tasks/lifecycle.md`, 3). The marks and the child are thus
  stored together: no restart falls between them. The next step is
  offered only after it, and reads the child live. So concurrent wakes,
  a restart and a release each make at most one replacement.
- **Marks with no child** are reached only where the child ended with
  no answer, or the round no longer keeps it (section 2, case 4).
- **Its events:** the replacement child's attempts write their own
  `job` and `job-ended` (`landing-history.md`, 3.3). A step that finds
  the prepare live or answered writes nothing (`landing-history.md`,
  3.2).

## 6. A stop and a release

- **A stop** of a landing at preparing places a hold on its subtree
  (jig's `domain/tasks/lifecycle.md`, 10): neither the landing nor its
  prepare child steps, and the child's live attempt is cancelled.
- **A release** of that hold, the landing's own policy phase not held,
  is a step like any other: jig offers it with the landing's context,
  and section 2 decides. A released child lives again, and is retried
  while the round waits; an absent prepare is asked once (section 5).
  The marks are not cleared by it, and no one need act after it.
- **`released`** (`landing.md`, 4.2) still means the release of the
  landing's own hold, which prepares again in a new round.

## 7. Testing

Focused and fuzzy suites alone, each test failing first:

- **The root's reading:** the four cases of section 2, each from a
  landing row, its job record and the landing task's inbox and
  delegates: an answer in the inbox; a live child, due before its
  retry, and a held one; an ended child whose answer the round keeps;
  no record, an earlier round's record, another suffix's, an ended
  child with no answer, and an answer the round no longer keeps.
- **The forge's start:** a start read decides nothing, a failed read
  included, and leaves a real step's pending hearing and entry; a
  hearing with no preparation reads no prefix of a round that ends.
- **The domain's world**
  (`temper/tests/engine/domain/tests/suite/forge_routes/landing_restarts.rs`,
  beside the restart that keeps the same worker job), each with no
  person acting and the prefixes recovered kept:
  1. a restart right after the decision that saves the round's marks
     and admits the prepare child, before any worker claims its first
     attempt (the nearest cut to the marks alone, section 5): that one
     child prepares, no second is made, and the round gates and lands;
  2. an admitted prepare whose worker is lost, without a restart and
     with one: jig retries the same child once, its second attempt
     claimed after the first's writer slots are free, and the round
     lands;
  3. a prepare that ended with its answer kept before a restart: the
     answer is read, and nothing is asked again;
  4. a stop and release at preparing with the prepare absent, its child
     ended with no answer while the landing is stopped: the release asks
     it once.

## 8. Later

- **Why an engine loses its workers** is its deployment's matter
  (`../draft/prepare-restart.md`, 2).
