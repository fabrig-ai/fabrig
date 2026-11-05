+++
id = "hearth-values"
part = "hearth-values"
kind = "migration"
needs = []
files = ["hearth/docs/design/README.md", "hearth/docs/design/domain/build.md", "hearth/docs/design/domain/render.md", "hearth/docs/design/domain/store.md", "hearth/docs/design/draft/brainstorm.md", "hearth/docs/design/protocol/format.md", "hearth/tests/config/examples/fleet.json", "hearth/tests/config/examples/tasks/locale.json", "hearth/tests/render/fleet/fleet.json", "hearth/tests/render/fleet/expected/**", "hearth/tests/render/fleet/rendered/*/etc/localtime", "hearth/tests/render/fleet/tasks/locale/tree/etc/localtime", "hearth/tests/builder/tests/suite/goldens.rs"]
effort = "medium"
minutes = 40
+++

# The example fleet's SSH port and zone become neutral

## Goal

The example fleet's SSH port becomes `2222` and its zone `Etc/UTC`, everywhere under `hearth/`. A sentence and the open questions about these values leave the design. Only example values change.

**Departure from your text:** `docs/design/` and a draft are yours here, for the lines the table in Step 2 names and nothing else, because the request names them and the design's examples change with the fixtures.

Acceptance:

- With the values of Step 1, these print nothing:

  ```sh
  git grep -n -I -w "$port" -- hearth
  git grep -n -I -i "$city" -- hearth
  git ls-files -s hearth | awk '$1=="120000"{print $4}' | xargs readlink | grep -i "$city"
  ```

- `git ls-files -s hearth | awk '$1=="120000"{print $4}' | xargs readlink` prints `/usr/share/zoneinfo/Etc/UTC` three times, and git still records all three paths as symlinks (mode 120000).
- `git diff --stat` against your base shows only the files under `files`, and under `hearth/tests/render/fleet/expected/` a diff of only the lines Step 4 allows.
- `tools/citations/citations.py --all` reads 0 and `outward` passes (`notes/common.md`, 4).
- No memory ceiling moved, unless the option below applies.
- The tests of Step 6 pass.

## Inputs

- `hearth/docs/plan/hearth-names-2/notes/common.md`: binding
- `hearth/docs/plan/hearth-names-2/inputs/request.md`
- `hearth/docs/design/domain/render.md`, 9.2: the goldens and `HEARTH_BLESS=1`
- `hearth/docs/design/protocol/format.md`, 3.7 and 5.10: the examples this changes

No test depends on these design files' words: `.config/gate/read-by-tests.txt` lists none of hearth's, and a search of hearth's and `tests/`'s Rust finds no reader but `outward`, which reads every tracked file for names.

## Owns and may change

- **Owns:** the lines of Step 2's table, the three symlinks and the blessed goldens.
- **May change, and only so:** a digest literal in another of hearth's tests that fails because the fleet's values moved it: that literal alone, listed in the report. At base no test is known to hold one.
- **Not to touch:**
  - `hearth/testing/mirror/universe`: its line 404 holds the port's digits by chance, inside a package's SHA256. A replacement without word boundaries would corrupt it.
  - A bare area word with no zone beside it, in `hearth/crates/hearth-client-protocol-records/src/tests/**`, `hearth/crates/hearth-domain-render/src/tests/**`, `hearth/tests/builder/src/fixtures/packages.rs` and `hearth/tests/builder/tests/real/main.rs`: those are debconf values of their own fixtures, not the fleet's zone.
  - `hearth/docs/design/protocol/format.md`, 3.5, its container name and base id, and `hearth/docs/design/domain/slots.md`: the plan's question `container-name` holds them.
  - Every status file: none names these values or the sentence.
  - Anything outside `hearth/`.
- **Options:**
  - A memory test that fails: run the same test at your base and on your branch and read both peaks. If the peak moved with these values' bytes, move that ceiling by what the measurement needs, and give both peaks and why in the report. If not, `notes/common.md`, 3 applies.
  - The line numbers are the base's. If one is off because your base moved, find the line by its words: a section's content, not its number, is what changes.
  - `golden.rs` may have moved to `hearth/testing/hearth-test-render/src/` if `debt-21/hearth-kit-render` has landed, and `hearth/tests/builder/tests/suite/goldens.rs` may hold another draft's one-line change: neither changes these steps. If the bless in Step 4 no longer works as written, `git grep -n HEARTH_BLESS -- hearth` finds its place; use it and say so in the report.

## Steps

1. **Read the old values, before any edit,** into an untracked `values.sh` at the slot's root, which each later command sources. Check that both are non-empty.

   ```sh
   cat > values.sh <<'SH'
   port=$(git show "BASE:hearth/docs/design/protocol/format.md" | sed -n 442p | grep -oE '[0-9]+')
   zone=$(git show "BASE:hearth/docs/design/draft/brainstorm.md" | sed -n 342p | grep -oE '[A-Z][a-z]+/[A-Z][a-z]+')
   area=${zone%/*}; city=${zone#*/}
   SH
   sed -i "s/BASE/$(git rev-parse HEAD)/" values.sh
   ```

