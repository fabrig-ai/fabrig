# configuring-p3: plan

Phase P3 of `temper/docs/design/draft/configuring.execution.md`: the configuring agent, built
on main (P1, P2 and the reference's M1 to M10 landed). Released in batches; this file is
brought up to date with each.

## Parts

| Part | Slice | Design | Tasks |
|---|---|---|---|
| `charter-tools` | 1 | jig's `domain/engine.md` 7.1; temper's `domain/agent.md` 4.3 | `tools-surface` (batch 1); the formats and the offer follow |

Still to come (batch 2 on): the brief's room checked at a start and at a change's entrances
(`domain/agent.md` 4.3); charter 3 and template 3 in the stock with the settings family in
policy 1 and the rules (`protocol/configuration.md` 13.4, `domain/agent.md` 8.1); the agent's
reads and proposals of every settings-file kind; its stories (`domain/testing.md` 6.4); the
reference pages `limits`, `configuration` and `configuration-engine`
(`protocol/documentation.md` 8); the walk.

## The design this cut wrote (batch 1)

- temper's `domain/agent.md`, 8.1 in its own words (no rule taken from the draft by citing
  it), its tools and the kinds it reads and proposes; section 8's table row without
  `withdraw`, a decide-family tool policy 1 does not give.
- temper's `protocol/configuration.md`, 13.4: charter 3's and template 3's rows in their own
  words; each stock charter's `tools` as numbers; the configuring agent as a setting.
- Statuses: the draft's `configuring.status.md`, its first Design lags entry closed; jig's
  `engine.status.md`, a 7.1 Not yet for a charter's `tools`.

## Decided along the way

- **A charter's `tools` is a member of `RunPolicy`** (contracts C1), so it travels wherever
  the charter's policy does.
- **temper gates the settings family alone by it** (`domain/agent.md` 4.3); the other
  families stay declared as today (`protocol/configuration.status.md` keeps its 13.4 entry for
  them).

## Critical path (as it stands)

`tools-surface` (60) -> the formats (store, worker side, client wire) and the offer -> the
stock's charter 3 -> the agent's stories -> the walk.

## Overlaps with other plans

- `tools-surface`'s compiler-forced lines in `temper-client-tools/src/jig/settings_adapter.rs`
  and its tests: public-reading-build's `public-ctl` owns `temper-client-tools/**`. One line
  per site.
- jig's `docs/design/domain/engine.status.md`: public-reading-build's `public-status` edits
  it; this cut adds one 7.1 entry. A catch-up conflict is a status merge, both kept.
