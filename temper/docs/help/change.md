# Changes

Kind: concept

A change is a task that writes one repository's branch and lands it: its pull request, CI at its head, its gates, the landing queue, its repairs and the merge, each step as change show prints it.

## What it is

- **A change task** is delegated with `change`, giving its words and,
  if the run may write several repositories, which one. Its coder
  reads those words alone (temper help task).
  With control, it also names `landing` (`goal`, a
  branch, or the default), `branch` (`mine`: its goal branch, landed
  whole), `after` (what it waits for: task numbers, or names from the
  same call), `merge` (a catch-up), `repairs` (the red landing's merge
  commit from its news: temper help tier), `changes` (several at once,
  all or none), and `questions`
  (`foreman`, `up`, or `none`, the default: how its producer's questions go).
- **Its pull request title** is its coder's, punctuation kept, cut at a
  UTF-8 boundary with “…” where the forge needs it.
- **Its panel,** as `change show` prints it: `change TASK ·
  OWNER/NAME on forge FORGE · BRANCH into BASE`, its pull request and
  its address, `step:` and the step, `way: turn · landing task N`
  where its project lands by turn, the CI at its head, its gates, and
  its repairs, resolutions and updates used against their limits.
  Next is `questions: ROUTE · TAKEN of LIMIT taken`, then each open
  question with its asker, number, first line and where it stands;
  with route `none` and nothing taken, `questions: none`.
- **Its result** is the change landed: its merge commit, its pull
  request with its address, a line `Landed: pull request #N merged into
  BASE as SHORT.`, then `Report:` and its coder's last report, then
  what the landing passed as the change saw it:
  `checks at HEAD:` each check at the landed head with its conclusion,
  `repairs: N:` why each repair was asked, and `reviews:`
  each verdict at the head it judged, cut from the end and marked when
  long. It names no check or test the change didn't see. Where tiers
  follow its landing branch (temper help tier), the result waits for
  their verdicts and adds a line, `Cleared by TIER run N.`, for a
  landing once red, `Cleared by TIER run N, which passed the failures
  blamed on it; repaired by COMMIT.` where a change naming it repaired
  it, or `Not cleared: no verdict within the verification bound.` Or it
  fails or is cancelled. A coder that concludes the request does not
  apply ends it successfully, saying why, with no pull request or
  claim of landing.

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
  changes, or the base broke it, its brief naming its pull request
  (number, branch, base, head and address) and holding the failed
  check and the end of its log, or the gate's remarks whole (cut only
  from their end, marked), and telling it to finish with the change,
  the pull request's new head; its words open with what it repairs
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
failed for good`, or `a person rejected a gate`. A release resets its
counts and resumes. After a coder fails for good before pushing,
release at `producing` makes a fresh coder, one more task. After a
repair fails for good, release at `repairing` makes a fresh repair
for the same reason, counted against its reset limit. Either holds
again if authority cannot cover one.
Held at `producing`, the hold and chat carry the coder's words whole
within 512 bytes: cancel if they hold, or release if the coder was
wrong. Its
panel also names a held task, the change's own or its writer's or
reviewer's, `held: task N (REASON)`, with that task's way out: an
effect vetoed or a step refused, and why (what it lacks, or the
forge's answer).

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
  A reviewer that cannot read what it needs fails saying why;
  `temper serve`'s stderr, unkept, names each refusal by task, tool
  and reason; Claude's Read denials are in the task's tool results.
- **Gates are temper's:** a change's gates run in temper, and `change
  show` prints each, `gate 1 · review · blocking · approved`; temper
  posts none to the forge as a review or a status, so the pull
  request's reviews there may be empty.
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
