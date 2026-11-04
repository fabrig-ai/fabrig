# debt-7: the plan

From `inputs/request.md`: three independent parts. `notes/common.md` holds the standing rules every
task names. This file grows with each release; what is here so far is what has been written.

## Parts and tasks

1. **smith-items** (`skein/docs/design/lib.md, 6`), slice 1. One task per smith item, as the request
   asks, plus what skein must offer first.
   - `tasks/smith-unused-deps.md`, medium, ~15 minutes, needs nothing: D014.
   - `tasks/smith-kind-text.md`, medium, ~20 minutes, needs nothing: D019.
   - `tasks/skein-concat.md`, medium, ~15 minutes, needs nothing: `skein_lib::bytes::concat`.
   - `tasks/smith-primitives.md`, medium, ~75 minutes, needs `skein-concat`: D016.
   - `tasks/skein-startup-store.md`, high, ~50 minutes, needs nothing: `skein_shell::store_file`.
   - `tasks/smith-startup-writers.md`, high, ~50 minutes, needs `skein-startup-store`: D009.

2. **jig D005 and temper D001** (`jig/docs/design/protocol/service.md, 11`). `notes/service-parts.md` holds the
   contracts. The parts' order is fixed there: each jig task moves a part into a product crate and `jig-test-service`
   adapts; each temper task deletes temper's copy. One writer each for `jig-test-service` and `temper-service`.
   - `tasks/skein-kernel-routes.md`, medium, ~30 minutes, needs nothing: `skein_io::kernel::Routes`.
   - `tasks/jig-accounts-host.md`, high, ~75 minutes, needs the routes: `jig_protocol_accounts::host`, net at most +130.
   - `tasks/temper-accounts-host.md`, high, ~55 minutes, needs the last: temper's copy goes, net at most -290.
   - `tasks/jig-worker-agents.md`, high, ~60 minutes, needs `jig-accounts-host`: the new crate, net at most +300.
   - `tasks/temper-worker-agents.md`, high, ~40 minutes, needs it and `temper-accounts-host`: net at most -420.
   - `tasks/jig-workers-host.md`, high, ~60 minutes, needs `jig-worker-agents`: net at most +120.
   - `tasks/temper-workers-host.md`, high, ~50 minutes, needs it and `temper-worker-agents`: net at most -200.
   - `tasks/jig-server-part.md`, high, ~120 minutes, needs `jig-workers-host`: net at most +170.
   - `tasks/temper-server-part.md`, medium, ~45 minutes, needs it and `temper-workers-host`: net at most -340.
   - `tasks/jig-store-host.md`, high, ~100 minutes, needs `jig-server-part`: net at most +120.
   - `tasks/temper-store-host.md`, medium, ~60 minutes, needs it and `temper-server-part`: net at most -90.
   - `tasks/jig-clients-host.md`, high, ~90 minutes, needs `jig-store-host`: net at most +80.
   - `tasks/temper-clients-host.md`, high, ~75 minutes, needs it and `temper-store-host`: net at most -200.
   - `tasks/service-items.md`, medium, ~30 minutes, needs the last two: D005 and D001 deleted, or rewritten.

   The service parts together are expected to end near -975 lines, and about -490 if every task only just meets its
   cap, from the families' estimates; the survey's -1,875 assumed more moved. Each task's cap is in its spec.

3. **fake-github** (`skein/docs/design/fake-forge.md, 10.6`), slice 3. `notes/fake-github.md` holds its contracts.
   - `tasks/github-fake-surface.md`, high, ~100 minutes, needs nothing: what the four faces build on, additions only.
   - `tasks/github-connection-face.md`, high, ~50 minutes, needs the surface: the step harness's face.
   - `tasks/github-machine-face.md`, high, ~100 minutes, needs the surface: the machine world's.
   - `tasks/github-protocol-face.md`, high, ~120 minutes, needs the surface: the protocol world's.
   - `tasks/github-fixture-readers.md`, medium, ~50 minutes, needs the surface: the fixture's other readers.
   - `tasks/github-joint-face.md`, high, ~90 minutes, needs `github-machine-face`: the joint world's.
   - `tasks/github-close.md`, medium, ~15 minutes, needs the five: pays D029.

   Each face's task has its net-lines cap. Their sum with the surface's +520 is at most -430.

