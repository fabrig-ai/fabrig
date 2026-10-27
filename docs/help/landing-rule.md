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

The rules are the configuration's: `application.landing` in
`temper.json`, the deployment's rules and each seeded project's. They
are not settings.

`temper init` writes one deployment rule, covering every branch of
every repository, that enforces CI passed, up to date, and gate 1, an
agent's review, blocking and fresh at a clean head; no approvals. Every
change is reviewed by an agent before it lands.

## How it changes

temper reads the landing rules at each start. A rule that enforces
becomes requirements on the forge's merge, one per criterion: the
deployment's rules join the default rules (`temper help policy`), and a
project's join its seeded policy at the deployment's first start.

To change them, edit `application.landing` and restart: to require
more, such as a person's approval or a second gate, add it to a rule.

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

- The configuration's `application.landing`, then a restart.
- `temper ctl`: `change show` reads a change's way to landing;
  `settings show rules` shows the requirements the deployment's landing
  rules became.

## Limits

- temper does not hold landing rules as settings: no ctl tool or host
  tool changes them, and a change needs a restart.
- A project made after the first start has the deployment's landing
  rules alone.
- A project's rules changed after its first start do not reach its
  policy, which keeps the requirements seeded then.
- The deployment's rules join the rules' default: a stored change of the
  rules wins over them. One that drops their requirements drops their
  blocking gates too, and a landing rule changed in the configuration
  then reaches no decision until that change is reset.
