# A repository's hooks

Provisional, 2026-11-27, from `../draft/hooks.md`. Which file of a repository temper runs at each of four moments, how long each may run, how each is found, and the rule by repository while the older paths and settings stay. What a hook is given and how it ends are the delivery check's convention (`../domain/agent/charter.md`, section 4) and the tooling's commands (`tooling.md`, 4.1 and 4.2); this document says which file is the command and how long it runs.

## 1. In one page

- **Four hooks, named by when they run:** `.temper/hooks/deliver`, `.temper/hooks/finish`, `.temper/hooks/merge` and `.temper/hooks/close`. Present, a hook runs; missing, that moment has none. No setting names a hook.
- **How long each may run** (2.3): `.temper/hooks/config.toml` says it per moment, never past the project's `hook_timeout_max_seconds`. Silent, the bounds from before hold.
- **Each runs from its own branch's version** (section 3), the one its job checks out. A change can edit any hook, so what must hold is CI and the landing rules.
- **Found in the checkout it runs in** (section 4): smith finds `deliver` and `finish` in each writable directory as a run starts, from the conventions temper's charter names; temper makes a merge job for each conflict and a close change for each goal's close, and the worker finds the hook in the job's checkout, none there being none.
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
- **Bounded as tooling is:** a hook runs with no credential and reaches no forge; `merge` and `close` are bounded as `tooling.md`, sections 2 and 6 say, and `deliver` and `finish` as the check is (`../domain/agent/charter.md`, section 4); each one's time is 2.3's.
- **Each file's head says its contract:** its moment, what it is given and its exit codes, citing this document and the contract above, so a person reading the directory needs nothing else.

### 2.2 A hook's file

- **A hook is an executable regular file** at `.temper/hooks/<moment>`, **or a symbolic link** whose target is a bare name, with no `/`, naming an executable regular file in the same directory, as fabrig's `finish` names `deliver`. One link, never a chain.
- **Nothing at the path is no hook.** Anything else there, a file with no execute bit, another link, a directory or any other kind, is the hook's failure, not its absence:
  - at `deliver` or `finish`, the delivery is refused as a failing check, with no process run (smith's `domain/run-delivery.md`, 1.1);
  - at `merge` or `close`, the job fails as the command's failure (4.3).
- **One open checks it,** in smith and in the worker alike: skein's sibling open (`OpenHow::PathSiblingFile`, skein's `runtime/kernel-files.md`, section 1) opens the name in the hooks directory, taking a regular file or one link to a bare name naming a regular file beside it and refusing anything else, and a stat of what it opened says whether it is executable.

### 2.3 How long each may run

- **The repository says it,** in `.temper/hooks/config.toml`, beside its hooks: a table per moment, each with `timeout_seconds`, whole seconds with the unit in the key's name, as temper writes its other figures (`configuration/file.md`).

  ```toml
  # .temper/hooks/config.toml: how long each hook may run
  [close]
  timeout_seconds = 1800
  ```

  It names no command and turns no hook on: a hook is still its file (2.2), a table for a moment with no hook means nothing, and a moment with no table keeps its bound from before (below).
