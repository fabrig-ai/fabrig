# Temper's own writes

The record of every write temper makes on a branch or a pull request, and how a read of the forge is judged against it, so that temper's own writes are never drift and a view behind them is a wait, not a hold. Written from `../../draft/whose-red.md`, 7 and 8's branches bullet. The forge connector owns it; the forge client observes (skein's `forge/client.md`, 4) and judges nothing of who moved a branch.

## 1. The record

- **One record per branch** of a repository temper writes: a change's, a goal's own, a landing branch, a landing's prefix, a saved work's branch. A pull request's entries are in the record of its head branch, each naming the pull by its number, since a pull never changes its head branch; a merge that moves its base's head (a merge commit, a squash, a rebase or a fast-forward) writes an entry in its base branch's record too; a recorded merge (skein's `MergeHow::Recorded { commit }`, Forgejo's `manually-merged`), which marks the pull merged at a commit its base already holds, moves no branch and writes the merge entry alone. So every branch and every pull temper writes has its entries, and one figure bounds them (6).
- **Its head** is the branch's head as temper's settled writes leave it, or absent: the commit the forge client is told to expect (`Pushed`, skein's `forge/client.md`, 4) and the one drift is judged from.
- **An entry** is one write: its **operation**, with its **before** and **after** states; its **sender**; its **order**, a number from 1 rising on its record, given at admission; and its **state**, one of *admitted*, *sent*, *confirmed*, *refused* and *uncertain*. Admitted, sent and uncertain are **unsettled**; confirmed and refused are settled.

| Operation | Sent by | Before → after |
|---|---|---|
| push | a worker, for an attempt: a delivery, a design delivery, a save, a slot writer's or a job's push (a merge job's, a close's), a landing's prefix, a base made at a checkout's preparation | the head it was leased on, or absent → its commit |
| create | the forge client, an outbox entry (a goal's branch, a branch made again on release) | absent → its commit |
| delete | the forge client | the head temper knew → absent |
| update | the forge client, on the pull's head branch | the pull's head → a head holding the base it asked for, filled in at its answer (an intent) |
| merged into | the forge client, on the base branch, for a merge that moves it (never a recorded merge) | the base's tip → the merge's commit, filled in at its answer (an intent); a fast-forward's after is its candidate |
| merge | the forge client, for the pull | open at its head → merged |
| open | the forge client | no pull for its branches → open on its base at its head; its number filled in at its answer |
| close, reopen | the forge client | open → closed; closed → open |
| retarget | the forge client | on one base → on another (temper sends none yet: `change/drift.md`, 1) |
| writer | the previous build's live or lost writer (7) | the head it knew → a head it moved forward to, found at readback (an intent) |
| compare | the previous build's head beside (7) | its waiting head, ordered against the head by one comparison |

- **What isn't an entry:** a write that moves no branch's head and changes no pull's state, head or base (a comment, a pull's title or body, a status, a label) is no entry, since no read is judged by it: the outbox carries it as before (`effects.md`, 1).
- **Who sent it:** a task and its attempt, for a worker's push and for a converted writer (7), whose hold names its attempt; a task and its outbox entry, for the forge client's; the previous build, for a converted compare. The sender is what drift's words name (5) and what a readback waits on (3).
- **Why the connector owns it, not the client:** the record names temper's senders, takes a worker's admission, outlives a resource's watch (an uncertain entry is never dropped) and decides drift, which is policy; the client keeps a bounded view and makes writes, and knows none of these. **Why per branch, not per pull too:** a pull's state and base are written only by temper's pull writes, and its head only by its head branch's, so one record holds both with one room.

## 2. Admission

Every write is admitted before it can reach the forge.

- **The forge client's writes** are admitted in the decision that commits their outbox entry (`effects.md`, 1): each write on a branch or a pull adds its entries, *admitted*. It is *sent* in the decision that saves its attempt (`Flight`), and settled by its outcome (3). An entry withdrawn before any attempt is erased with it.
- **A worker's push** waits for the engine's admission (`../worker/checkouts-push.md`, 1). The worker reads the destination's tip and asks, with the attempt's task and attempt beside its ask: the repository, the branch, the head it expects (absent for a creation) and the commit. The engine admits it when the attempt is its task's live one (jig fences the ask: jig's `domain/hosts/hub.md`, 10) and the branch is one its claim's workspace names for it to write (its push branch, its save branch, its prefixes); it saves the entry, *admitted*, before (expected) → after (commit), and answers `admitted` only once that decision is committed. It refuses with words, at once, a branch the attempt may not write and a record with no room (6). The push is then leased on that expected head, so it lands from that head or not at all.
- **What admission doesn't judge:** it records the head the worker says it read. Whether that head is temper's is the judgement of 4, at the read that shows the push.
- **Only workers that ask.** The engine marks every repository its assignments and jobs give to write as asking admission (`../../protocol/channel.md`, 4). A worker of the previous layout can't keep that mark, so the work waits for a worker that can, as a close's typed refusal does (`../../draft/whose-red.md`, 4.1); a previous engine marks none, and its workers push as before. The previous build's live attempts are converted (7).
- **Why the worker waits:** it pushes to git itself, so a record made from its answer would come after the forge already shows the commit, and a read meanwhile would find a change no entry explains. One exchange per push is the cost.

## 3. Settlement

- **By the forge client's outcome:** made, *confirmed*, its intent's after filled in from the answer (an update's head, a merge's commit, an open's number); a typed refusal, *refused*; a lost answer, raced or held for a person, *uncertain*. An uncertain write the client finds again is confirmed by its find (skein's `forge/client-effects.md`, 1.5).
- **A worker's push by readback,** since the engine never hears when it reaches the forge: confirmed by a read that shows its commit (4), or by a later admission whose expected head is its commit (the worker read it there); once its attempt has answered, a fresh read of the branch settles each of its entries still unsettled: the head holding its commit, *confirmed*; not holding it, *refused*, since the attempt's push ended before its answer. A lost attempt's entries are *uncertain*, since its push may still land: confirmed once a read's head holds the commit, refused once the head has moved off its leased head without it, which no lease on that head can then take.
- **Intents by readback:** an update's after is a head holding its base, a merge's the pull's merge commit, the previous build's writer's a head moved forward from the one it knew, each read as its readback names (4, *Readback*).
- **Never dropped.** An uncertain entry stays until a read settles it, through restarts (7), whatever its sender's end. A refused entry is erased in the decision that settles it, its sender told as before; a confirmed one stays until its room is needed (6).
- **The reads it needs** are fresh reads (`keeping-up.md`, 3): the branch's head, a `Commits` ancestry range read as a containment, a pull. A failed one is asked again as any fresh read is; the entry stays as it was.

## 4. Judging a read

A read of a branch, a fresh one or the forge client's keep, is judged against the record, in this order:

1. **Another actor:** a hint names the read's commit pushed by an identity that isn't temper's writer (skein's `forge/client.md`, 4): drift.
2. **Current:** the read is the head.
3. **Explained:** the read is the after of an unsettled entry whose before is the head, or the after of another such entry, in order: those entries are confirmed, and the read is the head. An entry whose before the record can't explain explains nothing until that before is (*Readback*), or is drift.
4. **Ordered:** the read is the after of a settled entry that is not the head. Where the record shows it older (an entry since built on it, its after another's before), it is behind. Where nothing shows which is newer, one comparison orders them: the read holding the head is the head; otherwise it is behind. So no record moves the head back to a head temper wrote over, in whatever order their answers come (`branches.md`, 3).
5. **Behind:** the read is a before, or an older after, of the record's history: the forge's view is behind temper's writes, or someone reset the branch to an older commit of temper's, which the forge can't tell apart (5).
6. **Readback:** the read is a commit the record doesn't hold while an unsettled intent could have made it: its readback first (the update's head holding its base and its before; the pull merged at that commit; the previous writer's head holding the one it knew), then the read is judged again.
7. **Drift:** anything else: another actor's push, a move backwards to a commit temper never wrote, a branch gone with no delete of temper's in the record. A drift names the head expected, the one observed and the record's nearest entry.

A read of a pull is judged by the same record: closed unmerged, explained only by temper's close, else drift; on another base than the record's, explained only by temper's retarget, else drift; merged with no merge of temper's, merged by someone else, as `change/drift.md`, 1 takes it. Its head is judged as a read of its head branch, its base's commit as a read of its base branch, so a pull whose view lags its branches is behind.

## 5. Who waits, and who holds

- **A caller that needs the fact** (a change's step, a landing's round, a release reading its branch) decides nothing on a read that is behind: it reads again, as its step reads when it next steps, until the read catches up, within its own wait's bound (`../../draft/bounded-waits.md`, 4): a change's forge stall (`change/stalls.md`), a landing's read wait (`../landing-waits.md`). At the bound it holds, saying it can't tell: `its branch reads F, an older head of temper's (expected E): the read may be stale or another's write`.
- **The keep alone** (the forge client's observation, which no caller asked for): a drift holds as today, a goal's kept branch with no change row among them (`change/drift.md`, 1); an explained read settles its entries; a readback is read; a read behind waits for nobody, since no caller needs it, and the next read compares again.
- **Drift's words** name the expected state, the observed one and the nearest entry: `its branch moved: expected E, found F; temper's last write on it: push of C by task T attempt A, uncertain`.
- **Explained is temper's.** A read the record explains is a head of temper's, never drift, whoever of temper's wrote it: a change takes it as its new head, checked again, as its own writes are (`change/drift.md`, 1). While a writer holds a branch's slot, a change still decides nothing on its head (`change/drift.md`, 1).

## 6. Its room

- **32 entries per branch,** the stored list's bound. A write that finds its branch's record full evicts the oldest confirmed entry that isn't the head's; with none, it waits before it is sent: an outbox write in the outbox's line, as for a full outbox (`room.md`, 3), its `room` wait (`../../draft/bounded-waits.md`, 3.2) naming the branch's record, its figure and its unsettled entries' senders, in the shape the outbox's `room` wait has (`room.md`, 2.1): its procedure object's things are the record, carrying its figure, then its oldest unsettled entries with their senders, as many as fit, and the branch's name and the senders are in its hold's words, within the effect account's bound (512 bytes): the branch's name, then its unsettled entries' senders oldest first, as many whole as fit, then the count of those left out (`and N more`), every sender staying in the record itself; the waiting writer's own row names its branch until then; a worker's admission is refused at once, with those words, as an agent's call finding no room is (`room.md`, 5), and its push isn't made.
- **Only settled entries go:** an admitted, sent or uncertain entry is never evicted, so a write that may still land can never turn into drift at the next read.
- **A record lives** as long as its branch's head row (`BranchHead`) and goes with it, unless an entry in it is unsettled, which keeps it.

## 7. Stored, restarted and converted

- **Stored** in temper's store as one row per branch: its head, its next order and its entries (`../../protocol/store.md`, 2), so a start finds them as they were. A start asks each uncertain entry's readback once the forge's fresh reads are in (`catching-up.md`, 4), and nothing on its branch is decided before it is read.
- **The previous build's branches** have no record. At the first start of this build, before the forge's links are made (`../root-restart.md`, 1.1), a written conversion gives each branch a record: its head the one the previous build expected (the forge client's pushed head for it, else its owned `BranchHead`); a head beside with a waiting head (`branches.md`, 3) a *compare* entry, uncertain, compared before any drift is judged on it; each live or lost writer of its hold, slot or design writer, a *writer* entry sent by that writer's task and attempt: a live one *sent*, settled as its attempt's pushes are when it answers or is lost (3); a lost one (a slot's lost writer, a design writer the previous build marked rereading) *uncertain*, read back at the start. A converted entry names its attempt, not the previous build, since a later start finds the hold's writers changed and only the sender says whose entry is whose; the previous layout's rereading mark is read by the conversion for this alone. A branch whose converted entries would pass the record's bound (6), which the previous hold's lists allowed, isn't converted by dropping one: the start refuses with words naming the branch and the count, and the store is left as the previous build wrote it. Nothing held is released, and nothing new is read as drift. The conversion goes once dev.fabrig.ai's store has run it (`../../draft/whose-red.execution.md`, 3).

## 8. Open questions

None.
