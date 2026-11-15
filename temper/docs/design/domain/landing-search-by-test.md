# A search's probes by test

Provisional, 2026-11-15. How a landing's search by prefixes
(`landing-search.md`) probes only the cases its top failed, and lands
nothing on such a probe. It is `../draft/probes-by-test.md`, written
from that draft's sections 3 to 10 as its execution's milestone T1. It
builds on the search's probes, boundary and turn (`landing-search.md`,
sections 3 to 5), on the landing's keys, receipts and proof by dispatch
(`landing.md`, sections 6 and 7), on `temper_probe` (`landing-search.md`,
3), and on a tier's report and its tests by id (`after-landing.md`,
sections 2 and 4). The policy's types it adds are section 10's.

## 1. In one page

- **A gate's report** (section 2): a gate may name the artifact its run
  uploads and its format, as a tier does. A failed run of such a gate
  keeps, on its failed receipt, the failed cases' ids its report names,
  or "many", or none.
- **The selection** (section 3): where every blocking gate that failed
  at the round's top has ids on its failed receipt there, the search is
  by test, each such gate's ids its selection; otherwise it is whole, as
  `landing-search.md` says.
- **A probe by test** (section 4) dispatches only the gates with a
  selection, each given `temper_tests`, and reads its verdict from what
  its selected cases did: red where one failed there, green where the
  run passed, none otherwise, which sends the search on whole.
- **One run per workflow and commit** (section 5): two dispatches of one
  workflow at one commit with different inputs are never in flight
  together, whoever makes them.
- **Nothing lands on a probe by test** (section 6): at the boundary the
  blamed unit leaves and the round ends unlanded; the next round keeps
  the green prefix, stacks the units above the blamed one onto it, and
  proves its top whole.
- **Every probe green** (section 7) blames the top unit, whose repair
  hears it.
- **Shown** (section 8): news by test, the next round's pull request's
  line, `landing show` and the changes view.

## 2. A gate's report

- **The setting:** a repository's gate (`landing.md`, 13.1) gains its
  report, the artifact its run uploads and its format, `junit` or
  `lines`, written in the gate's entry as a tier's are: `artifact` and
  `format`, both or neither, the artifact's name 1 to 32 bytes with no
  `/` (`after-landing.md`, 2). None by default: a gate with none is
  probed whole.
- **Its key:** a gate with a report appends it to its definition
  (`landing.md`, 6.1) after what `base` appends: the byte 2, the
  artifact's length in two bytes and its bytes, and the format's byte, 0
  for `junit` and 1 for `lines`. A gate with none encodes as before, so
  its keys are unchanged; a report added or changed moves its gate's
  keys, as a tier's does (`after-landing.md`, 2), so that a receipt's ids
  were read as its key's definition says.
