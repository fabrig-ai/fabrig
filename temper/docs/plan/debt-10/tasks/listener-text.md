+++
id = "listener-text"
part = "findings"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-engine-clients/src/host.rs", "jig/crates/jig-protocol-engine-clients-http/src/lib.rs", "jig/testing/jig-test-service/src/clients.rs", "jig/testing/jig-test-service/src/tests/startup.rs", "temper/crates/temper-service/src/clients/printed.rs", "temper/crates/temper-service/src/tests/checks.rs", "jig/docs/design/protocol/service.status.md"]
effort = "medium"
minutes = 25
+++

# B2: test `[::]:7100` at the start, and move `address_text` beside `printed_link`

## Goal

The review says `[::]:7100` should panic on `Listening`: `address_text` returns `None` for a non-loopback IPv6
address, and `printed_link`, temper's `clients.rs` and jig's `clients.rs` `.expect` it. This cut read otherwise:
both services refuse such a listener at the start, so no document reaches `Listening` with it. Show which by
test. Then move `address_text` and `HOST_BYTES` beside `printed_link`.

Acceptance:

- A test in each service feeds `[::]:7100` through its real start, and its outcome is in your report.
- `address_text` and `HOST_BYTES` are in `jig-protocol-engine-clients-http`, beside `printed_link`, and
  `jig_protocol_engine_clients::host` holds neither. `address_text` keeps its signature and its doc.
- `jig/docs/design/protocol/service.status.md` loses its **Not yet** line for 11.6 and its date is today's.
- **Size cap:** net at most +30 lines, tests included.

## Inputs

- `temper/docs/plan/debt-10/notes/common.md`, `AGENTS.md`, `jig/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11.6, "The printed link", on the trunk; 3, step 4.
- `temper/docs/design/protocol/configuration.md`, section 3, `clients`' `listener`: on loopback in standalone and
  wherever `engine.clients` enables the printed link.
- What this cut read. temper: `checks::standalone` (`temper/crates/temper-service/src/startup/checks.rs`) pushes
  `What::Loopback` for every clients' listener off loopback, in every engine shape, whatever the printed link; it
  runs before `Clients::new`. `Shape::ClientRemote` in `tests/checks.rs` pins it for `10.0.0.1:7100`. jig:
  `server` in `jig/testing/jig-test-service/src/startup.rs` refuses any listener off loopback, pinned by
  `a_link_on_a_non_loopback_listener_refuses_the_start`.

## Owns and may change

- Owns: `address_text` and `HOST_BYTES` in `jig/crates/jig-protocol-engine-clients/src/host.rs`, leaving; the same
  arriving in `jig/crates/jig-protocol-engine-clients-http/src/lib.rs`, public, beside `printed_link`.
- May change, the call site alone: `jig/testing/jig-test-service/src/clients.rs` (`kit::host::address_text` becomes
  `http::address_text`); `temper/crates/temper-service/src/clients/printed.rs` (its `pub(crate) use` names the
  mapping). temper's other readers go through `printed` and don't change.
- May add a test to: `jig/testing/jig-test-service/src/tests/startup.rs`, `temper/crates/temper-service/src/tests/checks.rs`.
- May change `jig/docs/design/protocol/service.status.md`, its 11.6 line and date alone.
- **If the tests go as this cut read** (both services refuse at the start): the panic is not real from a document,
  and the request says to skip a wrong finding, saying why. Keep the `.expect`s and `address_text`'s `Option`, keep
  the tests, and say so in the report. Do not add a refusal at `Clients::new`: the start already holds it.
- **If a document does reach `Listening` with `[::]:7100`:** the panic is real, and `configuration.md` means an
  address off loopback to work outside standalone and the printed link. So make `address_text` print any IPv6
  address, in RFC 5952's form, `[..]` around it, within `HOST_BYTES`; it then returns `Box<[u8]>`, and every
  `.expect`, `?` and `.ok_or` on it goes (temper's `clients.rs`, `clients/google.rs`, `route.rs`,
  `clients/provider.rs`, jig's `clients.rs`). A test on `[::]:7100` pins the text. This option raises the size cap to
  +60; say so in the report.

## Steps

1. temper, in `tests/checks.rs`: `a_v6_clients_listener_off_loopback_is_refused_at_the_start`, in the standalone
   document and in `fixtures::engine_alone_document()`: `application.clients.listener` set to
   `{"socket":"[::]:7100"}` by `fixtures::with`, then `crate::named`, expecting
   `application.clients.listener: must be a socket address on loopback`.
2. jig, in `tests/startup.rs`: a row or a sibling of `a_link_on_a_non_loopback_listener_refuses_the_start` on
   `fixtures::with_server_listener(br#"{"socket":"[::]:7100"}"#)`.
3. Run both alone. Bite: drop the arm that refuses, see the test fail and, if the service then gets as far as
   `Listening` in that test, what it does.
4. Take the option the outcome names. Then move `address_text` and `HOST_BYTES`, and fix the readers.
5. Update the status file. `heavy cargo nextest run -p jig-protocol-engine-clients -p jig-protocol-engine-clients-http
   -p jig-test-service`, `heavy cargo nextest run -p temper-service --lib`, each narrowed to what you changed where
   you can. fmt, clippy, old forms, citations. Commit, push.

## Report

As the implementer's text says, with `notes/common.md`'s table and the net lines. First say in one line whether the
panic was real, and what ran to show it.

## Rebase (2026-11-04, step merge, foreman debt-10/listener-text/foreman-1)

Resolve the status-file conflict by preserving trunk’s added gaps and removing only the completed 11.6 gap.
The merge conflicts in `jig/docs/design/protocol/service.status.md`.
Commit `127f855119592a8758dbd1844f7398f4a8b1a28b` added gaps for 11.2 (`Undecoded`), 11.4 (`bundle`), and 11/11.5 (`Drivers`/`mark_durable`) beside the line this branch deletes.
Rebase onto the landing trunk and retain those additions and all other trunk status entries.
Remove only `**Not yet:** 11.6: address_text stands in jig_protocol_engine_clients::host` (with its existing Markdown formatting).
Keep the status date current; both inspected versions already say `2026-11-04`.
Preserve the branch’s formatter move, call-site updates, and startup-refusal tests. No code change is needed for this conflict.
