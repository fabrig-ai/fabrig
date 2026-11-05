# adopt-access: what every task holds to

- **No test reaches a real host.** The process tier's Forgejo is a throwaway on loopback, started by the fixtures.
- **Not on the table without a question:** the client wire (`temper-client-protocol-wire/schema/**`), the store's schema and records, any budget or timeout, `temper/docs/help/**` by hand (the generated reference regenerates by its own tool).
- **Leave alone:** `temper-client-view`, `temper-client-shell-web` and the gallery: the UI's drafts are in them.
- **A managed forge's listener stays as it is:** `temper/docs/design/protocol/setup.md`, 5.5, gives a forge connected as its admin no listener, so it polls, and `string_managed_serves_no_listener_even_when_owned_forges_have_webhooks` pins that. Whether that changes is this plan's question `managed-listener`; no task here changes it or waits for it.
- **No token in argv or a log;** a paste stays a 0600 file, as `temper/crates/temper/tests/support/forgejo_elsewhere.rs` keeps it.
