# after-landing-repairs: a goal ends at landing, and temper repairs what after landing finds

After landing today is a gate for goals: a landing's change lives until every tier following its branch clears it, its goal waits for that, and a red reaches the goals whose landing it is blamed on, which repair it one by one (`../domain/after-landing.md`, sections 6 and 7). This draft ends a goal as soon as its branch lands, and has temper repair what after landing finds by itself, in bulk: each judged run's new failures become one finding, repaired by a goal temper starts, whatever caused them. A failure already being repaired starts nothing more, and a repair that doesn't repair is tried once more, then left to triage (the person, 2026-11-18).

## 1. In one page

- **A goal ends at landing** (section 3), where its project says so: a change into a branch a tier follows ends at landed, and its goal's closer waits for landed, not cleared. The tiers still judge each landing, as they judge a landing temper didn't make.
- **A run's new failures** (section 4) are those that fail again alone and that no unclosed red finding holds, by test id. Passing alone, a failure is a flake, filed as one.
- **One red finding per run** (section 5) holds the run's new failures, with each one's blame as context. It is cleared when a judged run fails none of them, which is not the same as being dropped, and it is never evicted while uncleared.
- **Repair goals** (section 6) are made by temper beneath the project's repairs milestone, which a person makes with its budget and names in the project's settings: that is the standing approval. At most `repair_goals` live at once (2 on the reference deployment); a free place takes every waiting finding into one goal.
- **A repair that doesn't repair** (section 7) gets one more goal, up to `repair_attempts` (2), then goes to triage; a person's cancel sends it to triage at once.
- **A failing test's cap** (sections 4 and 8) rises from 16 to 64, so that new failures stop hiding behind old ones.

## 2. Where it starts

- **A landing is verifying until cleared,** its change alive and its goal waiting (`../domain/after-landing.md`, section 6; `../domain/forge/branches.md`, section 2, the closer's *Land*). Past the project's `verification_bound` it ends landed and uncleared; a change already red has no bound and waits until a later run passes the failures blamed on it. No setting turns the wait off: `../domain/after-landing.md`, section 13 leaves open whether a goal may close before the bound.
- **A red goes to the goals of the landing it is blamed on** (`../domain/after-landing.md`, section 7), new failures blamed on the range's one landing or bisected to the first that fails them (5.2). A red no goal can hear is a finding of its project, once per landing, deduplicated by landing, never by test (`../domain/findings.md`, section 4). temper never starts a repair goal, and nothing in temper groups findings: grouping is the triager's judgement (`../domain/areas.md`, section 6).
- **At most `failing_tests` per run, 16:** past it, a run's failures are many, blamed by landing alone, and many after many counts as nothing new (`../domain/after-landing.md`, sections 4 and 10).
- **What the reference deployment showed** (2026-11-17 and 18): its main's tier failed 28 to 48 tests for hours, so new failures hid behind old ones. To stop goals waiting, its project's `verification_bound` was cut to one minute, before any tier run (15 to 20 minutes) can judge, which frees no change already red: four were held red until cancelled by hand. Blame went astray twice, once on failures left by an earlier change that the blamed one couldn't reach, and once on load flakes of a loaded worker that failed again alone. Meanwhile repairs were made by hand, one repair goal per run's new failures, and a later run's new failures were added to the open goal by message.

## 3. A goal ends at landing

- **Kind 4's `wait`**, `cleared` (today's) or `landed`, per project (`../domain/landing-settings.md`, 1.1).
- **Under `landed`, a change ends at landed:** its result is given at its landing, naming the landing commit and saying that after landing judges it later; its branch's hold is released and the branch deleted as it closes, by queue and by turn alike. It makes no `Verification` row, is never red, and no bound applies to it (`../domain/after-landing.md`, sections 6 and 10).
- **The closer's *Land*** waits for its landing's result at landed (`../domain/forge/branches.md`, section 2), so a goal ends as soon as its branch lands into its base.
- **The tiers judge it as any landing:** each landing of temper's is in a run's range by its commits, bisected and blamed by commit and key as one temper didn't make (`../domain/after-landing.md`, sections 3 and 5.2). Its members and their pull requests come from the landing log (section 10 there), which keeps each tier's verdict and keeps a landing until every tier following its branch has judged it, within `landings_pending` (stock 64); past it, the oldest pending landing is judged with no member to comment on. The forge still shows each verdict's status and comments on each member's pull request (section 9 there).
- **No red news reaches a goal of main's tiers:** the run's new failures go to a red finding (section 5). Until that is built, a red is filed once per landing as a red of no goal's is today (`../domain/after-landing.md`, section 7).
- **A tier following goal branches** is unchanged: a change into a goal's own branch is cleared as it lands, and its reds are news to the goal (`../domain/after-landing.md`, 5.4).
- **The settings together:** `cleared` with repairs off is today's; `landed` with repairs off files each run's red finding for triage; `landed` with repairs on repairs them (section 6). Repairs on with `cleared` is refused, since a red would go both to the landing's goals and to a repair.

## 4. A run's new failures

