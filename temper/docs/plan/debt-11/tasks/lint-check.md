+++
id = "lint-check"
part = "check"
kind = "interface"
needs = []
files = ["tools/lints/lints.py", "tools/lints/test_lints.py", "tools/lints/profiles/**", "tools/lints/clippy/**", "tools/lints/design.status.md"]
effort = "high"
minutes = 60
+++

# The check and the definitions

## Goal

Write `tools/lints/`: the four profile definitions, the three `clippy.toml` variants, `lints.py` and its
tests. Nothing else in the repository moves: the manifests move in later tasks, and the gate's run of the
check is `lint-gate`'s, so `python3 -I -B tools/lints/lints.py` reports findings when you finish. That is
expected. Every later task of this plan stands on these files, which no later task changes, so they are
written exactly as the contracts give them.

Acceptance:

- `tools/lints/profiles/{step,harness,binary,ring}.toml` and
  `tools/lints/clippy/{step,ordinary,ring}.toml` hold the text of `notes/contracts.md`, 1.
- `lints.py` does what `tools/lints/design.md`, 5 says, and takes no third-party package. It stays small:
  about 120 lines.
- `python3 -I -B tools/lints/test_lints.py` passes. It stays about 160 lines.
- The run on the repository lists a finding for every package of the contracts' section 2 and none for a
  package on `step`: `python3 -I -B tools/lints/lints.py | rg -c Cargo.toml` reads 135, the sum of that
  section, give or take a package a draft beside this one added. Say the number you read.

## Inputs

- `temper/docs/plan/debt-11/notes/common.md`
- `temper/docs/plan/debt-11/notes/contracts.md`, 1
- `tools/lints/design.md`, 2, 4 and 5
- `tools/citations/citations.py` and `tools/citations/test_citations.py` for the form of a tool here

## Owns and may change

- **Owns:** `tools/lints/lints.py`, `tools/lints/test_lints.py`, `tools/lints/profiles/`, `tools/lints/clippy/`.
- **May change:** `tools/lints/design.status.md`: drop from **Not yet** what you built (sections 2 and 4's
  definitions, section 5's `lints.py`), keeping what is left, with today's date. Nothing else.
- **Options:**
  - If `jig/clippy.toml` holds a line the contracts' step text doesn't foresee, keep the line and say so.
  - If `tomllib` is the shortest way to read `workspace.members`, use it; the members list names some
    packages twice, and each counts once.
  - If a manifest's `[lints]` block is not last in the file, `--set` keeps what follows it.

## Steps

1. Write the definitions, then `lints.py`:

   ```text
   lints.py [PATH...]                  check; PATHs are prefixes, default the four trees
   lints.py --set NAME PATH...         give these manifests profile NAME
   lints.py --write [PATH...]          rewrite blocks and clippy.toml files from their definitions
   ```

   ```python
   TREES = ("skein", "jig", "temper", "hearth")
   HERE = Path(__file__).resolve().parent       # the definitions
   # The repository is the working directory, as citations.py's is.

   def block(lines) -> tuple[int, int] | None
       """The first [lints table's line and the line past its last, trailing comments and blanks out."""
   def profiles() -> dict[str, str]
   def variants() -> dict[str, str]
   def packages(prefixes) -> list[Path]          # workspace.members, once each
   def clippy_files(prefixes) -> list[Path]      # git ls-files
   def check(prefixes) -> list[str]              # "PATH: what is wrong"
   ```

   A block is compared as text, right-stripped. A finding names the path and says which: no block, a second
   block, or a block that matches no profile, with the first differing line against the profile its first
   line names. A `clippy.toml` finding names the path and the first differing line.

2. Write `test_lints.py` in `unittest`, on a small repository built per test under a temporary directory:
   `tools/lints/` copied in, a root `Cargo.toml` naming members under a tree and outside one, `git init`,
   `git add`. Run the copied script with that directory as the working directory.

   ```python
   class Lints(unittest.TestCase):
       def test_a_block_that_is_a_definition_passes(self): ...      # each of the four
       def test_a_changed_level_is_a_finding(self): ...
       def test_an_added_lint_is_a_finding(self): ...
       def test_a_package_with_no_block_is_a_finding(self): ...
       def test_a_second_block_is_a_finding(self): ...
       def test_a_package_outside_the_trees_is_not_read(self): ...
       def test_paths_narrow_the_check(self): ...
       def test_a_member_named_twice_is_one_finding(self): ...
       def test_a_clippy_toml_that_is_no_variant_is_a_finding(self): ...
       def test_set_gives_a_profile_and_keeps_the_rest(self): ...   # tables after it, comments above it gone
       def test_write_follows_a_changed_definition(self): ...       # blocks and clippy.toml
       def test_no_definition_allows_a_disallowed_lint(self): ...   # the four, and the root's real [workspace.lints]
       def test_harness_alone_allows_wildcards(self): ...           # binary and ring deny it
   ```

3. Bite: break each rule once by hand (the text compare, the second-block check, the `disallowed_*` test's
   rule by allowing one in a definition), see the test fail for that reason, restore.
4. `python3 -I -B tools/lints/test_lints.py`, the run on the repository, citations, commit, push.

## Report

The report of your implementer text, and: the count the run on the repository read; the bite table (rule
broken, test, what it read); the sizes of `lints.py` and `test_lints.py`.
