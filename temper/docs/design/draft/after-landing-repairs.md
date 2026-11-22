# after-landing-repairs: a goal ends at landing, and each red after landing is blamed and repaired

After landing today is a gate for goals: a landing's change lives until every tier following its branch clears it, its goal waits for that, and so does every goal that depends on it; a red reaches the goals of the landing it is blamed on as news, and they repair it (`../domain/after-landing.md`, sections 6 and 7). This draft ends a goal at its landing, so that the goals after it start then, and keeps what after landing finds as a record: each red a finding blamed on the landing, and so the goal, that caused it, shown on that goal and its milestone. The project's triager sorts each one: a routine one gets a repair goal with no one asked, and one that needs a product decision goes to the person who asked for the goal (the person, 2026-11-18). Sections 3 to 6 are for v0.1.0. Sections 7 and 8 may follow it, with reds repaired by hand from the record until then. Section 12 holds the rest of the vision.

## 1. In one page

- **A goal ends at landing** (section 3), where its project says so: its change ends at landed, the closer's *Land* waits for landed, and a goal that depends on it starts then. The tiers still judge each landing.
- **Every red is a finding** (section 4): a run's failures that no uncleared red finding holds, filed as one finding per landing they are blamed on, owned (proven by bisection, or the range's one landing) or suspected, and one unblamed finding for those failing before the run's range. A failure that passes alone is a flake, filed as one.
- **Cleared by a run that passes it** (section 5): a red finding holds its tests, whatever its state, until a judged run fails none of them, and is never evicted meanwhile.
- **Shown on the goal and its milestone** (section 6): a goal's page lists the reds blamed on it, and a milestone counts those its goals owe without being held for them.
- **The triager sorts each red** (section 7). A routine red gets a repair goal, with no one asked. One that needs a product decision is asked of the culprit goal's person. A flake is dropped, and wrong blame is moved.
- **Repair goals** (section 8) are started by the triager with one act for the findings one goal settles. At most `repair_goals` are live at once (2), and each finding gets at most `repair_attempts` goals (2) before a person is asked. Each goal is judged by the first run that covers its landing.
- **A failing test's cap** (section 4) rises from 16 to 64, so that new failures stop hiding behind old ones.

## 2. Where it starts

