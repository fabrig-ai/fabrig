# Landing: one model, the forge's best where it fits

Draft, 2026-10-29, revision 3 (the second prover and the train). The
north star for temper's landing (`../domain/landing.md`): one model,
generic over forges and projects, expressive enough that fabrig's own
merge queue (`tools/queue/`, run by hand since 10-28) is one of its
configurations, and the forge's native machinery serves a step only
where its semantics match. It records a conversation with the user on
2026-10-29 and the queue's numbers of that night. It changes
`landing.md` and `forge.md` (section 10 lists how). The coordinator
keeps it current as fabrig's queue changes (the user, 2026-10-29); its
decisions stay the user's.

## 1. In one page

- **One model:** ready landings are **prepared** into candidates,
  **proved** on provers into receipts kept by key, and **landed** by one
  serializer per project, fast-forward only, at exactly the commit that
  was proved. Batching, speculation and the number of provers are only
  how many candidates are in flight and where they prove; they never
  change what proves a commit or who lands it.
- **temper's landing is the contract.** What lands, when, what proves
  it and what a failure means are temper's (`landing.md`: landings,
  prepare, receipts by key, proof by dispatch, the turn), never a
  forge's queue's. A project moving between GitHub and Forgejo, and
  temper's own tests, meet one behaviour.
- **Keys per package, so selection is receipts** (section 3). A change
  re-proves only the packages it reaches, and that is safe by
  construction: an unreached package's key, and so its receipt, is
  unchanged. Receipts outlive candidates: a discarded candidate's
  receipts stay good for their keys.
- **Candidates carry landings: one, a batch, or a train** (section 4).
  Under contention ready landings join one candidate, proved once and
  bisected by halves on a red; with more than one prover, candidate N+1
  is prepared on candidate N and proved while N proves, and lands only
  on exactly the parent its proof certified.
- **Provers are machines with exclusive, warm use** (section 5): a
  forge's runner or the project's own, reached by the same dispatch;
  each proves one candidate at a time and answers with a certificate.
- **Every outcome is classed before it is anyone's** (section 6): a
  red is the change's, a conflict its resolution's, a fault the
  prover's (retried, nobody rejected), a starved test neither, a stale
  base the landing's to prepare again.
- **One serializer per project** (section 7): the only writer of main,
  its decisions persisted before their effects, its effects at least
  once under stable ids, its recovery decided by reading main.
- **A forge's native queue is an accelerator, never a dependency**
  (section 8). Switched off, nothing changes but the forge's display.
- **fabrig lands on temper.** fabrig's queue is this model run by hand;
  section 9 maps each of its operations onto the model, the test of
  the model's reach. temper's landing replaces the queue once
  `landing.md` grows what section 10 lists, and the queue's tests
  become the landing world's.

## 2. What fabrig's queue showed (10-28 22:13Z to 10-29 02:10Z)

- **The cost is builds, not tests:** a full check was 7 to 11 minutes,
  of which tests ran about 41 seconds; building and linking about 1,366
  test executables was the rest, until each package had one (then a
  warm full check took 2.5 to 3.5 minutes).
- **Selection decides throughput:** every batch ran the whole workspace
  while lockfile changes forced it; selecting the packages a change
  reaches (and their reverse dependencies) is what keeps checks short.
- **Most red was not the change's:** a full disk, a missing tool on
  PATH, the queue's own wrong flags, a ratchet file both sides lowered,
  a test that ran another package's binary, a starved test, a cleanup
  race in the test harness. Each was a queue rule or a tool fix, none a
  repair of the change.
- **Mechanical conflicts settle themselves:** a conflict confined to a
  counted baseline (`old-forms.txt`) merged by re-running its writer,
  which refuses a raise; `landing.md`'s prepare by a project's tooling
  (5.3) is the general form.
- **Throughput doubled, then the queue saturated:** 23 merges an hour
  at midnight, 47 an hour by 01:08Z with about eleven implementers at
  once, 41 to 47 an hour after, with about seventeen; the queue was
  busy 96 to 98% of the time and the ready branches waiting grew from
  36 (01:21Z) to 58 (02:10Z). The builders were not the limit: builds
  waited a median of 2 to 7 seconds for a slot in the builder's build
  cap, and averaged about two of eight cores. So capacity moved to the
  queue: batches grew from 8 to 16 (01:34Z), and a builder was drained
  to become the second prover (01:30Z to 01:56Z), which proves nothing
  until the train lands.
