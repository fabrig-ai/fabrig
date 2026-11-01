# Where the code stands (playground-reset-r5)

Against `temper/docs/design/draft/playground-reset.md`, section 2, at
fabrig `main` `c30cf6a03` (the trunk's base `bc365feae8` adds only this
run's inputs). From read-only surveys, kept whole in `notes/surveys/`
(S1/S1a/S1b the document's forges, the connector, its store and acts; S2
the box; S3 the playground today; S4 jig's projects; S5 the set up,
the welcome, earlier work, old forms).

- **Forges are the document's.** `temper-service` parses
  `application.forges`, `identities`, `setup` and `demo`
  (`startup/application.rs`, `settings.rs`); numbers come from the
  document, no allocator; the connector gets `Configured` rows at
  construction (`temper-engine-domain-forge`, `Domain::new`); the service
  sizes per-forge tables from the configured count (tokens, records,
  identities, origins). No `forge.forges` limit; `forge.repositories`
  and its kin are profile values, not sums. `temper init`'s template
  writes forges 1 (fake, the demo's), 2 (GitHub), 3 (owned, asked).
  About 130 `fixtures::engine` calls in `temper-service` alone, and
  document fixtures in every temper world and the `temper` crate's
  process tests (S1a, section 3).
- **The box builds steps from the document** (`temper-forge-setup`'s and
  `temper-fake`'s `steps`); its pipe knows `start`/`stop` only; its only
  files are `<data>/box/started/N` and logs. No group files, no `remove`;
  `stop` keeps the record and the forge's files (S2).
- **The set up writes as it goes** (secrets at reconcile, project and
  adoption as the client's own acts after connect, accounts when added);
  the first repository's intent is volatile twice over (the connector's
  `Domain.first`, the service's `lifecycle::Intent`), and a test asserts
  so (`qa_first_repository.rs`, `first_repository_row_is_volatile_across_restart`)
  (S5).
- **The demo** is `Stored::Demo { forge, state }` (no row = not started,
  `Running`, `Ended`), `StartDemo`, `DemoAct` rows (`Close`,
  `RemoveAccount`) kept and resent at start, ended when another configured
  forge connects; `application.demo` in the document; grants `close` and
  `remove_account` for connector 1. The demo's start of a runtime project
  and deployment-owned account is designed but not built
  (`setup.status.md`) (S3).
- **jig deletes nothing.** No project lifecycle field, no project
  deletion act, no general deployment cancel (only `Close` of a person's
  conversation), no whole-task eraser (`forget.rs` erases only the
  forgotten-once-read rows; `Write::DropTranscript` exists in the store,
  unused by the domain), no adoption-release or settings-scope-erased
  notice to connectors; retention past a horizon is unbuilt (S4).
- **Earlier work:** none usable: `playground-apart.md` is not in the
  history; no remote branch before the cutoff builds any of this (S5).
- **Old forms:** the counters track memory forms only (all 0 in temper
  and jig); none concerns this draft (S5).
