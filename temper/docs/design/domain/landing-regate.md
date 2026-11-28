# A landing's gate run once more

Provisional, 2026-11-19; its regate 2026-11-26. When a landing's gate,
red at a key, is run once more at the same content rather than
repaired: a red made only of known flakes, rerun before its receipt is
published (section 2), and a failed receipt whose repair changed
nothing, superseded at once by one fresh run (section 3). It
carries `../draft/archive/flake-regate.md` on the landing of `landing.md`,
whose words it uses: keys and receipts (`landing-proof.md`, 1), proof by dispatch
(`landing-proof.md`, 2.1), a real failure (`landing-proof.md`, 2.3) and
a fault of the runner (`landing-proof.md`, 2.4); flake
findings are `findings.md`'s, and a repair, its result and its release
`forge/change/repairs.md`'s.

## 1. In one page

- **A flake rerun** (section 2): a gate run that failed where every
  test its report names failed is named by an open flake finding is
  run once more, its failed jobs again, before any receipt is made.
  Only that attempt's verdict makes the receipt; green counts the
  flakes, red is a real failure as ever. Once per key per landing,
  kept on the landing's row, so a restart neither repeats nor loses it.
- **A repair's flake verdict** (`forge/change/repairs.md`, *A landing
  repair's verdicts*): a landing repair whose failure's tests are known
  may end with a verdict saying every failure is a flake.
- **A repair's no-test verdict** (the same): a landing repair whose
  failure names no failed test comes only from a gate that promises no
  report. A promised report that was not read or names none asks no
  repair (`forge/judgement.md`, 2). The repair may end with verdict 2,
  saying no test failed and nothing needs changing: a red no test caused
  is run once more, never repaired again for nothing (the requester,
  2026-11-19).
- **A superseded receipt** (section 3): a landing repair that leaves
  its change's head unchanged, with either verdict or none
  (`forge/change/repairs.md`, *A repair that changes nothing is checked
  once more*), makes the change name the failed receipt, by its key and
  its run, on its row, and go on as at an unchanged head, with no
  person's release. A landing that meets that key for a member naming
  it dispatches the gate afresh instead of failing at once, and the
  fresh run's receipt replaces the one under the key and is the
  change's one check of the same content.
- **Bounds** (section 4): each is once. A flake rerun's second red is
  published and repaired as ever, counted against the change's repairs;
  a regate's red holds the change, its red standing. Nothing needs a
  commit.

## 2. A flake rerun

