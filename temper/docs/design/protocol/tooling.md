# The project's tooling

Provisional, 2026-11-10. The contract between temper
and a project's own tooling: the `prepare` and `key` commands a project
that lands by turn may name (`../domain/landing.md`, 5.3 and 6.1),
the `merge` command any project may name for the merge job
(`../domain/forge/change/repairs.md`), and the `close` command any project may name
for a goal's closer (`../domain/forge/branches.md`, 2); what temper gives them,
what they write, and how they end. temper runs them as jobs on a worker
(`../domain/worker/jobs.md`). What is open is `tooling.md`, section 7.

## 1. In one page

- **Four commands, each optional.** `prepare` makes one repository's
  stack of prefixes and `key` says what one gate's result depends on, both
  paths in the project's home, run at the home's pinned revision;
  `merge` settles the conflicts it knows in a merge in progress, a path
  in the repository it merges, run from that merge (4.1); `close`
  settles what a goal leaves once its changes have ended, a path in the
  goal's branch's repository, run at the branch's tip (4.2).
- **A directory, not a forge.** temper fetches every repository the
  command needs into a directory it gives the command, and pushes what
  the command made. The command holds no credential, reaches no forge,
  and needs no model.
- **Files in, files out:** a request file in JSON, the repositories as
  git directories, and a result file in JSON, each bounded.
- **The exit code says which:** 0, a candidate, a merge settled, or a
  goal's branch settled; 3, a conflict it could not settle; anything
  else, the command's own failure.
- **Deterministic by contract:** the same request gives the same
  candidate's tree, so that a candidate prepared again keeps its keys.

## 2. How a command is run

- **Its argument list:** the command's path, then `prepare`, `key`,
  `merge` or `close`, then the request file's path. Its directory is the
  job's directory, and the command's path is relative to it: under
  `home/` for `prepare` and `key`, under the merged repository's name for
  `merge`, and under the goal's branch's repository's name for
  `close`.
- **Its environment** is `PATH` from the worker's configuration,
  `HOME` the job's directory, `LANG=C.UTF-8`, `TEMPER_JOB` the job's
  directory, and nothing else: no credential, no token, no variable of
  the worker's.
- **Its time** is the project's `prepare_timeout`, or for `merge` and
  `close` its kind's deadline (4.1, 4.2), after which it and its
  children are killed and the job fails.
- **Its network** is the worker's containment's (`../domain/worker/README.md`,
  section 8): a command that fetches what the directory lacks does so
  without temper's credentials.

## 3. The directory

```
<job>/
  request.json           what temper asks
  home/                  the project's home at the pinned revision, read-only
  repos/<name>/          each repository it needs, a git work tree
  result.json            what the command writes
```

- **`repos/<name>/`** for the repository being prepared: checked out at
  the request's `from`, with each member's head fetched as
  `refs/temper/members/<n>`, n from 1 in the landing's order; for each
  upstream the pins name: checked out at its candidate.
- **The command commits in `repos/<name>/`;** temper reads the commits
  `result.json` names from there, each prefix's parent chain containing
  the one below it, the lowest's containing `from`.

## 4. `prepare`

- **`request.json`:** `{"version": 1, "repository": NAME, "main":
  COMMIT, "from": COMMIT, "combine": "merge" | "squash",
  "train_upstream": COMMIT, "members": [{"number": N, "branch": BRANCH,
  "head": COMMIT, "message": TEXT, "squash": TEXT, "prefix": BRANCH}],
  "upstreams": [{"repository": NAME, "main": COMMIT, "candidate":
  COMMIT}], "gates": [NAME]}`; commits as 40 or 64 hexadecimal digits,
  names as the project's settings give them, `main` the round's base.
  - **`from`** is the commit the round stacks from: `main`, or the last
    prefix the round keeps (`../domain/landing.md`, 5.1).
  - **`members`** are those the job stacks, in the landing's order,
    each onto the one below and the first onto `from`: `number` its
    number in the landing, from 1; `branch` its own branch; `prefix`
    its branch in the landing, where its prefix is pushed
    (`../domain/landing.md`, 5.1).
  - **`combine`** is the train's (`../domain/landing.md`, 5.2 and
    13.1), and **`train_upstream`** its upstream's tip as the round read
    it, present only where the train has an upstream: a member whose
    head holds a commit `main` lacks and `train_upstream` holds is
    merged under `squash` too.
  - **`message`** is the commit message temper's own merge gives the
    member: `Merge pull request 'TITLE' (#N) from BRANCH into BASE`, a
    blank line, then `Landed by landing L for change C.`, L and C task
    numbers, BASE the train's branch.
  - **`squash`** is its squash's: `TITLE`, a blank line, then three
    trailers, a line each, `Temper-Change: C`, `Temper-Pull: N` and
    `Temper-Head: HEAD`, HEAD the member's head in hexadecimal at the
    forge's length. The engine knows a squash on the landing's branch by
    its `Temper-Head` trailer (`../domain/landing.md`, 5.2).
  - **TITLE** is the member's pull request's title, N its number.
