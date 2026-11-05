+++
id = "forgejo-rename"
part = "d011-rename"
kind = "migration"
needs = ["github-on-core"]
files = ["skein/crates/skein-fake-forge-forgejo/**", "skein/crates/skein-fake-forge-protocol/**", "Cargo.toml", "Cargo.lock", ".config/gate/wasm32.txt", "skein/**/*.rs", "skein/**/Cargo.toml", "temper/**/*.rs", "temper/**/Cargo.toml", "skein/docs/design/**", "skein/docs/development/debt/**", "temper/docs/design/**"]
effort = "medium"
minutes = 60
+++

# The Forgejo fake says so in its name, and D011 is paid

## Goal

`skein-fake-forge-protocol` becomes `skein-fake-forge-forgejo`, with temper's uses following, as the user approved in this
plan. The route's readers name the neutral crate. D011's **Paid when** holds, so the item is deleted. The rename stands
alone, as the last task, so a draft still changing that crate meets it once.

Acceptance:

- `rg -n 'fake.forge.protocol' --glob '!temper/docs/plan/**' --glob '!**/docs/development/debt/reviews/**' .` finds
  nothing.
- `git log --follow` reaches the old path for `skein/crates/skein-fake-forge-forgejo/src/server.rs`: the rename is a `git
  mv`.
- Forgejo's `lib.rs` has `pub use skein_fake_forge_http::{Faults, Hosted};`, as `notes/contracts.md`, 5, and no `git`. No
  file names `skein_fake_forge_forgejo::git`.
- `skein/docs/development/debt/D011-fake-github-on-forgejo-fake.md` is gone. Before it goes, its measure reads 0 and each
  clause of **Paid when** holds; your report says how each does.
- `fake-forge.status.md`'s **Not yet** entry for 12 is gone.
- `python3 -I -B tools/citations/citations.py --all` reads 0 and `python3 -I -B tools/debt/debt.py skein` exits 0.
- `heavy cargo check --workspace --tests` passes, then the packages of step 6. A rename's readers are the workspace, so
  this departs from the implementer's "never the whole workspace", for this check alone.

## Inputs

- `temper/docs/plan/debt-15/notes/common.md`
- `temper/docs/plan/debt-15/notes/contracts.md`, 5
- `skein/docs/design/fake-forge.md`, 12
- `skein/docs/development/debt/D011-fake-github-on-forgejo-fake.md`, whole

## Owns and may change

- **Owns:** the crate's directory, its `Cargo.toml`'s `name`, and the item's deletion.
- **May change, this name alone, in any file that holds it:** `Cargo.toml`'s three places, `Cargo.lock`,
  `.config/gate/wasm32.txt`, each package's `Cargo.toml` dependency line, `use` and path lines in Rust in skein and temper,
  and Markdown under `skein/docs/design`, `skein/docs/development/debt` (never `reviews/`) and `temper/docs/design`. 111
  files and 216 lines at this plan's base. This departs from the implementer's text for those lines of `docs/design/`.
- **May change, the route's readers:** a line naming `skein_fake_forge_protocol::git` becomes `skein_fake_forge_http::git`,
  and a package with such a line gains `skein-fake-forge-http.workspace = true`, in skein and temper alike. `use` groups
  split where they must. A dependency list in `temper/docs/design/protocol/setup.md` or a crate's description gains
  `skein-fake-forge-http` where that package took it.
- **May change:** `skein/docs/design/fake-forge.status.md`: the **Not yet** entry for 12 goes, and its other crate names
  follow.
- **Not to touch:** anything of hearth, smith or jig (`rg` finds no name there at this plan's base); anything under
  `temper/docs/plan/`; a raw review under `docs/development/debt/reviews/`.
- **Options:**
  - The rest of `fake-forge.md` keeps the words "the protocol layer" for Forgejo's: they name section 4. Only the crate's
    name moves.
  - A comment that reads wrongly once renamed, as `Forgejo's HTTP vocabulary over the fake forge`, reads rightly: leave it.
  - If a draft beside this one has landed new files under the old directory, `git mv` them too, and say so.
  - If `.config/gate/wasm32.txt` is ordered by build, keep the new name's place.

## Steps

1. `git mv skein/crates/skein-fake-forge-protocol skein/crates/skein-fake-forge-forgejo`; the root `Cargo.toml`.
2. The sweep of `skein-fake-forge-protocol` and `skein_fake_forge_protocol`, and of `fake-forge-protocol` in
   `skein/docs/design/README.md`'s dependency table: one commit, mechanical.
3. The route's readers, and `git` leaves the re-export: one commit.
4. `heavy cargo check --workspace --tests`.
5. Read D011's clauses against the code. Delete the item. `fake-forge.status.md`. `rg -n 'D011-fake-github' skein temper/docs
   --glob '!temper/docs/plan/**'` finds nothing else to fix.
6. `cargo fmt`; `heavy cargo clippy -p skein-fake-forge-forgejo -p skein-fake-forge-http -p skein-fake-forge-github -p
   skein-fake-peers --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-fake-forge-forgejo -p
   skein-fake-forge-http -p skein-fake-forge-github -p skein-fake-peers-tests -p skein-forge-protocol-world -p
   temper-fake-git -p temper-fake -p temper-playground`; `python3 -I -B scripts/old_forms.py` from `skein/` and `temper/`;
   the citations check; the debt tool. Commit and push.

## Report

The common note's items. Each **Paid when** clause and the evidence for it. How many files and lines the sweep changed. No
break applies to a rename.

## Rebase (2026-11-05, step merge, foreman debt-15/forgejo-rename/foreman-1)

Rebase the rename over main’s migration support and preserve the copy-retry world changes.
The merge conflicts are the branch’s: main added files under the directory this task renames.
Main commit `3118ce5062` added `src/setup/migrate.rs` and `src/tests/migrate.rs` under `skein/crates/skein-fake-forge-protocol/`.
Move both with `git mv` into the corresponding paths under `skein/crates/skein-fake-forge-forgejo/`; retain their module declarations and main’s migration implementation.
In the moved `src/tests/migrate.rs`, remove `git` from the `crate::{...}` import and add `use skein_fake_forge_http::git;`.
In `decoded_clone`, pass `&h.env.limits.git()` to `git::client::decoder` and `git::client::step`; keep the tokenizer’s document limits unchanged.
Keep Forgejo’s root re-export as `pub use skein_fake_forge_http::{Faults, Hosted};`.
Main commit `9e30a6e985` changed `skein/tests/forge/setup/src/host.rs` and `skein/tests/forge/setup/src/world.rs`.
Resolve `host.rs` by retaining main’s domain import, persistent-domain driving, copy bookkeeping and event handling, while renaming the fake import to `skein_fake_forge_forgejo`.
Resolve `world.rs` by retaining main’s `copying` field, initialization and `file`/`put`/`forget` handling, while applying every crate-name substitution from this branch.
Preserve main’s behavior and assertions throughout; these conflicts require name and API adaptation only.
Validate the migration lib tests in `skein-fake-forge-forgejo` and the copy suite in `skein-forge-setup-world` through heavy, then complete the spec’s workspace check and remaining acceptance checks.
Report the two newly landed files moved from the old directory, as the spec requests.
