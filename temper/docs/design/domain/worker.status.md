# worker: status

2026-11-09. Where temper's code lags or departs from [worker.md](worker.md).
Section 4.3's failed and refused answer diagnostics reach serve's stderr and the box's
`serve.log`; that operator-output path has no gap. A failed-run hold's
`way out:` points to that log, and `temper help hold` names its location
under both `box` and `serve` (`../client/tools.md`, 6.1).
Section 4.3's busy start has no known gap: smith's busy, an agent with
no room or a run it has no room for, reaches the hub as
`FromAgent::StartBusy` (`temper-worker-domain`'s `agents.rs`, `ended`,
and `hand_out.rs`, `Finish::of`), and spends no try.

Section 4.4's failure lines have no known gap but the result's why below.
When its hub answers a job failed or refused, the worker writes its line
(`temper-service/src/worker.rs`, `job_answer`, the why kept by
`temper-worker-domain/src/jobs.rs`), an in-process worker's among serve's
notes, and the engine writes its own once, when its hub accepts the answer
(`temper-engine-domain/src/link.rs`, `answer`;
`temper-service-shell/src/host.rs`, `job_line`), an assignment it
cannot encode included. A preparation's failure carries its step and git's
fault as its detail (`temper-worker-domain-checkout/src/hold.rs`,
`preparation_detail`), a command's its exit or signal and its error
output's end (`temper-worker-domain-tooling/src/domain.rs`), and a delivery
not made names its push's outcome and diagnostic on the worker's line
alone. `job_stderr_forms_escape_and_bound_their_why`,
`job_answer_evidence_reaches_operator_once`,
`failed_jobs_name_the_workers_why_once` and the checkout's prepare tables
hold them.

Sections 4.4 and 5 have no known gap in a merge past the checkout's
`conflicts`: its preparation leaves the merge in progress, every file
marked and unmerged, and names the first `conflicts`
(`../protocol/git.status.md`), and the composed system world's
`suite::merge_job` runs a merge job that settles `conflicts + 1` files and
delivers (`testing.status.md`, 2.1). That story's rows also hold the
request's other candidates, on fake git and a fake forge: the hello's
kind, the command's path and request, the spec's and result's bounds,
and a base fetched by its id into a cached clone that never fetched its
branch. That the listing past `conflicts` was the fault of the request
`merge-job-runs-2` of 2026-11-09, every merge job held `out of tries:
transient failures` with no line in either log, stands on reading: its
evidence is outside the repository, and its proof is the next
deployment's lines, which no record shows.

Section 5's later goal start has no known implementation gap. The checkout
world's three goal stories cover r already contained in g, a clean merge
committed before the run and a conflict retained for the run to settle;
each delivery fast-forwards r and leaves g unchanged. They use fake git
and disk, and do not settle the deployment replay gaps below. A goal's
saved start overtaken by a repair remains deferred (`forge.md`, section 20).

Section 3.1 has no known gap in reading the link: the worker reads it
and fires its deadlines on every pass, whatever the entrance holds, and
the held entrance holds the engine's cap to a worker in frames, from the
link's sizes, and three records more for the link's own state; a new
record past it faults the link (`temper-service/src/worker.rs`,
`Worker::new` and `drain_link`). `temper-service`'s
`a_worker_whose_engine_records_wait_keeps_its_link`,
`the_link_is_read_while_its_last_output_waits` and
`a_new_wire_record_past_the_entrance_faults_the_link` hold it. A link
left unread under load is the likeliest cause of a deployment's worker
relinking every two to two and a half minutes on 2026-11-08, but that is
unproven: its evidence is outside the repository, and no deployment on
record has run since.

Section 6 has no known gap in every temper worker naming its agent's
`host_reply_text` as the hello's reply bound, the in-process worker among
them (`temper/crates/temper/src/agent.rs`). The service gives the worker's
`Limits::reply_bytes` the same figure (`temper-service/src/worker_settings.rs`).

