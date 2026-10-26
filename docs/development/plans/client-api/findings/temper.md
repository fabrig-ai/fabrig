# temper on the new jig, at the base

The code audit of temper at `c7095516` (`origin/main`), against jig at
`d91a89c4` (temper's pin) and `a3b53546` (jig's main), 2026-10-15, by a
read-only sub-agent.

## 1. The repin probe

`cargo update -p jig-domain-core --precise a3b53546…`, in a
scratch worktree of temper, failed (exit 101, 433 s): "no matching
package named `jig-fake-person`", required by `tests/one-process`
(skein-ui moved it to skein's `skein-fake-person`). `Cargo.lock` was
not changed, and the skein and smith updates and `cargo check` were not
run. Of temper's 43 jig packages, only `jig-fake-person` is missing at
jig's main: 4 `Cargo.toml` files and 14 lines in 6 files name it.

The errors the repin will meet, predicted from the source, not
measured: jig-view's tree types now skein-ui's (about 150 uses: `Tree`
36, `Role` 33, `Builder` 15, `Act` 13, `State` 11, `Limits` 10, `NodeId`
9, `Run` 7, others 16); `jig_fake_person` unresolved; four `Limits`
literals without `scene_marks` and `scene_bytes`; `Reply::SignOut`
without `sign_in` and a non-exhaustive `Incoming`
(`tests/client/src/front.rs`); `State::Provisional` dropped. These are
skein-ui's `later.md` rows for temper's next repin.

## 2. temper's uses of jig's client crates (files/lines)

- `jig_domain_client`: client-domain 12/90, client-view 6/26,
  shell-terminal 8/36, temper 1/5, tests/client 6/24, one-process 4/9.
- `jig_view`: client-view 10/47, shell-terminal 10/66, temper 2/6,
  tests/client 4/18, one-process 2/6.
- `jig_test_domain_client`: tests/client 1/2.
- `jig_shell_native`: shell-terminal 6/12, temper 1/1, one-process 1/1.
- `jig_protocol_client_link`: shell-terminal 7/7, one-process 2/3.
- `jig_protocol_client_wire`: service 6/8, shell-terminal 2/2, codecs
  1/1, one-process 2/2.
- `jig_client_world`: shell-terminal 1/10, tests/client 8/16.
- `jig_fake_person`: temper 1/3, tests/client 3/6, one-process 2/5.
- `jig_protocol_engine_clients`: service 9/14.
- `jig_protocol_config`: service 30/58, temper 6/13, engine 2/2,
  one-process 1/7, system 1/1.

## 3. temper's client crates (production lines / all lines)

- `temper-client-domain` 1,241/1,830. Its root (lib, domain, route,
  parts, store, owner, boundary, address, limits: 615 lines) copies
  `ops`'s client root (484 of 593 non-blank lines match); the forge's
  side (forge, form, vocabulary: 644) is temper's.
- `temper-client-view` 1,163/1,875. `root.rs` and `limits.rs` (167) copy
  `ops`'s view root; `root.rs` calls `jig_view::jig` and `after_page`, so
  every page it shows is jig's reference UI. The forge's modules (758)
  are temper's.
- `temper-shell-terminal` 2,484/4,280 and 14 golden files. Its native
  client (`client.rs`, `route.rs`, `route/family.rs`,
  `terminal/native.rs`, `terminal/run.rs`: 1,227) follows jig's pattern
  on `jig_shell_native::Shell`; `render.rs` and `command.rs` (741) are
  temper's own printer, which skein's `skein-ui-text` replaces.
- `temper-protocol-client-wire` 1,731/1,887 (1,665 generated).
- `tests/client` 1,380/1,609 (front 375, translate 494, fixtures 290,
  app 216).

## 4. The engine side

- `chats` is written by hand only at
  `crates/temper-service/src/tests/fixtures.rs:46`; the other
  configurations derive from jig's `engine_document()` fixture. Core
  `Settings { chat: 1, close_grace }` is built by hand at
  `tests/engine/domain/src/walking.rs:252`.
- The clients' listener: `crates/temper-service/src/clients.rs` (498
  lines) passes `config.clients.listener` (line 51) to `kit::Settings`
  and reads `ProviderKind` (55-60); `crates/temper/src/host.rs:57`
  (`Stop::Listener`); two test configurations
  (`crates/temper/tests/support/configuration.rs:27`, `fixtures.rs:48`).
