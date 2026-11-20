# A brief for each person

Draft, 2026-11-18, from a session with the user that day. The few lines
at the top of a project's Home, the status agent's brief
(`../domain/forge/attention.md`, 3), become each person's own, written
when they look and paid as their own work. The user: "we'll need this
sorted", on finding that a project with several members never has one
unless a key pays for it.

## 1. In one page

- **Today a project's brief is the project's work.** Its run's payer is
  the project (`../domain/forge/attention.md`, 3, *What funds it*), and a
  project with another party is paid for by a key alone: the
  deployment's, or one whose uses name the project. No plan pays for it
  (jig's `domain/accounts.md`, 4.2, *A shared project's automatic
  work*). On dev.fabrig.ai on 2026-11-18, project 2 had several members
  and two plan accounts, so each sitting's run was made and waited for
  good, and Home never showed a brief.
- **A plan can't simply pay for it.** A plan's account serving another
  person's work is refused by the LLM providers' rules: "You may not
  share your account credentials or make your account available to
  anyone else" (jig's `domain/providers.md`, 2.6), and plan limits
  "assume ordinary, individual usage" (3.4). A brief the owner's plan
  writes for every member is the owner's plan working for others.
- **So the brief becomes each person's.** The look that starts a run is
  a person's. Their run writes their brief, from the same facts, and is
  their own work: its payer is the person, so their own plan pays for it
  as it pays for their chats. Every project with a member who has an
  account gets briefs, with nothing to set. A key naming the project
  still pays, as for any work inside it.

## 2. A person's brief

- **When:** in the decision of a party's `Looked` that moves `previous`
  (`../domain/forge/attention.md`, 2), by section 3's rule, read per
  person: something new since this person's brief (a landing past its
  `landed`, or the open findings' mark other than its `findings`), their
  brief over an hour old or none, and no run of theirs in flight. A
  party's first look starts none, as today.
- **Kept per person:** `ProjectFacts.brief` and `ProjectFacts.running`
  move to a row a party and a project, beside `Looked`, at most as many
  as `Looked`'s rows. `landed` and `last_landed` stay the project's. A
  run in flight over an hour counts as none, and its late result is
  dropped, as today.
- **Funded as the person's work:** the procedure's task is made in the
  person's pool in the project (jig's `domain/authority.md`, section 7),
  within that pool's room and the rules' run cap, so its payer is the
  person. The project's period no longer funds briefs. An account is
  chosen as for any of the person's runs (jig's `domain/accounts.md`,
  4.3): their own, then those naming them or the project, then the
  deployment's.
- **Its words:** section 3's words, with the span since this person's
  last brief, and their name where the words name the reader. The facts
  are the project's and read the same for everyone.
- **Drawn** on that person's Home alone, as today's brief is drawn, new
  to them by the same rule (`../client/pages/home.md`, 1).

## 3. When nothing pays

- A person whose run waits on an account, or whose pool has no room, has
  no brief. Home says why, in one line where the brief would be: "No
  brief: no account pays for your work here." with a link to Accounts,
  rather than nothing. Its read is the run's wait, as the accounts
  document shows it (jig's `client/domain/contract/documents.md`, 1).

## 4. Upgrading

- `ProjectFacts` changes its layout: its rows lose `brief` and `running`,
  and the per-person rows are new (the user's upgrade rule, 2026-11-17:
  a stored value at an older layout is converted as it is read). A
  project's old brief and run are dropped when converted. A brief is
  remade at each person's next sitting, so nothing a person relies on is
  lost.

## 5. Out

- **A shared brief** paid by a key, beside the personal ones: a key
  naming the project pays each person's brief already.
- **Briefs across projects** (`surfaces.md`, section 15's open question).

## 6. Open questions

- **The cost** of one run a person a sitting, at most hourly, against one
  a project. Default: accepted, since a run is at most 4 turns and a
  person who doesn't look starts none.
- **A person with no account** on a deployment with a key that names
  them or the project. Default: that key pays, by 4.3's order.
