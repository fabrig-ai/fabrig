+++
id = "absolute"
part = "absolute"
kind = "feature"
needs = []
files = [
  "tests/boundary/tests/outward.rs",
  "tests/boundary/outward.txt",
  "tests/boundary/Cargo.toml",
  ".config/gate/README.md",
  "smith/docs/design/shell.md",
  "temper/docs/design/domain/testing.status.md",
]
effort = "medium"
minutes = 30
+++

## Goal

The outward check becomes absolute: any match in a file it reads fails it. The inventory, its
counts, the `# Raised:` handling and the "fewer is green" logic leave, and
`tests/boundary/outward.txt` is deleted. The matcher and its unit tests stay as they are. The base
holds no match: 25 fixed names plus the `hetzner` form, replayed over 16,905 tracked text files at
`d4bd15e25b`, found none.

Acceptance:

1. `tests/boundary/outward.txt` does not exist, and this prints nothing:

   ```sh
   git grep -n -i -E 'inventory|raised' -- tests/boundary
   git grep -n -E 'fn (needed|entry)' -- tests/boundary/tests/outward.rs
   git grep -n -E 'outward\.txt' -- . ':!*/docs/plan/*'
   git grep -n -w -i 'dogfood' -- smith/docs/design/shell.md
   ```

2. `heavy cargo nextest run -p fabrig-boundary` is green, and `--test outward` selects nine tests.
   `no_file_names_the_lab` passes at your base.
3. A file with a match fails the check, and the failure lists each file, line and match
   (`any_name_fails_and_the_failure_lists_each_file_line_and_match`).
4. Nothing is left out but a plan's directory, the test's own file and binary files
   (`the_check_leaves_out_plan_directories_and_its_own_file`,
   `a_symlinks_target_is_read_as_its_text`).
5. `python3 -I -B tools/citations/citations.py --all` reports 0 on your commit (it reads committed
   trees).

## Inputs

- `temper/docs/plan/outward-zero/inputs/request.md`, all of it.
- `AGENTS.md`, 1, the rule this check holds.
- `DESIGN.md`, 2, for the one status line you change.

This spec departs from your implementer's text once, because the request asks for it: you change
the two lines under `docs/design/` given below, word for word, and no others (the one exception is
the option of the next section).

## Owns and may change

**Owns:**

- `tests/boundary/tests/outward.rs`.
- `tests/boundary/outward.txt`, which you delete.

**One edit each, exactly as given:**

- `tests/boundary/Cargo.toml`, `description`, one line:

  ```text
  Checks the repository's boundaries: the trees' dependencies point one way, and no file names an outside lab
  ```

- `.config/gate/README.md`, item 7, whose last three lines become:

  ```text
     `default`. Rule: `tests/boundary/tests/outward.rs`'s module doc and `AGENTS.md`, 1.
     The rule is absolute: any match fails, and the check has no inventory.
  ```

- `smith/docs/design/shell.md`, 6.5, the `job_wait` row: its status cell
  `provisional; the dogfood's polls` becomes `provisional; the polls of real runs`.
- `temper/docs/design/domain/testing.status.md`, line 63: the word `counts` becomes `matches`, so
  that the test `matches the ~/ form alone`. The file's date line stays.

**Leave as they are:** `AGENTS.md`, 1 (it cites the test file, which stays);
`.config/gate/always.txt` and `.config/nextest.toml` (they name the package, which keeps its name);
every `*/docs/plan/` directory, where older plans cite `outward.txt`: they are left out of both the
outward check and the citations checker, and leave main with their plans.

**Option, pre-approved:**

- If `no_file_names_the_lab` finds a match at your base (a merge brought it): clean that file in
  this task, never except it. Reword the match in generic words (`whoever runs fabrig's
  development`) or remove it, in any tree, `docs/design/` included. If the file matches a glob of
  `.config/gate/read-by-tests.txt`, run the tests that read it. If the match is a shipped string or
  a golden, so that the rewording changes what the product does, stop with a `Question:`.

## Steps

### `tests/boundary/tests/outward.rs`

The sketches bind names and shape.

**The module doc:** the first and last paragraphs become the text below; the paragraph that starts
`Names match ASCII case-insensitively` stays word for word, between them.

```rust
//! fabrig names nothing of whoever runs its development: no repository,
//! tool, machine or session of that lab (AGENTS.md, 1). The rule is absolute:
//! any name in a file the check reads fails it. The check reads git's tracked
//! files from the workspace root with `git ls-files`, so `target/`, `.git/`
//! and other untracked build output are never listed. A symlink's text is its
//! target's path, as git stores it.
//!
//! Three things are left out and nothing else: the directories
//! `*/docs/plan/<draft>/`, the transient run files of whoever runs a plan, each
//! of which leaves main when its plan retires; this file, which spells the
//! names; and binary files, a NUL byte in the first 8000 bytes, as git decides.
//!
//! (the `Names match` paragraph)
//!
//! A file it cannot read is a failure, never a skip.
```

**Removed:** `INVENTORY`, `entry`, `inventory`, `needed`, and the tests
`an_inventory_line_the_check_does_not_read_fails_it`, `fewer_names_than_listed_pass` and
`the_needed_inventory_keeps_the_comments_and_takes_todays_counts`.

