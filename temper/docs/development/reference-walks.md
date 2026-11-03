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
