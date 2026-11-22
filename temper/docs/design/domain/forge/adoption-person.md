# A box's person on its forges

This document holds a box's person on its forges, split from [`adoption.md`](adoption.md).

## 1. A box's person on its forges

On a box, the forge knows who its one person is on each forge it can vouch
for, with no collaborator and no adopting again (`../../draft/forge-identity.md`,
sections 3, 4 and 9). It keeps that as one fact of the forge's record, its
**link** (`../../protocol/forge/engine.md`, 1): on this forge, that party is that user.
One person on a box means at most one link per forge, so it has no family, no
bound and no restore of its own, and it goes with the forge if that's removed.

- **The person is the deployment's sole owner,** handed by the root at each
  start, once the core's and the forge's records are loaded (`../root-restart.md`, 1.1):
  the party jig's people answer as the deployment's sole owner (jig's
  `domain/people/README.md`, section 4), or none. In that decision every link whose
  person isn't that party is cleared and logged, and while the root has
  handed none, no link is made.
- **Only under a box** (`../../protocol/setup/box.md`, 1): a forge connector built
  without `--box` makes no link, whatever it's told, and keeps none: in the
  owner's decision above, every link it restored is cleared and logged,
  whoever the owner is, so a box's data started by plain `temper serve` keeps
  no link and its lookups fall back to seeds (`../../draft/forge-identity.md`,
  section 9). It is the `--box` the connector was built with that decides,
  not the box's pipe: `BoxClosed` (`adoption-connection.md`, section 1) stops new links for the rest of
  the run but clears none, and the next start judges them again.
- **From an owned Forgejo's `Reconcile`** (`../../protocol/setup/reconcile.md`, 2): given
  the owned forge's person's login, it answers that account's id or none for
  an account gone. An id links that user to the person; the same link again
  changes nothing; a different id replaces the link, as a person may remake
  their account, and that's logged. An account gone leaves the link as it is,
  and that's logged. A managed or fake Forgejo is given no login, so it never
  links.
- **From a GitHub App's account** (`../../protocol/setup/github.md`, 2): each `GET
  /app` a plan answers says whether the App is the box's person's, a
  manifest's App whose `AppMade` equals its `AppId` and whose account's
  `type` is `User`. If it is, its account's id links that user to the person,
  as above; if not, the link is cleared and logged. So a restart before the
  link was kept loses nothing, a pasted App replacing a manifest's has no link,
  and an App transferred away and back links again. A read that fails changes
  nothing.
- **Never temper's writer:** a link to the forge's retained writer is refused
  and logged, and the forge keeps what it had. Reviews skip the writer before
  any lookup (`change/gates.md`, 2), so one human counts once on a forge, and temper never.
- **Its log lines** name the forge and what happened: the account gone and the
  link kept, refused as temper's writer, replaced, or cleared.
- **A link replaced** leaves its old user a stranger, or the seed that was made
  for it before the link: a second account of a person who made one on
  purpose, which counts only if a rule asks for two people.

## 2. Open questions

- **Links beyond a box's one person** (`adoption-person.md`, 1): several members need a
  family of links and evidence of the connecting owner; a pasted App or
  a second account needs user authorization or a verified claim on a
  review (`../../draft/forge-identity.md`, section 12). Default: adoption's
  seeds there. Containing agents and CI is required before an owned
  Forgejo's approval can resist an agent trying to bypass it
  (`../../protocol/setup/README.md`, section 9).
- **The approval story on GitHub** (`adoption-person.md`, 1, `change/questions.md`, 2): the complete box story
  belongs beside the owned-Forgejo domain story, then a real person's
  review in the live tier (`../../testing/README.md`, section 7). Default: lower
  setup, link and review checks establish those separate pieces alone.
