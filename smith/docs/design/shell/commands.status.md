# commands: status

2026-11-10. Where the tree's code lags or departs from [commands.md](commands.md).

- **Not yet:**
  - the whole document: the `--account` flag and the `account` setting.
    `smith-local-shell/src/command.rs` parses no `--account`, and
    `settings/schema.rs` has no `account` key, so 1's refusal that
    names `--account` points at nothing yet.
  -  the `anthropic` preset names an account, its endpoint's
    `account = 'anthropic'` and `[accounts.anthropic]` with `variable =
    'ANTHROPIC_API_KEY'` (`smith-local-shell/src/presets.rs`), where no
    preset names one.
- **Departs:**
  -  in `smith/crates/smith-agent-document/src/config/`,
    `TrustDocument`'s fields, `EndpointTransport::Tls`'s and the
    parameter `errors` of `endpoints`, which only the `cfg(unix)` trust
    loading reads, stay on every target under expectations scoped to
    `not(unix)`, not behind that `cfg` as the user's request of
    2026-11-04 asks of unix-only items, because fencing the fields
    would have wasm32 refuse, under `deny_unknown_fields`, keys it
    decodes, `tls::Name::new` is a portable check, and a call's
    argument takes no `cfg`, with `Decoded::resolve`, which passes
    `errors`, public on every target.
  - 5: `smith login` tells the sign-in deadline from other time-outs
    by the clock alone, because `skein_oauth::Failure::TimedOut` does
    not say which deadline passed: a redirect taken in the deadline's
    last `request_time` (20 s) whose code exchange is not answered
    before the deadline ends with the browser's words, though the
    browser answered.
