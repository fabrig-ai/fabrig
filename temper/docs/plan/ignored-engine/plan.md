# ignored-engine: the plan

Twelve ignored temper tests come back into the focused suite that
selects them (`inputs/request.md`; `skein/docs/foundation/testing-strategy.md`,
8: an ignored test is a finding its owner runs until it's fixed). All
twelve are in the focused suite (`.config/nextest.toml`, `[profile.default]`).
One task per group of the request, effort medium. Each task un-ignores
its tests first, then fixes each failure by its cause: the product
against its design, or the world, joint or fake that lacks a capability
(`notes/common.md`). Read-only surveys of three groups, run beside the
call, settled the causes below by reading; no test was run by this cut.

No design section was written: the sections the tests cite stand. A
task leaving a test ignored for an unbuilt feature writes
`notes/<task>-ignored.md`, as the request asks.

## Parts and tasks

| Part | Task | Tests | Where | Needs | Minutes |
|---|---|---|---|---|---|
| `restore` | `restore-stories` | `settings/kinds.rs`'s three restore stories; `settings_tools.rs`'s each-kind story | engine-domain and client worlds; product only where a story shows it wrong; `configuration.status.md` and `tools.status.md`'s clauses | | 60 |
| `change-panel` | `change-panel` | `change_panel.rs`'s three | the story file; temper-engine-domain-forge (`Event::Priority`); temper-engine-domain (`project_goal`, `admission.rs`) | | 120 |
| `forge-routes` | `forge-routes` | `goal_changes.rs`, `goal_landing_held.rs`, `landing.rs`, one each | the three story files | | 70 |
| `github-joint` | `github-joint` | `github_land.rs`'s two | `temper/tests/engine/protocol-forge/src/protocol.rs` and the story file | | 75 |

## Decisions

- **Restore:** `Restore` admission and `ForgetReplaced` are on main
  (`jig/docs/design/domain/engine.status.md`, 17.10; temper's
  `route/kinds.rs` answers `ForgetReplaced`). The task un-ignores and
  repairs what the run shows. jig's own eleven ignored restore stories
  stay as they are (the request touches no other ignored test), and
  jig's status entry for them stays.
- **Change panel: the product's events, not a replacing `Change`.** The
  request reads the gap as the world's `Change` step. The survey found
  the product refusing a second `Change` for a task it holds
  (`register_change`), rightly: `Change` registers an activated change,
  and the root sends it only for a task with no row. Making it replace
  rows would give the product a path no design asks. So two stories
  drive the events that do save an existing record (`Relayed`; a hold
  by `RelayRefused` and its `ReleaseChange`). The third needs a
  priority move, which the product lacks: forge.md 9 has a change's
  priority move with its goal's, while a change keeps the priority its
  call gave (parameter 5) and `Domain::priority` has no caller. The
  task builds it: a forge `Event::Priority` dispatched to
  `Domain::priority`, sent by the root's `project_goal` from the goal's
  feed. If the feed does not reach the goal's changes, the root's half
  falls back to a **Not yet** entry in `forge.status.md`, pre-approved.
- **Forge routes: none waits on product work, by reading.**
  `goal_changes.rs` asserts a two-parent merge commit; no design asks it.
  The product merges by the repository's style (forge.md 4.4), the
  fake's style is a squash (skein's `fake-forge.md`, 2.2), and the
  ignore's "C5" is the reference case's change of that name, whose merge
  job merges g into its own branch. The assertion is rewritten to a
  squash at the producer's head; the fake is not changed, since every
  forge world shares its merge. `goal_landing_held.rs` closes before the
  forge hears main move after its landing; the restart's fresh read then
  publishes landing news, which takes a `Deployment.messages` number.
  The story hears main's move before closing, as its sibling
  `goal_landing.rs` does; the oracle is not loosened. `landing.rs`'s
  story waited on jig's kept start of a declared procedure with a
  pending dependency, which is on main (connectors.md 6.1, no gap).
- **GitHub joint: two world capabilities, no skein change.** skein's
  fake already enforces protection at merge and answers GitHub's 405,
  which the product maps to `Protected`. The world gains `protect`
  (an admin's `SetProtection` outside the component's requests) and
  `cut_after_next_merge` (a run stopped once the fake has applied a
  merge, before its answer reaches the component). Each story also
  asserts that its capability acted.

## Critical path

None: the four tasks are independent and released together. The
longest is `change-panel`, about 120 minutes.

## Overlaps with other drafts

- `tui-box-front/world-front` changes `temper/tests/client/src/`'s
  `fixtures.rs`, `component.rs`, `tui.rs` and `front.rs`;
  `restore-stories` owns none of them and only adds lines if it must.
- `look-round-24` (`asked-sentence`, `ci-workflows`, `rule-view`) edits
  `temper/docs/design/domain/forge.status.md`; `change-panel` writes
  there only in its fallback, as a bullet of its own at the end.
- `restore-stories`, `change-panel` and `forge-routes` all live in
  `temper-engine-domain-world`, in different story files; the world's
  library (`temper/tests/engine/domain/src/`) gains added helpers only.
