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
- Sightings: (none)

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
- Sightings: (none)

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