## What the cut decided

- **D016's `join` is paid here, by a separate skein task.** The item says it waits on skein's D007, and
  its **Paid when** needs smith's `join` gone. `skein-concat` adds `bytes::concat` alone: a branch stays
  in its tree, and a missing generic mechanism goes into skein first. D007 stays open, its copies
  untouched, and its text stops citing smith's D016. So the smith items are one task each, as the request
  says, and each skein mechanism they need is one more.
- **D014 has seven lines, not six.** `smith-local-service`'s dev-dependency on itself turns on its
  `fixture` feature, and has no comment. It stays, with one.
- **D014's check is not built.** The item's **Paid when** does not ask for it.
- **D016's measure misses longer receivers:** 90 against 105 by a wider search. The spec holds both.

- **D009's call takes an open root.** The two sites differ: credentials open their directory through io's
  private-file checks, while the generated configuration's directory may be any directory a person named,
  so a private check there would refuse what runs today. `store_file` serves both, with
  `StoreDirectory::{Root, Private}` (`skein/docs/design/shell.md`, 6.7). The caller opens the root, so the
  errors both sites answer stay as they are.
- **D009 adds lines.** A skein host and its first tests, against two copies smith loses, and the item asks for
  a new cleanup host too: about +190 in skein, at most -50 in smith, both capped in their specs. Part
  smith-items may not end below its start, against the request's aim. The tasks' reports give the sum.

- **The whole of D005 is cut, in the order the code allows, with every part one chain.** jig and temper each move a
  part at a time. The last task, `service-items`, reads D001's measure and either deletes both items or rewrites
  them to say what's left, as the request allows. The clients' part decides whether the measure falls below 200.
- **The accounts' host goes first after all.** The survey says it needs the store's core moved first, because
  `iterate` takes `&mut Store`. A pull breaks that: the host hands out its store requests, as
  `jig_protocol_store::host::Host` hands out submissions, and the adapter calls its own `store.secrets`.
- **The new smith-linked crate is `jig-protocol-worker-agents`,** and `jig-protocol-worker-link` stays bytes: an
  application whose workers run no agents leaves the new crate out. It widens `protocol/README.md`, 5's rule.
- **The server goes in `jig-protocol-engine-clients-http`,** though it names no jig type and skein's rule points to
  skein: the request and D005 say jig ships it, and skein's `D016-transport-four-copies.md` is where a generic one
  would come from.
- **Behaviour differences stay differences:** `notes/service-parts.md` lists each, and how both sides keep theirs.
  One carries a question for the user (`wrong-secret-kind`).
- **The accounts' pass is `begin`, then a bounded loop over `output`.** `begin` keeps the order in jig, as
  `jig/AGENTS.md` requires, and `for _ in 0..count` stands in for `while`, which step code can't use. temper's
  one-slot hold is `Accounts::hold`, which only temper calls. temper's 15 host tests stay as composition tests.
- **`jig_protocol_config::Files` holds the shell's files;** `Seeds` doesn't move. `jig/tests/boundary` pins what these
  crates may link, which the survey missed, so each task that adds a dependency edits that list.
- **`Evidence` is a value of `route::decode`:** with no callback, the shared `decode` reaches either form, and a
  start's failures stay in their order. jig's raw form ships beside temper's decoder.
- **`Workers::new` takes `&config::Engine` and `&config::Files`,** so the digest and listener checks leave both services.
  `jig-protocol-engine-workers` links configuration for it, and `jig/tests/boundary` pins that: its exact list changes,
  and so does `smith_breaches` for the new crate. The survey was wrong that nothing checks which jig crates link smith.
- **Tests stay.** Only `J/tests/program_sources.rs`' three and the evidence test move; `T/tests/workers.rs`' 26 stay.
- **The server's `Debug` counts, an addition:** both copies print heads and cookies, and the shipped type is a
  credential-holder in that sense. Each server task runs its browser tier once by hand, as the gate doesn't.
