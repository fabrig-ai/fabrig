+++
id = "page-policy"
part = "served"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-service/src/clients/policy.rs",
  "temper/crates/temper-service/src/clients/bundle.rs",
  "temper/crates/temper-service/src/tests/server.rs",
  "temper/crates/temper-client-shell-web/playground/headers",
]
effort = "medium"
minutes = 45
+++

# One page policy for the service and the playground, and the playground's `headers`

## Goal

"The page's policy is temper's (`client/browser.md`, 2.2) with `worker-src 'self'`, and no
`connect-src` beyond `'self'` ... One function of temper's gives it, for the service and the
playground alike; the build writes it beside the files as `headers`, one `path: header` line
each (the policy on every answer, the digest directory cached for good, the page `no-cache`) ...
and a test holds the two equal" (`temper/docs/design/playground.md`, 7). Give temper-service's
policy module that function and the cache values, and check in the playground's `headers`
template that the bundle's build fills (`notes/contracts.md`, C6).

Acceptance:

- `temper/crates/temper-service/src/clients/policy.rs` gives C6's `Page`, `fields`,
  `FILES_CACHE`, `PAGE_CACHE` and `headers`; `FIELDS` stays and equals `fields(Page::Service)`;
  `add` uses `fields(Page::Service)`; `clients/bundle.rs`'s `serve_call` uses the two constants in
  place of its literals. The service's answers are byte for byte what they were.
- `temper/crates/temper-client-shell-web/playground/headers` exists, ASCII, equal to
  `policy::headers(b"{bundle}")`.
- New tests in `temper/crates/temper-service/src/tests/server.rs`, beside the policy's test there
  (~210):
  - `the_playground_policy_adds_only_its_worker_source`: `fields(Page::Playground)` is
    `fields(Page::Service)` but for the CSP, which is the service's followed by `; worker-src
    'self'`; neither names a `connect-src` other than `'self'`.
  - `the_playgrounds_headers_file_is_the_policys`: the template, read by `include_bytes!` from
    `../../../temper-client-shell-web/playground/headers` (relative to the test file; fix the
    depth to the real path), equals `headers(b"{bundle}")`; and `headers(b"/bundle/0123")` holds,
    in order, one `/*: <Name>: <value>` line per playground field, then `/bundle/0123/*:
    Cache-Control: public, max-age=31536000, immutable`, then `/*: Cache-Control: no-cache`.
- `heavy cargo nextest run -p temper-service` passes with the trunk's count plus the two;
  `heavy cargo clippy -p temper-service --all-targets -- -D warnings` passes.

## Inputs

- `temper/docs/plan/playground-browser/notes/common.md`;
  `temper/docs/plan/playground-browser/notes/contracts.md`, C6.
- `temper/docs/design/playground.md`, 7; `temper/docs/design/client/browser.md`, 2.2.
- `temper/crates/temper-service/src/clients/policy.rs` (the fields, `add`),
  `temper/crates/temper-service/src/clients/bundle.rs` (~201-208, the cache literals),
  `temper/crates/temper-service/src/tests/server.rs` (~210-229, the policy's test).

## Owns and may change

- The four files in `files`. Nothing else of temper-service changes: its fixtures' literal copies
  of the policy (`src/tests/fixtures/bundle.rs`, `POLICY`) and jig's copies stay as they are,
  since the service's policy does not change.
- Pre-approved options:
  - `headers` returns `Vec<u8>` (the module is `alloc`); if the module's `no_std` setting makes
    another owned type fit better (`Box<[u8]>`), use it and say so; C6's readers take bytes.
  - If `include_bytes!`'s relative path would be clearer through a `const` in a fixtures module,
    keep it in `server.rs` anyway: this task owns no other test file.

## Steps

1. `Page`, `fields`, the constants and `headers` in `policy.rs`, its module doc citing
   `playground.md`, 7 beside `client/browser.md`, 2.2. `add` and `serve_call` on them.
2. Write the template by running `headers(b"{bundle}")` once in a throwaway test or by hand, then
   the two tests; delete any throwaway before the push.
3. `cargo fmt`, the old-form count for temper, clippy and the package's tests; commit; push.

## Report

As the implementer's text says, and also: the template's text, quoted whole.
