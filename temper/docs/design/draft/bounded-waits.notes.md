# bounded-waits: notes

The reasons behind [bounded-waits.md](bounded-waits.md), and the inventory it rests on.

## The inventory, at main 362c345f74

Taken on 2026-11-24 by a read-only survey of jig and temper, and read again where the draft relies on it.

- **What a task waits on is computed, one value per task,** from a party's entry in its subtree, its run, its own question, its dependencies, its delegates or news (`jig/docs/design/domain/engine/views.md`, 3). It names a kind, sometimes a number, and never a *since* or a bound. `phase_since`, `run_since`, `last_activity` and a task's deadline are separate facts.
- **Bounds that exist:** a resource queue's `tasks.hold_wait`, 120 s, then `HoldsWaited`; an escalation's and a proposal's `escalation_stall`, 3,600 s, then passed up, with no alarm at the last holder; a person task's one stall notice after an hour; a change's 30-minute forge stall and its CI clock; a landing's prepare-job bound and gate timeout; a worker's 360 s fleet grace and 20-minute suspension.
- **Waits with no bound:** dependencies and delegates (but for the task's deadline), ordinary questions, a parked task's news, task and topic subscriptions, host and slot places, accounts with no known reset, forge disconnection and restart freshness, outbox room, read admission for makings and landings, sign-off, queue and train order, the operator's landing hold, the closer's unknown ancestry.
- **Reads again with no bound:** a landing's `prefixes`, `run`, `receipt`, `ancestry` and `read` waits read at the forge's poll (`../domain/landing-waits.md`, 3). That is how landing 15091 read one comparison more than 400 times.
- **Silent paths:** a change's relay keeps a held answer as `Stands::Held` and tells nobody; the closer's relay drops an unmatched answer; a party's plain words to a change are taken and dropped (`../domain/forge/change/questions.md`, 1).
- **No project-wide view:** `task list` gives one wait per row, the inbox one party's entries, Team a count, and landing and forge reads their own scopes. No read lists every live wait with its kind, since and bound.
- **The watchers:** agents' `goal-watch` and the bridge parse `task tree` and inbox text; temper/dev's `health.py` and `tick.sh` join several reads and filter hold words (among them every line mentioning 1,800 seconds); `stuck.py`, `devwatch.py` and the landing-stall watches diff text and JSON between passes.

## Why these choices

- **A bound counted from *since*, which no read moves.** Reading a fact again is right for a fact that may change, and wrong as the only end of a wait. Keeping both lets the landing's polls stay as they are while ending the wait that never advances.
- **Most bounds hold rather than retry.** A wait past a generous bound has almost always stopped for a reason a person or the requester must see. A hold names it, and escalation routes it to the goal's requester first since settings revision 79 on the reference deployment, where the member role decides escalations.
- **Stuck is found when a task parks, not by a timer.** The edges are in the core already; walking them at the decision that parks costs nothing between changes, and finds a circle at once instead of at a bound.
- **A party's words to a change with no producer are refused, not stored.** The amendment already carries a requester's direction durably (`../domain/forge/change/questions.md`, 1, *Its requester's words and amendments*), so a second channel would be a second place to look. Refusing with words is the visible half that was missing.
- **The figures are the user's.** Where a bound exists it keeps its figure. The new kinds' stock figures are listed as open in the status file.
- **Clocks that exist stay.** CI's checking clock counts running, waiting for a runner and unknown apart, for good reasons (`../domain/forge/change/stalls.md`, 2). A single wall clock from a step's start would hold healthy queued CI. So a wait with its own clock keeps it, and only waits with none take a figure.
- **A wait behind another is bounded by it.** A follower in a landing's queue, or a waiter behind the outbox's entries, would otherwise be held while a healthy holder works. Naming the holder and bounding the holder's own wait puts the hold where the stall is. The one exception is P124's leak, where the holder has ended and can't be bounded.
- **The outbox stays a wait first** (`archive/outbox-room-wait.md`): a full outbox drains by itself within minutes, so a write still waits for room. Only a wait behind an ended owner's entries, which never drains, holds.
- **Stuck spends tries at each park.** The P43 scenario's contract is a retry at each unanswered park and then a hold with the question's words, and a stuck run may still answer. A single wake would leave the next park idle again, as `question_woken` does.
- **Prerequisites aren't wakes.** A dependency or a resource hold must be met, whatever else can wake the task, so stuck detection reads only wakes; prerequisites have their deadline and their queue's bound, and their own circle check.
- **A held answer's notice has its room reserved.** A relay short of message room can't send a notice through that same room, so the notice is an entry in the sender's inbox, reserved when the question is asked.
- **Old records start their clocks at the upgrade.** The previous build's records carry none of the new facts. Reading an absent *since* as the first start of the new build means no live work passes a bound because of an upgrade, at the price of one bound's delay for waits already stuck before it.
- **The watchers retire after landing, outside the goals.** They live in the team's directories, not in this repository, so no goal can remove them, and they should outlive the change by a little, to confirm the views say what they did.
