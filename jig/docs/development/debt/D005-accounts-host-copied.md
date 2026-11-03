# D005: The reference service's parts are written twice, in temper and in jig's test service

- **Kind:** duplication
- **Where:** `testing/jig-test-service/src/accounts.rs`, `program_sources.rs`, `server.rs`, `store.rs`, `worker_route.rs`, `clients.rs`, `workers.rs`; temper's copies under `temper-service/src/`; `jig-protocol-accounts/src/component.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T4, A4 F3, A11 F6, A11 F7)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 11: temper's accounts adapter differs from the test service's copy)
- **Seen:** 2026-11-03 at `41d63da8c3`, in temper's backfill (`temper/docs/development/debt/D001-reference-service-copy.md`, which waits on this item)
- **Checked:** 2026-11-03 at `a6a8155d35`
- **Value:** high (one ordering of token refreshes; a fix in jig's service parts reaches temper; about 1,400 lines stop being kept in step by hand)
- **Size:** L
- **Status:** open
- **Paid when:** jig ships the parts temper copies (the accounts host, the program sources, the server, the store's host, the worker route, the clients' and workers' sides) as product code that both `jig-test-service` and temper's service compose; neither keeps its own copy of them.

## What

`jig_protocol_accounts::Component` now exists (`component.rs`, `pub struct Component`),
which paid the first half of the accounts part. Around it, each process writes the same
host: temper's `accounts.rs` is 440 lines, the test service's 406, and a whitespace-blind
diff differs in only 164 lines. Both build `driver::Component::new(check, limits,
crypto.clone())` the same way. The differences the quality run listed (trust roots, the
TLS bound, `Corrupt` against `PageRefused`, an output-room check) are where the copies
drifted.

The accounts host is one of seven parts. `jig/docs/design/protocol/service.md`, 7 has the
reference service "written as step code, so that it can be copied", and temper copied it:
its seven files of the same names share 1,369 distinct lines with the test service's
(the measure in temper's D001). The copies already differ, for example on what
`program_sources.rs`'s `Debug` shows.

## Why it matters

"The engine alone refreshes, so no two processes race a rotating token" is jig's
invariant; jig owns it (`jig/AGENTS.md`). Two copies can order refreshes differently, and
a fix lands in one. The same holds for every other copied part: a change of jig's protocol
layer has to be found and made again in temper by hand.

## What pays it back

jig turns the copied parts into product code both services compose (the review's
"jig-test-service is jig machinery"), with the points where the copies differ as
configuration. That changes `jig/docs/design/protocol/service.md`, 7, so the designer
writes it first. The accounts host goes first: it carries jig's invariant, and
`jig-protocol-accounts` already holds its component. temper's side, moving
`temper-service` onto the shipped parts and deleting its copies, is temper's D001.