- **A red costs a big batch most:** one member's lint failure in a
  batch of 16 took five bisection rounds before the good members
  landed. Bisection keeps the good members, but every round is a full
  proof; the train (4.3) overlaps proofs, it does not shorten them.
- **The checker's own rules break it as the tree changes:** a selector
  that names a test binary fails the whole run the day the last such
  binary goes (nextest refuses a `binary()` matcher that matches
  nothing); selection by test names that a refactor cannot empty
  stays valid.
- **Starved is not the only congestion:** a CPU-bound test took 2.5
  times its idle CPU on the loaded prover with no time spent waiting
  for a CPU (shared cores, cache, clock), which no runnable-wait rule
  sees. The remedy was the testing pyramid's own: the costly story
  moved down a tier, where it belongs (`../domain/testing.md`, 2.4).

## 3. Keys per package

- **A gate per package, or a gate whose key is per package:** a test
  gate's key for a package is a digest of the package's own inputs
  (its directory's tracked files but the paths the gate ignores), its
  dependencies' keys, the toolchain, the gate's definition and the
  workspace's shared inputs (the lockfile entries it resolves, the
  profiles). A package's key changes exactly when something its build
  or tests read changes.
- **A candidate is proved** when every package's key has a passed
  receipt; only packages with no receipt run, as one dispatch with the
  set (one build for the set, as fabrig's batches build once).
- **Receipts outlive candidates:** a receipt is a key's, not a
  candidate's. A candidate discarded (a bisection half, a train's
  successor invalidated, a stale base) leaves its receipts good for
  every key they name, so its next preparation runs only the keys the
  change of parent reached.
- **What a key cannot see** is named by the project and checked: a test
  that reads files by path or runs another package's binary
  (`tools/queue/always.txt` and the binaries a test runs, in fabrig's
  terms) joins the keys it depends on, and a periodic full proof (every
  N landings, hourly, or idle) finds a missing one: a full proof that
  fails where the selections before it passed is the project's key gap,
  reported to its operator, never a member's red.
