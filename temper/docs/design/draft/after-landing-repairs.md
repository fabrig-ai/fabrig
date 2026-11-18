# after-landing-repairs: a goal ends at landing, and temper repairs what after landing finds

After landing today is a gate for goals: a landing's change lives until every tier following its branch clears it, its goal waits for that, and a red reaches the goals whose landing it is blamed on, which repair it one by one (`../domain/after-landing.md`, sections 6 and 7). This draft ends a goal as soon as its branch lands, and has temper repair what after landing finds by itself, in bulk: each judged run's new failures become one finding and one repair goal, whatever caused them. A failure already being repaired starts nothing more, and a repair that doesn't repair is tried once more, then left to triage (the person, 2026-11-18).

## 1. In one page

- **A goal ends at landing** (section 3), where its project says so: a change into a branch a tier follows ends at landed, and its goal's closer waits for landed, not cleared. The tiers still judge each landing, as they judge a landing temper didn't make.
- **A run's new failures** (section 4) are those that fail again alone and that no open repair holds, by test id. Inherited failures, those being repaired and those whose repair has run out of attempts are never new again. Passing alone, a failure is a flake, filed as one.
- **One repair finding per run** (section 5): a red finding whose signature is the run's new failures, carrying the run, its range and each failure's blame as context. It is closed when a judged run fails none of its tests.
- **One repair goal per finding** (section 6), started by temper on the project's standing approval, from its settings: at most `repair_goals` live at once (2 on the reference deployment). A finding past the cap waits, and the next free place takes every waiting finding at once.
- **A repair that doesn't repair** (section 7): when a repair goal has ended and the first judged run that covers its end still fails some of its finding's tests, the finding gets another goal, up to `repair_attempts` (2); then it goes to triage.
- **A failing test's cap** (section 4) rises from 16 to 64, so that new failures stop hiding behind old ones.

## 2. Where it starts

- **A landing is verifying until cleared,** its change alive and its goal waiting (`../domain/after-landing.md`, section 6; `../domain/forge/branches.md`, section 2, the closer's *Land*). Past the project's `verification_bound` it ends landed and uncleared; a change already red has no bound and waits until a later run passes the failures blamed on it. No setting turns the wait off: `../domain/after-landing.md`, section 13 leaves open whether a goal may close before the bound.
- **A red goes to the goals of the landing it is blamed on** (`../domain/after-landing.md`, section 7), new failures blamed on the range's one landing or bisected to the first that fails them (5.2). A red no goal can hear is a finding of its project, once per landing, deduplicated by landing, never by test (`../domain/findings.md`, section 4). temper never starts a repair goal.
- **At most `failing_tests` per run, 16:** past it, a run's failures are many, blamed by landing alone, and many after many counts as nothing new (`../domain/after-landing.md`, sections 4 and 10).
- **What the reference deployment showed** (2026-11-17 and 18): its main's tier failed 28 to 48 tests for hours, so new failures hid behind old ones. To stop goals waiting, its project's `verification_bound` was cut to one minute, before any tier run (15 to 20 minutes) can judge, which frees no change already red: four were held red until cancelled by hand. Blame went astray twice, once on failures left by an earlier change that the blamed one couldn't reach, and once on load flakes of a loaded worker that failed again alone. Meanwhile repairs were made by hand, one repair goal per run's new failures, and a later run's new failures were added to the open goal by message.

## 3. A goal ends at landing

- **Kind 4's `wait`**, `cleared` (today's) or `landed`, per project (`../domain/landing-settings.md`, 1.1).
- **Under `landed`, a change ends at landed:** its result is given at its landing, naming the landing commit and saying that after landing judges it later; its branch's hold is released and the branch deleted as it closes, by queue and by turn alike. It makes no `Verification` row, is never red, and no bound applies to it (`../domain/after-landing.md`, sections 6 and 10).
- **The closer's *Land*** waits for its landing's result at landed (`../domain/forge/branches.md`, section 2), so a goal ends as soon as its branch lands into its base.
- **The tiers judge it as any landing:** each landing of temper's is in a run's range by its commits, bisected and blamed as one temper didn't make (`../domain/after-landing.md`, sections 3 and 5.2), its changes and pull requests named from the landing log (section 10 there), where each tier's verdict is kept. The forge still shows each verdict's status and comments on each member's pull request (section 9 there).
- **No red news reaches a goal,** since none waits: a red's new failures are a repair finding (section 5).
- **A tier following goal branches** is unchanged: a change into a goal's own branch is cleared as it lands, and its reds are news to the goal (`../domain/after-landing.md`, 5.4).

