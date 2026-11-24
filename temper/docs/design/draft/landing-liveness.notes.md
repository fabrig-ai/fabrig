# landing-liveness: notes

The reasons behind [landing-liveness.md](landing-liveness.md), and what it rests on.

## The inventory, at main 05297db1a6

Taken on 2026-11-24 by a read-only survey of the landing train against the review of 2026-11-20 (its group on the landing train), and read again where the draft relies on it.

- **Membership is kept twice.** The landing row holds its members and one ask slot per member; each change stores its landing's number and one handed-over ask. Forming and ask lookup read the change's copy. A member leaving, a landing finishing, a cancel and a restart each update their own share, and the start runs two repairs: one drops links to missing or ended landings, the other erases unmade landings and their links.
- **Forming** takes changes stored ready under the turn, with no link and no handed-over ask, and also reads jig's task phases and due flag. A same-train change still gating can defer forming once per head, through `forming_holds`, which lives only in memory. A refusal from jig is logged and retried from memory; a full `landings` table is dropped without a name.
- **No check of a landing as a whole** runs in production. A member's cancel wakes its landing; a person's hold of a member is seen only when the landing steps. An absent prepare reads `Absent` when the landing next runs, and two identical asks of it hold. S7's test releases the landing by hand to make it run.
- **Goal 15436** (80391b3461, 2026-11-24) pages a kept prefix's history and bounds a fresh read: past the stall bound the landing holds with the exact read and its last answer.
- **Endings:** landed and cancelled landings have their own paths; a cancelled one withdraws unsent entries and settles held ones. A `Failed` task ending goes through generic settlement only.
- **P124's leak is fixed:** dispatch and rerun writes give back their outbox place at settlement (900dd738d8). **P90** holds with the close's account (5646e141a2). **P88** is open: local release failures carry no account.

## Why these choices

- **Derive the link, don't sync it.** Two copies of membership can only be kept in step by every transition remembering both. A derived index can't disagree with the row it is built from, which is what retires the sweeps.
- **An ask is a queue, and only a superseded command expires.** A landing hands a member more than one thing over its life (a prepare, then the word it landed), and a change whose head moved can't take a prepare meant for an older one. A queue keeps them in order. Dropping only what a newer head has made stale, once readiness is checked again, keeps such an ask from holding the change out of every train, without ever losing `Landed`, the authoritative result.
- **A cohort instead of jig's due flag.** "Made ready by one event" is a fact the releasing decision knows when it commits. Storing it there replaces guessing from scheduling state, which is what changed four times in a day.
- **End if nothing is kept, hold if something is.** A landing with nothing landed and nothing in flight loses nothing by ending, and its members go back to the queue. One with a member landed, or a write it can't take back, needs a person or its requester to see it.
- **Wake the landing from the member.** The landing is the one that must act when a member is held, so the hold has to reach it, as a member made ready already does.
- **What may have landed is settled first.** A landing can end with a merge that reached the forge while its member isn't yet recorded landed. Ending or holding it before reading that back would lose or repeat a landing, so uncertain merges are read back, as today, before the check's verdict.
- **The policy's departures come before the check.** A held member is an ordinary departure, which the policy handles with its unit and dependents. Checking first would cancel a landing that only needed to let a member go.
- **A cohort is left at a candidate's first decision.** Waiting for a candidate to become ready could last as long as a repair or a sign-off. Waiting only for each candidate's first step after the release keeps the cohort's purpose (changes released together land together when they're ready together) and bounds it by steps that are bounded anyway.
- **Endings share settling, not everything.** A landed landing still follows its run for a receipt and frees its train at the merge; a failed one may hand branch holds to another root as generic cleanup allows. One table of steps keeps the differences visible instead of flattening them.
- **The old asks are converted, not dropped.** The previous build keeps asks in the landing row's slots as well as on the change, and a legacy single landing can owe a `Landed` derived from its prefix. A written conversion with an order of precedence keeps every one of them once.
- **The second draft first.** The landing's waits and the outbox's room wait get their records and bounds in its goal for temper's procedures; this draft's goals start after that one has landed, so they don't both rewrite the same waits.
