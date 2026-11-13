# A landing's search by prefixes

Provisional, 2026-11-13. What a landing does when its gates fail at the
top of a round of several units: it dispatches the same gates at the
unit boundaries below the top, one at a time, until it holds a green
prefix and the red one a unit above it; the unit between them is
repaired and leaves, the green prefix lands in the round's turn, and the
units above the red one go on in the landing's next round. It is the
merge queue's search by prefixes (`../draft/merge-queue.md`, 7.2 to
7.4, its execution's M3), on the landing of `landing.md`, whose words it
uses: its rounds (5.1), keys and receipts (section 6), proof by dispatch
(section 7) and the turn (section 8). It replaces M2's fallback, under
which a red round of several units ended and its units landed one at a
time. The search is the landing's policy (`landing.md`, section 2): the
top dispatches, reads and writes for it as for the round's top.

## 1. In one page

- **A red round of several units searches** (section 2); a round of one
  unit repairs its changes as `landing.md`, 7.3 says.
- **Unit boundaries:** the prefix of each unit's last member in the
  round's stack, the round's base below them all. The base is green;
  the candidate, where the gates failed, is red.
- **Probes** (section 3): the train's blocking test gates dispatched at
  the middle boundary between the highest known green and the lowest
  known red, receipts by key as at the top, one probe at a time, at
  most `ceil(log2(n))` of them for n units.
- **The boundary** (section 4): once a green boundary and a red one are
  a unit apart, that unit is blamed: its changes repair and leave, the
  stack above the green prefix is cut, the green prefix lands in the
  round's turn (section 5) unless it is the base, and the units above
  the blamed one are stacked on the new tip in the next round.
- **Its identity** (section 6): the round's. A round that ends ends its
  search; a late outcome makes only a receipt.
- **The blamed unit's repair** (section 7) is told the red prefix, the
  green one below it and the changes landing in that green prefix.
- **Shown** (section 8): news of each probe, each probed prefix's gate
  statuses, the round's pull request naming the search, and the search
  in `landing show` and the changes view.

## 2. When it starts

- **A real failure at the candidate** (`landing.md`, 7.3): a blocking
  test gate's failed receipt at the round's candidate, in a round whose
  stack holds two units or more. A refused judgement gate makes its
  member leave (`landing.md`, 4.3), a runner's fault is retried
  (`landing.md`, 7.4) and a run past `gate_timeout` holds the landing:
  none of them starts a search.
- **The round's units** are the units with a prefix in the round, in
  the stack's order, numbered here from 1 to n. Boundary k, for k from 1
  to n, is the prefix of unit k's last member: the round's base with
  units 1 to k on it. Boundary 0 is the round's base, green: the train's
  tip, which landed proved. Boundary n is the candidate, red.
- **It starts at once,** in place of the repairs a round of one unit
  makes: no member leaves, and the round goes on. The candidate's
  gates still in flight go on to their receipts and decide nothing
  (section 6).

## 3. Probes

- **The next probe** is at boundary `g + floor((r - g) / 2)`, g the
  highest boundary known green and r the lowest known red, while
  `r - g` is 2 or more. It starts with g at 0 and r at n.
- **A probe** dispatches the train's blocking test gates at the
  boundary's prefix as `landing.md`, 7.1 dispatches them at the
  candidate: each gate keyed at that prefix (`landing.md`, 6.1), with
  the round's base's tree for a gate that reads the base; a key with a
  receipt holds by it; a key in flight is waited for; otherwise the
  gate's workflow is dispatched on the branch in the landing of the
  boundary's member, whose head is its prefix, with `temper_commit` the
  prefix, `temper_key` its key and, for a gate that reads the base,
  `temper_base` the round's base. Each gate's status,
  `temper/gate/<name>`, goes on that prefix (`landing.md`, section 10).
  An advisory gate is the candidate's alone: no probe dispatches it.
- **Its verdict:** red at the first of its blocking gates with a failed
  receipt at that prefix, which makes r the boundary; green once every
  one has a passed receipt there, which makes g the boundary. A run
  still in flight once the verdict is taken goes on to its receipt and
  moves no bound.
- **One probe at a time,** as a train runs one proof at a time
  (`../draft/merge-queue.md`, section 6): the next is dispatched once
  the last has its verdict. A search makes at most `ceil(log2(n))`
  probes beyond the candidate's runs: with `forge.landing_members` at
  most 5 (`../protocol/configuration/limits.md`), three.
- **Bounded as any run:** a runner's fault at a probe is retried within
  `gate_retries` per key, never repaired, and past it holds the landing
  (`landing.md`, 7.4); a probe's run in flight past `gate_timeout`
  holds it (`landing.md`, 7.1). Their holds' words are those of a run at
  the candidate. A release prepares again in a new round
  (`landing.md`, 4.2), which ends the search (section 6).
