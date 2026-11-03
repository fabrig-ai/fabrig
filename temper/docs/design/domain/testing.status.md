# testing: status

2026-11-03. Where temper's code lags or departs from [testing.md](testing.md).

- **Not yet:** 2.4: GitHub's real-loop stories 3, 4, 5, 7 and 8; 1, 2 and 6
  run (`tests/real-loop/tests/suite/github_land.rs`, `github.rs`). 2.4, 7: the twins' documents in `temper-playground`; `tests/playground`. 2.6:
  `a_default_box_with_a_sign_in_registration_starts_without_dns` is ignored:
  a start without DNS fails while the sign-in's endpoints resolve at start.
  2.6, 5.3: the cluster's 4.6 process story is ignored: a retry placed on a
  stopping worker's still-linked channel is journaled as an extra refused
  attempt, until jig takes a draining worker out of placement. 2.6: the
  playground's stories, in and out and a reset the box's death cut, and
  GitHub gone on after a restart, as `../protocol/setup.status.md` records
  them.
  2.7: a live
  story ending at its task's first terminal failure. 2.7: the live GitHub
  story connects the probe App without `--read-only`, so its record's
  `read_only` is false (`crates/temper/tests/support/live_github.rs`).
  2.7: the owned-forge
  landing pair has not passed after a landing (the runner is fab's, in the
  agents repository): neither story has passed live on its own store, the
  Claude Code one on `claude-haiku-5-5` included; the pair's `#[ignore]`
  reasons still say "a person's sitting" and "owner-run only"
  (`crates/temper/tests/live.rs`). 4.3: the shell subset;
  skein's fake machine runs no `sh` and no `rg`. 6.4: of
  temper's kinds' stories, the forge's world's (`forge.md`'s status,
  18); the domain world's project's rule refused to another project's
  owner, a project made after the first start covered by the
  deployment's rule, the two requirements removed from the rules and the
  resume fixture restarting with changed values
  (`tests/engine/domain/tests/suite/settings/kinds.rs` tells the rest);
  the fuzzy sweep's changes of landing rules
  (`tests/engine/forge`'s `suite::fuzzy::simulation` makes none); and
  the real loop's smoke of a gate added to a project's rule from the
  client. 6.4: the configuring agent's stories, since charter 3 is not
  built (`agent.status.md`); the client's How work runs, a proposal's
  card and its review, in their step and view tests and joined with
  the engine, since the pages are not built
  (`../client/README.status.md`). 6: a state
  digest for replay (`testing-strategy.md`, section 6); the walking
  world compares its formatted state
  (`tests/engine/domain/src/walking.rs`, `run_replayed`).
- **Departs:** 5.3: the case library's purpose is retired: the reference
  case it was written from is replaced (`../draft/reference-case.md`), and
  its code awaits removal with its design.
