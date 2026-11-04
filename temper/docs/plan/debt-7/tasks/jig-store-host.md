+++
id = "jig-store-host"
part = "service-clients"
kind = "migration"
needs = ["jig-server-part"]
files = ["jig/crates/jig-protocol-store/src/host.rs", "jig/crates/jig-protocol-store/src/host/tests.rs", "jig/crates/jig-protocol-store/src/sizes.rs", "jig/testing/jig-test-service/src/**", "jig/docs/design/protocol/service.status.md"]
effort = "high"
minutes = 100
+++

# jig: the store's host as product code

## Goal

`J/store.rs` and `T/store.rs` (`J` and `T` are the paths `notes/service-parts.md` gives) wrap the same component, drivers
and queues the same way. That core moves to a new type, `jig_protocol_store::host::Hosted`, which `J`'s `Store` owns as
its `host` field. The adapter keeps its routes, its token tagging and its dispatch. `temper-store-host` does the same.

**The call this task carries: a field path change, not accessors.** `component`, `above`, `secrets` and `sources` become
public fields of `Hosted`, and their readers read `store.host.above`. An accessor changes the same 266 lines in the two
services, adds a forwarder for each field, and hides nothing: a `&mut Queue` from a method is a public field with more
syntax. Tests push fake terminals into `above` at some 60 sites, so a pull-only surface would need public methods only
tests call, which `cfg(test)` can't give across crates. `Deref` is out. So the path changes, by one `sed`, and the
compiler checks every site.

Acceptance:

- `jig_protocol_store::host::{Hosted, NotReady}` exist with the outer layer below, and `jig_protocol_store::sizes::fill`.
- `Host`, `Lane`, `Submission` keep their surface: `jig/testing/jig-test-store-files/src/disk.rs` reads them unchanged.
- `J/store.rs`'s `Store` holds `host`, `routes`, `restoring`, `routed`, `submitted`, `opened`, `pending`, `installation`
  and its `cfg(test)` `models`. `ask`, `secrets`, `clock` and `next_deadline` are gone from it.
- Every test passes, changed only in the path of a moved field or method. `tests::engine::the_store_root_closes_after_delayed_driver_completions`
  keeps its name and module.
- `jig/docs/design/protocol/service.status.md` no longer lists the store's host.
- **Size cap:** the branch's net lines are at most +120, tests included: about 170 lines ship, a new test takes 40, and `J/store.rs`
  falls by about 90. `temper-store-host` then falls by about 100. Measure it, and stop and ask if it's passed.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `temper/docs/plan/debt-7/notes/service-parts.md`, `AGENTS.md`, `jig/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11 and 11.5, and 5 for what closing owes.
- `jig/docs/design/protocol/store.md`, 9 and 13. It's read by tests: don't change it.
- `temper/crates/temper-service/src/store.rs`, for what the shape must also serve.

## Owns and may change

- Owns: `jig-protocol-store`'s `src/host.rs`, `src/host/tests.rs`, `fill` in `src/sizes.rs`; `J/store.rs`.
- May change, in the way listed: every `.rs` under `J` by the one `sed` of step 4; `J/tests/**` by that `sed` alone;
  `service.status.md` as step 6 says, which departs from the implementer's text for that file alone.
- The readers, by file, at this plan's base. Each is one token on one line, by the `sed`; counts are lines:

  | Where | Lines |
  | --- | --- |
  | `J/engine.rs` 6, `J/accounts.rs` 5, `J/clients.rs` 4, `J/lib.rs` 2 | 17 |
  | `J/store.rs`'s free functions `answer`, `answer_routed`, `request` | by the same `sed` |
  | `J/tests/engine.rs` 34, `J/tests/accounts.rs` 10, `J/tests/fixtures.rs` 8, `J/tests/first_period.rs` 6, `J/tests/clients.rs` 5, `J/tests/worker.rs` 2 | 65 |

  `jig-accounts-host` and the worker's tasks have moved some of these lines since: the `sed` sweeps them wherever they are.
- Not yours: `T`, and `store.md`. Any `pub` of `Host` you'd want to change: stop and ask.
- If a test reads `store.env`: there is none at base; `Hosted::limits` is the read.

## What differs, and which holds

From `notes/service-parts.md` and the survey, as it falls here. Every line keeps both services' behaviour.

- **`Store::secrets`' immediate set:** `NotReady`, a value at `new`. jig's is `NotReady::InPass`, so `SecretsNotReady`
  is answered in the pass; temper's is `NotReady::Above`.
- **Seed:** a value at `new`. jig passes its parameter.
- **Wires:** the owner calls `host.component.wires(..)` after `new`. jig doesn't.
- **Start walk guard (`restoring`):** stays in jig's `down`.
- **Admission and page size:** stay in each `limits`. `fill` holds only what both write alike; `kv.page.bytes` differs
  and stays out of it.
- **`joins`, the route's table:** stay each owner's.
- **`Lane` and `Submission`'s tagging:** stays the owner's. `publish` and `complete` stay in `J/store.rs`.

`Store`'s `Debug` stays derived. `Hosted`'s is hand-written, because a terminal can hold a kept digest: `above` holds
`SecretValue::Digest { digest }`, a plain array. No test reads either.

## Steps

1. `sizes.rs`:

   ```rust
   /// What the KV limits take from checked sizes (store.md, 5.4); the page's bytes are the owner's.
   #[must_use]
   pub fn fill(kv: &mut skein_kv::Limits, sizes: &Sizes) -> Option<()>;
   ```

   Its body is `J/store.rs`'s `limits` from `kv.key = sizes.key;` through `kv.chunk = ..`, with `?` as there.