- Shape-specific requests in production code:
  `crates/temper-engine-domain/src/store.rs:528-584` (an exhaustive
  match on `people::Ask`: `SetGoal`, `Prioritise`, `StartChat`,
  `Close{disposals}`, `ReadChat`); `archive.rs:211-213` and
  `boundary.rs:582-583` (`SeenChat`, `ForgetChat`: an ended chat erased
  once read, which step 2 removes); `restart.rs:427-429` (the
  conversation and goal keys);
  `temper-protocol-store/src/records/application/refusal.rs:292, 328`
  (`Disposal`). Tests: 45 uses in 20 files (engine 33, system 4,
  conformance 4, service 4, client 1).
- `crates/temper-service/src/family/refusals.rs`: 230 lines on
  `jig_protocol_client_wire::v7`, called once (`family.rs:141`); the v7
  wire also at `family.rs:105`, the service's `tests/clients/peer.rs`,
  and the one-process referee (355 lines). All break at step 3.

## 5. What temper must build to pass its gate on the new jig

- `client.md` 2, `protocol.md` 3 and 5: the crates' renames.
- `client.md` 3.1 and 3.3: the root on `jig-client`, two owners, routing
  by asker (`jig-domain-client`'s `Domain`, under the copied root, goes).
- 4.5, 4.6: the forge's request as bytes through `jig-client`'s request
  table, `Owner` in the wire crate (4.4 and 4.7 are built).
- 5, 5.1: the view on `skein-ui`.
- 6.2, 6.6, 6.7 and 7, `credentials.md` 6: the terminal on
  `skein-ui-text` over `jig-client-shell-native`, the credential kept by
  the host and given back as `Held` (the native shell keeps none after
  step 4).
- 8, `protocol.md` 6, `configuration.md` 3: `temper-service` owning the
  listener, `application.clients` (step 5 takes the listener out of
  jig's component; without it no client connects and the one-process
  and process tiers fail).
- `configuration.md` 2: `templates` (step 2); 7 (`documents.lines`) only
  if the task line refuses the forge's deepest path, not checked.
- 3.6 for the acts the present stories make (jig's check needs revisions
  named; chats made with `shapes::chat`).
- 3.5: at least Inbox, Chats, Chat, Task (a held task released) and
  Adopt, the pages the stories of `tests/client`, the one-process world
  and the process tier use.
- `domain/testing.md` 2.5, 2.6 and 6.3: the worlds on `jig-client-world` and
  skein's fake person.

Can wait: Board (and reordering), Changes, the agent's task page,
confirmations no story uses, the rebound-Host story (domain/testing.md, 2.6),
the browser, `Mint` and the forge as a sign-in provider.

temper cannot keep its present pages on jig's helpers without
`jig-client-view`: they are jig's reference UI (jig-view, 26,047
production lines at jig's main, and the page half, about 7,000). The
mockup pages above are about 2,800 to 3,500 production lines; Board,
Changes and the agent's task about 900 more.

## 6. Order

- On jig's main now: temper's two renames; the skein-ui repin (the fake
  person, the tree's types, `Limits`, the front); the printer on
  `skein-ui-text` (-741/+150); `Owner` in the wire crate.
- After jig's steps 1 to 3: the step-1 renames (about 180 lines); the
  engine on step 2 (`store.rs`, `archive.rs`, `restart.rs`, templates,
  tests creating from templates); the wire on step 3 (`family.rs`,
  `refusals.rs`, the one-process referee).
- After step 4: the root on `jig-client` with the forge as bytes; the
  terminal on `jig-client-shell-native` with `Held` and the token's
  file; `tests/client` on `jig-client-world` with jig's check. This does
  not fit increments of 300 lines unless temper follows jig's step-4
  increments one by one, or temper's view uses `jig-client-view` while
  its pages are replaced (about 450 lines at the repin).
- After step 5: `temper-service` owns the listener and mounts the
  mapping under `/jig` with the guards, the link's `GET` not found;
  `application.clients` at start; the one-process and process tiers
  through the mapping.

## 7. Found by the plan's rehearsal of temper 01.1

The planner's rehearsal of the repin onto jig's base (the three `cargo
update` commands, then `cargo check --workspace --all-targets` with
`--keep-going`: 50, then 18, 13, 2 and 0 errors as each was fixed) met
three things this audit did not list: temper's three generated codecs,
rewritten by skein's newer generator (their goldens unchanged); smith's
model `effort`, the agent configuration's `prices` and a 119-byte
charter; skein-ui's `Read::Arrived`, the forge's page wrapped in `Main`,
a task's sections shown as tabs, and a check taking `Toggle` only.
temper 01.1's file names each, with the tests that failed (10 of 85 in
the client crates, 17 of 444 on the smith side, 1 of 17 in the binary's
and one-process tests).
