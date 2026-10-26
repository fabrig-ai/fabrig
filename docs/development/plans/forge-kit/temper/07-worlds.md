# Session 07: the moved worlds and the conformance deleted

Delete from temper what skein 06 to 09 moved: the machine and listener parts
and the five `fake_protocol` tests of the protocol world, the client's world,
setup's reconcile world, the dialect's document tests with their helper, and
`tools/forgejo-conformance` with its transcripts, deliveries and `facts.txt`.
The protocol world stays, with the 10 tests of temper's own policy and the
fixtures they use, now on skein's component world (decision 22). Nothing is
rewritten: each deleted part lives on in skein under its names, and what stays
changes only its imports and manifests.

**Needs at start:** skein 06.3 and temper 06.1 (README). By increment: 07.1
needs skein 07.1 (the client's world), 07.3 (setup's), 08.1 (the machine
world) and 09.1 (the protocol world); 07.2 needs skein 06.2 and 06.3 (the
recorder with its corpus, and the document tests) and temper 06.1 (it
repointed the conformance, which this session deletes); 07.1 also needs
**temper 05.2** (README), which deletes `tests/one-process/src/forge.rs`,
the one user of the protocol world outside the worlds (Files). Each capability must be on origin/main; skein's later
retained test increments (06.4, 07.2, 08.2 to 08.4) may land after
this session (README, Repin).

## Read first

- skein `docs/design/forge.md` 2.3 and 11, `forge-connection.md` 10,
  `forge-client.md` 10, `forge-setup.md` 7, `fake-forge.md` 7; temper
  `docs/design/domain/testing.md` 2.2 and 7.
- skein `docs/foundation/testing-strategy.md` 8 and `development.md` 5.5-5.7.
- README (sessions, Rules, Repin, Done when); code-organization 1; decisions
  11, 22 and 23.
