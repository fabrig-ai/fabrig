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

The binary supplies one deployment rule, covering every branch of every
repository, that enforces CI passed, up to date, and gate 1, an agent's
review, blocking and fresh at a clean head; no approvals. Every change
is reviewed by an agent before it lands.

## How it changes

Use `temper ctl settings apply` to change the rules. A kept change
reaches the next merge decision with no restart. To require more, such
as a person's approval or a second gate, add it to a rule.

The deployment's rules contain two standing requirements on the forge's
merge: requirement 5 checks the landing rules, guarded; requirement 6
checks being up to date by the landing rules, observed. The forge reads
its held landing rules when it judges them.

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

- Rules add: a project's rule cannot relax a deployment rule that
  covers the same branch.
- The rules' requirements 5 and 6 enforce the held landing rules.
  Removing these requirements leaves landing to the procedure alone.
