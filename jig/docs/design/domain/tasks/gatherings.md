# Gatherings

Provisional, 2026-11-16. A gathering is a task that holds the delegates
parties make beneath it, and ends on its requester's word once they have
ended: the primitive under an application's set of goals, such as
temper's milestone (`temper/docs/design/draft/milestones.md`, section
7). It is a core procedure (`README.md`, section 5) that does nothing
but wait. Parties make its delegates with `Delegate` (`README.md`,
section 4), and its requester ends it with `Conclude` (section 3).

## 1. In one page

- **A core procedure, never stepped.** Its executor is the core's, so no
  connector steps it and nothing it hears runs anything (section 4).
- **Made by a party, who ends it.** A party's `Create` makes it, as a
  root, with `ended_by: Requester` (`lifecycle.md`, section 9) and a
  report for its contract (section 2).
- **Filled by parties.** A party who may act on it makes its delegates
  with `Delegate`, funded from its budget, within its delegation and its
  deadline, as if it had made them itself (`README.md`, section 4;
  `../authority.md`, 8.4).
- **Concluded by its requester's word.** `Conclude` says no delegate
  joins it any more. It ends done once that is said and every delegate
  it has has ended, with a report counting their endings (section 3).
- **Everything else is a task's:** its tree, its budget and spend, a
  placed hold over its subtree, a cancel of its tree, its document and
  its delegates' reads.

## 2. A gathering

- **Its executor:** `Procedure { connector: RECURRING, code: GATHERING }`:
  the core's reserved connector number (`RECURRING`, `u16::MAX`, in
  `jig-domain-core-vocabulary`) and code `GATHERING`, 2, beside the
  recurring procedure's 1. The deployment always has it, and a party's
  role's delegation names it among the executors they may create with,
  as any procedure (`../authority.md`, section 3).
- **Its shape,** checked with every batch's checks (`README.md`, section
  4), whoever makes it, each refused as a task its executor cannot take
  is:
  - *its requester a party:* a party's `Create`, or an accepted proposal
    whose tasks are its accepter's. A task's batch, and a `Delegate`,
    whose tasks a task requests, make none;
  - *`ended_by: Requester`,* since nothing else would end it; a
    gathering with any other ending is refused as a misplaced ending
    is (`EndedBy`), and `Requester` stays refused on every other
    procedure;
  - *a report* for its contract;
  - *no conversation* (an agent's alone), and *no question owed*: its
    spec's `asks` false;
  - *no dependencies:* a batch naming one for it, and an amendment
    adding one, are refused. It waits on nothing, so it is active from
    the commit that makes it, and never kept (`lifecycle.md`, 8).
- **The rest is any task's:** its words (what it is for), parameters
  none, tracked and priority, label, listed and retention, budget,
  deadline and authority. Its authority bounds its delegates, as any
  task's bounds those it makes: its delegation's kinds, tasks and depth,
  its budget, its deadline (`../authority.md`, 8.1).

## 3. Concluding

- **`Conclude`** names a gathering: its requester's word that no delegate
  joins it any more (`../people/acts.md`, 1). It needs the task in the
  party's tree, or maintainer; a live gathering. An ended one is refused
  *ended*; a task that is not a gathering *not well formed* (`EndedBy`).
- **Kept for good:** the task's record keeps
  `concluded: Option<Concluded>`, `Concluded { by: u64, at: Wall }`: the
  person who said it and the wall time of the decision. Nothing clears
  it. A second `Conclude` is answered done and changes nothing: the
  first is kept. Its history keeps the line, as an amendment's.
- **No delegate joins it once concluded.** A batch beneath a concluded
  gathering is refused `Concluded { at }`, naming when, in the tasks
  child and to the party (*concluded*,
  `../../client/domain/contract/README.md`, section 3).
- **It ends** in the decision where the last of these comes to hold: it
  is concluded; no delegate of it lives; it is active, not held. So at
  `Conclude` itself when it has no live delegate; when its last live
  delegate ends; or when a release lifts its last hold. It ends done,
  closing as a procedure's result closes it (`lifecycle.md`, 1 and 7),
  with a report whose words are exactly
  `Concluded: D done, F failed, C cancelled.`, the counts of its direct
  delegates' endings its record keeps (`README.md`, section 3), in
  decimal, all three always.
- **Otherwise it ends as any task:** a `Cancel` closes it and its tree,
  concluded or not, and it ends cancelled. A placed hold over it holds
  it, and its delegates' failures hold nothing of it, since it depends
  on none of them.
- **Its delegates move neither in nor out.** A move (`changes.md`) into
  or out of a gathering is refused, when moves are built: a delegate
  joins it at its making alone.

## 4. What it never does

- **It is never stepped.** As the recurring procedure, it is never due
  for a connector's step: no step is asked for it, at its activation or
  on anything it hears.
- **It keeps no delegate's result.** Each result reaching its inbox is
  taken as it arrives, so the delegates it has over its life are bounded
  by its delegates live at once, never by its inbox (`messages.md`, 2).
  The results stay in the store, and its record counts their endings.
  Words a party sends it wait in its inbox as any task's, within its
  bounds; it reads none.
- **It decides nothing,** as no procedure does (`proposals.md`): its
  delegates' proposals and escalations pass it to its requester. A
  question a delegate asks its requester in the question form waits in
  its inbox, which no run reads (section 7).
- **It spends nothing of its own.** It has no run; what it spends is its
  delegates' (`../authority.md`, section 7).

## 5. Reads

- **Its document** carries `concluded`, or none while delegates may join
  (`../../client/domain/contract/documents.md`, 1).
- **Its delegates,** live and ended, are read as any task's: its live
  subtree, and the ended delegates' index (`../../protocol/store/README.md`,
  4.4).

## 6. Its world

Focused stories in the tasks world (`jig/tests/tasks`): a gathering made
and refused by its shape; concluded with no delegate and ended at once;
concluded with delegates live and ended with the last of them, its
words counting each ending; a batch beneath it refused once concluded; a
cancel of it concluded or not; a hold that keeps it from ending until
its release; more delegates over its life than its inbox holds; the
latch across a restart. The core world (`jig/tests/core`) has a party's
`Create`, `Delegate` and `Conclude` through to its end, and the client
protocol world (`jig/tests/client-protocol`) the requests and its
document on the wire.

## 7. Later

- **Questions to a gathering's requester:** a delegate's question to its
  requester waits in the gathering's inbox. An application whose
  delegates ask routes them to the gathering's requester itself
  (temper's milestone's goals ask its maker,
  `temper/docs/design/draft/milestones.md`, section 3); whether jig
  passes such a question up past a procedure is open.
- **A task line's `concluded`,** for lists that show it without the
  document: an application's need first.
- **A gathering made by a task,** gatherings within gatherings, a
  `Conclude` in a compound request, and proposing a `Delegate`: none
  until wanted.