- **A peer's end is `Limits::peer_end`:** `PeerEnd::Connection` in temper, `PeerEnd::Call` in jig, each as today.
- **The store's core is `jig_protocol_store::host::Hosted`, by a field path change, not accessors.** About 266 lines
  change by one `sed` in the two services. An accessor would touch the same lines and hide nothing, and tests push fake
  terminals at some 60 sites, which `cfg(test)` can't reach across crates.
- **The clients' pass stays in each service.** temper's google and provider code has its places in that order, and
  moving it isn't worth the risk to reach a number. The bridge and the endings ship.
- **The settings go to `jig_protocol_config::clients`, and the printed link to `jig-protocol-engine-clients-http`.** The
  clients' crate can't read configuration, and it speaks no HTTP. The designer kept the link in `host`, as the lead's
  table said; the lead moved it, as `README.md`, 5 says "no HTTP" of that crate.
- **D001's bound will most likely not be reached.** The family's estimate is about 330 against the bound of 200,
  though the survey's best is 192: the adapters each service keeps share lines by design, and the clients' pass is
  about 150 of it. So `service-items` most likely rewrites both items, as the request allows. Nothing aims at the
  number.
- **D001's and D005's facts have moved.** The measure reads 1,388, not 1,369, and `Resources`' `Debug` is now hidden
  in both copies. `service-items` corrects what survives.

- **A citation this plan found and left:** `jig/docs/design/domain/root.md`, 5 names
  `tests::accounts::an_early_account_request_waits_for_the_previous_moves_dispatch`, a test `47494b7f7a` deleted.
  `notes/service-parts.md` lists it too; there is nothing to keep. It predates this plan.
- **D029's fixture is built once in skein, not across trees.** temper's four fixture builders stay: the item is
  skein's and they speak no HTTP. A scenario's own tables, such as the conformance's, stay its own too.
- **D029 ends near -600 lines, not the review's -1.2k.** The faces' gross fall is about that, but the surface and the
  fixture's literals stay in the fake.
- **Two things change below D029's tests, which none reads:** the joint world meets rate fields and `ETag`, so
  `Answered.rate` becomes `Some` and a log poll may answer `304`; the protocol world sends real media types and
  `Location`. Each face's spec checks its suite files unchanged, its counts equal to base, and a run compared with
  base. A seeded sweep going red stops and asks.
- **One test helper line moves:** `github_peer.rs`'s `request` gains a `Host` line, since the real server refuses a
  request without one. No assertion moves. `notes/fake-github.md` states this departure from the common note.

## Critical path

`skein-kernel-routes`, `jig-accounts-host`, `jig-worker-agents`, `jig-workers-host`, `jig-server-part`, `jig-store-host`,
`jig-clients-host`, `temper-clients-host`, `service-items`: about 640 implementer-minutes. `jig-test-service` takes one
writer at a time, and that sets it. temper's chain runs beside it, each task after its jig task.

Beside it: `github-fake-surface`, `github-machine-face`, `github-joint-face`, `github-close`, about 305 minutes;
`skein-concat`, then `smith-primitives`; `skein-startup-store`, then `smith-startup-writers`.

## Overlaps

`fabrig-scale` changes none of the seven parts' files. It does change, beside them: `T/limits.rs`, `T/startup*`,
`T/tests/fixtures.rs`, `J/tests.rs` (the store's limits' figures) and `jig/crates/jig-protocol-store` (86 files, not
`host.rs` or `boundary.rs`), with `store.md`. Those are unavoidable for `jig-store-host` and `temper-store-host`,
which come late in their chains for that reason as well. Their `sed` crosses 38 files, tests among them: a
`fabrig-scale` test that reads `store.above` after they land is one token's compile error, for whichever merges second.
Inside this plan the overlaps are all serial on the chains: `jig/tests/boundary`, both status files, `T/engine.rs`,
`T/system.rs`, `J/engine.rs`, `J/lib.rs`.
