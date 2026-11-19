# codec-shape: execution

2026-11-18. How [codec-shape.md](codec-shape.md) gets built. The user approved both drafts on 11-18, to run as two milestones, one per draft, on the lean design template. This draft is the first milestone, its six goals below. [layouts-both-ways.execution.md](layouts-both-ways.execution.md) is the second, made once this one has ended. Each goal writes its own design from the draft as it runs, and lands whole, with its own tests, fixtures, workspace entries and gate rules. Plans use the focused and fuzzy suites alone.

## 1. The goals

| Goal | Draft parts | Needs | Its walk | State |
|---|---|---|---|---|
| K1, one pin: `every` completed (defaults, secret holds, held enumerations, grouped booleans, split functions) with the check that lists the grammar's constructs; each family's memory test built from its schema at run time; the hand-written tests that read golden files rewritten to build their values; every family's golden directory, every generated test and the generator's test emitter removed | 5 | none | none: the suites | landed |
| K2, merges: modules named without versions, no counts, no numbered splits; fingerprints computed from the embedded texts, `jig-protocol-store`'s `generated/layouts.rs` and the `Layouts` row removed; fabrig's merge job regenerating a family whose sources merged clean; `.gitattributes`, `.ignore` and `AGENTS.md`'s line | 6 | K1 | none: the suites | landed |
| K3, lean records on jig's tasks store family alone: the subset's exception for `skein-codec`, the `Record` trait and its helpers, public fields checked at encode, bounds and paths as tables; its lines, compiles and decode times measured | 3 | K2 | none | landed; lines met, compile not shown slower, figures in [temper/docs/development/codec-shape-k3-figures.md](../../development/codec-shape-k3-figures.md) |
| K4, lean records everywhere: decoding timed alone first; each family's own path and problem types and a sealed `Field`; every other family, owner by owner; then the sealed form, `generate_as` and the `lean` marks removed | 3 | K3 | none | landed |
| K5, generated translation for the stores: the maps file and its generator, the shared converters in `skein-codec`, owner by owner: jig's store, then temper's store. The wires' translation is layouts-both-ways' L5, after their versions freeze | 4 | K4 | none | not started |
| K6, the words: skein's codec design and command, the subset, jig's store records and crates, `AGENTS.md` | all | K1 to K5 | none: docs | not started |

## 2. Order

- **One chain, K1 to K6,** each goal waiting on the one before it.
- **K1 first:** it removes the most files at the least cost, and every later goal then regenerates less.
- **K2 next:** from then on, two goals touching one family no longer collide in generated files.
- **K3, then K4,** after K2, so their regenerations merge themselves. If K3's figures miss 3.3, K4 asks before it starts.
- **K5 after K4,** since the translation it generates targets the lean records.
- **The words close the milestone.**
- **layouts-both-ways follows,** as the second milestone, once this one has ended. Its L4 needs K2, and its L5 the maps file of K5.

## 3. Acceptance

- **K1:** no file under a codec family's `golden/` but `every`'s; the completeness check fails on a schema using an unlisted construct; each family's drift, fuzzing and memory tests pass without reading a file.
- **K2:** the merge job settles a conflict in a family's generated files whose sources merged clean; no generated file holds a `[T; N]` or a version in its name.
- **K3:** tasks' module at most half its lines, with every byte, bound and fingerprint unchanged by the drift test and `every`, and neither compile slower.
- **K5:** each store owner's hand-written translation holds only records marked `by hand` and its converters.