- `skein/06-conformance.md` 06.3 (Removes), `skein/07-worlds.md` 07.1,
  `skein/08-machine-world.md` 08.1, `skein/09-protocol-world.md` 09.1 (Don't);
  findings `survey-06-tests.md`, `base-runs.md`, `base-temper-07.md`.

## What the design asks

- forge.md 2.3: the connector's worlds' stories and cost ceilings stay the
  service's and run on the kit's fakes; forge.md 11 and forge-connection.md
  10: the dialect's document tests, the machine and listener worlds and the
  protocol world's invariants are skein's, under `tests/forge/`.
- forge-client.md 10 and forge-setup.md 7: the client's world and reconcile's
  world are skein's. fake-forge.md 7: the conformance is skein's,
  `tests/forge/conformance`, outside the suites.
- domain/testing.md 2.2 and 7: temper keeps `tests/engine/protocol-forge`, "over
  skein's crates", for what translation adds in temper.
- domain/testing.md 2.2 also asks that world to take the fake forge's deliveries to
  the engine's listener, signed, late, out of order or never: unbuilt at the
  base (`tests/protocol.rs` injects `Event::Hint`) and here (`later.md`).

## Choices this plan makes

Counts and Files describe the base, dd12b8e; each step edits origin/main as it
stands at its start, where temper 01 to 06 have renamed imports and changed
counts. Counts below are written "the count at the step's start minus N".
The current conformance count is35: base25, the unset-variable refusal,
and nine additions from06.2–06.4 including06.3's diagnostic test. Refresh
further reached count copies and subtractions from the actual step-start
listings and name their users in the commit body; historical base examples
keep their original counts.

- **Two increments, readers before the corpus.** The conformance's
  transcripts and deliveries are read by `tests/engine/protocol-forge`
  (`src/machine.rs`, `tests/hooks.rs`) and by `tests/codecs/src/forgejo.rs`
  (3 reads). 07.1 deletes the first two with the protocol parts, and with
  reconcile's world the other user of `forgejo.rs` (its `src/exchange.rs`
  calls `decode_answer`); 07.2 deletes `forgejo.rs`, then the tool, so no step
  leaves a reader without its corpus (skein 06.3: "after their remaining
  readers have moved").
- **`translate.rs` stays temper's, where its other user is.**
  `tests/engine/forge/src/lib.rs` includes `tests/engine/forge/client/src/translate.rs`
  by `#[path]`; skein 07.1 left "the translation-path user" to this session.
  `git mv` puts the file at `tests/engine/forge/src/translate.rs`, unchanged,
  and the `#[path]` line goes. Its `KEY` is `<!-- temper:key `, the word the
  connector under test reads; skein's copy is on `kit`.
- **The protocol world keeps its fixtures.** The 10 tests reach `limits`,
  `protocol_config`, `writer_settings`, `protocol_shared`, `goal_view`,
  `token_absent` and `TOKEN`, which call `shared`, `settings`, `WRITER` and
  `FORGE`. Skein 08.1 copied the rest; those no kept test reaches are deleted
  (90 lines). Nothing in `temper-engine-forge-world` was used only by the deleted
  parts: its items (`REPO`, `LIMITS`, `World`, `fake_config`, `translate`,
  `changes`) have users in `tests/system`, `tests/engine/domain` and
  `tests/conformance`.
- **`tests/codecs/tests/fixtures.rs` goes with its one reader**,
  `forgejo_setup.rs` (`mod fixtures;`, the only one in codecs): skein 06.3
  copies it to `crates/skein-forge/tests/fixtures.rs`, so 07.2 deletes it here.
- **Seams, not a second Joint.** `protocol.rs` reads `Joint`'s fields and
  calls from skein's crate (08.1 makes them public); no alias, no wrapper.
- **No process tier.** Nothing `-p temper` builds changes: the five deleted or
  trimmed worlds and the tool are not in its dependency tree (Proof). The tier
  runs at the plan's end (README, Done when 6).

## Rules that bind it

- 07.1's **first commit** is its repin: `cargo update -p skein-lib`, then
  `cargo check --workspace --all-targets`. Repair in that commit what
  skein's steps since temper's last repin broke. A lock conflict takes
  origin/main's Cargo.lock and repeats the update. 07.2 uses no new skein
  step, so no repin.
- Hotspots: `Cargo.toml` and `Cargo.lock` (every temper session). No alias, no
  `pub use` of a skein item: every `use` names the skein crate. Test builds stay
  unoptimized; a moved test keeps its name, budget and time. Temper's suites
  take no more than at the base, by package.

## Increments

### 07.1 The repin; the protocol world trimmed; the client's and reconcile's worlds deleted

Scope: forge-connection 10, forge-client 10, forge-setup 7; repin, delete what
skein 07.1, 07.3, 08.1 and 09.1 moved, repoint what stays.
Needs: skein 07.1, 07.3, 08.1, 09.1; temper 05.2 (above).
Files (at the base):
- `Cargo.lock`: the repin commit, then the entries `cargo check` writes.
- `Cargo.toml`: delete members `"tests/engine/forge/client"` and
  `"tests/forge-setup/reconcile"`; `[workspace.dependencies]` gains
  `skein-forge-machine-world = { git = "https://github.com/fabrig-dev/skein.git" }`.
- Moved: `tests/engine/forge/client/src/translate.rs` (567 lines) ->
  `tests/engine/forge/src/translate.rs`; changed `tests/engine/forge/src/lib.rs`
  (the `#[path = "../client/src/translate.rs"]` line deleted).
- Changed, `tests/engine/protocol-forge/`: `Cargo.toml` (`skein-forge-machine-world.workspace
  = true` in; out, the lines nothing kept reads: at the base `skein-http`,
  `skein-json`, `temper-forge-forgejo`, `skein-world`, `temper-service`, each
  checked with `rg` in the kept files under the names 01 to 05 left); `src/lib.rs` (the five
  `mod` lines); `src/fixtures.rs` (prune: `REPO`, `response`, `branch`,
  `fake_limits`, `hook_limits`, `HOOK_SECRET`, `PING`, `hook_settings`,
  `hooked_protocol_config`, `push_hook`, and the `api` import: 252 -> 163 lines);
  `src/protocol.rs` (head); `tests/protocol.rs`, `tests/fuzzy_protocol.rs` (imports).
- `tests/one-process`: nothing; temper 05.2 deleted its `src/forge.rs` and its
  `temper-engine-protocol-forge-world` line.
- Deleted (Removes): `tests/engine/forge/client/` but `translate.rs`
  (`Cargo.toml`, `src/{lib,referee,world}.rs`, `tests/{fuzzy_simulation,job_answer,
  locate,memory,simulation}.rs`); `tests/forge-setup/reconcile/` whole;
  `tests/engine/protocol-forge/src/{hooks,joint,machine,peer,pipe}.rs` and
  `tests/{fake_protocol,fuzzy_hooks,fuzzy_machine,hooks,machine}.rs`.
Shape (compiled in a scratch worktree of the base, with a stand-in crate for
  skein 08.1's, its `Joint` seams public; clippy `-D warnings` clean):
```rust
// tests/engine/protocol-forge/src/protocol.rs, head and Protocol::new
use crate::fixtures;
use skein_forge_machine_world::joint::{self, Joint, REPO};   // was: pub use crate::joint::REPO; use crate::{fixtures, joint::Joint};
//   ... Joint::with(.., joint::checks(1000))                // was: crate::joint::checks(1000)
// tests/protocol.rs and tests/fuzzy_protocol.rs
use skein_forge_machine_world::joint::REPO;
use temper_engine_protocol_forge_world::{fixtures, protocol::Protocol};   // was: ::{fixtures, protocol::{Protocol, REPO}}
```
  `protocol.rs` reads `Joint`'s `component`, `env`, `above`, `below`,
  `fake_env`, `answers` (08.1 makes them public), `domain` and `observations`
  (already), and calls `busy`, `time`, `replace` (08.1), `checks` (08.1,
  `#[must_use]`), `with`, `call`, `tick`, `tally`. The `pub use` of `REPO` is
  removed: its three users name skein's. Nothing is handed to the kit.
Test shape: no new test. The 10 tests of temper's policy stay, on skein's
  component and fakes (decision 22): `tests/protocol.rs` (9)
  `a_change_produced_opened_checked_queued_and_landed`,
  `a_created_issue_with_its_answer_lost_is_found_after_restart`,
  `a_merge_with_its_answer_lost_is_found_after_restart_without_merging_twice`,
  `a_failed_ci_head_is_repaired_then_checked_again`,
  `an_external_move_of_a_held_branch_is_reported_as_drift`,
  `idle_forge_call_cost_does_not_follow_old_issue_history`,
  `honest_costs_equal_the_requests_the_fake_served`,
  `a_busy_pass_lists_at_most_changed_items_over_the_page_plus_one`,
  `the_api_token_reaches_no_record_request_or_fact`; `tests/fuzzy_protocol.rs` (1)
  `stories_through_cut_streams_answer_each_call_once`. Tests that leave, all
  under their names in skein, with their times at the base: 38 focused and 5
  fuzzy of the protocol world (hooks 19, machine 14, fake_protocol 5; fuzzy_hooks
  1 at 0.46 s, fuzzy_machine 4 at 4.0 to 5.2 s here, 7.5 to 7.7 s in `base-runs.md`,
  decision 11); 29 and 2 of the client's world (simulation 25, locate 2,
  job_answer 1, memory 1; 0.025 s, and 0.119 s); 21 and 1 of reconcile's
  (0.075 s, and 1.27 s). In all 88 focused and 8 fuzzy at the base. Temper 02
  and 04 change the client's and reconcile's worlds first, so the step reads
  each package's count with `cargo nextest list -p <package>` before it
  deletes, and the gate's subtraction below is their sum.
Proof:
- joint: after the repin commit, `cargo check --workspace --all-targets`
  prints Finished (0.25 s at the base, already built).
- world: `cargo nextest run -p temper-engine-protocol-forge-world`: 9
  passed, 0 skipped, in about 0.14 s (47 at the base, 0.155 s here, 0.404 s
  in `base-runs.md`); each under 0.5 s.
- world: `cargo nextest run -p temper-engine-protocol-forge-world --profile fuzzy`:
  1 passed, about 0.2 s (6 in 5.2 s at the base).
- world: `cargo nextest run -p temper-engine-forge-world`: the count at
  the step's start, as before (54 passed in 0.044 s at the base; fuzzy 1 in 0.024 s); it compiles the relocated `translate.rs`.
- joint: `scripts/check.sh` exits 0: clippy clean, focused count =
  the start's minus 88 (1879 -> 1791 in the scratch worktree: 6.2 s against the
  gate's 6.1 s at the base), fuzzy = the start's minus 8 (50 -> 42: 21.5 s
  against 23.1 s; one untouched engine-domain test is nearly all of it); the
  gate's fmt and clippy are clean in the scratch tree.
- The process tier does not run: `cargo tree -p temper -e normal,dev`
  names none of the five packages (203 lines at the base, none of them).
Don't: edit any of the 10 tests; rewrite a moved or relocated part (`translate.rs`
  keeps `temper:key`); add an alias, a wrapper `Joint` or a `pub use`; delete or
  touch `tests/engine/forge`'s other files, `tests/codecs` or the tool (07.2); split a file.
