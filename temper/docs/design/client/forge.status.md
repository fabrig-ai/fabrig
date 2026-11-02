# forge: status

2026-11-02. Where temper's code lags or departs from [forge.md](forge.md).

- **Not yet:** 2.1, 3: the `Changes` page, its queues and a queued
  change's place (no `Changes` address in `temper-client-domain`).
  2.1, 2.5: the steps after `landed` (`verifying`, `red`, `cleared`)
  and a result given at cleared. 2.3:
  amending a change's gates from its panel. 4: approvals, a person
  task's card at a head and the forge's reviews counted in the panel.
  5.1: "Land without CI" for a repository with no CI. 5.2: health. 5.3:
  the forge's part of a project's settings. 5.4: a row's summary by its
  name and host and its repositories' line, where the view says "forge N
  · KIND" (`temper-client-view`'s `forge/forges.rs`); its load, the
  App's setup URL and the `Lacking` line on the row; Replace
  and Forget, with no request on the wire; the warning of no git on the
  machine. 6.2: a drift hold's card, what changed, by whom and when.
  7: a goal's issue, landings as news, findings on a project's page. 9:
  the reads of the forge's other parts.
- **Departs:** 1, 5.4, 9: forges are records kept in the store, made by
  the flow, not configured. A row is removed, confirmed, by
  `RemoveForge` (`temper-client-view`'s `forge/forges.rs` `removal`),
  which 10's "Disconnecting a forge" leaves for later. A set up found
  unfinished offers Go on and Start over (`forge/connect.rs`
  `Shown::Unfinished`). 5.4, 10: the welcome's Forgejo card opens its
  Details, whose "Set up a Forgejo here" sends the set up, where "Run on
  this machine" sent it at once. The playground's forge is listed while the
  playground runs, not while a demo's half holds, and 9's read `Demo` is
  the read `playground`. Each because forges are store records and the
  playground is reset (`temper/docs/design/draft/playground-reset.md`,
  3, 4, 6).
