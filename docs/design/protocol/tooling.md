# The project's tooling

Provisional, 2026-10-27. Designed, not built. The contract between temper
and a project's own tooling: the `prepare` and `key` commands a project
that lands by turn may name (`../domain/landing.md`, 5.3 and 6.1), what
temper gives them, what they write, and how they end. temper runs them
as jobs on a worker (`../domain/worker.md`, 4.4). What is open is
section 7.

## 1. In one page

- **Two commands, both optional,** paths in the project's home, run at
  the home's pinned revision: `prepare` makes one repository's
  candidate; `key` says what one gate's result depends on.
- **A directory, not a forge.** temper fetches every repository the
  command needs into a directory it gives the command, and pushes what
  the command made. The command holds no credential, reaches no forge,
  and needs no model.
- **Files in, files out:** a request file in JSON, the repositories as
  git directories, and a result file in JSON, each bounded.
- **The exit code says which:** 0, a candidate; 3, a conflict it could
  not settle; anything else, the command's own failure.
- **Deterministic by contract:** the same request gives the same
  candidate's tree, so that a candidate prepared again keeps its keys.

## 2. How a command is run

- **Its argument list:** the command's path, then `prepare` or `key`,
  then the request file's path. Its directory is the job's directory.
- **Its environment** is `PATH` from the worker's configuration,
  `HOME` the job's directory, `LANG=C.UTF-8`, `TEMPER_JOB` the job's
  directory, and nothing else: no credential, no token, no variable of
  the worker's.
- **Its time** is the project's `prepare_timeout`, after which it and
  its children are killed and the job fails.
- **Its network** is the worker's containment's (`../domain/worker.md`,
  section 9): a command that fetches what the directory lacks does so
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
  its main's tip, with each member's head fetched as
  `refs/temper/members/<n>`, n from 1 in the landing's order; for each
  upstream the pins name: checked out at its candidate.
- **The command commits in `repos/<name>/`;** temper reads the commit
  `result.json` names from there, and its parent chain must contain the
  main's tip.

## 4. `prepare`

- **`request.json`:** `{"version": 1, "repository": NAME, "main":
  COMMIT, "members": [{"number": N, "branch": BRANCH, "head": COMMIT}],
  "upstreams": [{"repository": NAME, "main": COMMIT, "candidate":
  COMMIT}], "gates": [NAME]}`; commits as 40 or 64 hexadecimal digits,
  names as the project's settings give them.
- **`result.json`, exit 0:** `{"candidate": COMMIT, "keys": [{"gate":
  NAME, "key": HEX64}]}`, `keys` present only where the project names
  a `key` command and the command chose to give them here.
- **`result.json`, exit 3:** `{"conflict": {"member": N, "files":
  [PATH]}}`: the member whose change does not apply, and the files.
  temper resolves it under that member in a workspace of its own,
  main's tip merged into the member's head (`../domain/landing.md`,
  5.3); nothing of the command's directory is kept.
- **Any other exit,** a missing or malformed result, a candidate absent
  or not containing main's tip, or a file past its bound: the command's
  failure, the stderr's last `tooling_log_bytes` kept for the landing's
  goals.

## 5. `key`

- **Run** inside a prepare job, after `prepare`, when the project names
  both and `prepare` gave no key for a gate; or alone, as a key job, at
  a candidate the forge's rebase made (`../domain/worker.md`, 4.4).
- **`request.json`:** `{"version": 1, "repository": NAME, "gate": NAME,
  "commit": COMMIT}`, with `repos/<name>/` checked out at the commit.
- **`result.json`, exit 0:** `{"key": HEX64}`: 32 bytes the command
  derives from all its gate reads, the gate's tree among them. temper
  takes it as the key, under the gate's name and repository (6.1 of
  `../domain/landing.md` holds otherwise).

## 6. Bounds

`request.json` and `result.json` at most `tooling_file_bytes` each;
`files` at most `conflict_files`, each a path within a branch name's
bounds; members and upstreams within the landing's limits. Each is a
limit of the worker's (`configuration.md`, section 7).

## 7. Open questions

- **A command's own network:** whether a worker gives it none, so that
  a pin bump needs nothing fetched, or the project's.
