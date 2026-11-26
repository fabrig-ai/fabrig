# A task's release

As a task closes, the forge connector releases what the task holds on
the forge: it settles the task's effects, reads again what its writers
left, comments on and closes its pull request where its ending asks, and
deletes, hands to its root or drops each branch it holds (jig's
`domain/tasks/holds.md`, 5). What each ending releases is its task's: a
change's in `change/README.md`, 4, a task's branches and saved work in
`branches.md`. A release's bound is `waits.md`, 8, and its place in the
outbox's line `room.md`, 3. A landing's round releases its pull request
and branches as the round ends (`../landing-prepare.md`, 1.1). This
document holds the two rules every release keeps, the round's among them
(`../../draft/landing-liveness.md`, 7): it finds what it releases and
its owner before it writes, and each failure carries its account.

## 1. It finds first

A release asked again, after a refusal, after a person's release of its
hold or after a restart, does no harm, because each resource is found,
with its owner, before anything is written for it:

- **Its own record:** the release kept for the closing task names the
  root that asked it and its ending. A held release takes another
  ending from that root, and a cancel takes over any other ending
  (`change/README.md`, 4).
- **A pull request** is found by temper's own record of it, as its last
  poll kept it (`change/README.md`, 4).
- **A branch** is found by the hold temper keeps of it: the task that
  holds it and the writer that writes it, read again where a run of it
  ended (`branches.md`).
- **A round's** pull request and branches are read afresh before the
  round's cleanup writes (`../landing-prepare.md`, 1.1).

Then:

- **Already released counts as released** when the owner it finds is
  the one releasing: a pull request closed or merged takes no close, a
  branch the forge no longer has counts as deleted (skein's
  `forge/client-effects.md`, 1), and a hold already dropped asks
  nothing. Nothing is written twice.
- **Another's is refused, with words:** a release asked by a root that
  is not its record's, or for an ending its record refuses; a branch
  still written by a writer once its writers are read again; a round's
  branch whose hold another task holds. Nothing is written for it, and
  the release fails with its account (2).

## 2. Its account

A release that fails holds its closing task for a failed effect (jig's
`domain/tasks/holds.md`, 5), and the hold's words are its account: the
operation, the resource and why. The forge hands the root those words
with every failure, and the write's outbox entry where the forge
answered one, so no failure reaches the root without words and no hold
says only that the forge could not release a task's resources. The root
holds the task with them as it holds any failed effect: by the entry
where there is one, and otherwise with a change's branch and the way
out added, as any procedure hold of its gets.

- **The shape** is an outbox entry's account (`effects.md`, 1):
  `OPERATION: WHY; resource REPOSITORY TARGET`, OPERATION and TARGET in
  the forge's effect words (`delete branch` and `branch B`, `close item`
  and `#N`, `post comment` and `#N`), REPOSITORY `owner/name`. A failure
  with no resource at stake is `release of task T: WHY`, T the closing
  task. A round's cleanup puts `cleanup: ` before its account
  (`../landing.md`, 4.2).
- **The forge's answer,** for a write the forge refused or failed: WHY
  is the answer in the effect words, so `delete branch: forge refused
  permission; resource acme/web branch temper/7/c42`.
- **temper's own cause,** for a failure before anything is written: WHY
  names it as its row says.

  | Cause | When | WHY |
  |---|---|---|
  | a release state that doesn't match | asked by another root, or for another ending while its release goes on unheld (1); no resource | `under way for root R as ENDING; asked by root R2 as ENDING2` |
  | a writer still holding | a branch of the task's still written once its writers are read again (1) | `still written by WRITER` |
  | a writer's reread failed | the reread a lost writer of the task's branch needs fails (`branches.md`) | `the reread after run A of task W failed: ANSWER` |
  | another's | a round's branch whose hold another task holds (1) | `held by task N` |
  | a refused write | a write that can never be sent (`room.md`, 6): its repository is context, or its adoption doesn't take the write's kind; its entry past the client's bounds; its comment past `forge.client.op_bytes`; its entry's number already kept | `not sent: REASON`, REASON one of `the repository is context`, `its adoption does not take kind K`, `past BOUND` with the figures, `outbox entry E is already kept` |
  | a limit | the table of releases is full; no resource | the limit's words, as any limit refusal's: its name in temper-limits, its figure and what this needs |
  | a record gone | its pull request to close, with no change kept for the task; no resource | `no change record for pull request #N` |

  ENDING is `done`, `cancelled` or `failed`; WRITER `outbox entry E`
  or `run A of task W`; ANSWER the forge's answer in its effect words.
- **A wait past its bound** keeps its words as `waits.md`, 9 gives them
  (`no progress at release for N minutes: ...`), and a wait for room
  past its bound as `waits.md`, 7.

## 3. Open questions

None.
