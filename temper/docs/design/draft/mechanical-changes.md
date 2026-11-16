# mechanical-changes: a person opens a change from a branch they pushed, and temper lands it as any change

Under the turn, a landing branch moves only by its train's landing (`../domain/landing.md`, sections 8 and 9): a push by hand to main makes the round in flight stale, and the next landing forms as soon as main is free, so there is no quiet gap to push in. A branch whose landing needs no judgement, a docs branch for one, then has no way in but a goal: one whose agent copies the branch into its goal's branch and lands it, an LLM session, a change and a closer for work that is mechanical. This draft is the first step of a mechanical landing: a person pushes a branch, opens a change from it with one act, and temper lands it as it lands a change an agent produced, under the same rules. Nothing of the rules changes. What may be opened, and sending trouble back to the opener rather than to an agent, are later steps (section 11).

## 1. In one page

- **One act,** `temper ctl change open` (section 3): the project, the branch the person pushed, optionally the commit they mean, the branch to land into, a title and a budget.
- **A change task, the person's own** (section 4): the root expands the act into one task of the forge's change procedure, as it expands a run's `change` call, with the person's authority and budget in place of a run's; the person requests and funds it, as a goal they start. No goal and no producing task.
- **The forge is its producer** (section 5): the change's first state, taking, reads the branch's head once, checks it against the commit the act named, keeps it, and makes the change's own branch at it. No agent and no job.
- **Then an ordinary change** (section 6): its pull request opened by temper, its way the project's at its first readiness, the gates the rules covering its branch ask for, and its landing by turn or by queue, as any change's.
- **The pushed branch stays the person's:** nothing writes, holds or deletes it, and pushing it again changes nothing of the change.
- **Trouble, in this step, as any change's:** a conflict goes to a resolution and a red to a repair, under the forge's change charter.
- **The opener hears it** as a requester hears a task it made (section 7).
- **No new rule, setting or bound** (section 8).

## 2. Where it starts

- **A change's branch is one of three** (`../domain/forge/change/README.md`, 1): its own, pushed by its producing task; one handed to it already pushed, a chat's run branch (`../domain/forge/branches.md`); or its requester's held goal branch, which it lands whole as it stands, with no producer (`../domain/agent/tools.md`, 1, `"branch": "mine"`).
- **A producer need not be an agent:** a catch-up's is the merge job, given the commit to merge, and a goal's close step's is the close job (`../domain/forge/change/README.md`, 1; `../domain/forge/README.md`, section 6).
- **Only a run's `change` call, or the forge itself, makes a change today.** The root expands a run's call into the change's batch from the run's delegation context: its authority narrowed to the change's grants and charters, a share of its remaining tasks and its budget (`../domain/forge/change/README.md`, 1). A party may make a procedure task in a batch of its own, but only by naming the whole batch, spec, contract, authority and budget, and it holds nothing, so it hands nothing down (jig's `domain/tasks/README.md`, 4). No act of a client's expands a change for a party.
- **Docs-only branches** land today as goals of a template made for them, or by hand at a quiet moment before main landed by turn. An out-of-band fix is a hand push to main.

## 3. The act

```
temper ctl change open --project PROJECT [--repository OWNER/NAME] --branch BRANCH [--at COMMIT]
                       [--into BRANCH] --title TEXT [--body TEXT] (--budget AMOUNT | --as-allowed)
```

- **`--repository`:** the project's repository; left out, its only one.
- **`--branch`:** the branch the person pushed, in that repository. **`--at`:** the commit they mean, 40 or 64 hexadecimal digits; left out, the branch's head when the change takes it (section 5).
- **`--into`:** the landing branch; left out, the repository's default branch, as a run's call that names none. A branch under the repository's prefix, temper's own (`../domain/forge/branches.md`), is refused: a goal's branch as the target is later (section 11). A branch the repository lacks holds the change at its opening, the forge refusing its pull request, as a run's call naming one does (`../domain/agent/tools.md`, 1).
- **`--title`:** the pull request's title, fitted as a handed-in branch's title is (`../domain/forge/change/README.md`, 1). **`--body`:** more words. The change's words are `Opened by PERSON from BRANCH at COMMIT into BRANCH.`, the commit as given or `its head`, then a blank line and the body where one is given; its pull request's body is the words, as a change's with no body of its own.
- **`--budget` or `--as-allowed`:** what the change may spend, as `goal start` takes them: an amount, or what the person's role allows (`../client/ctl/areas.md`, area 2). Its reviews, resolutions and repairs spend it; the change itself spends nothing.
- **Its answer,** exit 0: `task N`, the change's task, which `change show` and `landing history --change` then read, as for any change. A replay under the same key answers the first, as every act does (`../client/ctl/tools.md`, 3).
- **Usage,** exit 1, nothing sent: `--branch`, `--title` or the budget's control missing, named together as `goal start` names its (`change open needs: --branch BRANCH; --title TEXT; --budget AMOUNT or --as-allowed`); a commit that is not 40 or 64 hexadecimal digits; a title or words past their bounds, a branch name past the forge's, each naming the bound.
- **Refused by the engine,** exit 2, nothing made, one line `temper ctl: change open: WHY`:
  - `project P has several repositories, R1, R2: name one with --repository`;
  - `R is not a repository of project P`;
  - `BRANCH is under R's prefix PREFIX: a change lands into one of its landing branches`;
  - the authority's and the budget's refusals in `goal start`'s words, jig's reason kept (`../client/ctl/areas.md`, area 2): a role that may not make the change, its task allowance or depth, a budget past the offer, or one too small for any run.
