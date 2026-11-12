# checkouts: status

2026-11-12. Where the tree's code lags or departs from [checkouts.md](checkouts.md).

- **Not yet:** *One object store per repository* and *One operation at a time*: every prepare admitted before its store is made makes the store itself, a full clone each, since whether to make it is fixed at admission (`temper-worker-domain-checkout/src/cache.rs`, `Cache::hold`; `src/tests.rs`, `concurrent_prepares_make_one_remote_independently`), so a deployment of 2026-11-12 saw up to 36 clones of one repository against one forge at once; and a store's token is its slab place, so a restart may hand a remote another's [in flight: change 1508].
- **Not yet:** *One object store per repository*: a start sets `stores` aside and `Store` clones before it looks for its store (`temper-worker-protocol-git/src/files.rs`, `layout`; `src/files/objects.rs`, `Store`), so every prepare after a restart starts cold: 5 to 15 minutes of prepares failing `store: TimedOut` after each redeploy of 2026-11-12 [in flight: change 1509].

`checkouts.md`'s design delivery has no known gap in running the repository's
`.temper/pre-pr` and refusing a failed check. Its landing pin stays frozen
for the hold. `checkouts.md`'s delivery retains the checkout's ancestry through
the git snapshot operation (`../../protocol/git.md`, 4.7): staged and committed merges
keep their second parent, checked against the remote, and an unchanged
local commit is delivered intact. The checkout step test holds the request's
remote and account; the git protocol's snapshot tests hold its invocations
and outcomes on scripted output. The checkout world's `suite::delivery`
checks the resulting fake object graph, preserved ids and messages, and
refusal of an unpushed second parent. No deployment replay is on record.

Unchecked saves use the same snapshot. The checkout world's delivery rows
cover both checked and unchecked pushes. A checked delivery after an
unchecked save keeps the fresh-commit requirement even for an equal tree;
the protocol's `a_checked_delivery_after_an_unchecked_save_makes_a_fresh_equal_tree_commit`
checks that replacement and its HEAD update.

`checkouts.md`'s later goal start has no known implementation gap. The checkout
world's three goal stories cover r already contained in g, a clean merge
committed before the run and a conflict retained for the run to settle;
each delivery fast-forwards r and leaves g unchanged. They use fake git
and disk, and do not settle the deployment replay gaps below. A goal's
saved start overtaken by a repair remains deferred (`../forge/README.md`, section 11).

- **Not yet:** the whole document: a missing base branch fails the
  preparation; the checkout creates it from the default branch for a
  repository that may be written
  (`temper-worker-domain-checkout/src/hold.rs`, `PrepareStep::Create`).
- **Not yet:** the whole document and `README.md`, 8: git's invocations in contained process trees;
  `temper-worker-protocol-git` runs them as plain child processes.
- **Not yet:** the whole document: a base with heads merged in has no real run on record: `temper-fake-git`
  holds a two-head start's commands and the `merge-in-progress` transcript git's
  refusal of a second merge, and the request `parallel-faults` of 2026-11-06 has
  its proof in a rerun of its replay, which no record shows. It asks whether this
  is the fault of the early start in the request `landing-fixes` of 2026-11-06,
  item 3, an uncommitted merge at `7913a87678`. Both deployments ran before
  `f6b4c408a7` had each merge it commits checked out, so it may be, by reading
  alone: neither was reproduced on a real run, and `landing-fixes` has its proof in
  the next landing plan on a real deployment, which no record shows either.
- **Not yet:** *A merge in progress*: a producer's start that finds its base contained stands
  on `temper-fake-git` alone (`a_resumed_producer_holding_the_tip_pushes_no_merge`,
  `temper/tests/worker/checkout/tests/suite/merges.rs`), with that same proof.
- **Not yet:** the whole document and `README.md`, 9: a parked producer's delivered and saved edits survive a moved landing
  tip, repeated questions and a save refused for markers in the composed system
  world's two `parked` stories. Those use fake git and a fake disk, and their
  first committed edit is a delivery, not a run's own `git commit`. The
  `resume-told` request asks for proof in a real checkout; none is on record,
  nor its next deployment replay. The clean-merge preparation and the conflict
  advice, including the help page and stock charters, have scripted coverage
  alone; whether real producers and their answerers now read the tree rightly
  awaits that replay, as the request's `walk = false` leaves it.

- **Departs:** the whole document and `README.md`, 8: with `TEMPER_INTERNAL_WORKSPACE_REUSE` in the worker's
  environment at its start, a workspace stays as it is between workstreams and
  across a restart; unset, which is the default and the only shipped form,
  `checkouts.md` and `README.md`, section 8 hold exactly. Any value counts as set, and it is read once.
  Set:
  - where `checkouts.md` has a new workstream take the least recently used
    workspace, made again, that workspace keeps its clone when it holds exactly
    the repositories the spec names, by directory and remote, and none starts
    saved. It is fetched and checked out afresh, so an unchanged file keeps its
    mtime. Anything else is made again, as `checkouts.md` has it, a workspace whose
    disk is not known included;
  - where `../../protocol/git.md`, section 2 has a start set `workspaces` aside, it
    stays, and a workspace's first hold looks at each repository the spec names:
    one there, a git repository, with `remote.origin.url` exactly its remote, as
    `git clone` wrote it (`../../protocol/git.status.md`), is treated as cloned. A
    look that finds otherwise makes the workspace again; one that breaks fails
    the prepare as transient, and the next prepare makes it. A look names only
    the spec's repositories, so a directory beside them stays until a make. A
    saved start makes its empty directory as `checkouts.md` has it;
  - where `checkouts.md` does not trust a clone a killed git may have damaged, and
    `README.md`, section 8 has what a worker left behind fail no prepare, a look cannot tell
    a damaged clone from a whole one. The operation that breaks on it fails that
    prepare as transient and spoils it, and the next prepare makes it again, as
    `temper/tests/worker/checkout/tests/suite/reuse.rs` shows with a stale
    `index.lock`. Damage no operation breaks on stays in use;
  - a kept repository's alternates name the store it was cloned with, by
    token, which is its remote's (`checkouts.md`), so after a restart they
    name the store `stores` kept;
  - a workspace's path stays `<root>/workspaces/<token>/<repository>`, its
    token its place in the order workspaces are made, since none is retired: the
    first `0`, the second `4294967296`. A restart does not bring back which
    workstream held which workspace, so after one a path holds whichever
    workstream's first hold came in that place.

  Because a deployment that runs one trusted person's work on a machine it
  builds on needs a workspace to act as a long-lived build directory, so that
  new work rebuilds only what changed (the user, 2026-11-05). It is off by
  default because a run can write its `.git`, and the worker's next git command
  obeys what that says, now in another workstream, with that workstream's
  identity on a fetch or a push. The user's request and
  `../../draft/dogfood-hacks.md`, 2.1 name its hooks, which `../../protocol/git.md`,
  section 3's settings hold off; those settings do not hold off a filter driver
  or a URL rewrite (`../../protocol/git.md`, 10). That is acceptable only for one
  trusted person.

Under those settings real git ran no `post-checkout` hook and did run a
filter driver (the `worker-slots` cut, 2026-11-05).