- **A landing is verifying until cleared,** its change alive and its goal waiting (`../domain/after-landing.md`, section 6; `../domain/forge/branches.md`, section 2, the closer's *Land*). Past the project's `verification_bound` it ends landed and uncleared; a change already red has no bound and waits until a later run passes the failures blamed on it. No setting turns the wait off: `../domain/after-landing.md`, section 13 leaves open whether a goal may close before the bound.
- **A milestone's next goal waits with it:** a dependency counts once it has ended done (`../domain/milestones.md`, section 4), and a goal ends done only once its landing is cleared, so a slow or red tier run holds every goal behind it.
- **A red goes to the goals of the landing it is blamed on** (`../domain/after-landing.md`, section 7), new failures owned by the range's one landing or bisected to the first that fails them (5.2 there). A red no goal can hear is a finding of its project, once per landing, with no area, so that the project's triager holds it (`../domain/findings.md`, section 4; `../domain/areas.md`, section 5). A triager may route a finding to a goal or start a new one for it, asking its person first unless the project's classes decide `findings` (`../domain/findings.md`, section 6; `../domain/forge/attention.md`, section 6).
- **At most `failing_tests` per run, 16:** past it, a run's failures are many, blamed by landing alone, and many after many counts as nothing new (`../domain/after-landing-runs.md`, section 3, and `../domain/after-landing-storage.md`, section 1).
- **What the reference deployment showed** (2026-11-17 and 18): its main's tier failed 28 to 48 tests for hours, so new failures hid behind old ones. To stop goals waiting, its project's `verification_bound` was cut to one minute, before any tier run (15 to 20 minutes) can judge. That frees no change already red: four were held red until cancelled by hand. Blame went astray twice: once onto a change that couldn't have reached the failures an earlier change had left, and once onto load flakes of a loaded worker that failed again alone. Meanwhile repairs were made by hand, one repair goal per run's new failures, and a later run's new failures were added to the open goal by message.

## 3. A goal ends at landing

- **Kind 4's `wait`**, `cleared` (today's) or `landed`, per project (`../domain/landing-settings.md`, 1.1).
- **Under `landed`, a change ends at landed:** its result is given at its landing, naming the landing commit and saying that after landing judges it later; its branch's hold is released and the branch deleted as it closes, by queue and by turn alike. It makes no `Verification` row, is never red, and no bound applies to it (`../domain/after-landing.md`, section 6, and `../domain/after-landing-storage.md`, section 1).
- **The closer's *Land*** waits for its landing's result at landed (`../domain/forge/branches.md`, section 2), so a goal ends as soon as its branch lands into its base.
- **Its dependents start then.** A goal waiting on it, in a milestone or not, starts at its landing. Holding them for verification would protect nothing: the change is on main already, under every goal started after it.
- **The tiers judge it as any landing:** each landing of temper's is in a run's range by its commits, bisected and blamed by commit and key as one temper didn't make (`../domain/after-landing-runs.md`, section 2, and `../domain/after-landing.md`, section 5.2). Its members come from the landing log (`after-landing-repairs.md`, section 10 there), each with its pull request and, new here, its goal and that goal's milestone, so that blame names them after the change's row is gone. The log keeps each tier's verdict, and keeps a landing until every tier following its branch has judged it, within `landings_pending` (stock 64). Past that bound, the oldest pending landing is judged with no member to comment on. The forge still shows each verdict's status and comments on each member's pull request (`after-landing-repairs.md`, section 9 there).
- **No red news reaches a goal from main's tiers:** a red is a finding (section 4), so a goal that has ended and one still live hear of it the same way, through triage. Until that is built, a red is filed once per landing, as a red of no goal's is today (`../domain/after-landing.md`, section 7).
- **A tier following goal branches** is unchanged: a change into a goal's own branch is cleared as it lands, and its reds are news to the goal (`../domain/after-landing.md`, 5.4).
- **The settings together:** `cleared` is today's, with reds as news to goals and a red of no goal's a finding; under `landed` every red of main's tiers is a finding (section 4). Repairs (section 8) take only findings filed under `landed`.

## 4. Every red is a finding

- **Judged as today** (`../domain/after-landing.md`, 5.2): the failed tests are rerun once alone, and those that pass are flakes. Those that fail again are inherited where the previous judged run failed them too, and new otherwise. A new one is owned by the range's one landing, or by the first landing whose commit fails it, found by bisecting. Under `tell_all`, every landing in the range is suspected of it.
- **Filed under `landed`,** by the root in the decision that keeps the verdict: the tests that failed again and that no uncleared red finding of the tier's repository and tier holds, nor a filing waiting on the tier's record (section 5). New ones owned by a landing make that landing's finding. Suspected ones make one finding naming every suspect. Inherited ones make one unblamed finding. So:
  - a run whose failures are all held files nothing;
  - a failure is in one finding at most, however many runs fail it;
  - the first run under `landed` files what was failing before it, unblamed.

  This replaces the red of no goal's filed per landing (`../domain/after-landing.md`, section 7) for that project.
- **Its blame,** kept on its record:
  - each landing it names, by commit, with its members' goals, each goal's milestone where it has one, and each goal's person, whom its own questions reach: its requester where that is a party, else its milestone's maker or its chat's person (`../domain/milestones.md`, section 5), all from the landing log (section 3);
  - its strength: *owned* ("proven", in the person's words: bisected, or the range's one landing), *suspected* (one of several under `tell_all`), or *none*.

  A landing temper didn't make has no goal, so its finding names the landing alone. The triager may move the blame (section 7).
