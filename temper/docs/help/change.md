# Changes

Kind: concept

A change is a task that writes one repository's branch and lands it: its pull request, CI at its head, its gates, the landing queue, its repairs and the merge, each step as change show prints it.

## What it is

- **A change task** takes `change`'s words and repository. Its coder reads
  only those words (temper help task). With control: `landing` (`goal`, a branch, or the default), `branch` (`mine`: its whole goal branch), `after` (task numbers or names from the same call it waits for), `merge` (a catch-up),
  `repairs` (the red landing commit from news: temper help tier), `changes` (one batch), `producer` (the charter
  number for its coder, repairs and resolutions; default: the forge's change charter), `llm` (`producer` and `review`
  model and effort names; default: its charter's) and `questions` (`up` or `none`: how its producer's questions go;
  default: the project's route, `none` in the stock).
- **A producer going on after an answer** keeps its work: what it delivered is on its branch, and what it had not was saved as a commit when it parked, so those edits are in its head and `git status` shows none of them. Only a worker's death, or parking with a conflict marker left, loses that run's own edits; earlier saved work stands. If its landing branch moved, temper merges the new tip in before the run starts. A clean merge is committed: its head is a commit titled `Merge SHA`, over a clean tree that holds other tasks' files. A conflict stays a merge in progress: `HEAD` is its work, `MERGE_HEAD` the tip, other tasks' files are staged, and its start names the files in conflict. Neither is contamination: it settles what is named and carries on, and its delivery commits the lot as the merge. Never answer it to remove or restore those files: that takes other tasks' work out of the merge.
- **Its title:** the coder's, punctuation kept, cut on UTF-8 with “…”.
- **Its panel:** task, repository, branch, base, pull URL, step, CI, gates, repair/update/resolution limits;
  `way: turn · landing task N`; questions' route, count, words and standing. `read_task` gives its answer number.
  `CI not required` means its repository or rules ask for none. JSON separates `ci_required` from `ci`, the head observation retained once ended.
- **Approvals:** rule, role, freshness, required/remaining counts, people and heads; unread and incomplete differ from zero. Unmapped accounts do not count. An approval wait's `next` points to the pull request.
- **Its result** names the merge commit, tier verdict, pull request and
  address, and `Landed: pull request #N merged into BASE as SHORT.`.
  With repairs, `repairs: N:` lists reasons and successful reports by
  task, in completion order. `Producer report (historical; before repairs):`
  marks the coder's pre-repair report (`Report:` without repairs). `checks at HEAD:` gives conclusions and `reviews:`
  verdicts at their heads, with `repairs: 0` between them without repairs.
  End cuts are marked; no unseen check or test is named. On tiered branches (temper help tier), the result waits for
  `Cleared by TIER run N.`, or, after a red landing, `Cleared by TIER run N, which passed the failures blamed on it; repaired by COMMIT.`
  where a change naming it repaired it, or `Not cleared: no verdict within the verification bound.` Verdicts fit before the address in
  `task show TASK`. Or it fails or is cancelled. No change applies: ends
  successfully with why, no pull request or landing.

## How it changes

As `step:` prints it:

- `producing`: the coder writes the change; temper pushes its branch
  once it finishes. Failure holds it here. With dependencies pending,
  it may start early from their pushed heads. A head that does not merge
  ends that producer `Cancelled · its early start conflicts with SHA`;
  nothing holds: once they are done, the change goes on with a new producer;
- `opening`: temper opens its pull request;
- `checking`: the forge's CI runs at its head, where its landing
  branch's rules ask it;
- `gating`: its blocking gates give their verdicts (temper help landing-rule);
- `queued`: it waits its turn in its landing branch's queue, `queued: place N`;
- `first`: at the front, temper checks CI and the blocking gates again;
- `updating`: a branch behind its landing branch is updated by the
  forge, which merges the base in, where its rules ask up to date;
- `resolving`: the merge job (its command: temper help configuration),
  then an agent, settles a conflict with the base and pushes the merge
  commit;
- `repairing`: an agent repairs it, after CI failed at its head (a
  review still running there is cancelled first), a gate asked for
  changes, or the base broke it. Its brief names the pull request
  (number, branch, base, head and address), the failed check and log end,
  or the gate's remarks (cut from their end, marked). It finishes with
  a new head; its words open with the repair
  (`Repair CI at HEAD: CONTEXT` (`no failed check named` if unknown), `Repair gate N (KIND) at HEAD: changes requested`; a clean update's red opens as CI's),
  so `task show` names it; the change is checked again;
- `landing`, then `landed`: merged; `change show` names its commit;
  without tiers, it ends;
- `verifying`: awaiting tier verdicts;
- `red`: merge, tier run and full owned or suspected failing IDs
  (or `many tests`), until a later run passes them;
- `cleared`: all its tiers cleared it, and its result comes; with no
  verdict by the verification bound, it ends `landed, uncleared`.

With landing by turn (temper help landing), it has no CI at its head or queue:
`gating` follows `opening`, then:

- `ready`: its gates hold at its head, and its landing proves its
  candidate and waits for the turn;
- `preparing`: the forge rebases its branch onto main for its
  landing, and it is `ready` again at the new head;
- `repairing`: a gate failed at its candidate, given that run's
  output, and its new head makes it ready again;
- `landed`: main was fast-forwarded to its candidate.

After release, `reopening` reopens its pull; `recreating` pushes its missing branch.

**Held,** `held: REASON at STEP`, it makes no progress: `its wait
stalled`, `its repairs are spent`, `its resolutions are spent`, `its
updates are spent`, `its branch was moved on the forge`, `its branch
is missing on the forge`, `its pull request was closed on the forge`,
`its pull request was retargeted on the forge`, `a delegate or a write
failed for good`, or `a person rejected a gate`. Release resets counts
and resumes the held step. Failed coders and repairs get fresh delegates for that reason, spending a task;
repairs use the reset limit. Without authority for one, it holds.
Cancelled delegates hold likewise; their tasks say so.
Held at `producing`, the hold and chat carry the coder's words whole
within 512 bytes: cancel if they hold, or release if the coder was wrong. Its
panel names any held task (the change, writer or reviewer),
`held: task N (REASON)`, and its way out: an effect vetoed or step
refused, naming what it lacks or the forge's answer.

## What temper guarantees

- **Landed as checked:** temper merges the exact head after required CI and gates pass; it rechecks a moved head or base.
- **Nothing overwritten:** a branch or pull request changed by someone
  else on the forge holds the change.
- **Gates decide:** a blocking gate that asks for changes starts a
  repair; a person's rejection holds it; an advisory gate never blocks.
  No verdict: ask once more, then hold.
  A reviewer unable to read fails saying why. `temper-box serve`'s
  unkept stderr names each refusal's task, tool and reason; Claude's
  Read denials are in the task's tool results.
- **Gates are temper's:** they run in temper; `change show` prints each, `gate 1 · review · blocking · approved`. Temper posts none as forge reviews or statuses; the pull's reviews may be empty.
- **The parent hears its result,** and whether tiers cleared it.

## Acting on it

- **ctl:** `change show` reads its panel and waits for its pull request,
  CI, a hold or landing (temper help ctl); landing waits end at a hold:
  `not met: held` (exit 3). `task show` reads and acts on its task, including
  `task release`; `task tree` shows delegates; `landing show` reads a
  project's landing by turn.
- **Runs** ask with `change`, and hear branch landings with `subscribe_forge`.
- **The forge** holds its diff, CI logs and people's reviews at the pull
  request `change show` prints.

## Limits

`temper help configuration`: what needs a restart.

- A change writes one repository, and lands on its landing branch.
- Batches share task room within policy caps; minimum two tasks per change.
  Refusals give capacity and whether fewer changes fit. Profiles,
  overrides and lifetime counts: temper help limits-engine.
- Only runs start changes, as chats do when asked (temper help chat).
- `no change panel for task N` (exit 2): task N is no change.
- Release keeps a pull request's target as retargeted on the forge.