- **Not yet:** 4.3: a held task names no run's failure, which the user's
  request of 2026-11-05 asked for if cheap: it needs a stored-format
  change, so it is section 12's question. The `ctl-words` request requires
  the user's decision on the bound and readers before that text is kept;
  no such decision is supplied. The log directions leave this question open.
  4.3: `temper help task` says an "invalid results" hold may mean an
  assignment was refused before any model turn (`temper/docs/help/task.md`),
  where such a start spends a permanent try, and a hold for it says
  permanent failures; `temper-docs`'s `tests/rules.rs` finds that page by
  those words.
  4.4: a result missing and one past its bound fail with the same why,
  `result missing or past its bound`, where 4.4 has the why say which: the
  job capability's `Read` answers no bytes for both
  (`temper-worker-domain-tooling/src/boundary.rs`, `Event::Read`).
  4.4: the prepare and key kinds: neither kind's entry is
  built, so the worker's hello declares the merge kind alone
  (`temper/crates/temper-service/src/limits.rs`, `job_kinds`), and with prepare the
  `Rewrite` delivery (`../protocol/git.md`, 4.10). 5: a missing base branch fails the
  preparation; the checkout creates it from the default branch for a
  repository that may be written
  (`temper-worker-domain-checkout/src/hold.rs`, `PrepareStep::Create`).
  5, 9: git's invocations in contained process trees;
  `temper-worker-protocol-git` runs them as plain child processes.
  5: a base with heads merged in has no real run on record: `temper-fake-git`
  holds a two-head start's commands and the `merge-in-progress` transcript git's
  refusal of a second merge, and the request `parallel-faults` of 2026-11-06 has
  its proof in a rerun of its replay, which no record shows. It asks whether this
  is the fault of the early start in the request `landing-fixes` of 2026-11-06,
  item 3, an uncommitted merge at `7913a87678`. Both deployments ran before
  `f6b4c408a7` had each merge it commits checked out, so it may be, by reading
  alone: neither was reproduced on a real run, and `landing-fixes` has its proof in
  the next landing plan on a real deployment, which no record shows either.
  5, *A merge in progress*: a producer's start that finds its base contained stands
  on `temper-fake-git` alone (`a_resumed_producer_holding_the_tip_pushes_no_merge`,
  `temper/tests/worker/checkout/tests/suite/merges.rs`), with that same proof.
  5, 10: a parked producer's delivered and saved edits survive a moved landing
  tip, repeated questions and a save refused for markers in the composed system
  world's two `parked` stories. Those use fake git and a fake disk, and their
  first committed edit is a delivery, not a run's own `git commit`. The
  `resume-told` request asks for proof in a real checkout; none is on record,
  nor its next deployment replay. The clean-merge preparation and the conflict
  advice, including the help page and stock charters, have scripted coverage
  alone; whether real producers and their answerers now read the tree rightly
  awaits that replay, as the request's `walk = false` leaves it.
  6: a standalone start past the process's `limits::SOCKETS`, 256, by reading
  21 slots on the owned example with workspaces and operations as many, is
  refused `engine.limits.io.sockets: 258 bytes, past its bound of 256`, naming a
  limit no document raises (`checks::process_io` sets it), and a worker alone
  past the link's `u32` caps, by reading about 31 slots at that example's hub
  (`jig-protocol-worker-wire/src/sizes.rs`, `worker_cap`), is refused
  `workers[0].limits: past its bound`, naming none. The `worker-many-slots`
  request of 2026-11-05 has each start's refusal name what to raise; it asked
  for 8 and 10, which fit, so this is section 12's question. No test runs 8
  slots past `temper::prepare_in`: that request takes the next deployment at 8
  and 10 as its proof, with no walk, and no record shows one.
- **Design departs:** 4.4, from the request `merge-job-runs-2` of
  2026-11-09, item 2, decided by temper/migration's answer of 2026-11-09:
  a job that fails for good holds with its failures' class alone, each
  failure's why in the worker's and the engine's lines, where the request
  has its reason in the hold. Words in the hold need section 12's
  stored-format change (*A hold naming its run's last failure*), left
  until the next deployment's lines are read.
- **Departs:** 5, 9: with `TEMPER_WORKSPACE_REUSE` in the worker's
  environment at its start, a workspace stays as it is between workstreams and
  across a restart; unset, which is the default and the only shipped form,
  sections 5 and 9 hold exactly. Any value counts as set, and it is read once.
  Set:
  - where section 5 has a new workstream take the least recently used
    workspace, made again, that workspace keeps its clone when it holds exactly
    the repositories the spec names, by directory and remote, and none starts
    saved. It is fetched and checked out afresh, so an unchanged file keeps its
    mtime. Anything else is made again, as section 5 has it, a workspace whose
    disk is not known included;
  - where `../protocol/git.md`, section 2 has a start set `workspaces` aside, it
    stays, and a workspace's first hold looks at each repository the spec names:
    one there, a git repository, with `remote.origin.url` exactly its remote, as
    `git clone` wrote it (`../protocol/git.status.md`), is treated as cloned. A
    look that finds otherwise makes the workspace again; one that breaks fails
    the prepare as transient, and the next prepare makes it. A look names only
    the spec's repositories, so a directory beside them stays until a make. A
    saved start makes its empty directory as section 5 has it;
  - where section 5 does not trust a clone a killed git may have damaged, and
    section 9 has what a worker left behind fail no prepare, a look cannot tell
    a damaged clone from a whole one. The operation that breaks on it fails that
    prepare as transient and spoils it, and the next prepare makes it again, as
    `temper/tests/worker/checkout/tests/suite/reuse.rs` shows with a stale
    `index.lock`. Damage no operation breaks on stays in use;
  - a workspace's path stays `<root>/workspaces/<token>/<repository>`, its
    token its place in the order workspaces are made, since none is retired: the
    first `0`, the second `4294967296`. A restart does not bring back which
    workstream held which workspace, so after one a path holds whichever
    workstream's first hold came in that place.

  Because a deployment that runs one trusted person's work on a machine it
  builds on needs a workspace to act as a long-lived build directory, so that
  new work rebuilds only what changed (the user, 2026-11-05). It is off by
  default because a run can write its `.git`, and the worker's next git command
  obeys what that says, now in another workstream, with that workstream's
  identity on a fetch or a push. The user's request and
  `../draft/dogfood-hacks.md`, 2.1 name its hooks, which `../protocol/git.md`,
  section 3's settings hold off; those settings do not hold off a filter driver
  or a URL rewrite (`../protocol/git.md`, 10). That is acceptable only for one
  trusted person.

  6: with `TEMPER_DUMP_LLM` in the worker's environment at its start, an agent
  process's environment holds that one variable, an absolute directory, and its
  log's lines also go to `dump.log` there; unset, which is the default and the
  only shipped form, section 6 holds exactly. Because the deployment's
  comparison with another pipeline must read what each run was sent
  (`../draft/dogfood-hacks.md`, 2.13). It carries no credential. No test
  runs the variable from a worker through a real agent process to a file, as
  the request of 2026-11-05 leaves that to the next comparison's run.
