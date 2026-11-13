# Jobs

Jobs are jig's (jig's `domain/hosts/hub.md`, 9). The worker's root
composes a job capability, `temper-worker-domain-tooling`, beside
smith's agent processes, and declares in its hello the kinds it runs:

| Kind | What it runs | Its spec | Its result |
|---|---|---|---|
| 1, prepare | a landing's candidate for one repository (`../landing.md`, 5.2 and 5.3): temper's own merge train, or the project's prepare command | the repository, the round's base, the train's `combine` and its upstream's tip, its members' heads in order with their messages, the upstream candidates and the pins between them, the command and the home's revision | how each member was combined, merged or squashed, or the conflict's member and files; each gate's key, when the project's key command is named |
| 2, key | a gate's key at a candidate, by the project's key command (`../landing.md`, 6.1) | the repository, the candidate, the gate, the command and the home's revision | the key |
| 3, merge | a merge of a base into a head, the conflicts the project's merge command knows settled (`../forge/change/repairs.md`) | the command, the head and the base | the files settled, and those left |
| 4, close | what a goal leaves in its branch settled by the project's close command, before its landing (`../forge/branches.md`, 2) | the command and the close request | the files settled |

- **Its kinds are entries** the root gives the capability: each a
  kind's number, the command's verb (its second argument,
  `../../protocol/tooling.md`, section 2) and whether it delivers. The hello
  declares each with the worker's limits for jobs: its spec's and
  result's bytes and its longest deadline. Prepare and key are the first
  two, by the table, merge the third and close the fourth; a worker
  declares a kind only once it is built, and none before. A kind's
  meaning to a worker is fixed by its number: a change of it is a new
  kind, so a job is never placed on a worker of a build that reads it
  otherwise (`../../protocol/wires.md`, section 7).
- **One spec and one result for every kind:** the spec, temper's record
  in its payload codec, the command's path in the job's directory and
  the request file's bytes, the path under `home/` for a command of the
  project's home (prepare and key) and under the repository's own name
  for one of the repository it merges or closes (merge, close); the
  result, the result file's bytes as the
  command wrote them. Exit 0 and 3 finish the job with that result
  (`../../protocol/tooling.md`, section 1); any other exit fails it as its
  own, with the end of its error output as detail; a result file missing
  or past its bound fails it too. The engine's procedure that made the
  job reads the result.
- **In the checkout,** prepared from the job's items as a run's
  workspace is (`checkouts.md`): the repository at its main's tip, each
  member's head fetched beside it, each upstream at its candidate, and
  the project's home at the pinned revision, read-only.
- **A command is a process** of the protocol layer's, its argument list
  the contract's (`../../protocol/tooling.md`), its directory the job's,
  its environment free of credentials and of the worker's own, its
  wall time the job's deadline, its output files bounded; it reaches no
  forge, since the checkout fetched everything it reads.
- **Then the delivery:** a prepare job's candidate is pushed to the
  landing's branch the items name as writable, expecting it new
  (`../../protocol/git.md`, 4.4, `Create`); a branch already there fails the
  job as undelivered, and the landing reads the branch (`../landing.md`,
  5.2). A key job delivers nothing. A delivering
  kind other than prepare delivers, on exit 0, the head of the repository
  its items name as writable, as a fast-forward onto that branch through
  the checkout's push (`checkouts.md`), as a run's delivery is; a push refused
  fails the job as undelivered.
- **The worker's limits** bound a job's spec, its result, its output
  files and its processes, and a job beyond them is refused as invalid.
- **The merge kind,** kind 3, verb `merge`, delivering, is built first
  (`../../protocol/tooling.md`, 4.1). Its checkout holds one repository,
  the change's, under its own name, writable, prepared as a resolution's is:
  the head checked out and the base merged in, not committed, the
  conflicts marked (`checkouts.md`); its command is the project's
  `merge_job`, a path in that repository, so the spec's command is the
  repository's name, `/`, then that path. On exit 0 the delivery commits
  the tree as the merge, both parents, and pushes it as a fast-forward
  onto the branch the item names as writable, a resolution's push; a
  file still unmerged or marked refuses it, and the job fails as
  undelivered. On exit 3 nothing is delivered. The engine learns which
  from the branch, not the result: its head moved, or not (`../forge/change/repairs.md`
  ). Its bounds, the engine's declaration and the worker's alike:
  a spec of 4,096 bytes and a result of 512, within jig's
  `tasks.spec_bytes` and `tasks.result_bytes` in the small profile,
  which the core holds every declared kind to, and a deadline of 10
  minutes, the small profile's constants. The engine declares the kind
  only where its tasks limits hold those bounds; a deployment that
  configures them smaller declares no kind 3 and makes no merge job.
