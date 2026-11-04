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
   - The accounts' host, the worker's side and the clients' side are being written.

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
- **D001's and D005's facts have moved.** The measure reads 1,388, not 1,369, and `Resources`' `Debug` is now hidden
  in both copies. `service-items` corrects what survives.

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

So far: `skein-concat`, then `smith-primitives`; `skein-startup-store`, then `smith-startup-writers`;
`github-fake-surface`, `github-machine-face`, `github-joint-face`, `github-close`.

## Overlaps

`fabrig-scale` changes none of the seven parts' files. It does change, beside them: `T/limits.rs`, `T/startup*`,
`T/tests/fixtures.rs`, `J/tests.rs` (the store's limits' figures) and `jig/crates/jig-protocol-store` (86 files, not
`host.rs` or `boundary.rs`), with `store.md`. Those are unavoidable for `jig-store-host` and `temper-store-host`,
which come late in their chains for that reason as well.
