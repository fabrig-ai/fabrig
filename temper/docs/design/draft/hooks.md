# A repository's hooks

Draft, 2026-11-27, from a session with the user on 2026-11-27. It takes the hooks out of `ways.md` (sections 8.4, 10 and 11), so that they can be built on their own while that draft may still change: "let's extract a separate draft for this change only". The user simplified them on the way: "all hooks are ran from the version of their branch". Once everything runs a build that reads hooks, a later pass removes the old paths and settings: "a followup pass for removing all this logic and the old config entirely". It gives `finish` a hook of its own beside `deliver`, since they are two moments: "an explicit 'finish' hook, which for us in fabrig would be just a symlink to deliver, but in principle a user might want to differentiate the 2". Each hook also says how long it may run, in `.temper/hooks/` itself: "every hook should have a timeout, and maybe that could be configured directly in .temper/hooks", fabrig's `close` at 1,800 seconds (the user, 2026-11-27). It revises the delivery check's convention (`../domain/agent/charter.md`, section 4) and the project's `merge_job` and `close_job` (`../domain/landing-settings.md`, 1.1; `../protocol/tooling.md`, 4.1 and 4.2), and it moves fabrig's scripts. It wins over `ways.md` where they differ.

## 1. In one page

- **Four hooks, named by when they run:** `.temper/hooks/deliver`, `.temper/hooks/finish`, `.temper/hooks/merge` and `.temper/hooks/close`. Present, a hook runs. Missing, that moment has none. No setting names a path.
- **Each runs from its own branch's version** (section 3): the one its job checks out, as today's scripts do. A change can edit any hook, so no hook is what must hold: that is CI and the landing rules.
- **How long each may run** (3.4): `.temper/hooks/config.toml` says it per moment, under a ceiling the project's owners set. Unset, today's bounds hold.
- **By repository** (section 4): a repository with a `.temper/hooks/` directory runs hooks alone. One without it runs today's `.temper/pre-pr`, `merge_job` and `close_job`.
- **fabrig moves its scripts to hooks** (section 6), and keeps the old paths as wrappers while any build serving it reads them.
- **Then the old ones go** (section 5): once dev.fabrig.ai runs a build that reads hooks, temper stops reading the old paths and settings, and fabrig drops its wrappers.

## 2. Today

- **The delivery check** is `.temper/pre-pr`, a path temper's conventions give smith with the guide's (`../domain/agent/charter.md`, section 4). smith looks for it in each writable directory when the run starts, and runs it on the run's checkout before each delivery: at a `deliver` and at a `finish` that delivers. A failure refuses the delivery, and a `save` runs none (smith's `domain/run-delivery.md`, 1.1).
- **The merge job** is the project's `merge_job`, a path in each repository it merges. Set, a conflict goes to the merge job first, run in the merge in progress with the head checked out and the base merged in. Unset, a conflict goes to a resolution at once (`../domain/landing-settings.md`, 1.1; `../protocol/tooling.md`, 4.1).
- **The close job** is the project's `close_job`, a path in the goal's branch's repository. Set, the goal's closer makes a close change into the goal's branch before it lands whole, its job run at the branch's tip (`../domain/forge/branches.md`, section 2). A close may check what it settles and refuse (`../protocol/tooling.md`, 4.2).
- **fabrig's `.temper/`** holds `pre-pr`, `merge` and `close`, and `checked-close`, which settles with `close` and then runs pre-PR's close check. Two more are run by no moment of temper's: `goal-pre-pr`, a completed goal's check, for CI's runner (`.forgejo/workflows/ci.yml`), and `goal-child-pre-pr`, an explicit check of a goal's child (`tools/pre-pr/design.md`).
- **Git's own hooks are another thing:** a worker's Git runs none, from any branch (`../protocol/git.md`, section 3), and that stays.

## 3. The hooks

### 3.1 By moment

| Moment | When it runs | What it's given, and its exit |
|---|---|---|
| `deliver` | on the run's checkout, before a `deliver`'s push, after which the run goes on | as `.temper/pre-pr` is today (`../domain/agent/charter.md`, section 4): `refs/temper/landing` pinned; 0 passes, anything else refuses the delivery, its output's tail the feedback |
| `finish` | on the run's checkout, before the push of a `finish` that hands in a change, which ends the run once it lands | as `deliver` |
| `merge` | in a merge in progress, on a conflict | as `merge_job`'s command is (`../protocol/tooling.md`, sections 2 and 4.1) |
| `close` | once a goal's changes have ended, before its branch lands whole | as `close_job`'s command is (`../protocol/tooling.md`, sections 2 and 4.2) |

