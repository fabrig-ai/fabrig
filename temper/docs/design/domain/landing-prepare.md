# Preparing a landing

This document holds preparing landing candidates and rounds, split from [`landing.md`](landing.md).

## 1. Prepare

### 1.1 Rounds

- **A round** is one stack: its base, the train's tip read afresh as it
  starts; each member's head, read afresh; the train's upstream's tip,
  read afresh, where the train has an upstream (1.2); each member's
  prefix, the commit the prepare job pushed for it, and how it combined
  the member, merged or squashed; and its pull request, from the top's
  branch. The top is the last member with a prefix; its prefix is the
  round's candidate. The landing's rounds are counted from 1.
- **A branch per member:** member n's, n its number in the landing from
  1 (`landing.md`, section 12), is `temper/<root>/l<task>`, the landing's own branch,
  for the first member, and `temper/<root>/l<task>-<n>` for each other
  (`forge/branches.md`), the landing's task; each held by the landing
  from its making, since its members are sealed. The round's job pushes
  each member's prefix to its branch, expecting the branch new; a member
  that conflicted, or left, has none in that round.
- **A round ends** at its turn, when a member leaves after the job
  stacked it (`landing.md`, 4.3), but for a unit a whole search blames, or when its base
  goes stale (`landing-turn.md`, section 1). Ended
  unlanded, the landing closes the round's pull request and deletes the
  branches of its first changed member and of every member above it,
  its own effects (`landing.md`, section 11), before anything else, so that the next
  round's job finds them new: from the member that left, or every
  member's for a stale base. With no member remaining, no next round
  needs new branches: task closure releases them instead, without a
  separate kind-15 effect. The members below keep their branches,
  prefixes and receipts. A prepare still running settles before cleanup;
  its delivered prefixes are read and retained even when the round ends
  before preparation recorded them. A restart at that boundary resumes
  the same cleanup before any new prepare. Once the cleanup is
  answered, the next step is due at once: the next round's prepare, its
  opening, its proof and turn, or the landing's end, reading no fact of
  the ended round (`landing-waits.md`, 3).
- **The next round** takes the members still in the landing: its job
  stacks only those above the last kept prefix, onto it or onto the
  base (1.2); with none, the last kept prefix is its top, and no job is
  made. Its receipts hold for every key that did not change.
- **A member whose head the base contains** has landed already, by
  another hand: nothing is merged for it, its change is told it landed
  at the base, and the stack goes on without it.
- **Released** as the landing closes: its branches deleted, as any held
  branch's release does (`forge/branches.md`); one never pushed is
  dropped with no write.

### 1.2 temper's merge train

The default, for a project with no prepare command:

- **A job** (`worker/jobs.md`, kind 1, `prepare`), delegated by the
  landing, pushes under its branch holds (jig's `domain/tasks/holds.md`,
  1). It checks out the train's repository by the round's starting
  commit, its base or last kept prefix, and combines each member above
  onto the prefix below in landing order by `combine` (`landing-settings.md`, 1.1). Once all
  are combined, it pushes each prefix to its member's branch, expecting
  it new, lowest first. temper's prepare runs no project command. The
  two ways use `../protocol/tooling.md`, 4's messages:
  - **merge:** the head merged, no fast-forward, the prefix below as
    the first parent and the head as the second, committed with the
    member's merge message: its title, its description's first
    paragraph, and its change, pull request and head as trailers,
    with its goal's for a train into its repository's main;
  - **squash:** the same merge, its tree committed as one commit whose
    only parent is the prefix below, with the member's squash message:
    its title, its description's first paragraph, then its change,
    pull request, head and goal as trailers (`../draft/squash-in.md`,
    section 3). A squash whose tree is the one below's is still a
    commit.
- **A conflict:** a member that cannot merge cleanly onto the prefix
  below is skipped; the job stacks the next on that prefix. In the
  same job it merges the conflicting head onto the round's base:
  a conflict there is with the branch, uses that merge's files, and
  the change resolves (`landing.md`, section 3). Otherwise it conflicts only with an
  earlier member, uses the stack's files and waits first in line:
  once that earlier member lands, any conflict is with the branch.
  A member stacked directly onto the base conflicts with the branch.
- **Authored and committed** by the repository's git identity
  (`../protocol/credentials.md`, 4), both ways: the identity the
  deployment's agents' commits carry, and temper's own.
