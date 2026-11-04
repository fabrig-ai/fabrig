+++
id = "forge-common"
part = "forge-entry"
kind = "feature"
needs = ["forge-entry"]
files = ["skein/crates/skein-forge-connection/src/component.rs", "skein/crates/skein-forge-connection/src/component/**", "skein/crates/skein-forge-connection/src/connection.rs", "skein/crates/skein-forge-connection/src/puller.rs", "skein/crates/skein-forge-connection/src/tests/**", "skein/crates/skein-forge-connection/src/tests.rs"]
effort = "high"
minutes = 120
+++

# A step on a forge is handed its entry

## Goal

Every method takes a position and looks its forge up again. With the data whole in entries (`forge-entry`), the
component splits in two, the forges and what they share, so a step can hold one forge's entry and everything else
at once. Positions go. This is mechanical across the crate, and nothing a test can see changes.

Acceptance:

- The item's **Measure** reads 0 or nothing: `rg -c '\.get(_mut)?\(at\)' skein/crates/skein-forge-connection/src -g
  '!**/tests/**' | awk -F: '{s+=$2} END{print s}'` (155 at this plan's base).
- `rg -n '\bat: u32' skein/crates/skein-forge-connection/src -g '!**/tests/**' -g '!tests.rs'` finds nothing. An
  origin's or a page's position has its own name, `origin` or what it is, as at `Connection::at_origin`.
- `call_connections` and `assign` read `Connection::role`, and neither walks the forges.
- `Component` has exactly the fields `forges` and `common`, and every `pub` method keeps its signature.
- `heavy cargo nextest run -p skein-forge-connection -p skein-forge-machine-world -p skein-forge-protocol-world -p
  github-conformance -p skein-fake-peers-tests -p temper-engine-protocol-forge-world` passes, the same tests as at
  base. Those worlds read the component from outside, so a moved event shows there: stop and ask.
- The crate's production lines don't rise above what `forge-entry` left.

## Inputs

- `temper/docs/plan/debt-13/notes/common.md`
- `temper/docs/plan/debt-13/notes/contracts.md`, 1
- `skein/docs/development/debt/D013-forge-connection-parallel-lists.md`
- `skein/docs/design/forge-connection.md`, 7.1 and 7.2

## Owns and may change

- **Owns:** `skein-forge-connection/src/component.rs` and `src/component/`.
- **May change:** `connection.rs`: `Connection::role` and `Use`, and each `Some(at)` of a log origin, renamed
  `origin`. `puller.rs`: its `.get(at)`'s local name. The `#[cfg(test)]` helpers and how any test reaches
  them; `src/component/tests.rs` reads `c.call_connections()` and an own token by position.
- **Not to touch:** the `ForgeEntry` accessors `github` and `github_mut`, which `forge-dialect` removes. `Waiting`,
  `CallPlan`, `Heard`. Connection deadlines, wherever they're set: another item's. `hooks` beyond the receiver's
  spelling.
- **Change of meaning:** `Connection::role` replaces a scan, so the count it gives must be the scan's at every
  moment, including while a connection whose forge let go of it is closing. The worlds above are this task's check.
- **Options:**
  - A method that needs two forges at once: there's none at this base. If you find one, stop and ask.
  - If a function exceeds clippy's argument count by the change, group `env`, `up` and `io` no differently: it
    takes `entry` in `at`'s place, so its count stands. If it can't, a scoped `#[expect]` with its reason.
  - If iterating `forges` while calling `Common` needs a different loop shape, keep the iteration order.

## Steps

1. `component.rs`: `Component { forges: List<ForgeEntry>, common: Common }`, with every other field in `Common`
   (`pub(crate)`), `owner` among them.
2. `impl Common` takes every method that works on one forge, with `entry: &mut ForgeEntry` in `at`'s place, and any
   method that touches no forge (`finish`, `expire`, `open_call_origin` and the like, the last taking `entry`'s
   `&Forge`). `impl Component` keeps the public face and the methods that walk the forges: `begin`, `down`, `up`,
   `fire`, `drive`, `drive_own`, `drive_calls`, `shed`, `connect`, `own_closed`, `refused_hint`, `installations`,
   `offer_pullers`, `add`, `remove`. They split the borrow with `let Self { forges, common } = self;`.
3. Finding a forge by number is one function:

   ```rust
   fn find(forges: &mut List<ForgeEntry>, number: u16) -> Option<&mut ForgeEntry>;
   ```

   It replaces each `for at in 0..` that compares numbers, in `drive_calls`, `connect`, `installations`, `remove` and
   `follow_github_log`, the last taking its entry from its caller.
4. `connection.rs`:

   ```rust
   /// Whose a connection is (forge-connection.md, 7.2).
   #[derive(Clone, Copy, PartialEq, Eq, Debug)]
   pub(crate) enum Use {
       /// The calls' pool.
       Calls,
       /// A Forgejo forge's own connection, which a call to that forge may take.
       Own,
       /// A GitHub forge's own connection, outside the calls' count.
       Reserved,
   }
   ```

   `Connection::role: Use` starts `Calls`. `open_own` sets `Own` or `Reserved` by the forge's kind, where it sets
   `base.own`. Where `base.own` goes `None` on a connection still open (`connect`'s take, `assign`'s hand-over), the
   same line sets `Calls`. So `call_connections` counts what isn't `Reserved` and isn't removed, and `assign`'s
   `reserved` is `role == Reserved` and its `start` is `role != Calls`.
5. The `.get(at)` and `.expect` chains become the entry's fields. Keep each function's statements in their order: this
   task moves no decision.
6. `#[cfg(test)]` helpers stay on `Component`, and `test_local` passes the first entry.
7. `cargo fmt`, `heavy cargo clippy -p skein-forge-connection --all-targets -- -D warnings`, the tests above, commit,
   push. Commit the role apart from the mechanical move, so the role can be read alone.

## Report

What changed, the commit, what ran with counts, the measure, any test that changed how it reaches the code, and the
net lines, production and tests apart.
