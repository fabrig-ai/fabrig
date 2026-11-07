# testing: status

2026-11-07. Where temper's code lags or departs from [testing.md](testing.md).

Sections 2.5, 6 and 6.1 have the profiled fuzzy repairs: engine walking,
terminal-cut, escalation and role sweeps compare exact traces and
complete frozen-state digests each iteration, with full text on a
mismatch and at each seed's end. Reachable state types derive `Hash`;
role observations consume new history, and teardown transfers the store
to its cold oracle. One-process setups reuse fixed codec/default/settings
construction and refresh observations at history or durable changes,
including the initial empty observation. The public-record sweep keeps
all 2,000 seeds and 24,000 drawn checks, with six fixed checks once.
Shared list append bookkeeping and fixed-width generated measurement
are cheaper. No test, draw, crash, invariant or ending is removed or
moved. Remaining mechanisms and reliability investigations are deferred
in section 11; the measured suite overage remains below.

Section 5.2's conformance sweeps fork every live commit from one baseline,
retaining cold restart and independent readback. Section 5.3's retained
stories and lower holders agree with its table; further costs and the
survivors' placement decision are carried in section 11.

Section 2.6's two one-link assertions count complete link lines and carry
the box's stderr with every `secret=` line removed. The focused helper
table covers two or no links, non-link and unfinished lines, filtered
diagnostics and absence of secret bytes from the panic. The original
second link's cause is unproved; investigation of a new red is deferred
in section 11. This retirement supplies no process-tier runtime result.

Sections 2.1, 2.3 and 6.1 have the cheaper service fixtures, consuming
bootstrap-store transfer, restart configuration construction, phrasebook
loading and lookup, root-only client setups, setup settling and checked
startup-machine reuse. The `limits-fixtures` figures at `7a5cd795d9`
record all request rows, 68 passing single-test traces, and unchanged
suite membership, assertions, build settings and budgets for that plan.
The sampled setup median falls 59.63%, the engine question/restart median
19.62%, the joined-client dismissal median 60.42%, and the view median
20.48% in instructions. Remaining costs and the work they need are
deferred in section 11; these samples establish no whole-suite CPU total.

Sections 2.2, 2.6 and 6.1 have the `limits-heavy` row repairs: cheaper
fake preparation, unique-position page ordering, projection updates,
protocol readiness/deadline reuse, gallery inputs and checks, launcher
snapshots and the four single-test fixtures/byte constructions. The
permanent account is `temper/docs/development/limits-heavy.md`; its
selected instruction counts, including increases and unknowns, establish
no whole-suite CPU total. Its listed request rows total 69.7 CPU s against
the request's about 89; the unassigned 19.3 CPU s remains unexplained.
Further mechanisms, including causes above that plan's stopping threshold,
are deferred in section 11, with all bounds, orders and checks retained.

- **Not yet:** 2.6: the hangup/rebind and Linux killed-box stories in
  `crates/temper/tests/suite/box_lifetime.rs` are registered in the
  process binary and compiled, but have no runtime result in that tier
  from `limits-heavy`. Both passed in focused tests before moving whole.
  Their original CPU medians transfer about 15.9 CPU s to a tier whose
  current margin is unknown; neither a current tier total nor its fit
  within 360 CPU s is established by the instruction account.

- **Departs:** 6.1: temper's measured focused request-world sum is
  81.892959 CPU s against 30, and the real-loop focused median
  is 31.769481 CPU s against its standing 15 s ceiling. The whole-temper
  fuzzy median is 125.579032 CPU s against 120, in three warm serial runs
  including the driver (119.288299–127.664356 CPU s).
  All 81 tests passed in each run, but two runs and the median exceed
  120; the request's roughly 15 CPU s reduction and reliable headroom
  are not achieved. The gate's own per-test CPU result is unknown from
  these serial samples, whose driver and feature unions differ from
  the request's gate baseline.
  [`limits-fuzzy.md`](../../development/limits-fuzzy.md) records these runs,
  their accounting limits, retained causes and next decisions.
  The focused measurements, their load and accounting limits, retained
  causes and next decisions are in
  [`limits-worlds.md`](../../development/limits-worlds.md). The whole-temper
  focused suite total remains unknown; package sums include warm nextest
  overhead.
  No budget or ceiling has been raised.