**`left_out`** matches `*/docs/plan/<draft>/` and no more: one directory, then `docs/plan`, a draft,
and something inside it. A file directly in `docs/plan/` and a root `docs/plan/` are read; neither
is tracked today.

```rust
/// Whether the check leaves out `path` (from the root, `/`-separated): its own file, and
/// anything inside a plan's directory, `*/docs/plan/<draft>/`.
fn left_out(path: &str) -> bool {
    let parts: Vec<&str> = path.split('/').collect();
    path == OWN || matches!(parts.as_slice(), [_, "docs", "plan", _, _, ..])
}
```

**`scan`** no longer skips a symlink: it reads the link's target path as the link's text, which is
what git stores, so that a name in a link's target fails like any other. Its doc comment says so.
The one tracked link, `smith/tests/transcript-patches/clippy.toml`, holds no name.

```rust
let bytes = if metadata.is_symlink() {
    fs::read_link(&absolute).map_err(|why| format!("{path}: {why}"))?.into_os_string().into_encoded_bytes()
} else if metadata.is_file() {
    fs::read(&absolute).map_err(|why| format!("{path}: {why}"))?
} else {
    return Err(format!("{path}: neither a file nor a symlink"));
};
```

**`verdict`** takes only what was found:

```rust
/// The verdict on `found`: a failure listing each file, line and name, if any file holds one.
fn verdict(found: &BTreeMap<String, Vec<(usize, String)>>) -> Result<(), String> {
    if found.is_empty() {
        return Ok(());
    }
    let mut failure = String::from("Files name the lab running fabrig's development (AGENTS.md, 1):\n");
    for (path, names) in found {
        for (line, name) in names {
            writeln!(failure, "{path}:{line}: {name}").expect("writing to a string succeeds");
        }
    }
    Err(failure)
}
```

### Tests

**Unchanged, byte for byte:** `each_name_of_the_lab_is_found`, `names_on_a_line_never_overlap`,
`product_words_pass`, `a_nul_in_the_first_8000_bytes_makes_a_file_binary`.

**Renamed and changed:**

- `no_file_names_the_lab_beyond_the_inventory` becomes `no_file_names_the_lab`: the same reads, the
  same asserts and the two `eprintln!`s, then `verdict(&scan.found)` panics with the failure.
- `the_check_leaves_out_plans_its_own_file_and_its_inventory` becomes
  `the_check_leaves_out_plan_directories_and_its_own_file`, with this table:

  ```rust
  ("temper/docs/plan/outward-rule/inputs/request.md", true),
  ("smith/docs/plan/sign-in-wait/plan.toml", true),
  (OWN, true),
  ("temper/docs/plan/a.md", false),
  ("docs/plan/a/b.md", false),
  ("temper/crates/a/docs/plan/b/c.md", false),
  ("temper/docs/planning.md", false),
  ("temper/docs/design/plan/a.md", false),
  ("tests/boundary/tests/direction.rs", false),
  ```

- `a_file_beyond_its_count_fails_with_its_names_and_the_needed_inventory` becomes
  `any_name_fails_and_the_failure_lists_each_file_line_and_match`: `a.md` with `(2, "fab")` and
  `(7, "heavy cargo")`, `b.md` with `(1, "hel8")`; the failure contains `a.md:2: fab`,
  `a.md:7: heavy cargo` and `b.md:1: hel8`.

**New:**

- `no_name_passes`: `verdict(&BTreeMap::new())` is `Ok(())`.
- `a_symlinks_target_is_read_as_its_text`: in a `skein_scratch::Scratch`, run `git init -q`, create
  a symlink `link` to `/home/free/elsewhere` with `std::os::unix::fs::symlink`, `git add link`;
  `scan(scratch.path())` finds exactly `("link", [(1, "/home/free/")])`. A small `git` helper in the
  test file is fine, in the style of `scan`'s `Command`.

Bite both new tests and the changed `left_out` table (your implementer's text): put the symlink
skip back; return `Ok(())` from `verdict`; and widen `left_out` to the old `contains("/docs/plan/")`.

### Docs

Make the four edits under `Owns and may change`.

### Checks

Narrowest first, through heavy, one at a time:

```sh
heavy cargo nextest run -p fabrig-boundary --test outward
```

The two design lines can't reach their readers, but `smith/docs/design/**` is read by tests, so run
the two that read `shell.md`: one reads its `> Chat ` quotes, the other its section headings.

```sh
heavy cargo nextest run -p smith-local-domain --lib -E 'test(older_store_words_match_all_three_shell_contract_blockquotes_verbatim)'
heavy cargo nextest run -p smith-bench --test suite -E 'test(every_catalogue_manifest_is_checked_offline)'
```

Then `cargo fmt`; `heavy cargo clippy -p fabrig-boundary --all-targets -- -D warnings`; the old-forms
check for `smith` and `temper`, whose docs you touch, with no count to change;
`heavy cargo nextest run -p fabrig-boundary`; commit; the citations checker of acceptance 5; push.

## Report

As your implementer's text says, and: the count of `--test outward`; whether `no_file_names_the_lab`
found any match at your base, and what you cleaned if so; the three bites and what each failed with.
