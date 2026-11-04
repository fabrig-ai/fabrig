+++
id = "names-account"
part = "names"
kind = "migration"
needs = ["names-hosts", "citations-code"]
files = ["hearth/crates/hearth-client-protocol-records/src/tests/**", "hearth/crates/hearth-domain-render/src/tests/**", "hearth/crates/hearth-protocol-builder/src/tests/**", "hearth/tests/builder/**", "hearth/tests/config/examples/**", "hearth/tests/render/fleet/**", "hearth/tests/render/src/schemas.rs"]
effort = "high"
minutes = 70
+++

# hearth's fixtures name no private account

## Goal

The user's account `free` leaves hearth's code, fixtures and goldens: it becomes `alice`, by `notes/contracts.md`, 1.1. It reads
as one example, so a fixture's account, its group, its home and its password's path all move together. `free` sorts after `ci` and
`alice` before it, and `alice` is a letter longer, so orders and columns move too; a design document isn't this task's.

Acceptance:

- This prints nothing:

  ```sh
  git grep -n -E '/home/free|home-free|free@|Free User|[uU]sers/free' -- hearth ':!hearth/docs'
  ```

- This prints only section 1.2's uses, the English word and Debian's:

  ```sh
  git grep -n -w -i free -- hearth/crates hearth/tests hearth/testing ':!hearth/testing/recordings'
  ```

- The tests below pass, and each expected value changed only for the rename.
- `python3 -I -B tools/citations/citations.py --all` reads 0 and `outward` passes.

## Inputs

- `hearth/docs/plan/hearth-docs/notes/common.md`
- `hearth/docs/plan/hearth-docs/notes/contracts.md`, 1
- `hearth/docs/plan/hearth-docs/inputs/request.md`, 1
- `hearth/docs/design/domain/render.md`, 9.2: `HEARTH_BLESS=1`
- `names-hosts`' commit, for how the goldens moved

## Owns and may change

- **Owns:** the account's name in the files under `files`, and what follows from it.
- **May change, and only so:** any other test or fixture of hearth that holds the account, or an expected value that follows from it.
  Hits outside `files`: `hearth/testing/hearth-fake-mmdebstrap/src/tests/resolve.rs` and `hearth/testing/hearth-fake-machine/src/linux/files.rs`
  are the English word, and stay.
- **Not to touch:** step code. No `src/` file outside `tests/` holds the account; if one does, stop and say so. Design documents;
  `hearth/testing/recordings/**`.
- **Options:**
  - An order, a column, a width, a digest or an id that moves because `alice` sorts or measures otherwise is the rename's own effect:
    update it and list it in the report. Known: the rows of `hearth/crates/hearth-domain-render/src/tests/additions.rs` come out by
    user name, so its `ci` line now follows `alice`; a collision's `first` and `second` may swap; `at(2, 15)`-style positions in
    `hearth/crates/hearth-domain-render/src/tests/schema.rs` follow the key's length.
  - Where the fixture already holds an `alice` beside `free`, `free` becomes `bob` (`notes/contracts.md`, 1.1).
  - **A test that fails for any other reason is a finding, not something to update:** leave it, and stop with `Question:`, naming the
    test and the log.

## Steps

1. Find the hits, reading each, since a blind `sed` breaks `non-free`:

   ```sh
   git grep -n -E '(b"|")free"|free[ ,]+[0-9]{4}|\bfree [a-z/]|[uU]sers/free|home-free|/home/free|free@|"free|Free User|free\.passwd|free\\' -- hearth/crates hearth/tests hearth/testing
   ```

   39 files on this plan's base, most in `hearth-domain-render`'s and `hearth-client-protocol-records`' tests, and
   `hearth/tests/builder/tests/real/*`. JSON keeps the shape and order it has, so `hearth/tests/render/fleet/fleet.json` and
   `hearth/tests/config/examples/x1.json` stay the same documents with one name changed. The two `examples` files must agree with
   each other as they did.
2. Update `.rs` fixtures and tests. Bless the render world as `names-hosts` did (`HEARTH_BLESS=1 heavy cargo nextest run -p hearth-render-world`,
   which fails by design, then without the variable), and read the diff of `expected/`: only names, columns, orders and digests should move.
3. Update the remaining literals from each failure's output, after reading why it failed.
4. `cargo fmt`, commit (`hearth: fixtures use a reserved account`), then clippy and nextest over `-p hearth-client-protocol-records
   -p hearth-domain-render -p hearth-protocol-builder -p hearth-protocol-config -p hearth-render-world -p hearth-builder-world
   -p hearth-config-world -p hearth-accounts-world -p hearth-fake-machine -p hearth`; the checks in `notes/common.md`. Push.

## Report

Files changed by kind; each place `bob` was taken; each order, column, id and digest the rename moved; test counts; any name that looked
private and isn't in the contract.