- **Its form** is this subset of TOML, UTF-8, read line by line: a blank line; a comment, from `#` to the line's end, alone or after a line's content; before any header, `format = N` at most once, the file's format (`configuration/file-formats.md`, 2), N a decimal integer written in digits alone, 1 where the line is absent; a table's header, `[deliver]`, `[finish]`, `[merge]` or `[close]`, each at most once; and, under a header, `timeout_seconds = N`, at most once in its table, N a decimal integer written in digits alone, at least 1 and at most 4,294,967,295. Spaces and tabs may stand around each token. The file is a regular file of at most 4,096 bytes, read without following a link; missing, it is silent.
- **Its format is read first:** the whole file is parsed, then its format judged before any table: a `format` that isn't a whole number from 1 is ``.temper/hooks/config.toml:L: format: a whole number from 1``, and one above the newest, which is 1, is ``.temper/hooks/config.toml:L: format N; this temper-worker reads format 1: upgrade it, or write the file at format 1``, as `configuration/file-formats.md`, 5 words it, naming the binary that reads it; either is a fault, alone. Format 1 has no moves.
- **A fault is the hook's failure,** named with its line: a file that doesn't parse (a line of any other form, valid TOML or not; a header, a key or `format` given twice; `format` after a header; a key before any header; bytes that aren't UTF-8), a format the worker doesn't read, a value of 0, a table or key it doesn't know, or anything at its path but a regular file of at most 4,096 bytes. The line is ``.temper/hooks/config.toml:L: WHY``, L the line from 1, or 0 for the file as a whole, as ``.temper/hooks/config.toml:3: unknown key `timeout` ``. The whole file is checked before any of it is used. A fault fails each hook that runs with it, as a hook 2.2 refuses does (4.1 and 4.3); a moment with no hook has nothing to fail.
- **One reader, two callers:** the worker reads the file where a hook runs, and temper's check of a repository's `.temper/` reads it with the same `temper-protocol-hooks` reader before it lands (`temper-dir.md`, 3).
- **Silent, the bounds from before hold:** for `deliver` and `finish`, the check's bound, the charter's, its project's `check_timeout_ms`, or the agent's own (`../domain/agent/charter.md`, section 4); for `merge` and `close`, ten minutes, their jobs' deadline before the ceiling (`tooling.md`, 2). A command a project's `merge_job` or `close_job` names has ten minutes too (section 5).
- **Never past the ceiling:** the project's `hook_timeout_max_seconds` (`../domain/landing-settings.md`, 1.1), 3,600 unless an owner sets it, from 1 to 86,400. A value above it runs at the ceiling, and so does a `merge` or `close` whose ten minutes pass it. A value cut so is said: in a `deliver` or `finish` check's feedback, beside the time it was given (smith's `domain/run-delivery.md`, 1.1), and for `merge` and `close` in the worker's log line for the job and first in the job's failure tail where it fails, as ``hook `.temper/hooks/close`: asks 7200 s, past the project's ceiling: runs at 3600 s``. The file is the work's own, as a hook is (section 3), so the ceiling, which spends a worker's time, stays in settings, where a change can't reach it.
- **Read where the hook runs,** from the version it runs from (section 3), by temper's worker: for `deliver` and `finish`, in each writable directory of the run's checkout that holds `.temper/hooks/` as a directory, as the run starts (4.1); for `merge` and `close`, in the job's checkout once its hook is found, before it runs (4.3). Each value is cut at the ceiling the engine sends (`wires.md`, 8). An edit to the file within a run takes effect at the next run's start, as a hook's presence does.
- **The hook's own time ends it.** A `merge` or `close` job's deadline is its project's ceiling (`../domain/worker/jobs.md`, 2), so the hook's time, at most the ceiling, ends a hook that runs long: the worker kills its group then, and the job fails as the command's failure, its tail first naming the hook and its time, ``hook `.temper/hooks/close`: ran past its 1800 s``, then the command's error output's tail, tried again within its tries (`tooling.md`, 4.1 and 4.2). A `deliver` or `finish` hook's time is the check's bound at its moment, smith's cuts and its one retry applying as to any check, the retry cut at the ceiling too (smith's `domain/run-delivery.md`, 1.1).

## 3. From its own branch

- **Each runs from the version its job checks out** (the user, 2026-11-27): `deliver` and `finish` from the run's checkout, `merge` from the merge in progress, `close` from the goal's branch at its tip. That was each older script's rule, kept.
- **So a hook is the work's own code.** A change can edit any hook, what it calls, and its time, in its own checkout:
  - `deliver` and `finish` are feedback, as smith treats a check (smith's `domain/run-delivery.md`, 1.1);
  - `merge` makes a candidate, which CI and the gates judge again at its head;
  - `close` settles and may refuse; its refusal is the project's own check, which a goal's changes could loosen as they could loosen `deliver`'s.
- **What must hold is CI and the landing rules.** A project that wants a person to see any change to its hooks, or to the tools they call, says so with a rule by path naming `.temper/hooks/` and those tools (`../draft/what-reaches-you.md`, section 7).
- **A hook uses only what its checkout holds** (the user, 2026-11-27): the files of the version it runs from and what its job's directory or its run gives it (`tooling.md`, 2). One that needs anything else is a bug, of the hook or of what it does, and is fixed there.

## 4. How they are found

### 4.1 `deliver` and `finish`: smith's conventions

- **temper's charter names three paths** in its conventions (smith's `protocol/charter.md`, section 2): the guide, `AGENTS.md`; the check for both moments, `.temper/pre-pr`; and the hooks directory, `.temper/hooks`. smith looks in each writable directory as the run starts: where `.temper/hooks` is a directory, its `deliver` and `finish` are the checks, each for its moment, and `.temper/pre-pr` is not run; elsewhere `.temper/pre-pr` runs at both (smith's `domain/run-delivery.md`, 1.1).
- **Their times are in the start:** before the run starts, the worker reads `.temper/hooks/config.toml` in each writable directory that holds `.temper/hooks/` as a directory (2.3), cuts each value at the ceiling its items carry (`wires.md`, 8), and names in that directory's entry of the start the time of each of `deliver` and `finish` the file sets, with what the file asked, the ceiling, and the file's fault, if it has one (smith's `protocol/channel.md`, section 3). smith runs a moment's hook within its time in place of the checks' bound, and a fault breaks each hook it finds in that directory, as 2.2 says (smith's `domain/run-delivery.md`, 1.1). A directory the start names no time for keeps the checks' bound.
- **The prompt names the checks smith found** and the tool that runs each (smith's `domain/run-delivery.md`, 1.1), so the run knows what its `deliver` and its `finish` will run.
- **Older agents** follow `wires.md`, sections 5 and 10.2: the conventions' hooks directory is `droppable`, so a charter sent to an agent whose layout lacks it loses it, and that agent runs `.temper/pre-pr` at both moments, which a repository moving to hooks keeps as a wrapper (5). A charter an older engine wrote names no hooks directory, and an agent of this build runs `.temper/pre-pr` at both, as before. A worker and its agents are one build, so the start's times need no bridge (smith's `protocol/channel.md`, section 2).

### 4.2 `merge` and `close`: the job's checkout decides

- **temper makes the job wherever a hook could run** (the user, 2026-11-27), and reads nothing of the repository first:
  - **`merge`:** a merge job for each conflict its change's merge job may take, at the head the merge is made on: the change's head for a pull the forge can't merge, an update's head for an update's conflict, the branch's tip for a catch-up (`../domain/forge/change/repairs-resolutions.md`, 1; `../domain/forge/updates.md`, section 1);
  - **`close`:** a close change once a goal's changes have ended, at its branch's tip (`../domain/forge/branches-closer.md`, section 1).
- **Its spec asks the worker to find the hook** (`wires.md`, 8): its command the repository's name, `/`, then `.temper/hooks/merge` or `.temper/hooks/close` (`../domain/worker/jobs.md`, section 2); its `find` true; and its `fallback` the project's `merge_job` or `close_job`, as the same kind of path, or empty where the project names none (section 5). Its arguments, directory and environment are as `tooling.md`, section 2 says.
- **A hook there runs; none there is none** (4.3). The job that finds none ends at once, having cost its checkout. It finishes with nothing delivered, its result `{"settled":[],"hook":"none"}`, compact as `tooling.md`, 4.1 writes results:
  - **at `merge`,** a merge left in conflict goes to a resolution at once, spending no try, its words carrying that result as any merge job's (`tooling.md`, 4.1). A merge with no file in conflict, which only a catch-up's job meets, is not answered so: temper commits and delivers it, as it would a hook's that settled nothing;
  - **at `close`,** the close change ends with no change (`tooling.md`, 4.2).
- **A job that fails is no answer:** it is tried again within its tries, as a job is (`tooling.md`, 4.1 and 4.2), and never taken as no hook, which would skip a close's check.
- **Where no job can be made,** the engine declaring no merge or close kind, or the change's authority granting none (`../domain/worker/jobs.md`, 2), a conflict goes to a resolution at once, a catch-up holds, its words naming why, and the closer skips its close step.

### 4.3 The worker finds and checks it

- **A spec with `find`** (4.2): the worker looks at the repository's `.temper/hooks` in the job's checkout, without following a link.
  - **A directory:** the repository has hooks, and the moment's entry decides. Nothing there is none, and the job answers so (4.2). Anything there is checked by 2.2 and runs only if it passes; otherwise it fails the job as the command's failure, spawning nothing, its tail one line naming the hook and why, as ``hook `.temper/hooks/close`: not executable``, tried again as a failure is.
  - **Anything else,** nothing there, a file or a link: the repository has no hooks, and section 5 decides. A `fallback` runs as any command does, one that names nothing in the checkout failing the job as an absent command does (`tooling.md`, 4.1 and 4.2); with none, the job answers none.
- **A hook left in conflict** in the merge in progress is the command's failure, spawning nothing.
- **Then its time** (2.3): the worker reads `config.toml` beside the hook it found, a fault failing the job as the command's failure with the fault's line as its tail, and runs the hook within its time, cut at the ceiling its items carry and at the job's deadline.
- **A spec without `find`,** an earlier engine's: its command runs, checked by 2.2 where its path in its repository starts `.temper/hooks/`, nothing there the command's failure, as that engine expects, and timed by 2.3 as a found hook is.
- **A worker of an older build** can't keep a spec that sets `find`: the engine sends it none, and the job waits for a worker of this build, spending no try (`wires.md`, 8).

## 5. By repository, while the older ones stay

- **A repository with `.temper/hooks/`** in the job's checkout, or in the run's checkout as the run starts, runs hooks alone: a missing hook means none, `.temper/pre-pr` isn't run, and the project's `merge_job` and `close_job` are ignored for that repository.
- **A repository without it** runs as before: `.temper/pre-pr` at `deliver` and `finish`, the project's `merge_job` for `merge` and its `close_job` for `close`, each setting's presence switching its command on, as the job's fallback (4.3; `../domain/landing-settings.md`, 1.1).
- **The settings stay settable:** `temper ctl` takes and shows `merge_job` and `close_job` as before, for repositories without hooks and for older builds.
- **Wrappers for older builds:** a repository moving to hooks keeps its older scripts as wrappers that run the hooks. An older build, which reads only the older paths and settings, then runs the same code, and going back to one finds them.
- **Then the older ones go,** once dev.fabrig.ai, its engine and its workers' agents run a build that reads hooks, on the user's word (`../draft/hooks.md`, section 5).

## 6. Why

- **The job's checkout decides, not a read before it** (the user, 2026-11-27): the checkout is the version the hook runs from, so what it holds is what runs, where a read at the job's commit could only disagree with it, the branch having moved or the merge having brought the base's side. A job that finds none costs a checkout; a conflict and a goal's close are rare enough to pay it, and the engine keeps no answer, no kept read and no parameter for it.
- **None is an answer, a failure is not:** a job finishes for none, so it spends no try and the conflict's resolution comes at once; a job that broke is tried again, so a close's check is never skipped for a fault.
- **A catch-up's merge is judged at its job, not at the `change` call:** the call answers at once and reads nothing, and a project whose repository keeps a merge hook needs no `merge_job` (`../domain/agent/tools-change.md`, 1). A catch-up without a hook merges as one whose hook settled nothing: a clean merge lands and a conflict goes to its resolution, as with a hook.
- **One hooks directory in smith's conventions,** not two paths: a moment's hook is named for smith's own tool, so the directory is all a host names.
- **`droppable`, now:** every build that reads a layout's text from a peer or a registry reads the mark, since the plans that read texts came after it (`../draft/layouts-both-ways.execution.md`, L1 before L2 and L3).
- **One open checks the link rule** (2.2): smith's check and the worker's are one rule, and the sibling open checks it whole, so skein keeps no link read for it. A link read would only let a failure's tail name the link's target.
- **The time in the file, the ceiling in settings:** a hook's time is the work's, edited with the hook it bounds and read from the same version; the ceiling is what a worker's time costs, which an owner sets.
- **A subset of TOML,** not the whole: the file holds four tables of one key each, a worker reads it in its own steps without a TOML library, and any other form, valid TOML or not, is refused by its line rather than misread.
- **A check's retry is cut at the ceiling too:** smith's one retry at twice the time, for a check that only ran out of time, stays the agent's rule, and the start's ceiling cuts it beside the call's and the run's deadlines, so no run of a hook passes the ceiling. A moment the file doesn't time keeps the check's bound and its retry as before.
- **The items' ceiling is `droppable`:** a worker of an older build reads no `config.toml`, so the ceiling means nothing to it, and its runs go on as before.

## 7. Later

- **`prepare` and `key`,** a turn landing's commands (`tooling.md`, sections 4 and 5), become hooks of the same form when they are taken up.
- **A `close` a goal can't loosen,** run from the landing branch rather than the goal's own, should a project need its refusal to hold (`../draft/hooks.md`, 7).
- **A way's path** shows a goal's catch-up where the project names a `merge_job` alone (`../draft/ways.md`), and the goal's charter still calls a catch-up without a merge job refused at the call, where it now goes to its merge job, which a checkout with no hook answers as 4.2 says: both follow when `ways.md`'s next pass takes hooks.
- **A shared hook link check.** The worker reads the link target with skein-io's `Op::ReadLink` (4.3), while smith opens the sibling it names with skein-io's `OpenSiblingFile` (4.1). One skein mechanism could check the same link rule (2.2) for both.
