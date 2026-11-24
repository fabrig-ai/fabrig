# The landing turn and its trains

This document holds the landing turn and its trains, split from [`landing.md`](landing.md).

## 1. The turn

- **Held by the landing whose train it is,** for the seconds its merge
  takes: a train has one landing, so no other waits for its turn, and
  trains turn side by side.
- **First, freshness.** The train's tip and each member's head are
  read afresh. A head that moved makes its member leave (`landing.md`, 4.3). A tip
  that is not the round's base makes the round stale: a push from
  outside the landing moved the branch. The round ends unlanded (`landing-prepare.md`, 1.1),
  the stale count grows, and the landing prepares again in a new round,
  every member stacked afresh, keeping its receipts. These reads are
  the landing's reads: one that fails or finds its fact not there yet
  waits `read`, and a run of such waits the forge keeps answering while
  the turn stays where it was holds past the stall (`landing-waits.md`,
  3), as a round's reads at preparing do.
- **Then the branch fast-forwards to the round's proved prefix,** its
  candidate or, after a search, its green prefix, at which this
  section's candidate reads (`landing-search.md`, 5), by the merge of
  the round's pull request, fast-forward only, at the candidate (skein's
  `Write::Merge { how: FastForwardOnly }`), made by the landing task
  and judged by the landing rules for every member in the stack (`forge/change/gates.md`, 2, where a rule by path compares the train’s branch tip read afresh
  with the candidate, so it sees every member's paths, and `ci` is the
  test gates by receipt at the candidate; a gate's verdict counts at
  each member's prefix as `landing.md`, section 3 says, and an approval counts each
  member's pull request's reviews at the heads it counts at, the prefix
  being a clean merge of the member's head, or a squash with that
  merge's tree; the merge is admitted only where each member's
  requirements hold).
  On GitHub, the landing advances the branch with a fast-forward-only
  `Write::UpdateRef`; temper’s App must always bypass a rule requiring
  a pull request (`forge/adoption.md`, 4).
- **Every member lands** when the branch's tip reads back as the
  candidate, or as a commit containing it. In stack order, the landing records each landed at its
  prefix, waits for its pull request's merge mark until one configured
  poll after readback, then tells its change it landed at that prefix.
  Readback is authoritative: absent or delayed forge detection cannot
  strand the turn. The merge is the landing task's effect, but each
  landed commit belongs to its member change; its goal hears that
  change's landing as under `queue`. Remaining members above a blamed
  unit start the next round (`landing-prepare.md`, 1.1); otherwise the landing ends.
