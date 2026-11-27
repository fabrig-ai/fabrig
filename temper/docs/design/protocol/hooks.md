# A repository's hooks

Provisional, 2026-11-27, from `../draft/hooks.md`. Which file of a repository temper runs at each of four moments, how it finds each, and the rule by repository while the older paths and settings stay. What a hook is given and how it ends are the delivery check's convention (`../domain/agent/charter.md`, section 4) and the tooling's commands (`tooling.md`, 4.1 and 4.2); this document says which file is the command.

## 1. In one page

- **Four hooks, named by when they run:** `.temper/hooks/deliver`, `.temper/hooks/finish`, `.temper/hooks/merge` and `.temper/hooks/close`. Present, a hook runs; missing, that moment has none. No setting names a hook.
- **Each runs from its own branch's version** (section 3), the one its job checks out. A change can edit any hook, so what must hold is CI and the landing rules.
- **Found two ways** (section 4): smith finds `deliver` and `finish` in each writable directory as a run starts, from the conventions temper's charter names; temper finds `merge` and `close` by one forge read of `.temper/hooks/` at the job's commit.
- **By repository** (section 5): a repository with `.temper/hooks/` runs hooks alone; one without it runs `.temper/pre-pr`, `merge_job` and `close_job`, as before.

## 2. The hooks

### 2.1 By moment

| Moment | When it runs | Its contract |
|---|---|---|
| `deliver` | on the run's checkout, before a `deliver`'s push, after which the run goes on | the delivery check's (`../domain/agent/charter.md`, section 4): `refs/temper/landing` pinned; 0 passes, anything else refuses the delivery, its output's tail the feedback |
| `finish` | on the run's checkout, before the push of a `finish` that hands in a change, which ends the run once it lands | as `deliver`'s |
| `merge` | in a merge in progress, on a conflict its change's merge job may take | the merge command's (`tooling.md`, 2 and 4.1) |
| `close` | once a goal's changes have ended, before its branch lands whole | the close command's (`tooling.md`, 2 and 4.2) |

- **`deliver` and `finish` are two moments, each with its own hook:** each delivery runs its moment's hook and only that one, so a project checks the two differently or, linking one to the other, the same. A `save` runs none, and neither does a `finish` with a report, a failure or a verdict.
- **Bounded as tooling is:** a hook runs with no credential and reaches no forge; `merge` and `close` are bounded as `tooling.md`, sections 2 and 6 say, and `deliver` and `finish` keep the check's bound (`../domain/agent/charter.md`, section 4).
- **Each file's head says its contract:** its moment, what it is given and its exit codes, citing this document and the contract above, so a person reading the directory needs nothing else.

### 2.2 A hook's file

