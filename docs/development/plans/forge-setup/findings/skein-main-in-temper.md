# Does temper build and pass its gate on skein origin/main?

Yes. Moving temper's Cargo.lock from skein 8a10437f to ec686b8a changes no
compiled code and breaks nothing. Checked 2026-10-16.
## Setup and pins

- `git -C ~/src/rust/temper worktree add --detach ~/src/rust/tmp/temper/forge-setup-plan/skein-main origin/main`:
  exit 0. temper's origin/main is dceef129, one commit after 0ff45022 (Ruling
  153 of client-api, 5 files under docs/), so the base is equivalent.
- skein origin/main: ec686b8a94e39d4579d8b4d80f8c4bda1b5b9c63.
- Cargo.toml names skein, jig and smith by GitHub git URL only (no branch, no
  rev); Cargo.lock pins them: 29 skein packages at `#8a10437f...`, jig
  `#2900e9b3` (48), smith `#29fa97c9` (28). jig's and smith's crates bring no
  second skein source: the lock holds one skein URL.
## Commands as run (directly)

1. `cargo update -p skein-lib --precise ec686b8a94e39d4579d8b4d80f8c4bda1b5b9c63`:
   exit 0 (queued 79s, ran 3s). All 29 skein packages moved at once.
   `git diff --stat Cargo.lock`: 29 insertions, 29 deletions, all skein
   `source =` lines. After: 29 packages at `#ec686b8a94e3...`, one source.
2. `cargo check --workspace --all-targets`: exit 0, 48s, no warnings.
3. `git commit -qam 'scratch: skein main'` (345a4bea, not pushed), then
   `scripts/check.sh`: exit 0, 102s. Stages: fmt, clippy
   -D warnings, nextest default (1297 run, 1297 passed, 1 skipped, 5.1s),
   nextest fuzzy (46 run, 46 passed, 27s; one slow test, engine-domain-world
   fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure, 21.5s).
   No failing test.
4. `git -C ~/src/rust/temper worktree remove --force .../skein-main`: exit 0.
   Nothing pushed, no shared checkout changed.
## Why nothing breaks

- `git -C ~/src/rust/skein log --oneline 8a10437f..origin/main | wc -l` is 5;
  8a10437f is an ancestor of origin/main.
- `git diff --name-only 8a10437f origin/main -- crates testing tests Cargo.toml Cargo.lock`
  lists nothing. The 5 commits (ce02728, e6fd9fb, 121daa4, ec686b8, f6c14c0:
  the forgejo-17 retrospective and review, run kits, and the design of
  HMAC-SHA256 and a constant-time comparison in lib) touch only docs/ and
  scripts/dev/. skein's Rust sources are byte-identical at the two revisions.
- The suspect d21d90d (design of a directory lock, cross-directory renames,
  paged listings) is already inside 8a10437f, temper's current pin, and
  changed only docs/design/*. `Rename`'s fields are unchanged in code;
  temper's one use, `Op::Rename { .. }` (crates/temper-client-shell-terminal/
  tests/terminal.rs:181), compiles.
## What a fix would need

Nothing. Caution: ec686b8 and 121daa4 only design HMAC in skein-lib
(docs/design/lib.md); no `hmac` exists in skein's Rust yet, so moving the lock
gives temper no HMAC; that needs a skein plan first.
