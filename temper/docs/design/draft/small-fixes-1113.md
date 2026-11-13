# Four small fixes

Draft, 2026-11-13. Four separate faults, each small, seen on a deployment
the same day. Each lands with a focused test where it changes behaviour.

## 1. The forge client's lifetime defaults to a test fixture's 10 s

- **Seen:** `forge.client.lifetime_ms` defaults to 10 s
  (`temper/crates/temper-engine-service-vocabulary/src/limits.rs`, "from
  the engine forge world fixture"), and the forge component's call
  deadline is that lifetime. A Forgejo merge of a pull request of about
  7,000 files was killed at 10 s and answered 500, read as a timeout, and
  the change was held. A deployment's override of 300 s fixed it.
- **Wanted:** each deployment profile's default fits a real forge's merge:
  300 s, under Forgejo's own default git timeout of 360 s. The fixture's
  own value stays in the tests that need it. The limit's line in
  `../protocol/configuration/limits.md` says what it bounds and why its
  default is what it is.

## 2. ctl's sign-in into a signed-in directory

- **Seen:** `temper ctl DIR sign-in FILE` with a fresh, unspent link, into
  a directory already signed in, was refused with `the sign-in link was
  refused` (exit 1). The same link signed a fresh, empty directory in at
  once. The words sent the reader looking for a spent or expired link.
- **Wanted, ranked:**
  1. ctl refuses before sending anything, naming the state and the way
     out: the directory is signed in as party N; `sign-out` first, or use
     another directory. The link stays unspent.
  2. If the engine refuses for another reason, its words say which:
     spent, expired, or a directory already signed in.

## 3. The journeys tool writes under /tmp

- **Seen:** `temper/crates/temper-client-journeys-tool/src/run.rs` falls
  back to `/tmp` when no directory is prepared, and its `tui_again` tests
  use `/tmp/journey-environment`, ignoring `TMPDIR`. On a host whose
  `/tmp` is held in memory, that scratch belongs elsewhere.
- **Wanted:** the fallback and the tests use the platform's temporary
  directory, which honours `TMPDIR` (`std::env::temp_dir`), with a
  directory of their own under it; nothing in the tool names `/tmp`.

## 4. The declared rust-version isn't true

- **Seen:** the workspace declares `rust-version = "1.95"`, but main does
  not build on stable 1.95.0:
  `temper/crates/temper-engine-service-stock/src/stock.rs:416` iterates
  `&mut charters` over a `Box<[Charter; 12]>`, which 1.95 rejects (E0277).
  Builds pin a dated nightly, so nothing caught it.
- **Wanted:** that line builds on 1.95 (`charters.iter_mut()`), and a
  search of the workspace for the same pattern (`&mut` or `&` over a boxed
  array in a `for`) fixes any others the same way. Checking the declared
  version in CI is out of scope.

## 5. Tests

Cheap, focused and fuzzy suites only; walk = false. Run only the new and
affected tests by name.