- **When:** a gate's run at a key ends failed (7.3), its failed jobs
  and its report read (7.3, `after-landing-runs.md`, 3), and before the
  receipt is made: the report names its failed tests (`landing-proof.md`,
  1.2's report ids: not unset, unread or past its bound), at least one
  and at most 64 (a signature's own bound, `findings.md`, section 4),
  and each of them is named by an open flake finding; this landing
  has not rerun this key for flakes before; and its project's checks
  can flake, its `flakes` not `none` (`forge/judgement.md`, 4).
  Otherwise the receipt is made as 7.3 says, at once.
- **Named by an open flake finding:** a finding of kind `flake`, in
  any state but `dropped` (`findings.md`, section 4), whose signature's
  repository is the gate's, whose source is the gate's name or `check`
  (a delivery's check runs the same tests under the same keys:
  `forge/reds.md`, 5.8), and whose ids hold the test's key as the
  receipt's report keeps it, byte for byte (`test <binary_id> <name>`
  for a JUnit case, a `lines` report's line as written). Each test may
  be named by a different finding.
- **Rerun:** the run's failed jobs run again, one skein `Write::Rerun`
  each, naming the job's attempt, as 7.4's retry does; where the forge
  refuses a rerun (skein's GitHub adapter does), the gate is dispatched
  afresh, with the same inputs, as 7.4 dispatches. It counts nothing
  against `gate_retries`. While it runs, the key is in flight: no
  receipt, the gate's status on the candidate pending, another landing
  at the key waiting for it (7.1), and its execution clock counted
  afresh from the rerun (7.1, *Bounded*).
- **Kept first:** the landing's row keeps the rerun (section 5) in the
  step that makes its writes, so a restart finds it spent and follows
  the run it made (section 4).
- **Its verdict alone makes the receipt.** The run, read again once it
  ends, is judged as any (7.2 to 7.4): passed, the receipt is a pass,
  and each finding the rerun matched counts the flake once more (its
  `seen`, as a delivery check's flake is counted: `findings.md`,
  section 4), in the step that makes the receipt; failed, its jobs and
  report are read again at their new attempts and the receipt is a
  failure, repaired as 7.3 says, never rerun again in this landing; a
  runner's fault is 7.4's, retried within `gate_retries`, the flake
  rerun staying spent.
- **Every run of the landing's gates** is so treated: the round's top
  and each probe (`landing-search.md`, 3), after a probe by test has
  shown a selected test failed (`landing-search-by-test.md`, 4), and a
  superseding run (section 3).

## 3. A superseded receipt

- **A change's regate:** a change whose landing repair left its head
  unchanged (`forge/change/repairs.md`, *A repair that changes nothing
  is checked once more*), or a change released from a previous build's
  hold on a flake or no-test verdict (*The previous build's holds*
  there), names, on its row, in the step that reads that head, the
  failed receipt its repair's reason resolves to, as the repair's brief
  resolves it (`forge/change/repairs.md`, *A landing's repair*): its key
  (the project, the repository, the gate and the 32-byte key, as
  `Receipt` is keyed) and its run's id, untaken. It goes on at its head
  as a repair that left its head unchanged would: checked at its gates
  (`forge/change/README.md`, 2, step 3), ready, and formed into a
  landing, with no new commit. Where none resolves (its receipt gone,
  past `receipts_per_gate`), it names none and the change holds at once,
  saying the same content can't be checked again.
- **In a landing:** where a member names an untaken regate whose key
  is a key the landing proves (at its candidate or a probe), the key
  counts as having no receipt, whatever receipt stands under it: the
  gate is dispatched there afresh (7.1), and its run's receipt, passed
  or failed, replaces the one under the key and takes the regate, its
  run kept as the regate's `taken` in the step that makes the receipt.
  Only that receipt is the change's measurement: a receipt under the key
  made by another run, before the regate or by a landing none of whose
  members names it, is not, and a key another landing makes at the same
  head over another base is another key. A taken regate supersedes
  nothing.
- **Its verdict** is the change's (`forge/change/repairs.md`, *A repair
  that changes nothing is checked once more*): green, the landing goes
  on as at any green key; red, the member leaves as at any failed key,
  and its change, handed the failure, holds with its red standing
  rather than asking a repair, since the failure is its taken regate's
  receipt. A run the runner failed is retried within `gate_retries` as
  any (`landing-proof.md`, 2.4), the regate untaken.
- **One run for every landing:** while the superseding run is in
  flight, another landing at the key waits for it as for any key in
  flight; a landing none of whose members names the regate meets the
  failed receipt and fails at once, as before.
- **The change keeps one regate at most:** a landing repair that left
  its head unchanged sets it, and it is cleared when the change's head
  moves, a repair is asked for it, or the change ends. Taken, it is kept
  until then.
- **Shown:** the gate's status on the candidate is pending once
  dispatched, then the fresh receipt's verdict (`landing.md`, section
  10), as for any key.

## 4. Restart and bounds

- **Restored whole** (`landing.md`, section 15): the landing's row
  keeps its flake reruns and the change's row its regate, so a run in
  flight for either is found again by its entry and followed (7.1), a
  failed receipt standing under a superseded key not taken as the key's
  verdict while a member's regate names it, and no rerun or fresh
  dispatch is made twice.
- **Once:** a key is rerun for flakes once per landing; a regate
  supersedes one receipt, once, since the fresh run's receipt takes it.
- **Bounded:** a flake rerun's red is a failed receipt, repaired (7.3)
  and counted against the change's repairs
  (`forge/change/repairs-releases.md`, 1, *Bounded apart*); a regate's
  red holds the change with its red standing, for the goal or a person,
  so no red loops without one. No dummy commit is made or needed.

## 5. In the store

- **The landing's flake reruns,** on its row (`Landing`, `landing.md`,
  section 12), each the receipt's key, the failed run's id, the
  numbers of the findings it matched (at most 64, one per failed test
  at most) and the failed jobs' attempts it has yet to rerun (at most
  16, a receipt's own bound), at most one per key the landing proves; a
  registered layout's added field, empty by default. Its first rerun's
  entry is kept in the step that keeps it, and the rest one per step
  after, as 7.4's retry keeps them, so a restart reruns what it still
  lists.
- **A change's regate,** on its row (`forge/change/README.md`; family
  `ChangeHot`), `regate: Option<Regate>`, `Regate { receipt:
  ReceiptKey, run: u64, taken: Option<u64> }`, the receipt's whole key
  as `Receipt` is keyed, its run's id (`ReceiptRun::id`) and, once
  taken, the id of the run whose receipt took it; a registered layout's
  added field, none by default, and `taken` its added field, none by
  default.
