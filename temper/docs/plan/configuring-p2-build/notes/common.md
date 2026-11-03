# configuring-p2-build: standing rules

Every task of this plan names this note.

- **configuring-p2's standing rules hold** (`temper/docs/plan/configuring-p2/notes/common.md`),
  whole, with this plan's notes in place of that plan's: the shapes that cross tasks are
  `temper/docs/plan/configuring-p2-build/notes/contracts.md`, which takes configuring-p2's
  `notes/contracts.md` (P1, P2) as binding and adds the names that note left to this cut.
- **The trunk holds M3's landed code** (main merged into this plan's trunk with its first
  batch): jig's application settings (`Setting::Application`, `settings/application.rs`, the
  connector's `JudgeSettings`, `KeepSettings`, `KeepProposalSettings`, `DropProposalSettings`,
  `SaveSettingsDefaults` asks) and temper's kinds as settings. Extend them; never copy them.
- **An `unreachable!` arm another task removes:** where a spec gives a match an arm
  `unreachable!("a project's policy: <task>")` or `unreachable!("a restore: <task>")`, the
  named later task replaces it with a body; every other task leaves it as it is.
- **The one `docs/design/` edit a task makes** is the removal of its own **Not yet** text from
  the status file its spec names, as configuring-p2's rules say.
- **Design you read but never edit:** jig's `docs/design/domain/engine.md`, 17.2, 17.5, 17.6,
  17.10; `domain/people.md`, 5.1 and 5.2; `domain/connectors.md`, 17;
  `client/domain/contract.md`, 3.2, 5.1, 6; temper's `protocol/configuration.md`, 13;
  `client/tools.md`, 5.2 and 5.3. Each spec names the sections it builds.