Size: about 580 lines moved (`translate.rs` 567 by `git mv`, 12 users' lines
  changed), 10 new (the workspace and world dependency lines, the three `use`
  lines of skein's `joint`, the pruned `client` import).
Removes: about 5,082 lines: the client's world 2,239 (9 files), reconcile's
  world 900 (7 files), the protocol world's moved files 1,843 (10 files) and
  their 100 lines of fixtures, `mod` lines and manifest lines; with the two
  workspace members and the dependency lines.

### 07.2 The dialect's tests and the conformance deleted

Scope: forge 11, fake-forge 7; delete what skein 06.2 and 06.3 moved.
Needs: skein 06.2 and 06.3; temper 06.1; 07.1 (no reader of the corpus is left
  in the protocol world). No repin: this increment imports nothing new.
Files (deleted, at the base):
- `tests/codecs/src/forgejo.rs` (725 lines; it reads the transcripts and
  deliveries) and, in `src/lib.rs`, `pub mod forgejo;` with its doc line.
- `tests/codecs/tests/forgejo_{deliveries,files,items,reviews,setup,start,writes}.rs`
  (1,632 lines), `fuzzy_forgejo.rs` (48), `fixtures.rs` (122).
- `tests/codecs/Cargo.toml`: the dependency lines no remaining file reads (at
  the base `[dependencies]`: the dialect, service, component, JSON crates and
  `serde_json`; `[dev-dependencies]`: the service and the component: by role,
  since 01.2 and 03.1 renamed them; check each with `rg` in the files left).
- `tools/forgejo-conformance/`: 377 files, 15,619 lines (22 `src` files 4,485;
  303 `transcripts` and 48 `deliveries` files 10,817; `facts.txt` 221; README
  53; `clippy.toml` 13; `Cargo.toml` 30); the member line in `Cargo.toml`; the
  lock's entry.
Shape: nothing new. `tests/codecs` keeps `src/lib.rs` (`mutated`,
  `mutated_into`, read by its three fuzzy tests) and `tests/{client_wire,drift,
  worker_wire}.rs` (5 tests) with `fuzzy_{client_wire,store,worker_wire}.rs` (3).
Test shape: no new test. Leaving, with their times at the base: 55 focused of
  codecs (forgejo_setup 22, writes 8, files 6, reviews 6, items 5, deliveries 4,
  start 4; 0.154 s), 3 fuzzy (`fuzzy_forgejo`, 2.1 s) and the conformance's 35
  at the step's start (the base's 25 in 0.122 s, temper 06.1's refusal test and 06.2 to
  06.4's 9), whose
  recorder, document tests and facts are skein's. The refusal test leaves with
  the tool and has no counterpart: skein's tool reads no variable (answer 31).
Proof:
- world: `cargo nextest run -p temper-codecs-world`: 5 passed, 0 skipped,
  0.084 s in the scratch tree (60 in 0.222 s at the base, 0.427 s in `base-runs.md`).
- world: `cargo nextest run -p temper-codecs-world --profile fuzzy`: 3
  passed, 0.10 s in the scratch tree (6 in 2.3 s at the base).
- joint: `cargo check --workspace --all-targets` prints Finished; then
  `scripts/check.sh` exits 0: focused = the start's minus 90 (codecs'
  55 and the conformance's 35, with temper 06.1 to 06.4's tests; 80 in the
  scratch worktree, which lacks them: 1791 -> 1711), fuzzy = the start's minus
  3 (42 -> 39). After both increments: 1879 -> 1711 and 50 -> 39 at the base.