2. `host.rs`, beside `Host`, whose body doesn't change. The module doc cites `jig/docs/design/protocol/service.md, 11.5` too.

   ```rust
   /// Whether a get the store can't answer yet is answered in the pass (protocol/service.md, 11.5).
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub enum NotReady {
       /// `Hosted::secrets` returns it, as any answer to a get.
       InPass,
       /// It stays in `above`, for the owner's dispatch.
       Above,
   }

   pub struct Hosted {
       pub component: crate::Store,
       /// The store's terminals, in order, for the owner's dispatch.
       pub above: Queue<crate::Event>,
       /// Terminals the owner's dispatch hands its clients' side.
       pub secrets: Queue<crate::Event>,
       /// The answers to the accounts' records' loads and writes (store.md, 9).
       pub sources: Queue<crate::Event>,
       env: Env<crate::Limits>,
       drivers: Host,
       not_ready: NotReady,
       closing: bool,
   }

   impl Hosted {
       pub fn new(
           limits: &crate::Limits,
           application: crate::sizes::Application,
           models: &[jig_domain_core::Model],
           seed: u64,
           not_ready: NotReady,
       ) -> Self;
       pub fn limits(&self) -> &crate::Limits;
       pub fn clock(&mut self, now: Time, wall: Wall);
       pub fn adopt(&mut self, fd: Fd, user: u32, installation: [u8; 16]) -> Result<(), Fd>;
       pub fn ask(&mut self, request: crate::Request);
       /// Get answers stay in this pass; secret writes' terminals are handed to their owner.
       pub fn secrets(&mut self, owner: Token, request: crate::Secrets) -> Option<crate::Event>;
       pub fn iterate(&mut self);
       pub fn complete(&mut self, lane: Lane, complete: Complete);
       pub fn submissions(&self) -> u32;
       pub fn pop_submission(&mut self) -> Option<Submission>;
       pub fn next_deadline(&self) -> Option<Time>;
       pub fn work_pending(&self, now: Time) -> bool;
       pub fn closing(&self) -> bool;
       pub fn idle(&self) -> bool;
       pub fn closed(&self) -> bool;
       pub fn close(&mut self);
   }
   ```

   The bodies are `J/store.rs`'s, in this mapping:
   - `new`: `Host::new(limits, seed)`, then `crate::Store::on_directory(limits, application, models, drivers.seeded())`.
   - `adopt`: `drivers.adopt(fd, user, Token::new(1), installation)`.
   - `ask` and `secrets`: as they stand, `ask` calling `Hosted::secrets` for its `Secrets` arm. `secrets`' scan takes
     `SecretsNotReady` only under `NotReady::InPass`.
   - `work_pending`: `secrets`, `sources` and `above` not empty, or the drivers'. `idle`: those three empty and the drivers'
     `idle`. `closed`: the drivers' `closed`.
   - `close`: sets `closing`, closes the drivers, and drains those three.

   `Debug`, hand-written, with a comment saying why: `above`, `secrets` and `sources` as their `len()`, and `closing`.
   No `component` and no `drivers`.

3. `J/store.rs`. `new` keeps its routes and builds `Hosted`. `limits` calls `fill`. Then:

   ```rust
   pub(super) fn adopt(&mut self, fd: Fd, user: u32) -> Result<(), Fd> {
       self.host.adopt(fd, user, self.installation)
   }
   pub(super) fn work_pending(&self, now: Time) -> bool {
       self.host.work_pending(now) || !self.routed.is_empty() || !self.submitted.is_empty()
   }
   pub(super) fn closed(&self) -> bool {
       self.host.closed() && self.idle()
   }
   pub(super) fn idle(&self) -> bool {
       (self.host.closing() || self.routes.idle())
           && self.routed.is_empty()
           && self.submitted.is_empty()
           && self.host.idle()
   }
   pub(super) fn close(&mut self) {
       self.host.close();
       for _ in 0..self.routed.len() {
           let _up = self.routed.pop();
       }
   }
   pub(super) fn iterate(&mut self) {
       self.host.iterate();
       self.publish();
   }
   ```

   `down` calls `self.host.ask(request)`. `complete` and `publish` stay, on `self.host`.

4. The path change, its own commit, from the repository's root:

   ```bash
   rg -l '\bstore\.' jig/testing/jig-test-service/src | xargs sed -E -i \
     's/\bstore\.(above|sources|component|secrets|ask|clock|next_deadline)\b/store.host.\1/g'
   ```

   `cargo fmt`. A site the `sed` missed or wrongly took is a compile error: fix it by hand and name it in the report.
5. Tests in `jig/crates/jig-protocol-store/src/host/tests.rs`, over `fixtures::store_limits()` and `fixtures::APPLICATION`:
   - `a_get_not_ready_follows_its_value`: on a `Hosted` not marked, a `Secrets::Get`. `NotReady::InPass`: `secrets` returns
     `SecretsNotReady` and `above` is empty. `NotReady::Above`: it returns `None` and `above` holds it. Break: the scan
     ignoring `not_ready`.
   - `debug_of_the_hosted_store_counts_its_terminals`: an `Event::Secret` holding `SecretValue::Digest { digest: [171; 32], .. }`
     pushed to `above`; `{hosted:?}` shows `above: 1` and no `171`. Break: a `derive(Debug)`.
6. `service.status.md`: remove `the store's host` from **Not yet**, 11; date line.
7. Checks, one at a time: `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-store -p
   jig-test-service --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-protocol-store -p jig-test-store-files -p
   jig-test-service -p jig-test -p jig-store-world -p jig-one-process-world -p jig-channel-world -p jig-engine-clients-world
   -p jig-client-protocol-world -p jig-ops-world`; `heavy cargo check -p temper-service --tests`; the citations check.
8. Commit and push.

## Report

The common note's items, with the two tests' breaks, any site the `sed` got wrong, and net lines against the cap.