- **A change that fails later** was opened: the act answered `task N`, and the change ends as section 5 says, its words saying why.

## 4. The change it makes

- **A request of its own,** on the client's wire: the act's members as the person gave them. The root expands it into one change task as it expands a run's `change` call, by the same builder, with the person's party in place of the run's delegation context, and admits it as the person's batch (jig's `domain/tasks/README.md`, 4): the person is its requester and its funder.
- **Its authority,** the person's role's in the project, as their creating offer gives it, narrowed as a run's change is: its grants the repository's a change holds; its delegation to its producer's charter and its review charter, the person gate's role where a rule covering the branch asks for one, and the merge job where the role's delegation holds it; its task allowance a change's own; its depth a change's, within the role's. A role that holds less is refused, naming what it lacks.
- **Its budget,** the act's: the amount, or what the offer allows, reserved from the person's role's pool as a goal they start is (jig's `domain/authority.md`, section 7). One at or below the rules' least run spend is refused, since no review, resolution or repair could run.
- **Its parameters:** the repository and the landing branch, as the builder gives them; the title; one new parameter, the source, the branch's name and the commit `--at` named or none, in place of a handed-in branch's; no producer's charter, so its resolutions, repairs and checks' gates run under the forge's change charter and its agents' gates under its review charter, their numbers kept on its row as any change's (`../domain/forge/change/README.md`, 1); and its questions the project's route, as a change that names none. They count against jig's `tasks.parameters` as the builder counts a run's change, one fewer than a handed-in branch's, and a change past it is refused as the builder refuses one.
- **Nothing above it:** no goal, no goal branch, no coordinator and no closer. Its tree is the change alone until a review, a resolution or a repair delegates under it.

## 5. Taking the branch

- **Its own branch is held from admission:** the batch names `temper/<root>/c<task>`, as any change's names its own branch (`../domain/forge/branches.md`), and the hold is taken as the task becomes active, before the branch exists (jig's `domain/tasks/holds.md`, 2).
- **Taking is its first state,** before opening, where a produced change has producing:
  1. **Select:** read the source branch's head through the forge's reads, once. With `--at`, a head that is not that commit ends the change failed, `BRANCH is at HEAD, not COMMIT: push it and open again`; a branch the repository lacks ends it failed, `BRANCH is not in R`. Otherwise the head is kept on the change's row before anything is sent.
  2. **Make:** the change's existing effect that makes a branch, its own branch at the kept head, under temper's identity.
  3. **Taken:** the kept head is its pushed head, and it goes on as a change whose producer pushed it (`../domain/forge/change/README.md`, 2): a head its landing branch already contains ends it with no change, in that section's words; any other opens its pull request.
- **Never read twice:** after selecting, the change never reads the source branch again. A restart resumes from the kept head: the branch is made again, and one found already at the kept head counts as made.
- **A read or a make that errors** is retried as any of the change's effects are, the wait counted against the change's stall bound as its producing would be (`../domain/forge/change/repairs.md`). Past it, the change is held, `taking BRANCH: WHY`, and the hold reaches the person; released, it resumes where it stopped.
- **Cancelled while taking,** it ends as a cancel does, its own branch deleted where it was made (`../domain/forge/change/README.md`, 4).
- **temper's move, then temper's moves:** the branch was made by temper's identity, so its repairs', resolutions' and updates' pushes are the change's own, and a push to it by another identity is drift (`../domain/forge/change/README.md`, 3), as for any change. Drift reads heads and writers, not who authored the commits.
- **The pushed branch is never written, held or deleted.** The person may push it again or delete it; to land a newer head they open another change, and cancel this one or let it land first.

## 6. Its way to landing

- **As any change:** its way is its project's at its first readiness (`../domain/landing.md`, section 1).
  - **Under `turn`,** it gates on what the rules covering its branch ask for, is ready, waits for its train, joins its next landing and lands when the branch contains its prefix (`../domain/landing.md`, section 3). A rule asking CI and up to date alone, as a project's rule 1 under Draft first does, asks nothing a landing doesn't meet, so it is ready once its pull request is open.
  - **Under `queue`,** it goes the ordinary way: CI at its head, its gates, its turn in the queue, its updates and its merge (`../domain/forge/change/README.md`, 2).
