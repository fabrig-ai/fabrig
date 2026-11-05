# worker: status

2026-11-05. Where temper's code lags or departs from [worker.md](worker.md).
Section 4.3's failed and refused answer diagnostics reach serve's stderr and the box's
`serve.log`; that operator-output path has no gap.

- **Not yet:** 4.3: a held task names no run's failure, which the user's
  request of 2026-11-05 asked for if cheap: it needs a stored-format
  change, so it is section 12's question, and `temper help server` says
  where the reason is read. 4.4: the prepare and key kinds: neither kind's entry is
  built, so the worker's hello declares the merge kind alone
  (`temper/crates/temper-service/src/worker.rs`), and with prepare the
  `Rewrite` delivery (`../protocol/git.md`, 4.10). 5: a missing base branch fails the
  preparation; the checkout creates it from the default branch for a
  repository that may be written
  (`temper-worker-domain-checkout/src/hold.rs`, `PrepareStep::Create`).
  5, 9: git's invocations in contained process trees;
  `temper-worker-protocol-git` runs them as plain child processes.
- **Departs:** 5, 9: with `TEMPER_WORKSPACE_REUSE` in the worker's
  environment at its start, a workspace stays as it is between workstreams and
  across a restart; unset, which is the default and the only shipped form,
  section 5 holds exactly. Any value counts as set, and it is read once. Set:
  - where section 5 has a new workstream take the least recently used
    workspace, made again, that workspace keeps its clone when it holds exactly
    the repositories the spec names, by directory and remote, and none starts
    saved. It is fetched and checked out afresh, so an unchanged file keeps its
    mtime. Anything else is made again, as section 5 has it, a workspace whose
    disk is not known included;
  - where section 9 has a start set `workspaces` aside, it stays, and a
    workspace's first hold looks at each repository the spec names: one there,
    a git repository, with `remote.origin.url` exactly its remote, as `git
    clone` wrote it (`../protocol/git.status.md`), is treated as cloned, and any
    other answer makes the workspace again. A look names only the spec's
    repositories, so a directory beside them stays until a make. A saved start
    makes its empty directory as section 5 has it;
  - a clone a killed git left damaged heals as section 5 says: the operation
    that breaks on it spoils it, and the next prepare makes it again;
  - a workspace's path stays `<root>/workspaces/<token>/<repository>`, its
    token its place in the order workspaces are made, since none is retired. A
    restart does not bring back which workstream held which workspace.

  Because a deployment that runs one trusted person's work on a machine it
  builds on needs a workspace to act as a long-lived build directory, so that
  new work rebuilds only what changed (the user, 2026-11-05). It is off by
  default because a run can write its `.git`, and the worker's next git command
  obeys what that says, now in another workstream, with that workstream's
  identity on a fetch. `../protocol/git.md`, section 3's settings hold a hook
  off but not a filter driver or a URL rewrite (`../protocol/git.md`, 10). That
  is acceptable only for one trusted person.
