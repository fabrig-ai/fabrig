# mechanical-changes: a person opens a change from a branch they pushed, and temper lands it as any change

Under the turn, a landing branch moves only by its train's landing (`../domain/landing.md`, sections 8 and 9): a push by hand to main makes the round in flight stale, and the next landing forms as soon as main is free, so there is no quiet gap to push in. A branch whose landing needs no judgement, a docs branch for one, then has no way in but a goal: one whose agent copies the branch into its goal's branch and lands it, an LLM session, a change and a closer for work that is mechanical. This draft is the first step of a mechanical landing: a person pushes a branch, opens a change from it with one act, and temper lands it as it lands a change an agent produced, under the same rules. Nothing of the rules changes. What may be opened, and sending trouble back to the opener rather than to an agent, are later steps (section 11).

## 1. In one page

- **One act,** `temper ctl change open` (section 3): the project, the branch the person pushed, optionally the commit they mean, the branch to land into, and a title.
- **A change task, the person's own** (section 4): one task of the forge's change procedure, made in the person's own batch, requested and funded by them, as a goal they start is. No goal and no producing task.
- **The forge is its producer** (section 5): the change's first step reads the branch's head, checks it against the commit the act named, and makes the change's own branch at it. No agent and no job.
- **Then an ordinary change** (section 6): its pull request opened by temper, its way the project's at its first readiness, the gates the rules covering its branch ask for, and its landing by turn or by queue, as any change's.
- **The pushed branch stays the person's:** nothing writes, holds or deletes it, and pushing it again changes nothing of the change.
- **Trouble, in this step, as any change's:** a conflict goes to a resolution and a red to a repair, under the forge's change charter.
- **The opener hears it** as a requester hears a task it made (section 7).
- **No new rule, setting or bound** (section 8).

## 2. Where it starts

- **A change's branch is one of three** (`../domain/forge/change/README.md`, 1): its own, pushed by its producing task; one handed to it already pushed, a chat's run branch (`../domain/forge/branches.md`); or its requester's held goal branch, which it lands whole as it stands, with no producer (`../domain/agent/tools.md`, 1, `"branch": "mine"`).
- **A producer need not be an agent:** a catch-up's is the merge job, given the commit to merge, and a goal's close step's is the close job (`../domain/forge/change/README.md`, 1; `../domain/forge/README.md`, section 6).
- **Only a run's `change` call, or the forge itself, makes a change today,** the root filling in from the project what the call leaves out (`../domain/forge/change/README.md`, 1). A party may make tasks in a batch of its own, a procedure's among them, and holds nothing, so it hands nothing down (jig's `domain/tasks/README.md`, 4).
- **Docs-only branches** land today as goals of a template made for them, or by hand at a quiet moment before main landed by turn. An out-of-band fix is a hand push to main.

## 3. The act

```
temper ctl change open --project PROJECT [--repository OWNER/NAME] --branch BRANCH [--at COMMIT]
                       [--into BRANCH] --title TEXT [--body TEXT]
```

- **`--repository`:** the project's repository; left out, its only one. A project with several refuses its absence, naming them.
- **`--branch`:** the branch the person pushed, in that repository. **`--at`:** the commit they mean, 40 or 64 hexadecimal digits; left out, the branch's head when the change takes it (section 5).
- **`--into`:** the landing branch; left out, the repository's default branch. A branch under the repository's prefix, temper's own (`../domain/forge/branches.md`), is refused, naming it: a goal's branch as the target is later (section 11).
- **`--title`:** the pull request's title, fitted as a handed-in branch's title is (`../domain/forge/change/README.md`, 1). **`--body`:** its body; left out, the change's words. The change's words name what the person gave: `Opened by PERSON from BRANCH at COMMIT into BRANCH.`, the commit as given or `its head`.
- **Its answer:** `task N`, the change's task, which `change show` and `landing history --change` then read, as for any change. A replay under the same key answers the first, as every act does (`../client/ctl/tools.md`, 3).
- **Refused before anything is sent,** naming the bound: a title or a body past its bound, a branch name the forge's bounds refuse, a commit that is not 40 or 64 hexadecimal digits. **Refused by the engine,** in the act's words: a repository not the project's, a landing branch under the prefix, a role whose authority can't make the change (section 4).

## 4. The change it makes

- **One procedure task in the person's own batch:** the forge's change procedure, in the person's project, requested by the person and funded as a goal they start is (jig's `domain/authority.md`, section 7). A role whose authority can't make it is refused as jig refuses any batch, naming what it lacks (jig's `domain/tasks/README.md`, 4).
- **Its parameters:** the repository and the landing branch; the title and the words; questions `none`; no producer's charter, so its repairs, resolutions and checks' gates run under the forge's change charter and its agents' gates under its review charter, as a change made with none does (`../domain/forge/change/README.md`, 1); and one new parameter, the branch to take, with the commit `--at` named, or none.
- **The rest, as for a run's call:** what the root fills in for a run's `change` from the project and its change policy, its bounds, its charters' numbers kept on its row and the procedure's delegation to them, the forge fills in the same way for a party's change, in the same decision, so the act names only what the person gives.
- **Nothing above it:** no goal, no goal branch, no coordinator and no closer. Its tree is the change alone until a review, a resolution or a repair delegates under it.

## 5. Taking the branch

