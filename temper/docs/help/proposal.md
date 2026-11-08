# Proposal

Kind: concept

A request for an act beyond its proposer's authority, which waits for a party or task that may decide it: a batch of tasks, an effect, a widening, an amendment, or a change of settings.

## What it is

A proposal asks someone with more authority to authorize one act:

- a batch of delegate tasks;
- one effect, such as a write to the forge;
- a widening of a task's authority;
- an amendment that needs more authority than its task has;
- the release of a task a party's hold holds, which only the hold's
  owner decides (`temper help hold`);
- a change of settings, which only the owners of every scope it touches decide
  (`temper help settings`).

A proposal is named by its proposer and its number: `t41/3` is task 41's
third, `p3/17` party 3's seventeenth. Every result that offers it prints
its name.

A proposal of settings carries its items, the revision it was made
against, and its reason. Its review shows, for each item, the value now
and the value it would be, each whole. It groups changes under "temper
enforces" and "your agents follow": instructions and documents are for
agents to follow; other charter fields and rules are enforced. A charter
changing both appears in both groups, with its whole values below.

The review counts live tasks by the revisions they hold. New work takes
accepted settings; accepting restarts nothing. With `--running stays`
(the default), tasks keep their charter revision; `--running moves`
moves compatible tasks at their next run. Conversations move at their
next run with either choice. A task whose charter is removed or whose
model is no longer allowed retains its revision. A live run continues
until it ends. Authority, including current rules and project policy,
stays current even when a task keeps its charter revision. The review
explains these conditions; it does not count which tasks could move.

## How it changes

A proposal waits with a holder: a party or a task whose role decides its
kind and whose authority covers the act. It is then:

- accepted, which makes the act, funded by the accepter;
- rejected, with a reason its proposer hears;
- passed up, to the holder above;
- withdrawn by its proposer, including when a run's fresh proposal of settings
  replaces its own that waits.

A proposal of a release waits for the hold's owner alone and is never
passed up; lifting the hold, or a newer revision of it, withdraws it.

A pending proposal of settings is withdrawn as stale when another
change, or a start, makes a new revision.

## What temper guarantees

- A proposal grants nothing until it is accepted.
- A proposal is judged again when it is accepted, against the authority
  and policy that stand then.
- A holder decides only the kinds of proposal their role decides.
- A proposal of settings is accepted only by a person who owns every
  scope its items touch (`temper help settings`),
  with a running choice: running work stays on its revision, as when no
  choice is given, or moves.
- Its proposer reads the outcome with `proposal show`. A run that
  proposed settings first receives its ctl name (`tN/K`) and who
  decides it, then hears its outcome in its task's messages:
  `accepted: revision N made`, `rejected: REASON`, or
  `withdrawn as stale: the newest is N`.
- A decision asked again under its key gets its first answer again. A
  decision of a proposal decided already, by anyone, answers `decided
  first:` with who decided it, how and when, and changes nothing.

## Acting on it

- `temper ctl`: `inbox` lists what waits on you, proposals among it;
  `proposal show` reads one, with its review for settings; `proposal
  decide` accepts, rejects or passes it up; `settings propose`
  proposes a change of settings.
- A run proposes with the host tool `propose` (a batch, an effect, an
  amendment, a widening or a release), decides with `decide`, and
  withdraws its own with `withdraw`.
- A run whose charter names the settings family and whose authority
  holds it proposes a change of settings with the host tool
  `propose_settings`: charters 3 and 11 in the stock
  (`temper help charter`). Its answer tells the run that the card waits in this
  chat and in Needs you, for a deployment owner to decide; an asking charter
  may give a large set in parts. The configuring chat reports an acceptance
  with its revision, revises after a rejection's reason, and reads the
  settings again and proposes afresh after a withdrawal as stale
  (`temper help chat`).

## Limits

For the configuration members that need a restart, read `temper help configuration`.

- The person's client shows a proposal of settings with its reason and
  summary, without its whole values. An owner can accept it, with running
  work staying, or reject it with a reason under Needs you.
- A run's fresh proposal of settings replaces its own that waits, in one commit.
  A refused fresh proposal leaves the one waiting standing. The stock's
  configuring chat holds no decide family and cannot call `withdraw`.
- A party's proposal withdrawn as stale names the revision it was made
  against, not the newest: read `settings show`, and propose again.
- temper does not let ctl withdraw a proposal.
- A proposal of a batch names amounts: a budget of as much as fits is
  not proposable.
