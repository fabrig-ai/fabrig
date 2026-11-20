# A landing's gate run once more

Provisional, 2026-11-19. When a landing's gate, red at a key, is run
once more at the same content rather than repaired: a red made only of
known flakes, rerun before its receipt is published (section 2), and a
failed receipt a change's repair found to be flakes alone, or caused by
no test, superseded by a fresh run once a person releases the change
(section 3). It
carries `../draft/flake-regate.md` on the landing of `landing.md`,
whose words it uses: keys and receipts (section 6), proof by dispatch
(7.1), a real failure (7.3) and a fault of the runner (7.4); flake
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
  repair that finds only flakes*): a landing repair whose failure's
  tests are known may end with a verdict saying every failure is a
  flake, rather than a no-change taken as a failure. Its change holds
  on that verdict for a person.
- **A repair's no-test verdict** (`forge/change/repairs.md`, *A landing
  repair that finds no failed test*): a landing repair whose failure
  names no failed test (its gate names no report, its report was not
  read, or it names none) may end with verdict 2, saying no test failed
  and nothing needs changing, rather than a no-change taken as a
  failure. Its change holds on that verdict for a person, as on a
  flake's, and its release regates as a flake's does (section 3): a red
  no test caused is run once more, never repaired again for nothing
  (the requester, 2026-11-19).
- **A superseded receipt** (section 3): released from that hold, the
  change names the failed receipt, by its key and its run, on its row,
  and goes on as at an unchanged head. A landing that meets that key
  with that failed receipt dispatches the gate afresh instead of
  failing at once, and the fresh run's receipt replaces the failed one.
- **Bounds** (section 4): each is once. A second red is published and
  repaired as ever, counted against the change's repairs, and a flake
  verdict always waits for a person's release. Nothing needs a commit.

## 2. A flake rerun

- **When:** a gate's run at a key ends failed (7.3), its failed jobs
  and its report read (7.3, `after-landing.md`, 4), and before the
  receipt is made: the report names its failed tests (`landing.md`,
  6.2's report ids: not unset, unread or past its bound), at least one
  and at most 64 (a signature's own bound, `findings.md`, section 4),
  and each of them is named by an open flake finding; and this landing
  has not rerun this key for flakes before. Otherwise the receipt is
  made as 7.3 says, at once.
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

- **A change's regate:** a change released from its hold on its
  repair's flake verdict (`forge/change/repairs.md`, *A landing repair
  that finds only flakes*) or its no-test verdict (*A landing repair
  that finds no failed test*) names, on its row, the failed receipt its
  repair's reason resolves to, as the repair's brief resolves it
  (`forge/change/repairs.md`, *A landing's repair*): its key (the
  project, the repository, the gate and the 32-byte key, as
  `Receipt` is keyed) and its run's id. Where none resolves (its
  receipt gone, past `receipts_per_gate`), it names none. Either way
  it goes on at its head as a repair that left its head unchanged
  would: checked at its gates (`forge/change/README.md`, 2, step 3),
  ready, and formed into a landing, with no new commit.
- **In a landing:** where a member names a regate whose key is a key
  the landing proves (at its candidate or a probe) and whose run is the
  run of that key's failed receipt, the key counts as having no
  receipt: the gate is dispatched there afresh (7.1), and its run's
  receipt, passed or failed, replaces the failed one under the key.
  A regate whose run is not the run of its key's receipt (another run's
  receipt made since, or none kept) supersedes nothing: it is spent.
- **One run for every landing:** while the superseding run is in
  flight, another landing at the key waits for it as for any key in
  flight; a landing none of whose members names the regate meets the
  failed receipt and fails at once, as before.
- **The change keeps one regate at most:** a release from a flake or a
  no-test hold sets it, and it is cleared when the change's head moves or a repair
  is asked for it. Spent, it is kept until then and changes nothing.
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
  supersedes one failed receipt, once, since the fresh run's receipt
  has another run.
- **Bounded as before:** a red from either is a failed receipt,
  repaired (7.3) and counted against the change's repairs
  (`forge/change/repairs.md`, *Bounded apart*); a flake or no-test
  verdict holds the change until a person releases it, every time, so
  no red loops without one. No dummy commit is made or needed.

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
  ReceiptKey, run: u64 }`, the receipt's whole key as `Receipt` is
  keyed and its run's id (`ReceiptRun::id`); a registered layout's
  added field, none by default.
- **A no-test verdict,** on the change's row beside the flake's
  (`delegate_flake`): its delegate's verdict 2, a registered layout's
  added field, false by default; and its hold, `ChangeHold`'s added
  arm `no_test` after `flake`, as `Hold::NoTest` in the change's
  policy. The hold reaches the client's wire and the public one as an
  arm `no_test` of `Hold` and `PublicHold`: each moves its version,
  unbridged (`../draft/redeploys.md`).
- **No store version moves:** each is a registered family's added
  field or arm (`../protocol/store.md`, section 4).

## 6. The world

Focused and fuzzy suites alone; a story of a slower tier is written and
compiled, not run.

- **In the forge's domain** (`temper-engine-domain-forge`'s tests): a
  red whose failed tests all match open flake findings runs once more,
  green proceeds and counts them; red publishes and repairs, never a
  third run; a restart between the rerun's write and its end neither
  repeats nor loses it; a test no finding names, a report not read and
  a gate with no report make the failed receipt at once.
- **A regate:** a member naming a failed receipt's key and run is
  dispatched afresh across a restart, green proving with no new
  commit; a spent regate fails at once.
- **In the change's policy** (`temper-engine-domain-forge-change`'s
  empty-repair tests): a flake verdict holds, its release goes on at
  the head with no repair asked, and a plain no-change still holds as
  a failure whose release asks a fresh repair; a no-test verdict holds
  as `NoTest` at its unchanged head and its release goes on as a
  flake's.
- **A no-test verdict** (`temper-engine-domain-forge`'s and the
  engine's route tests): offered only to a landing repair whose
  failure names no failed test, never beside the flake's, nor to one
  naming tests or more than N, nor to a CI repair; read from the inbox
  and recovered alike; its release regating once, across a restart,
  the same candidate dispatched afresh and green proving with no
  commit, and a red regate repaired and counted, whose next no-test
  verdict waits for another release.
- **In the engine's world** (`temper-engine-domain-world`): the loop
  of 2026-11-16 ends in a landing: a gate red on a fuzzy test, a
  landing repair ending with the flake verdict, the change held and
  released, its gate run once more at the same content, green, and the
  change landed with no commit of its own since; and a red matching an
  open flake finding run once more and landed, the finding's count
  grown. The fake forge's workflow names a test that fails at the
  first run over a tree alone (skein's `testing/fakes/forge.md`, 2.3).

## 7. Open questions

- **A comment on the counted finding** (`findings.md`, section 4: a
  comment once per day at most): the rerun counts `seen` alone, as a
  delivery check's flake does. Default: no comment until findings'
  issues are projected.
- **A flake verdict for a CI repair:** a CI repair that changes
  nothing checks its head again (`forge/change/repairs.md`), so no
  verdict is offered there. Default: landing repairs alone.
