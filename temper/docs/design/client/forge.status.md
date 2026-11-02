# forge: status

2026-11-02. Where temper's code lags or departs from [forge.md](forge.md).

- **Not yet:** 2.1, 3: the `Changes` page, its queues and a queued
  change's place (no `Changes` address in `temper-client-domain`). 2.3:
  amending a change's gates from its panel. 4: approvals, a person
  task's card at a head and the forge's reviews counted in the panel.
  5.1: "Land without CI" for a repository with no CI. 5.2: health. 5.3:
  the forge's part of a project's settings. 5.4: a row's summary by its
  name and host and its repositories' line, where the view says "forge N
  · KIND" (`temper-client-view`'s `forge/forges.rs`); its load, the
  App's setup URL and the `Lacking` line on the row; Replace
  and Forget, with no request on the wire; the warning of no git on the
  machine. 5.4: Forgejo on this machine's Details for a forge to make,
  their words and notes, where the view draws "Show how", a link to the
  deployment's help, and the fields alone (`forge/connect.rs` `body`,
  `Choice::Owned`). 5.4, Elsewhere: the dot leaving out the playground's
  forge, which `temper-client-domain`'s `forge/listed.rs` `rows` counts.
  6.2: a drift hold's card, what changed, by whom and when.
  7: a goal's issue, landings as news, findings on a project's page. 9:
  the reads of the forge's other parts.
- **Departs:** 5.4: `ConnectForge`'s `?forge=elsewhere` draws A Forgejo
  elsewhere's Details, the configuration's entries for a forge and a
  restart (`forge/connect.rs` `elsewhere`); Your Forgejo's card has the
  muted line "Needs a change to temper’s configuration and a restart"
  (`words.rs` `ForgeElsewhereTime`); and Setting up says "You may leave
  this page: the steps go on, and the banner says where they are."
  (`words.rs` `ForgeLeave`), where no banner says it: words for a forge
  added to the configuration by hand, and for a banner's set up under
  way, which the store's records and the playground's one line leave
  without a place.