- **A no-test verdict,** on the change's row beside the flake's
  (`delegate_flake`): its delegate's verdict 2, a registered layout's
  added field, false by default. The holds `ChangeHold`'s arms `flake`
  and `no_test` name are a previous build's alone
  (`forge/change/repairs.md`, *The previous build's holds*), read as
  `Hold::Flake` and `Hold::NoTest` in the change's policy and reaching
  the client's wire and the public one as their arms of `Hold` and
  `PublicHold` (`../protocol/wires.md`, 9).
- **No store version moves:** each is a registered family's added
  field or arm (`../protocol/store-versions.md`, section 1).

## 6. The world

Focused and fuzzy suites alone; a story of a slower tier is written and
compiled, not run.

- **In the forge's domain** (`temper-engine-domain-forge`'s tests): a
  red whose failed tests all match open flake findings runs once more,
  green proceeds and counts them; red publishes and repairs, never a
  third run; a restart between the rerun's write and its end neither
  repeats nor loses it; a test no finding names and a gate with no
  report make the failed receipt at once. A promised report not read is
  no verdict, retried and then held with no receipt (`forge/judgement.md`, 2).
- **A regate:** a member naming a failed receipt's key and run is
  dispatched afresh across a restart, green proving with no new
  commit; a spent regate fails at once.
- **In the change's policy** (`temper-engine-domain-forge-change`'s
  empty-repair tests): a flake verdict, a no-test verdict and a plain
  no-change at an unchanged head each go on at the head with no repair
  asked and no hold; with no failed receipt resolving, each holds at
  once; a red at its taken regate holds with its red standing, and that
  hold's release asks a fresh repair; a previous build's `Flake` or
  `NoTest` hold's release goes on at the head as they do.
- **A no-test verdict** (`temper-engine-domain-forge`'s and the
  engine's route tests): offered only to a landing repair whose
  failure names no failed test, which comes only from a gate that
  promises no report. A promised report that was not read or names
  none asks no repair (`forge/judgement.md`, 2). The verdict is never
  offered beside the flake's, nor to a repair naming tests or more
  than N, nor to a CI repair; read from the inbox and recovered alike;
  its regate dispatched once by itself, across a restart, the same
  candidate dispatched afresh and green proving with no commit, and a
  red regate holding with its red standing. A receipt under the key made
  before the regate, or by another run, never takes it.
- **In the engine's world** (`temper-engine-domain-world`): the loop
  of 2026-11-16 ends in a landing: a gate red on a fuzzy test, a
  landing repair ending with the flake verdict, its gate run once more
  at the same content with no release, green, and the change landed
  with no commit of its own since; P58's scenario
  (`a_repair_whose_tests_pass_regates_by_itself`), a no-test verdict
  regating by itself, in the same world, and P67's
  (`a_no_change_landing_repair_regates_once`), a plain no-change
  regating once at the failed key, in the engine's forge world; and a
  red matching an
  open flake finding run once more and landed, the finding's count
  grown. The fake forge's workflow names a test that fails at the
  first run over a tree alone (skein's `testing/fakes/forge.md`, 2.3).

## 7. Open questions

- **A comment on the counted finding** (`findings.md`, section 4: a
  comment once per day at most): the rerun counts `seen` alone, as a
  delivery check's flake does. Default: no comment until findings'
  issues are projected.
- **A flake verdict for a CI repair:** a CI repair that changes
  nothing is checked once more by its job run again
  (`forge/change/repairs.md`), so no verdict is offered there. Default:
  landing repairs alone.