## 4. A run's new failures

- **Judged as today first** (`../domain/after-landing.md`, 5.2): the failed tests are rerun once alone; those that pass are flakes, the rest fail again, and new ones are blamed on the range's one landing or bisected over its landings.
- **New, for repairs:** a test that failed again and that no unclosed red finding of the tier's repository and tier holds (section 5), whatever its blame and whether or not the previous judged run failed it. So the first run after `repairs` is turned on takes every failure there is, and an inherited failure is held by the finding that first took it. A red finding filed per landing before this holds no ids, and holds nothing.
- **Flakes are filed:** a test that passed alone is filed as a `flake` finding of the tier, or counted on the open one whose ids hold it (`../domain/findings.md`, section 4). It never enters a repair.
- **`failing_tests` is 64** in the stock profiles, the most a finding's signature holds (`../domain/findings.md`, section 4). Past it, a run's failures are many, as today. A many run whose failures no unclosed many finding of the tier holds files one, with no ids. While one is unclosed, the tier's next enumerated run gives it that run's failures as its ids, rather than filing them new.

## 5. The repair finding

- **Filed by the root,** in the decision that keeps the verdict, once per judged run with new failures, under `wait: landed` or `repairs` on: kind `red`, severity `high`, no area, its signature the repository, the tier, the new failures' ids in byte order and the first line of a failed job's failed step (`../domain/findings.md`, sections 2 and 4). It replaces the red of no goal's filed per landing (`../domain/after-landing.md`, section 7) for that project.
- **Its title,** `Red after landing on BRANCH: TIER run N, K new`. **Its body,** within its bound: the run's link and its artifact's name; the range, its first and last commits; each new failure's id, with its blame as context, owned by a landing, suspected of several or unblamed (its commit and pull request, from the landing log); the failed jobs' logs' ends, as a red's news has them; then `and K more` past the bound, the record keeping every id.
- **Closed by observation:** a judged run of its tier that fails none of its ids, those passing alone included, closes it, naming the run, at any state. A run that is many closes none. A `Fixes #n` or `Fixes-finding: n` names it from the repairing change, and closes nothing by itself (`../domain/findings.md`, section 7): a fix that lands cleared can leave the old failure failing.
- **Unclosed holds its ids** whatever its state: routed to a repair goal, waiting for one, handed to triage, in the backlog or dropped. A dropped red finding starts no goal, and its failures start none either until a run passes them.

## 6. Repair goals

- **Kind 4's `repairs`**, off by default. Turned on by a project owner, it is that owner's standing approval of the goals it starts (the person, 2026-11-18). Its members: `repair_goals`, the most live at once in the project (stock 2); `repair_attempts` (stock 2, section 7); `requester`, the party the goals are requested by, whose inbox hears their questions, by default the owner who turned it on; the goal's charter, models, priority, budget and deadline, by default those a `goal start` of the requester's would take.
- **Started by the root,** in the decision that files the finding, where fewer than `repair_goals` repair goals of the project are live: a goal (template 2) in the project, requested by `requester`, funded from the project's spend, as if that party had started it. The finding is routed to it (`../domain/findings.md`, section 6), and kept on its record with the attempt's number.
- **Its words:** a plain title line, `Repair K reds after landing on BRANCH` (at most 60 bytes), a blank line, then each finding's number, title and body, oldest first, cut to the goal's words bound with `and more: finding P/N` naming the rest. It repairs them by changes like any goal's, naming each finding by `Fixes-finding: n`, and lands as any goal does, ending at landing under `wait: landed`.
- **Past the cap, a finding waits:** routed, with no goal yet, oldest first. When a repair goal ends, the root starts one goal for every waiting finding of the project, so that a slow repair never leaves a queue of goals behind it. A waiting finding is never added to a live goal: a later run's new failures get a goal of their own (the person, 2026-11-18).
- **Triage still acts:** a finding's holder may hand it on, route it to a goal of its choice, put it in the backlog or drop it (`../domain/findings.md`, section 6), and it leaves the repairs then. A person cancels a repair goal as any goal; it counts as an attempt.

