+++
id = "forge-entry"
part = "forge-entry"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-connection/src/component.rs", "skein/crates/skein-forge-connection/src/component/**", "skein/crates/skein-forge-connection/src/settings.rs", "skein/crates/skein-forge-connection/src/limits.rs", "skein/crates/skein-forge-connection/src/hooks.rs", "skein/crates/skein-forge-connection/src/hooks/connection.rs", "skein/crates/skein-forge-connection/src/tests/fixtures.rs", "skein/crates/skein-lib/src/list.rs", "skein/crates/skein-lib/src/tests/containers.rs"]
effort = "high"
minutes = 75
+++

# One entry per forge: the five parallel lists go

## Goal

`Component` keeps a forge in five lists by position, and removing one swaps across all five. This task moves the
data: `List<ForgeEntry>` holds each forge whole. Methods keep taking a position for now; `forge-common` changes that,
and this task doesn't start on it.

Acceptance:

- `rg -n 'self\.(starts|github|own_tokens|own_plans|settings)\b' skein/crates/skein-forge-connection/src -g
  '!**/tests/**' -g '!tests.rs'` finds nothing.
- `rg -n 'OwnPlan::Forgejo' skein/crates/skein-forge-connection/src` finds nothing, and `OwnPlan` keeps its five
  GitHub variants.
- `component/runtime.rs`'s `remove` no longer swaps by hand: it's `List::swap_remove`, and `get_disjoint_mut` is
  gone.
- `Settings`, `Forge`, `Component::new` and every other `pub` item keep their signatures.
- `limits::check` keeps its signature and the order it refuses in.
- `heavy cargo nextest run -p skein-lib -p skein-forge-connection -p skein-forge-machine-world -p
  skein-forge-protocol-world` passes, the same tests as at base.
- The crate's production lines don't rise (7,517 at base: `find skein/crates/skein-forge-connection/src -name '*.rs'
  -not -path '*/tests/*' -not -name 'tests.rs' | xargs cat | wc -l`).

## Inputs

- `temper/docs/plan/debt-13/notes/common.md`
- `temper/docs/plan/debt-13/notes/contracts.md`, 1
- `skein/docs/development/debt/D013-forge-connection-parallel-lists.md`
- `skein/docs/design/forge-connection.md`, 3.1 and 7.1
- `skein/docs/design/lib.md`, 5

## Owns and may change

- **Owns:** `skein-forge-connection/src/component.rs`, `src/component/` (new `entry.rs`), `settings.rs` (`Owner`),
  `skein-lib/src/list.rs` and its test in `src/tests/containers.rs`.
- **May change:**
  - `limits.rs`: `check`'s helpers take what they read, and a new `check_added`. `hooks.rs` and `hooks/connection.rs`:
    the settings parameter, as below.
  - The `#[cfg(test)]` helpers in `component.rs` and `component/github.rs`; `src/component/tests.rs`,
    `src/tests/fixtures.rs` and any test file, for how they reach a list (`c.own_tokens.get(..)`,
    `h.component.settings.forges`, `&self.settings` handed to a hook's pump). Calls only.
- **Not to touch:** `connection.rs` and its `Heard`, `Exchange` and the rest; `Waiting` and `CallPlan`;
  `installations.rs`. Connection fields that name a forge's kind stay.
- **Options:**
  - If a helper of `limits::check` can't take its parts without changing the order of refusals, leave `check` whole
    and have `check_added` repeat only the per-forge part, and say so in the report.
  - If `add`'s check at base can refuse something `check_added` can't reach, it's a change of behaviour: stop and ask.

## Steps

1. `skein-lib`'s `List`:

   ```rust
   /// Takes the item at `index` out, the last item taking its place. `None` past the end.
   pub fn swap_remove(&mut self, index: u32) -> Option<T>;
   /// Each item, to change in place.
   pub fn iter_mut(&mut self) -> slice::IterMut<'_, T>;
   ```

   Tests in `src/tests/containers.rs`: first, middle, last, past the end, and that capacity holds.
2. `component/entry.rs`, by `notes/contracts.md`, 1: `ForgeEntry`, `Base`, `Dialect`, `GitHubForge`, with

   ```rust
   impl ForgeEntry {
       /// A forge as it's added: its start from its access, its dialect from its kind.
       pub fn new(forge: Forge, limits: &Limits) -> Self;
       // Until `forge-common` and `forge-dialect` pass these down.
       pub fn github(&self) -> Option<&GitHubForge>;
       pub fn github_mut(&mut self) -> Option<&mut GitHubForge>;
   }
   ```

   `Component::new` and `runtime.rs`'s `add` build entries the same way today: both become `ForgeEntry::new`. A
   Forgejo forge's own plan sits in `Dialect::Forgejo { plan }`, so `OwnPlan` loses its `Forgejo` variant, and each
   `OwnPlan::Forgejo(_) => unreachable!` arm and `send_start`'s go with it.
3. `settings.rs`: `pub(crate) struct Owner { webhooks, word, user_agent }`. `Component` holds `forges:
   List<ForgeEntry>` and `owner: Owner` in place of `settings`, `starts`, `github`, `own_tokens` and `own_plans`. The
   rest of its fields stay on it: `forge-common` makes the split.
4. `hooks.rs`: `Hooks::drive(env, forges: &List<ForgeEntry>, word: &[u8], up, io)`, and `hooks/connection.rs`'s `pump`
   and `call` likewise. They read a forge's number and its webhook secret and the owner's word, nothing else.
5. `limits.rs`: `Forges::of` and `check` stand. `runtime.rs`'s `add` today pushes, runs `check` over everything and
   truncates on a refusal, ignoring why. Add

   ```rust
   /// The refusals one more forge can meet, past the ones a configured set has passed.
   pub(crate) fn check_added(l: &Limits, s: &Shared, owner: &Owner, forge: &Forge) -> Result<(), Unusable>;
   ```

   It holds the per-forge part of `check_settings` and of `check_github`, which `check` calls for each forge. `add`
   calls it before it pushes, and a refused forge is never pushed. `add`'s duplicate check and its count check stand.
6. Replace every list access by the entry's field, keeping `at`: `self.forges.get(at).expect("forge").base.start`.
   `remove` is one `swap_remove`, in the place of the swaps and truncates: the last forge takes the removed one's
   position, as at base, so the order forges are driven in doesn't change.
7. The `#[cfg(test)]` helpers (`test_github`, `test_github_mut`, `test_calls_ready`) read the entry, and test files
   follow.
8. `cargo fmt`, `heavy cargo clippy -p skein-lib -p skein-forge-connection --all-targets -- -D warnings`, the tests
   above, `heavy cargo check -p temper-service --tests` for the public face, commit, push.

## Report

What changed, the commit, what ran with counts, the measure (`rg -c '\.get(_mut)?\(at\)'
skein/crates/skein-forge-connection/src -g '!**/tests/**' | awk -F: '{s+=$2} END{print s}'`: 155 at base), and the
net lines, production and tests apart.