- **Departs:** 5.3 and 6.1: eighteen named real-loop stories remain in the fuzzy
  suite: sixteen scripted stories (fourteen reference-case stories,
  kept-delegate landing and the program backend) and the B0/B1 recorded
  transcript twins.
  They remain there because the request for `limits-worlds` kept surviving
  stories in place. Being past a guide is not a reason to
  choose that suite (`skein/docs/foundation/testing-strategy.md`, 8.1).
  [`limits-worlds.md`](../../development/limits-worlds.md) gives every
  survivor's unique coverage, CPU and profiled instructions, and names
  the closest lower-story candidates. Their placement is the user's to
  decide; no exception or cost-driven move resolves this departure.

Section 2.4's real-loop goal-delivery story is present, alongside the six
engine domain stories and the checkout world's later-run cases. It uses
real smith and a worker with scripted model replies, fake git and the
fake forge; no real model or real forge is proved by it. Section 6.5.1's
reference obligations retain their current charter words; folding in the
goal-as-designer workflow remains in section 11.

Sections 2.4 and 2.6 have the reusable-port chooser and its callers:
socket-claimed blocks below Linux's configured ephemeral range, the
retained deployment's listeners, the browser's reserved sockets and the
fake box's nine-port override. Focused tests cover disjoint choosers,
released ports listening again and retained sandbox ports. The supplied
inventory establishes no slow-tier restart outcome for `process-ports`;
that proof remains with the after-landing runs (11).

- **Departs:** 2.6: `temper-fixtures/src/ports.rs`, `ephemeral_first`,
  uses 49152 when `/proc/sys/net/ipv4/ip_local_port_range` is absent,
  rather than reading macOS's configured range. The plan chose its
  default because macOS has no Linux range file. A lower configured
  boundary can overlap the chosen ports; reading it remains later (11).

Sections 6.3 and 6.4 have the owned-Forgejo box's linked-person stories:
inventory and owner-role acceptance at one approval, refusal at two, clearing
on a second owner after restart, then a change waiting under Needs you,
another project's party reading none, and the person's review landing it.
The joined client story draws the card and files-page link and checks its
removal after the review lands the change. The connector removes review
facts on `Committed` for `Write::Merge`. The no-link cases keep the stock
permission mapping, owner-role refusal after admin adoption and fresh
maintainer-role acceptance. These are scripted worlds on fake Forgejo.

