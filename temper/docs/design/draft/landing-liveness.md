# landing-liveness: a landing train that always moves

Draft, 2026-11-24, on the user's word: the third of the five drafts the user accepted on 2026-11-20, from the review of goal execution on the reference deployment. It is failure-worlds' *landing train* (`failure-worlds.md`, 7), and it builds on [bounded-waits.md](bounded-waits.md): every wait here is a wait record with its bound. Its reasons are in [landing-liveness.notes.md](landing-liveness.notes.md), its execution in [landing-liveness.execution.md](landing-liveness.execution.md).

## 1. In one page

A landing train moves only while facts kept in several places agree: a landing's members, each change's link to its landing and its handed-over ask, the branch holds and outbox entries, jig's landing task and its prepare child, and memories the engine keeps only while it runs. Each transition (a member leaving, a landing ending, a cancel, a refusal from jig, a restart) updates its own share of them. Nothing checks the whole, so each disagreement has surfaced as a train that stopped, patched where it was seen: the forming rule was rewritten four times in one day, and two sweeps at start repair stale links.

This draft gives the train one owner for each fact and one check that it can move:

1. **One owner for a landing's members** (3). The landing holds its members; a change's landing is derived from the live landings, never stored on the change. A handed-over ask is a queue on the change, which its step takes under a bounded wait. No link can go stale, and the start's sweeps retire.
2. **Forming from stored state alone** (4). A train forms from changes that are ready, in no live landing and with no ask outstanding. Changes made ready by one event are a cohort the releasing decision stores. Every refusal to form is named on the train.
3. **Liveness at every step and at start** (5). A live landing has a live jig task, and either work in flight or a named wait with a bound, and live, unheld members. A member's hold or cancel wakes its landing. A landing that breaks the check ends, or holds with words.
4. **Every ending settles what it holds** (6): landed, cancelled or failed alike.
5. **Every release is idempotent and accounted** (7): its operation, its resource and the forge's answer, on every path.

What it leaves to the other drafts is in section 10.

## 2. Where it starts

| What happened | Why it went on | Section |
|---|---|---|
| Changes stayed ready for hours naming a landing that had ended (P102), watched for by a script that flags them after an hour | A change stores its landing's number, and an ending that missed it left the link standing | 3 |
| A ready change whose handed-over prepare ask no longer matched its head waited with the ask set, out of every train | The single handed-over ask has no bounded path to be taken or dropped | 3.2 |
| The forming rule was rewritten four times on 11-19 (7b614ffc09, fc3a630aea, 74447ea993, a21addd38f), the last two by hand | Forming reads jig's due flag and an unstored settling memory, so "made ready by one event" is a guess about scheduling | 4 |
| A train with room refused by a full `landings` table showed nothing | The forge's refusal is dropped at forming | 4.3 |
| A live landing outlived its prepare child, with a member held by a person, until someone cancelled it (S7) | A member's hold doesn't wake its landing, and no check says a live landing must be able to move | 5 |
| Main took no landing for four hours on 11-24 (landing 15091) | A round re-read one comparison without advancing. Goal 15436 (80391b3461) pages that history and holds a fresh read past its bound, read admission included; nothing yet checks the landing as a whole | 5 |
| A landing that ends failed skips what a cancelled one does: members left linked, unsent entries not withdrawn | Landed and cancelled endings have their own paths; a failed one goes through generic settlement alone | 6 |
| A release failure read "forge could not release this task's resources" (P88); its cause was never found, and its scenario repeats a refused round deletion silently | Local release failures carry no account | 7 |
| The team keeps a landing-stall watch and a stale-landing watch | Nothing in temper names a stopped train | 9 |

Fixed since the review and kept as they are: a cancelled landing's outbox entries are settled (P124's leak, 900dd738d8), and a refused close of an open pull holds with the forge's answer (P90, 5646e141a2).

## 3. One owner for a landing's members

### 3.1 The landing holds its members

The landing row is the one record of who is in it: each member, in order, with its state (in, left, landed). A change's landing is derived: the core keeps an index from change to live landing, built from the landing rows when it starts and changed in the same decision that changes a row. A change stores no landing number. So a landing that ends takes its members with it, and there is no link to go stale. Before indexing at start, the rows are checked: a change that two live landings both claim keeps the older landing, and the newer holds with words naming the change.

Branch holds stay owned by the landing's task, as today, and go when it closes (6).

### 3.2 A handed-over ask is a queue the change takes

What a landing hands to a member, a prepare, a judgement, a resolve, a repair or the word that it landed, is an entry in a queue on the change, each naming the landing, the round and its kind, with an identity of its own:

- **At most one ask of each kind per landing:** a newer ask of a kind replaces the older one, so the queue is bounded by its kinds.
- **`Landed` comes first and never expires:** it is the authoritative result, taken before any other ask and before the change's ordinary step, as today.
- **Taking an ask commits with its outcome:** the change's step names the ask it took, by identity, in the decision that records what it did, so a restart can't take it twice or lose it.
- **Only a superseded command expires:** a prepare or judgement for a head that has since moved is dropped, with words to the change's record ("prepare for 6c5fd3a6e048 dropped: the head is now 1b949e1dda"), once the change's readiness is checked again. A held change stays held.

An ask is taken at the change's next step, which is due at once; a change that doesn't step is itself waiting, with its own bound (`bounded-waits.md`, 4), so an ask needs no clock of its own.

## 4. Forming from stored state alone

### 4.1 Who may join

A change may join a train when, in stored state, it is ready under the turn, in no live landing (3.1), with no ask in its queue (3.2) and its dependencies met, and when the guards that apply today still pass: its task isn't held or closing and has no live delegate, its repository's facts are fresh, the operator's landing hold doesn't stand, and it comes with its whole dependency unit (`../domain/landing-forming.md`, 2). Forming reads nothing else: not jig's due flag, and no memory kept only while the engine runs. It is judged when the root's work drains, as today, so a change mid-step is judged on what it has stored.

### 4.2 A cohort waits for its own, once

Changes made ready by one decision, the landing of a dependency that releases several, say, are a **cohort**: the decision that releases them stores the cohort's number on each candidate. A train with a member of a cohort whose other candidates haven't taken their first step since waits for them: each candidate leaves the cohort at its first decision after the release, ready or not, and a candidate that regates, starts a repair, gains a dependency or waits for a sign-off has left it. When none is left settling, the train forms with the ready ones, in queue order and within its batch, as today. The cohort is the train's own, its wait is a record whose wake is each candidate's next decision, and it is stored, so a restart keeps it, and the settling memory (`forming_holds`) retires. It needs no figure of its own: it lasts no longer than its candidates' first steps, each of which is bounded where it waits (`bounded-waits.md`, 4.1).

### 4.3 Every refusal is named

- **A refusal that won't pass**, a unit too large or crossing repositories, stays where it is today: a hold on the unit's first change with its account. The train passes over the unit, so later work forms.
- **A refusal that may pass**, jig refusing to make the landing task, or the forge's `landings` table full, is a wait the train owns before any landing exists: a record on the train naming the refusal (jig's words, or the table's figure and the landings holding it, ended ones still following a receipt or settling a write among them), woken by what can change it (a slot freed, a setting or a jig decision changed), and cleared when the train forms or its candidates are gone. It is stored with its retry time, so a restart neither forgets it nor repeats it at once (`forge_refused` retires), and `landing list` and the waits view show it (`bounded-waits.md`, 9).

## 5. Liveness at every step and at start

### 5.1 The check

In every decision that steps a landing, the landing first does what its policy does today: members held, cancelled or ended leave, with their units and dependents as the policy says (`../domain/landing.md`, 4.3), and an absent prepare is asked again or held as today. Then, on the state that leaves, the check: a live landing can move when

