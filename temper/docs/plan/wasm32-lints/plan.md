# wasm32-lints: plan

Draft: `temper-playground` and `smith-agent-shell` pass the wasm32 clippy and join
`.config/gate/wasm32.txt` (`inputs/request.md`). One part, one task, no design, as the request
asks.

## Parts and tasks

- **clippy-both** (slice 1): `tasks/clippy-both.md`, no needs, released in the first and only push.
  Fixes the named lint failures, then whatever each further layer of dependents reports, by
  attributes and `cfg`s alone; adds both packages to the list; closes the status entries in
  `playground.status.md`, 9, `shell.status.md`, 2.5, and the summaries in smith's and temper's
  `STATUS.md`.

The critical path is that one task.

## Decisions

- **The named failures are not all of them, likely.** No build ran for this cut: the call has no
  builder. `cargo metadata` for wasm32 shows 17 packages depending on `jig-domain-core`,
  `smith-local-service` or `smith-agent-shell`, which cargo has therefore never linted for wasm32,
  `temper-engine-domain` and `temper-service` among them. A source scan finds 19 unconditional
  `#[expect]`s of target-sensitive lints in them, listed in the spec as likely, unmeasured. So the
  spec pre-approves the same forms across everything the runs reach, and `minutes` allows several
  rounds. The request's "in jig's and smith's crates" holds only for what was reported.
- **Only what the wasm32 clippy reports changes.** Scoping every candidate up front would lose the
  wasm32 expectation of each lint met on both targets.
- **Three items take a scoped `expect`, not a `cfg`,** where the request prefers a `cfg`:
  `TrustDocument`'s fields, `EndpointTransport::Tls`'s fields and the parameter `errors`. A `cfg`
  on the fields would change portable behaviour: with `deny_unknown_fields`, wasm32 would refuse
  keys it decodes today, and `tls::Name::new` is a portable check (`shell.md`, 2.5). A call's
  argument can take no `cfg`, and `Decoded::resolve` is public on every target.
- **jig stays attributes-only,** as the request asks: a jig fix beyond that is a question, not a
  reshape.
- **Tests narrow to `smith-agent-shell`:** these forms change no native code, and the trunk's end
  check runs the wider set.
- **`STATUS.md` follows:** smith's and temper's summaries both repeat the wasm32 gap, so they are
  edited with the entries they index, though the request names only the two status files.

## Overlaps

Nothing in this draft's early edits meets another draft: at `98385b96c1`, debt-7, debt-8 and
serve-without-box change none of `public/index.rs`, `credentials.rs`, `config/native.rs`,
`config/document.rs`, `config/endpoints.rs`, the list or the two status files.

The likely later edits could not be kept clear of them, since they sit where the wasm32 clippy
reports: `temper-service/src/stock.rs`, `worker_git.rs` and `setup/lifecycle.rs` fall under
`debt-7/temper-store-host`'s `temper/crates/temper-service/src/**`, and
`temper-engine-domain/src/boundary.rs` under `debt-8/core-prepare-agent`. Each is one attribute
line, and no trunk's diff touches those attributes so far, checked on `boundary.rs`, `store.rs` and
the three `temper-service` files.