- **Paths no gate reads** (documentation outside every package, a
  gate's ignored paths) reach no key: a landing of them is proved on
  receipts alone (`landing.md`, 7.2), as fabrig's docs-only batches
  run no build.

## 4. Candidates: one landing, a batch, a train

### 4.1 A candidate

- **A parent and an ordered list of landings,** each pinned at its
  members' heads, prepared once into exact commits: per repository, a
  commit whose history holds the parent. Its identity is the parent,
  the pinned heads and the prepare's tooling at its revision.
- **What is proved is what lands:** the prepare's commits are made
  once and travel to the prover and to the turn as they are; nothing
  is merged, rebased or amended again after proof.
- **Pinned heads:** a landing joins a candidate at the heads it had
  when selected. A head that moves later enters only a later
  candidate; a member is in at most one live candidate.
- **The prepare's tooling may write** what the project's rules make
  from the members (regenerated files, lowered counted baselines that
  refuse a raise) as commits of the candidate's own, named for what
  they are; no member's branch carries them.
- **Merge or rebase** (`landing.md`, 17): a candidate of several
  landings is best made by merging each member's head onto the parent
  in order (fabrig: `Merge ready/<lead>/<task>`), which forces no push
  and keeps each member's commits as they were; a landing of one may
  keep the forge's rebase.

### 4.2 Batches

- **Side by side while proofs keep up:** `landing.md`'s model,
  unchanged: each landing proves at its candidate; the turn
  fast-forwards the first proved; the others' stale proofs keep what
  still holds (their unchanged keys).
- **One candidate when they don't:** a project whose turns make more
  proofs stale than finish, or whose ready landings wait longer than a
  proof (measures the landing keeps), groups ready landings into one
  candidate, oldest ready first, up to a batch size (fabrig: 16).
- **A red batch is bisected by halves,** the left half first, each half
  prepared again from the parent it will land on (after a green left
  half, on that half's candidate), recursively, until each red is one
  landing's: that landing's red is its own (a repair), and every green
  member lands. A conflict in preparing a batch leaves that member out
  and the rest goes on.
- **Members stay their own landings:** a batch is a grouping of the
  serializer's, not a landing; each member keeps its own task, news and
  outcome, so one member's red rejects only that member.
- **The size is measured, not fixed:** larger batches land more per
  proof while most are green and cost more rounds per red (section 2);
  the serializer keeps the red rate and the rounds spent bisecting, and
  the batch size follows them.

### 4.3 Trains

- **Depth follows the provers:** with N provers, up to N candidates are
  in flight, each prepared on the one ahead of it: candidate k+1's
  parent is candidate k's exact commit, not main's tip.
- **Proved out of order, landed in order:** a successor whose proof
  ends first waits, with no effect: no landing, no rejection, no
  report. The serializer lands only the train's head, once main equals
  that head's parent.
- **An ancestor's failure discards its successors' candidates, not
  their receipts:** a red, a conflict, a fault or a recovery doubt at
  candidate k invalidates every candidate prepared on it; their
  results, arriving later, are ignored. Their landings return to the
  ready set unchanged and are prepared again on the new parent, where
  their unchanged keys need no run (section 3).
- **No speculation past a doubt:** while candidate k is bisected, or
  its preparation left a member out, nothing is prepared on it; the
  train refills once its outcome is known.
- **Landing a successor needs its whole chain:** a successor lands only
  after every candidate ahead of it has landed, at exactly the parent
  its proof certified; a successor's green never lands an ancestor.

### 4.4 Fairness

- **First ready, first prepared; first proved, first landed**
  (`landing.md`, 9): within a train, landing order is the train's
  order, which is preparation order. A batch is proved when all its
  members are.
- **The reservation** (`landing.md`, 9) holds across batches: a landing
  whose proof went stale twice is prepared at the head of the next
  candidate for its repositories.

## 5. Provers

- **A prover runs one candidate's gates at a time, with the machine to
  itself:** a gate is a measurement, and a congested prover makes tests
  starved or slow that are neither (`../domain/testing.md`; skein's
  `testing-strategy.md`, 2.10). Nothing else builds there.
- **Warm by affinity:** a prover keeps its own checkout and build cache
  across candidates (fabrig: one target per gate machine, never shared
  across machines), so a candidate goes to the prover whose cache it
  reuses most, where several are free.
- **A forge's runner or the project's own,** reached the same way: by
  dispatch (`landing.md`, 7.1). A project's own prover is a
  self-hosted runner with a label of its own, one per machine. A
  candidate reaches it as a commit on the forge, on a branch of
  temper's (`temper/<root>/c<n>`, as pin-only members' branches are
  temper's: `landing.md`, 4.1), deleted when the candidate is decided;
  fabrig's train, whose serializer alone talks to the forge, sends the
  exact commits as a bundle instead.
- **A prover answers with a certificate:** the candidate and its
  parent, each gate's verdict per key with its log, the prover, and the
  identity of the tooling and selection lists it ran with. The
  serializer accepts only a complete certificate matching the
  candidate it dispatched; anything else is a fault.
- **Capacity is the prover's to check:** a prover below its disk floor,
  missing a tool, or unable to fetch the candidate refuses the
  candidate as a fault (section 6) before it runs a gate, and keeps its
  own cache within its disk (fabrig: stale test executables trimmed
  after each batch).
- **How many provers** is a project's setting, measured by the share
  of time proofs run, the ready landings waiting, the candidates
  invalidated and the serializer's preparation lag; a second prover is
  worth it while the first is busy most of the hour with landings
  waiting.

## 6. Outcomes

| Outcome | What it is | What follows | fabrig's cases |
|---|---|---|---|
| green | every gate passes at the candidate | the turn (section 7) | a batch landed |
| red | a gate's own step failed | a batch is bisected; one landing's red is a repair of the failing member (`landing.md`, 7.3), its brief the failing step and the log | clippy, a test, a raised count |
| conflict | the prepare could not settle a member | conflicts the project's rules know are settled by its tooling; any other leaves that member out for a resolution (`landing.md`, 3) | `old-forms.txt` recounted; a rebase task |
| fault | the prover's, not the change's: set-up failed, a refusal for capacity, a missing tool, a transport or certificate failure, a crash | retried with nobody rejected and no member marked tried, up to `gate_retries`; past it the project's operator is told and the landing held (`landing.md`, 7.4) | the disk floor; a queue restarted without its tools |
| starved | a test that the prover starved of time it was owed | neither a pass nor a fail: recorded apart in the receipt, the gate's verdict unchanged by it (`testing-strategy.md`, 2.10) | real-loop judges under load |
| stale | the candidate's parent is no longer main's tip, or an ancestor in the train failed | prepared again; unchanged keys keep their receipts | a train's successor; a main moved by hand |
| key gap | a full proof fails where selections passed | reported to the project's operator; the missing input joins its key | `always.txt`, `read-by-tests.txt` |

- **A flake is a red until shown otherwise:** a failure whose tests
  all match open flake findings is rerun once (`landing.md`, 6.2); a
  failure the change cannot reach, passing alone, is a finding, never a
  silent rerun.

## 7. The serializer and the turn

- **One per project, the only writer of main:** `landing.md`'s turn,
  held by one serializer that alone prepares candidates, dispatches
  them, adjudicates certificates, lands and reports. Provers never
  write the forge's refs or report.
- **It lands the train's head only,** after reading main afresh: main
  must equal the certified parent, then main is fast-forwarded to
  exactly the certified commit. No force, no merge at the turn; a
  fast-forward check alone is not enough, since main might be another
  ancestor of the candidate.
- **Decisions before effects:** each decision (a landing, a rejection,
  a bisection step, an invalidation) is persisted, versioned, before
  its effects, and the effects (posts, statuses, the members' branches'
  release) go out at least once under stable ids, so a reader drops a
  repeat; a failed report never changes an outcome. A branch is
  released only at the head it landed, never a head pushed since.
- **Recovery reads main:** after a crash, main at the candidate means
  the landing happened, to be finished; main at its parent means it did
  not, to be retried under the same decision; anything else is stale.
  A prover's result for an invalidated candidate is ignored.
- **Draining:** a stop admits no new candidate, lets proofs in flight
  finish or discards them, and lands nothing past an unresolved
  outcome; restarting resumes from the persisted state, with the
  settings it had.
- **Nothing runs inside the turn:** no gate, no prepare (`landing.md`,
  8). The turn is seconds; the train is how proofs overlap.

## 8. Each forge, by what it offers

| Step | temper's own (every forge) | GitHub native | Forgejo native |
|---|---|---|---|
| Prepare a candidate | the forge's rebase for a landing of one; a merge chain on a branch of temper's for a batch or a train; the project's tooling (5.3) | the same | the same |
| Prove | dispatch-only gate workflows, receipts by key (7); a project's own provers as self-hosted runners | the same; the merge queue's `merge_group` runs would be gate runs temper did not dispatch, so a queue-backed turn reports receipts already made | the same |
| Speculate (a train) | temper's own: candidates on temper's branches, landed in order | the merge queue builds each group on the groups ahead of it: a train of GitHub's sizing | none: temper's own |
| The turn (check base, then land) | read mains, fast-forward each (8): Forgejo by a fast-forward merge, GitHub by a ref update with an `Always` bypass | the merge queue: temper enqueues a proved candidate; its required check is temper's status, already met; GitHub merges the group it built, fast-forward | none: temper's own |
| Show it | the candidate on its pull request, `temper/gate/<name>` statuses (10) | plus the queue's "queued" and position | the same as temper's |
| Batching | section 4 | the queue's groups, sized by GitHub: temper caps them by its own batch | temper's own |

- **GitHub's queue is worth it for its display and its protection**
  (required checks, merge rules a person already trusts); it costs that
  GitHub sizes the groups and trains, that landing goes through pull
  requests, and that its runs are not temper's dispatches. A project
  chooses it in its landing settings; temper's turn stays the default
  until a project's queue-backed turn has been walked on real backends.
- **Forgejo has no merge queue** (its "merge when checks succeed" is a
  scheduled merge, not a queue): temper's turn is the only one.

## 9. fabrig's queue in the model's terms

The test of the model's reach: each operation fabrig's queue runs today
(`tools/queue/design.md`, and the train design approved 10-29) has its
term here, and `landing.md` has, or section 10 asks for, its mechanism.

| fabrig's queue | The model | Where |
|---|---|---|
| a lead pushes `ready/<lead>/<task>` | a change becomes ready; its landing of one | `landing.md`, 3-4 |
| `leads.txt`: a branch's lead to a post address | the member's goal hears its landing's news | `landing.md`, 4.2 |
| tips tried, pinned at selection | a landing pinned at its heads | 4.1 |
| batch of 16, oldest first | a batch | 4.2 |
| `git merge --no-ff -m "Merge <branch>"` per member | prepare by a merge chain | 4.1 |
| `old_forms.py --write` lowers baselines; ratchet-only conflicts recounted | the project's prepare tooling, its candidate's own commits; conflicts its rules know | 4.1; `landing.md`, 5.3 |
| selection: owners, reverse dependencies, lockfile entries, `always.txt`, `read-by-tests.txt` | keys per package; what a key cannot see | 3 |
| docs-only batches skip cargo; tools-only batches run the tools' suites | paths no gate reads; a gate's ignored paths | 3; `landing.md`, 6.1 |
| steps: ratchets, tools, fmt, clippy, build, focused, fuzzy | gates of the repository, each with its key | 3; section 12 |
| `heavy --gate`: the gate machine builds nothing else | a prover's exclusive use | 5 |
| a full run every 5 batches, hourly, or idle | the periodic full proof; the baseline at main's tip | 3; `landing.md`, 6.2 |
| a red batch halved, left before right | bisection | 4.2 |
| a rejection posted with its step and log | a red receipt; the member's repair brief | 6; `landing.md`, 7.3 |
| below the disk floor; an exception; a missing tool | faults, retried, nobody rejected | 6 |
| a full run red where selections passed | a key gap, to the operator | 3; 6 |
| test executables trimmed after a batch | the prover's own housekeeping | 5 |
| `queue.stop`, `tools/queue/restart` | draining and restart | 7 |
| `queue.state.json`, `queue.jsonl` | the serializer's persisted state; its audit stream | 7; `landing.md`, 12 |
| `git push origin HEAD:main` | the turn's fast-forward | 7 |
| the ready branch deleted after landing | the member's branch released at the head it landed | 7 |
| the train: one controller, two gate machines, bundles, certificates, invalidation | a train of depth two; provers; certificates; the serializer | 4.3; 5; 7 |
| batch size 8 then 16; hetzner2 drained to prove | settings measured: batch size, provers | 4.2; 5 |

## 10. What this asks of `landing.md` and `forge.md`

- **Batches** (`landing.md`, 1, 4, 9): a candidate carrying several
  landings, each still its own task; bisection by halves; the batch
  size a setting with its measures.
- **Trains** (`landing.md`, 8, 15): a candidate prepared on a proving
  candidate; landing only the train's head on its certified parent;
  invalidation of successors; recovery by reading main.
- **Keys per package** (`landing.md`, 6.1): a gate's key per package,
  beside today's filtered tree, from the project's key command or the
  build tool's graph; receipts reused across discarded candidates.
- **Provers** (`landing.md`, 7): a project's own provers as labelled
  runners with affinity; candidates on temper's branches; certificates;
  faults the prover raises before a gate runs.
- **Starved** (`landing.md`, 6.2, 7): a receipt records starved tests
  apart from passes and failures.
- **The serializer's effects** (`landing.md`, 11, 12): persisted
  decisions, at-least-once effects under stable ids, release at the
  landed head.
- **Settings** (`landing.md`, 13.1): `batch` (size, or by measure),
  `provers` and `train_depth`, `full_proofs` (every N landings, hourly,
  idle), `prepare` merge or rebase for batches.
- **Stories** (`landing.md`, 16): a batch with one red lands the rest;
  a train whose head fails discards its successor and re-proves only
  the keys that changed; a successor proved first waits; a prover's
  fault rejects nobody; a restart mid-train lands nothing twice;
  fabrig's queue tests, moved into the landing world.
- **`forge.md`:** the merge queue's speculation (section 8) where a
  project chooses GitHub's queue.

## 11. Later

- Keys per package for languages other than Rust (the project's tooling
  names a package's inputs).
- Trains deeper than two when one serializer's preparation keeps up.
- Elastic provers: more when proofs queue, fewer when they idle, by
  the measures of section 5.
- Partitioned full proofs: one full proof split across provers.

## 12. Open questions

- **A gate per step or one gate of steps:** fabrig's steps (fmt,
  clippy, build, focused, fuzzy) as gates of their own, each with a
  receipt (a lint red then re-proves nothing else), or one gate whose
  receipt names its failing step. Recommended: a gate per step whose
  result a key can carry.
- **A package's key from the build tool or from the project:** cargo's
  metadata gives the Rust graph; other projects name their packages.
- **Where a speculative candidate lives:** on a branch of temper's at
  the forge (visible, dispatchable, an extra ref per candidate) or
  only on the provers (fabrig's bundles). Recommended: temper's
  branches, since dispatch needs the commit on the forge.
- **The batch's size and when to batch:** a fixed size, or from the
  measures of section 4.2.
- **GitHub's merge queue and dispatch-only gates:** whether a
  queue-backed turn can always report temper's receipts as the queue's
  required check without a run of its own.
