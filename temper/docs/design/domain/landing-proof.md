# Landing keys, receipts and proof

This document holds landing keys, receipts and proof, split from [`landing.md`](landing.md).

## 1. Keys and receipts

### 1.1 A key

- **What a gate's result depends on,** as 32 bytes: SHA-256 of a
  canonical encoding, the gate (its repository and name), the gate's
  definition (its setting's value, `landing-settings.md`, section 1: its workflow, its
  ignored paths, whether it blocks, whether it reads the base), the
  candidate's tree filtered by the gate's ignored paths, and, for a gate
  that reads the base, the base's tree filtered the same way.
- **The filtered tree:** the entries of the commit's root tree,
  ordered by their full paths from the root, byte by byte (not git's
  tree order), where an entry is listed whole unless an ignored path
  lies at or below it: an ignored entry is left out, and a directory
  above one is replaced by its own entries, recursively. An ignored
  path is a path prefix of whole segments (`docs`,
  `crates/x/README.md`), never a pattern.
- **The encoding,** each number big-endian: the version, one byte, 1
  for a gate that does not read the base and 2 for one that does; the
  repository's forge and id, eight bytes each; the gate's name and its
  definition's bytes, each after its length in two bytes; for version 2,
  the base's filtered tree's entry count in four bytes and its entries;
  then each entry of the candidate's filtered tree: its path's length in
  two bytes and its path from the root, its mode as four bytes, and its
  object id as 32 bytes, as skein's vocabulary carries every id, a SHA-1
  one padded. A definition that does not read the base encodes as it
  did before the base was a member, so its keys are unchanged.
- **Read from the forge:** the top reads the trees it needs (the root,
  and each directory above an ignored path) with skein's `Read::Tree`,
  and the landing computes the key.
- **A project's `key` command** gives its own instead: in its prepare
  job's result when the project names a prepare command too, and
  otherwise by a key job (`worker/jobs.md`) per repository and gate at
  the candidate, delegated by the landing; the engine reads only the
  job's result.
- **A judgement gate's key** is not a tree: it is the gate and the head
  its verdict was given at, carried over clean merges (`landing.md`, section 3).

### 1.2 A receipt

- **A gate's result under a key:** passed or failed, the run (the
  forge's run, its number and its link), the candidate it ran at, and
  when it ended; a failed one, the round's base too and each failed
  job's id, attempt and name, at most 16, and its report's ids, each
  with whether its name matched the report's prefix, and the report's
  two labels as they stood at the run (`landing-search-by-test.md`, 2).
- **Made only from a run temper dispatched** for that key and saw end
  at exactly that candidate. A status someone else wrote, or a run
  temper did not dispatch, is never a receipt.
- **Kept per project,** in the family `Receipt` (`../protocol/store.md`,
  section 2), and reused across landings, rounds and workers. A gate
  keeps its newest receipts up to `receipts_per_gate`, oldest first
  out, but never the receipt of the key at its repository's main tip,
  which stays while it is main's.
- **A baseline at main's tip.** When a repository first lands by turn,
  or a gate is added or changed, and the key at main's tip has no
  receipt, the gate is dispatched once at main's tip, on a branch of
  temper's own made at that commit, `temper/gate/<gate>.<n>`, as a
  tier's are (`after-landing-runs.md`, section 1), so
  that a landing whose keys equal main's (a documentation landing) has a
  receipt to land on.
- **A failed receipt** makes a key that fails at once, with its run's
  output, until the tree changes or a change's regate supersedes it
  (`landing-regate.md`, 3). A failure whose tests all match open flake
  findings is rerun once first (`landing-regate.md`, 2), unless its
  project's checks can't flake (`forge/judgement.md`, 4).
- **Only a refusal is a failed receipt** (2.3). A retained failed
  receipt whose gate names a report, and whose report was read with no
  failed test or not read, names nothing: it is judged again before it's
  reused (`forge/judgement.md`, 5), dropped from the project's receipts,
  and its key is dispatched as one with no receipt.

## 2. Proof

### 2.1 Dispatch

- **Each gate workflow runs only when dispatched** (skein's
  `Write::Dispatch`), never on a push or a pull request event:
  adoption says when a gate's workflow names another trigger (`landing-settings.md`, 1.2).
- **A train's gates:** a repository's test gates (`landing-settings.md`, 1.1) prove the
  trains into its main. A train into any other branch, a goal's, has
  no test gate: each delivery's own check stands for it, and its
  landing is proved by its judgement gates alone.
- **For each of its train's test gates,** at the candidate, the
  round's top, and at its search's probes, each blocking gate at a
  probed prefix on that member's branch (`landing-search.md`, 3): a key with
  a passed receipt holds; one with a failed receipt fails (2.3); one in
  flight for another landing is waited for; otherwise the landing
  dispatches the gate's workflow on the top's branch, whose head is
  the candidate, with the inputs `temper_commit` (the candidate) and
  `temper_key` (the key in hexadecimal), and, for a gate that reads the
  base, `temper_base` (the round's base). A gate that does not read the
  base is given no `temper_base`.
- **At a search's probe** each dispatch also carries `temper_probe`,
  `yes`, so that the workflow may stop at its first red, and, by test,
  `temper_tests`; the round's top neither (`landing-search.md`, 3;
  `landing-search-by-test.md`, 4).
