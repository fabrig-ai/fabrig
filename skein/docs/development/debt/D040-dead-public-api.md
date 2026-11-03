# D040: Public items with no caller, or only a test caller

- **Kind:** dead code
- **Where:** `skein-oauth/src/record.rs`, `skein-forge/src/forgejo/time.rs`, `skein-forge/src/forgejo/client.rs`, `skein-forge/src/forgejo/client/delivery.rs`, `skein-forge/src/forgejo/server/writes.rs`, `skein-fake-llm-protocol/src/provider.rs`, `skein-llm-connection/src/component.rs`, `skein-llm/src/openai/response.rs`, `skein-ui-web/src/timer.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (phase 1 item 5) and `reviews/2026-10-31-skein-areas.md` (A2 F11, A9 F11, A10, A11 F11)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low
- **Size:** S
- **Status:** open
- **Paid when:** each item below is deleted, or has a caller in shipping code, or is made test-only (`#[cfg(test)]`) where only tests need it.

## What

No caller in any tree:

- `skein_oauth::RECORD_VERSION` (`record.rs`);
- `forgejo::time::RFC3339_BYTES` (`time.rs`);
- `provider::head_bound` (`skein-fake-llm-protocol/src/provider.rs`).

Only tests call them:

- `Component::admit_with_reasoning_drop`
  (`skein-llm-connection/src/component.rs`);
- `openai::encode_completion` (`skein-llm/src/openai/response.rs`);
- Forgejo's `client::first`, `client::delivery::first` and
  `server::body_first`, which ignore their argument and return the same
  request;
- skein-ui-web's pure `Timer` (`timer.rs`): only
  `src/tests/history.rs` uses it, while the web timer keeps its own
  deadline.

## Why it matters

Each is surface a reader must learn and a change must keep compiling,
for nothing that ships.

## What pays it back

Delete them, or fold the test-only helpers into the tests that use
them. One task.
