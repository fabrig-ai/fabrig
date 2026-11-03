# configuring-p3: plan

Phase P3 of `temper/docs/design/draft/configuring.execution.md`: the configuring agent, built
on main (P1, P2 and the reference's M1 to M10 landed). Released in batches; this file is
brought up to date with each.

## Parts

| Part | Slice | Design | Tasks |
|---|---|---|---|
| `charter-tools` | 1 | jig's `domain/engine.md` 7.1; temper's `domain/agent.md` 4.3 | `tools-surface` (batch 1); `tools-offered`, `tools-store`, `tools-client` (batch 2) |
| `room` | 2 | temper's `domain/agent.md` 4.3 | `room-table`, `room-checks` (batch 3) |
| `configuring-agent` | 3, **the walk** (`walk = true`) | temper's `domain/agent.md` 8.1; `protocol/configuration.md` 13.4; `domain/testing.md` 6.4 | `agent-kinds`, `stock-configuring` (batch 3); `agent-stories`, `agent-ctl-story`, `help-stock` (batch 4) |
| `pages` | 4 | temper's `protocol/documentation.md` 8 | `limits-jig`, `limits-pages`, `members-jig`, `configuration-pages` (batch 4) |

15 tasks, all released in four batches, about 1,785 implementer-minutes. The cross-task shapes
are `notes/contracts.md` (C1 a charter's tools, C2 its brief room, C3 the limits' names, C4 the
document's members); the standing rules, `notes/common.md`.

## The design this cut wrote (batches 1 and 3)

- temper's `domain/agent.md`, 8.1 in its own words (no rule taken from the draft by citing
  it), its tools and the kinds it reads and proposes; section 8's table row without
  `withdraw`, a decide-family tool policy 1 does not give.
- temper's `protocol/configuration.md`, 13.4: charter 3's and template 3's rows in their own
  words; each stock charter's `tools` as numbers; the configuring agent as a setting.
- temper's `protocol/documentation.md`, 8: the pages as M3 and P2 left the document: `limits`
  (temper's names) and `limits-engine` (jig's), `configuration` (temper's shapes) and
  `configuration-engine` (jig's), the members gone to the store one row each; how each table is
  held to its parser or decoder.
- Statuses: the draft's `configuring.status.md`, its first Design lags entry closed; jig's
  `engine.status.md`, a 7.1 Not yet for a charter's `tools`; temper's `agent.status.md`, a
  Not yet for a run's reads of temper's kinds and of a project's policy;
  `documentation.status.md`, its 8 entry split by page pair.

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

4. `agent-stories` (150; needs `stock-configuring`, `agent-kinds`): 6.4's row in the
   engine's domain world, charter 3's flows; `agent-ctl-story` (120; the same needs): the
   walk's story through ctl in the joined client world; `help-stock` (docs, 60; the same
   needs): the help pages that state the stock; `limits-jig` (interface, 120; no needs):
   jig's names table, the parser's lookup; `limits-pages` (150; needs `limits-jig`): temper's,
   the setup's and the box's tables, the pages `limits` and `limits-engine`; `members-jig`
   (interface, 150; needs `limits-jig`): jig's members table, its decoders reading through
   it; `configuration-pages` (180; needs `members-jig`, `limits-pages`, `help-stock`):
   temper's members table, the pages `configuration` and `configuration-engine`, the concept
   pages' link.

The surveys found that neither the worker wire nor the tasks family carries a `RunPolicy`:
the engine picks a run's host tools before it encodes smith's charter
(`temper-service/src/assignment.rs`), and a task keeps its charter's number and revision.

## Critical path (as it stands)

`tools-surface` (60) -> `tools-offered` (45) -> `agent-kinds` (120) -> `room-table` (90) ->
`room-checks` (150): about 465 implementer-minutes. `stock-configuring` (180) ends at about
285 and the agent's stories at about 435; `tools-store` -> `tools-client` (210) runs beside
them; the pages' line, `limits-jig` (120) -> `members-jig` (150) -> `configuration-pages`
(180), about 450. The walk (slice 3) can run once `agent-ctl-story` and `stock-configuring`
land; it reads better with the pages and `tools-client` landed.

- **The first web chat becomes the configuring chat** once the stock offers template 3
  (`temper-client-domain`'s `first_chat`, written for it). The playground keeps its own:
  `stock-configuring` leaves charter 3 and template 3 out of the defaults it builds from the
  stock, since its scripted model plays the stock chat.
- **One writer per crate, bent where files never meet:** `temper-service` is written by
  `stock-configuring` (`stock.rs`), `limits-pages` (`limits/`) and then `room-checks` and
  `configuration-pages` (`startup/`); the last two both touch `startup.rs`, each told to keep
  its hunks apart. Every other shared crate is held in a line by `needs`.
- **The pages' tables are new files beside the parsers;** each parser looks a name up first,
  so the one hunk in a parser another plan changes is small.
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
- `limits-jig` and `members-jig` change `jig/crates/jig-protocol-config/src/**`, which
  public-reading-build's `public-limits` (merged into its trunk) also changed; its new names
  and members reach this plan's tables by a repair when it lands (the trunk's end check finds
  them). `limits-pages` touches `temper-service/src/limits.rs` (theirs too) in its module lines
  alone.
- `agent-ctl-story` adds a `mod` line to `temper/tests/client/tests/suite/main.rs`, which
  public-reading-build's `public-fuzzy-temper` changes.
- jig's `docs/design/domain/engine.status.md` and temper's `docs/design/protocol/documentation.status.md`
  carry this cut's entries beside public-reading-build's status edits (`public-status`).