- **Its signature:** kind `red`, severity `high`, no area but a follow's owner's (`../domain/forge/follows.md`, section 6). Its signature is the repository, the tier, its ids in byte order and the first line of a failed job's failed step (`../domain/findings.md`, sections 2 and 4).
- **Its title,** within 200 bytes, saying in one line what is wrong:
  - `K tests fail in TIER since GOAL landed`, GOAL the goal's id where its title has one (`../client/pages/work.md`, 1, *A goal's id*), else `goal N`;
  - `since COMMIT landed` for a landing with no goal, its commit's first twelve digits;
  - `since one of J landings` where it is suspected;
  - `K tests fail in TIER from before run N` where it is unblamed.
- **Its body,** within its bound:
  - the run's link and its artifact's name, the branch, and the range's first and last commits;
  - each blamed goal's title, pull request, `Draft:` path where its words name one, and person, so that the triager reads the culprit without a read of its own;
  - each failing test's id;
  - the failed jobs' logs' ends, as a red's news has them (`../domain/after-landing.md`, section 7);
  - then `and K more` past the bound. The record keeps every id.
- **Flakes are filed:** a test that passed alone is filed as a `flake` finding of the tier, or counted on the open one whose ids hold it (`../domain/findings.md`, section 4). It never enters a red finding.
- **`failing_tests` is 64** in the stock profiles. A red finding's signature holds as many ids as the tier's `failing_tests`, each within `test_id_bytes`. A start refuses a `failing_tests` above the signature's 64 or a `test_id_bytes` above its 512 (`../domain/findings.md`, section 4; `../protocol/configuration/limits.md`), so no id is ever cut.
- **Many:** past `failing_tests`, a run's failures are many, blamed by landing alone (`../domain/after-landing-runs.md`, section 3, and `../domain/after-landing.md`, section 5.2).
  - A landing that owns many gets a finding whose signature says `many` in place of ids.
  - A many run that blames no landing files one unblamed many finding, where no many finding of the tier is uncleared.
  - At the next enumerated run, an uncleared many finding takes the ids that no other uncleared finding holds. Its blame then turns suspected, since many may hide inherited failures, and from then on it is a red finding like any other. Given no ids, it is cleared.
  - A many run clears nothing and judges no repair (section 8): it can't say which tests failed.

## 5. Cleared by a run that passes it

- **Cleared by observation:** a judged run of its tier that fails none of its ids clears it, at any state, counting those that passed alone. Its record keeps the run and the time, its issue is closed with a comment naming the run, and it leaves its holder's list. It never reopens: a later failure of its tests is new.
- **Its repair is named, not trusted:** the run's comments on the blamed landing's members say `Repaired by #M` where a landing in the run's range names the finding by `Fixes #n` or `Fixes-finding: n` (`../domain/after-landing.md`, section 9). That name clears nothing by itself (`../domain/findings.md`, section 7), since a fix that lands can leave the failure failing.
- **Uncleared, it holds its ids** at any state: `new`, `routed`, `backlog` or `dropped`. A red finding filed under `landed` is open until cleared, where any other finding is open until dropped (`../domain/findings.md`, section 4). A finding closed on the forge by another hand is dropped (section 3 there) and stays uncleared. So a dropped red files nothing more until a run passes its tests.
- **Never evicted while uncleared:** filing makes room by evicting old findings (`../domain/findings.md`, section 9), but never an uncleared red one. Where no room can be made, the filing is refused, and its findings wait on the tier's record, their blame kept, as a red of no goal's waits today until findings take it (`../domain/after-landing-storage.md`, section 1). They are filed at the next run that finds room. The tier's record counts them, and the run's report says so.

## 6. Shown on the goal and its milestone

