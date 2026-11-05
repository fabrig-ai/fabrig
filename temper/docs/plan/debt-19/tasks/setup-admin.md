+++
id = "setup-admin"
part = "setup-vocabulary"
kind = "feature"
needs = ["setup-replies"]
files = ["skein/crates/skein-forge-setup/src/forgejo/**", "skein/docs/design/forge-setup.status.md", "skein/docs/development/debt/D014-forge-setup-vocabulary.md"]
effort = "high"
minutes = 60
+++

# skein: the admin step is written once

## Goal

The last step of `skein/docs/development/debt/D014-forge-setup-vocabulary.md`: **Paid when** holds, and the item goes. Five Forgejo plans each read the admin's token, ask who it is, and refuse one who isn't an admin. One sub-machine does it, and each plan keeps what differs.

Acceptance:

- `rg -n '!\w+\.is_admin' skein/crates/skein-forge-setup/src -g '!tests/**' -g '!*tests.rs'` lists only `forgejo/admin.rs` (4 files at base). `check.rs` and `reconcile.rs`'s `if user.is_admin` are other rules, and stay.
- `heavy cargo nextest run -p skein-forge-setup -p skein-forge-setup-world` passes with no assertion changed. Each plan's differences hold (below).
- By hand: `forgejo/admin.rs` lets a non-admin pass (the plans' `NotAdmin` tests go red); it reads no token on `Begin` (a plan's test goes red). Restore each.
- The item is deleted, and `skein/docs/design/forge-setup.status.md`'s **Not yet** entry for 4.2 goes.
- Net production falls (estimate -40); the part's whole net is below zero (estimate -60), and your report gives it.

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `skein/docs/development/debt/D014-forge-setup-vocabulary.md`
- `skein/docs/design/forge-setup.md`, 4.2, 4.3
- `skein/docs/foundation/programming-model.md`, 10.3 (no closures, no traits, no generics)

## Owns and may change

- **Owns:** the files in `files`.
- **The shape:**

  ```rust
  // forgejo/admin.rs
  pub(crate) enum Admin { Begin, Read, Check }   // take_over starts at Check
  pub(crate) enum Move { Read, Token(Secret), Passed(User) }
  pub(crate) fn step(admin: &mut Admin, event: Event) -> Result<Move, Why>;
  ```

  Each owner holds an `Admin`, keeps its own credential field and next state, sends its own request on `Move::Token` and stops with `Stage::Admin`.
- **What differs stays with its owner:**

  | Plan | Differs |
  |---|---|
  | `reconcile.rs` | a missing token is `Missing`; next is `UserOf` the bot |
  | `create.rs` | returns `Result<Need, Why>`; keeps `admin_login`; next is `Org` |
  | `adopt.rs` | next is `access.begin()` |
  | `reachable.rs` | `Read(None)` falls back to `ApiRead`, not `Missing`; its `Begin` also checks `installation`; next is `ApiSettings`. It catches `Err(Why::Missing)` in its read alone, which is safe since `account` never yields `Missing` |
  | `take_over.rs` | the token is given in the plan, so it starts at `Check` with no read, and `Missing` at `Begin`; keeps `login`; next is `Write(AdminToken)` |

- **If two free functions fit better than `Admin`,** that's still "written once", but say why in your report: the item asks for a sub-machine.
- **If a stage or a subject in a stop must change,** stop and ask.

## Steps

1. `forgejo/admin.rs`; `reconcile.rs` onto it, then one plan a commit, each compiling and passing.
2. Break the rule by hand, as the acceptance says.
3. Delete the item and the status entry. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-forge-setup --all-targets -- -D warnings`; the tests above; `heavy cargo check --tests -p temper-service -p temper-fake`. The citations check. Commit and push.

## Report

The common note's items, what you broke and what failed, and the part's net production against the plan's base.
