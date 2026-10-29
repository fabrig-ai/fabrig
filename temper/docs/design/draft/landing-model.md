# Landing: one model, the forge's best where it fits

Draft, 2026-10-29, revised the same night as the queue grew. What fabrig's own merge queue (`tools/queue/`, run by
hand on 10-28 and 10-29) taught temper's landing (`../domain/landing.md`),
and how temper's landing should use each forge: the model is temper's,
the same on every forge; a forge's native machinery serves a step only
where its semantics match, and temper does the step itself where it has
none. It records a conversation with the user on 2026-10-29 and the
queue's numbers of that night. It changes `landing.md` and `forge.md`.
The coordinator keeps it current as fabrig's queue changes (the user,
2026-10-29); its decisions stay the user's.

## 1. In one page

- **temper's landing is the contract.** What lands, when, what proves
  it and what a failure means are temper's (`landing.md`: landings,
  prepare, receipts by key, proof by dispatch, the turn), never a
  forge's queue's. A project moving between GitHub and Forgejo, and
  temper's own tests, meet one behaviour.
- **A forge's native queue is an accelerator, never a dependency.**
  Where one exists and its semantics match a step (GitHub's merge queue
  for the turn's check-then-merge), temper may hand it that step for its
  presentation and protection; temper still decides what enters, what
  proves it and how a conflict is settled first. Switched off, nothing
  changes but the forge's display.
- **Keys per package, so selection is receipts** (section 3). A gate's
  key today is the candidate's tree without the paths it ignores; per
  package, a key is that package's input closure (its sources, its
  dependencies' keys, the gate's definition). A change re-proves only
  the packages it reaches, and that is safe by construction: an
  unreached package's key, and so its receipt, is unchanged.
- **Batched landings under contention** (section 4). Landings prove side
  by side and turn first-proved-first; when the turn makes proofs stale
  faster than they finish, ready landings join one candidate, proved
  once, bisected by halves on a failure.
- **A runner's fault is never a red** (`landing.md`, 7.4, already), and
  a starved test is neither (skein's testing-strategy.md, 2.10): a
  receipt records it apart.
- **fabrig lands on temper.** fabrig's queue is this model's first
  implementation, by hand; temper's landing replaces it once these
  sections are built, and the queue's tests become the landing world's.

## 2. What fabrig's queue showed (10-28 22:13Z to 10-29 00:29Z)

- **The cost is builds, not tests:** a full check was 7 to 11 minutes,
  of which tests ran about 41 seconds; building and linking about 1,366
  test executables was the rest, until each package had one (then a
  warm full check took 2.5 to 3.5 minutes).
- **Selection decides throughput:** every batch ran the whole workspace
  while lockfile changes forced it; selecting the packages a change
  reaches (and their reverse dependencies) is what keeps checks short.
- **The queue is the next bottleneck:** busy 84% of the time at 23
  merges an hour, with one designer's leads feeding it.
- **Most red was not the change's:** a full disk, a missing tool on
  PATH, the queue's own wrong flags, a ratchet file both sides lowered,
  a test that ran another package's binary, a starved test. Each was a
  queue rule or a tool fix, none a repair of the change.
- **Mechanical conflicts settle themselves:** a conflict confined to a
  counted baseline (`old-forms.txt`) merged by re-running its writer,
  which refuses a raise; `landing.md`'s prepare by a project's tooling
  (5.3) is the general form.

- **Throughput doubled, then the queue saturated:** 23 merges an hour
  at midnight, 47 an hour by 01:08Z with about eleven implementers at
  once; the queue was then busy 98% of the time and 36 ready branches
  waited. The builders were not the limit: builds waited a median of
  2 to 7 seconds for a slot in the builder's build cap, and averaged
  about two of eight cores. So capacity moved to the queue: a builder
  became the second prover, and batches grew from 8 to 16.
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
  set (one build for the set, as fabrig's batches built once).
- **What a key cannot see** is named by the project and checked: a test
  that reads files by path or runs another package's binary
  (`tools/queue/always.txt` and the binaries a test runs, in fabrig's
  terms) joins the keys it depends on, and a periodic full proof (every
  N landings, or idle) finds a missing one, which the project fixes.

## 4. Batched landings

- **Side by side while proofs keep up:** `landing.md`'s model, unchanged:
  each landing proves at its candidate; the turn fast-forwards the
  first proved; the others' stale proofs keep what still holds (their
  unchanged keys).
- **One candidate when they don't:** a project whose turns make more
  proofs stale than finish (a measure the landing keeps) groups its
  ready landings into one candidate, oldest first, up to a size; proved
  once; on a failure, halves proved again, so the good ones land. A
  merge train is the same with the next group prepared on the current
  one's candidate while it proves.
- **Two provers, one serializer** (fabrig's queue, 10-29): one
  controller prepares candidates and alone lands and reports; two
  provers each check one candidate; batch N+1 is prepared on N's
  candidate and proved while N proves, so its parent is exactly the
  commit N's proof certified. When N fails, every candidate built on it
  is discarded and prepared again, and N is bisected by halves as
  before. The candidate travels to its prover as the exact prepared
  commits, so what lands is what was proved.
- **A failure that is the machine's is never the change's:** a refused
  build (a full disk), a missing tool, a runner's fault: the batch is
  an error, retried, and no branch is rejected.
- **Counts that may only fall are lowered by the landing,** not by the
  changes: the prover rewrites each touched baseline after merging and
  refuses a raise, so no two changes conflict over a counted figure.
- **Fairness stays section 9's:** first proved, first landed; a batch
  is proved when all its members are.

## 5. Each forge, by what it offers

| Step | temper's own (every forge) | GitHub native | Forgejo native |
|---|---|---|---|
| Prepare a candidate | rebase via the forge, or the project's tooling (5.3) | the same | the same |
| Prove | dispatch-only gate workflows, receipts by key (7) | the same; the merge queue's `merge_group` runs would be gate runs temper did not dispatch, so a queue-backed turn reports receipts already made | the same |
| The turn (check base, then land) | read mains, fast-forward each (8): Forgejo by a fast-forward merge, GitHub by a ref update with an `Always` bypass | the merge queue: temper enqueues a proved candidate; its required check is temper's status, already met; GitHub merges the group it built, fast-forward | none: temper's own |
| Show it | the candidate on its pull request, `temper/gate/<name>` statuses (10) | plus the queue's "queued" and position | the same as temper's |
| Batching | section 4 | the queue's groups, sized by GitHub: temper caps them by its own batch | temper's own |

- **GitHub's queue is worth it for its display and its protection**
  (required checks, merge rules a person already trusts); it costs that
  GitHub sizes the groups, that landing goes through pull requests, and
  that its runs are not temper's dispatches. A project chooses it in its
  landing settings; temper's turn stays the default until a project's
  queue-backed turn has been walked on real backends.
- **Forgejo has no merge queue** (its "merge when checks succeed" is a
  scheduled merge, not a queue): temper's turn is the only one.

## 6. Later

- Keys per package for languages other than Rust (the project's tooling
  names a package's inputs).
- More than two provers when one serializer's preparation keeps up;
  a prover added or removed by the measure of proofs waiting.
- Elastic gate capacity: more provers when proofs queue, fewer when they
  idle.

## 7. Open questions

- **A package's key from the build tool or from the project:** cargo's
  metadata gives the Rust graph; other projects name their packages.
- **GitHub's merge queue and dispatch-only gates:** whether a
  queue-backed turn can always report temper's receipts as the queue's
  required check without a run of its own.
- **The batch's size and when to batch:** a fixed size, or from the
  measure of stale proofs (section 4).