- **A goal's reds:** its page (`../client/pages/goal.md`, section 2) and `goal show` list the red findings blamed on it, uncleared first. Each line gives the number, the title and where it stands: `waiting for triage`, `being repaired by goal G`, `in the backlog`, `dropped: REASON` or `cleared by TIER run N`. The goal's own state is unchanged: done stays done, and the red belongs to the record. The page reads them from the forge's `findings` subject, filtered by the goal they blame (`../client/forge/records.md`).
- **A milestone's reds:** `MilestoneTotals` (`../domain/milestones.md`, section 7) gains `reds`, the uncleared red findings blamed on its goals, counted at the read from the project's findings. Its headline adds `; R reds after landing owed.` (`../client/pages/milestone.md`, section 2), where Home's row and Work's card draw it, and the status agent's line adds `; R reds owed` (`../domain/milestones.md`, section 9).
- **The milestone isn't held for them:** complete still means its goals ended done, as today (the person, 2026-11-18, for v0.1.0; held later, section 12).
- **A finding's own reads:** `finding show` names its blame, its strength and its goals, and `finding list` filters by goal and by milestone.

## 7. The triager sorts each red

- **It holds each red** filed with no area (`../domain/areas.md`, section 5) and hears it on the project's findings topic. It reads the failures and their logs, the blame, and the culprit goal's words and draft, then sorts each finding, or each group that one act settles (section 6 there). A group is the findings of one culprit where it can be, with those that have no culprit together.
- **Routine,** a bug with a clear repair within what the culprit was asked to do: a repair goal (section 8), with no one asked. The triager calls `decided`, kind `findings`, under the project's item for routine repairs, so that its person reads the choice under Decided for you. The stock tables of Agents review and Draft first hold that item at `Decide`, with the words "a repair goal for a routine red after landing" (`../domain/forge/attention.md`, section 6). A project whose classes lack the item asks its person as for any finding (the person, 2026-11-18: repairs need no approval).
- **Needs a product decision,** where the red shows something the culprit's request didn't foresee: two wants at odds, a test and the request disagreeing, or a repair that would change what a person sees beyond the request.
  - It is asked of the culprit's person (section 4's blame), with a person task as `need ask` makes one (`../client/ctl/tools.md`, section 3): the question in one sentence, then each choice it sees on its own line, its recommendation first.
  - It is asked of the triager's own person where its role may address no other, or where the finding names no person.
  - The triager carries the answer out with the finding's acts: a repair goal whose words carry the decision, the backlog, or a drop.
  - A repair goal's own question that needs a product decision goes the same way.
- **A flake,** a failure of load or of the machine that happened to fail again alone: dropped, saying so. Since it stays uncleared, it files nothing more until a run passes it.
- **Blamed wrongly:** the act `blame` moves a finding's blame to a landing of the tier's branch that it names, or to none, with why in a line. Its strength becomes *moved*, and the move's author and why are kept. The finding is then sorted as above. Suspected blame is confirmed or moved before any repair.
- **A culprit still live,** a goal whose changes land straight into main, may have its red routed to it (`../domain/findings.md`, section 6), as today's news would have reached it.
- **Its words:** charter 14's instructions (`../protocol/configuration/stock.md`) gain these sorts, one a finding, and the act `blame`.

## 8. Repair goals

- **`repair`** is an act of the `finding` tool, and of `temper ctl finding repair FINDING... TEXT`, by the findings' holder. It makes one goal for one or more red findings of one repository, each filed under `landed`, uncleared and `new`. It is one decision: the goal is made as a keyed effect, so that a restart makes it once, and each finding is routed to it and its attempt counted.
- **`repair` is refused,** naming why, where:
  - kind 4 has no `repairs`;
  - `repair_goals` repair goals are live, naming them;
  - a finding is past `repair_attempts`, or its last repair goal was cancelled: `ask before repairing finding P/N again`;
  - the findings are in two repositories;
  - a finding isn't an uncleared red filed under `landed`;
  - the triager's period's budget is short of the goal's, naming both, since money waits for its person (`../domain/agent/standing.md`, section 5).
