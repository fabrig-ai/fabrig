# Changes

Kind: concept

A change is a task that writes one repository's branch and lands it: its pull request, CI at its head, its gates, the landing queue, its repairs and the merge, each step as change show prints it.

## What it is

- **A change task** is delegated with `change`: words,
  and a repository when several may be written. Its coder reads only
  those words (temper help task).
  With control it also names `landing` (`goal`, a
  branch, or the default), `branch` (`mine`: its whole goal branch),
  `after` (task numbers or names from the same call it waits for), `merge` (a catch-up), `repairs` (the red landing
  commit from news: temper help tier), `changes` (one batch), `producer` (the charter number for its coder,
  repairs and resolutions; default: the forge's change charter) and
  `llm` (its `producer` and `review` choices, by configured model and
  effort names, defaulting to its charter's), and `questions`
  (`foreman`, `up`, or `none`, the default: how its producer's questions go).
- **Its pull request title** is its coder's, punctuation kept, cut at a
  UTF-8 boundary with “…” where the forge needs it.
- **Its panel,** as `change show` prints it: `change TASK ·
  OWNER/NAME on forge FORGE · BRANCH into BASE`, its pull request and
  its address, `step:` and the step, `way: turn · landing task N`
  for landing by turn, its CI, gates, and
  its repairs, resolutions and updates used against their limits.
  Next is `questions: ROUTE · TAKEN of LIMIT taken`, then each open
  question with its asker, number, first line and where it stands;
  with route `none` and nothing taken, `questions: none`.
- **Its result** names its merge commit, any tier verdict, then its pull
  request and address and `Landed: pull request #N merged into BASE as
  SHORT.`. `Report:` gives the coder's report, followed by what
  the landing passed: `checks at HEAD:` with conclusions,
  `repairs: N:` with repair reasons, and `reviews:` with each
  verdict at its judged head, cut from the end and marked when long.
  It names no check or test the change didn't see. Where tiers follow
  its landing branch (temper help tier), the result waits for them:
  `Cleared by TIER run N.`, or, for a landing once red, `Cleared by TIER
  run N, which passed the failures blamed on it; repaired by COMMIT.`
  where a change naming it repaired it, or `Not cleared: no verdict
  within the verification bound.` Verdicts precede the address
  to fit `task show TASK`'s text window. Or it fails or is
  cancelled. A coder finding no change applies ends
  successfully, saying why, with no pull request or claimed landing;
  failure for good holds it at `producing` (below).

## How it changes

Its steps, as `step:` prints them:

- `producing`: the coder writes the change; temper pushes its branch
  once it finishes. Failure holds it here with the coder's words;
  an explicit conclusion that no change applies ends it successfully;
- `opening`: temper opens its pull request;
- `checking`: the forge's CI runs at its head, where its landing
  branch's rules ask it;
- `gating`: its blocking gates give their verdicts (temper help
  landing-rule);
- `queued`: it waits its turn in its landing branch's queue, `queued:
  place N`;
- `first`: at the front, temper checks CI and the blocking gates again;
- `updating`: a branch behind its landing branch is updated by the
  forge, which merges the base in, where its rules ask up to date;
- `resolving`: the merge job, then an agent, settles a conflict with
  the base and pushes the merge commit;
- `repairing`: an agent repairs it, after CI failed at its head (a
  review still running there is cancelled first), a gate asked for
  changes, or the base broke it. Its brief names the pull request
  (number, branch, base, head and address), the failed check and log end,
  or the gate's remarks (cut from their end, marked). It finishes with
  the change's new head; its words open with what it repairs
  (`Repair CI at HEAD: CONTEXT`,
  `Repair gate N (KIND) at HEAD: changes requested`,
  `Repair the base's break at HEAD: BASE`),
  so `task show` names it; the change is checked again;
- `landing`, then `landed`: merged; `change show` names the commit;
  a change no tier follows ends there;
- `verifying`: awaiting tier verdicts;
- `red`: merge, tier run and full owned or suspected failing IDs
  (or `many tests`), until a later run passes them;
- `cleared`: all its tiers cleared it, and its result comes; with no
  verdict by the verification bound, it ends `landed, uncleared`.

With landing by turn (temper help landing), it has no CI at its head or queue:
`gating` follows `opening`, then:

- `ready`: its gates hold at its head, and its landing, `way: turn ·
  landing task N`, proves its candidate and waits for the turn;
- `preparing`: the forge rebases its branch onto main for its
  landing, and it is `ready` again at the new head;
- `repairing`: a gate failed at its candidate, given that run's
  output, and its new head makes it ready again;
- `landed`: main was fast-forwarded to its candidate.

After release, `reopening` reopens its pull request; `recreating`
pushes its missing branch again.

**Held,** `held: REASON at STEP`, it makes no progress: `its wait
stalled`, `its repairs are spent`, `its resolutions are spent`, `its
updates are spent`, `its branch was moved on the forge`, `its branch
is missing on the forge`, `its pull request was closed on the forge`,
`its pull request was retargeted on the forge`, `a delegate or a write
failed for good`, or `a person rejected a gate`. Release resets its
counts and resumes the held step. Failed coders and repairs get fresh
delegates for the same reason,
spending a task; repairs count against the reset repair limit.
It holds again if authority cannot cover one.
Held at `producing`, the hold and chat carry the coder's words whole
within 512 bytes: cancel if they hold, or release if the coder was wrong. Its
panel names any held task (the change, writer or reviewer),
`held: task N (REASON)`, and its way out: an effect vetoed or step
refused, naming what it lacks or the forge's answer.

## What temper guarantees

- **Landed at the head that passed:** temper merges naming the head
  whose CI and gates passed; a moved head or base is checked again
  first.
- **Nothing overwritten:** a branch or pull request changed by someone
  else on the forge holds the change.
- **Gates decide:** a blocking gate that asks for changes starts a
  repair; a person's rejection holds it; an advisory gate never blocks.
  A reviewer that gives no verdict is asked once more, then the change
  is held.
  A reviewer unable to read fails saying why. `temper serve`'s
  unkept stderr names each refusal's task, tool and reason; Claude's
  Read denials are in the task's tool results.
- **Gates are temper's:** a change's gates run in temper, and `change
  show` prints each, `gate 1 · review · blocking · approved`; temper
  posts none to the forge as a review or a status, so the pull
  request's reviews may be empty.
- **Bounded work:** past a limit of repairs, resolutions or updates,
  it holds.
- **The parent hears its result,** landed, and cleared where tiers follow its branch, or not.

## Acting on it

- **ctl:** `change show` reads its panel and waits for its pull
  request, its CI, a hold or its landing (temper help ctl); a wait for
  its landing ends at a hold, `not met: held` (exit 3); `task show`
  reads its task and its acts, `task release` among them; `task tree`
  shows its delegates; `landing show` reads its landing, where its
  project lands by turn.
- **temper's runs** ask for one with `change`, and hear landings on a
  branch with `subscribe_forge`.
- **The forge** holds its diff, CI's logs and any review a person posts
  there: read them there, at the pull request `change show` prints.

## Limits

- A change writes one repository, and lands on its landing branch.
- temper does not let ctl start a change: a run asks for one, as a
  chat's does when asked (temper help chat).
- `no change panel for task N` (exit 2): task N is no change.
- A release does not retarget back a pull request retargeted on the
  forge.
