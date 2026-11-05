# D013: A record's shape is written up to three times, with hand translations both ways

- **Kind:** duplication
- **Where:** `hearth-protocol-config`, `hearth-domain-render/src/parts`, `hearth-domain-render/src/errors.rs`, `hearth-client-protocol-records/src/parts`, `hearth-client-protocol-records/src/errors.rs`, `hearth-protocol-builder/src/translate`, `hearth-protocol-builder/src/tests/fixtures`
- **Seen:** 2026-11-04 at `66968a30ef`, new, from a search for repeated blocks; the layering it comes from is `hearth/docs/design/README.md`, 4.2
- **Checked:** 2026-11-05 at `13fb737baf`
- **Value:** low (a field added once, if the layering allows)
- **Size:** L
- **Status:** kept (the layers' own vocabularies)
- **Paid when:** a reviewer sees either fewer hand translations, or this item `kept` with its reason.
- **Measure:** `git ls-files hearth/crates/hearth-protocol-builder/src/translate | xargs cat | wc -l`: 2869

## What

A part or an error exists in the render domain and again in the client's
records, and a declaration exists in config and again in render. The
protocol layer translates each, field by field:

- `hearth-domain-render`'s parts and errors are 3,224 lines, and
  `hearth-client-protocol-records`' 4,228. `parts/packages.rs` and
  `errors.rs` of each repeat most type definitions: 109 runs of eight lines
  in `errors.rs` alone.
- `hearth-protocol-builder/src/translate` is 2,869 lines.
- `hearth-domain-tree/src/vocabulary.rs` and
  `hearth-protocol-tree/src/tree.rs` are the same shape, on a smaller scale.
- The translations' tests add 3,446 lines, and `src/tests/fixtures` holds 177 `_back`
  functions translating the other way, for round trips.

## Why it matters

A new field is edited in each copy, in each translation and in its test.
Yet the copies are what the layers ask for: a domain names no protocol
machine (`skein/docs/foundation/programming-model.md`, 4), and a record's
wire form has a version of its own.

## What pays it back

Don't merge the vocabularies by default. Ways to shrink it: a shared
vocabulary crate for the parts that are the same by definition, if the
layering allows it, or generated translations and round-trip checks. If
neither is worth it, set the status to `kept` with the layering as reason.