- **The goal** is `repairs.goal` in kind 4, which a start checks is complete: its charter and executor, its models, its priority and its budget, a goal with its own branch and closer as `goal start` makes one (`../domain/forge/branches.md`, section 2). Its repository is the findings', and its deadline `repair_deadline`, past which it ends failed. It is the triager's delegate, funded from the triager's period's budget, so the triager is repairs' one home in the project and its budget is theirs. The goal's questions reach the triager, which answers them as its own work's.
- **Its words:** a title line, `Repair K tests failing in TIER`, at most 60 characters; a blank line; the triager's words, its sort and, where a person decided, the answer; then each finding, oldest first, with its number, title, attempt and body, and, where it was tried before, the tests still failing and the goal before. They are cut to the goal's words bound, with `and more: finding P/N` naming the rest. The goal names each finding by `Fixes-finding: n`, and lands as any goal does, ending at landing.
- **Judged at its end:**
  - a goal that ended done or failed having landed is judged by the first judged run of the tier whose tip holds its last landing;
  - one that ended without landing is judged at once, by the tier's last judged run, since nothing it did reached the branch;
  - a many run judges nothing, and the next enumerated run does;
  - a held tier judges at its release.

  Each finding the run fails none of is cleared (section 5). Each that still fails goes back to its holder, `new`, its tests still failing on its record, with why: `repair goal G ended and TIER run M still fails K of its tests`. A cancelled goal's findings go back at once, with why: `repair goal G was cancelled`.
- **Then a person:** past `repair_attempts`, or after a cancel, `repair` refuses the finding. The triager then asks, as for a product decision (section 7), or backlogs or drops it. A goal it starts on the answer, with `goal` and `route`, is judged at its end as a repair goal is, and counts in no cap.
- **Any goal a red is routed to** is judged in the same way at its end, the culprit's live goal included.
- **The cap:** at most `repair_goals` repair goals live per project. Each one's result wakes the triager, which then starts what waits.

## 9. Settings and bounds

- **Kind 4** (`../domain/landing-settings.md`, 1.1) gains `wait` (`cleared`) and `repairs` (none; when set, `repair_goals` 2, `repair_attempts` 2, `repair_deadline` 24 hours, and `goal`). Each is an added field with its default, taken at the next start as the kind's others are. The kind refuses `repairs` without `wait: landed` and findings on.
- **`forge.tiers.failing_tests`** is 64 (`../protocol/configuration/limits.md`): a tier's record keeps at most that many ids of its last judged failures and of its flakes, within `test_id_bytes`. **`forge.landings_pending`** is 64.
- **A finding's record** gains:
  - its blame: its landings, each with its commit, goals, milestones and persons; its strength; and a move's author and why;
  - its clearing run and time;
  - its repair: its goal, its attempts from 0 to `repair_attempts`, whether its last goal was cancelled, and its tests still failing.

  Its signature, kept as `../domain/findings.md`, section 4 has it, may say `many`.
- **The landing log's landing** gains each member's goal and milestone, and each tier's pending state (section 3).
- **The forge's records:** the `findings` subject gains filters by goal and milestone; `MilestoneTotals` gains `reds`; `TriageFinding` gains the acts `repair` and `blame` (`../client/forge/records.md`).
- **The stock:** the classes item of section 7, and charter 14's sorts.
- **Bounds:**
  - live repair goals: at most `repair_goals` per project;
  - findings: within `forge.findings_per_project`, as any finding is;
  - a finding's blame: at most `landings_pending` landings, each with at most `landing_members` goals.

## 10. Upgrades

- **Stored:** kind 4's two members; the finding's blame, clearing and repair; the landing log's members' goals and pending verdicts; and the tier record's larger failures and waiting filings. Each is read from its previous layout with its defaults: `cleared`, no repairs, no blame, uncleared, no repair, and verifying (`../protocol/store.md`, sections 2 and 4). A red finding filed before reads as it was.
- **Wires:** the worker sees a repair goal as any goal. `MilestoneTotals`' `reds`, the `findings` subject's filters and the two acts come in the client's next layout. The finding tool's `act` enumeration grows, which moves the host tools' contract as `findings` moved `decided`'s, each older version's adapter handing its calls on (`../domain/agent/tools.md`).
- **Switching a project** from `cleared` to `landed`, at a start: its changes verifying or red end landed and uncleared, as a cancel would end them, and their later verdicts count through the landing log (`../domain/after-landing.md`, section 6). Each red landing's `Verification` row first files its finding, blamed on that landing, so that its blame isn't lost.