- **The run is found and followed:** among the runs of that workflow
  at the candidate (skein's `Read::Runs`), one made by a dispatch,
  triggered by temper's own user on that forge, after the dispatch's
  start; then read as it goes (`Read::Run`), on the job deliveries'
  hints where they come and on the landing's own polling of its runs in
  flight: a dispatch-only job publishes no commit status on its head
  (skein's `forge/forgejo.md`, section 2), so no status read wakes it. Its polls
  are due at the client's `poll_at` for the run's repository, every
  `poll` or, while that repository hears, at its last hint plus
  `fallback` (`forge/keeping-up.md`, 2). A
  run another user triggered is never taken, and one dispatch of a
  workflow at a commit is in flight at a time, keeping its inputs
  (`landing-search-by-test.md`, 5). A run whose
  commit is not the candidate is a fault (2.4): the branch moved under
  it. A run an ended round dispatched is followed to its end for its
  key's receipt, and acts in a later round only through that receipt,
  where its key is that round's (`landing-search.md`, section 6).
  A dispatch's or rerun's entry leaves the outbox at its outcome
  (`forge/room.md`, 2): the landing's row keeps what following needs,
  the dispatch, its entry's start and the run's mark.
- **Bounded:** a run in flight that has run longer than the project's
  `gate_timeout` holds the landing (`landing.md`, 4.2), counted by a change's
  execution clock (`forge/change/stalls.md`, 2.2) from its dispatch or
  re-run: the run follower (`runs::Follow`) reads the run each poll, its
  time waiting or blocked for a runner counts nothing, the clock starts
  at the first reading that sees it running, and the run's mark on the
  landing's row keeps it across a restart. A run waiting past
  `forge.change_policy.runner_wait_ms` holds it too. Before the hold, the run
  is read afresh (`Read::Run`), and each of its jobs not completed by
  skein's `Read::JobState` (skein's `forge/job-state.md`), only then,
  so the hold names why each waits; the words live in the hold alone.
- **Its status:** `temper/gate/<name>` on the candidate, pending once
  dispatched, then the receipt's verdict (`landing.md`, section 10).

### 2.2 Proved

- **When the round's blocking gates hold:** its test gates by receipt
  at exactly its proved prefix, its candidate or the green prefix its
  search held (`landing-search.md`, 5), and each of that prefix's
  members' judgement gates by their verdicts at its own prefix (`landing.md`, section 3). An advisory gate is dispatched,
  read and reported, and holds nothing.
- **A landing whose keys all have passed receipts** (documentation,
  when every gate ignores it) is proved with no run.
- **A train with no test gate** is proved by its judgement gates alone,
  and its repository's adoption says so for its main.
- **One proof or probe at a time per train,** since a train has one landing;
  trains prove side by side, and a key in flight is dispatched once for
  all of them.

### 2.3 A real failure

- **A run whose workflow's own step failed, by its gate's own verdict,**
  makes a failed receipt, read with each failed job's failed step and
  the end of its log (skein's `Read::Job`) as the landing makes it, and,
  where its gate names a report (`landing-settings.md`, 1.1), its failed
  tests, read as a tier's (`after-landing-runs.md`, 3). The gate's
  verdict is its report, judged as `forge/judgement.md`, 2 says: a
  report naming failed tests, or more than it carries, refuses; one its
  run never uploaded, one there but not read, and one naming no failed
  test name nothing, and are the runner's or broke (2.4), never a
  receipt. A gate naming none is judged by its failed step (the
  requester, 2026-11-19). A red job stands beside a runner's fault in
  another. A gate's report is read before any failed job's step and log,
  which are read only for a refusal.
- **A round of one unit:** its changes are repaired
  (`forge/change/repairs.md`), its members leave (`landing.md`, 4.3), and the landing ends with nothing landed. A
  repaired head waits for a later landing, whose keys that did not
  change hold by their receipts.
- **A round of several units is narrowed by prefixes**
  (`landing-search.md`): the same gates at the unit boundaries find a
  green prefix and the red unit above it, whose changes are repaired and
  leave; the green prefix lands in the round's turn, the units above it
  in the next round.

### 2.4 A fault of the runner

- **The runner's, not the change's:** a job whose set-up step failed, or
  whose completion step alone failed, or that failed with no step
  failed; a run or a job cancelled by no one of temper's; a run never
  found, or found at another commit; a dispatch refused for a missing
  workflow or runner; and a report that names nothing (2.3) from a run
  so interrupted. Never read from a log's words
  (`../draft/ci-on-forgejo-17.md`, 2.1).
- **Broke, not the change's either:** a report that names nothing from a
  run that wasn't interrupted, missing, not read as its format, or
  naming no failed test (`forge/judgement.md`, 2). It is judged no
  differently from here on, but for its words.
- **Retried, never repaired or searched:** the failed jobs rerun (skein's
  `Write::Rerun`, naming each job's attempt, so that a lost rerun is
  found by the job's next), or the gate
  dispatched again, up to the project's `gate_retries` per key: a run
  cancelled, or never found, is dispatched afresh, never rerun. No
  receipt is made.
- **Past its bound** the landing is held, saying which gate and why
  (`landing.md`, 4.2): the runner's fact and the job it was in, or that
  the gate's check broke, what its report showed, and that the change
  wasn't judged. A blocking gate holds; an advisory one holds nothing
  (2.2).

## 3. Open questions

None.
