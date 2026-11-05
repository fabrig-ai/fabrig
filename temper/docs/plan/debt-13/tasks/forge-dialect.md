+++
id = "forge-dialect"
part = "forge-entry"
kind = "feature"
needs = ["forge-common"]
files = ["skein/crates/skein-forge-connection/src/component.rs", "skein/crates/skein-forge-connection/src/component/**", "skein/crates/skein-forge-connection/src/tests/**", "skein/crates/skein-forge-connection/src/tests.rs"]
effort = "high"
minutes = 90
+++

# The dialect is matched once, at the entry

## Goal

A GitHub method still asks its entry for its GitHub state each time, and `github.rs` carries a whole `Start` through
its work, with an `unreachable!` for each Forgejo state. Here each walk over the forges matches `Dialect` once,
and what runs below it takes its own dialect's state. GitHub's active work gets its own small enum, so those arms
stop existing.

Acceptance:

- `rg -n 'fn github(_mut)?\(' skein/crates/skein-forge-connection/src/component/entry.rs` finds nothing.
- `rg -n 'expect\("GitHub' skein/crates/skein-forge-connection/src/component` finds nothing.
- `rg -n 'unreachable!\(.*(active GitHub|GitHub selection)' skein/crates/skein-forge-connection/src` finds nothing.
- `Credential::Token(_) => unreachable!("checked App access")` in `github.rs` stands, since it names a credential.
- A forge's `kind` is read in `ForgeEntry::new` and as `limits::access`'s argument alone among the component's files:
  `rg -n '\.kind\b' skein/crates/skein-forge-connection/src/component.rs
  skein/crates/skein-forge-connection/src/component -g '!tests.rs'` shows nothing else.
- `heavy cargo nextest run -p skein-forge-connection -p skein-forge-machine-world -p skein-forge-protocol-world -p
  github-conformance -p skein-fake-peers-tests` passes, the same tests as at base.
- The crate's production lines fall from what `forge-common` left.

## Inputs

- `temper/docs/plan/debt-13/notes/common.md`
- `temper/docs/plan/debt-13/notes/contracts.md`, 1
- `skein/docs/design/forge-connection.md`, 3.4, 7.1 and 8.3

## Owns and may change

- **Owns:** `skein-forge-connection/src/component.rs` and `src/component/`.
- **May change:** the `#[cfg(test)]` helpers and how a test reaches them.
- **Not to touch:** `start::Start`, which is public, which worlds match on, and which stays one enum in `Base`.
  `start::heard`. `Waiting`, `CallPlan` and `github_plan`, which are `forge-calls`'s. `connection::Heard` and its
  `unreachable!`s, which are `forge-exchange`'s.
- **Options:**
  - If `github_state`'s callers need a state that isn't active, the function answers `Option<Active>`, and `None`
    takes the path its caller's arm took for those states.
  - A call path that needs the forge's GitHub state while holding `Waiting` may keep taking `gh` as a parameter from
    `drive_calls`'s match.

## Steps

1. Each walk that branches on a forge's kind matches `entry.dialect` once, by a split borrow:

   ```rust
   let ForgeEntry { base, dialect } = entry;
   match dialect {
       Dialect::Forgejo { plan } => common.drive_forgejo(base, plan, env, up, io),
       Dialect::GitHub(gh) => common.drive_github(base, gh, env, up, io),
   }
   ```

   The walks: `drive_own`, `own_closed`, `shed`, `connect`, `offer_pullers`, `refused_hint`, `installations`,
   `next_deadline`, `drive_calls`. `initial` takes the `Dialect` in place of a `Kind`, so `begin` and `connect` don't
   read the forge's kind.
2. Below that match, GitHub's methods take `(base: &mut Base, gh: &mut GitHubForge, ..)`, reading `gh.state` and
   `gh.plan`, and Forgejo's take `(base, plan: &mut Option<start::Plan>, ..)`. The accessors go.
3. In `github.rs`:

   ```rust
   /// What a GitHub forge's own connection is working on (forge-connection.md, 3.4 and 8.3).
   #[derive(Clone, Copy, PartialEq, Eq, Debug)]
   enum Active {
       Listing { next: Duration },
       Minting { next: Duration },
       Identity { next: Duration },
       Ready { cap: u32 },
   }
   ```

   `github_state` makes one from `base.start` and `gh.state.work`, and `github_phase`, `github_wait`, `github_begin`,
   `github_send`, `github_error` and `active_next` take it. The six arms listing `Unconnected`, `Checking`, `Settings`,
   `Waiting` and `Stopped` go where their function can't be reached in those states. Where a function can be, the
   arm's code runs on `None`.
4. `cargo fmt`, `heavy cargo clippy -p skein-forge-connection --all-targets -- -D warnings`, the tests above, commit,
   push.

## Report

What changed, the commit, what ran with counts, which functions answer `Option<Active>` and why, and the net lines,
production and tests apart.
