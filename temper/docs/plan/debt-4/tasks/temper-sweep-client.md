+++
id = "temper-sweep-client"
part = "temper-forms"
kind = "migration"
needs = []
files = [
  "temper/crates/temper-client-tools/**",
  "temper/crates/temper-settings-form/**",
  "temper/crates/temper-client-view/**",
  "temper/crates/temper-client-domain/**",
  "temper/crates/temper-client-shell-cli/**",
]
effort = "medium"
minutes = 45
+++

# temper D003, D004: the client crates use `len_u64`, `usize_of` and `env.with`

## Goal

The client crates spell their widenings with skein-lib's helpers and hand their child steps their limits with `env.with`. `temper-client-tools`'s test `Harness` keeps its own clocks, so its seven rewraps call one new method. Both items stay open: `temper-sweep-paid` deletes them.

Acceptance:

- With `OWN="temper/crates/temper-client-tools temper/crates/temper-settings-form temper/crates/temper-client-view temper/crates/temper-client-domain temper/crates/temper-client-shell-cli"`, the three searches of `temper/docs/plan/debt-4/notes/temper-forms.md` read nothing.
- `temper-client-tools/src/tests/fixtures.rs`'s `Harness` has `Harness::env`, and no test of the crate builds an `Env` from `root.now` or `self.now`.
- `temper-client-tools/src/jig/runner/specification.rs`'s `u64::try_from(page.text.bytes().len())` and `temper-client-shell-cli/src/tests/fixtures.rs`'s `u64::try_from(self.contents(fd).len())` are `len_u64` too.
- `heavy cargo nextest run -p temper-client-tools -p temper-settings-form -p temper-client-view -p temper-client-domain -p temper-client-shell-cli` passes, every count above zero; no assertion of an existing test changes.
- `heavy cargo clippy -p temper-client-tools -p temper-settings-form -p temper-client-view -p temper-client-domain -p temper-client-shell-cli --all-targets -- -D warnings` passes.

## Inputs

- `temper/docs/plan/debt-4/notes/common.md`, `temper/docs/plan/debt-4/notes/temper-forms.md`, `AGENTS.md`, `temper/AGENTS.md`.
- `temper/docs/development/debt/D003-widening-try-from.md` and `temper/docs/development/debt/D004-env-with.md`, whole.
- `skein/docs/design/lib.md`, 6 (the widenings) and 9 (`env.with`); `skein-lib/src/length.rs`, `skein-lib/src/env.rs`.

At base, `$OWN`'s sites by the note's searches:

| Package | `u64::try_from(..len())` | `"u32 fits usize"` | `Env` rewraps | Files |
|---|---|---|---|---|
| `temper-client-tools` | 9 | 0 | 8 | 17 |
| `temper-settings-form` | 7 | 4 | 0 | 2 |
| `temper-client-view` | 5 | 0 | 0 | 3 |
| `temper-client-domain` | 2 | 3 | 1 | 3 |
| `temper-client-shell-cli` | 1 | 0 | 0 | 2 |

## Owns and may change

- Owns: `$OWN`'s Rust sources, for the forms the note names and nothing else.
- May add: `Harness::env`, sketched in the steps; nothing else is new.
- Pre-approved: the note's `Pre-approved fallout`, each with its default.

## Steps

1. Sweep `$OWN` by the note's table, package by package, and read what the first two rows did not take.
   `Harness` (`temper-client-tools/src/tests/fixtures.rs`, 441) holds `now: Time` and `wall: Wall` of
   its own: it is no `Env`, and tests set both. Those fields stay. Add one method beside its
   `event`, and call it at the seven sites (`tests/fixtures.rs`, `fixtures/acts.rs`,
   `fixtures/waits.rs`, `tests/root.rs` twice, `tests/forge_connection.rs`,
   `tests/jig/account_acts.rs`):

   ```rust
   impl Harness {
       /// The harness's clocks with the limits of the step a test calls.
       pub(crate) fn env<L>(&self, limits: L) -> Env<L> {
           let (now, wall) = (self.now, self.wall);
           Env { now, wall, limits }
       }
   }
   ```

2. Run the three searches; then what the note's `What runs` names.
3. Commit (`D003, D004: <packages> use skein-lib's helpers`, saying why), and push.

## Report

The counts before and after; each site the note's fallout touched; each `#[expect]` you added;
any site you left and why.
