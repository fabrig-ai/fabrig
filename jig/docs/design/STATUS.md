# Status

Updated 2026-10-29. How far jig's code is from its design.

Each design document's status is in `<doc>.status.md` beside it:
- [`README.md`](README.status.md)
- [`domain/README.md`](domain/README.status.md)
- [`domain/core.md`](domain/core.status.md)
- [`domain/tasks.md`](domain/tasks.status.md)
- [`domain/authority.md`](domain/authority.status.md)
- [`domain/connectors.md`](domain/connectors.status.md)
- [`domain/engine.md`](domain/engine.status.md)
- [`domain/root.md`](domain/root.status.md)
- [`domain/people.md`](domain/people.status.md)
- [`domain/hosts.md`](domain/hosts.status.md)
- [`domain/testing.md`](domain/testing.status.md)
- [`domain/accounts.md`](domain/accounts.status.md)
- [`domain/providers.md`](domain/providers.status.md)
- [`client/domain/README.md`](client/domain/README.status.md)
- [`client/domain/client.md`](client/domain/client.status.md)
- [`client/domain/contract.md`](client/domain/contract.status.md)
- [`client/domain/root.md`](client/domain/root.status.md)
- [`client/domain/pages.md`](client/domain/pages.status.md)
- [`client/domain/objects.md`](client/domain/objects.status.md)
- [`client/domain/view.md`](client/domain/view.status.md)
- [`client/domain/testing.md`](client/domain/testing.status.md)
- [`examples.md`](examples.status.md)
- [`protocol/README.md`](protocol/README.status.md)
- [`protocol/service.md`](protocol/service.status.md)
- [`protocol/channel.md`](protocol/channel.status.md)
- [`protocol/llm.md`](protocol/llm.status.md)
- [`protocol/configuration.md`](protocol/configuration.status.md)
- [`protocol/client.md`](protocol/client.status.md)
- [`protocol/store.md`](protocol/store.status.md)
- [`protocol/browser.md`](protocol/browser.status.md)

Not single documents:

A cancel grace emits durable `FinishRequested { revision }`, translated to
smith's finish flag live and in Start; clients see Amendment. Tasks store
v9 and worker wire v9 are incompatible with v8. Other store families and
client versions keep their versions.

| Area | Built | Not yet |
|---|---|---|

Configuring E-QA-2's newest-only compatibility interim is replaced by the full retained-revision and charter-version bound (17.8). A refused acceptance keeps the proposal pending. E-QA-3 is pending the application-owned judged-settings mechanism; no test-only hook substitutes for it.
