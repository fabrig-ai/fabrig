+++
id = "narrow-engine-clients"
part = "narrowing"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-engine-clients/src/component.rs", "jig/crates/jig-protocol-engine-clients/src/services.rs", "jig/crates/jig-protocol-engine-clients/src/host.rs", "jig/crates/jig-protocol-engine-clients/src/request.rs", "jig/crates/jig-protocol-engine-clients/src/translate/objects.rs", "jig/crates/jig-protocol-engine-clients/src/translate/objects/ended.rs"]
effort = "high"
minutes = 75
+++

# jig-protocol-engine-clients: what's waiting says whose it is

## Goal

Part of `jig/docs/development/debt/D017-unreachable-re-narrowing.md`. A `Waiting::Get` and a `CarriedRequest::Application` are stored wide, then matched again when they come back. Private fields carry the owner and the request, and twelve sites go.

Acceptance:

- `rg -c 'unreachable!'` reads, under `jig/crates/jig-protocol-engine-clients/src`: `services.rs` 1, `host.rs` 1, `translate/objects/ended.rs` 1, and nothing in `component.rs` or `request.rs`.
- `heavy cargo nextest run -p jig-protocol-engine-clients --lib` and `heavy cargo nextest run -p jig-engine-clients-world --test suite --test requests` pass with no assertion changed.
- Net production lines: the cut's estimate is about +15.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/narrowing.md`, binding
- `jig/docs/development/debt/D017-unreachable-re-narrowing.md`
- `jig/docs/design/protocol/service.md`, 11.6; `jig/docs/design/protocol/client.md`, 5

## The sites

- `component.rs:548`: pass `sign_in`.
- `services.rs:86, 87, 95, 96`: `Waiting::Get` holds a private `GetOwner { SignIn, Service }`, and `services.rs:40-60` uses that owner.
- `host.rs:49, 84`: split `owners` into `owners` and `carries`.
- `request.rs:290`, `services.rs:151`: `CarriedRequest::Application` holds the request's fields.
- `request.rs:433, 470`: `account_key_digest` also returns the kind. The survey was least sure of these two. If it takes more than that private helper's return, they stay, and you say so.
- `translate/objects/ended.rs:330`: match once at `translate/objects.rs:30`.

`services.rs:126` stays.

## Owns and may change

- **Owns:** the files in `files`. `src/tests/closing.rs` names `Waiting::Core`, which is unchanged.
- **Stay off** `translate/asks.rs`, `translate/reads.rs` and `translate/task.rs`: other plans are changing them.
- **No public item changes.** If `Waiting` or `CarriedRequest` is visible outside the crate, its sites stay, and you say so.

## Steps

1. The sites, file by file.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py --write)`; `heavy cargo clippy -p jig-protocol-engine-clients --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the counts; any site that stayed, and why.
