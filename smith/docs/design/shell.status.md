# shell: status

2026-11-08. Where smith's code lags or departs from [shell.md](shell.md).

Section 2.2 has no known gap in Linux's `--program-child` startup for
programs that die when their agent aborts. The shell supplies its own
process ID on Linux and no parent on macOS; the child mode is Unix-only
and absent from the portable face. macOS's direct launch and deferred
crash lifetime are protocol/programs.md, sections 5.1 and 9's.

- **Not yet:**
  - 2.1, 2.6: `smith-agent-profile` and `smith-agent-document` aren't
    built. The document's reader, its trace's settings and the program
    helpers are `smith-agent-shell`'s; the profile, the configuration's
    values and its checks `smith-agent-service`'s; `host_limits`
    `smith-local-service`'s.
  - 4, 5.2: the `--account` flag and the `account` setting.
    `smith-local-shell/src/command.rs` parses no `--account`, and
    `settings/schema.rs` has no `account` key, so 4.1's refusal that
    names `--account` points at nothing yet.
  - 5.2: the `anthropic` preset names an account, its endpoint's
    `account = 'anthropic'` and `[accounts.anthropic]` with `variable =
    'ANTHROPIC_API_KEY'` (`smith-local-shell/src/presets.rs`), where no
    preset names one.
- **Departs:**
  - 2.5: in `smith/crates/smith-agent-shell/src/config/`,
    `TrustDocument`'s fields, `EndpointTransport::Tls`'s and the
    parameter `errors` of `endpoints`, which only the `cfg(unix)` trust
    loading reads, stay on every target under expectations scoped to
    `not(unix)`, not behind that `cfg` as the user's request of
    2026-11-04 asks of unix-only items, because fencing the fields
    would have wasm32 refuse, under `deny_unknown_fields`, keys it
    decodes, `tls::Name::new` is a portable check, and a call's
    argument takes no `cfg`, with `Decoded::resolve`, which passes
    `errors`, public on every target.
  - 4.5: `smith login` tells the sign-in deadline from other time-outs
    by the clock alone, because `skein_oauth::Failure::TimedOut` does
    not say which deadline passed: a redirect taken in the deadline's
    last `request_time` (20 s) whose code exchange is not answered
    before the deadline ends with the browser's words, though the
    browser answered.
