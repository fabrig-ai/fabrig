# bundle-list: the slow tiers' browser bundles in one data file

The request: `inputs/request.md`. `tools/tiers/run`'s `bundle()` holds each browser pair's bundle in code; a runner outside this repository builds the same bundles from a copy, and the copy has fallen behind. One list, as data, that both read.

## Parts and tasks

| Part | Task | Needs | What it makes run | Effort, minutes |
|---|---|---|---|---:|
| `list` | `tasks/list.md` | | `tools/tiers/bundles.toml`, read by `run`'s `bundle()`; today's commands byte for byte; each bad list refused before any build | medium, 45 |

**Critical path:** `list`, about 45 minutes. One task, as the request asks: four files in `tools/tiers/`, no crate, no build.

## Design

Written by the cut, released with the task: `tools/tiers/design.md`, 2 (each browser pair builds from the list), 2.2 (new: the list's fields, the three commands a runner builds from a table, the refusals), 5 (the offline checks gain the list's), 8 (the trees' own bundle scripts read no list, left for later) and 10.4 (new: the list in the caller's contract). `tools/tiers/design.status.md` carries **Not yet** 2.2, 5, which the task removes.

## What was decided

- **`features` holds feature names,** `["loopback-navigation", "small-window"]`, not Cargo arguments; a runner passes `--features` with them joined by commas, and nothing when the array is empty. Data rather than argv, so the other runner needs no Cargo syntax from the file.
- **`out` is a plain directory name** beneath the target directory, refused otherwise, so section 2's "bundle output directories are beneath `target_directory`" still holds now that the value is data.
- **A field the list does not define is refused,** as `packages.toml`'s binary records are: a field one runner reads and the other ignores would be the drift this plan ends.
- **`main` keeps no tree list either:** `args.tree in ("skein", "jig", "temper")` goes, and every browser pair builds its tree's bundle. The configured browser pairs are those three today; a new one with no table is refused by the list.
- **The read sits in `bundle()`,** after `cargo metadata` and before the first build, as the request puts it; the refusal tests pin that the metadata call is the only one.
- **`test_all_bundle_prerequisites` stays whole,** against a copy of the real file: the request's byte-for-byte check. The rule that a table pinning a refactor shrinks is waived for it, and the spec says so: the file is shared, and the pinned values make a change to it deliberate.
- **The trees' own bundle scripts** stay as they are (the request), and design 8 records it as later.

## Overlaps

None: no other draft beside this one names a file in `tools/tiers/`.