## 7. A repair that doesn't repair

- **Judged at the first run that covers its end:** the first judged run of the tier whose tip holds the repair goal's landing or, for a goal that ended without landing, the first judged run after its end. If it fails none of the finding's ids, the finding closes (section 5).
- **Another goal:** if it still fails some of them, and the finding has had fewer than `repair_attempts` goals, it is routed to a new one as a waiting finding is (section 6), its words naming the tests still failing and the goal before it.
- **Then triage:** past `repair_attempts`, it is handed to its project's triager (`../domain/findings.md`, section 6, `../domain/areas.md`, section 5), `new`, with why: `N repair goals ended and TIER run M still fails K of its tests`. It stays unclosed, so its tests start nothing more until a run passes them or a person acts.

## 8. Settings and bounds

- **Kind 4** (`../domain/landing-settings.md`, 1.1) gains `wait` (`cleared`) and `repairs` (off, with section 6's members), each an added field with its default, taken at the next start as the kind's others are.
- **`forge.tiers.failing_tests`** is 64 (`../protocol/configuration/limits.md`): a tier's record keeps at most that many ids of its last judged failures and of its flakes, within `test_id_bytes`.
- **A finding's record** gains its repair: the goal it is routed to, if any, and its attempts, from 0 to `repair_attempts`. Its signature, kept as `../domain/findings.md`, section 4 has it, is now a red's too.
- **Repair goals live** at most `repair_goals` per project, the waiting findings at most `forge.findings_per_project`, as findings are.

## 9. Upgrades

- **Stored:** kind 4's two members, the finding's repair, and the tier record's larger failures, each read from the previous layout with its defaults: `cleared`, repairs off, no repair (`../protocol/store.md`, 2 and 4). A finding filed per landing before keeps reading as it was, with no signature.
- **No wire moves:** the worker sees a repair goal as any goal. ctl's finding reads add the repair's goal and attempts, in the client's next layout.
- **Switching a project,** `cleared` to `landed`, at a start: changes verifying or red then end landed and uncleared, as a cancel would end them, and their later verdicts count through the landing log (`../domain/after-landing.md`, section 6).

## 10. Acceptance

Focused and fuzzy suites alone, each failing first:

- under `landed`, a change into main ends at landed with no `Verification` row; its goal's closer ends the goal at landing; a later red run blames that landing by commit, its status and comments posted on its members' pull requests; under `cleared`, today's stories unchanged;
- switching a project to `landed` ends its verifying and red changes landed and uncleared;
- a judged red run files one red finding of its new failures; a run failing only what unclosed findings hold files none; the first run with `repairs` on takes every failure; a flake is filed or counted, never in a repair; a dropped finding's tests start nothing until passed;
- 64 failing tests enumerated; past it a many finding, which the next enumerated run gives its ids;
- a judged run failing none of a finding's ids closes it, at any state; a many run closes none;
- with `repairs` on, a finding starts a goal requested by `requester`, its words' title line within 60 bytes; at `repair_goals` live, findings wait; a goal's end starts one goal for every waiting finding; a later run's new failures never join a live goal;
- a repair goal ended, the first run covering its end still failing its tests: a second goal; after `repair_attempts`, the finding handed to triage with its why, and no goal more;
- a restart between filing and starting a goal starts it once;
- fuzzy: random runs, landings, goal ends and triage acts keep every failing id in at most one unclosed red finding of its tier, live repair goals within `repair_goals`, and each finding's goals within `repair_attempts`.

## 11. Later

- **Main's reds for goals' checks:** a goal's branch carries main's failures, which its check could pass as inherited from the repair findings, once a tier's test ids map to the check's keys (`branch-reds.md`, section 12).
- **A repair goal's own tier run,** to see its fix pass before it lands, rather than at the next run after.
