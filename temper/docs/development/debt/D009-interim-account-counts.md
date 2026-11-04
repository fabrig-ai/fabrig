# D009: The client keeps interim account counts beside jig's

- **Kind:** duplication
- **Where:** `temper-client-domain/src/limits.rs`, `temper-client-view/src/accounts.rs`, `temper-client-view/src/pages/accounts.rs`, `temper-client-shell-web/src/limits.rs`, `temper-client-tools/src/limits.rs`
- **Seen:** 2026-11-03 at `41d63da8c3`, in the code: "Interim view counts until jig owns receiving counts (owner, 17:51Z; x1g)" (`temper-client-domain/src/limits.rs`, on `AccountLimits`; new here)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (one set of account counts to configure and check)
- **Size:** S
- **Status:** open
- **Paid when:** temper's client reads the account list's counts from `jig_client::Limits.accounts` (`AccountCounts`), and `AccountLimits` is gone or holds only what jig's lacks, with a comment that says why.

## What

`temper-client-domain`'s `AccountLimits` (`accounts`, `sources`, `uses`,
`sign_ins`) was a stand-in "until jig owns receiving counts". jig owns
them now: `jig_client::Limits.accounts` is an `AccountCounts` with
`accounts`, `sources` and `sign_ins` (`jig-client-domain/src/limits.rs`,
in jig's tree). So every temper shell sets both, side by side with the
same values: `temper-client-shell-web/src/limits.rs` writes
`AccountCounts { accounts: 16, sources: 8, sign_ins: 4 }` and, 34 lines
below, `AccountLimits { accounts: 16, sources: 8, uses: 8, sign_ins: 4 }`;
`temper-client-tools/src/limits.rs` and the fixtures of
`temper/tests/client/src/fixtures.rs`, `temper-client-view/src/tests/fixtures.rs`
and `temper-client-view/tests/suite/{task_pages,change_page}.rs` do the
same. The view sizes its account pages from temper's copy
(`temper-client-view/src/accounts.rs`, `binding_capacity`, which also
reads `uses`).

## Why it matters

Two sets of the same bounds can disagree, and nothing checks that they
agree; the comment cites a session's ruling no reader can find (D008).

## What pays it back

The view reads `accounts`, `sources` and `sign_ins` from jig's counts.
`uses` stays temper's if jig has no such count, named for what it
bounds, or jig's `AccountCounts` gains it (a jig change). One task.
