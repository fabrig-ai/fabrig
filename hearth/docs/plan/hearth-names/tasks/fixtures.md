+++
id = "fixtures"
part = "names"
kind = "migration"
needs = []
files = ["hearth/tests/config/examples/**", "hearth/tests/config/src/examples.rs", "hearth/tests/config/tests/suite/world.rs", "hearth/tests/render/fleet/**", "hearth/tests/render/exports/**", "hearth/tests/render/errors/exported/**", "hearth/tests/builder/src/fixtures.rs", "hearth/tests/builder/src/fixtures/**", "hearth/tests/builder/tests/real/**", "hearth/crates/hearth-client-protocol-records/src/tests/**", "hearth/crates/hearth-domain-render/src/tests/**", "hearth/crates/hearth-protocol-builder/src/tests/fixtures/exports.rs", "hearth/crates/hearth-protocol-config/src/tests/document.rs"]
effort = "medium"
minutes = 60
+++

# hearth's code and fixtures name no real machine, provider or key

## Goal

Apply the mapping (`hearth/docs/plan/hearth-names/notes/names.md`, 1) to hearth's tests, fixtures and goldens, by script, then read the diff. The design documents are already done. Only example names change: no production code holds one, and no test's meaning changes.

Acceptance:

- These three print nothing:

  ```sh
  git grep -n -i -E 'serverhino|serve rhino|ionos|vultr|m3NY8|\b(rhi|ion|vul)\b|\b(rhi|ion)-[a-z]+' -- hearth ':!hearth/docs/plan/hearth-names'
  git grep -n -E '\b(RHI|ION|VUL)_|_(RHI|ION|VUL)(\b|_)' -- hearth ':!hearth/docs/plan/hearth-names'
  git ls-files hearth | grep -v '^hearth/docs/plan/hearth-names/' | grep -E '(^|[/_.-])(rhi|ion|vul)([/_.-]|$)'
  ```

- The tests in Steps 5 and 6 pass, each expected value changed only for a reason in `notes/names.md`, 2.
- `git status` and `git diff --stat` against your base show nothing outside `hearth/`, nothing under `hearth/docs/` and nothing under `hearth/testing/`.
- Under `hearth/tests/render/`, only `fleet`, `exports` and `errors/exported` changed.
- No memory ceiling moved, unless the option below applies.
- After your commit, `python3 -I -B tools/citations/citations.py --all` reads 0 and `heavy cargo nextest run -p fabrig-boundary --test outward` passes.
- The focused suite's budget (`[gate.hearth.default]` in `tools/tiers/budgets.toml`) is measured by the trunk's end check. A rename adds no work: give the time nextest printed for Step 5's run in your report.

## Inputs

- `hearth/docs/plan/hearth-names/notes/names.md`, 1 and 2: binding
- `hearth/docs/plan/hearth-names/inputs/request.md`
- `hearth/docs/design/domain/render.md`, 9.2: the goldens and `HEARTH_BLESS=1`
- `hearth/tests/render/src/golden.rs`: what a golden and a witness are

## Owns and may change

- **Owns:** the hits of the acceptance's searches in the files under `files`, the files and directories they rename, and the expected values that follow from them.
- **May change, and only so:** any other test file of hearth holding such an expected value, for that value alone.
- **Not to touch:** `hearth/docs/`; `hearth/testing/recordings/**`; production code outside `src/tests/`; anything outside `hearth/`.
- **The script is throwaway:** it lives untracked in the slot and is deleted before you push.
- **Options:**
  - A pad, position, digest or order that moves as `notes/names.md`, 2 says: update it and list it in the report.
  - A limit in a test computed from a literal's bytes: update it to the new length and list it.
  - A memory test that goes over its ceiling: run the same test at your base and on your branch and read both peaks. If the peak moved with the names' bytes, raise that ceiling by what the measurement needs, and give both peaks and the reason in the report. If it didn't, this is the case below.
  - **A test that fails for any other reason is a finding, not something to update:** leave it, push what you have, and stop with `Question:`, naming the test and its log.

## Steps

1. **Rename files with `git mv`, parents first.** The third search lists 27 files at base: the directories `hosts/ion` and `hosts/rhi` in `hearth/tests/render/{fleet,exports,errors/exported}/`, the files in them, `expected/*` and `rendered/*` of those worlds, and `hearth/tests/config/examples/ion-prod.json`. Commit this alone, so git keeps the history of each.
2. **Rewrite the contents by script,** over the files the first two searches list, outside `hearth/docs/`:
   - The hosts match case-sensitively on `(?<![A-Za-z0-9_])(rhi|ion|vul)(?![A-Za-z0-9_])`. Read the hits first: at base every one is a host, and `vul` has none outside the documents.
   - Then the three providers, the key fragment and the two constants of `notes/names.md`, 1.
   - A run of four or more spaces after a renamed name gives up what the name gained (`notes/names.md`, 2).
   - `tests/render/*/rendered/**` are witnesses written independently of the output: the script rewrites them, and nothing regenerates them.
3. **Bless the goldens, from the builder world:** that world is the only one that blesses.

   ```sh
   HEARTH_BLESS=1 heavy cargo nextest run -p hearth-builder-world --test suite --no-fail-fast -E 'test(/^goldens::/)'
   ```

   Each blessed test rewrites its expected files and fails on purpose. Run it again without the variable. Then read `git diff -- hearth/tests/render`: only names, 12-digit digests and the order of whole targets may move. Any other golden changing is a finding.
4. **Update the remaining literals from each failure,** after reading why it failed: `hosts/west/west-prod.json:1:97:` and the `/etc/vpn/peers` digests in `hearth/tests/builder/tests/real/exports.rs`, and whatever else `notes/names.md`, 2 explains.
5. `cargo fmt`, then, one command at a time:

   ```sh
   heavy cargo clippy -p hearth-client-protocol-records -p hearth-protocol-config -p hearth-domain-render -p hearth-protocol-builder -p hearth-render-world -p hearth-builder-world -p hearth-config-world -p hearth --all-targets -- -D warnings
   heavy cargo nextest run -p hearth-client-protocol-records -p hearth-protocol-config -p hearth-domain-render -p hearth-protocol-builder -p hearth-render-world -p hearth-builder-world -p hearth-config-world -p hearth
   ```

6. `heavy cargo nextest run --profile fuzzy -p hearth-render-world -p hearth-builder-world -p hearth-config-world`: the fuzzy suites read the same fixtures. Then `python3 -I -B hearth/scripts/old_forms.py`.
7. **Read the whole diff by kind, then run the acceptance's searches.** Commit with messages that spell no old name (`hearth: fixtures use neutral host and provider names`), push, and run the citations check and `outward`.

## Report

Files changed by kind; each golden and literal whose digest moved; each pad, position and order the rename moved; any ceiling or limit touched, with its measurement; test counts and the time of Step 5's run.
