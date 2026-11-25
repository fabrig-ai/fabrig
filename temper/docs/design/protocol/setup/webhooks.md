# Webhooks on a box

This document holds webhooks on a box, split from [`README.md`](README.md).

## 1. Webhooks on a box

The listener is skein's (`forge/connection-webhooks.md`, section 1), and so are
the signature, the events and the hints they become (`../forge/README.md`, section
6). This section says what makes them reach the engine on a box, and
that they cover what its work waits on.

- **Made at adoption, checked at each start** (`reconcile.md`; skein's
  `forge/setup/forgejo.md`, 1.5, and 1.3, step 6): one hook per adopted
  repository, to `http://<listener>/hooks/<number>`, with the events of
  skein's `forge/README.md`, section 6, and the forge's secret. An owned
  Forgejo's `app.ini` allows the listener's address (`owned.md`, 2). On a box,
  nothing is left to the person. A managed Forgejo on this host has the
  same hooks and its `app.ini` is the person's (`reconcile.md`, 6); one on another
  host has none (`reconcile.md`, 5).
- **What a box's work waits on, and what ends each wait,** with the keep
  reading the forge within `hinted` of the hint, which gathers one
  change's hints into one read (`../../domain/forge/keeping-up.md`, 2):

  | The domain waits on | Done on the forge by | Its delivery | Its hint |
  |---|---|---|---|
  | a landing branch's tip | a merge, a person's push | `push` | the branch moved |
  | a pull request's head, base or state | a push to its head or base; closing, reopening, retargeting or editing it | `push`; `pull_request` | the branch moved; the item |
  | CI on a head | each Actions job ending | the job's end | the commit |
  | a participating object's comments and reviews | a comment, a review, an approval or a rejection | the item's comment or review events | the item |
  | an issue of a live goal | editing, closing, labelling or assigning it | `issues` | the item |
  | drift of a branch temper holds | a push, a deletion, a creation | `push`; `delete`; `create` | the branch moved; the branch |

- **Echoes go up too.** Forgejo delivers temper's own writes. The client
  reads on every hint it holds a resource for, so each costs one read;
  temper, the owner, drops its own echoes by key and by pusher (skein's
  `forge/client.md`, section 4; `../../domain/forge/keeping-up.md`, 2).
- **What Forgejo settles after it delivers,** a pull request's
  mergeability, which it answers not mergeable while it checks (skein's
  `forge/forgejo.md`, section 2), is read again at `hinted`, doubling up to
  `poll` apart, within `settle` of the move, as is a hinted job's CI
  that reads pending.
  These are the owner's fresh reads: the client runs no window (skein's
  `forge/client.md`, sections 4 and 5). A pull request that reads not
  mergeable within `settle` of the move is never taken as conflicted
  (`../../domain/forge/keeping-up.md`, 2).
- **What sends no delivery,** and so is found by polling alone: a
  commit status that a system other than Actions posts, which a
  box has none of.
- **Polling is unchanged** (skein's `forge/client.md`, section 4): the
  keep's alarms, its paces and the slow pass are what they are with no
  webhook. A delivery lost while `temper-box serve` is down, refused for its
  size, or never sent is found by them; Forgejo does not send a failed
  delivery again (skein's `forge/forgejo.md`, section 2).
- **GitHub sends no delivery to temper:** its App's changes come from
  the App's delivery log, read by the forge component, with no address
  GitHub must reach and no webhook secret (../forge/README.md, a GitHub App's
  delivery log; `github.md`, 6).
- **Checked:** the protocol-forge world's hinted case
  (`testing.md`, 4) checks the table's third row, CI on a head: the one
  wait of a landing that the forge ends. Its other waits end on temper's
  own writes, its push, its pull request and its merge, whose deliveries
  are echoes. Within the case's deadline, on its virtual clock, only the
  job's end delivery brings a read of its CI, so the change lands with
  that delivery and has not landed at the deadline without it:
  - the keep's `poll`, `poll_max` and `slow` are past the deadline
    (../configuration/limits.md);
  - so is the change's `stall`, whose end reads its pull request afresh
    before it holds the change (`../../domain/forge/keeping-up.md`, 3 and `../../domain/forge/change/repairs.md`);
  - its job outlasts `settle` from its change's push, so the rereads
    within `settle` end before CI does (`../../domain/forge/keeping-up.md`, 2).

  That a real Forgejo sends a job's end to the hook reconcile made,
  signed, is the process tier's deliveries test (`../../testing/process-tier.md`).

  The first two rows, moved by someone else's push or edit, and the
  others, comments, reviews, issues and drift, are checked from hints
  in the worlds (`../../testing/tiers.md`, 1.2, and `../../testing/checks.md`, 3).

## 2. Open questions

- **Webhooks from another host,** over TLS (skein's
  `forge/connection-webhooks.md`, section 1), once skein has TLS's server side.
  Until then a managed Forgejo on another host polls (`reconcile.md`, 5).
- **A hook left at an old listener's address,** after the listener
  moves, is skein's question (skein's `forge/setup/README.md`, section 6).
  Default: left alone; the person deletes it.
- A managed forge whose host list refuses the listener: `reconcile.md`, section 7.
