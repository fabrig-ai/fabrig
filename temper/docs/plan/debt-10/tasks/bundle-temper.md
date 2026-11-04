+++
id = "bundle-temper"
part = "bundle-helpers"
kind = "feature"
needs = ["bundle-part"]
files = ["temper/crates/temper-service/src/clients/bundle.rs", "jig/docs/development/debt/D005-accounts-host-copied.md", "jig/docs/development/debt/README.md", "temper/docs/development/debt/D001-reference-service-copy.md", "jig/docs/design/protocol/service.status.md"]
effort = "high"
minutes = 35
+++

# temper composes the bundle helpers, and D005 goes

## Goal

temper's `clients/bundle.rs` copies what `jig_protocol_engine_clients_http::bundle` now ships. Compose it, keeping
temper's routes and policies, and D005's **Paid when** then holds: the common helpers are shared product code both
services compose. Behaviour stays unchanged.

Acceptance:

- `temper/crates/temper-service/src/clients/bundle.rs` holds no copy of `kind`, `target`, `field`, `demand`, nor the bodies
  of `Bundle::file` or `chunk`'s slicing. `File` and `kind` are re-exported, so its readers don't change. Its head comment
  no longer says it is copied.
- `Answer`, `Reply`, `route`, `revalidates`, `serve_call`, `serve_event`, `fill_sign_in`, `fill_forge_sign_in`,
  `NO_BUNDLE_PAGE` and `NAME_BYTES` keep what they do: the busy answer, the no-bundle page, weak `If-None-Match`, the
  `done` on every ending.
- Every test of `temper-service`'s `tests/bundle.rs` and `tests/server.rs` passes unchanged.
- `jig/docs/development/debt/D005-accounts-host-copied.md` and its row in `jig/docs/development/debt/README.md` are gone.
- `jig/docs/design/protocol/service.status.md` loses its **Not yet** line for 11.4 and its date is today's.
- `python3 -I -B tools/citations/citations.py --all` reads 0.
- **Size cap:** net on `*.rs` at most -60 lines.

## Inputs

- `temper/docs/plan/debt-10/notes/common.md`, `temper/docs/plan/debt-10/notes/contracts.md`, `AGENTS.md`, `temper/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11.4; `temper/docs/design/client/browser.md`, 2.2.
- `jig/docs/development/debt/D005-accounts-host-copied.md`, whole.
- The reference's `jig/testing/jig-test-service/src/server/bundle.rs`, on the trunk, for how the composing reads.

## Owns and may change

- Owns: `temper/crates/temper-service/src/clients/bundle.rs`.
- This branch crosses into jig for the debt item and its status line alone: a paid item goes with the branch that pays
  it. `temper/docs/development/debt/D001-reference-service-copy.md` cites D005's path once, on a **Seen** line: replace
  the citation with the words `jig's D005` and change nothing else there. D001 stays open: `measure-recheck` updates it.
- `chunk` keeps temper's choice of body, `NO_BUNDLE_PAGE` or the bundle's content, and hands that to the shipped one.
- If temper needs something the shipped surface doesn't give, **stop and ask**: a pushed contract doesn't change.

## Steps

1. `pub use` of `File` and `kind`; `Bundle::file` calls the shipped `file`; `target`, `field`, `demand` and `chunk`'s
   slicing go. `heavy cargo check -p temper-service --tests`.
2. `heavy cargo nextest run -p temper-service --lib -E 'test(bundle) | test(server) | test(sign_in_configuration)'`.
3. D005, its row, the status line, D001's citation. fmt, clippy, old forms, citations. Commit, push: code, then the
   item.

## Report

As the implementer's text says, with the net lines. This moves code and pins no new rule.

## Rebase (2026-11-04, step merge, foreman debt-10/bundle-temper/foreman-1)

resolve service.status.md by removing the revised 11.4 gap and retaining the integration trunk’s other gaps
The merge conflicts in `jig/docs/design/protocol/service.status.md`.
Branch commit `bd9700464862` deletes the 11.4 bundle-copy gap; integration commit `fcf65bd27174` rewrites that same line after incorporating main’s `73233ed8b177`.
Rebase onto the current integration trunk. Resolve this file by deleting its entire revised **Not yet:** 11.4 paragraph, including the D005 citation.
Preserve the separate **Not yet:** 11 paragraph recording the 356-versus-200 adapter bound and citing D001; that debt remains open for `measure-recheck`.
Preserve the other status entries and the `2026-11-04` date.
Keep the task’s bundle composition and debt changes intact; this conflict needs no Rust change.
Run `python3 -I -B tools/citations/citations.py --all` and confirm 0 findings.
