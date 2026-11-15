# probes-by-test: a search's probes run only what its top failed

Draft, 2026-11-15, asked for by the user that day. When a landing's round of several units goes red, its search dispatches the train's blocking gates at the unit boundaries below the top, one probe at a time, each probe the whole gate (`../domain/landing-search.md`, section 3). This draft makes a probe run only the cases its top failed, named by the top's report, and lands nothing on a probe: the green prefix a search finds by test is proved whole, with the units above it, at the next round's top. It builds on the search, on the landing's keys, receipts and proof (`../domain/landing.md`, sections 6 and 7), on `temper_probe`, the input that tells a gate's workflow a run is a probe (`../domain/landing-search.md`, 3, by goal 3821), and on a tier's tests by id (`../domain/after-landing.md`, sections 2 and 4). It is the "probes by test" `merge-queue.md`, section 13 left for later.

## 1. In one page

- **A probe runs only the top's failed cases** (sections 3 and 4): their ids read from the gate's report at the top, as a tier reads its run's, and given to the gate's workflow as `temper_tests`, one per line, as a tier's rerun is given them. A failed step the report names as a case reruns alone.
- **A probe finds the boundary and proves nothing** (section 4): a passed probe by test is the search's alone, never a receipt; one where a selected case failed is a failed receipt under the gate's key, since that case fails the gate at that tree.
- **Only a selected failure moves the red bound** (sections 3 and 4): the selection narrows to the cases that failed there and never widens, so the green bound below stays green for every case still selected. A probe that cannot say whether a selected case failed sends the search on whole.
- **Nothing lands on a probe by test** (section 5). Once the boundary holds, the blamed unit leaves and the round ends unlanded. The next round keeps the green prefix's members as they were, stacks the units above the blamed one onto them, and proves its top whole.
- **Every probe green** (section 6): the top unit is blamed, as whole probes blame it, and its repair hears that its failing cases passed at every probed prefix below.
- **Whole otherwise:** a gate with no report, or a top whose report names no failed case or too many, searches and lands as today.
- **What it saves** (section 5): a red in a batch of four costs up to three whole gates beyond the top today, and one by test, beside two probes by test.

## 2. A probe today

- **In fabrig's own landings, 2026-11-15:** a round of two units, its top's gate run 975 red in 872 s (clippy 117 s, build 291 s, focused 253 s red, fuzzy 183 s red), six tests failing in four test binaries. Its probe at the first unit's prefix, run 976, green in 558 s (clippy 91 s, build 82 s, focused 209 s, fuzzy 157 s). The probe ran the whole gate to answer for six tests.
- **A search of n units** makes at most `ceil(log2(n))` probes beyond the top, one at a time: three with `forge.landing_members` at 5 (`../domain/landing-search.md`, 3). Each is a whole gate on the train's runner, so a batch's red holds the train for two or three gates before its green prefix lands.
- **What a probe asks** is only whether the top's failure appears at its prefix. Every other case already ran at the top, with more members on it.

## 3. The selection

- **A gate's report:** a gate's setting (`../domain/landing.md`, 13.1, kind 5) gains `report`, the artifact its run uploads and its format, a JUnit file or one id per line, as a tier's has (`../domain/after-landing.md`, 2). A gate with none is probed whole.
- **Read as a tier's is** (`../domain/after-landing.md`, 4): its failed cases' ids, distinct and in order, each within `test_id_bytes`, at most `failing_tests` of them; past either bound, or past the client's read, the run's failures are "many". An id is UTF-8 with no control character; a report holding another reads as many. A failed step is a case where the report names it as one. Errored cases are not selected: a case the run did not observe failed nowhere.
- **A failed receipt of a gate with a report** keeps the ids its run's report gave, or "many", or none where the run uploaded no report or it could not be read or named no failed case. Its failure stands as `../domain/landing.md`, 7.3 reads it either way.
- **A search by test:** where each blocking gate that failed at the top has ids on its failed receipt there, they are that gate's selection, and the search is by test. Otherwise, many or none, every probe is whole, as today.
- **The selection narrows, never widens:** at a red probe, the gate's selection becomes the selected cases that failed there. So boundary g, green for a selection that held every case still selected, stays green for them.