- **`deliver` and `finish` are two moments, each with its own hook:** a `deliver` pushes checked work and the run goes on; a `finish` that hands in a change pushes it and ends the run. Each runs its own hook and only that one, so a project checks the two differently or, linking one to the other, the same. A `save` runs none, and neither does a `finish` with a report, a failure or a verdict. What follows a `finish`, review, CI and the landing, is temper's rather than the run's, and its hooks are `merge` and `close`.
- **A hook is an executable file** at `.temper/hooks/<moment>`, or a symbolic link to another hook beside it, its target a bare name in the same directory, as `finish` to `deliver`. Any other link, or a file that isn't executable, is the hook's failure, not its absence.
- **Present, it runs. Missing, that moment has none:** a repository with no `merge` hook sends a conflict to a resolution at once, one with no `close` hook gets no close change, and one with no `finish` hook pushes a finished change unchecked, for CI to judge.
- **Each file's head says its contract:** its moment, what it's given and its exit codes, pointing at `../protocol/tooling.md` or the check's convention, so that a person reading the directory needs nothing else.
- **Bounded as tooling is:** each runs with no credential and reaches no forge. Its time, its output's size and its result's shape are bounded and checked as `../protocol/tooling.md`, sections 1, 2 and 6 say, and `deliver` and `finish` keep the check's bound (`../domain/agent/charter.md`, section 4).
- **The repository's, for every piece of work:** all work delivers and lands through the same hooks, and only a change to the file turns one off.

### 3.2 From its own branch

