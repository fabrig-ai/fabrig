# Repairs, resolutions and releases: open questions

- **A green head no wake reaches** (`repairs.md`, `../keeping-up.md`, 2): a held change reads on a wake,
  and a poll wakes it only when what it read of the pull request moved. Under
  a rule's required contexts, a check they do not name can keep the pull
  request's CI as it was while theirs pass, so the hold waits for a hint or a
  person's release, which makes no fresh repair. Default: so; it costs time,
  not a repair.

- **A retry before the read** (`repairs.md`): a run ending into a retry wakes no change,
  so a backoff shorter than `poll_ms` can start another run at a green head,
  left to finish as any run is. Whether jig tells a procedure of its delegate's
  retry, so that none starts. Default: as it stands.

- **A hold at gating at a green head** (`repairs.md`): past the bound or short of
  authority there, a hold can be a gate's or CI's, and its state does not say
  which. Saying so changes what a hold stores. Default: it waits for a person's
  release, which decides afresh.

- **A repair held for its brief** (`repairs.md`): the repair's task is held, naming
  its reason, while the change stays `Repairing` and its panel shows no hold:
  a person reads why on the task, and releases it there. Default: so; whether
  the panel says it too stays open.

- **A review's or a repair's brief under a drift hold** (`repairs.md`, *Another
  writer's*): while its change is held for a moved head, each try of its brief
  meets the same refusal and counts as capacity, taking a brief slot for its
  reads every time, until a release takes the head or a cancel ends the task,
  the loop a resolution no longer makes (a resolution's moved head, above).
  Whether the change ends those too when it holds for a drift, as it ends a
  resolution. Default: as it stands; the request of 2026-11-12 named a
  resolution's brief alone.

- **A forge out of reach or not connected** (`repairs.md`): `Unavailable` counts as
  transient, so a long outage holds each repair preparing through it, naming
  why; `NotConnected` pauses with no bound and no hold, its notice naming why.
  Default: so; the allowance is the user's.

- **A failing check's repair brief:** a check gate that says no is
  repaired as a review that says no is, and its repair's brief carries
  the pull request's reviews section (`repairs.md`). A check's verdict is not a
  review on the forge: whether the brief should carry the check's own
  output instead is open.

- **What failed on a queue's group** (`repairs.md`, `../queues.md`, 1): an ejection's repair
  brief carries the forge's reason alone, the pull request's `dequeued`,
  since skein's `Pull` names no commit of the group. The alternative
  reads the group's CI and gives the repair its failing output.
  Default: the reason alone.

- **A clean update's other words** (`repairs.md` and `../updates.md`, section 1): the result still says
  `the base broke it`, and the client panel calls its repair a semantic
  conflict, although the path proves only that CI failed after an update.
  Aligning those words needs the client's owner too. Default: the task's
  instructions and brief name the failed check; the result and panel keep
  their words until that follow-up.

- **A read left at its cutoff keeps running** (`repairs.md`, *A forge
  section is sized by its cutoff*): skein's forge client has no way to
  withdraw a read it admitted (skein's `forge/client.md`, 5), so a read a
  section left, a page of files with each side of each file its own
  request, holds one of the client's calls to its end, and a brief tried
  again at once can find the client still reading for its last attempt.
  Default: the read runs on and its answer is dropped; withdrawing an
  admitted read is skein's client's to design (2026-11-14).

- **A repair of another reason that changes nothing** (`repairs.md`, *A CI
  repair that changes nothing checks its head again*): a gate's or a
  review's verdict at the head stays valid there, so checking the same
  head again would only ask the next repair, and a landing's, a
  resolution's and an ejection's red is not CI's at the change's head.
  Default: such a repair finding nothing to do holds the change, as a
  failed one does, until a person decides (2026-11-14). A landing repair
  that finds only flakes says so with its verdict instead, and its
  release runs the gate once more (*A landing repair that finds only
  flakes*; 2026-11-17).

- **Every failed job at once** (`repairs.md`, *A CI repair that changes
  nothing checks its head again*): one job runs again for each repair that
  changed nothing, as one write is one job's; a head with several failed
  jobs spends a repair on each left red. Default: one job, since a
  deployment's CI fails in one gate job (2026-11-14).

- **A landing repair that only merges its base** (*A landing's
  repair*; `tools/pre-pr/design.md`, 2): judged against
  the landing's base, a merge of it alone selects the change's own
  packages and their readers, as the landing lands them, but not a failed
  test outside them, so it can pass though the gate failed; and a repair
  whose receipt keeps no base is judged against its head, where a merge
  alone selects no tests. Default: its words name the failed
  tests, which its agent runs by name; carrying them into the check's
  selection, as `temper/reds` carries reds (`../reds.md`, 5.2), is a
  wire to its worker and waits (2026-11-15).
