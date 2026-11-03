# D018: The native shell panics on requests only the unlinked client makes

- **Kind:** coupling
- **Where:** `jig-client/src/client.rs`, `jig-client-shell-native/src/shell.rs`, `jig-client-shell-native/src/connections.rs`, `jig-client-shell-native/src/files.rs`
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Signals in the step code": `panic!` in production code)
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (A8 F6)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list ("Client stack": the unlinked mode shares the linked enums)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (a shell cannot be handed a request it cannot serve)
- **Size:** S
- **Status:** open
- **Paid when:** the linked client's requests are a type without `Down` and `SignOut`, so the native shell's `panic!` arms go.
- **Measure:** `rg -o 'panic!' jig/crates/jig-client-shell-native/src -g '!**/tests/**' -g '!tests.rs' | wc -l` (4 at base)

## What

`jig_client::Client` has two modes, `new` and `unlinked` (`client.rs`, lines 143 and 157),
and one `Request` enum for both. `Request::Down` and `Request::SignOut` are made only
without a link (`client.rs`, around line 495), so the shell, always linked, answers them
with `panic!("linked hosts emit wire work")` (`shell.rs`, line 137; `connections.rs`,
line 162). Two more `panic!` guard events the shell should never see (`shell.rs`, line 261;
`files.rs`, line 292).

## Why it matters

The type admits requests the shell must not get; the compiler cannot help, and the
shell is ordinary Rust where a panic ends the process.

## What pays it back

Split the request type by mode (or move the unlinked mode's requests behind their own
type), so each host matches only what it can receive.
