# merge-job-runs-2: a change's merge job runs on a remote worker

The request (`inputs/request.md`): every merge job on a real deployment
ended held `out of tries: transient failures`, with no line in either
log. It asks, ranked: the failing story at the lowest tier holding the
engine, a remote worker and the merge job, four rows; the fix, each
failure a line on the side that failed; the design saying so (`worker.md`
4.4, `forge.md` 8.4). Four tasks at most.

## What the cut found

Read, not run: the deployment's evidence is outside the repository.

- **The likeliest fault:** a merge with more files in conflict than the
  checkout's `conflicts` (4, both profiles; `temper-service/src/limits.rs`)
  makes `git diff-files` list more paths than the git layer admits, which
  it reads as `Broken` (`temper-worker-protocol-git/src/output.rs`,
  `paths`; git.md 4.6 said so). The preparation fails as transient, every
  try alike, so the job is held out of tries and a release changes
  nothing: the request's three symptoms. fabrig's merges of main into a
  goal's branch conflict in several baselines and status files at once.
  Not proven on the deployment; the story's `PastTheBound` row shows it
  in the composed world.
- **The candidates the request names, by reading:** the worker declares
  kind 3 with spec 4,096, result 512, 600 s (`worker.rs`, `worker_root`;
  its test holds it), the remote worker included (one constructor); a host
  without the kind is never placed the job, so (a) would leave jobs
  queued, not failed. The base is fetched by its id, whatever branch holds
  it (`hold.rs`, `FetchBase`); git asks protocol 2, where a server gives
  any commit it has, so (b) fails only on a server that falls back to
  protocol 0, and then as an unparsed error, `Broken`, transient: possible,
  unproven, and the story's `OtherBranch` row pins the fetch by id. The
  command's path resolves to the checkout's `<name>/.temper/merge` (c);
  the spec and result are within their bounds, and a spec past its bound
  makes no job (d). The story's common checks pin all four.
- **Why nothing was logged:** the engine's 4.3 line skips a job's answer
  (`temper-engine-domain/src/link.rs`, `answer`), the worker writes none,
  and a preparation's failure reaches the hub with an empty detail. The
  hold's way out says the engine's log names each failure: for jobs it
  named none.

## Design written by the cut

- `temper/docs/design/domain/worker.md`, 4.4: a merge job prepares
  whatever the count of conflicts; a job's failure is a line on the worker
  and on the engine, their forms, the why. Section 5, *A merge in
  progress*: past `conflicts`, the first ones named, all marked.
- `temper/docs/design/protocol/git.md`, 4.6: the listing past the bound
  keeps its first `conflicts` paths; section 6, the one stdout read as a
  kept prefix; section 10, a merge's commit past the bound (a resolution
  settling more than `conflicts` files still cannot deliver: left open).
- `temper/docs/design/domain/forge.md`, 8.4: a merge job's failures are
  lines, and its hold's way out points at them.
- `temper/docs/design/domain/testing.md`, 2.1: the merge job's story.
- Status: `worker.status.md` (4.4 lines; 4.4, 5 conflicts),
  `git.status.md` (4.6, 6), `forge.status.md` (8.4), `testing.status.md`
  (2.1).

## Parts and tasks

| Task | Part | Needs | What |
|---|---|---|---|
| `story` | story | | the composed world runs a merge job; rows Settled, Left; production's kind, job translation and payload limits on the way |
| `conflicts` | conflicts | | the git layer's listing past the bound keeps the first `conflicts`; the fake's `perform_within` |
| `lines` | lines | | the worker's and the engine's lines, the why's path |
| `story-rows` | story | `story`, `conflicts` | rows Clean (a catch-up), OtherBranch, PastTheBound |

Three start at once. Critical path: `story` (180) then `story-rows`
(150), about 5.5 hours of implementer time; `conflicts` (120) lands
before `story-rows` needs it.

## Decisions

- **The tier:** the composed system world (`temper/tests/system`), the
  lowest that runs the real engine root and the real worker root
  together (testing.md 2.1). Its link plays the protocol layer's
  translation without bytes; a socket worker and the in-process one share
  the same job code above the bytes, so the world holds what the remote
  worker runs. The real loop's cluster would add sockets and real files,
  but it keeps one path per feature (testing.md 2.4) and its fake git
  checks out no executable command.
- **Failing first** sits inside each fixing task (`conflicts`; a fault a
  story row shows), so the trunk stays green between merges.
- **The fault's fix changes no figure:** what happens past the bound,
  never the bound (the request's *Not in this plan*).
- **Both sides log:** the worker's line holds what only it saw (a push's
  diagnostic, a deadline: jig's answer carries no detail for them); the
  engine's keeps 4.3's form so `temper help hold`'s way out is true for
  jobs too, without a help edit.
- **The hold's words:** a job's own hold stores its failures' class
  alone; words in it need a stored-format change, already worker.md 12's
  question. Asked again as `[[question]] hold-words`; no task waits.

## Overlaps

- `story` and `lines` both edit `temper-service/src/worker.rs`: `story`
  only `worker_root`'s kind list (line ~1172), `lines` only `job_answer`
  (line ~607) and what it reads.
- `conflicts` and `lines` both edit `temper/tests/worker/checkout/src/merges.rs`:
  `conflicts` adds a `Case`, `lines` adds a field or `..` to
  `Prepared::Failed` matches.
- The drafts beside this one (reference-walks, v0.1.0-ladder,
  configuring-walks, post-landing-probes, main-reds) are queued, with no
  plan on main: no overlap known.

## Dropped

Nothing of the request. Left open, named in `git.md` 10: a resolution that
settles more files than `conflicts` cannot deliver, since its commit's
scan must read every unmerged path.
