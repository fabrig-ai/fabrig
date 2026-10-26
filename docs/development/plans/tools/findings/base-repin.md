# Base repin: jig and temper onto skein origin/main

Date: 2026-10-18. Raw measurement for the tools plan (development.md, section 5.7, "a plan repins what it may have broken").

## Revisions

| Repository | Revision | Note |
|---|---|---|
| skein | 49e7eb6d2ba7787f62f55c903e4f73ef39b447a8 | `origin/main` after `git fetch` |
| smith | a277e9226273bf71c845db98b62a59b9ca4c9bfd | `origin/main`; its own lock pins skein 841f4877 |
| jig | 08430b94 | `origin/main`, detached worktree |
| temper | 4735d1852 | `origin/main`, detached worktree |
| rustc/cargo | cargo 1.95.0 (f2d3ce0bd 2026-03-21) | |

## How each repository pins

Cargo.toml dependencies are unversioned git dependencies, no `rev`, no `[patch]`; the revision lives only in `Cargo.lock`.

```
jig/Cargo.toml:163    skein-io = { git = "https://github.com/forgetic/skein.git" }
jig/Cargo.toml:113    smith-agent-service = { git = "https://github.com/forgetic/smith.git" }
temper/Cargo.toml:62  jig-boundary = { git = "https://github.com/forgetic/jig.git" }
temper/Cargo.toml:118 skein-io = { git = "https://github.com/forgetic/skein.git" }
temper/Cargo.toml:125 smith-agent-world = { git = "https://github.com/forgetic/smith.git" }
```

Locked revisions before the repin (count = packages in the lock):

```
jig:    skein 0f093bb74454842bf393c236a021f5e82ceacd60 (34)   smith 29fa97c9cdfdcd1a0d0bc28e270de5be221c0928 (23)
temper: skein 8a51e10b873f39ea699853361408812eb80a5f87 (29)   smith 29fa97c9...  (28)   jig 08430b948f7fe1271fa6dd79796aebf4447a0fc8 (52)
```

jig's skein pin (0f093bb) is an ancestor of temper's (8a51e10); 92 skein commits lie between 8a51e10 and 49e7eb6 (93 from 0f093bb), 15 of the 93 touching `crates/`.

## The finding that shapes the fix: skein cannot be repinned alone

smith at 29fa97c does not compile against skein 49e7eb6 (smith itself repinned skein at b415f737, "Repin skein and handle its new file operation variants"). The first smith revision that does is b415f737, which already sits on smith's transcript v5. So a skein repin of jig or temper forces a smith repin, and that brings smith's own breaking changes (transcript `v3` module gone, `jobs` fields, `Request::Job`), none caused by skein. This matches development.md 5.7: "jig moves smith and skein together".

Three configurations were measured:

- **A, skein only** (smith, and for temper jig, left at their pins): the "expected" counts appear here. jig 15 = 7 own + 8 in smith's git crates; temper 19 = 6 own + 13 in jig's and smith's git crates. Not a coherent configuration: temper cannot fix the 13.
- **B, skein + smith** (smith moved to origin/main): the realistic repin. jig 35 errors over several layers, temper 17 errors over 3 layers.
- For temper, B uses a path `[patch."https://github.com/forgetic/jig.git"]` to the scratch jig worktree *after* its probe fixes (83 jig packages listed from `cargo metadata`), since no repinned jig exists on `origin/main` to pin. Temper therefore depends on the jig repin landing first.

## Commands

All cargo runs went (build lane), per development.md sections 5.7 and 11.1.

```
$ git -C ~/src/rust/skein fetch
$ git -C ~/src/rust/jig worktree add --detach <scratch>/base-jig origin/main      # same for temper
$ cargo update -p skein-io          # in each worktree: moves the whole skein git source
$ cargo update -p smith-domain      # config B: moves the whole smith git source
$ cargo check --workspace --all-targets --message-format=short              # first run, jig: stops at jig-protocol-store
$ cargo check --workspace --all-targets --keep-going --message-format=short # every run after
```

`--keep-going` was needed: without it cargo stops at the first failing crate. Even with it, a failing crate hides its dependents' errors, so the real total only shows after the earlier layers are fixed. Each layer below was revealed by stubbing the previous layer's errors with the smallest edit that compiles (probe edits, in the scratch worktrees only), and re-running until `cargo check --workspace --all-targets` finished clean. Probes were compile-only: no clippy, no `fmt`, no tests, no wasm32 target run.

Final state: both worktrees compile clean under config B with the probe edits. Probe size (excluding `Cargo.lock`): jig 23 files, +59 -30 lines; temper 12 files, +30 -13 lines. About 30 of jig's lines and nearly all of temper's are the skein-caused ones.

## jig, config A (skein only) as cargo reports it