- **Judged as today first** (`../domain/after-landing.md`, 5.2): the failed tests are rerun once alone; those that pass are flakes, the rest fail again, and new ones are blamed on the range's one landing or bisected over its landings, as context now, never as an addressee.
- **New, for repairs:** a test that failed again and that no uncleared red finding of the tier's repository and tier holds (section 5), whatever its blame and whether or not the previous judged run failed it. So the first run under `landed` takes every failure there is, and an inherited failure is held by the finding that first took it. A red finding filed per landing before this has no signature, and holds nothing.
- **Flakes are filed:** a test that passed alone is filed as a `flake` finding of the tier, or counted on the open one whose ids hold it (`../domain/findings.md`, section 4). It never enters a repair.
- **`failing_tests` is 64** in the stock profiles. A red finding's signature holds as many ids as the tier's `failing_tests`, each within `test_id_bytes`, so a start refuses a `failing_tests` above the signature's 64 or a `test_id_bytes` above its 512 (`../domain/findings.md`, section 4; `../protocol/configuration/limits.md`); no id is cut.
- **Many:** past `failing_tests`, a run's failures are many, as today. A many run files a many finding of the tier where none is uncleared: its signature says `many` in place of ids. While one is uncleared, the next enumerated run gives it the ids no other uncleared finding holds, and it is a red finding like any from then on; given none, it is cleared. A many run clears nothing and judges no attempt (section 7): it can't say which tests failed.

## 5. The red finding

- **Filed by the root,** in the decision that keeps the verdict, once per judged run with new failures, under `wait: landed`: kind `red`, severity `high`, no area, its signature the repository, the tier, the new failures' ids in byte order and the first line of a failed job's failed step (`../domain/findings.md`, sections 2 and 4). It replaces the red of no goal's filed per landing (`../domain/after-landing.md`, section 7) for that project.
- **Its title,** `Red after landing on BRANCH: TIER run N, K new`. **Its body,** within its bound: the run's link and its artifact's name; the range, its first and last commits; each new failure's id, with its blame as context, owned by a landing, suspected of several or unblamed (its commit and pull request, from the landing log); the failed jobs' logs' ends, as a red's news has them; then `and K more` past the bound, the record keeping every id.
- **Cleared by observation:** a judged run of its tier that fails none of its ids, those passing alone included, clears it, at any state: its record keeps the run and the time, its issue is closed with a comment naming the run, and it never reopens; a later failure of its tests is new. A `Fixes #n` or `Fixes-finding: n` names it from the repairing change and clears nothing by itself (`../domain/findings.md`, section 7): a fix that lands cleared can leave the old failure failing.
- **Uncleared holds its ids,** whatever its state: waiting for a repair, being repaired, handed to triage, in the backlog or dropped. Dropped, it starts no goal, and its tests start none either until a run passes them. A red finding with a signature is open until cleared, where any other finding is open until dropped (`../domain/findings.md`, section 4), and a finding closed on the forge by another hand is dropped (section 3 there), still uncleared.
- **Never evicted while uncleared:** filing makes room by evicting old findings (`../domain/findings.md`, section 9), never an uncleared red one. Where nothing can go, the filing is refused: the tier's record counts it and the run's report says so, and its tests, held by no finding, are new again at the next run, which files them once there is room.

## 6. Repair goals

- **The repairs milestone** is the standing approval: a milestone of the project (`../domain/milestones.md`) a person makes with the budget repairs may spend, and names in kind 4's `repairs` (the person, 2026-11-18). Its maker requests every repair goal, so its inbox hears their questions; its budget funds them; concluding or cancelling it stops repairs, and a settings change naming none turns them off. temper makes its goals on the maker's standing grant, a delegation jig checks as the maker's own: its authority, the milestone's budget and its delegation, a refusal reported (`../../../../jig/docs/design/domain/tasks/gatherings.md`, section 1).
- **`repairs`' other members:** `repair_goals`, the most live beneath the milestone at once (stock 2); `repair_attempts` (stock 2, section 7); `repair_deadline` (stock 24 hours), past which a repair goal ends failed; and the goal itself, which a start checks is complete: its repository, its charter and executor, its models, its priority and its budget, a goal with its own branch and closer as `goal start` makes one (`../domain/forge/branches.md`, section 2).
- **A finding's repair** is kept on its record, in one of these phases:

| Phase | Its state | Holds a place | Leaves |
|---|---|---|---|
| waiting | `new` | no | to active when a goal admits it; out of repairs at any triage act (`../domain/findings.md`, section 6) |
| active | `routed` to its goal | yes | to judging when the goal ends done or failed; to triage when it is cancelled |
| judging | `routed` to its ended goal | no | at its judging run (section 7): cleared, waiting again, or to triage |
| triage | `new`, handed to the triager | no | as any finding |

