# Topics and landings as news

This document holds topics and landings as news, split from [`README.md`](README.md).

## 1. Topics, and landings as news

### 1.1 Topics

| Topic | Published when | Read by |
|---|---|---|
| landings on a repository's branch | a landing branch's tip moves | goals working in that repository; landing queues; tiers (`../after-landing.md`, section 3); follows of that branch (`follows.md`, section 3); a subscription naming trailers (`../after-landing.md`, section 8) |
| CI on a head | a head's CI, as `keeping-up.md`, 3 reads it, changes | changes; tasks that subscribe |
| a pull request's state | opened, closed, merged, its head moved, its mergeability known | changes; tasks that subscribe |
| comments and reviews on a participating object | another party writes there | the task dealing with it (`README.md`, section 7) |
| a project's findings | a finding is filed, handed on or routed to a task, or its labels change | the task that holds it, or the goal it is routed to, which the root subscribes in that decision (`../areas.md`, section 5; `../findings.md`, section 6) |

The change child domain reads its facts directly and needs no
subscription; topics are for tasks whose executors are agents, and for
goals.

### 1.2 Every landing is news

- **For two kinds of goal:** the goal whose change landed, whose step is
  done; and every other active goal with changes open or planned in the
  same repository, whose base has moved. Concurrent plans meeting this
  way is the ordinary case, not an exception for outside activity.
- **A goal hears of landings** through the forge's topic for goals
  (jig's `domain/connectors.md`, 5.4): the landings on the branches its
  subtree's changes target, which the core subscribes it to as the first
  change into each is made, until the goal ends. A goal ended by its
  closer (`branches.md`, 2) takes no subscription to the repository's default
  branch's landings, including for a repair there. Landing such a goal's
  held branch creates no main-landing subscription, either at the change's
  start or when its plan is projected. Reds still reach the goal through
  its branch's tiers (`updates.md`, section 1).
- **Those are a goal's only topics from the core.** CI on its changes'
  heads is each change's own, read by its procedure (1.1); the goal hears
  a change's end or hold as any requester hears its delegate's (jig's
  `domain/tasks/messages.md`, 1), and a run that wants a head's CI subscribes to
  it. So the topics the core keeps for a goal number its landing branches,
  however many changes it makes, and its run keeps room in its task's
  subscriptions for its own: a person task it waits on, a repair before a
  held landing's release, its timers (`../agent/tools.md`). A goal's
  subscription to CI on a head that the core holds at restart is dropped
  there, its jig row with it.
- **A landing temper did not make** is news like any other: what landed
  is read, a pull request or a range of commits.

### 1.3 Which landings wake

Not every piece of news wakes. The connector classifies each landing for
each subscribed goal (jig's `domain/connectors.md`, 5.3), without an
LLM:

- **its own change landed:** kept; what comes after it is due
  mechanically, through the goal's dependencies;
- **another landing overlaps its work:** wakes. Overlap is the files the
  landing touched against those of the goal's own open pull requests at
  their heads, and, for changes not produced yet, the paths their specs
  name, as a hint;
- **no overlap:** kept, for its next run to read;
- **overlap unknown** (a landing too large to list, a read that failed):
  wakes, since a wasted turn is cheaper than a missed conflict.
- **A hold parked until a landing** (jig's `domain/tasks/lifecycle.md`, 10):
  subscribed to the landings on a branch, with path prefixes or none; a
  landing wakes it, and so lifts it, when one of its files lies under a
  prefix, or with none named, or when the overlap is unknown.

The files are read from the forge (a pull request's files by their paths
alone, `PullFiles` without `sides`, or the comparison of the branch's
old and new tips, by names alone: skein's `forge/README.md`, 3.2 and
3.3), cached per head and cut at a limit past which
the overlap is unknown. A comparison names at most `forge.client.rows`
paths and counts the rest (`changed`): one that leaves a path out, an
unusable answer or an uncertain completeness makes the overlap unknown
and wakes every affected subscriber, and its paths are a landing's files
for workflow discovery too (`adoption.md`, 7). A pull
request's pages count only while fresh reads before and after them name
the same head; a head that moved voids the listing.

## 2. Open questions

None.
