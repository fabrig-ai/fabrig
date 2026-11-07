# ui-live-cheap-2: the plan

The request (`inputs/request.md`) is one task, and this plan makes it one: the live
Claude Code owned-forge landing story's settings are refused before they reach Claude
(`settings apply: This model cannot be chosen: claude-haiku-5-5`, exit 2), and after
this plan the sitting makes them in two applies, a person's order, which a focused test
applies through ctl to the joined world holding the Claude deployment's catalogue.

## Parts and tasks

- **live-claude-settings** (`temper/docs/design/domain/testing.md`, 2.7), slice 1.
  - `two-applies` (needs none): a red-first focused test in temper's `suite`
    (`forge_land.rs`) applies `live_settings`' real output through ctl on the joined
    client world (`temper-client-world`'s `Joint`, its engine holding the Claude
    deployment's stock catalogue, endpoints, providers and charters 1 and 2), and shows
    the live refusal and its why; then `live_settings` returns the sitting's changes in
    order (the model item alone, `live-model`; then the charters, `live-settings`),
    the judge applies each against the revision before it, and the test turns green.
    The table test of `live_settings` takes the new return. The live story is
    compiled, not run.

Critical path: `two-applies`, alone.

## What was decided

- **The refusal is the clients' component's, and it's designed.** jig's
  `jig-protocol-engine-clients` checks every model a settings change's charters name
  against the catalogue as the component holds it, before the core sees the change
  (`src/request.rs`, its call to `translate::llm::refused`; `src/translate/llm.rs`,
  `settings_items` and `settings_model`). A model the same change sets isn't in that
  catalogue yet, so the change is refused, the why `unconfigured` (the task's red
  confirms it through `--json`). `ui-live-cheap` put the model and the charters in
  one change on the strength of jig's core catalogue tests
  (`jig/tests/core/tests/suite/models.rs`), which step the core below the component
  and so never meet this check.
- **So the fixture takes a person's path, and jig stays as it is.** A person adds a
  model on Settings' Models, and chooses it for a charter after, in two acts; a
  change of the catalogue tells the clients' component before its commit (jig's
  `domain/engine.md`, 17.11, *Who hears*), so the second apply's check finds the
  model. That isn't the core refusing a model a person added; it's one change doing
  two acts' work. The request's fallback, stock row 2 (`claude-sonnet-5-5`), is not
  taken: nothing says the core refuses a Claude Code account a model outside stock.
  The spec pre-approves a jig fix at its lowest place only if the charters' apply is
  still refused once the catalogue holds the model.
- **The model item stays a settings item, not `ChangeModel`.** temper ctl has no verb
  for the Models page's *Add a model by name*, and `ChangeModel` gives an added model
  no efforts, which would drop `--effort low`; the item keeps stock row 3's figures
  and efforts with `low` its default, as `ui-live-cheap` decided.
- **The test world is the joined client world in temper's `suite`** (focused): only
  there can a test call the fixture's own `live_settings` (temper's test support) and
  reach both the component and the core in memory. temper's `suite` already builds the
  Claude deployment and already uses `temper-client-world`. No process starts beyond
  the `temper init` the suite's helper already runs. temper-service's own tests step
  the root below the component, so they can't show this refusal.
- **One new test, not a row:** no existing table applies a sitting's settings to an
  engine. The pure table of `live_settings` keeps its three rows, each taking the new
  return.
- **Design:** `testing.md`, 2.7 now says two applies and why, and names the focused
  test; `testing.status.md`'s 2.7 entry records the gap, and the task replaces that
  sentence word for word when it lands.

## Overlaps with other drafts

- `tui-approval/temper-story` (merged into its integration) changed
  `temper/tests/client/src/front.rs`. This task may make that file's
  `fn engine_limits` `pub` (one word), and only if its world takes those limits; no
  other line. Otherwise no file this plan touches is another draft's.
