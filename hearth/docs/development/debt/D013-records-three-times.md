# D013: Copies beyond render's shared values remain to be examined

- **Kind:** duplication
- **Where:** `hearth-domain-render-vocabulary`, `hearth-domain-render/src/parts`, `hearth-domain-render/src/errors.rs`, `hearth-client-protocol-records`, `hearth-protocol-config`, `hearth-protocol-template`, `hearth-protocol-builder/src/translate`, `hearth-protocol-builder/src/tests`, `hearth-domain-tree/src/vocabulary.rs`, `hearth-protocol-tree/src/tree.rs`
- **Seen:** 2026-11-04 at `66968a30ef`, new, from a search for repeated blocks; the layering it comes from is `hearth/docs/design/README.md`, 4.2
- **Seen:** 2026-11-14 at `2e457d3f00`, in `temper/docs/design/draft/one-owner.md`, 3: `hearth-client-protocol-records`' 162 struct and enum names share 140 short names with render's
- **Checked:** 2026-11-14 at `6b222dfd059733cca9db821c65fbea171c0bdc38`
- **Value:** low (a field added once, if the layering allows)
- **Size:** L
- **Status:** planned (`temper/docs/design/draft/one-owner.md`, 8: adoption beyond the pilot, area by area)
- **Paid when:** a reviewer has examined the remaining areas for copied values and either removed the copies or recorded why each is kept.
- **Measure:** `git ls-files hearth/crates/hearth-protocol-builder/src/translate | xargs cat | wc -l`: 1034

## What

Render's values have one owner, `hearth-domain-render-vocabulary`. The
client's records, the builder's translations and the config, git and
mmdebstrap machines name those values and hold no copy of them
(`hearth/docs/design/domain/render/README.md`, 5.1).

At the checked commit, counting whole lines, including blanks and comments:

- The parts module (`src/parts.rs` and `src/parts/`) and `src/errors.rs`
  total 1,125 lines in `hearth-domain-render-vocabulary`, 1,920 in
  `hearth-domain-render` and 2,100 in `hearth-client-protocol-records`.
  These are values and their operations, render's decisions and re-exports,
  and the client's printers, respectively, not three copies of the shapes.
- `hearth-protocol-builder/src/translate/` is 1,034 lines. Its remaining
  translations compile, project or refuse, rather than copy render's values.
- The translations' tests are 1,137 lines: 322 in `src/tests/translate.rs`
  and 815 under `src/tests/translate/`. Across `src/tests/`, 41 `_back`
  functions check remaining boundaries; none is under `src/tests/fixtures/`.

The copies kept have work of their own:

- Configuration's positioned documents and declarations keep the byte offset
  their decoder orders refusals by. The builder compiles declarations,
  including their expressions, into render's types, rather than copying
  them (`hearth/docs/design/domain/render/README.md`, 5.1).
- The template machine's instructions and expressions are a tool's
  vocabulary (`temper/docs/design/draft/one-owner.md`, 3), not render's
  values. Its translations and their checks stay.
- The client's own records and envelopes add what render doesn't have:
  startup and progress, build layers, the outcome line and `unavailable`,
  and the store's full error with its layers limit
  (`hearth/docs/design/client/README.md`, 3.2).
- The tools' and store's vocabularies still have their translations:
  instruction sequences, bounded requests and outputs, the listing's object
  id printed as hex, and store answers. Config faults are refused or supplied
  with limits (`hearth/docs/design/domain/render/README.md`, 5.1).

Nobody has examined `hearth-domain-tree/src/vocabulary.rs` against
`hearth-protocol-tree/src/tree.rs` for the rule; that area remains open.

## Why it matters

A copied shape makes a new field an edit in each copy, translation and test.
Render's shared vocabulary removes that work for its values without making
its decisions or the client's versioned printed forms shared. A translation
that compiles, projects or refuses is work, not duplication; equal short
names or similar line counts alone don't show a copy.

## What pays it back

Examine the unreviewed areas one at a time under
`temper/docs/design/draft/one-owner.md`, 8. Share values that are the same by
definition where the layering allows it; keep and name the reason where a
layer adds something. Don't merge a tool's vocabulary or the client's own
records into render's just because the shapes resemble one another.