```
jig-protocol-store (lib)       5 errors   E0004 file::Event::{Locked,Busy} not covered
jig-protocol-store (lib test)  +2         E0004 file::Request::{Lock,ListPage} not covered
smith-protocol-machine (git)   2          files.rs:106 Event::{Locked,Busy}; files.rs:146 kernel::Error::Busy
smith-local-protocol (git)     6          markers.rs:78, plain.rs:98, store/machine/ephemeral.rs:90, selection.rs:77, selection.rs:106 (Error::Busy), machine.rs:246
total 15
```

## jig, config B (skein + smith), errors by group

35 errors; 11 from skein, 24 from smith. The first run shows 9 (J1 5, J2 2 in the store tests, J4 2 in jig-protocol-transcript); the other 26 appeared over six more re-runs, as each crate went green and its dependents compiled (the smith v5 edits alone took three: J5/J6/J7 in jig-protocol-transcript, then J4 and J7 in tests/client, then J7 in tests/ops).

| Group | Cause | Errors | Sites | Fix | Size |
|---|---|---|---|---|---|
| J1 | skein `file::Event::{Locked,Busy}` (3432cf5, 303c829) | 5 E0004 | jig-protocol-store: `directory.rs:79`, `directory.rs:102`, `transcripts/cut.rs:143`, `transcripts/read.rs:724`, `transcripts/write.rs:189` | add the two variants to each exhaustive ignore arm (jig matches every variant, no wildcard) | S, ~10 lines |
| J2 | skein `file::Request::{Lock,ListPage}` (3432cf5, 2112fce) | 4 E0004 | store tests `tests/fixtures/store.rs:394`, `tests/transcripts.rs:255`; `testing/jig-test-service/src/accounts.rs:409` (`file_owner`); `testing/jig-test-store-files/src/files.rs:48` (hidden behind J3 until J3 is fixed) | add to the panic/ignore arms and to `file_owner`; fakes that never lock keep them impossible | S, ~12 lines |
| J3 | skein `Request::Rename { root }` became `{ from_root, to_root }` (24be04f) | 2 (E0026, E0027) | `testing/jig-test-store-files/src/files.rs:104` | pass `opened(from_root)`, `opened(to_root)` to `machine.rename` | S, 3 lines |
| J4 | smith transcript `v3` module removed (now `v5`) | 12 (E0432/E0433) | `jig-protocol-transcript/src/lib.rs:17`, `summary.rs:6`; `tests/client/src/`: `joint/fixture.rs:16,126`, `joint/front.rs:298`, `stories/unread_turn.rs:46` (lib), `joint/session.rs:907`, `joint/wire/activity.rs:380`, `joint/wire/turns.rs:23`, `joint/deployment_story.rs:50`, `joint/historical_story.rs:26`, `joint/role_stories.rs:140` (lib test) | rename `v3` to `v5`; the golden bytes and format-version behaviour of jig's own tests were not run, and may need real work | S to compile, unknown to pass |
| J5 | smith `Returned::NotRun` now carries a payload (`NotRun`, with a `why`) | 4 (E0532, E0308) | `jig-protocol-transcript`: `summary.rs:81`, `summary.rs:134`, `lib.rs:218`, test `tests.rs:357` | pattern `NotRun(_)`; showing the reason in the summary is a design choice | S to compile, M if the reason is displayed |
| J6 | smith v5 new variants: `Outcome::{Running,Job}`, `CallProblem::{JobsFull,UnknownJob}` | 3 E0004 | `jig-protocol-transcript/src/summary.rs:51`, `:104`, `:157` | probe stubbed them as "Job", "jobs full", "unknown job"; jig needs a rule for how a turn's jobs read in the client's vocabulary | M (a decision; the stub is S) |
| J7 | smith `Turn.jobs: u32` new field (record and transcript `TurnParts`) | 5 E0063 | `jig-protocol-transcript/src/tests.rs:44`, `testing/jig-test-worker/src/tests/fixtures.rs:171`, `testing/jig-test-service/src/tests/fixtures.rs:572`, `tests/client/src/joint/fixture.rs:36`, `tests/ops/src/conformance/scripts.rs:494` | `jobs: 0` | S, 5 lines |

Sum: 5 + 4 + 2 + 12 + 4 + 3 + 5 = 35. By cause: skein (J1-J3) 11, smith (J4-J7) 24.

Excerpts:

```
crates/jig-protocol-store/src/directory.rs:79:11: error[E0004]: non-exhaustive patterns: `skein_io::file::Event::Locked { .. }` and `skein_io::file::Event::Busy { .. }` not covered
testing/jig-test-store-files/src/files.rs:104:44: error[E0026]: variant `skein_io::file::Request::Rename` does not have a field named `root`
crates/jig-protocol-transcript/src/lib.rs:17:5: error[E0432]: unresolved import `smith_transcript::v3`: no `v3` in the root
testing/jig-test-worker/src/tests/fixtures.rs:171:16: error[E0063]: missing field `jobs` in initializer of `smith_domain_session::record::Turn`
```

