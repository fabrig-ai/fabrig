+++
id = "list"
part = "list"
kind = "feature"
needs = []
files = [
  "tools/tiers/bundles.toml",
  "tools/tiers/run",
  "tools/tiers/test_run.py",
  "tools/tiers/design.status.md",
]
effort = "medium"
minutes = 45
+++

# The browser bundles in one data file

## Goal

`tools/tiers/run`'s `bundle()` holds each browser pair's bundle in code. Move those values, exactly as they are today, into `tools/tiers/bundles.toml`, the one list of the browser bundles that any runner building them reads, and make `run` read it and keep no list of its own. A missing file, a missing tree or a bad field refuses the pair before any build, naming the file and the field.

Acceptance:

- `tools/tiers/bundles.toml` holds a `[skein]`, a `[jig]` and a `[temper]` table, each with exactly `package`, `out`, `variable`, `options` and `features`, holding today's values of `bundle()` (`tools/tiers/design.md`, 2.2).
- `run` holds no bundle value in code: `bundle()` reads the tree's table and builds the three commands of 2.2 from it; `main` builds a bundle for every browser pair, with no tree list.
- Each refusal of 2.2 fails `run TREE browser` with exit 2, a message naming `tools/tiers/bundles.toml` and the field, and no call but `cargo metadata`.
- `python3 -I -B tools/tiers/test_run.py` passes, and `test_all_bundle_prerequisites` passes unchanged, reading the copy of the real file: it is the check that the file gives today's commands for each tree, byte for byte.
- `tools/tiers/design.status.md` no longer carries the 2.2, 5 entry.

## Inputs

- `tools/tiers/design.md`, 2 and 2.2 (the list, its fields, the commands built from it and the refusals), 4 (exit 2), 5 (what the offline checks hold). Read 2.2 first: it is binding, and another runner outside this repository reads the same file by it.
- `tools/tiers/design.status.md`: the **Not yet** entry `2.2, 5`, which this task closes.
- The request: `temper/docs/plan/bundle-list/inputs/request.md`.

Departures from the implementer's text, and why:

- **No `heavy`, no cargo:** nothing here builds. The check is `python3 -I -B tools/tiers/test_run.py`, run directly (`test_cpu.py` reads nothing this task changes); no `cargo fmt`, clippy or nextest, and no `old_forms.py` (the task touches no tree).
- **The table stays whole:** `test_all_bundle_prerequisites` repeats `bundle()`'s values literally, and the request asks for it to: the file is shared with a runner outside this repository, and a pinned value is what makes a change to the file a deliberate one. Do not cut it to the rows that fail on a rule.
- **No slow tier:** never run `tools/tiers/run` against the real checkout, nor any `browser-*` profile. The tests drive `run` through their fixture checkouts and fake commands only.
- **`tools/tiers/design.status.md` is yours** to remove the one entry this task closes; `tools/tiers/design.md` is not.

## Owns and may change

Owns: `tools/tiers/bundles.toml` (new), `tools/tiers/run`, `tools/tiers/test_run.py`, `tools/tiers/design.status.md` (the `2.2, 5:` clause of the **Not yet** entry, removed whole; the rest of the entry stays as it is).

Reader and neighbour files: none. The trees' own bundle scripts (`temper/scripts/bundle.sh`, `skein/scripts/browser.sh`, `jig/scripts/browser.sh`) stay as they are; nothing else reads `bundle()` (the lead's survey: `git grep` for the values and for `bundle(`).

Pre-approved options:

- **Where the read sits:** in `bundle()`, before its first build, so `cargo metadata` (`target_directory`) has run and nothing else. If keeping the read there needs anything in `main` beyond the one-condition change below, do it there and say so.
- **TOML layout:** one table per tree in the order `skein`, `jig`, `temper`, fields in the order `package`, `out`, `variable`, `options`, `features`, long arrays one option and its value per line. A first comment line citing `tools/tiers/design.md, 2.2`. No other comment.
- **Helper name:** `bundle_entry(tree)` returns the validated table as a `dict`, or raises `FailedStep`. Another name is yours; say it in the report.

## Steps

1. **`tools/tiers/bundles.toml`**, today's values exactly. The temper table, as the shape for the other two (copy theirs from `bundle()` verbatim):

   ```toml
   # The browser bundles, one table per tree (tools/tiers/design.md, 2.2).
   ...
   [temper]
   package = "temper-client-shell-web"
   out = "temper-bundle"
   variable = "TEMPER_TEST_BUNDLE"
   options = [
       "--title", "temper",
       "--placeholder", "temper/crates/temper-client-shell-web/loading.html",
       "--placeholder-stylesheet", "temper/crates/temper-client-shell-web/loading.css",
       "--stylesheet", "temper/crates/temper-client-shell-web/temper.css",
       "--stylesheet", "temper/crates/temper-client-shell-web/task-pages.css",
       "--files", "temper/crates/temper-client-shell-web/files",
       "--noscript", "temper needs WebAssembly and scripts.",
   ]
   features = ["loopback-navigation", "small-window"]
   ```

   `features` holds Cargo feature names, not arguments: `run` builds `["--features", ",".join(features)]` when it is non-empty and nothing when it is empty (2.2, step 1). `skein` and `jig` have `features = []`.

