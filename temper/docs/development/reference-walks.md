# reference-walks backlog

fab writes this ladder's backlog from its log. Do not edit by hand.

## finding-113: A tier run's hold says 'last: no step' and 'resolve the reason' without naming the failed step or its error

- Rung: rm8
- Walk: walk-50
- Backend: chatgpt-forgejo
- Commit: 8488691cc819e6000e75d4db7a959252a4b27d5a
- Area: tier run task words
- Severity: medium
- Sightings: (none)

### Reproduction

As the node finding: a tier whose workflow faults three times on the runner; temper ctl DIR task show 1.

### Expected

The fault named in words (the failed step actions/checkout@v4 and its error) with the run's link, so the person knows what to resolve.

### Seen

held: Tier slow run 1 held after 3 runner faults, last: no step; way out: resolve the reason, then task release 1 --hold 1. The cause is only in the forge's job log ('Cannot find: node in PATH').

### Triage reason

words on a runner-fault hold, off the happy path (the walk found the cause in the forge's log without stalling); the hold should name the failed step and its error instead of "last: no step"

## finding-115: A start refused for a tier's format says 'invalid form' without naming the accepted forms junit or lines

- Rung: rm8
- Walk: walk-50
- Backend: chatgpt-forgejo
- Commit: 8488691cc819e6000e75d4db7a959252a4b27d5a
- Area: configuration refusal words
- Severity: low
- Sightings: (none)

### Reproduction

Fresh box per kit; add a tier with "format": "ids" to application.landing.projects[0].gates[0].tiers; start temper box --config CONFIG --data DATA --cache CACHE.

### Expected

The refusal names the accepted values, junit or lines.

### Seen

temper box: application.landing.projects[0].gates[0].tiers[0].format: invalid form (the box exited).

### Triage reason

the refusal of a tier's format should name the accepted forms junit and lines; non-blocking, and the walk's stall over the format is finding-114's help gap, whose repair puts the right name in front of the person

## finding-117: project show prints the owner's pool only as 'spend per period', not its reserved, spent and left balances

- Rung: rm8
- Walk: walk-50
- Backend: chatgpt-forgejo
- Commit: 8488691cc819e6000e75d4db7a959252a4b27d5a
- Area: ctl project show
- Severity: low
- Sightings: finding-121

### Reproduction

Fresh box; settings apply raising the limits; temper ctl DIR project fund 1 449696 --key budget-period; temper ctl DIR project fund 1 415104 --role 0 --key budget-owner; temper ctl DIR project show 1.

### Expected

Role 0's pool with its budget, reserved, spent and left, as for the period.

### Seen

role 0, owner: spend per period 415104; parties: party 1 owner; only the period shows budget, reserved, spent and left.

### Triage reason

project show's owner pool lacking reserved, spent and left balances is a budget read, not part of M8's tier story, and blocked nothing

## finding-131: A chat's read_branches with prefix '' answers 'Forge read failed: empty' while the repository has main and temper/tier/slow.1

- Rung: rm8
- Walk: walk-53
- Backend: chatgpt-forgejo
- Commit: 8b15ca51842f5bb3295bbe94783161fa92e7e91b
- Area: chat forge reads
- Severity: low
- Sightings: (none)

### Reproduction

Walk-53 steps 1-5; chat show 2 after its first reply.

### Expected

The branch list, or words saying why none.

### Seen

call read_branches {repository: playground/widget, prefix: ''} -> 'Forge read failed: empty'; serve.log 'read refused ... tool=read_branches layer=forge-client reason=forge read failed error=Some(Empty)'.

### Triage reason

the chat's read_branches with an empty prefix fails with "Forge read failed: empty" instead of listing branches; the chat worked around it and nothing on the walk stalled

## finding-136: A box restart kills the host runner's in-flight tier job; Forgejo marks it failed only ~10 min later, temper reruns it, and the task lists the lost run as a Judged run

- Rung: rm8
- Walk: walk-55
- Backend: chatgpt-forgejo
- Commit: 677e405ca112365653d5d504de259b00fd11a0d4
- Area: box runner / tier run task
- Severity: medium
- Sightings: (none)

### Reproduction

Box with tier slow; while a tier run's job is in its sleep (forge run 7), run give-back, systemctl --user stop the box, then start it again with the same directories.

### Expected

The stop drains or marks the runner's job, or temper names the lost run at the restart; the help warns that a stop loses running jobs; the task does not call a lost run judged.

### Seen

Run 7's job log stops at 08:52:00 after checkout; the box stopped at 08:56:27Z; Forgejo marked run 7 failed at 09:06:49Z; temper dispatched run 12 at 09:06:51Z (green 09:16:53Z). task 7 lists 'Judged run: ... forge run 7' and '... forge run 12'. About 15 minutes lost.

### Triage reason

the restart in the middle of a run came from finding-135's workaround, not from the scenario (its one restart, step 3, is before any run); losing the runner's job and calling the lost run judged is real but off the path

## finding-138: Chats do not tell the person about tier verdicts: the red change's chat never says landed or red; others say merged, not cleared; a held change's chat says nothing

- Rung: rm8
- Walk: walk-55
- Backend: chatgpt-forgejo
- Commit: 677e405ca112365653d5d504de259b00fd11a0d4
- Area: chat / charter 1
- Severity: medium
- Sightings: finding-156, finding-162

### Reproduction

Box with tier slow; chat 'Add the line "BREAK the slow check" to the end of README.md in widget.' and another chat for a third line; follow to the tier's red; chat show each.

### Expected

Each chat tells the person its change landed and whether it was cleared or red, and tells of a hold at once, as charter 1 says.

### Seen

Chat 8: only 'The README change has started.' Chat 2: 'Added ... Pull request #1 was merged into main as 157e049ca6fe.' and chat 11 the same of PR #3, though results say 'Cleared by slow run N.'; chat 11 said nothing of change 13's hold.

### Triage reason

What passes does not include the chats' words, and step 7 only asks the walker to report them; the chat saying 'merged' rather than cleared or red, and nothing of a hold, is the model's reply, off the acceptance; after-landing.md 7's red news to the goal can be checked when the repair is scheduled

## finding-141: Closing the chat of a red change cancels it, and its panel then reads 'landed, uncleared' with the red and its blame gone

- Rung: rm8
- Walk: walk-55
- Backend: chatgpt-forgejo
- Commit: 677e405ca112365653d5d504de259b00fd11a0d4
- Area: chat close / change panel
- Severity: low
- Sightings: finding-157

### Reproduction

After run 3 blames change 9: chat close 8 --key K; change show 9.

### Expected

The panel keeps the red, its run and its failing test after the cancel.

### Seen

chat close 8: done; task 9 Cancelled; change show 9: 'step: landed, uncleared'.

### Triage reason

the panel losing the red and its blame after the chat's close cancels the change happens at clean-up, after the verdicts What passes reads; finding-128's repair (close no longer holds) held

## finding-147: Spend scales disagree: task panel 'spent 0.71 of 2075.52' while activity says spent 71 and the owner's pool says spent 143 for two such chats

- Rung: rm4
- Walk: walk-57
- Backend: claude-forgejo
- Commit: 4ab8fac8a347249c755d2db69b91c75033ec575f
- Area: ctl task show spend words
- Severity: low
- Sightings: (none)

### Reproduction

As finding 1, two chats; task show 1; task activity 1 --json (run_ended spent); chat close both; project show 1.

### Expected

Backlog (off the happy path): one scale for a task's spend across task show, activity and the pool.

### Seen

task show: 'spent 0.71 of 2075.52'; activity run_ended spent 71 (chat 2: 72); project show pool: 'spent 143, left 414961'.

### Triage reason

one scale for a task's spend across task show, activity and the pool is a words mismatch the rung's steps read past; not the same cause as finding-117 (the pool's balances missing from project show)

## finding-148: A second chat's first run is numbered run 2 (turn 2.1) with no attempt 1

- Rung: rm4
- Walk: walk-57
- Backend: claude-forgejo
- Commit: 4ab8fac8a347249c755d2db69b91c75033ec575f
- Area: task activity / chat show words
- Severity: low
- Sightings: (none)

### Reproduction

Fresh box, one chat (task 1), then a second chat start (task 2); chat show 2; task activity 2 --json.

### Expected

Backlog (off the happy path): a new task's first run is attempt 1, turn 1.1.

### Seen

chat show 2: '2 · run 2 began', 'turn 2.1'; activity run_ended attempt 2; task turn 2 1.1 prints 'unknown'. Workspace path .../workspaces/4294967296.

### Triage reason

a second task's first run numbered run 2 / turn 2.1 is a numbering oddity off the rung's What passes; nothing the story needs is blocked by it

## finding-152: settings apply refusals name the wrong item: 'item 0; rules: shape' when item 0 was charter 5 and the rules were item 7

- Rung: rm3
- Walk: walk-56
- Backend: chatgpt-forgejo
- Commit: 4ab8fac8a347249c755d2db69b91c75033ec575f
- Area: settings refusal words
- Severity: low
- Sightings: (none)

### Reproduction

Fresh box; settings apply a file whose items are charter 5, charter 7, charter 2, template 4, landing-rule 1, landing-rule 2, charter 6, rules (9 delegation kinds), policy 1 --against NEWEST.

### Expected

The refusal names the faulty item's index (7) and its setting.

### Seen

'settings apply: out of shape; item 0; rules: shape'; a file whose item 0 was charter 7 gave the same 'item 0; rules'.

### Triage reason

a wrong item index in a refusal's words; the walk found the faulty setting by bisection, and finding-151's repair makes the refusal name its bound

## finding-155: No help says how the person pushes with the box's header files; forge setup's starter commit on main makes the person's push a forced one

- Rung: rm3
- Walk: walk-56
- Backend: chatgpt-forgejo
- Commit: 4ab8fac8a347249c755d2db69b91c75033ec575f
- Area: help box / forge hands
- Severity: low
- Sightings: (none)

### Reproduction

Fresh box; forge setup --org fabrig --repository fabrig; push an existing history to main as the person with DATA/forge-1/person_write.header.

### Expected

The help names a way to push as the person by the header file's path, and says main holds a starter commit.

### Seen

git takes no header file (pushed with GIT_CONFIG_* env from a script); 'git push' refused 'fetch first' because main held bd45bc81 'Add .forgejo/workflows/ci.yml'; forced. help box steps 3 and 4 disagree on whether me lists project 1.

### Triage reason

pushing an existing history as the person is the person's own git and login, not temper's tool, and the walk got through; forge setup's starter commit making that push forced and help box steps 3 and 4 disagreeing on me's project list are words to settle in the backlog

## finding-158: A change cleared by a red run's bisection reads 'step: cleared' naming no run, while one cleared by a green run reads 'cleared · slow run 2'

- Rung: rm8
- Walk: walk-59
- Backend: chatgpt-forgejo
- Commit: 599f62cb9d3e5acb689afc5e8176b06f8660c9f7
- Area: change panel words
- Severity: low
- Sightings: (none)

### Reproduction

As the walk: change 3 cleared by green run 2; change 12 cleared by red run 3 (blame on change 8); change show 3; change show 12.

### Expected

Both panels name the run that cleared them, e.g. 'cleared · slow run 3' (backlog; off the happy path).

### Seen

change 3: 'step: cleared · slow run 2'; change 12: 'step: cleared' (its result does say 'Cleared by slow run 3.').

### Triage reason

panel wording only; the change reads cleared and its result names the clearing run ('Cleared by slow run 3.'), so the walk read the verdict and rm8 passed; not blocking, off the happy path

## finding-159: With qa.md's budget figures each chat reserves 207552 of the owner's 415104 pool, so M8's third concurrent chat is refused 'pool in project 1 had 0 left'

- Rung: rm8
- Walk: walk-59
- Backend: chatgpt-forgejo
- Commit: 599f62cb9d3e5acb689afc5e8176b06f8660c9f7
- Area: QA budget recipe / chat reservation
- Severity: low
- Sightings: finding-161

### Reproduction

Fresh box per kit; settings apply the kit's budget; forge setup; project fund 1 449696, --role 0 415104; two chats live (2 and 7); chat start a third.

### Expected

The kit's budget holds M8's three chats, or the recipe sizes the pool for them (backlog; refusal named its remedy).

### Seen

12:03:35Z chat start: refused: your pool in project 1 had 0 left, 1 needed; project show: pool reserved 415104. Raised rules period to 657248 (settings apply), project fund 657248 and --role 0 622656; the chat then started at 12:04:11Z.

### Triage reason

the refusal named the shortfall, the walker raised the period and pool through settings apply and project fund and the third chat started within a minute; rm8 passed; sizing qa.md's budget recipe for three concurrent chats is a QA-recipe improvement, not a step the story could not do

## finding-163: ctl shows no sub-agent session: a child's effective choice (main's high, not the offered medium), its times and spend are visible only in host ps

- Rung: rm4
- Walk: walk-61
- Backend: claude-forgejo
- Commit: 95d170a5aaa3e073c84bcd06b07ae8c24d04c381
- Area: ctl task show / task turn / chat show
- Severity: low
- Sightings: (none)

### Reproduction

Fresh box per rung M4 steps 1-4 (Claude Code, claude-sonnet-5-5, owned Forgejo, forge setup --playground). temper ctl DIR chat start --project 1 - --model claude-sonnet-5-5 --effort high --sub-agent claude-sonnet-5-5:medium --key K < request (the brief's words). temper ctl DIR task show 1 --until replied; task show 1; task turn 1 1.1 --json; chat show 1. Meanwhile sample ps for claude -p on the host.

### Expected

Each sub-agent's session read through ctl: its model and effort as run, its start and end, its turns and spend.

### Seen

task show prints the offered choices (main model: claude-sonnet-5-5/high, sub-agent model: claude-sonnet-5-5/medium) and each sub_agent call a 'Child usage: used 2 turns and 11 seconds; stopped for budget: no' line; no child model/effort, times or spend. The parent's calls named no llm, so the children ran --effort high (host ps), which ctl does not say. Backlog per the user's narrowing of finding-145.

### Triage reason

rm4 passed without it; the user narrowed finding-145 (same ask, not among the open findings, so no merge) to backlog, and the children running at main's effort when the parent's sub_agent call names no llm is what `help chat start` says ("omitted, it uses main's model and effort"); ctl naming each child's choice, times and spend waits in temper's backlog

## finding-165: task list --project 1 --ended prints no rows while tasks 1 and 2 show Ended (off the happy path, backlog)

- Rung: rm4
- Walk: walk-61
- Backend: claude-forgejo
- Commit: 95d170a5aaa3e073c84bcd06b07ae8c24d04c381
- Area: ctl task list
- Severity: low
- Sightings: (none)

### Reproduction

Fresh box per rung M4 steps 1-5 (two chats, tasks 1 and 2). temper ctl DIR chat close 1 --reason 'walk done' --key c1; chat close 2 --reason 'walk done' --key c2; task show 1 (Ended); task list --project 1 --ended; again 40 s later; task list --project 1 --ended --json.

### Expected

The ended page lists task 1 and task 2, a line each.

### Seen

'no more' with no rows; --json has ended.rows [] at 13:07:29Z and 13:08:08Z, while task show 1 and 2 print 'task N · chat · Ended'.

### Triage reason

the rung's story never lists ended tasks (it reads the chat's task by `task show`, which said Ended); a wrong `task list --ended` page is off the happy path and waits in temper's backlog
