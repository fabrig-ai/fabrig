+++
id = "args-fleet-host"
part = "payloads"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-fleet/src/attempt.rs", "jig/crates/jig-domain-fleet/src/domain.rs", "jig/crates/jig-domain-host/src/hosted.rs", "jig/crates/jig-domain-host/src/domain.rs"]
effort = "medium"
minutes = 55
+++

# jig-domain-fleet and jig-domain-host: each attempt and hosted helper takes its arguments folded

## Goal

Part of jig D016: fleet's five sites (`attempt.rs`'s `start`, `start_job`, `located`, `assigned`, `hosted`) and host's three (`hosted.rs`'s `inbound`, `serve`, `finished`) lose their `too_many_arguments` exception, by `notes/arguments.md`'s folds.

Acceptance:

- `rg -o 'too_many_arguments' jig/crates/jig-domain-fleet/src jig/crates/jig-domain-host/src -g '!**/tests/**' -g '!**/tests.rs' | wc -l` reads 0.
- `heavy cargo nextest run -p jig-domain-fleet --lib`, `-p jig-fleet-world`, `-p jig-domain-host --lib` and `-p jig-host-world` pass, with no assertion changed.
- About +25 net lines.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/notes/arguments.md`
- `jig/docs/development/debt/D016-positional-payloads.md`

## Owns and may change

- **Owns:** the files in the header.
- **`State::Waiting`:** `Claiming` holds what `State::Waiting` holds less `serial`. If `Waiting` holding a `Claiming` beside its `serial` is shorter, do that, and say so: stored forms aren't touched, as `State` isn't stored. If it is, keep `Waiting` as it is.
- **`inbound`** only wraps `inbound_named`. If its caller can build the `NamedEvent` and call `inbound_named` directly, drop the wrapper rather than add a constructor.

## Steps

1. Fleet; its tests.
2. Host; its tests.
3. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-domain-fleet -p jig-domain-host --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; each site's fold where it departs from the note's table.
