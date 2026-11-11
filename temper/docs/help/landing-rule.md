# Landing rule

Kind: concept

What a change must meet before temper merges it into a branch: CI, being up to date, gates and approvals, by rules the configuration holds.

## What it is

A landing rule covers some branches of some repositories, by a pattern
of their names, and says what a change into them must meet before temper
merges it:

- CI passed at the change's head;
- the head up to date with its base's tip;
- its gates, each an agent's review, blocking or not, or, for a
  repository with no CI, one of its checks;
- its approvals, each a number of distinct people holding a role of the
  project, by reviews on the forge.

A gate's or an approval's freshness says at which heads a verdict still
counts: `exact`, the head it was given at; `clean`, every head reached
from it by updates temper made that merged cleanly.

The rules are settings: kind 1 holds the deployment's rules, kind 2 each
project's. Rules are numbered from 1 within their scope and add together.
The forge judges each change to them before it is kept.

A deployment rule's pattern begins at `forge` (hex `666f726765`), then
the forge's host, repository owner and name, then `branch` and its segments.
A project's rule may use that form or begin at `branch` (hex `6272616e6368`)
and its segments alone, covering that branch in each of the project's
repositories, including those adopted later.

An enforcing approval rule on a box needs nothing added for your approval:
temper knows your account on the box's own Forgejo, and on GitHub when temper
made the App from your own account. Name your role in the rule, owner (0) on a
project you made.
On this machine temper's agents and CI jobs run as you, so this stops their mistakes, not one trying to get past it.
Acceptance needs enough distinct people known on the forge of each adopted
writable repository the rule covers, holding the required project role. A
refusal names the project, forge, role, required count and current eligible
count. A rule covering the playground's forge is refused: no one can approve
there.

With several members (`temper-box serve`), a managed Forgejo or a pasted App:
A deployment owner login or a matching display name does not establish a forge identity.
Add the human collaborator on the forge. On Forgejo, a repository administrator must also grant temper's
forge user repository admin: collaborator reads can require it even with write.
This bot permission is separate from the human permission mapped to a project
role. Configure the existing project's project-policy permissions where
needed, re-adopt with `repository adopt` and a new key, preserving its options,
and verify the required role with `project show`. Then submit a fresh proposal
against the newest revision.
The commands and policy setup are in temper help configure, Human approvals.
Stock maps admin to maintainer (1), write to member (2), read to observer (3);
use the mapped human role in the rule. Owner (0) is reserved for the project
owner; never map all permissions to owner. Changing policy 1 affects future
projects; re-adoption does not replace existing project roles. temper's own
forge writer never counts, even with an existing role. Or leave approvals
out and use agent review alone.
Membership can change later; only known people with the role at the merge
count.

Each project's rule 1 is the stock's: every branch of each of its
repositories, enforcing CI passed, up to date, and gate 1, an agent's
review, blocking and fresh at a clean head; no approvals. The project's
owners may change it; removing or resetting it brings the stock back.
The deployment has no rule of its own until its owners add one; its rules
add to every project's.

## How it changes

Use `temper ctl settings apply` to change the rules. A kept change
reaches the next merge decision with no restart. To require more, such
as a person's approval or a second gate, add it to a rule.

The deployment's rules contain two standing requirements on the forge's
merge: requirement 5 checks the landing rules, guarded; requirement 6
checks being up to date by the landing rules, observed. The forge reads
its held landing rules when it judges them. Being up to date is checked
from commit ancestry; a large goal's diff does not have to fit a comparison
read. A failed or incomplete ancestry read cannot permit a merge.

## What temper guarantees

- A rule that enforces holds as requirements: the forge merges no change
  into a branch it covers until CI passed at its head, the head is up to
  date, its blocking gates passed and its approvals are met, whatever
  the procedure does.
- A change's procedure asks for each blocking gate that holds as a
  requirement, each gate that does not block, and, for a repository with
  no CI, each of its checks, which block. It waits for CI and the
  blocking gates before it queues the change, and at the queue's front
  checks them again, and that the head is up to date, before it asks to
  merge.
- A rule that does not enforce makes no requirement: its blocking gates
  are not asked for, and its approvals are not waited for.

## Acting on it

- `temper ctl settings show landing-rule` reads a deployment rule;
  `--project PROJECT` reads a project's rule.
- `temper ctl settings apply` changes them; `settings propose` offers
  a change for an owner to decide.
- `temper ctl change show` reads a change's way to landing.

## Limits

For the configuration members that need a restart, read `temper help configuration`.

- Rules add: a project's rule cannot relax a deployment rule that
  covers the same branch.
- The rules' requirements 5 and 6 enforce the held landing rules.
  Removing these requirements leaves landing to the procedure alone.
