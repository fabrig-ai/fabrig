+++
id = "fixture-readers"
part = "layout"
kind = "feature"
needs = ["test-layout", "conformance-tier"]
files = [".config/fixture-readers.toml"]
effort = "medium"
minutes = 45
+++

# hearth's fixtures have readers: 119 tracked paths in `.config/fixture-readers.toml`

## Goal

`.config/fixture-readers.toml` declares the readers of every tracked `**/fixtures/**` file. hearth has
119, and no entry. Add them, now that the tests stand in their last layout.

Acceptance:

- `git ls-files '*/fixtures/*' | grep '^hearth/'` lists 119 paths at this cut, and each path your base
  lists matches exactly one entry's `paths`, by a throwaway check.
- Every reader you add selects at least one test: `heavy cargo nextest list --profile PAIR -p PACKAGE -E
  'FILTER'` for each, its count over 0, by a throwaway check that's gone before you push.
- Provenance only for what `tools/tiers/design.md`, 9, allows. On this cut's reading, nothing of hearth's
  is a provenance file.

## Inputs

- `notes/common.md`, in this plan's directory (`hearth/docs/plan/hearth-workspace/`).
- `tools/tiers/design.md`, 9: the contract and schema; the file's own header and entries, the form to
  follow.
- This cut wrote no design: section 9 holds.

## Owns and may change

**Owns:** `.config/fixture-readers.toml`: new entries at its end, in the file's form, each with its one-line
comment. No existing entry changes.

Pre-approved: a directory's `*.rs` fixture helpers take one entry with a glob, as the jig entries at the file's
head do; a reader's filter may conservatively include other tests of a shared helper, as the header says.

## Steps

1. This cut's count of the 119, by owner: `hearth/crates/` `hearth-client-protocol-records` 14,
   `hearth-domain-builder` 1, `hearth-domain-render` 36, `hearth-protocol-builder` 24; `hearth/tests/` `builder`
   28, `config` 2, `git` 1, `render` 13.
2. Step crates: `pair = "default"`, `package` the crate, `filter = "kind(lib)"`. Worlds: `pair = "default"`,
   `package` the world, `filter = "binary(=suite)"`, with a `fuzzy` reader beside it where a fuzzy test reads
   the helper. `hearth/tests/builder/src/fixtures/reserved/hearth.json` is data: find its reader with `rg`.
3. The acceptance's checks; commit and push.

## Report

The implementer's report, and the number of entries, and the path count they cover.