- **temper's own,** where the project names no prepare command: the
  same request, read by the worker itself, with no command run, and the
  results below with `{"combined": [{"member": N, "by": "merge" |
  "squash"}]}` in place of `prefixes`, a member's way as it combined
  it, its prefix pushed by the worker (`../domain/worker/jobs.md`).
- **A conflict** is `{"member": N, "with": "branch" | "member",
  "files": [PATH], "cut": BOOL}`: the member whose change does not
  apply onto the prefix below, what it conflicts with (`branch`, the
  round's base; `member`, an earlier member alone), and the files,
  `cut` true when the list was cut to its bound. The member is skipped
  and the next stacked on the same prefix. temper resolves a conflict
  with the branch under that member in a workspace of its own, main's
  tip merged into the member's head (`../domain/landing.md`, 5.2), and
  lets one with a member wait; nothing of the command's directory is
  kept.
- **`result.json`, exit 0,** where at least one member combined:
  `{"prefixes": [{"member": N, "commit": COMMIT}], "conflicts":
  [CONFLICT], "keys": [{"gate": NAME, "key": HEX64}]}`, the prefixes in
  the stack's order, `conflicts` present only where a member was
  skipped, `keys`, the top's, only where the project names a `key`
  command and the command chose to give them here.
- **`result.json`, exit 3,** where none combined: `{"conflicts":
  [CONFLICT]}`, and nothing is pushed.
- **Any other exit,** a missing or malformed result, a prefix absent or
  not on the one below, or a file past its bound: the command's
  failure, the stderr's last `tooling_log_bytes` kept for the landing's
  goals.

### 4.1 `merge`

The merge job's command (`../domain/forge/change/repairs.md`; `../domain/worker/jobs.md`
): it settles the conflicts the project's rules know (a counted
baseline recounted, a generated file made again), with no model.

- **Named by the project's `merge_job`** (`configuration/engine.md`
  ), a path in the repository being merged, run from the merge in
  progress, not from the home: the files it settles and the scripts
  that settle them are that repository's. A command file left in
  conflict, or absent from the merge, is the command's failure.
- **Its directory** holds one repository, the one the job writes, under
  its own name (`<name>/`): checked out at the head being merged into, with the base
  merged in by temper and not committed, git's merge in progress, the
  conflicting files marked and unmerged in the index.
- **`request.json`:** `{"version": 1, "head": COMMIT, "base": COMMIT}`:
  the head merged into, and the commit merged in. The conflicting files
  are the index's (`git diff --name-only --diff-filter=U`), which the
  command reads itself.
- **What it does:** it edits the conflicting files it knows and stages
  each it settles (`git add`); it never commits, aborts the merge or
  writes another branch. temper commits the tree as the merge, with
  both parents, and pushes it.
  The reference recounts counted baselines and regenerates story A's
  playground log with its focused walk test on the merged tree,
  `SKEIN_CODEGEN_WRITE=1`. A generator that is unavailable, fails or
  does not produce a fresh file free of conflict markers leaves the
  file unmerged. The reference's B0 and B1 recordings require captured data
  outside the checkout; without that input they stay unmerged.
- **`result.json`, exit 0:** `{"settled": [PATH]}`, no file left
  unmerged: temper commits and delivers the merge (`../domain/worker/jobs.md`
  ). A file left unmerged or still marked fails the delivery, and
  the job with it.
- **`result.json`, exit 3:** `{"settled": [PATH], "left": [PATH]}`:
  nothing is committed or pushed, and the change makes a resolution in
  a merge in progress of the same base, its words carrying this result
  (`../domain/forge/change/repairs.md`); nothing of the job's directory is kept.
- **Any other exit,** or a result missing or past its bound: the
  command's failure, as `prepare`'s. The bound is the merge kind's
  result bound, 512 bytes in the small profile (`../domain/worker/jobs.md`
  ): a command lists the paths that fit, in index order, and adds
  `"more": N`, the count of those it left out.
- **Deterministic by contract,** as `prepare`: the same head and base
  give the same tree.

temper writes the merge request exactly as
`{"version":1,"head":"HEAD","base":"BASE"}`, without spaces, in that member
order. Each commit is lowercase hexadecimal, 40 digits for SHA-1 and 64 for
SHA-256. The catch-up commit carried by bytes parameter 16 uses that same
text form; another form refuses the change's start (2026-11-04).

### 4.2 `close`