- **The close kind,** kind 4, verb `close`, delivering
  (`../../protocol/tooling.md`, 4.2). Its checkout holds one repository,
  the close change's, under its own name, writable, prepared as a
  producer's is: its own branch made at the goal's branch's tip and
  checked out, clean (`checkouts.md`); its command is the project's
  `close_job`, a path in that repository, so the spec's command is the
  repository's name, `/`, then that path, and its request the close
  request's bytes, which the engine writes whole. On exit 0 the delivery
  commits the tree as the command left it, one parent, the tip, its message
  `Settle goal G's status`, G the request's goal, and pushes it as a
  fast-forward onto the branch the item names as writable, a producer's
  push; with nothing staged it commits and pushes nothing, and the job
  is done. The engine learns which from the branch, as for a merge: its
  head moved, or not. Its bounds, the engine's declaration and the
  worker's alike: a spec of 16,384 bytes, a result of 512 and a deadline
  of 10 minutes, the small profile's constants; the engine declares it
  only where its tasks limits hold those bounds, as kind 3's.
- **The prepare kind,** kind 1, verb `prepare`, delivering, built as
  temper's own merge train for a round of one member
  (`../landing.md`, 5.2), the project's prepare command not yet. Its spec's
  command is empty, which names temper's own: the job runs no process.
  Its request is `../../protocol/tooling.md`, 4's, read by the worker. Its
  checkout holds one repository, the train's, under its own name,
  writable, prepared as a base with heads merged in (`checkouts.md`) but
  from the round's base by its commit, not a branch's tip: the base
  checked out, the member's head fetched by its commit and merged with no
  fast-forward (`../../protocol/git.md`, 4.6), a head the base contains
  still a merge, and, where the request names the train's upstream, its
  tip fetched by its commit beside them. The merge is committed at once
  (`../../protocol/git.md`, 4.7) by the request's `combine`, the
  repository's identity as its author:
  - **`merge`,** or **`squash` for a member that brings its upstream's
    commits** (`../landing.md`, 5.2), which `Brings` (`../../protocol/git.md`,
    4.9) tells of the head, the base and the upstream's tip: two
    parents, the base and the head, and the member's `message` from the
    request as its message;
  - **`squash`** otherwise: the same tree, one parent, the base, and the
    member's `squash` message, committed even when the tree is the
    base's (`always`).

  A head that does not merge cleanly is no failure, whatever `combine`
  says: the job ends with exit 3's result, `{"conflict": {"member": N,
  "with": "branch", "files": [PATH]}}`, the files as the merge names
  them, cut to the result's bound and marked `"cut": true` when cut, and
  nothing is pushed. A clean merge ends with exit 0's result,
  `{"combined": [{"member": N, "by": "merge" | "squash"}]}`, and
  delivers its commit to the writable branch, the landing's, expecting
  it new (above). A request of more than one member is
  refused as invalid until the merge queue's batches
  (`../../draft/merge-queue.md`, 5.1). Its bounds, the engine's declaration
  and the worker's alike: a spec of 16,384 bytes, a result of 4,096 and
  a deadline of 10 minutes, the small profile's constants; the engine
  declares it only where its tasks limits hold those bounds, as kind
  3's. The engine learns the candidate from the branch, read afresh,
  and the conflict from the result (`../landing.md`, 5.2).
- **The spec is written by the engine's root** as it makes the job, in
  the worker wire's `JobSpec` record, since jig takes a job's spec as
  the application's bytes when the job is made (jig's
  `domain/tasks/lifecycle.md`, 3.1) and only the root makes it: the one record
  of a protocol crate the root encodes, a pure function of the
  command's path and the request's two commits. The domain reads no
  job's result: the change acts on its branch, read afresh, and carries
  the result's bytes as words.

Before writing the request or starting the command, the job capability
refuses a command path that is absolute or contains a `..` segment. The path
resolves relative to the job's workspace root (2026-11-04).

A merge job prepares whatever the count of the files in conflict. Past
the checkout's `conflicts`, every file stays marked in the tree and
unmerged in the index, and the checkout names the first `conflicts` of
them (`checkouts.md`; `../../protocol/git.md`, 4.6). The command reads the
index, never that list, and stages each file it settles, so its delivery
lists only the files it left (2026-11-09).

A job's failure is a line on the side that met it, and on the engine's
too (2026-11-09):

- **On the worker,** when its hub answers a job failed or refused, one
  line on its stderr: `<binary> worker: run RUN attempt ATTEMPT job KIND
  VERB failed CLASS: WHY`, or `refused REASON: WHY`. RUN and ATTEMPT are
  the assignment's, KIND and VERB the kind's entry, CLASS the job's
  failure as jig types it (`Unprepared(Transient)`, `Failed`,
  `Undelivered(Failed)` and the rest; jig's `domain/hosts/hub.md`, 9).
- **The why** is what the worker saw. A preparation that failed names its
  step (clone, fetch, the base's fetch, check out, merge) and git's
  fault, then the end of git's error output where the operation kept it.
  A command's failure names its exit or signal and the end of its error
  output. A passed deadline says so. A result missing or past its bound
  says which. A delivery not made names the push's outcome and its
  diagnostic. A refusal names its cause. Where jig's answer carries a
  detail (a preparation's, a command's, a start's), the why is that
  detail, within the host's `detail_bytes`. Before sending the answer,
  the root also puts a delivery's cause or a deadline's step in that
  detail. A failed request write names `request.json`; a process setup
  failure names the command and directory; a process gone without an
  exit names that step. A missing or oversized result names exit 0 or
  3 and the stderr tail. The worker's line may keep more of a delivery's
  diagnostic than the bounded answer.
- **On the engine,** when the hub accepts that answer, `README.md`, section 4.3's
  operator line, in its form: `<binary> serve: task TASK run RUN attempt
  ATTEMPT failed CLASS: job KIND: DETAIL`, or `refused REASON: job KIND:
  WHY`. A job held for its failures has the way out of a run's (`temper
  help hold`), so the engine's log names each of its failures as it does
  a run's. An assignment the engine cannot encode, which it answers
  itself as refused, is the engine's line alone.
- **Each line** is written once per answer, escaped to one line, its why
  cut to its last 4,096 bytes, as `README.md`, 4.3's is. The lines are operator output:
  no fact or turn holds them. A failed job's answer carries its cause;
  jig keeps the last `REASON_BYTES` (128 bytes) as UTF-8 on the retry notice
  and, past its allowance, the hold (jig's `domain/tasks/lifecycle.md`, 6).
  The change's held merge delegate therefore says why it failed, alongside
  its failure class and the way out.

A delivering job's writable repository is the one its items name with a
branch to push, and the engine names one only for a job its change names
as its delegate: that change's repository and own branch, on every
attempt, a released job's too (`../forge/change/repairs.md`, its close job
held for its failures). Nothing else makes one writable, neither the
job's authority nor the goal's branch, which a close job starts from and
never pushes. Every other repository of its checkout is read-only, a
context repository or one whose adopted push the forge has refused
(jig's `domain/connectors.md`, section 12) among them. When the command
ends with a delivery due, the worker counts the writable repositories of
the job's prepared checkout: with one, it delivers there; with none or
several, it delivers nothing and fails the job, its detail
`no unique writable repository: saw ` then each repository of the
checkout in the items' order, its name, a space and `writable` or
`read-only`, joined by `, `, as many as fit the host's `detail_bytes`
whole, then `, and N more` for the rest
(`no unique writable repository: saw home read-only`); with no
checkout left to count, `no unique writable repository: no checkout`
(2026-11-12).
