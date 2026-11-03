# bugs-security: the plan

From `inputs/request.md` (the debt backfill's "Bugs", routed by the director, 2026-11-03): bugs
that leak a secret, each fixed with a test in the lowest world that shows it, failing on the old
code. Every task names `notes/common.md`; the `Debug` tasks also name `notes/debug.md`.

## The design

Written by this cut, each with its status entry, closed by the task that builds it:

- `skein/docs/design/lib.md`, 8: `CryptoRng` "is neither `Copy` nor `Clone`, since a copy would
  yield its bytes again"; its owner lends it by `&mut`, and a component that keeps one is made
  one on a stream of its own. `lib.status.md`: **Not yet** 8.
- `skein/docs/design/oauth.md`, 6.7: `Component::check` checks accounts as making the component
  does, taking no generator. `oauth.status.md`: **Not yet** 6.7.
- `skein/docs/design/io.md`, 5: "A file's contents print as their length" (`Store`, `WriteAt`,
  `Loaded`, `Read`). `io.status.md`: **Not yet** 5, a clause.

`skein/docs/design/STATUS.md` is not edited: its summary names oauth.md among the documents
with no gap, and so is stale for oauth.md until `oauth-accounts-check` lands. It is
`public-reading-build/public-status`'s file now, and the window is one task long.

## Parts and tasks

1. **jig-debug** (`jig/docs/design/domain/hosts.md, 5.1`), slice 1.
   - `tasks/jig-resources-debug.md`, medium, ~30 minutes, needs nothing (request item 1):
     `jig-test-service`'s `program_sources::Resources` gets temper's count-only `Debug`; tests in
     a new `src/tests/program_sources.rs` format it, and a configured `Component`, with a canary.
     Landed.
2. **crypto-rng** (`skein/docs/design/lib.md, 8`), slice 2 (request item 3).
   - `tasks/oauth-accounts-check.md`, medium, ~40 minutes, needs nothing:
     `skein-oauth-accounts`' `Component::check(accounts, limits)`, the check half of `new`, no
     generator.
   - `tasks/jig-accounts-check.md`, medium, ~30 minutes, needs `oauth-accounts-check` and
     `jig-resources-debug` (one writer for `jig-test-service`): `provider_templates` checks with
     `Component::check`, no `crypto.clone()`.
   - `tasks/temper-accounts-check.md`, medium, ~30 minutes, needs `oauth-accounts-check`: the
     same in `temper-service`.
   - `tasks/crypto-rng-not-clone.md`, high, ~50 minutes, needs both: skein-lib drops the derive,
     an inherent-constant trait check holds `CryptoRng: !Clone` (failing on the old code), and
     skein-forge-setup's two oracle clones replay a fresh fixture generator.

Still being cut (later pushes): skein-io's `file::Request` and `file::Event` (item 2), and the
other credential holders the search of item 4 finds, tree by tree.

Critical path so far: `oauth-accounts-check`, then `temper-accounts-check` or
`jig-accounts-check`, then `crypto-rng-not-clone` (~2 hours).

## What was decided, and why

- **Released first:** item 1 needs no survey: `Resources` is built through `from_paths` and
  `new` only, and nothing reads its `Debug` text (no test, trace or golden file holds it).
- **The check-only copies get a check, not a fresh stream.** The survey found four clones of a
  `CryptoRng`: jig-test-service's and temper-service's `provider_templates` give a throwaway
  accounts component `crypto.clone()` to check one provider's account, and two skein-forge-setup
  tests clone the host's generator as an oracle. `Component::new` never draws: it only checks
  and stores the generator. So the owners need no generator for the check at all; drawing a
  fresh one per provider would need an undocumented stream number for each, in processes whose
  streams are already chosen by hand (1, 5, the seed itself), for generators that never draw.
  `Component::check` sits beside `skein_oauth_accounts::valid` (oauth.md, 6.7). The two test
  oracles replay the fixture generator the host is made with, skipping what it drew.
- **The order keeps the trunk building:** the owners stop cloning before skein-lib drops the
  derive, so `crypto-rng-not-clone` needs both; each branch stays in its tree.
- **The trait check is an inherent-constant probe** in skein-lib's step tests, a control type
  beside it: nextest runs no doctests, so a `compile_fail` doctest would not run in the gate, and
  no repository has a compile-fail mechanism. The spec allows the doctest only if the probe does
  not compile, saying so.

## Overlaps with the drafts beside this one

- `jig-resources-debug` and `jig-accounts-check` edit files inside `fabrig-scale/standard-scale`'s
  glob `jig/testing/jig-test-service/src/**` (`program_sources.rs`, a new
  `src/tests/program_sources.rs` and one `mod` line in `src/tests.rs`; `accounts.rs`'s
  `provider_templates`). That task's work is the standard profile's scale, not program sources
  or accounts, so only the `mod` line could meet a textual conflict.
- `temper-accounts-check` edits `temper/crates/temper-service/src/accounts.rs`, which no draft
  beside this one names.