2. **`run`:** a reader beside `bundle()`, then `bundle()` built on it.

   ```python
   def bundle_entry(tree):
       """The tree's table in the bundle list (tools/tiers/design.md, 2.2)."""
       # reads Path("tools/tiers/bundles.toml") relative to the checkout root, as packages.toml is

   def bundle(tree, target, env):
       entry = bundle_entry(tree)
       features = ["--features", ",".join(entry["features"])] if entry["features"] else []
       # wasm module, bundle tool and browser bundle steps as today, from entry's package, options and out
       env[entry["variable"]] = str(target / entry["out"])
   ```

   The refusals, each `raise FailedStep(...)`, so `main` prints `tiers: failed step ...` and returns 2. The messages, binding (tests match on them):

   | Case | `FailedStep` text |
   | --- | --- |
   | file missing, unreadable or not TOML (`OSError`, `ValueError`) | `bundle list: read tools/tiers/bundles.toml from the checkout root: <error>` |
   | no key `<tree>`, or its value no table | `bundle list: tools/tiers/bundles.toml has no [<tree>] table` |
   | a key outside the five | `bundle list: tools/tiers/bundles.toml: <tree>.<key> is no bundle field` |
   | `package` or `variable` missing, not a string, or empty | `bundle list: tools/tiers/bundles.toml: <tree>.<field> must be a non-empty string` |
   | `out` missing, not a string, empty, `.`, `..` or holding `/` | `bundle list: tools/tiers/bundles.toml: <tree>.out must be a plain directory name` |
   | `options` or `features` missing, not an array, or holding a non-string | `bundle list: tools/tiers/bundles.toml: <tree>.<field> must be an array of strings` |

   Check unknown keys first (sorted), then the five in the order `package`, `out`, `variable`, `options`, `features`, and report the first failure. In `main`, `if args.tier == "browser" and args.tree in ("skein", "jig", "temper"):` becomes `if args.tier == "browser":`: the configured browser pairs are exactly those three today, and a browser pair whose tree has no table is refused by the list (2.2). The `shell` local and the three `if tree == ...` branches go.

3. **`test_run.py`:**
   - `setUp` copies the real list into the fixture checkout, `shutil.copyfile(RUN.with_name("bundles.toml"), self.root / "tools/tiers/bundles.toml")`, and keeps the path as `self.bundles`. Every browser test then reads the real file's values.
   - `test_all_bundle_prerequisites` stays as it is.
   - A new `test_bundle_list_refusals_name_the_file_and_field`, shaped as `test_missing_and_invalid_package_maps_refuse_before_commands`:

     ```python
     def test_bundle_list_refusals_name_the_file_and_field(self):
         fields = {"package": '"temper-client-shell-web"', "out": '"temper-bundle"',
                   "variable": '"TEMPER_TEST_BUNDLE"', "options": '[]', "features": '[]'}
         def table(**changes):  # None drops a field; a new key adds one
             ...  # "[temper]\n" + one "key = value" line for each field, after changes
         metadata = [["metadata", "--format-version", "1", "--no-deps"]]
         cases = (
             ("", "has no [temper] table"),
             ("temper = 1\n", "has no [temper] table"),
             ("[temper\n", "read tools/tiers/bundles.toml from the checkout root"),
             (table(title='"temper"'), "temper.title is no bundle field"),
             (table(package=None), "temper.package must be a non-empty string"),
             (table(package='1'), "temper.package must be a non-empty string"),
             (table(package='""'), "temper.package must be a non-empty string"),
             (table(variable='[]'), "temper.variable must be a non-empty string"),
             (table(out=None), "temper.out must be a plain directory name"),
             # out: 1, "", ".", "..", "a/b", "/abs"
             (table(options='"--title"'), "temper.options must be an array of strings"),
             (table(options='[1]'), "temper.options must be an array of strings"),
             (table(features=None), "temper.features must be an array of strings"),
             (table(features='"small-window"'), "temper.features must be an array of strings"),
             (table(features='[true]'), "temper.features must be an array of strings"),
         )
         for contents, message in cases:
             with self.subTest(contents=contents):
                 self.bundles.write_text(contents)
                 result, calls = self.invoke("temper", "browser")
                 self.assertEqual(result.returncode, 2)
                 self.assertIn("tiers: failed step bundle list: ", result.stderr)
                 self.assertIn("tools/tiers/bundles.toml", result.stderr)
                 self.assertIn(message, result.stderr)
                 self.assertEqual([call['args'] for call in calls], metadata)
         self.bundles.unlink()
         # the same four assertions, with "read tools/tiers/bundles.toml from the checkout root"
     ```

     A positive row is not needed: `test_all_bundle_prerequisites` runs all three real tables, two with `features = []`.
   - Bite: break each rule once by hand (drop the unknown-key check, accept an `out` holding `/`, accept a non-string option, read `features` without its check), run the new test, see the row fail for that rule, restore. Also change one option in `bundles.toml` by hand and see `test_all_bundle_prerequisites` fail. The report names each.

4. **`tools/tiers/design.status.md`:** remove the `2.2, 5:` clause from the **Not yet** entry, so it starts again at `4:`.

5. **Checks**, once, from the checkout root: `python3 -I -B tools/tiers/test_run.py` (count the tests it ran against the count before your change: one more), `python3 -I -B tools/citations/citations.py --base origin/main` at 0 new broken. Commit with a message that says why (one list both runners read), and push.

## Report

As the implementer's text says, and: the test counts before and after, each bite (the rule broken, the row or test that failed and its message), the helper's name if not `bundle_entry`, and any departure from the binding messages.