- **Not yet:** 6.4: the box's approval-identity and approval-wait stories
  have no GitHub row: their fixtures offer Forgejo alone. Setup and
  connector tables cover GitHub's identity evidence and link decisions,
  but not the composed story the table asks for. A real human GitHub
  review counted (the draft's section 10) has no new live story or supplied
  result; it needs the user's review credential. These proofs remain
  section 11's later work, along with the real owned-Forgejo walk and
  rendered comparison. The joint-world card replaces the browser step
  by the plan's explicit decision.
- **Not yet:** 6.3: the joined story checks the card gone after landing,
  without a separate observation between merge admission and landing.
  It therefore does not independently hold that timing in the joint world,
  although the connector drops the review facts at admission (6.3's
  *A change waiting on a review on the forge*).

Sections 6.4 and 6.5.3 have no gap in scripted note-backed proposals and single-revision collector evidence.

Section 2.8 lists the killed-box lifetime test as Linux-only, matching its
parent-death mechanism. The combined default-runner browser and terminal
proof is not established by the supplied inventory (`../protocol/setup.status.md`,
7.1, 7.2 and 13.4); the existing real-box sign-in story still awaits its
after-landing execution (`../protocol/sign-in.status.md`, 9.9).

- **Not yet:** 2.6: the supplied inventory has no result for any of
  `ui-restart-red`'s five repaired process stories
  (`../protocol/setup.status.md`, 13.4). The stock-purse regression and
  late-handover tests are present, and the started-cut fixture uses
  init's no-sign-in configuration while it holds the engine; none is a
  recorded pass of the five stories.

- **Not yet:** 2.6: no passing whole run is recorded for the three-test
  profile with the four fixed-callback walks serialized. Its proof belongs
  to the next after-landing run on main (11), per the request
  `process-threads` of 2026-11-06; the profile and its callback group are
  configured in `.config/nextest.toml`.
  2.6: the tier's CPU against its 360 s: of the seven whole runs in
  `test-speed`'s figures at `b23901eab7`, at differing loads, four read
  over it (367.8 to 433.7 s) and three within (287.1 to 349.7 s), and
  they hold no run with that plan's parts together. The budget is the user's.
  2.6: Goal and Away keep the shipped 5 s poll, where the request `test-speed` has them
  wake on an event or take the status walks' paces: no poll is what they wait
  for, and a trial of short paces on Away saved less wall than it cost CPU. The
  fake-box walks retain the stock charter's 10 s wait on their single worker
  (`agent.md`, 4.6); Away also waits through two 5 s Undo holds
  (`../client/README.md`, 3.5), and the personal plan's walk has its brief wait
  for the chat resumed on landing to release that worker, after its deliberate
  4 s sitting gap. A person on one worker meets the charter's wait and the
  holds too, for reasons that still hold, and these waits have no cheap repair
  without changing what the walks show. Home's status walk still costs about
  19 s, including that 10 s wait and four client visits; the personal plan's
  also provisions and restarts an owned Forgejo and runner. Their shared forge
  paces are 1 s with 100 ms hints; the measured ranges, remaining costs and
  rejected pace change are in the plan's figures on the walks' waits,
  at `289ef784ef`.
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
  2.6: Forgejo signing in through temper: the story's connect gives no
  `--web`, its `redirect_uri` being at `git`, where `temper help server`'s
  gives one, and no test takes a connect's `--web` through to the page's
  meta: a `web` apart from `git` is shown on a kept record alone
  (`temper-service/src/tests/forge_sign_in.rs`). The bullet's figures are
  those from before its bundle and its connect; whether they still hold
  is unknown. No test reaches a host's units, a proxy or a
  redeploy, as the user's request of 2026-11-05 says, and its plan ran no
  walk, so the page's steps on a host stand on these stories alone.
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
  (`crates/temper/tests/live.rs`).
  Its `owned_forge_landing` also passes seven setup calls on a fresh
  first run and four otherwise to `assert_forgejo_setup`, which expects
  six and three (`crates/temper/tests/support/forge_land.rs`). Those
  lengths disagree before the token-file assertion. The late-handover
  cause reaches both stories through that shared assertion, but its
  repair establishes no live pass and does not fix this oracle mismatch.
  2.7: the held record's guard is shown on the fake issuer and on stubs
  alone (`crates/temper/tests/process.rs`,
  `a_story_panicking_after_its_box_stopped_leaves_the_held_record_at_its_path`,
  and `crates/temper/tests/suite/startup.rs`). Its return with no box up
  holds through red-26's repair (660556eab1,
  `a_failed_terminal_return_allows_recovery_without_reopening_import`),
  and that red closed recovered. Nothing offline runs its lines in
  `crates/temper/tests/live.rs`, and whether a live store's box, with its
  owned Forgejo, answers the guard within its 240 s is unknown.
  4.3: the shell subset;
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
  starts from `Configure` and `Settings`' "Configure temper" finds it,
  but it proposes, is accepted, waits for an owner and goes stale
  through ctl alone (`tests/client`'s `configuring_chat.rs`).
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
  2.7: the held record's guard takes the goal of the live-login-guard
  draft of 11-04, every exit path exports the record, and departs from
  its mechanism, by the designer's decision: it stands before the import,
  where the draft has it right after, because the import is inside the
  first sitting's referee and one whose reply was lost still owes the
  record; it starts a box of its own where none is up, where the draft
  has its `Drop` run with the box still up, because a panic through a
  sitting kills the box; and a `stat` of the record's path disarms it,
  where the draft has an explicit export, because a record back at its
  path owes nothing whichever return brought it.
- **Not yet:** 6.3: the TUI's host does not run in the joint world: its front carries no
  linked root's exchanges, and its component makes no printed link (`tests/client/src/component.rs`).