Whole jig repin estimate (skein part only J1-J3): S, about 30 lines. With the smith part (J4-J7): about 60 lines of probe edits, which is within 5.7's "about 200 lines" bound, but J5 and J6 are design questions (what jig's client shows for jobs and a not-run reason), which 5.7 says go to a question and "Not planned", not into the repin.

## temper

### Config A (skein only; jig 08430b9 and smith 29fa97c left pinned): 19 errors

```
temper-worker-protocol-git (lib)       2   files.rs:296, files.rs:352                       E0004 kernel::Op::Lock
temper-worker-protocol-git (lib test)  +4  tests/fixtures.rs:218, 257, 296, 424             E0004 kernel::Op::Lock
jig-protocol-store (git dep)           5   same five sites as jig J1
smith-protocol-machine (git dep)       2
smith-local-protocol (git dep)         6
total 19  (6 in temper, 13 in git dependencies temper cannot edit)
```

Not coherent: the 13 need the jig and smith repins first.

### Config B (skein + smith origin/main, jig = scratch worktree with its probe fixes via path patch): 17 errors

Revealed in three layers: 10, then 2, then 5. All sites are in temper.

| Group | Cause | Errors | Sites | Fix | Size |
|---|---|---|---|---|---|
| T1 | skein `kernel::Op::Lock` new variant | 11 E0004 | `temper-worker-protocol-git`: `src/files.rs:296`, `:352`; `src/tests/fixtures.rs:218`, `:257`, `:296`, `:424`. `temper-client-shell-terminal/tests/terminal.rs:144`. `tests/worker/protocol-git/src/io.rs:383`. `testing/temper-fake-git/tests/transcripts.rs:165`, `:221`. `temper-service/src/tests/fixtures/system.rs:76` | add `Op::Lock { .. }` to each exhaustive arm (unreachable/panic arms: temper takes no locks) | S, ~14 lines |
| T2 | skein `file::Request::{Lock,ListPage}` | 1 E0004 | `crates/temper-service/src/accounts.rs:419` (`file_owner`) | add both to the owner arm | S |
| T3 | skein `kernel::Error::Busy` new variant | 1 E0004 | `crates/temper/src/lib.rs:97` (`errno`) | `Error::Busy => 16` (EBUSY) | S, 1 line |
| T4 | smith `Request::Job` new variant | 1 E0004 | `crates/temper-engine-smith/src/finish.rs:35` | probe maps it to `Problem::Type` like the other non-turn requests; does temper route jobs? | S to compile, M if temper must handle job requests |
| T5 | smith `record::Turn.jobs` | 2 E0063 | `tests/worker/domain/src/protocol.rs:67`, `crates/temper-worker-domain/src/tests/fixtures.rs:236` | `jobs: 0` | S |
| T6 | smith transcript `v3` removed | 1 E0433 | `tests/client/src/fixtures.rs:100` | `v3` to `v5` (and `CEILINGS` must still be the right limits) | S |

Sum: 11 + 1 + 1 + 1 + 2 + 1 = 17. By cause: skein 13, smith 4. The skein part alone is S, about 20 lines; the two design-bearing smith items are T4 and (via jig) J5/J6.

Excerpts:

```
crates/temper-worker-protocol-git/src/files.rs:296:37: error[E0004]: non-exhaustive patterns: `skein_io::kernel::Op::Lock { .. }` not covered
crates/temper/src/lib.rs:97:11: error[E0004]: non-exhaustive patterns: `skein_io::kernel::Error::Busy` not covered
crates/temper-engine-smith/src/finish.rs:35:11: error[E0004]: non-exhaustive patterns: `smith_domain::Request::Job { .. }` not covered
```

## What the skein changes are

Every skein-caused error traces to four commits in `skein-io` (all after 0f093bb): 303c829 (kernel `Op::Lock`, `Error::Busy`), 3432cf5 (`file::Request::Lock`, `Event::{Locked,Busy}`), 2112fce (`file::Request::ListPage`), 24be04f (`file::Request::Rename` takes `from_root` and `to_root`; the kernel `Op::Rename` already had `from_dir` and `to_dir`). All are new enum variants matched exhaustively, plus one renamed field. smith's b415f737 shows the house style of the fix (new arms in exhaustive matches, locks kept impossible where no caller requests one, Busy kept on the generic failure mapping).

## Not measured

- clippy (`-D warnings`), `cargo fmt --check`, nextest, the wasm32 clippy and browser tier of jig's gate. The probes were not formatted: rustfmt may reflow some of the added arms.
- Whether jig's and temper's tests still pass with smith's transcript v5 (golden bytes, format version 3 to 5).
- Third-party crate drift: `cargo update -p` changed `Cargo.lock` only for the skein and smith git sources (jig: 117 changed lines in the lock; temper: 293 changed lines, mostly the jig packages turned into path packages by the patch).
