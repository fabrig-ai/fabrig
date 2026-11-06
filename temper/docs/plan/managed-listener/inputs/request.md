# managed-listener: a Forgejo connected as its admin, on this host, gets its webhook

The user's word, 2026-11-06, answering `adopt-access`'s question `managed-listener` with its recommendation (b). Serve the listener to a managed forge whose API is on loopback, so on this host, and keep polling for one elsewhere.

From that question: `temper/docs/design/protocol/setup.md`, 5.5 closes "This form serves no organizations or organization owner and no listener: it polls", and section 1 says a forge on another host polls. The code follows it: `temper/crates/temper-service/src/setup/runtime.rs`, `forge` and `local`, and `string_managed_serves_no_listener_even_when_owned_forges_have_webhooks`. 5.3, 5.4, 5.6, skein's `skein/docs/design/forge-setup.md`, 4.5 and setup.md, 16 read as if it had one or record the disagreement: settle them in the same landing.

**The change, as the question has it:** those two lines in `runtime.rs` read the forge's address; the pinning test and `setup_admin.rs`'s `repository_hooks(1)` check turn round; the process story's no-hook assertion becomes one hook at the listener's URL; and 5.5's sentence changes.

`temper-forge-setup/src/ini.rs` already writes `[webhook] ALLOWED_HOST_LIST` for an owned forge. A managed forge's `app.ini` isn't ours: say what a person needs, in the help, if the list isn't set. A hook that can't deliver leaves polling working: confirm it, and the deliveries' failures aren't surfaced as faults of a task.

A failing test first, in the lowest tier; no test reaches a real host. One task. fabrig/ui has priority. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1). `walk = false`: ops proves delivery at the next deployment of dev.fabrig.ai.
