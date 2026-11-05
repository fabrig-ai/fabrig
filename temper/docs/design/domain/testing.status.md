# testing: status

2026-11-05. Where temper's code lags or departs from [testing.md](testing.md).

- **Not yet:** 2.4: GitHub's real-loop stories 3, 4, 5, 7 and 8; 1, 2 and 6
  run (`tests/real-loop/tests/suite/github_land.rs`, `github.rs`). 2.4:
  the twins' documents among `temper-playground`'s stories: B0's and
  B1's are still the real loop's own fixtures
  (`tests/real-loop/tests/fixtures/b0`, `b1`). 2.4: B2's twin
  (`../../development/acceptance.md`, section 8). 2.6:
  `a_default_box_with_a_sign_in_registration_starts_without_dns` is ignored:
  a start without DNS fails while the sign-in's endpoints resolve at start.
  2.6, 5.3: the cluster's 4.6 process story is ignored: a retry placed on a
  stopping worker's still-linked channel is journaled as an extra refused
  attempt, until jig takes a draining worker out of placement. 2.6: the
  playground's stories, in and out and a reset the box's death cut, and
  GitHub gone on after a restart, as `../protocol/setup.status.md` records
  them.
  2.7: the ladder's live stories but B0's: `scripts/proof-live.sh` takes
  `rung-b0` alone, on ChatGPT with an owned Forgejo, and
  `crates/temper/tests/live.rs` holds no other rung.
  2.7: neither GitHub proof has shown it holds on `fabrig-ai` alone:
  `scripts/proof-live-github.sh` and `proof-live-github-managed.sh live`
  stay for the next live walk, the second with `TEMPER_MANAGED_PERSON`
  set. That sitting's relay admits GraphQL's `VIEWER` alone
  (`crates/temper/tests/support/relay_managed.rs`) where the story's
  admits `PULL` on the probe too (`crates/temper/tests/support/relay.rs`),
  and an open pull on an organization's repository reads through `PULL`
  (`skein/crates/skein-forge/src/github/plans/reads/pulls.rs`); whether
  a live sitting's engine reads the pull it opens is unknown.
  2.7: a live
  story ending at its task's first terminal failure. 2.7: the live GitHub
  story connects the probe App without `--read-only`, so its record's
  `read_only` is false (`crates/temper/tests/support/live_github.rs`).
  2.7: the owned-forge
  landing pair has not passed in the after-landing tier: neither story
  has passed live on its own store, the
  Claude Code one on `claude-haiku-5-5` included; the pair's `#[ignore]`
  reasons still say "a person's sitting" and "owner-run only"
  (`crates/temper/tests/live.rs`). 2.8: the one story has not run green
  on macOS yet: no recorded run has passed the box's check of its held
  listeners (`skein/docs/design/shell.md`, 6.6), and that the bottles'
  first lines name the pins stands unproved. No builder runs macOS, and
  the workflow's runs show them. 4.3: the shell subset;
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
  client. 6.4: the client's How work runs, with its "Configure
  temper", a proposal's card and its review, in their step and view
  tests and joined with the engine, since the pages are not built
  (`../client/README.status.md`); the joined world's configuring chat
  starts, proposes, is accepted, waits for an owner and goes stale
  through ctl alone (`tests/client`'s `configuring_chat.rs`). 6: a state
  digest for replay (`testing-strategy.md`, section 6); the walking
  world compares its formatted state
  (`tests/engine/domain/src/walking.rs`, `run_replayed`).
- **Departs:** 5.3: the case library's purpose is retired: the reference
  case it was written from is replaced (`../draft/reference-case.md`), and
  its code awaits removal with its design.
  4.3: `temper-fake-git/src/exchange.rs`, `decode`, and its
  `src/tests/net.rs`, `decode`, each drive a tokenizer over one whole body
  in a `Held` by hand, with their own step bounds, where
  `skein/docs/design/json.md`, 5.3 gives one whole buffer to `parse` or a
  `Feed`, and the user's request of 11-04 asked that no crate under
  `temper/crates` do so. No reason is recorded, and no debt item holds
  them.
  2.7: `scripts/proof-live.sh`'s `TEMPER_LIVE_STORE` and
  `TEMPER_LIVE_KEEP`, `proof-live-github.sh`'s fixtures and
  `proof-live-github-managed.sh`'s target and evidence default under
  `$HOME/src/rust/tmp/`, a scratch layout of whoever runs fabrig's
  development; 2.7 names the first, and
  `crates/temper/tests/suite/forge_land.rs` holds it. The user's request
  of 11-04 makes `AGENTS.md`, 1 absolute and says fabrig names nothing
  outside plan directories, but `tests/boundary/tests/outward.rs`
  matches the `~/` form alone, so these pass it. They stay because the
  scripts' behaviour held but for their build wrapper, and no step of
  that request moves them.
  2.7: `proof-live-github-managed.py`'s `ssh -L` mark names a machine
  alias of whoever runs fabrig's development, and the outward check
  matches that name only with digits after it. The same request left it
  in place.
  2.7: the managed sitting's `fake` and `fake-wide` runs and `startup.rs`'s
  `the_live_github_configuration_passes_startup_admission` keep
  `fabrig-ai`, where the user's request of 11-04 gives tests on the fake
  forge a neutral owner, because the first are the live sitting's dry
  run, whose owner is a fixed name and never an input, and the second's
  shaper renders the template's `acme` to the owner, so `acme` would
  prove nothing. The relay's own tests take `acme` and `alice`.
  2.7: Offline startup admission: no shared helper clears goal projection
  grants. `crates/temper/tests/support/live_github.rs`'s
  `configure_engine` clears none, and `github_land.rs`'s, a separate
  function, removes the charters, templates, policies and rules for
  stock policy 1. No reason is recorded.
