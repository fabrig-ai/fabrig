# Preparing a checkout

This document holds checkout preparation, its starting points and failure diagnostics, split from [`checkouts.md`](checkouts.md).

## 1. Prepare

- **Prepare** fetches what the assignment names and checks out exactly its
  starting point. It fetches each branch of a repository at most once: a
  pin or a baseline naming a branch this prepare has fetched, or a commit
  it has fetched, takes that fetch's commit, and a pin is pointed at it
  locally (`../../protocol/git-operations.md`, 1.12). A branch or base that does not exist is a preparation
  failure, permanent until something changes on the forge: the worker
  creates no branch but by pushing a run's change, and branches temper
  needs before then are made by the engine, as effects (../forge/branches.md
  ).

## 2. The landing pin

- **The landing pin.** A repository whose item names a landing branch
  (`hosted-runs.md`, 1.1) has it fetched once per hold, during prepare, after its start is
  fetched, on the same remote, into
  `refs/temper/landing` (`../../protocol/git-operations.md`, 1.3), or, where it is the
  branch the start fetched, pointed at that fetch's commit: every start of a
  change's producer, a saved or resumed one's included, and a goal's later
  run. That tip is frozen for the hold: no later operation of the hold
  moves the reference but a catch-up, which fetches it afresh (*A landed
  dependency's catch-up*, below), so the project's checks take the same base at every
  delivery and at finish until then (`tools/pre-pr/design.md`, section 2). A missing
  landing branch fails preparation. The next hold fetches it afresh, so
  the landing tip may move between attempts; a pin a cached checkout kept
  from an earlier hold is overwritten, never read. The pin changes neither
  the checkout start nor ownership, drift admission or the saved-work
  baseline.

## 3. The replaced head's pin

- **The replaced head's pin.** A writable repository whose item names no
  landing branch has the head its push branch has at prepare fetched once
  per hold, on the same remote, and the same `refs/temper/landing` pointed
  at it: the head its delivery will replace, the one fetch serving its
  baseline, its pin and, where it is the start's branch, its start. For a repair, that is the change's head it
  repairs, whatever its start (its branch, or saved work); for a resolution
  or a merge job, the branch's head the base is merged into. Where the push
  branch does not exist yet (a chat's first run), its start's commit is
  pinned. So a repair the change asks for itself checks what its own edits
  changed against the head it repairs, never against the branch the change
  lands on: a CI repair of a goal's landing into main checks its own edits,
  not the goal's (`tools/pre-pr/design.md`, section 2), and no pin a cached
  checkout kept from an earlier hold is read. A repair asked by a landing
  names a pin instead (*A named pin*, below). The pin is frozen for the hold
  as a landing branch's is. A read-only repository delivers nothing and runs
  no check, and its reference is left as it is. The item still names no
  landing branch, so nothing else of it changes: it is never taken for a
  goal's design delivery (`../agent/README.md`, section 5), and its start,
  push and expected head are as before.

## 4. A named pin

- **A named pin.** A writable item may name a pin, a commit by its full
  object id, and then names no landing branch. The engine names one on the
  item of a repair asked by a landing alone: the base that landing's round
  used, its failed receipt's (`../forge/change/repairs.md`, *A landing's
  repair*). Prepare fetches the push branch's head as for the replaced
  head's pin, the one fetch serving its baseline and, where it is the
  start's branch, its start, and points `refs/temper/landing` at the pin in
  place of that head: a `Point` (`../../protocol/git-operations.md`, 1.12) where this
  prepare already fetched that commit, otherwise one fetch of it into
  `Into::Landing` (`../../protocol/git-operations.md`, 1.3), its want the commit. A
  pin the remote lacks fails preparation as a missing commit; it never
  falls back to the replaced head. So a landing's repair is judged as that
  landing judged its change: its diff, its figures and its tests against
  the base the landing used, whatever the change's head and however old
  its source (`tools/pre-pr/design.md`, section 2). The pin is frozen for
  the hold as every pin is, and changes nothing else of the item: its
  start, push, expected head, own delivery, ownership and drift admission
  are a repair's as before. An item naming both a pin and a landing branch
  is refused at the worker's entrance as an invalid start that names it
  (`hosted-runs.md`, 1.1).

## 5. Main's counts

- **Main's counts** (`../../draft/test-budgets.md`, 3). Each repository
  whose prepare writes `refs/temper/landing` (any pin above) asks one
  `Counts` at once after it, on the same remote and identity: the forge's
  `refs/temper/counts`, the commit holding `counts.json` that the gate's
  runner publishes, fetched into the store with its blob and pointed at in
  `refs/temper/counts`, which the view copies into the run's `.git`
  (`../../protocol/git-operations.md`, 1.3 and 1.13), so the run's checks
  read it offline. Then the pin's continuation follows.
  - **Where the forge has none,** nothing is fetched and nothing fails:
    no reference is written, a cached checkout's from an earlier hold is
    removed, and the prepare goes on as without it.
  - **Frozen for the hold:** written afresh by each prepare alone, never
    by a catch-up, so a cached checkout's is replaced, never read. A
    repository whose prepare writes no pin asks none, its reference left
    as it is.
  - **A fault** fails the preparation as a fetch's does, at the stage
    `fetch counts` (section 12.1): its remote has just served the pin, and
    no prepare ends with a reference it could not bring up to date. Its
    fetch counts toward the store's next `Pack` (*Kept packed*).

## 6. A goal's reds for each check

- **A goal's reds for each check** (`../forge/reds.md`, 5.2 and 5.3), in
  each writable repository with a pin, under the run's git directory, at
  `.git/temper` (`../../protocol/git-operations.md`, 1.13): prepare
  removes `temper/reds` and writes `temper/reds-wait`; a check's start
  removes `temper/reds`, and, once the engine answers, the checkout finds
  the check's base and each red's ancestry with git, in the worker's
  directory, whose HEAD and pin the run's view holds too
  (`git merge-base`, `git merge-base --is-ancestor`), and writes
  `temper/reds` through `temper/reds.tmp` and a rename; a check's end
  removes it. These are the checkout's typed operations, run as its
  others are, one at a time; a failed one writes no file and fails
  nothing else of the run.

## 7. An existing producer branch

- **An existing producer branch** compares with the pinned tip: it is the
  frozen comparison for existing work below, only for an inherited
  producer that starts on its push branch.

## 8. A base with heads merged in

- **A base with heads merged in** fetches the base branch and each head,
  checks out the base's tip, and merges each head into it in their
  order, committing each merge at once, with two parents (what was
  checked out so far and the head) and the repository's identity as its
  author, and checks that commit out before going on (`../../protocol/git-operations.md`,
  1.7: a commit moves neither HEAD nor the merge's state), so that the next
  merge can run and the agent is handed a checkout on the last commit with no
  merge in progress; a head the checkout already contains is no merge. The last
  commit is the checkout's start, for its saves and its push alike, and
  the branch it pushes is made by its first push, as a base's is. A
  head that does not merge cleanly fails the preparation as a conflict,
  naming the head, permanent for this assignment: nothing of the
  merges is kept, and nothing is pushed. The change it was for then
  waits for its dependencies to land (`../forge/change/questions.md`, 1).
  This is the one conflict a prepare refuses, and only an early producer's
  start meets it; its detail (`checkouts-prepare.md`, section 12) is stage and fault
  `merge: Conflicted`, and its words say why,
  `head <head> does not merge into <branch>`, `<head>`
  the head's full id in lowercase hexadecimal, 40 digits for a SHA-1 id and
  64 for a SHA-256 one, as a merge's title spells it
  (`../../protocol/channel.md`, 4), and `<branch>` the start's branch.

## 9. A merge in progress

- **A merge in progress** fetches the branch and the base commit, checks
  out the branch, and merges the base into it as git does, leaving the
  files in conflict with their markers and the merge's state in the
  repository. The agent edits the files and runs the checks; it has no
  git writes (smith's `domain/tools.md`, section 5). Its push commits the
  tree as the merge, with two parents, the branch's head and the base
  commit, even when the tree is unchanged from either; a file the merge
  left in conflict that still holds a conflict marker is refused at
  push, and the run is told which. More files in conflict than the
  checkout's `conflicts` still make a merge in progress: each stays
  marked, and the start names the first `conflicts` of them, in git's
  order (`../../protocol/git-operations.md`, 1.6). A conflict this start meets is the
  run's to settle, never a failure of the prepare: a resolution's start,
  whether it merges the base into the change's branch or, with nothing of
  the change pushed, into the base branch, is always prepared so, its
  files named in its start. A producer's start, the one naming its
  landing branch (`../forge/change/questions.md`, 1), first asks whether its branch already
  contains the base: if so there is nothing to merge, and it goes on as a
  start from that branch or saved work does, with no merge state and no
  second parent. A resolution's or a merge job's names no landing branch and
  always merges. A producer's base that merges cleanly is committed at once,
  as a base with heads merged in is: two parents (the commit it merged into
  and the base), the repository's identity as its author, the title
  `Merge <base>`, and that commit checked out (`../../protocol/git-operations.md`, 1.7). So
  its run starts on a clean tree with no merge state, its earlier work and the
  base's files both in its head, and nothing staged to misread or remove. The
  commit it merged into stays the checkout's start, so a save with nothing new
  saves the merge commit, and its first push carries the merge, even with
  nothing new. A producer's conflict stays in progress as above, and its start
  names the files in conflict (smith's `domain/run-delivery.md`, 1.3).

## 10. A goal's later run

- **A goal's later run** (`../forge/branches.md`, 3) starts at g's tip; where its
  run branch holds a head g lacks, it starts as a producer does, on a merge
  in progress naming its landing branch: the branch is g, the commit merged
  into it is that head. A clean merge is committed at once, and a conflict
  stays in progress. It pushes to g, a design delivery (below).

## 11. Merge commits' titles

Each commit made while preparing a base with heads merged in has the title
`Merge <head>`, where `<head>` is the merged head's full hexadecimal object
ID (2026-11-04).

## 12. A failed prepare says where and why

A prepare that fails answers with a detail (`Prepared::Failed`'s `detail`):
operator evidence, bytes never parsed, that says which workspace and
repository, which stage and git command, and how long each part took, so
that a timeout reads as the operation's own time or as time spent before
it. It travels in jig's failure detail (`Failed.detail` and
`JobFailed.detail`, at most 4,096 bytes, jig's worker wire), so no wire or
stored format changes; the worker's line and serve's print it escaped
(12.3, 12.4).

### 12.1 Its form

```
<stage>: <fault> (workspace <w>, repository <i> <name>, command <command>, operation <o> ms of <d> ms, prepare <p> ms, queued <q> ms)[: <words>]
```

- **`<stage>`:** the step the prepare was in, in the words it has always had
  (`make`, `store`, `clone`, `fetch`, `check out`, `fetch landing`,
  `fetch counts`, `merge`, `check out merged`, a job's `fetch member` and the rest;
  `hold.rs`, `preparation_detail`). A step that checks reds files fails
  nothing and has no detail.
- **`<fault>`:** git's fault as `git::Fault::name` writes it
  (`Missing(Branch)`, `Unreachable`, `Broken`, `TimedOut`, ...), or
  `Conflicted` for a head that does not merge.
- **`workspace <w>`:** the workspace's token in decimal, as io names its
  directory, `workspaces/<w>` (`../../protocol/git.md`, section 2).
- **`repository <i> <name>`:** the repository the step names: its place in
  the spec in decimal, and its directory's name. A step that names none
  (`make`) says `repository none`.
- **`command <command>`:** the invocation the failure ended in, as the git
  component names it (`../../protocol/git.md`, section 6): git's subcommand,
  so `checkout` and `clean` are told apart within the one `check out`
  stage, or a file operation's word. A terminal that names none (one that
  ran no invocation, as an operation the component received past its
  deadline, and a conflict) says `command none`.
- **`operation <o> ms of <d> ms`:** the operation's time, from when the hold
  asked it of io to its terminal, against its deadline's length:
  `checkout.remote_timeout_ms` for one that reaches the forge, the local
  timeout otherwise (`git::Op::is_remote`).
- **`prepare <p> ms, queued <q> ms`:** the prepare's time, from its arrival
  to its failure, and the part of it spent queued for a permit, from its
  arrival to its admission, 0 for one admitted at once. Earlier operations
  and a wait for another hold's store are in `<p>` alone, so `<p> - <q> -
  <o>` is what the admitted prepare spent before this operation.
- **`<words>`:** the rest of why, last: git's diagnostic, the tail of its
  stderr (`PushDiagnostic`), or, for a head that does not merge,
  `head <head> does not merge into <branch>`. With nothing to say, the
  detail ends at the parenthesis.

Times are in whole milliseconds, rounded down, from the domain's clock at
each step (`env.now`): the checkout arms no timer for them. For example,
the stage that failed six times on 2026-11-15 would now read:

```
check out: TimedOut (workspace 3, repository 0 fabrig, command checkout, operation 60000 ms of 60000 ms, prepare 63120 ms, queued 2950 ms)
```

### 12.2 Its bound

A detail is at most 2,048 bytes. Its stage, fault and fields come first and
whole, but for a name past 256 bytes, cut to its first 256; its words take
the room left, and past it keep their first bytes. With the default
`name_bytes` (256) and a diagnostic's 512 bytes nothing is cut. Every cut
on the way, this one, the worker's (12.3) and serve's line's (12.4), falls at
the end, on the words: the fields keep their room. jig's hub keeps a
detail's last `hub.detail_bytes` (4,096 by default), past which a detail
of this bound never goes.

### 12.3 The worker names itself and logs it

- **The host.** As it answers a run or a job whose preparation failed
  (jig's `Unprepared`), the worker service puts `host <name>: ` before the
  detail, `<name>` its configured name (`workers[].name`, jig's
  `protocol/configuration.md`, section 4), and cuts the whole
  to `hub.detail_bytes`, its wire's bound, keeping its head. An answer the
  hub sends again after a relink carries the same. A prepare refused at
  the entrance or aborted (`prepare refused: Busy`, `prepare aborted`,
  `hosted-runs.md`, 1.3) takes the host too, with no fields.
- **One line each.** The worker logs a line for each failed preparation, a
  run's or a job's, through its notes (`worker_alone`'s `Note`), which its
  shell writes, an in-process worker's among serve's:

  ```
  <binary> worker: run <run> attempt <attempt> failed <failure>: <detail>
  ```

  `<failure>` is jig's failure as serve's run-failure line writes it
  (`Unprepared(Transient)`), and `<detail>` the detail with its host,
  escaped as serve's lines escape theirs, one line. It is logged once,
  when the answer first goes up: a kept answer sent again after a relink
  logs none. A job's own line (`jobs.md`) stays beside it.

### 12.4 Serve prints it whole

Serve's run-failure line (`hosted-runs.md`, 1.3) prints the detail whole, every
byte escaped. The wire bounds it at 4,096 bytes; a longer one, which no
wire carries, keeps its first 4,096, where the fields are.

## 13. Open questions

None.