- its jig task is live;
- each member still in it is live, not held, and in this landing alone by the index (3.1);
- at most one live landing holds its train's turn;
- it has a wake that can come: an effect in flight in its current round, a live prepare child of its current round, a read it is owed, or a wait with a bound or a reason for none (`bounded-waits.md`, 4.1: CI's clocks, an operator's hold, a person's release), or it is due to step now.

The check runs in every decision that steps a landing, when one of its wakes is withdrawn or its armed deadline fires, and, as a structural pass, at start once the landing rows and changes have been read, with anything that needs the forge's facts waiting for them to be fresh (`reconciling-start.md`). It looks the landing up by its rows and the index, never by scanning every task. A landing held by a person is not checked: the hold is the person's.

### 5.2 A landing that can't move

- **First, what it may have landed:** a landing with a merge whose outcome is unknown, a write in flight or answered uncertain, settles it before anything else. It keeps its turn and its resources, reads back the tip and the pull, and records each landed member once, as uncertain merges are read back today (`../domain/landing-turn.md`, 1).
- **Nothing landed:** a landing that still fails the check, with no member landed and no write it can't take back, ends cancelled, through its ending (6), naming the reason on its record and to its members.
- **Something landed:** it holds, with words naming what failed the check ("member change 15057 is held by party 1"; "its prepare child 15120 has ended with no result"). The hold escalates as any hold does.
- **No live task at all:** a landing row whose jig task is gone has no task to hold, so the forge runs its ending at start, reading back any merge first, and names it in the start's log.

### 5.3 A member's change wakes its landing

A member's hold, cancel or end wakes its landing in the same decision, through the index (3.1), as a change made ready does today. The landing's next step lets the member leave (5.1), so S7's landing moves without a person releasing it.

## 6. Every ending settles what it holds

The three endings share the settling of what a landing holds, and keep apart what differs between them:

| Step | Landed | Cancelled | Failed |
|---|---|---|---|
| Its members leave, each with the ask the ending owes it (3.2) | at the end, after `Landed` asks for those that landed | at the cancel | at the end |
| Its train's turn is freed | when the merge is recorded | at the cancel | at the end |
| Unsent outbox entries withdrawn; writes in flight awaited and settled | yes | yes | yes, as cancelled today (today it skips this) |
| A run followed for a receipt | followed, keeping no outbox entry (P124's fix) | no longer followed | no longer followed |
| Branch holds | released when its task closes | released when its task closes | released when its task closes, or handed to another root where generic cleanup allows today |

## 7. Every release is idempotent and accounted

- **Accounted:** a release that fails carries an account on every path, the round's branch deletions included: the operation, the resource, and the forge's answer, or, for a failure of temper's own (a release state that doesn't match, a writer still holding, a refused write), its own named cause. The hold's words are the account, never "could not release this task's resources" alone.
- **Idempotent:** a release finds the resource and its owner first. One already released, a branch already deleted, a pull already closed or merged, a hold already dropped, is a success when the owner it finds is the one releasing, and a refusal with words when it isn't. So a retried release after a restart or a refusal does no harm.

This is cleanup's accounting alone. How a change's reds and holds release is the fourth draft's (10).

## 8. Across restarts and builds

- **The index** (3.1) is built from the landing rows at every start, after the rows are checked for a change two landings claim.
- **The previous build's asks** live in two places: the landing row's slot per member, and the change's single handed-over ask; and a legacy single landing can owe a `Landed` ask derived from its recorded prefix. A written conversion runs at the first start of the new build: it puts every one of them in the change's queue, `Landed` first, one per kind and landing, the row's ask kept over the change's when both name the same kind, and it reads the change's stored landing number only to log one that disagrees with the rows. Its tests start from stores the previous build wrote.
- **The start's repairs stay until they're not needed:** the stale-link sweep goes with the conversion; the unmade-landing pass and the drop of abandoned branch holds stay until the new ending (5.2, 6) handles their cases. The conversion and the sweep go once the reference deployment's store has run it, by the rule that no code is kept for a layout older than what that deployment runs.
- **Cohorts and refusals** (4.2, 4.3) are optional members with defaults: none.

## 9. The watchers retire

Once this draft's milestone and the bounded-waits milestone's waits view have both reached the reference deployment, the team retires `stale-landing-watch.sh` (stale links can't exist, 3.1) and ops' `landing-stall.py` (a stopped landing fails the check and holds with words, 5, its waits have bounds, and the view lists them). The hand landing path and its scripts are not this draft's (10).

## 10. Out

- **Waits and bounds:** the second draft. Its goal for temper's changes, landings and closers gives the landing's waits their records and bounds, and the outbox's room wait its bound; its last goal lists every wait. This draft uses them, and shows nothing of its own.
- **One way to classify a red, one release rule, and a record of temper's own writes:** the fourth draft.
- **Landing by hand,** and the checks the hand path skips: the review's seventh pattern, not drafted.
- **Capacity:** a later root design.

## 11. Tests

Focused and fuzzy suites only.

- **The check as a property:** the landing policy's fuzzy (`../domain/landing-waits.md`, 3) and the engine forge and root worlds check 5.1 for every live landing at each quiet point, over random landings with conflicts, departures, members held and cancelled by a person, uncertain merges, failed and cancelled endings, cohorts of several changes made ready at once, and restarts mid-landing, in a busy world as well as a quiet one.
- **The catalog's scenarios pass** (`failure-worlds.md`, 7): S7's landing moves without an explicit release; P88's refused release is accounted.
- **No stale link, no lost ask:** after any ending, no change's derived landing is the ended one; a store the previous build wrote, with stale links, asks in both places and a legacy single's prefix, opens with none stale and every ask in its queue once.
- **An ask that can't be taken** is dropped with words, and its change joins the next train.
- **A named refusal:** a train refused by a full `landings` table shows it, and forms once a slot frees.
- Focused tests for each ending's path and each release's account.

## 12. Later

- One landing per train holding several rounds in flight, once the turn's owner is one record.