## 4. A probe by test

- **Dispatched** as `../domain/landing-search.md`, 3 dispatches a probe, `temper_probe` `yes` among its inputs, for each gate that failed at the top, with `temper_tests` added: its selection, one id per line. A probe whose dispatch would pass the forge client's `op_bytes` is whole. A blocking gate that passed at the top is not dispatched: nothing it could find is the search's, and the next round proves it whole.
- **The gate's workflow, given `temper_tests`,** runs only those cases, building only what they need, judges them as it judges the top's, against the round's base for a gate that reads it, and reports their failures as at the top. A selected case absent at that tree, its test or its binary not there yet, fails nothing and passes the run: a failure cannot appear where its case does not exist. A workflow that does not declare the input runs whole, since Forgejo passes a workflow only the inputs it declares (`../domain/landing-search.md`, 3). Adoption says so (section 8).
- **Its verdict, from its report:** red where a selected case failed there; green where the run passed; none otherwise, as for a run that failed with no selected case among its failures (its report absent or many, a failed case outside the selection, a selection its command could not run). A gate's red makes r the boundary, as `../domain/landing-search.md`, 3 judges a probe; green once each gate dispatched there is green.
- **A red is a failed receipt** under the gate's key at that prefix (`../domain/landing.md`, 6.2), with the ids its report gave, as a probe's run that stopped at its first red is one already: a later top or probe at that key fails at once. A passed run is no receipt: it is kept on the round's search for its verdict alone, as a tier's rerun and a bisect's probe are kept on the tier's record (`../domain/after-landing.md`, 2). A run with no verdict makes no receipt.
- **No verdict sends the search on whole:** its bounds back to the round's base and candidate and every probe whole from there (`../domain/landing-search.md`, 3), receipts holding by key as ever. It is rare, and never wrong: a whole search assumes nothing a probe by test saw.
- **A receipt found at a probe's key** holds as before where it passed: the whole gate passed there. A failed one moves r only where its ids hold a selected case. Where they hold none, or it has none, the failure may be another's, from below g, so it is kept and the probe by test is dispatched as if it were not there.
- **One run per workflow and commit:** temper finds a run by its repository, workflow, commit, user and dispatch time, never by its inputs (`../domain/landing.md`, 7.1; skein's `forge/forgejo.md`, section 2), so two of its dispatches of one workflow at one commit with different inputs are never in flight together, whichever gates, tiers or landings make them. A dispatch whose inputs differ from those of the run in flight waits for that run to end, then takes what that run made for its own key, a receipt holding as any, or dispatches. Each run in flight keeps its inputs, its selection among them, through its retries, a restart and its round's end, so that its outcome is read for what it ran.
- **Late outcomes:** a run by test followed to its end after its verdict was taken, or after its round ended, makes its failed receipt where it is red, and nothing otherwise (`../domain/landing-search.md`, 6).
- **Faults and bounds** as any probe's (`../domain/landing-search.md`, 3): retried within `gate_retries`, held past `gate_timeout`.

## 5. What lands

- **The top's run does not stand for a prefix's.** A case that passed at the top can fail at a prefix below it, where a unit above the prefix fixed or hid what one below broke: the observed boundary the search already allows (`../domain/landing-search.md`, 4). A case the top did not run, after a compile error, ran nowhere. Landing a prefix on its probes by test would let the branch take a red no gate saw, and a gate that judges a failure against its base would then find it failing at the next round's base too, and pass.
- **So nothing lands on a probe by test:** a search by test never turns at its green prefix. Once its boundary holds:
  1. **The blamed unit leaves** with `../domain/landing-search.md`, 4's words, and its changes repair (section 6).
  2. **The round ends unlanded** (`../domain/landing.md`, 5.1), as a member leaving after the stack ends it: its pull request closed and the branches from the blamed unit's first member up deleted. The green prefix's members keep their branches, prefixes and receipts.
  3. **The next round** stacks the units above the blamed one onto the green prefix, or, with none, takes the green prefix as its top with no job (`../domain/landing.md`, 5.1). Its base is the train's tip read afresh, and a base that moved stacks every member again. Its top is proved whole, first in line, and lands at its turn, the green prefix's members with it. With no member left, the landing ends (`../domain/landing.md`, 4.2).
- **Where g is 0** no member is kept below the blamed unit, and the next round stacks the units above it on the base, as today.
- **What the next round's top proves** is the tree that lands. A failure a unit above hid stays hidden in it, the hiding unit landing too; where the hiding unit was the blamed one, the failure shows at the next top and its own search finds it below.
- **What it costs:** the next round's whole gate. Where units remain above the blamed one, that round runs today too, and the green prefix's proof is shared with theirs; where none do, it is the green prefix's own proof, in place of the whole probe that found it green. The green prefix's members land at that round's turn instead of a turn of their own, and a red above them there delays them by its own search. Whole gates beyond the top, in a batch of four with one gate, one unit breaking, no receipt found and a green next top:

| The unit that breaks | Today | By test |
|---|---|---|
| 1 | probes at 2 and 1, both red; units 2 to 4 in the next round: 3 | the same probes by test; the next round: 1 |
| 3 | probes at 2, green, and 3, red; units 1 and 2 land; unit 4 in the next round: 3 | the probes by test; units 1, 2 and 4 in the next round: 1 |
| 4 | probes at 2 and 3, both green; units 1 to 3 land: 2 | the probes by test; units 1 to 3, the next round's top with no job: 1 |

## 6. Every probe green

- **With every probe green** the search holds g at n − 1 and r at n, and the top unit is blamed, as whole probes blame it: the failure appears with it on top of a prefix where the selected cases passed. A prefix not probed may fail them too (`../domain/landing-search.md`, 4). The failure needs that unit on the others, a failure only the combination makes, or is a flake that failed at the top however its gate judges a flake there.
- **Its repair hears which it may be.** A blamed unit's brief by test opens as `../domain/landing-search.md`, 7's does, and its next sentence reads `Repair the failed landing gate. Its failing tests passed at GREEN and failed at RED, with this change's unit on top of it; a change below may still share the cause.` Where the blamed unit is the top, ` They passed at every probed prefix below it: they may fail only with this change's unit on the others, or be flaky.` follows. The rest is section 7's.
- **The next round** takes the green prefix, every unit but the top, as its top with no job, and proves it whole.

## 7. What is shown

- **A probe's news,** to the goal of each member with a prefix in the round: `landing L: change C probing at PREFIX by N tests, red at RED, on BASE`, N the selection's count, where `../domain/landing-search.md`, 8's has no `by`; a search gone whole, `landing L: probing whole again from BASE`.
- **After the boundary,** the blamed members' goals hear them leave with `../domain/landing-search.md`, 4's words, and the green prefix's goals, as the units above already do there, that they prove again in the next round.
- **The next round's pull request** names the search that ended the last, a line after its members': `Search by test: red at RED, change C1, C2 blamed; green at GREEN.`, as `../domain/landing-search.md`, 8's line. The landing keeps that line from the boundary until the pull request is opened with it, or the landing ends.
- **`landing show`** (`../domain/landing.md`, section 14): the search's selection per gate, its count and its first ids; each probe's verdict, `green by test` or by receipt.
- **The changes view:** `probing by test` with its prefix.

## 8. Settings, store and adoption

- **Kind 5's gate** gains `report`: the artifact's name and its format, `junit` or `lines`, none by default (`../domain/landing.md`, 13.1). Until kind 5 is a setting, it is in the document's `landing` member, as the gates are.
- **Adoption under `turn`** (`../domain/landing.md`, 13.2): for a gate with a report, whether its workflow declares `temper_tests`, or says otherwise in words.
- **The store:** a gate's failed receipt keeps its ids or "many"; each run in flight keeps its inputs; the round's search keeps each gate's selection, each probe's passes by test, and whether it went whole, until the round ends; the landing keeps the last search's line until the next pull request takes it. In place, before the first release (`../protocol/store.md`, section 2), with the golden store's line (`../protocol/baseline.md`, 7).
- **Restart:** a probe by test's run is found again as a proof's is (`../domain/landing.md`, section 15), its inputs read from the store.

## 9. What a project gives

- **A compact report:** a gate whose run uploads its failed cases' ids, a failed step among them, small enough for the client's read: a JUnit report that also carries each case's output may pass it with a handful of failures, and reads as many.
- **A workflow** that declares `temper_tests` and passes it to its command.
- **A command that runs only the cases named,** building only the packages that hold them, judging them as at the top (a failure rerun, a failure the base had), passing a case absent at its tree, and reporting the same way. A command that keeps its own record of a tree's results, for a later gate's base, keeps a run by test out of it: that run is not the tree's whole result.
- **fabrig's:** `.forgejo/workflows/gate.yml` declares `temper_tests` and passes it to the runner host's command as `CI_GATE_TESTS`, beside `CI_GATE_PROBE`, and uploads the command's failed ids, one per line, as an artifact of their own. The command maps an id `gate.<step>` to that step and `<binary>.<test>`, split at the first `.`, to that test, and builds only the packages of the binaries named. The project configures its gate's report, that artifact as `lines`, once its command runs a selection; until then temper probes whole.

## 10. Acceptance

Focused and fuzzy suites alone, in the forge's world (`../domain/landing.md`, section 16) and the engine's domain stories, each failing first:

- the selection: a top's failed receipt keeping its report's failed ids, many past `failing_tests`, `test_id_bytes` or the read, an id with a control character, none from an absent or unreadable report; errored cases left out; a search by test where every gate red at the top has ids, whole otherwise;
- probes by test at `../domain/landing-search.md`, 3's boundaries, `temper_tests` and `temper_probe` among their inputs, a gate green at the top not dispatched, a dispatch past `op_bytes` whole; the selection narrowing at a red;
- verdicts: red where a selected case failed, a failed receipt under its key and a later top failing on it; green where the run passed, no receipt; a selected case absent at a lower prefix, green there; no verdict, the search going whole from the base and candidate;
- receipts found: a passed one holding; a failed one with a selected id moving r; five units, the top failing A, the probe at 2 passing A, the key at 3 holding a whole failed receipt for B alone: the probe by test dispatched at 3 and r moved only by A;
- one run per workflow and commit: two dispatches with different inputs at one commit, one waiting for the other, from a probe and a top, and from two landings; a run's inputs kept through a retry and a restart; a late pass making nothing;
- what lands: no turn at a green prefix by test; the round ended unlanded, the green prefix kept, the units above stacked onto it, the whole proved at the next top and landed; the top blamed and the next round's top made with no job; g at 0; a stale base stacking every member;
- every probe green: the top blamed, its repair's words;
- a restart while probing by test, and between the boundary and the next round's pull request, its search line kept, nothing made twice;
- the news, the next round's pull request, `landing show` and the changes view;
- adoption's words for a gate whose workflow does not declare `temper_tests`.

In the process tier, the story of a red narrowed (`../testing/process-tier.md`) searches by test: written and compiled, run in that tier after landing.

## 11. Later

- **The top's failing cases run again at the top** when every probe passed, before its unit is blamed: a flake told from a combination for one probe by test.
- **A green prefix proved and landed by itself** when the units above it have gone red again in a later round, so that a batch with two breaking units lands its green prefix sooner.
