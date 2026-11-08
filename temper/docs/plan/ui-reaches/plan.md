# ui-reaches: what reaches you, its three classes and Away

The request: `inputs/request.md`, milestone N1 of `temper/docs/design/draft/whole-story.execution.md`, executing `temper/docs/design/draft/what-reaches-you.md`, sections 2 to 6. `notes/common.md` holds the rules every task follows; `notes/contracts.md` the shapes that cross tasks.

## Parts and tasks

Released so far:

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `kit` | `tasks/reach-kit.md` | | the crate `temper-protocol-reaches`: kinds, classes, a project's table, patterns, `judge`, Stop asking's offers; their tables and the fuzz test | 120 |
| `wire` | `tasks/reach-wire.md` | | `ProjectClasses` and `Reach` on the client wire, `ProjectClasses` and `SettingValue::classes` in the store | 60 |

Planned, in later batches (names may still move until released):

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `classes` | `classes-value` | `reach-kit`, `reach-wire` | the kit's record conversions; the forge's `Value::Classes`, kind 8's number and stock, its store codec |
| `classes` | `classes-kind` | `classes-value` | kind 8 declared, judged, kept, read and erased; `project-classes` for agents' `settings` and `propose_settings` |
| `classes` | `classes-tools` | `classes-value` | kind 8 in `temper ctl settings`, settings files and their help |
| `classes` | `classes-ways` | `classes-kind` | the stock ways lay kind 8's defaults |
| `decide` | `decide-forge` | `classes-value` | Away without kinds; `decided` judged by the table for any caller's person; decided and asked rows, kept 7 days, bounded by `forge.decided_per_project`; their documents |
| `decide` | `decide-root` | `decide-forge`, `classes-kind` | the root's caller's person for any task; the tool's kinds, paths and `under`; its words naming the reach; the charters; the engine world's stories |
| `keep` | `keep-asking` | `decide-forge` | the person's fact `KeepAsking` |
| `client` | Away by the table, Needs you's reason and Stop asking, Decided for you on Needs you and Home | `decide-forge` and the above | the client's pages, both shells |
| `client` | `ways-away` | the client's Away | kind 7's `away` member gone |

**Critical path as it stands:** `reach-kit` → `classes-value` → `decide-forge` → `decide-root` and the client's tasks.

## What was decided

- **One pure crate for the judgement,** `temper-protocol-reaches`, since both the forge's `decided` and the client read a project's classes: the Away switch's showing, the reason line and Stop asking's offers come from the same table and the same function (`temper/docs/design/domain/forge.md`, 5.10).
- **Kind 8, a project's classes,** a new kind of temper's, rather than members of kind 7: kind 7 is read by the client alone and the engine decides nothing by it, where kind 8 is judged in every `decided` (`temper/docs/design/protocol/configuration.md`, 13.2).
- **`paths` is no call's kind:** a call names its own kind and the paths it would touch, and each path set they touch is judged as an item of its own; the strictest class wins (5.10). The draft's table lists `paths` as a kind "per path set", which this keeps.
- **A rule in words** is an item with words narrowing its subject; code can't check words, so a call names the item it decides under (`under`), and that item's class stands for its subject (5.10).
- **Path sets are the project's,** their patterns over any of its repositories' paths, since the draft's example names one repository's tree and N2's rules by path judge a change in its own repository.
- **The bounds** of a table: 20 items, words 120 bytes; 6 sets, names 40 bytes, 6 patterns of 80; a call's paths 16 of 256 bytes. A full table's encoding is kind 8's bound.
- **A stored format and the wire change freely** (the request): kind 8's record and the store's variant are new; later batches change `Away`, `SetAway`, `Decided` and its document.

## Overlaps with drafts beside this one

- **`temper/docs/design/protocol/configuration.status.md`** is also `ignored-engine/restore-stories`': this cut adds its entries at the file's head, far from that task's 13.2 entry.
- **`temper/docs/design/domain/forge.status.md`** is also `look-round-24`'s (`asked-sentence`, merged; `ci-workflows`; `rule-view`): this cut adds its entries after the 5.8 and 5.9 departure, apart from theirs.
- **The client wire's schema and generated code** were changed by `look-round-24/ways-wire` and `tui-box-front/forges-linked` (both merged on their trunks): `reach-wire` adds records after `ProjectWays` alone; a merge that meets theirs regenerates rather than hand-merges.

## Who hears first

Every change to `temper-engine-domain` and `forge_route.rs` (the owner of coding runs writes them too), and every change to `temper-client-domain` (the TUI's owner), is listed here as its batch is released. In this batch: none.