- **Its pull request reads merged:** a merged member's by the forge's
  detection (`landing.md`, section 10), with nothing written. A squashed member's
  head is in no branch, so the landing records its pull request merged
  at the prefix, the merge (kind 4) recorded at a commit its base
  already holds (skein's `Write::Merge` with `how` `Recorded { commit
  }`; Forgejo's `manually-merged`), before its change is told, since a
  change's branch deleted as it closes would close its pull request
  unmerged. A record refused holds the landing, `turn: record of #N:
  WHY`, its member recorded landed; a release records it again, and the
  members above it go on. A landing cancelled once its members are
  recorded landed tells each change it landed, its pull request as it
  stands. On GitHub, which records no
  manual merge, the landing closes a squashed member's pull request
  instead (kind 12).
- **Nothing runs inside the turn:** no gate, no prepare, no run of an
  agent's.
- **A merge not made is read again,** never held at once: the landing
  settles its own merge, and no outcome of it holds the task for its
  effect (`landing.md`, 4.2). An uncertain merge is the client's, looked for and made
  again within `forge.client.write_attempts`, each try waiting out its
  lifetime (skein's `forge/client-effects.md`, 1.4, 1.5), the turn waiting with
  it and reading its freshness meanwhile. A fast-forward-only merge
  raced by a push to the branch, a goal's design delivery among them
  (`forge/branches.md`, 1), is one: Forgejo answers it `500`, which the
  client keeps as a lost answer (skein's `forge/forgejo-operations.md`, 1). An
  outcome but made reads afresh the train's tip, whether it holds the
  candidate, and the round's pull request; one not known refused (lost
  past those tries, withdrawn before completion, held uncertain) is
  unknown, never failed:
  - the tip holds the candidate, or the pull request reads merged at
    it: every member lands, as by readback;
  - the tip is another than the round's base: the round is stale;
  - the pull request reads closed, not merged: the round ends unlanded
    and the landing prepares again in a new round, keeping its
    receipts, as after a release (`landing.md`, 4.2);
  - else, the tip at the base, an unknown merge is asked again while
    the round's merges number fewer than `forge.client.write_attempts`;
    a refusal, or the round's merges at that bound, holds: `turn: merge
    of #N: WHY; read #N STATE, BRANCH at TIP`, WHY the last outcome in
    its effect's words (`forge timed out`, `withdrawn before
    completion`), STATE `open` or `merged at COMMIT`, commits in
    hexadecimal at the forge's length. Its release
    prepares it again in a new round (`landing.md`, 4.2), and it holds its own train
    alone (section 2).
- **On GitHub,** which merges no pull request fast-forward, the turn
  would update the branch's ref to the candidate, fast-forward only
  (skein's `Write::UpdateRef`), under temper's App's `Always` bypass of
  a rule requiring a pull request (`forge/adoption.md`, 4); GitHub
  marks the pull requests merged once the branch holds their heads.
- **No turn without a fast-forward:** if temper cannot fast-forward
  to a candidate, the repository cannot land by turn. On GitHub, this
  includes a landing branch requiring a pull request without the App's
  `Always` bypass, or the forge's merge queue, which lands by its own
  method at an unproved commit. Refusing the project's `turn` names the
  repository and remedy: remove the queue or add the App to that
  branch's repository ruleset bypass list with "Always allow". There is
  no merge-commit fallback: the user's answer to the process model's
  open question, 2026-10-27, "for now if GitHub doesn't have the bypass,
  let's fail".

## 2. Trains

- **A train is a repository and the branch its changes land into:** a
  repository's main, or a goal's branch, each a train of its own.
- **Held** by its live landing from the decision that forms it to its
  end (`landing.md`, 4.2), so a train has one live landing at a time. Held is derived
  from the landings' phases, never stored: one landed or cancelled
  holds it no more, though its task may still follow its runs (`landing-proof.md`, 2.1)
  and delete its branches. A
  landing held for a person (`landing.md`, 4.2) holds its own train alone: other
  trains form, prove and turn beside it.
- **Waiting changes** are ready (`landing.md`, section 3) under turn and in no
  landing. Their order is `forge/queues.md`'s: the goal's priority, then
  the time each first became ready, which a change keeps while it stays
  live, with aging past the deployment's queue window, and the task's
  number last. Derived, not kept.
- **Forming is level-triggered:** whenever a train is free and has a
  waiting unit (`landing.md`, 4.1), every change of it waiting, a decision forms its
  next landing: the waiting units in order, each whole, up to the
  project's `batch` changes for a train into a repository's main, and
  one unit for a train into any other branch (`landing-settings.md`, 1.1). The first unit is
  always taken, even past `batch`. The first unit that would pass it
  beside those before it ends the batch and waits, first in line. A refused unit (`landing.md`, 4.1) is held
  and passed over. Forming is judged when a change becomes ready, when
  a landing ends, when a change's priority moves, and at a restart, for
  each train they touch.
- **A due step leaves no change out:** a waiting change whose own step
  is due (woken when a landing ends, or awaiting its step's forge read)
  joins the forming judged while it is due, in its place in line, as a
  quiet one does, and never holds it. Its readiness drops only with its
  head moving, and its landing reads each member's head as it joins: a
  moved one leaves (`landing.md`, 4.3).
- **Changes made ready by one event** (a dependency they share ending,
  one decision's gates) are waiting together: a change into the train
  under turn, in no landing and not yet ready, whose due step will make
  it ready, holds that train's forming while that step is due, so they
  form one landing, never the first alone, and no timer or delay stands
  in for that settling. Its step will make it ready when it is gating
  at its head, each blocking gate passed there or its gate's result come
  back for that step to take in, and no dependency of it pending
  outside its unit (`landing.md`, 4.1).
- **One step at most:** the hold lasts that one step. A change whose
  step ends with it not ready holds no more while it stays gating at
  that head, whatever wakes it again. The running engine remembers
  that, unstored, so after a restart such a change may hold for one more
  step. No other due step holds a train:
  not a change still checking or producing, gating with a gate
  outstanding, kept by a pending dependency, or already ready, whatever
  step made it so.
- **Each train on its own:** a change's step or wake holds only its own
  train's forming; other trains settle and form independently.
- **No race, no reservation:** a train's branch moves only by its
  landing's turn, or by a push from outside temper, which makes a round
  stale (section 1). Stale rounds are counted on the landing and
  reserve nothing.
- **Bounded:** `landings` counts live landings, one per train; a train
  with no room waits for one, as a change waits for a hold.

## 3. Open questions

None.
