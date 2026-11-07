# bundle-list: the slow tiers' browser bundles in one data file

`tools/tiers/run`'s `bundle()` holds each browser pair's bundle in code: the wasm package, the output directory under the target directory, the environment variable, `skein-ui-bundle`'s asset options and the cargo features. Another runner outside this repository builds the same bundles for the same pairs from a copy of that list, and the two have drifted: the copy lacks `--title temper` and the `--placeholder` and `--placeholder-stylesheet` options added on 11-06 (`e8600f44be`, `22b34b222e`), so it tests a bundle that isn't the one `tools/tiers/run` builds. One list, as data, that both read.

One direct task, in `tools/tiers/`:

1. **`tools/tiers/bundles.toml`** holds, per tree (`skein`, `jig`, `temper`): `package`, `out`, `variable`, `options` (an array of strings, asset paths relative to the checkout root) and `features` (an array, empty when none), with today's values exactly.
2. **`tools/tiers/run` reads it** in `bundle()` and keeps no list of its own. A missing file, a missing tree or a field of the wrong type fails the pair's setup with a message naming the file and the field, before any build.
3. **Tests, focused only (`tools/tiers/test_run.py`):** the parsed file gives today's commands for each tree, byte for byte; each failure in 2 is refused with its message. No slow tier is run, by the plan or its tests.
4. **`tools/tiers/design.md`, 2** names the file as the one list of the browser bundles, read by any runner that builds them.

The trees' own bundle scripts (`temper/scripts/bundle.sh` and the like) stay as they are: not this plan.