- **Admission:** while fewer than `repair_goals` repair goals are live, the root makes one goal for every waiting finding of the project, oldest first, in one decision with its place, the findings' moves to active and each one's attempt counted; the goal is a keyed effect, so a restart makes it once. A refused admission leaves them waiting, the refusal's words kept on each. Admission is tried at each filing, each repair goal's end, each start and each change of the settings, so a finding filed while repairs were off, or refused once, is taken when it can be. A later run's new failures never join a live goal (the person, 2026-11-18).
- **Its words:** a plain title line, `Repair K reds after landing on BRANCH` (at most 60 bytes), a blank line, then each finding's number, title, attempt and body, oldest first, cut to the goal's words bound with `and more: finding P/N` naming the rest. It repairs them by changes like any goal's, naming each finding by `Fixes-finding: n`, and lands as any goal does, ending at landing.

## 7. A repair that doesn't repair

- **Judged once per goal's end:** a goal that ended done or failed having landed is judged by the first judged run of the tier whose tip holds its landing. One that ended without landing is judged at once by the tier's last judged run, since nothing it did reached the branch. A many run judges nothing, and the next enumerated run does. A held tier judges nothing until released, and holds no place meanwhile.
- **Its findings then:** each one the run fails none of is cleared (section 5). Each that still fails some has its tests still failing named on its record; with fewer than `repair_attempts` attempts, it waits again, its next goal's words naming the tests still failing and the goal before it.
- **Then triage:** past `repair_attempts`, it is handed to its project's triager (`../domain/findings.md`, section 6; `../domain/areas.md`, section 5), `new`, with why: `N repair goals ended and TIER run M still fails K of its tests`. A cancelled repair goal hands its findings there at once, with why: `repair goal G was cancelled`. Either way the finding stays uncleared, so its tests start nothing more until a run passes them or a person acts.

## 8. Settings and bounds

- **Kind 4** (`../domain/landing-settings.md`, 1.1) gains `wait` (`cleared`) and `repairs` (none, with section 6's members), each an added field with its default, taken at the next start as the kind's others are. It refuses `repairs` with `cleared`, and a milestone that isn't a live one of the project's.
- **`forge.tiers.failing_tests`** is 64 (`../protocol/configuration/limits.md`): a tier's record keeps at most that many ids of its last judged failures and of its flakes, within `test_id_bytes`. **`forge.landings_pending`** is 64.
- **A finding's record** gains its repair: its phase, its goal, its attempts from 0 to `repair_attempts`, a refused admission's words, its tests still failing, and its clearing run and time. Its signature, kept as `../domain/findings.md`, section 4 has it, is now a red's too, or `many`.
- **The landing log's landing** gains each tier's verdict and its pending state (section 3).
- **Bounds:** live repair goals at most `repair_goals` per project; repairs' findings within `forge.findings_per_project`, as findings are.

## 9. Upgrades

- **Stored:** kind 4's two members, the finding's repair and clearing, the landing log's verdicts, and the tier record's larger failures, each read from the previous layout with its defaults: `cleared`, no repairs, no repair, verifying (`../protocol/store.md`, 2 and 4). A finding filed per landing before keeps reading as it was, with no signature.
- **No wire moves:** the worker sees a repair goal as any goal. ctl's finding reads add the repair's phase, goal and attempts, in the client's next layout.
- **Switching a project,** `cleared` to `landed`, at a start: changes verifying or red then end landed and uncleared, as a cancel would end them, and their later verdicts count through the landing log (`../domain/after-landing.md`, section 6).

## 10. Acceptance

Focused and fuzzy suites alone, each failing first:

- under `landed`, a change into main ends at landed with no `Verification` row; its goal's closer ends the goal at landing; a later red run blames that landing by commit, its status and comments posted on its members' pull requests; a landing pending past `landings_pending` judged with no comment; under `cleared`, today's stories unchanged;
- switching a project to `landed` ends its verifying and red changes landed and uncleared;
- a judged red run files one red finding of its new failures; a run failing only what uncleared findings hold files none; the first run under `landed` takes every failure; a flake is filed or counted, never in a finding of repairs; a dropped finding's tests start nothing until passed;
- 64 failing tests enumerated, a larger `failing_tests` refused at start; a many run files a many finding, which the next enumerated run gives only the ids no other finding holds;
- a judged run failing none of a finding's ids clears it, at any state; a many run clears none; an uncleared red finding never evicted, a filing refused for room counted and filed at a later run;
- with `repairs`, a goal made beneath the milestone, requested by its maker, its words' title line within 60 bytes; at `repair_goals` live, findings wait; a goal's end admits one goal for every waiting finding; a later run's new failures never join a live goal; a restart mid-admission makes the goal once; a refused admission retried at the next start;
- a repair goal ended having landed, the first run covering its landing still failing its tests: a second goal; one ended without landing judged at once; after `repair_attempts`, the finding handed to triage with its why; a cancelled one handed there at once; a goal past `repair_deadline` ends failed and is judged;
- fuzzy: random runs, landings, goal ends, restarts and triage acts keep every failing id in at most one uncleared red finding of its tier, live repair goals within `repair_goals`, and each finding's attempts within `repair_attempts`.

## 11. Later

- **Main's reds for goals' checks:** a goal's branch carries main's failures, which its check could pass as inherited from the red findings, once a tier's test ids map to the check's keys (`branch-reds.md`, section 12).
- **A repair goal's own tier run,** to see its fix pass before it lands, rather than at the next run after.