- **Five units, the third breaking the gate:** g 0 and r 5 probe
  boundary 2, which passes; g 2 and r 5 probe boundary 3, which fails;
  with g 2 and r 3 the search holds its boundary after two probes: unit
  3 is blamed, units 1 and 2 land, and units 4 and 5 go on in the next
  round. **Three units, the first breaking it:** g 0 and r 3 probe
  boundary 1, which fails; unit 1 is blamed with the base green, nothing
  lands, and units 2 and 3 go on in the next round.

## 4. The boundary

- **Held** once `r - g` is 1: boundary g's prefix is the green prefix,
  boundary r's the red one, and unit r is blamed. It is an observed
  boundary, not the lowest red or the longest green prefix: a later unit
  may fix what an earlier one broke. The blame is unit r's in that
  context: the failure appears with it, on top of a prefix that passed.
- **Then, in this order,** each step recorded before the next, so that
  a restart finds where it stood (section 6):
  1. **The blamed unit leaves.** Each of its members leaves
     (`landing.md`, 4.3) with `gate NAME failed in RUN at RED, above
     GREEN`, NAME and RUN the first of the red prefix's blocking gates
     with a failed receipt there and its run's number, RED the red
     prefix and GREEN the green one, the base's commit where g is 0,
     each at the forge's length. Each of its changes repairs (section
     7). Leaving so does not end the round, as another leaving after
     the job stacked it would (`landing.md`, 5.1).
  2. **The stack is cut above the green prefix.** The round's pull
     request is closed and the branches from the blamed unit's first
     member up are deleted, as a round ending unlanded deletes them from
     its first changed member (`landing.md`, 5.1). The members above the
     blamed unit stay in the landing, with no prefix in this round, and
     none is stacked again in it.
  3. **The green prefix lands** in the round's turn (section 5), where
     g is above 0. Where g is 0, the round lands nothing and ends: no
     empty turn (`../draft/merge-queue.md`, section 8).
  4. **The next round** takes the members still in the landing, the
     units above the blamed one, in their order: its base the train's
     tip read afresh, the green prefix where it landed, and its job
     stacks them onto it (`landing.md`, 5.2). They are first in line,
     being the landing's own. With no member left, the landing ends
     (`landing.md`, 4.2), its result naming what landed and the blamed
     members that left.

## 5. The turn at the green prefix

- **The round's proved prefix is the green one:** its blocking test
  gates passed at exactly that prefix, by its probe or by receipts, and
  each of its members' judgement gates holds at the member's own prefix
  (`landing.md`, section 3 and 7.2). The landing waits for a judgement
  still asked there.
- **Its pull request** is the round's, opened anew from the branch of
  boundary g's member, the green prefix its head, titled for the
  members it lands and its body written as `landing.md`, section 10
  says, with the search's line (section 8).
- **The turn is `landing.md`, section 8's at that prefix:** the train's
  tip and the head of each member up to the green prefix read afresh; a
  moved head makes its member leave and ends the round unlanded, a moved
  tip makes the round stale; then the pull request is merged
  fast-forward only at the green prefix, judged by the landing rules for
  its members alone, a rule by path comparing the tip with the green
  prefix; and each of its members lands at its own prefix, recorded
  before the next round, its pull request read or recorded merged as
  there.
- **The round ends at its turn** and the next starts (section 4, step
  4). A round that ends otherwise between the boundary and the turn, by
  a member of the green prefix leaving, a stale base or a hold's
  release, ends unlanded as `landing.md`, 5.1 says: the next round
  stacks every member still in the landing, the units above the blamed
  one among them, from its first changed prefix.

## 6. Its identity

- **A search belongs to its round:** its base, its members and their
  heads, its prefix commits. The round keeps its two bounds, g and r,
  from the candidate's red to its end, so that a restart takes it up
  where it stood: its probes' runs found again as a proof's are
  (`landing.md`, section 15), and the boundary's steps found by what the
  landing recorded and by its effects' purposes (`landing.md`, section
  11), none made twice.
- **It ends with its round:** a member of the stack leaving, the blamed
  unit at the boundary apart; a head moving; a stale base; a hold's
  release. The next round proves its own candidate afresh and searches
  anew if that is red, a receipt found earlier holding for its key
  (`landing.md`, 6.2).
- **Late outcomes make receipts for their exact keys,** and nothing
  else: a run a probe or the candidate dispatched, followed to its end
  after its verdict was taken or its round ended (`landing.md`, 7.1),
  moves no bound and blames no unit. A later round's probe or candidate
  whose key has that receipt holds by it.
- **A key is a tree's, not a prefix's:** two prefixes with one tree
  (a squash that adds nothing), or one prefix kept across rounds, share
  each gate's key. One run answers every prefix with that key, and a
  run in flight is the key's, never one member's.

## 7. The blamed unit's repair

- **Its brief** carries the red as `landing.md`, 7.3's repair carries a
  failure at the candidate, the red prefix in the candidate's place:
  the gate, its run's link, the red prefix and the round's base, and
  each failed job's failed step with the end of its log, from the red
  prefix's failed receipt. Besides, it names the green prefix below it,
  or says that it is the round's base, and the changes landing in that
  green prefix, each its change and pull request, in the stack's order.