- The process tier does not run (07.1, Proof).
Don't: touch `tests/conformance` (the engine root on jig's harness, another
  thing), `tools/git-transcripts` and its corpus, or `tests/codecs`' other
  tests; re-record anything; edit `docs/design/` (the designer's: its drafts
  `draft/qa.md` and `draft/ci-on-forgejo-17.md` still name
  `tools/forgejo-conformance`).
Size: about 0 lines moved, 0 new (a deletion).
Removes: about 18,156 lines in 387 files: the tool's 15,619 (377 files), codecs'
  2,527 (10 files) and 10 lines of codecs' `lib.rs` and manifest; with the
  workspace member line and the tool's lock entry.

## Done when

Each command runs once in a worktree of temper at `origin/main`.

- `cargo check --workspace --all-targets` prints Finished, exits 0; the
  gate is `scripts/check.sh` (land owns it).
- `python3 -I -c 'from pathlib import Path; print("gone=" + str(not any(Path(p).exists() for p in ("tests/engine/forge/client", "tests/forge-setup", "tools/forgejo-conformance", "tests/codecs/src/forgejo.rs"))))'`
  prints `gone=True` (False at the base).
- `bash -c 'echo $(ls tests/engine/forge/src); echo $(ls tests/codecs/src); echo $(ls tests/codecs/tests)'`
  prints three lines: `changes.rs lib.rs translate.rs world.rs`, `lib.rs`, and
  `client_wire.rs drift.rs fuzzy_client_wire.rs fuzzy_store.rs fuzzy_worker_wire.rs worker_wire.rs`.