2. **Edit these lines** (line numbers at base). A number or a zone is replaced whole, keeping each line's padding. The new area is `Etc` and the new city `UTC`.

   | File | Line | Becomes |
   |---|---|---|
   | `hearth/docs/design/protocol/format.md` | 440 | `"area": "Etc", "zone": "UTC"` |
   | | 442 | `"port": 2222,` |
   | | 714, 715 | the two `doc` strings read `such as Etc.` and `such as UTC.`: a zone's example must lie in the area's |
   | | 1155 | the row opens `Etc/UTC and en_US preseeds` |
   | | 1159 | the row opens `SSH port 2222` |
   | | 2387 to 2392 | this whole bullet goes; section 13 keeps its number and its other bullets (the request's item 4) |
   | `hearth/docs/design/domain/render.md` | 1387 | the link's target is `/usr/share/zoneinfo/Etc/UTC` |
   | `hearth/docs/design/domain/build.md` | 1071 | `os's Etc/UTC and en_US preseeds` |
   | `hearth/docs/design/draft/brainstorm.md` | 342 | `the Etc/UTC timezone and en_US locale preseeds` |
   | `hearth/docs/design/domain/store.md` | 1302 to 1305 | drop the sentence that starts at the end of line 1302 and ends on line 1305, about how the fleet's containers clear the range. The rule keeps its words and ends `overlaps it [M5].` Rewrap the bullet if it needs it. |
   | | 1472 to 1476 | this whole bullet in section 12 goes: it asks whether that sentence stays |
   | `hearth/docs/design/README.md` | 753 to 756 | this last bullet of section 9 goes: it asks the same two questions. Line 752 now ends the list, so its `;` becomes `.` |
   | `hearth/tests/config/examples/fleet.json` | 15, 17 | `"area": "Etc", "zone": "UTC"`; `"port": 2222,` |
   | `hearth/tests/config/examples/tasks/locale.json` | 6, 7 | as `format.md`, 714 and 715 |
   | `hearth/tests/render/fleet/fleet.json` | its one line | `"area":"Etc","zone":"UTC"`; `"port":2222` |
   | `hearth/tests/builder/tests/suite/goldens.rs` | 96 | the link's target is `/usr/share/zoneinfo/Etc/UTC`; it is the line `render.md`, 9.2 prints |

   The last three design rows (the `store.md` bullet, the `README.md` bullet and the `;`) aren't in the request's list. They go because they ask what the request answers, and the plan says so.

3. **Point the three symlinks at the new zone.** `git grep` doesn't search a symlink's target, so the searches miss these:

   ```sh
   for f in hearth/tests/render/fleet/tasks/locale/tree/etc/localtime \
            hearth/tests/render/fleet/rendered/x1-prod/etc/localtime \
            hearth/tests/render/fleet/rendered/east-prod/etc/localtime; do
     ln -sfn /usr/share/zoneinfo/Etc/UTC "$f"
   done
   ```

   The first is the fixture. The other two are witnesses the golden harness checks, and no bless rewrites them.

4. **Bless the fleet's goldens, from the builder world:** it is the only world that blesses. The test rewrites its expected files and fails on purpose, so run it again without the variable.

   ```sh
   HEARTH_BLESS=1 heavy cargo nextest run -p hearth-builder-world --test suite -E 'test(=goldens::the_fleet_renders_its_six_targets_as_expected)'
   heavy cargo nextest run -p hearth-builder-world --test suite -E 'test(=goldens::the_fleet_renders_its_six_targets_as_expected)'
   ```

   Then read `git diff -- hearth/tests/render/fleet/expected`. Expect `x1-prod` and `east-prod` to change. Only the `tzdata/Areas` and `tzdata/Zones/...` preseed lines, the `/etc/localtime` line and 12-digit digests may move. Another target's golden may move only in its digests, if its inputs hold the port. Anything else is a finding (`notes/common.md`, 3).

5. **Run the acceptance's searches and read the whole diff by kind.**

6. `cargo fmt`, then, one command at a time:

   ```sh
   heavy cargo clippy -p hearth-builder-world --all-targets -- -D warnings
   heavy cargo nextest run -p hearth-render-world -p hearth-builder-world -p hearth-config-world -p hearth-template-world
   heavy cargo nextest run --profile fuzzy -p hearth-render-world -p hearth-builder-world -p hearth-config-world -p hearth-template-world
   ```

   These hold every reader of the fleet's files: the render and builder worlds read the fleet, the config world reads the examples, and the config and template worlds' fuzzy suites read the corpus. The memory ceilings live in the same suites. Then `python3 -I -B hearth/scripts/old_forms.py`.

7. **Commit with messages that spell no old value** (`hearth: the example fleet's SSH port and zone are neutral`), adding files by name. Delete `values.sh`, push, then run the checks of `notes/common.md`, 4.

## Report

Files changed by kind; each golden that moved and which of its digests; any literal outside the table, with why; any ceiling touched, with both peaks; test counts. It spells no old value.