- **Read after a failed run:** each run of a gate with a report whose
  workflow's own step failed (`landing.md`, 7.3), at a top, a probe or a
  baseline, has its artifact read (skein's `Read::Artifact`), bounded as
  a tier's report's read is, before its failed receipt is made. A
  passed run, or a runner's fault (`landing.md`, 7.4), reads none.
- **What is read:** the failed cases' ids, as `after-landing.md`, 4
  reads a tier's, but for errored cases, which are not taken: a case
  the run did not observe failed nowhere. A JUnit file's cases with a
  `failure` are its ids (`classname.name`), those with an `error` alone
  none; a `lines` file's each nonempty line, a trailing carriage return
  dropped. The ids are kept sorted, byte by byte, and unique, as a
  tier's are. A failed step is a case where the report names it as one
  (section 11).
- **Many:** more than `forge.tiers.failing_tests` ids, an id past
  `forge.tiers.test_id_bytes`, an id that is not UTF-8 or holds a
  control character (a byte below 32, or 127), or a file past the
  read's bound, each reads as "many".
- **None:** an absent artifact, a file its format cannot read, an
  answer of another shape, or a report that names no failed case. It
  is no fault: the run still failed.
- **The failed receipt** keeps the ids, or "many", or none, in its
  `failures` (`landing.md`, 6.2). Its failure stands as `landing.md`,
  7.3 reads it either way: its repair, its member's leaving and its
  status are those of any failed receipt.

## 3. The selection

- **At the search's start** (`landing-search.md`, 2), the top's facts
  are read for each blocking gate with a failed receipt at the top's
  key. Where each such gate's receipt keeps ids, the search is by test:
  each such gate's selection is its ids, and every other gate has none,
  so that it is not dispatched at a probe. Where any keeps "many" or
  none, the search is whole, as `landing-search.md`, 3 probes, and lands
  as `landing-search.md`, 5 says.
- **A gate not failed at the top** (passed, or still in flight when the
  search starts) has no selection: nothing it could find is the
  search's, and the next round proves it whole (section 6).
- **The dispatch's room:** a selection is sent as `temper_tests`, each
  id followed by a line feed. Where a selection's value would not fit
  the room a probe's dispatch leaves it within the forge client's
  `op_bytes` (skein's `forge/client.md`, 2.2), measured at the round's
  longest member branch with every other input a probe sends, the
  search is whole from its start. The selection only narrows (below),
  so no later probe's would pass it.
- **It narrows, never widens:** at a red probe, each gate red there
  keeps as its selection only its selected cases that failed there. So
  a boundary green for a selection holding every case still selected
  stays green for them.

## 4. A probe by test

- **Where:** at `landing-search.md`, 3's boundaries, one probe at a time,
  at most `ceil(log2(n))` of them while the search stays by test.
- **For each gate with a selection,** keyed at the probe's prefix as any
  probe's (`landing.md`, 6.1), in this order:
  1. a passed receipt at the key holds: the whole gate passed there, so
     that gate is green;
  2. a failed receipt whose ids hold a selected case makes that gate
     red: that case fails the gate at that tree;
  3. a pass by test the round's search keeps for that prefix and gate
     makes that gate green; a no verdict it keeps sends the search on
     whole (below);
  4. otherwise, a run in flight at the key, or of the gate's workflow at
     that commit (section 5), is waited for;
  5. otherwise the gate is dispatched as `landing-search.md`, 3
     dispatches a probe's gate, `temper_probe` `yes` among its inputs,
     with `temper_tests` added after the others: its selection. A
     failed receipt whose ids hold no selected case, or that keeps none
     or "many", may be another case's, from below the green bound: it is
     kept, and the probe dispatched as if it were not there.
- **A gate with no selection is not dispatched,** and an advisory gate
  never is (`landing-search.md`, 3).
- **Its verdict:** red at the first gate red there, which makes r the
  boundary and narrows the selections (section 3); green once every
  gate with a selection is green there, which makes g the boundary. A
  run still in flight once the verdict is taken goes on and moves no
  bound.
- **A run by test's end,** read by the top for the run's own inputs, its
  `temper_tests` among them, whichever round or search is current:
  - **passed:** no receipt. Where its landing's live round searches by
    test and probes that run's prefix, the round's search keeps it as a
    pass by test at that prefix and gate, with its run's number, for its
    verdict alone, as a tier's rerun and a bisect's probe are kept on the
    tier's record (`after-landing.md`, 2);
  - **failed, its report naming a case among its `temper_tests`:** a
    failed receipt under its key at that prefix, with its report's ids
    (section 2), reused as any (`landing.md`, 6.2): a later top or probe
    at that key fails at once;
  - **failed otherwise** (its report absent, unreadable or "many", a
    failed case outside its selection, a selection its command could not
    run): no receipt. Where its prefix is the probe of its landing's live
    search by test, the search keeps it as a no verdict at that prefix
    and gate;
  - **a runner's fault:** retried within `gate_retries`, held past it or
    past `gate_timeout`, as any probe's (`landing-search.md`, 3), its
    inputs kept (section 5).
- **No verdict sends the search on whole:** its bounds back to the
  round's base and candidate, g 0 and r n, its selections and passes
  dropped, and every probe whole from there, as `landing-search.md`, 3
  probes, receipts holding by key as ever; such a search lands as
  `landing-search.md`, 5 says. It is rare, and never wrong: a whole
  search assumes nothing a probe by test saw.
- **Statuses:** a probe by test posts no pending status on its prefix,
  and no success where it passes: its run answers for its selection
  alone, and the prefix is proved whole in a later round before it
  lands. A red's failed receipt posts its failure there, as any failed
  receipt's (`landing.md`, section 10).
- **Late outcomes:** a run by test followed to its end after its verdict
  was taken, or after its round ended, makes its failed receipt where it
  is red, and nothing otherwise (`landing-search.md`, 6).

## 5. One run per workflow and commit

- **Why:** temper finds a dispatched run by its repository, workflow,
  commit, user and dispatch time, never by its inputs (`landing.md`,
  7.1; skein's `forge/forgejo.md`, section 2), so two of its dispatches
  of one workflow at one commit in flight together could each take the
  other's run.
- **So they never are:** before any dispatch of temper's (a landing's
  gate at a top or a probe, a gate's baseline, a tier's run, rerun or
  bisect's probe, a follow's check), a dispatch of the same workflow in
  the same repository at the same commit still in flight with other
  inputs makes it wait. In flight is from the dispatch's effect being
  asked until its run is read ended, through its retries. A run in flight
  of the same key with the same inputs serves as `landing.md`, 7.1 says.
- **Then:** once that run ends, the waiting dispatch takes what that run
  made for its own key, a receipt holding as any, or is dispatched. A
  landing hears the end as it hears any of its keys' runs; a tier and a
  follow's check dispatch at their next step after it.
- **Each run keeps its inputs,** its selection among them, through its
  retries (a rerun of its jobs, or a dispatch afresh with the same
  inputs), a restart (found again from its dispatch's kept inputs,
  `landing.md`, section 15) and its round's end, so that its outcome is
  read for what it ran (section 4).

## 6. What lands

- **The top's run does not stand for a prefix's:** a case that passed at
  the top can fail at a prefix below it, where a unit above fixed or hid
  what one below broke, and a case the top did not run, after a compile
  error, ran nowhere (`../draft/probes-by-test.md`, 5). So a search by
  test never turns at its green prefix.
- **Once its boundary holds,** in this order, each step recorded before
  the next:
  1. **The blamed unit leaves** with `landing-search.md`, 4's words, and
     its changes repair (section 7).
  2. **The round ends unlanded** (`landing.md`, 5.1) as when a member
     leaves after the stack: its pull request closed and the branches
     from the blamed unit's first member up deleted. The green prefix's
     members keep their branches, prefixes and receipts; their gates'
     proofs are taken afresh from their keys, since a pass by test
     proves nothing.
  3. **The next round** stacks the units above the blamed one onto the
     green prefix, or, with none, takes the green prefix as its top with
     no job, as a round after a member's leaving does (`landing.md`,
     5.1), a stale base stacking every member again. Its top is proved
     whole, first in line, and lands at its turn, the green prefix's
     members with it. With no member left, the landing ends
     (`landing.md`, 4.2).
- **Where g is 0** no member is kept below the blamed unit, and the next
  round stacks the units above it on the base, as `landing-search.md`, 4
  does.
- **What the next round's top proves** is the tree that lands. A failure
  a unit above hid stays hidden in it, the hiding unit landing too;
  where the hiding unit was the blamed one, the failure shows at the
  next top, and its own search finds it below.

## 7. Every probe green

- **The top unit is blamed:** with every probe green the search holds g
  at n − 1 and r at n, as whole probes do. The failure appears with that
  unit on top of a prefix where the selected cases passed; it needs that
  unit on the others, or is a flake.
- **The next round** takes the green prefix, every unit but the top, as
  its top with no job, and proves it whole (section 6).
- **The repair's words:** a unit blamed by a search by test is repaired
  as `landing-search.md`, 7 says, its opening line the same, and its
  next sentence reads `Repair the failed landing gate. Its failing tests
  passed at GREEN and failed at RED, with this change's unit on top of
  it; a change below may still share the cause.` in place of that
  section's. Where the blamed unit is the round's top, ` They passed at
  every probed prefix below it: they may fail only with this change's
  unit on the others, or be flaky.` follows. The rest is that section's.
- **The repair ask** (`landing.md`, 4.3) carries, beside its green prefix
  and changes, whether its search was by test and whether the blamed
  unit is the top, and the change keeps them with its repair, so that a
  brief made after a restart says the same.

## 8. What is shown

- **A probe's news** by test, to the goal of each member with a prefix
  in the round, once its prefix is chosen: `landing L: change C probing
  at PREFIX by N tests, red at RED, on BASE`, N the count of its
  selections' cases there, as `landing-search.md`, 8's with `by N
  tests`; a search gone whole, once: `landing L: probing whole again
  from BASE`.
- **After the boundary,** the blamed members' goals hear them leave
  with `landing-search.md`, 4's words, and the goals of the green
  prefix's members, as those of the units above it, that they prove
  again in the next round.
- **The next round's pull request** names the search that ended the
  last, a line after its members': `Search by test: red at RED, change
  C1, C2 blamed; green at GREEN.`, the blamed unit's changes in order,
  GREEN `the base` where g is 0. The landing keeps that line from the
  boundary until a round's pull request is opened with it, or the
  landing ends.
- **`landing show`** (`landing.md`, section 14): a search by test, each
  gate's selection with its count and its first three ids, or whole, or
  whole again; at each probed prefix, each gate's verdict by test,
  `green by test` with its run's number, or by its receipt as before.
- **The changes view:** `probing by test at PREFIX` while a probe by
  test runs.

## 9. Settings, store and restart

- **Settings:** a gate's `artifact` and `format` in the document's
  `landing` member (`../protocol/configuration/engine.md`), as kind 5's
  until it is a setting (`landing.md`, 13.1).
- **Adoption** under `turn` (`landing.md`, 13.2): for a gate with a
  report, whether its workflow declares `temper_tests`, or says
  otherwise in words: a workflow that does not runs its probes whole,
  since Forgejo passes a workflow only the inputs it declares
  (`landing-search.md`, 3).
- **The store,** in the registered family `temper-store`
  (`../protocol/store.md`, section 2): fields added in place with their
  defaults, no version moved and no patch (`../protocol/baseline.md`,
  7):
  - a gate's failed receipt keeps its `failures`, ids or many, which
    the `Receipt` row holds already for tiers;
  - the round's search keeps how it probes (whole, by test, or whole
    again), each gate's selection, and its passes and no verdicts by
    test, until the round ends; an older row's search is whole;
  - the landing keeps the search's line (section 8) until a pull request
    takes it; an older row keeps none;
  - a change's repair context keeps whether its search was by test and
    whether its unit was the top; an older row's was not;
  - each run in flight keeps its inputs in its dispatch's entry, as it
    does already.
- **Restart:** a probe by test's run is found again as a proof's is
  (`landing.md`, section 15), its inputs read from its dispatch's
  entry, and its search at its stored bounds, selections and passes,
  nothing made twice.

## 10. The policy's boundary

The landing policy (`temper-engine-domain-forge-landing`) keeps nothing
(`landing.md`, section 2); these are the shapes its top and it share,
binding for the changes that build this document:

- `TestId`, `Box<[u8]>`: a case's id, as the tiers' crate's.
- `Gate` gains `report: Option<Report>`, `Report { artifact: Box<[u8]>,
  format: Format }`, `Format` the tiers' crate's `Junit` or `Lines`.
- `Search { green, red, tests: SearchTests }`, `SearchTests` one of
  `Whole`; `ByTest { gates: Box<[Option<Box<[TestId]>>]>, probed:
  Box<[Probed]> }`, `gates` one per repository gate in configuration
  order, its sorted and unique selection or `None`; and `WholeAgain`.
- `Probed { member: u16, gate: u16, run: u64, passed: bool }`: a run by
  test's end at a prefix, `member` its member's number from 1 as the
  search's bounds count them, `run` the run's number, `passed` false for
  no verdict. The top appends one as section 4 says, with the run's
  end; the policy reads them.
- `MemberFacts` gains `failed: Box<[Option<Box<[TestId]>>]>`, one per
  gate: the ids the failed receipt at that member's key keeps, `None`
  where it has no failed receipt or its receipt keeps none or "many".
- `Limits` gains `tests_bytes: u32`: section 3's room, which the top
  measures for each landing.
- `Landing` gains `searched: Option<Searched>`, `Searched { red: Commit,
  green: Option<Commit>, changes: Box<[u64]> }`, `green` `None` for the
  base: section 8's line, set at a boundary by test and taken when a
  round's pull request is opened.
- No decision changes: the top adds `temper_tests` to a `Dispatch` at
  the probe of a search by test for a gate with a selection.

## 11. What a project gives

- **A compact report:** a gate whose run uploads its failed cases' ids,
  a failed step among them as a case, small enough for the read: a JUnit
  report that also carries each case's output may pass it with a
  handful of failures, and reads as many.
- **A workflow** that declares `temper_tests` and passes it to its
  command.
- **A command that runs only the cases named,** building only the
  packages that hold them, judging them as at the top (a failure rerun,
  a failure the base had), passing a case absent at its tree, and
  reporting the same way. A command that keeps its own record of a
  tree's results keeps a run by test out of it.
- **fabrig's** `.forgejo/workflows/gate.yml` declares `temper_tests`,
  passes it to the runner host's command as `CI_GATE_TESTS` beside
  `CI_GATE_PROBE`, and uploads the command's failed ids, one per line,
  from `failed-tests.txt` as the artifact `failed-tests`. The command is
  the runner host's, no plan's; the project configures its gate's report
  (`artifact` `failed-tests`, `format` `lines`) once it runs a selection,
  and until then temper probes whole (`../draft/probes-by-test.execution.md`,
  section 2).

## 12. The world

Focused and fuzzy suites alone, in the forge's world (`landing.md`,
section 16), the landing policy's own cases and fuzzy stories, and the
engine's domain stories:

- **the report:** a failed top's receipt keeping its ids; many past
  `failing_tests`, `test_id_bytes` or the read, and for an id with a
  control character; none from an absent or unreadable report; errored
  cases left out; a key moved by a report added, and unchanged without
  one;
- **the selection:** a search by test where every gate red at the top
  has ids, whole otherwise; a selection past the room whole; the
  selection narrowing at a red;
- **probes by test** at `landing-search.md`, 3's boundaries,
  `temper_tests` and `temper_probe` among their inputs, a gate green at
  the top not dispatched;
- **verdicts:** red where a selected case failed, a failed receipt under
  its key and a later top failing on it; green where the run passed, no
  receipt; a selected case absent at a lower prefix, green there; no
  verdict, the search going whole from the base and candidate;
- **receipts found:** a passed one holding; a failed one with a selected
  id moving r; five units, the top failing A, the probe at 2 passing A,
  the key at 3 holding a whole failed receipt for B alone: the probe by
  test dispatched at 3 and r moved only by A;
- **one run per workflow and commit:** two dispatches with different
  inputs at one commit, one waiting for the other, from a probe and a
  top, from two landings, and from a tier and a gate; a run's inputs
  kept through a retry and a restart; a late pass making nothing;
- **what lands:** no turn at a green prefix by test; the round ended
  unlanded, the green prefix kept, the units above stacked onto it, the
  whole proved at the next top and landed; the top blamed and the next
  round's top made with no job; g at 0; a stale base stacking every
  member;
- **every probe green:** the top blamed, its repair's words;
- **a restart** while probing by test, and between the boundary and the
  next round's pull request, its search line kept, nothing made twice;
- **what is shown:** the news, the next round's pull request, `landing
  show` and the changes view.

In the process tier, the story of a red narrowed (`../testing/process-tier.md`)
searches by test: written and compiled, run in that tier after landing.

## 13. Later

- **The top's failing cases run again at the top** when every probe
  passed, before its unit is blamed: a flake told from a combination for
  one probe by test (`../draft/probes-by-test.md`, 11).
- **A green prefix proved and landed by itself** when the units above
  it have gone red again in a later round.