- **A hook is an executable regular file** at `.temper/hooks/<moment>`, **or a symbolic link** whose target is a bare name, with no `/`, naming an executable regular file in the same directory, as fabrig's `finish` names `deliver`. One link, never a chain.
- **Nothing at the path is no hook.** Anything else there, a file with no execute bit, another link, a directory or any other kind, is the hook's failure, not its absence:
  - at `deliver` or `finish`, the delivery is refused as a failing check, with no process run (smith's `domain/run-delivery.md`, 1.1);
  - at `merge` or `close`, the job fails as the command's failure (4.3).

## 3. From its own branch

- **Each runs from the version its job checks out** (the user, 2026-11-27): `deliver` and `finish` from the run's checkout, `merge` from the merge in progress, `close` from the goal's branch at its tip. That was each older script's rule, kept.
- **So a hook is the work's own code.** A change can edit any hook, and what it calls, in its own checkout:
  - `deliver` and `finish` are feedback, as smith treats a check (smith's `domain/run-delivery.md`, 1.1);
  - `merge` makes a candidate, which CI and the gates judge again at its head;
  - `close` settles and may refuse; its refusal is the project's own check, which a goal's changes could loosen as they could loosen `deliver`'s.
- **What must hold is CI and the landing rules.** A project that wants a person to see any change to its hooks, or to the tools they call, says so with a rule by path naming `.temper/hooks/` and those tools (`../draft/what-reaches-you.md`, section 7).

## 4. How they are found

### 4.1 `deliver` and `finish`: smith's conventions

- **temper's charter names three paths** in its conventions (smith's `protocol/charter.md`, section 2): the guide, `AGENTS.md`; the check for both moments, `.temper/pre-pr`; and the hooks directory, `.temper/hooks`. smith looks in each writable directory as the run starts: where `.temper/hooks` is a directory, its `deliver` and `finish` are the checks, each for its moment, and `.temper/pre-pr` is not run; elsewhere `.temper/pre-pr` runs at both (smith's `domain/run-delivery.md`, 1.1).
- **The prompt names the checks smith found** and the tool that runs each (smith's `domain/run-delivery.md`, 1.1), so the run knows what its `deliver` and its `finish` will run.
- **Older agents** follow `wires.md`, sections 5 and 10.2: the conventions' hooks directory is `droppable`, so a charter sent to an agent whose layout lacks it loses it, and that agent runs `.temper/pre-pr` at both moments, which a repository moving to hooks keeps as a wrapper (5). A charter an older engine wrote names no hooks directory, and an agent of this build runs `.temper/pre-pr` at both, as before.

### 4.2 `merge` and `close`: one read at the job's commit

- **The read** is `.temper/hooks` at a commit of the repository the job writes: one `Read::File` (skein's `forge/vocabulary.md`, 1.2) answered as a directory's, `skein_forge::api::Answer::Directory`, within the forge client's `answer_bytes`, as the workflows' directory is read (`../domain/forge/adoption.md`, section 11). One read answers every moment.
- **What it finds:**
  - **a directory:** the repository has hooks at that commit. An entry named for the moment, of any kind, is its hook, a link's entry among them (the forge lists a link as a file); no such entry is no hook for that moment. A listing the forge cut that names no such entry takes the hook as found, so the job's checkout decides (4.3), and a cut listing never skips a close's check;
  - **no directory,** the path missing or a file there: the repository has no hooks at that commit, and section 5's settings decide;
  - **no answer,** the read refused, failed or answered in another shape: the step waits and reads again, as its other reads do. It never takes a failed read as no hook, which would skip a close's check.
- **`close`** is read by the goal's closer at the goal's branch's tip once the goal's changes have ended, when it would make its close step (`../domain/forge/branches.md`, section 2). A hook found makes the close step, its change carrying the hook's path to its job (`../domain/forge/change/README.md`, 1); a directory with none, or no command by section 5, skips it.
- **`merge`** is read by the change, once a conflict is found that its merge job may take, at the head the merge is made on: the change's head for a pull the forge can't merge, an update's head for an update's conflict, the branch's tip for a catch-up (`../domain/forge/change/repairs-resolutions.md`, 1; `../domain/forge/updates.md`, section 1). The answer is kept while that head stands. A hook found makes the merge job run it; a directory with none sends the conflict to a resolution at once, and holds a catch-up.
- **The job's command** is the repository's name, `/`, then `.temper/hooks/merge` or `.temper/hooks/close` (`../domain/worker/jobs.md`, section 2), its arguments, directory and environment as `tooling.md`, section 2 says.

### 4.3 The job's checkout decides the rest

- **The worker checks a hook before it runs it,** in the job's checkout, by 2.2: an executable regular file, or a link to one beside it. Anything else, nothing there included, fails the job as the command's failure, its tail naming the hook and why, as an absent command fails today (`tooling.md`, 4.1 and 4.2), its tries and holds as those sections say. A hook left in conflict in the merge in progress is the command's failure too.
- **A hook the read found and the checkout lacks** is that failure: the branch moved between the read and the job, or the merge brought the base's side. The read only decides whether a job is made.
- **A worker of an older build** runs a hook's command as any command, without that check: the job's kind and spec are unchanged, so nothing tells the two apart (`wires.md`, section 8).

## 5. By repository, while the older ones stay

- **A repository with `.temper/hooks/`** at the commit read, or in the run's checkout as the run starts, runs hooks alone: a missing hook means none, `.temper/pre-pr` isn't run, and the project's `merge_job` and `close_job` are ignored for that repository.
- **A repository without it** runs as before: `.temper/pre-pr` at `deliver` and `finish`, the project's `merge_job` for `merge` and its `close_job` for `close`, each setting's presence switching its job on (`../domain/landing-settings.md`, 1.1).
- **The settings stay settable:** `temper ctl` takes and shows `merge_job` and `close_job` as before, for repositories without hooks and for older builds.
- **Wrappers for older builds:** a repository moving to hooks keeps its older scripts as wrappers that run the hooks. An older build, which reads only the older paths and settings, then runs the same code, and going back to one finds them.
- **Then the older ones go,** once dev.fabrig.ai, its engine and its workers' agents run a build that reads hooks, on the user's word (`../draft/hooks.md`, section 5).

## 6. Why

- **A directory, read once,** rather than a read per moment: the read that decides whether a repository has hooks also lists them.
- **The closer reads, not its close change,** so that a repository with no close gets no close change, as one whose project names no `close_job` gets none today; the close change carries what its closer found, so it runs that across a restart with no second read.
- **A catch-up's merge is judged at its change, not at the `change` call:** the call answers at once and reads nothing, and a project whose repository keeps a merge hook needs no `merge_job` (`../domain/agent/tools-change.md`, 1).
- **One hooks directory in smith's conventions,** not two paths: a moment's hook is named for smith's own tool, so the directory is all a host names.
- **`droppable`, now:** every build that reads a layout's text from a peer or a registry reads the mark, since the plans that read texts came after it (`../draft/layouts-both-ways.execution.md`, L1 before L2 and L3).

## 7. Later

- **`prepare` and `key`,** a turn landing's commands (`tooling.md`, sections 4 and 5), become hooks of the same form when they are taken up.
- **A `close` a goal can't loosen,** run from the landing branch rather than the goal's own, should a project need its refusal to hold (`../draft/ways.md`, 8.4's first pass).
- **A way's path** shows a goal's catch-up where the project names a `merge_job` alone (`../draft/ways.md`), and the goal's charter still calls a catch-up without a merge job refused at the call, where it now holds at its change (4.2): both follow when `ways.md`'s next pass takes hooks.
- **A shared hook link check.** The worker reads the link target with skein-io's `Op::ReadLink` (4.3), while smith opens the sibling it names with skein-io's `OpenSiblingFile` (4.1). One skein mechanism could check the same link rule (2.2) for both.