The close job's command (`../domain/forge/branches.md`, 2; `../domain/worker/jobs.md`
): it settles, with no model, what a goal leaves in its branch once
every change of it has ended, before the goal's landing takes the branch
into its base. fabrig's settles the goal's in-flight status entries
(fabrig's `DESIGN.md`, section 2); another project's may settle what it
keeps of a goal's work in its own files.

- **Named by the project's `close_job`** (`configuration/engine.md`
  ), a path in the goal's branch's repository, as `merge_job` is: the
  files it settles and the scripts that settle them are that
  repository's. A command file absent from the branch is the command's
  failure.
- **Its directory** holds one repository, the one the job writes, under
  its own name (`<name>/`): checked out at the goal's branch's tip, on
  the close change's own branch made there, clean, nothing staged. That
  repository is writable on every attempt, a released job's too
  (`../domain/forge/change/repairs.md`); a checkout with none or several
  writable fails the job, naming each repository it saw and whether it
  could write it (`../domain/worker/jobs.md`).
- **`request.json`:** `{"version": 1, "goal": G, "changes": [{"task": N,
  "landed": BOOL}]}`: the goal's task number, and each change of the
  goal's tree whose landing branch is the goal's branch, by task number,
  ascending, `landed` true where it ended done having landed into the
  branch and false where it ended otherwise (cancelled, failed, or done
  with no change). temper writes it exactly so, without spaces, in that
  member order, `true` and `false` as JSON writes them:
  `{"version":1,"goal":7,"changes":[{"task":9,"landed":true},{"task":12,"landed":false}]}`.
- **What it does:** it edits the files it settles and stages each
  (`git add`); it never commits, never writes another branch, and never
  reaches a forge. temper commits the tree as it left it, with one parent, the
  tip, its message `Settle goal G's status`, and pushes it as the close
  change's head; an unchanged tree is no commit.
- **fabrig's, on a tag:** a tag naming no change of the request,
  another goal's or one of another form, is kept: its entry stays as it
  is, the command succeeds, and its result lists the tag as kept, within
  the bound and counted past it (fabrig's `tools/close/README.md`). A tag
  followed by sentence punctuation (`[in flight: change 161].`) is a tag
  all the same: its entry is deleted when its change landed, and
  otherwise the tag alone goes, with the space before it, the
  punctuation staying.
- **`result.json`, exit 0:** `{"settled": [PATH]}`, the files it staged,
  in index order, cut to the kind's result bound with `"more": N` as
  `merge`'s are (4.1). Nothing staged with exit 0 is success: the close
  change ends with no change (`../domain/forge/change/README.md`, 1).
- **Any other exit,** or a result missing or past its bound: the
  command's failure, its stderr's last `tooling_log_bytes` kept; the
  close change's producer fails and holds it (`../domain/forge/change/repairs.md`),
  which escalates to the goal (11.2 there).
- **Deterministic by contract,** as `merge`: the same tree and request
  give the same staged tree, and a second run over its own commit stages
  nothing.

## 5. `key`

- **Run** inside a prepare job, after `prepare`, when the project names
  both and `prepare` gave no key for a gate; or alone, as a key job, at
  a candidate the forge's rebase made (`../domain/worker/jobs.md`).
- **`request.json`:** `{"version": 1, "repository": NAME, "gate": NAME,
  "commit": COMMIT}`, with `repos/<name>/` checked out at the commit.
- **`result.json`, exit 0:** `{"key": HEX64}`: 32 bytes the command
  derives from all its gate reads, the gate's tree among them. temper
  takes it as the key, under the gate's name and repository (6.1 of
  `../domain/landing.md` holds otherwise).

## 6. Bounds

`request.json` and `result.json` at most `tooling_file_bytes` each, a
merge job's and a close job's within their kinds' bounds
(`../domain/worker/jobs.md`);
`files`, `settled` and `left` at most `conflict_files`, each a path
within a branch name's bounds; members and upstreams within the landing's limits. Each is a
limit of the worker's jobs (`../domain/worker/jobs.md`).

## 7. Open questions

- **The worker's `PATH` and its jobs' limits as configuration:** section
  2's `PATH` and section 6's limits have no member of the worker's
  (`configuration/README.md`, section 3) and no override (`configuration/limits.md`). The first kind built, `merge`, keeps them the
  profile's constants, so they stay open until a deployment needs
  another.
- **A command's own network:** whether a worker gives it none, so that
  a pin bump needs nothing fetched, or the project's.
- **A resolution from the merge job's tree:** a resolution starts from a
  merge in progress made afresh (4.1), so its agent settles again what
  the command settled, its checks recounting a baseline. Handing it the
  job's tree would need the job's directory kept, or its unfinished
  merge saved, which no workspace does yet.