## 11. Acceptance

Focused and fuzzy suites alone, each failing first:

- **Ending at landing:**
  - under `landed`, a change into main ends at landed with no `Verification` row, and its goal's closer ends the goal at landing;
  - a goal waiting on it in a milestone starts then;
  - a later red run blames that landing by commit, with its status and comments posted on its members' pull requests;
  - a landing pending past `landings_pending` is judged with no comment;
  - under `cleared`, today's stories are unchanged.
- **Switching:** switching a project to `landed` ends its verifying and red changes landed and uncleared, and files each red landing's finding blamed on it.
- **Filing:**
  - a red run of one landing files one owned finding naming its goal, milestone and person;
  - bisected over three landings with two culprits, it files two findings;
  - under `tell_all`, it files one suspected finding naming every suspect;
  - inherited failures that no finding holds make one unblamed finding, and a run failing only held tests files none;
  - the first run under `landed` files the earlier failures unblamed;
  - a landing temper didn't make gets a finding with no goal;
  - a flake is filed or counted, never put in a red finding.
- **Bounds and many:**
  - 64 failing tests are enumerated, and a larger `failing_tests` is refused at start;
  - a many run files a many finding;
  - the next enumerated run gives it only the ids no other finding holds, its blame suspected.
- **Clearing:**
  - a judged run that fails none of a finding's ids clears it, at any state, dropped included;
  - a many run clears none;
  - an uncleared red finding is never evicted;
  - a filing refused for room waits on the tier's record and is filed, with its blame, at a later run.
- **Shown:**
  - `goal show` and the `findings` subject filtered by goal list a goal's reds and where each stands;
  - `MilestoneTotals`' `reds` counts its goals' uncleared reds, while a complete milestone stays complete;
  - the milestone headline's words and the status agent's line say the reds owed.
- **Repairs:**
  - `repair` makes one goal of `repairs.goal`'s shape for two findings, its title line within 60 characters, its words naming each finding by `Fixes-finding: n`;
  - it is refused at `repair_goals` live, past `repair_attempts`, after a cancel, across two repositories, and with `repairs` off;
  - a restart mid-decision makes the goal once.
- **Judging:**
  - a repair goal that landed, whose run still fails its tests, sends its finding back `new` with its why;
  - one ended without landing is judged at once;
  - a cancelled one's findings go back at once;
  - a goal past `repair_deadline` ends failed and is judged;
  - a finding routed by `route` is judged at its goal's end.
- **Blame:** `blame` moves a finding's blame to another landing or to none, and its goals' and milestones' counts follow.
- **Stock:** the Agents review and Draft first tables hold the routine-repair item at `Decide`.
- **Fuzzy:** random runs, landings, goal ends, restarts, triage acts and blame moves keep:
  - every failing id in at most one uncleared red finding of its tier;
  - live repair goals within `repair_goals`;
  - each finding's attempts within `repair_attempts`.

## 12. Later

- **Repairs in the culprit's milestone:** a red blamed on a goal of a milestone repaired by a goal beneath that milestone, its maker answering and its budget paying, and the milestone complete only once its landings are verified and their reds cleared.
- **A dependency that waits for verification,** as a goal's choice, for a project whose next goal must build on verified work.
- **QA on the same path:** a QA finding blamed on the landing whose case found it, suspected, since a case's landing may not be its cause, which the triager confirms or moves (`landed-qa.md`, section 8).
- **A revert as a red's option:** the culprit's landing reverted, where nothing landed since builds on it.
- **Main's reds for goals' checks:** a goal's branch carries main's failures, which its check could pass as inherited from the red findings, once a tier's test ids map to the check's keys (`branch-reds.md`, section 12).
- **A repair goal's own tier run,** to see its fix pass before it lands, rather than at the next run after.
