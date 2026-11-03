# configuring-p3: plan

Phase P3 of `temper/docs/design/draft/configuring.execution.md`: the configuring agent, built
on main (P1, P2 and the reference's M1 to M10 landed). Released in batches; this file is
brought up to date with each.

## Parts

| Part | Slice | Design | Tasks |
|---|---|---|---|
| `charter-tools` | 1 | jig's `domain/engine.md` 7.1; temper's `domain/agent.md` 4.3 | `tools-surface` (batch 1); `tools-offered`, `tools-store`, `tools-client` (batch 2) |
| `room` | 2 | temper's `domain/agent.md` 4.3 | `room-table`, `room-checks` (batch 3) |
| `configuring-agent` | 3, **the walk** (`walk = true`) | temper's `domain/agent.md` 8.1; `protocol/configuration.md` 13.4 | `agent-kinds`, `stock-configuring` (batch 3); the stories follow |

Still to come: the agent's stories (`domain/testing.md` 6.4) in the engine's domain world and
through ctl in the joined client world; the help pages that describe the stock; the reference
pages `limits`, `limits-engine`, `configuration` and `configuration-engine`
(`protocol/documentation.md` 8).

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

## Batches

1. `tools-surface` (interface, 60): `RunPolicy.tools`, every literal and decoder filling
   `EVERY_FAMILY`; no byte moves.
2. `tools-offered` (45; needs `tools-surface`): temper-engine-smith offers the settings
   family only where the charter's mask and the authority hold it; `tools-store` (60; needs
   `tools-surface`): settings-v1's `RunPolicy` keeps it, in place; `tools-client` (150; needs
   `tools-surface`, `tools-store`): client-v19's charter, jig's client crates, temper's
   settings form, `FIELDS` and the `settings-fields` page; it closes jig's 7.1 status entry,
   which is why it waits for `tools-store`.

3. `agent-kinds` (120; needs `tools-offered`): a run's `settings` and `propose_settings` name
   every settings-file kind, a read of temper's kinds answered with the forge's value;
   `stock-configuring` (180; needs `tools-offered`): charter 3, template 3, the settings family
   in policy 1 and the rules, each stock charter's `tools`, and the readers of a five-charter
   stock; `room-table` (interface, 90; needs `tools-surface`, `agent-kinds`): C2's `HostPart`,
   built by temper-engine-smith, held to smith's measure; `room-checks` (150; needs
   `room-table`, `stock-configuring`, `agent-kinds`): the start's and the three entrances'
   refusals.

The surveys found that neither the worker wire nor the tasks family carries a `RunPolicy`:
the engine picks a run's host tools before it encodes smith's charter
(`temper-service/src/assignment.rs`), and a task keeps its charter's number and revision.

## Critical path (as it stands)

`tools-surface` (60) -> `tools-offered` (45) -> `agent-kinds` (120) -> `room-table` (90) ->
`room-checks` (150): about 465 implementer-minutes. `stock-configuring` (180) ends at about
285, the agent's stories after it and `agent-kinds`; `tools-store` -> `tools-client` (210)
runs beside them.

- **The first web chat becomes the configuring chat** once the stock offers template 3
  (`temper-client-domain`'s `first_chat`, written for it). The playground keeps its own:
  `stock-configuring` leaves charter 3 and template 3 out of the defaults it builds from the
  stock, since its scripted model plays the stock chat.
- **No `withdraw`, a project's policy not read:** the agent proposes afresh rather than
  withdrawing (decide family, not policy 1's); a run's read of a project's own policy stays a
  status entry (`domain/agent.status.md`), jig's core answering it unknown.

## Overlaps with other plans

- `tools-surface`'s compiler-forced lines in `temper-client-tools/src/jig/settings_adapter.rs`
  and its tests: public-reading-build's `public-ctl` owns `temper-client-tools/**`. One line
  per site.
- jig's `docs/design/domain/engine.status.md`: public-reading-build's `public-status` edits
  it; this cut adds one 7.1 entry. A catch-up conflict is a status merge, both kept.
- `stock-configuring` edits one function of `temper-playground/src/documents.rs`
  (playground-browser's crate) so the playground keeps its settings and first chat; and the
  limits of the test documents that start from the stock.
- jig's `docs/design/domain/engine.status.md` and temper's `docs/design/protocol/documentation.status.md`
  carry this cut's entries beside public-reading-build's status edits (`public-status`).
