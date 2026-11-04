+++
id = "temper-store-host"
part = "service-clients"
kind = "migration"
needs = ["jig-store-host", "temper-server-part"]
files = ["temper/crates/temper-service/src/**"]
effort = "medium"
minutes = 60
+++

# temper: compose jig's store host

## Goal

`T/store.rs`'s `Store` (`T` and `J` are the paths `notes/service-parts.md` gives) composes `jig_protocol_store::host::Hosted`,
which `jig-store-host` shipped, as its `host` field, and stops holding its own copy. It keeps its routes, its admission, its
wires, its token tagging, its dispatch and its test hooks. `component`, `above`, `secrets` and `sources` are read through
`store.host`, by one `sed`, for the reasons that task gives.

Acceptance:

- `T/store.rs`'s `Store` holds its `cfg(test)` hooks, `host`, `routes`, `routed`, `submitted`, `opened`, `pending_generation`
  and `installation`.
- `Store::secrets` stays on the adapter, and counts as today: `secret_requests`, `forge_loads` and `erases` read as they do.
- temper's immediate set holds: `NotReady::Above`, so `SecretsNotReady` waits in `above` for `answer`.
- Every test passes, changed only in the path of a moved field or method.
- **Size cap:** the branch's net lines are at most -90. Measure it, and stop and ask if it's passed.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `temper/docs/plan/debt-7/notes/service-parts.md`, `AGENTS.md`, `temper/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11 and 11.5.
- `jig/crates/jig-protocol-store/src/host.rs`, from `jig-store-host`, and that task's `J/store.rs`: the same shape.

## Owns and may change

- Owns: `T/store.rs`.
- May change, in the way listed: every `.rs` under `T` by the one `sed` of step 3, and nothing else in those lines.
- The readers, by file, at this plan's base. Each is one token on one line; counts are lines:

  | Where | Lines |
  | --- | --- |
  | `T/engine.rs` 18, `T/accounts.rs` 4, `T/clients.rs` 2, `T/engine/diagnostics.rs` 2, `T/forge/gone.rs` 2, `T/forge/records.rs` 1, `T/forges.rs` 1, `T/system.rs` 1 | 31 |
  | `T/tests/fixtures.rs` 40, `T/tests/engine.rs` 31, `T/tests/accounts_host.rs` 18, `T/tests/setup.rs` 12, `T/tests/setup_admin.rs` 11, `T/tests/store.rs` 11, `T/tests/forge_gone.rs` 9, `T/tests/clients.rs` 5, `T/tests/setup_tokens.rs` 5, `T/tests/forge.rs` 3, `T/tests/forge_runtime.rs` 3, `T/tests/setup_replace.rs` 3, `T/clients/playground_tests.rs` 2, `T/tests/setup_adoption.rs` 2, `T/tests/setup_lifecycle.rs` 2, `T/tests/stop.rs` 2, `T/tests/workers.rs` 2, `T/tests/accounts.rs` 1, `T/tests/setup_github.rs` 1, `T/tests/setup_reachable.rs` 1 | 153 |

- Outside `temper-service`, `temper/testing/temper-github-app/src/store.rs` and `temper/tests/engine/domain/` read `store::limits`,
  `store::route` and `store::WIRES`, which stay. Nothing there reads `Store`'s fields.
- Not yours: anything in jig.

## What differs, and which holds

- **`hold_mark`, `held_mark`, `secret_requests`, `forge_loads`, `erases`:** stay on the adapter, each exactly where it is: the
  first two in `down`, the others in `secrets`, which then calls `self.host.secrets(owner, request)`. `Hosted::ask`'s
  `Secrets` arm passes no hook, and nothing in temper reaches it: `route::down` and `Up::Ask` make no `Request::Secrets`,
  and no test asks one. If you find one, stop and ask.
- **Seed:** temper's `new` passes `u64::from_le_bytes(*installation.first_chunk::<8>()..)`, as it does.
- **Wires:** `assert!(host.component.wires(&WIRES), "valid engine wires before open");` after `Hosted::new`.
- **Admission, `Refusal`, `page_bytes`, `read_room`:** stay. `layout` calls `protocol::sizes::fill(&mut kv, &sizes).ok_or(arithmetic)?`
  where it wrote those lines; `kv.page.bytes` stays below it.
- **`joins`, the route's table:** stay.
- **`Lane` and `Submission`'s tagging:** stays. `complete`, `publish`, `MASK` and `owns` stay, on `self.host`.

## Steps

1. `T/store.rs`:

   ```rust
   let mut host = Hosted::new(limits, application_sizes, models, seed, NotReady::Above);
   assert!(host.component.wires(&WIRES), "valid engine wires before open");
   ```

   `ask`, `clock` and `next_deadline` go; `down` calls `self.host.ask(request)`. `opening_limits` reads `*self.host.limits()`.
   `adopt`, `work_pending`, `closed`, `idle`, `close` and `iterate` take the forms `jig-store-host`'s `J/store.rs` has,
   `pub` as they are.
2. Remove the `cfg_attr(test, expect(clippy::struct_excessive_bools, ..))` on `Store`: `closing` has moved, so three bools
   remain and the expectation would go unfulfilled.
3. The path change, its own commit, from the repository's root. `secrets` the method stays on the adapter, so it needs its own form:

   ```bash
   rg -l '\bstore\.' temper/crates/temper-service/src | xargs sed -E -i \
     -e 's/\bstore\.(above|sources|component|ask|clock|next_deadline)\b/store.host.\1/g' \
     -e 's/\bstore\.secrets\b([^(]|$)/store.host.secrets\1/g'
   ```

   `cargo fmt`. A site the `sed` missed or wrongly took is a compile error: fix it by hand and name it in the report.
   `T/tests/fixtures.rs`'s `mem::swap` of `store.component` is one of its sites, and keeps working.
4. Checks, one at a time: `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-service --all-targets --
   -D warnings`; `heavy cargo nextest run -p temper-service`; `heavy cargo check -p temper --tests`; `heavy cargo nextest run -p
   temper-engine-domain-world -E 'test(switch)'`, which reads `WIRES`; the citations check.
5. D001's measure, per file, as `notes/service-parts.md` says.
6. Commit and push.

## Report

The common note's items, the measure by file, any site the `sed` got wrong, and net lines against the cap. No test is added.
