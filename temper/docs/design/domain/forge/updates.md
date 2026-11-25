# Lazy, mechanical updates

This document holds lazy, mechanical updates, split from [`README.md`](README.md).

## 1. Updates: lazy and mechanical

Under a project's queue; by turn, a landing prepares its candidates
instead (`../landing-prepare.md`, section 1).

- **Lazy:** a change takes its base when it is first in its queue, about
  to get a run for another reason (`change/README.md`, 2), or at its check with its head
  already behind and CI not concluded there, once for a head a push gave
  it (`change/README.md`, 2, step 3). It never does only because its base moved.
- **By its landing branch's rule:** only where the rules covering its
  landing branch ask up to date (`change/gates.md`, 2, `Requires`). Under rules asking
  none it is never updated, first, at its check or before a run, and a
  conflict with its base holds it (`change/README.md`, 2, step 6).
- **Mechanical:** the update merges the base into the change's branch,
  with no LLM. The forge makes it itself (its pull request's update, by
  a merge), so it is an effect, as a merge is. A merge rather than a
  rebase: the branch only moves forward, so it is never forced, and the
  heads before the update stay the change's own.
- **Made when the forge says, read afresh.** Forgejo has made the
  update when it answers; GitHub accepts it and makes it after (skein's
  `forge/github-operations.md`, 2.2). Either way the update is done once the head,
  read afresh, contains the base's tip it asked for, which is also how
  its recovery finds it (`effects.md`, section 1), and the new head is heard as any
  push. That head is the change's own, recorded as its branch's head in
  the step that settles the update, never drift (`change/drift.md`,
  1, *Its own writes are its own*).
