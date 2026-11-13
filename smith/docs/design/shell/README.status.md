# shell: status

2026-11-12. Where smith's code lags or departs from [README.md](README.md).

Section 2.1, and 2.6 but for its entry below, have no known gap at main's `fdc26f71c6`:
`smith-agent-profile` owns the values, derivation and checks beneath the
service; `smith-agent-document` owns the reader, trace, program helpers
and fixture without linking the agent service. The service, local
service and shell keep the old paths as re-exports. Reader tests move
with the reader; tests needing the agent stay in the shell.

Section 2.2 has no known gap in Linux's `--program-child` startup for
programs that die when their agent aborts. The shell supplies its own
process ID on Linux and no parent on macOS; the child mode is Unix-only
and absent from the portable face. macOS's direct launch and deferred
crash lifetime are ../protocol/programs.md, sections 5.1 and 9's.

- **Departs:**
  - 2.5: in `smith/crates/smith-agent-document/src/config/`,
    `TrustDocument`'s fields, `EndpointTransport::Tls`'s and the
    parameter `errors` of `endpoints`, which only the `cfg(unix)` trust
    loading reads, stay on every target under expectations scoped to
    `not(unix)`, not behind that `cfg` as the user's request of
    2026-11-04 asks of unix-only items, because fencing the fields
    would have wasm32 refuse, under `deny_unknown_fields`, keys it
    decodes, `tls::Name::new` is a portable check, and a call's
    argument takes no `cfg`, with `Decoded::resolve`, which passes
    `errors`, public on every target.
  -  the login deadline departure remains `commands.status.md`, 5.