- **The rules decide what it needs, unchanged:** a project on the stock's rule 1 asks it for gate 1, an agent's review, run under the review charter as any change's (`../protocol/configuration/stock.md`; `stock-ways.md`, section 2).
- **Trouble, in this step, as any change's** (`../domain/forge/change/repairs.md`): a conflict at prepare goes to a resolution, a red blamed on it or a review that says no to a repair, each an agent's run under the forge's change charter, within the change's bounds and its budget. Past them it is held, and the hold reaches the person as a hold of a task reaches its requester.
- **Its end:** landed, its branch deleted as a landed change's is; cancelled, by `task cancel`, it leaves its landing as a cancel does (`../domain/landing.md`, 4.3) and its branch is deleted; failed, its branch kept for a person to read until its tree's root closes, which is the change itself (`../domain/forge/branches.md`).

## 7. What the opener sees

- **The act's answer,** the change's task, then `change show`: its panel, its way, its landing and its head (`../domain/forge/change/README.md`, 5), with the source branch and the kept head while it takes and after.
- **Among the project's changes,** in the changes view and `landing history`, as a produced change is: what made it differs, not how it is shown.
- **Its news and holds** reach the person in their inbox, as those of a task they made do.

## 8. Settings and bounds

- **No new setting, rule or bound.** The words within jig's bounds of a task's spec, the parameters within `tasks.parameters`, the title within its 256 bytes, the branch's name within the forge's, the budget within the person's offer (section 3).
- **Who may open one** is who may make the change: the project's policy, through the person's role, as for any task they make. Narrowing it to some roles or some files is section 11's.

## 9. Upgrades

- **Stored:** temper's store is a registered family, so its version stays and nothing is patched (`../protocol/store.md`, section 4). The change's retained parameters gain the source, and its state the taking state with its kept head, each with a default: a change with no source is a change as today, so every stored change reads as it did. Nothing needs a written job.
- **The client's wire:** the new request, and the change's served document with its source and its taking state, move the client's wire's version, with no bridge (`redeploys.md`, section 8). A client of the new version meeting an engine of the previous one is refused by name, `version not served`, before any task is made, so an older engine never meets the source parameter.
- **Engine and workers:** nothing moves. No job runs and no worker is asked.

## 10. Acceptance

Focused and fuzzy suites alone.

1. **In the forge's world, under `turn`,** with a rule asking CI and up to date alone: a party opens a change from a pushed branch at its head into main; the change's own branch is made at that head by temper's identity, its pull request opened, it is ready with no gate asked, joins the next landing and lands; its own branch is then deleted, and the pushed branch is unchanged.
2. **The same under `queue`:** CI at its head, an update, the merge.
3. **Taking fails:** `--at` naming a commit other than the branch's head, and a branch the repository lacks, each end the change failed in section 5's words, with no branch made.
4. **Taking across a restart and a cancel:** a restart after selecting, before the make, makes the branch at the kept head, the source moved meanwhile; a restart after the make, before its answer, finds the branch and goes on; a read that errors past the stall bound holds the change, and its release resumes; a cancel while taking ends it, its own branch deleted where made.
5. **Already landed:** a head main contains ends the change with no change.
6. **Under the stock's rule 1:** the change waits for gate 1's review and lands once it approves.
7. **Trouble:** a conflict at prepare makes a resolution and a red blamed on it a repair, each under the forge's change charter, within the act's budget.
8. **The pushed branch:** pushed again after the change took it, nothing of the change moves; a push to the change's own branch by another identity holds it as drift.
9. **The root's expansion:** the change made from the act has the person as requester and funder, the authority, allowance, depth and budget of section 4, and the same parameters a run's change of the same branch would, but its source in place of a handed-in branch's; a role holding less, a budget too small and a change past `tasks.parameters` are each refused with nothing made.
10. **ctl:** the act's usage and bounds before sending, its answer, a replay under its key, and each of section 3's refusals in its words with its exit code.
11. **Fuzzy:** sequences of opens, cancels, restarts, pushes to the opened branches and landings under `turn`, with the trains' combine as `merge` and as `squash`, never write a pushed branch; every opened change's first pushed head is its kept head, and one that lands lands at its proved prefix, which contains that head under `merge` and has its merge's tree under `squash`, as any member's does (`../domain/landing.md`, section 8).
12. **Versions:** a client of the new version is refused by an engine of the previous one before any task is made.

## 11. Later

- **Trouble back to the opener:** a conflict, a red or a review's no goes to whoever opened the change, as a person task or in their inbox, never to a resolution or a repair, so that an opened change runs no LLM at all.
- **What may be opened:** a project setting naming the path sets of its classes an opened change must stay within and the roles that may open one, a docs set for one, checked at each head from the forge's comparison of the landing branch and the head (`../domain/forge/change/gates.md`, 2), every path within the set and the file list whole.
- **A goal's branch as the target,** for a person's change into a goal they hold.
- **Out-of-band fixes** through the act, on an owner's word, in place of a hand push.
- **The web:** opening a change from the Changes page.
- **A newer head in place:** moving an opened change to its branch's newer head without a new change.