- **Each runs from the version its job checks out** (the user, 2026-11-27): `deliver` and `finish` from the run's checkout, `merge` from the merge in progress, `close` from the goal's branch at its tip. That is today's rule for each script, kept.
- **So a hook is the work's own code.** A change can edit any hook, and what it calls, in its own checkout, as it can edit `.temper/pre-pr` today:
  - `deliver` and `finish` are feedback, as smith treats a check (smith's `domain/run-delivery.md`, 1.1).
  - `merge` makes a candidate, which CI and the gates judge again at its head. It doesn't judge.
  - `close` settles and may refuse. Its refusal is the project's own check, which a goal's changes could loosen as they could loosen `deliver`'s.
- **What must hold is CI and the landing rules.** A project that wants a person to see any change to its hooks, or to the tools they call, says so with a rule by path (`what-reaches-you.md`, section 7) naming `.temper/hooks/` and those tools.

### 3.3 How temper finds them

- **`deliver` and `finish`: smith finds them** in each writable directory when the run starts, as it finds the check today (smith's `domain/run-start.md`, 1.1, and `domain/run-delivery.md`, 1.1). temper's conventions name a check for each moment, `.temper/hooks/deliver` and `.temper/hooks/finish`, where today they name one for both, and, while section 4 holds, `.temper/pre-pr` beside them, taken for both only in a directory with no `.temper/hooks/`. The brief names the checks smith found, and which tool runs each.
- **`merge` and `close`: temper reads `.temper/hooks/`** at the commit the job will start from, with one read of the directory, as the forge reads the CI workflows' directory (`../domain/forge/adoption.md`, section 11). For `close`, that is the goal's branch's tip once its changes have ended. For `merge`, it is the head the merge is made on, when a conflict is found. An entry named for the moment is its hook, a link's entry among them.
- **The job's checkout decides the rest:** a hook the read found that the job's checkout doesn't hold as a regular executable file is the job's failure, as an absent command is today (`../protocol/tooling.md`, 4.1 and 4.2).
- **A read refused or failed is no answer:** the step waits and reads again. It never takes a failed read as no hook, which would skip a close's check.

### 3.4 How long each may run

- **The repository says it,** in `.temper/hooks/config.toml`: a table per moment, each with `timeout_seconds`, whole seconds with the unit in the key's name, as temper writes its other figures (`../protocol/configuration/file.md`).

  ```toml
  # .temper/hooks/config.toml: how long each hook may run
  [close]
  timeout_seconds = 1800
  ```

  It names no command and turns no hook on: a hook is still its file (3.1), and a table for a moment with no hook means nothing.
- **Unset, today's bound holds:** for `deliver` and `finish`, the check's bound, the project's `check_timeout_ms` or the agent's own, with its one retry at twice the bound (`../domain/agent/charter.md`, section 4); for `merge` and `close`, their kinds' deadlines (`../protocol/tooling.md`, sections 2, 4.1 and 4.2), ten minutes for `close`.
- **Never past the project's ceiling:** `hook_timeout_max_seconds`, a project landing setting beside `check_timeout_ms` (`../domain/landing-settings.md`, 1.1), 3,600 unless an owner sets it. A value above it runs at the ceiling, and the hook's run says so. The file is the work's own, as a hook is (3.2), so the ceiling, which spends a worker's time, stays in settings, where landing can't reach it.
- **Read where the hook runs,** from the same version (3.2): temper's worker reads the file in the checkout it prepared, when a run starts for `deliver` and `finish`, and before the job's command for `merge` and `close`, and bounds each value by the ceiling the engine sends. The job's deadline follows the ceiling, so that the hook's own timeout, not the job's, ends a hook that runs long.
- **A file that doesn't parse,** a value of 0, or a key it doesn't know is the hook's failure, named with its line, as an unreadable hook is.

## 4. By repository, while the old ones stay

- **A repository with `.temper/hooks/`** at the commit read runs hooks alone. A missing hook means none, `.temper/pre-pr` isn't run, and the project's `merge_job` and `close_job` are ignored for that repository.
- **A repository without it** runs as today: `.temper/pre-pr` for `deliver` and `finish`, `merge_job` for `merge` and `close_job` for `close`, each setting's presence switching its job on.
- **Wrappers for older builds:** a repository moving to hooks keeps its old scripts as wrappers that call the hooks. An older build, which reads only the old paths and settings, then runs the same code, and going back to one finds them.
- **Older agents and workers** follow `../protocol/wires.md`, sections 5 and 8: an agent whose layout lacks the hooks' paths is sent `.temper/pre-pr` alone, and runs the wrapper at both moments. A repository that keeps no wrapper gets no check from an older agent.

## 5. Then the old ones go

Once dev.fabrig.ai, its engine and its workers' agents run a build that reads hooks (the user, 2026-11-27):

- **temper stops reading the old ones.** It no longer reads `.temper/pre-pr`, `merge_job`, `close_job` or `check_timeout_ms`, which `config.toml` replaces (3.4): a stored one reads as none (`../protocol/wires.md`), and `temper ctl` refuses either by name. Section 4's rule goes with them: every repository runs hooks alone, and one without `.temper/hooks/` has none.
- **fabrig's wrappers go:** `.temper/pre-pr`, `merge`, `close` and `checked-close`.
- **`goal-pre-pr` and `goal-child-pre-pr` leave `.temper/` for `tools/pre-pr/`,** so that `.temper/` holds only what temper runs. Whatever calls them from outside the repository, CI's runner wrapper among them, moves to the new path first, a step of ops' by hand. The gate's scope (`.forgejo/scripts/gate_scope.py`), pre-PR's own list of hooks and their tests follow.

## 6. fabrig and dev.fabrig.ai

- **fabrig's repository,** with the first increment:
  - `.temper/hooks/deliver` is today's `pre-pr`, `.temper/hooks/finish` is a link to it, and `.temper/hooks/merge` is today's `merge`.
  - `.temper/hooks/close` is today's `checked-close`, with `close`'s settling folded in: it settles, then runs pre-PR's close check. It is the one close every deployment serving fabrig runs, the reference's among them.
  - `.temper/hooks/config.toml` gives `close` 1,800 seconds, which dev.fabrig.ai's builds set by a patch until then (3.4).
  - `pre-pr`, `merge`, `close` and `checked-close` stay as wrappers (section 4).
  - The reference profile (`../protocol/configuration/reference.md`) names neither `merge_job` nor `close_job`.
  - The gate's scope and pre-PR's list of hooks take in the hooks' paths, beside the old ones.
- **dev.fabrig.ai:** nothing by hand for the first increment. Its project's `merge_job` and `close_job` keep naming the old paths, which an older build runs and a build that reads hooks ignores. The second increment drops them as it reads them (section 5). Before it, ops moves CI's runner wrapper to `goal-pre-pr`'s new path.

## 7. Later

- **`prepare` and `key`,** a turn landing's commands (`../protocol/tooling.md`, sections 4 and 5), become hooks of the same form when they are taken up.
- **A `close` a goal can't loosen,** run from the landing branch rather than the goal's own, should a project need its refusal to hold (`ways.md`, 8.4's first pass).
- **The rest of `.temper/`:** ways, the project's charter words, and the read and sign-off that guard them stay `ways.md`'s.
