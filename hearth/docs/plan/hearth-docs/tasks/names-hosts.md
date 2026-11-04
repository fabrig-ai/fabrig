+++
id = "names-hosts"
part = "names"
kind = "migration"
needs = []
files = ["hearth/tests/config/examples/**", "hearth/tests/render/fleet/**", "hearth/tests/render/exports/**", "hearth/tests/builder/src/fixtures/**", "hearth/tests/builder/tests/real/**", "hearth/crates/hearth-client-protocol-records/src/tests/**", "hearth/crates/hearth-protocol-config/src/tests/json.rs"]
effort = "high"
minutes = 40
+++

# hearth's fixtures name no private host or address

## Goal

The domain, the hosts and repositories under it and the two addresses leave hearth's code, fixtures and goldens, by the mapping in
`notes/contracts.md`, 1.1. The account `free` isn't this task's (`names-account`), nor are design documents (`names-docs`). Digests in
goldens change for this reason alone.

Acceptance:

- This prints nothing:

  ```sh
  git grep -n -E 'ekanayaka|194\.164\.162\.' -- hearth ':!hearth/docs/plan' ':!hearth/docs/design'
  ```

- The tests below pass, each expected value changed only because the name it holds or hashes changed.
- `python3 -I -B tools/citations/citations.py --all` reads 0 and `outward` passes.

## Inputs

- `hearth/docs/plan/hearth-docs/notes/common.md`
- `hearth/docs/plan/hearth-docs/notes/contracts.md`, 1
- `hearth/docs/plan/hearth-docs/inputs/request.md`, 1
- `hearth/docs/design/domain/render.md`, 9.2: `HEARTH_BLESS=1`

## Owns and may change

- **Owns:** the hits of the acceptance's `git grep`, in the files under `files`, and the expected values and digests that follow from them.
- **May change, and only so:** any other test file of hearth holding such an expected value, for that value alone.
- **Not to touch:** design documents; the account `free`; `hearth/testing/recordings/**`.
- **Options:**
  - A digest, a column or an order that moves because a name changed length or sorted differently is the rename's own effect: update
    it and list it in the report.
  - **A test that fails for any other reason is a finding, not something to update:** leave it, and stop with `Question:`, naming the
    test and the log.

## Steps

1. Apply the mapping to the acceptance's hits. The JSON stays the shape and order it has; the domain reaches `hearth/tests/render/fleet/fleet.json`,
   `envs/lab.json` and `hearth/tests/config/examples/fleet.json`.
2. Bless the render world's goldens: `HEARTH_BLESS=1 heavy cargo nextest run -p hearth-render-world`. It rewrites the expected files and still
   fails. Run again without the variable; read the diff of `hearth/tests/render/**/expected/**`, where only names and digests should move.
3. Update the literals of the builder world and the unit tests from each failure's output, after reading why it failed.
4. `cargo fmt`, commit (`hearth: fixtures use reserved names for hosts and addresses`), then clippy and nextest over
   `-p hearth-client-protocol-records -p hearth-protocol-config -p hearth-domain-render -p hearth-protocol-builder -p hearth-render-world
   -p hearth-builder-world -p hearth-config-world -p hearth`; the checks in `notes/common.md`. Push.

## Report

Files changed by kind; each golden and literal whose digest moved; each column or order the rename moved; test counts.
