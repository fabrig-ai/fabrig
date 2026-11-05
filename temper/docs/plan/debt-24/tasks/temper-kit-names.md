+++
id = "temper-kit-names"
part = "jig-kits"
kind = "migration"
needs = ["channel-kit"]
files = ["temper/**/Cargo.toml", "temper/**/*.rs", "Cargo.lock"]
effort = "medium"
minutes = 20
+++

# Temper reads jig's kits

## Goal

Temper's edges onto jig's worlds: `jig_client_world`, about 125 references in 65 files; `jig_channel_world`, 22 in 13; `jig_engine_clients_world`, 2; `jig_core_world`, 1. Eight manifests. D031's **Paid when** says no package depends on a world, and this is the part no boundary test reads. By `temper/docs/plan/debt-24/notes/kits.md`, step 4.

Acceptance:

- `rg -n '^jig-(client|channel|engine-clients|core|client-protocol|protocol-accounts|test-connector)-world\b' temper -g Cargo.toml` reads nothing, and `rg -l 'jig_(client|channel|engine_clients|core|client_protocol|protocol_accounts|test_connector)_world' temper -t rust` reads nothing.
- `heavy cargo check --tests` passes on each package changed, naming them. `heavy cargo nextest run -p temper-client-world`, with its name from `temper/tests/client/Cargo.toml`, passes.
- **If trunk's merge at your run's start has added a use, the search finds it.** Run both searches once more before you push.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/kits.md`
- `jig/docs/development/debt/D031-worlds-as-kits.md`

## Owns and may change

- **Owns:** in those paths, a manifest's dependency line and a crate path's name, nothing else. No other line changes.
- **`temper/tests/client/**` and `temper/tests/client-browser/**` are other drafts' as well:** in their `.rs` files you change the crate's name in a path, whether in a `use` line, a type or an expression, and nothing else; `rustfmt` may rewrap a line it shortens. If a merge conflict shows another draft has reformatted the line, it's fab's merge and not yours: your line stays.

## Steps

A one-shot untracked `sed` per world over `rg -l`; manifests; `cargo metadata --offline --format-version 1 >/dev/null`; `cargo fmt`; `(cd temper && python3 -I -B scripts/old_forms.py)`; the checks above. Commit and push.

## Report

The common note's items; the packages and file counts. A rename: no rule to break by hand.

## Answers

2026-11-05. May the rename replace crate paths in types and expressions under `temper/tests/client/**` and `temper/tests/client-browser/**`, where the spec said a `use` line is all you touch? Yes. The search in Acceptance can't read nothing otherwise: 29 lines there name a world's crate outside a `use`, and the kits hold every module they reach (`harness`, `joint`, `fixtures`, `store_peer`). The spec's line now says so. Only the crate's name changes, with the rewrap `rustfmt` makes; every other token, assertion, test name and count stays. These packages' manifest lines change as Owns says.