- **A goal's branch reads one commit per change,** its title and
  trailers naming the change's task, pull request, head and goal, and a
  merge for each catch-up or resolution of main
  (`../draft/squash-in.md`, sections 2 and 6); a goal's landing into
  main is a merge titled with the goal's title, its trailers the goal's
  landing change's and the goal's, which `../testing/qa/landings.md`, 4
  reads.
- **Why merge or squash** is `../draft/merge-queue.md`, 5.3: a goal's
  branch goes on after it lands, and merges onto main; a change into a
  goal's branch ends when it lands, and squashes. A squash meets the
  conflicts its merge would, and has its tree, so its keys (`landing-proof.md`, 1.1).
- **A member that brings its upstream's commits is merged** whatever
  `combine` says: one whose head holds a commit the round's base lacks
  and the train's upstream holds, as a catch-up of main made as a
  change into a goal's branch does, so that those commits stay the
  upstream's. A train's upstream is the branch its own branch was made
  from and lands into, a goal's own branch's base (`forge/branches.md`,
  1); a train into a repository's main, or into a branch no goal holds,
  has none. The job tells, from the upstream's tip the round read
  (1.1), given in its request (`worker/jobs.md`).
- **Its outcome:** a successful report's `combined` member is taken at
  the fresh tip of its branch, with the reported combination. The branch
  is held exclusively for this round and delivery precedes the report;
  an additional commit-range read cannot refuse that delivery. The round
  retains each reported combination and conflict before reading
  its prefixes, so a read waiting for client room or a restart cannot
  consume that report. A changed round discards the earlier round's
  combinations while keeping departed members' conflict words.
  The engine reads each member's branch afresh once the job has ended.
  Without a successful report, its tip is recognized as the member's
  prefix when its parents
  are the prefix below and the member's head, a merge, or the prefix
  below alone with the member's head in its `Head` trailer, a line of
  its message's last paragraph, a squash (skein's `Read::Commits`); the
  round keeps which. The prefix
  below is that of the nearest member below with one in the round, or
  the commit the round stacks from. A member the job's result names in
  a conflict leaves (`landing.md`, 4.3), saying which.
- **A job's own failure** (a preparation that fails, a push refused, a
  deadline passed) is the project's, not a member's: jig tries it
  again within the job's tries, then holds it, and the landing holds,
  saying so (`landing.md`, 4.2).
- **Lost or partial:** a job whose attempt is lost is tried again by
  jig, in the same child. Before each delegation, and once a job has ended, the landing
  reads its members' branches: each prefix found there in order, each
  on the one below, for the round's base and that member's head, is
  taken, and a job is made only for the members above the last taken.
  At each step the landing reads afresh whether the round's prepare
  lives, has answered or is absent, from jig's context and its job
  record, never from its own marks: it waits while it lives, reads an
  answer first, and asks once again where it is absent
  (`landing-preparing.md`). A failed prefix
  read is no evidence of an absent push. A base found stale as preparation
  ends closes that round before proof. After cleanup, the next round reads
  its new base and advances its preparation number before asking for its
  suffix: an earlier round's admitted job cannot keep that suffix waiting.
  Before that base is initialized, no prepare report belongs to the new
  round: repeated reports and retained combinations from the ended round
  are discarded before its branches are read, including after restart.
  A repeated job compares each rebuilt prefix's whole tree with the
  existing branch, including Markdown paths, and checks that the existing
  tip contains the prefix below and, for a merge, the member's head.
  Where they agree it reuses that tip before stacking the next member,
  so the chain keeps the earlier delivery's commits. A create refused
  because that same tip is already there is verified by fetching it and
  counts as delivered. A failed job after which the read takes no new
  prefix holds the landing.
- **Not drift:** the landing's branches are its own, written only by
  its jobs under its holds; no member's own branch is written.

### 1.3 The project's tooling

- **A project's prepare command,** named in its landing settings, runs
  in the same job, at the home's pinned revision, with no model and no
  forge credentials: it may regenerate derived files and settle
  conflicts its rules know, and makes the prefixes the job pushes to
  the members' branches as temper's own does. Its conflict sends the
  member to a resolution as 1.2's does.
- **Its contract** (the arguments, the files, their formats, the
  environment, the exit codes) is `../protocol/tooling.md`.

## 2. Open questions

None.
