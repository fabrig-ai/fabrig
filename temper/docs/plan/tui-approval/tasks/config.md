+++
id = "config"
part = "jig-approval"
kind = "feature"
needs = ["surface"]
files = ["jig/crates/jig-protocol-config/src/clients.rs", "jig/crates/jig-protocol-config/src/members.rs", "jig/crates/jig-protocol-config/src/tests/provider.rs", "jig/testing/jig-test-service/src/startup.rs", "jig/testing/jig-test-service/src/tests/startup.rs"]
effort = "medium"
minutes = 45
+++

## Goal

jig's configuration enables the approval apart from the printed link: `engine.clients.approval`, an optional boolean, false when absent, copied to the component's `Settings.approval`; and jig's reference service refuses it on a listener off loopback, as it refuses `link` (`jig/docs/design/protocol/configuration.md`, section 3, clients; `jig/docs/design/protocol/service.md`, section 3, step 4; `temper/docs/plan/tui-approval/notes/contracts.md`, 4).

Acceptance:

- `Clients { heartbeat, link, approval, provider }`; `approval` absent reads false, `true` and `false` read as given, any other value is the boolean's problem at `engine.clients.approval`.
- `config::clients::settings` sets `Settings.approval` from it (where `surface` left `false`).
- Its `members.rs` row, optional, in `link`'s form; the members' agreement tests pass.
- `jig-test-service`'s startup refuses `approval: true` on a non-loopback listener with `What::Plaintext` at `engine.clients.approval`, beside the listener's own problem, as for `link`; both enabled name both.
- No other configuration file changes: every existing fixture leaves `approval` out and keeps meaning what it meant.

## Inputs

- `jig/docs/design/protocol/configuration.md`, section 3, the clients' row.
- `jig/docs/design/protocol/service.md`, section 3, step 4.
- `temper/docs/plan/tui-approval/notes/contracts.md`, section 4 (binding).
- `temper/docs/plan/tui-approval/notes/common.md`.

## Owns and may change

- Owned: `jig/crates/jig-protocol-config/src/clients.rs` (the field, its parse, `settings`), `members.rs` (one row after `link`'s), `src/tests/provider.rs` (or the test file that holds `clients`' parse rows: put yours beside `the_printed_link_uses_the_first_owner_at_its_provider`); `jig/testing/jig-test-service/src/startup.rs` (`link_enabled`'s sibling for `approval`, and `server`'s check), `src/tests/startup.rs`.
- Readers: a literal `Clients { .. }` elsewhere gains `approval: false` (compiler-forced). If a generated reference of the members (a help page or a printed table) goes stale, regenerate it with its tool and name it.
- **Overlap:** `per-wake` edits `members.rs` and other files of this crate on its trunk. Keep your hunk to the one row and the clients' parse; touch no profile, name or override.

## Steps

1. **Tests first, red:**
   - Beside the clients' parse tests, `the_approval_is_off_unless_configured`, rows `(&[u8], Result<bool, What>)`: no `approval` (false), `"approval": true`, `"approval": false`, `"approval": 1` (the boolean's `What`), each under the small fixture's clients.
   - `jig-test-service`'s `a_link_on_a_non_loopback_listener_refuses_the_start` becomes a table over what is enabled, rows `(&[u8], &[&[&[u8]]])`: link alone (today's paths), approval alone (`application.server.listener` and `engine.clients.approval`), both (all three), each on both listeners it tries today; `a_clients_listener_on_a_public_address_is_refused` keeps its single path with both off.
2. The field, its parse (optional: `check::optional` or `object.member` with `read::boolean`, as the crate reads other optional members), the `settings` line, the row; `approval_enabled` beside `link_enabled` in the startup and its problem.
3. Break each rule once by hand (default to true; skip the loopback check) and see its row fail, then restore.
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-config -p jig-test-service --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-protocol-config`, then `-p jig-test-service`; `heavy cargo check -p temper-service --tests`.
5. Commit and push.

## Report

As the implementer's text says, and: the optional reader you used; each red row and what it failed on.
