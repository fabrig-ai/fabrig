# Live delegate stall trace

The cause holds: a live producer can stall, and its later change result leaves
it held at producing with no known head. A dependency wait can explain time
before the producer's stall clock starts; the reported approximately 57 minutes
cannot be established from this world. The trace's change task is held and its
escalation waits for the goal; the still-running goal receives no new assignment
or delivered inbox message on this path.

## Reproduction before the fix

`forge_routes::live_delegate::a_live_producer_past_the_stall_delivers_its_change`
in `temper-engine-domain-world::suite`, on the small profile (30-second stall),
fails with `[true, true]` rather than `[false, false]`. It uses `goal_world`,
`make`, `next_producer`, and `finish_producing`; the latter delivers a change
form after pushing a branch. A branch hint offers the waiting change a step.
Times below are the world's wall seconds, observed after durable writes:

| Case | Before make | Producer requested (`since`) | Hold observed | Hold minus request |
| --- | ---: | ---: | ---: | ---: |
| No `after` | 9.1 | 11.0 | 42.3 | 31.3 |
| `after: [dependency]`, dependency pushed after a 20-second wait | 11.4 | 36.1 | 67.4 | 31.3 |

Both held rows are `Held { was: Producing { requested: true }, why: Stalled }`,
with `change.last_head = None`. Their jig phase is
`Held { was: Active(Idle), why: Stalled, source: None }`. Escalation is
`Waiting { revision: 1, holder: Task(1), entry: 2, since: 42.3 }` without
dependency, and entry 3, since 67.4, with dependency. The goal gets zero new
assignments and zero delivered inbox messages at the hold and after the result.
Its existing run is still active; this does not prove that its next assignment
would omit an escalation.

After the producer's pushed branch and `TaskResult::Change`, another 100 ticks
leave the same held state, unknown head, jig phase and escalation. No release
was sent. This reproduces the result not being taken, independently of the
neighbour task's possible failure to propagate a hold.

## What explains the clock

The dependency case stays `Producing { requested: false }` while waiting for
the dependency's first push. `request_producer` sets `since` when it requests
the producer, so this earlier wait precedes the stall (forge 8.11). Both
cases hold on the first completed read after the same 30-second deadline;
the extra 1.3 seconds here are asynchronous world reads and durable writes.
The dependency adds time before the producer request, not to that deadline.
Whether that accounts for the real request's approximately 27 extra minutes is
unknown: this story has no timestamps from that deployment.

An exploratory run without the branch hint observed no hold through wall 81.0
(request at 11.0); no completed title-wait step armed the stall. Once a hint
offers the step, its deadline is observed as above. The initial producer request
alone does not demonstrate a scheduled stall wake. A delayed or absent offered
step can therefore delay observation too; the real deployment's step schedule
is unknown. The test supplies the hint explicitly instead of assuming polling
that the initial delegate decision does not promise.

## With the fix

The same world story passes: both changes remain unheld across the producer's
stall and take their pushed head and title, opening a pull without a release.
Its teardown signs the fake person in again after the small profile's 60-second
sign-in expires, then cancels the goal through the existing early-start helper
and runs the restart oracle.

A live producer can no longer reach this stalled row through the world story.
The forge step tests seed the held row with `Event::Change` to cover a result
ending a producer, repair, resolution or merge-job stall, a title followed by a
result opening a pull, a no-change ending with its words, and a questions hold
keeping a result until release. They also check a delegate's result resets the
stall to its end while a gate's leaves the clock alone.

The reverse title/result order is not offered by the root: `keep_produced_title`
(`temper-engine-domain/src/route.rs`) emits the title from the durable ended
record before the result wakes its requester; `start_change` reads that result
from the procedure's inbox. The forge boundary also documents `ProducedTitle`
as preceding the result's wake. The tests keep that order rather than inventing
a later title event after the delegate's terminal cleared its identity.