- `bash -c 'echo $(ls tests/engine/protocol-forge/src); echo $(ls tests/engine/protocol-forge/tests)'`
  prints two lines: `fixtures.rs lib.rs protocol.rs` and `fuzzy_protocol.rs protocol.rs`.
- `rg -n 'temper_engine_forge_client_world|temper-engine-forge-client-world|temper_forge_setup_reconcile_world|temper-forge-setup-reconcile-world|forgejo-conformance|forgejo_conformance' Cargo.toml crates testing tests tools scripts.config -g '*.rs' -g '*.toml' -g '*.sh'`
  prints nothing (exit 1); at the base 33 lines in 15 files.
- `rg -n '(temper_engine_protocol_forge_world|crate)::(hooks|joint|machine|peer|pipe)\b' tests -g '*.rs'`
  prints nothing (exit 1); `rg -n '#\[path = "\.\./client' tests` prints nothing.
- `rg -n 'skein[-_]forge[-_]machine[-_]world' Cargo.toml tests -g '*.rs' -g '*.toml'`
  prints the workspace line, the world's dependency line and the three `use`
  lines (5; nothing at the base).
- `cargo nextest run -p temper-engine-protocol-forge-world -p temper-codecs-world`
  prints `14 tests run: 14 passed, 0 skipped`; with `--profile fuzzy`, `4 tests run: 4 passed`.
- `cargo nextest run --workspace` and `--profile fuzzy` print the count at
  07.1's start minus the deleted packages' tests, and minus 11 for fuzzy: at
  the base minus 168 (88 + 55 + 25) and 11, 1711 passed and 39 passed; with
  temper 06.1 to 06.4's ten tests, minus 178.