- **The first step, in place of producing:** the change reads the branch's head through the forge's reads. With `--at`, a head that is not that commit ends the change failed, `BRANCH is at HEAD, not COMMIT: push it and open again`; a branch the repository lacks ends it failed, naming it. Nothing was made, so nothing is kept.
- **Then the forge makes the change's own branch,** `temper/<root>/c<task>` as any change's (`../domain/forge/branches.md`), at that head, as the change's effect, the way a goal's own branch is made at its base's tip as its hold is taken. The change holds it from its making, as it holds any own branch.
- **That head is its pushed head:** from there it goes as a change whose producer pushed it (`../domain/forge/change/README.md`, 2). A head its landing branch already contains ends it with no change, in that section's words; any other opens its pull request.
- **temper's move, then temper's moves:** the branch was made by temper's identity, so its repairs', resolutions' and updates' pushes are the change's own, and a push to it by another identity is drift (`../domain/forge/change/README.md`, 3), as for any change.
- **The pushed branch is never written, held or deleted,** and never read again. The person may push it again or delete it; to land a newer head they open another change, and cancel this one or let it land first.

## 6. Its way to landing

- **As any change:** its way is its project's at its first readiness (`../domain/landing.md`, section 1). Under `turn`, it gates on what the rules covering its branch ask for, is ready, waits for its train, joins its next landing and lands when the branch contains its prefix (`../domain/landing.md`, section 3). Under `queue`, it goes through CI at its head, its updates and its merge (`../domain/forge/change/README.md`, 2).
- **The rules decide what it needs, unchanged:** a rule asking CI and up to date alone, as a project's rule 1 under Draft first does, asks it for nothing its landing doesn't meet, so it is ready once its pull request is open; a project on the stock's rule 1 asks it for gate 1, an agent's review, run under the review charter as any change's (`../protocol/configuration/stock.md`; `stock-ways.md`, section 2).
- **Trouble, in this step, as any change's** (`../domain/forge/change/repairs.md`): a conflict at prepare goes to a resolution, a red blamed on it or a review that says no to a repair, each an agent's run under the forge's change charter, within the change's bounds. Past them it is held, and the hold reaches the person as a hold of a task reaches its requester.
- **Its end:** landed, its branch deleted as a landed change's is; cancelled, by `task cancel`, it leaves its landing as a cancel does (`../domain/landing.md`, 4.3) and its branch is deleted; failed, its branch kept for a person to read until its tree's root closes, which is the change itself (`../domain/forge/branches.md`).

## 7. What the opener sees

- **The act's answer,** the change's task, then `change show`: its panel, its way, its landing and its head (`../domain/forge/change/README.md`, 5).
- **Among the project's changes,** in the changes view and `landing history`, as a produced change is: what made it differs, not how it is shown.
- **Its news and holds** reach the person in their inbox, as those of a task they made do.

## 8. Settings and bounds

- **No new setting, rule or bound.** The title and the words are within the change's own bounds; the branch's name within the forge's.
- **Who may open one** is who may make the change: the project's policy, through the person's role, as for any task they make. Narrowing it to some roles or some files is section 11's.

## 9. Upgrades

- **The change's new parameter** is a stored format's change where the forge keeps it on the change's row: its version moves, with its patch.
- **The act's request** is jig's batch, which the client already sends; an engine of the previous version refuses the new parameter by name, and a client of another version is refused by name as its wire's version says (`redeploys.md`).

## 10. Acceptance

Focused and fuzzy suites alone.

1. **In the forge's world, under `turn`,** with a rule asking CI and up to date alone: a party opens a change from a pushed branch at its head into main; the change's own branch is made at that head by temper's identity, its pull request opened, it is ready with no gate asked, joins the next landing and lands; its own branch is then deleted, and the pushed branch is unchanged.
2. **The same under `queue`:** CI at its head, an update, the merge.
3. **Taking fails:** `--at` naming a commit other than the branch's head, and a branch the repository lacks, each end the change failed in section 5's words, with no branch made.
4. **Already landed:** a head main contains ends the change with no change.
5. **Under the stock's rule 1:** the change waits for gate 1's review and lands once it approves.
6. **Trouble:** a conflict at prepare makes a resolution and a red blamed on it a repair, each under the forge's change charter.
7. **The pushed branch:** pushed again after the change took it, nothing of the change moves; a push to the change's own branch by another identity holds it as drift.
8. **Refusals:** a repository not the project's, a landing branch under the prefix, a role that can't make the change, each in its words, with nothing made.
9. **ctl:** the act's parsing and its bounds before sending, its answer, a replay under its key, and each refusal's words and exit code.
10. **Fuzzy:** sequences of opens, cancels, pushes to the opened branches and landings under `turn` never write a pushed branch, and every opened change that lands has its taken head in its landed commit.

## 11. Later

- **Trouble back to the opener:** a conflict, a red or a review's no goes to whoever opened the change, as a person task or in their inbox, never to a resolution or a repair, so that an opened change runs no LLM at all.
- **What may be opened:** a project setting naming the path sets of its classes an opened change must stay within and the roles that may open one, a docs set for one, checked at each head from the forge's comparison of the landing branch and the head (`../domain/forge/change/gates.md`, 2), every path within the set and the file list whole.
- **A goal's branch as the target,** for a person's change into a goal they hold.
- **Out-of-band fixes** through the act, on an owner's word, in place of a hand push.
- **The web:** opening a change from the Changes page.
- **A newer head in place:** moving an opened change to its branch's newer head without a new change.