- **The ask** that carries them from the landing to each blamed change
  is the landing's repair ask (`landing.md`, 4.3), with the red prefix
  as its candidate, the green prefix, and the green prefix's members in
  the round, each its change task and pull request number: at most
  four, since a blamed unit sits above them and `forge.landing_members`
  is at most 5 (`../protocol/configuration/limits.md`). The change keeps
  them with its repair, so that a brief made after a restart says the
  same; a repair asked at the candidate of a round of one unit has none.
- **Its words** open `Repair landing gate N at HEAD: failed at prefix
  RED above GREEN`, as the candidate's open `Repair landing gate N at
  HEAD: failed at landing candidate`, N the gate's number, HEAD, RED and
  GREEN at twelve hexadecimal digits; then, on the next line, `Repair
  the failed landing gate. The landing's gates passed at GREEN and
  failed at RED, with this change's unit on top of it; a change below
  may still share the cause.`, then, where the green prefix is not the
  base, ` Changes landing below it: C1 (#N1), C2 (#N2).`, C the change
  tasks and N their pull requests, then ` The CI section gives its run,
  base and failed jobs.` and the preservation words, as the candidate's
  repair ends (`forge/change/repairs.md`). A brief too small for the
  list keeps the opening and cuts the list from its end.
- **Its result's account** is the candidate's, the landing's gate
  failed; it waits for a later landing, first in line by its kept time
  (`landing.md`, section 3).

## 8. What is shown

- **News,** to the goal of each member with a prefix in the round, once
  a probe's prefix is chosen: `landing L: change C probing at PREFIX,
  red at RED, on BASE`, PREFIX the probed prefix, RED the lowest red
  prefix known, BASE the round's base (`landing.md`, 4.2). The blamed
  members' goals hear them leave with section 4's words; the green
  prefix's, each member landed; the units above, proving again in the
  next round.
- **Each probed prefix carries its gates' statuses**, so the round's
  pull request, whose commits are the prefixes, shows each one's
  gates (`landing.md`, section 10).
- **The round's pull request after a search** names it in its body, a
  line after the members': `Search: red at RED, change C1, C2 blamed;
  green at GREEN.`, the blamed unit's changes in order, GREEN `the base`
  where g is 0.
- **`landing show`** (`landing.md`, section 14): the round's search, its
  green and red bounds, each its member's number and prefix, the base's
  0; its step `probing at PREFIX` while a probe runs; a blamed member's
  leaving with its red and green prefixes.
- **The changes view** (`landing.md`, section 14): the landing's step,
  `probing` with its prefix while a probe runs.

## 9. In the store

- **The round's search** (`landing.md`, section 12): absent until the
  candidate fails in a round of several units, then its green and red
  bounds, each a member's number in the landing (`landing.md`, 5.1), 0
  for the base; kept until the round ends. Its probes' gates are the
  members' gate states at their prefixes, which `Landing` already keeps.
- **A member's leaving** gains `blamed`: the gate, its run, the red
  prefix and the green one.
- **Gone with M2's fallback:** a change's mark to land alone, and the
  leaving `red round of N units at TOP: landing one at a time`. The
  store's patch drops the mark and reads a stored fallback leaving as
  `held`, the public reading it already had.
- **Patched in place,** before the first release (`../protocol/store.md`,
  section 2), by the store's patch and the golden store's line
  (`../protocol/baseline.md`, 7).

## 10. The world

In the forge's world (`landing.md`, section 16), the landing policy's
own cases and its fuzzy stories:

- **the boundary,** for two to five units with one unit's change
  breaking the gate by the fake forge's CI following content: the
  probes at section 3's boundaries, at most `ceil(log2(n))`; the blamed
  unit's changes repairing; the green prefix landed by one fast-forward
  of a pull request from its branch, its statuses on it; the units
  above stacked on the new tip in the next round and landed;
- **the base green:** the first unit blamed, no turn, the rest in the
  next round;
- **an observed boundary:** a later unit fixing an earlier one's red,
  the search holding the boundary its probes saw;
- **receipts:** a probe whose keys have receipts runs nothing;
- **a late outcome:** a probe's run ending after its round ended makes
  its receipt and moves no bound of the next round;
- **faults and bounds:** a runner's fault at a probe retried; past
  `gate_retries`, or a probe's run past `gate_timeout`, a hold;
- **between the boundary and the turn:** a member of the green prefix
  leaving, or a stale base, ending the round unlanded, every member
  left stacked again;
- **a restart** while probing and at each step of section 4, found again
  with no effect made twice;
- **the blamed unit's repair** naming its red and green prefixes and
  the changes landing below it;
- **what is shown:** each probe's news to each member, the blamed
  members' leaving, the round's pull request's line, `landing show` and
  the changes view;
- **the fallback gone:** no change marked to land alone, and a batch
  forming whole after a red.

In the process tier, the story of a red narrowed
(`../testing/process-tier.md`), written and compiled, run in that tier
after landing.

## 11. Later

- **Probes by test** (`../draft/merge-queue.md`, section 13): a probe
  that builds and runs only the tests the candidate failed.