- **Lost, and made again.** An update whose answer is lost is looked for
  by that read and, not found after its lifetime, checked and made again,
  up to `write_attempts` (skein's `forge/client-effects.md`, 1.5). It holds no one.
- **Let go: a lost write.** An update settled `Failed` with `Timeout` or
  `Unavailable`, or with `Stale` or `BaseMoved` from the check after a
  send of its own (skein's `forge/client-effects.md`, 1.5), says nothing of the
  change: the forge did not answer, or what it was decided on has gone,
  perhaps to a copy of its own. The top keeps its account and reports no
  result, and a change updating or preparing (`../landing-prepare.md`, 1.2, and `../landing.md`, 12
  ) with no result and none outstanding decides again on a fresh read
  of its head and base, the base being its pin, the tip the update asked
  for, though the branch has moved on since:

  | Its head | Holds the pin | It |
  |---|---|---|
  | where it was | no | updates again, counted in `updates` |
  | where it was | yes | goes on at that head |
  | moved | yes | takes it as its update, as if made |
  | moved | no | holds, `BranchMoved`: no update of its own shows there |
  | any | not yet known | waits, up to `stall` |

  So a lost write is recovered against its pin, each new update spending
  one of `updates` and up to `write_attempts` lifetimes. Past `updates`
  it holds, `Updates`, a person told why in the last write's account
  (`update pull request: forge timed out; ...`).
- **Refused stale: a replan** (2026-11-16, `../../draft/archive/landing-update-loop.md`).
  An update or a preparation the client's check refuses before any send
  of its own was never made: `Stale`, its pull request at another head,
  or `BaseMoved`, at another base commit (skein's `forge/client-effects.md`,
  1.4). The top reports it as its own result, `EffectResult::Stale`, and
  the change decides again on a fresh read of its head, its base's tip,
  and its pull request's head and base commit:

  | Its head | Holds the pin | The base's tip and the pull request's base commit | It |
  |---|---|---|---|
  | moved | yes | any | takes it as its update, as if made |
  | moved | no | any | holds, `BranchMoved` |
  | where it was | yes | any | goes on at that head |
  | where it was | no | agree, at a commit other than the pin | replans at once: a fresh update, or preparation, against that commit, its new pin |
  | where it was | no | disagree, or agree at the pin | backs off, then tries the pin again |
  | any | not yet known | any | waits, up to `stall` |

  It never sends against a pin the forge has refused without backing
  off first. A replanned preparation is the change's own: its landing
  reads the member afresh once the change leaves preparing, as after any
  preparation (`../landing.md`, 12).
- **Its backoff.** The change saves the time of the first stale refusal
  of the update it has outstanding (`Change.stale`), so a restart goes
  on with the same backoff. It tries the pin again no sooner than its
  last decision plus a delay: the time from that first refusal to its
  last decision, at least 1 second and at most 60 seconds, so the
  delays double from 1 second. Past `stall` from the first refusal it
  holds, `Stalled`, a person told why in the last refusal's account
  (`update pull request: base moved; ...`). The time is dropped as the
  change leaves updating or preparing, and at a release.
- **`updates` counts sends.** The decision after a stale refusal, a
  replan or a try again, spends the refused update's count and none of
  its own, so `updates` counts what the forge was sent, and a forge
  whose view lags spends `stall`, never `updates`. An update after a
  send, lost, is counted as before.
- **Its words say what moved:** `head changed` for `Stale`, `base moved`
  for `BaseMoved`, in the write's account, whose layout is unchanged
  otherwise (`update pull request: base moved; resource OWNER/REPO #N`). The history's `update` and `updated` lines carry the pin
  as their base and the base commit the forge reported beside it
  (`../landing-history.md`, 2.2).
- **Other failures** of an update, apart from a conflict below, are a
  person's: a protection or a forbidden write holds it, `Failed`, in its
  account. A release of it, or of `Updates` or `Stalled`, drops the
  result and decides again as for a lost write, its count afresh: a pin
  the base has left is then refused at its check, and replans.
- **What a late copy costs.** Once the change has moved on, a copy that
  moves the head is drift (`change/drift.md`, 1): held, and its release takes the head and
  checks it, so a person and a CI run; gates ask again. A merge names its
  exact head (`effects.md`, section 1), so a copy lands nothing unchecked. A rebase's
  copy can also drop a push made since it read the head (skein's
  `forge/client-effects.md`, 1.3): the same hold, and a repair so dropped finds
  its failure again when the head is checked. A push by another hand so
  dropped is that hand's to make again, the hold saying the branch moved.
- **A conflict** makes the forge refuse the update, a merge's or a
  preparation's rebase alike, and the change is
  resolved (`change/repairs.md`): by the merge job, where its project names one, then
  a resolution for what the job leaves. The job's result names each file
  it leaves with its kind and, for a file one side deleted, the side and
  the commit that deleted it (`../../protocol/tooling.md`, 4.1). It settles no
  file one side deleted or renamed: the resolution's agent judges
  whether the change still needs it, a file the base deleted staying
  deleted unless it does, and its result says what it kept and why
  (`change/repairs.md`).
- **A catch-up** takes a base's commit into a branch that no update
  reaches, a goal's own (`branches.md`, 1): a change into that branch, made by the
  goal's coordinator naming the commit (`../agent/tools-change.md`, 1), main's
  tip as it reads it, only for a change of the goal that needs what the
  base gained (`../../draft/squash-in.md`, 4); its producer
  the merge job, which merges the commit into a branch made at the
  landing branch's tip (`change/README.md`, 2, step 1); its project must name a
  `merge_job`. It lands as any change into the branch does; under
  `turn` its train merges it, never squashing it (`../landing-prepare.md`, 1.2),
  so the goal's branch shows it as a merge of the base. Nothing makes
  one but its requester's call.
- **A semantic conflict** is CI failing on a change's head after a clean
  update. The name records that order, not proof that the base caused
  the failure. Its repair's brief carries the failed check and its output
  before the base merged in and what landed in it (`change/repairs.md`). While CI fails
  on the base's tip, the failure is
  not the change's: its queue is paused (`queues.md`), and the change
  waits for the branch's repair instead.
- **Gates carry over** a clean update temper made, where their
  `freshness` is clean (`change/gates.md`, 1); after a conflict an LLM resolved, they are
  asked for again.

## 2. Open questions

- **A late copy's hold** (`README.md`, 10): a head moved by a late copy of the
  change's own update, once the change has moved on, cannot be told from
  another's move by temper's identity, and holds. Keeping the writer slot
  past a settled update would spare it, for a time nothing bounds.
  Default: it holds, and is measured after landing.
- **Conflicts found before either side lands:** temper sees every open
  change, and could try them against each other and warn both goals.
