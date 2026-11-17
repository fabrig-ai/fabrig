# codec-shape: execution

2026-11-17. How [codec-shape.md](codec-shape.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole, with its own tests, fixtures, workspace entries and gate rules. Each is a goal the user approves. Plans use the focused and fuzzy suites alone. [layouts-both-ways.execution.md](layouts-both-ways.execution.md) carries the other half, and section 2 orders both.

## 1. The milestones

| Milestone | Draft parts | Needs | Its walk | State |
|---|---|---|---|---|
| K1, one pin: `every` completed (defaults, secret holds, held enumerations, grouped booleans, split functions) with the check that lists the grammar's constructs; each family's memory test built from its schema at run time; the hand-written tests that read golden files rewritten to build their values; every family's golden directory, every generated test and the generator's test emitter removed | 5 | none | none: the suites | not started |
| K2, merges: modules named without versions, no counts, no numbered splits; fingerprints computed from the embedded texts, `jig-protocol-store`'s `generated/layouts.rs` and the `Layouts` row removed; fabrig's merge job regenerating a family whose sources merged clean; `.gitattributes`, `.ignore` and `AGENTS.md`'s line | 6 | K1 | none: the suites | not started |
| K3, lean records on jig's tasks store family alone: the subset's exception for `skein-codec`, the `Record` trait and its helpers, public fields checked at encode, bounds and paths as tables; its lines, compiles and decode times measured | 3 | K2 | none | not started |
| K4, lean records everywhere: every other family, once K3's figures meet 3.3 | 3 | K3 | none | not started |
| K5, generated translation: the maps file and its generator, the shared converters in `skein-codec`, owner by owner: jig's store, temper's store, the worker wires, then the client wires on both ends, each after its wire's version is frozen | 4 | K4 | none | not started |
| K6, the words: skein's codec design and command, the subset, jig's store records and crates, `AGENTS.md` | all | K1 to K5 | none: docs | not started |

## 2. Order

- **K1 first:** it removes the most files at the least cost, and every later milestone then regenerates less.
- **K2 next:** from then on, two goals touching one family no longer collide in generated files.
- **L1 alongside K2** (`layouts-both-ways.execution.md`): it touches only `skein-codec`'s layout module.
- **K3, then K4,** after K2, so their regenerations merge themselves.
- **L2 to L4,** each after L1. A wire's K5 step follows its wire's milestone, so its maps are written once, against the frozen version.
- **The words close each draft.**

## 3. Acceptance

- **K1:** no file under a codec family's `golden/` but `every`'s; the completeness check fails on a schema using an unlisted construct; each family's drift, fuzzing and memory tests pass without reading a file.
- **K2:** the merge job settles a conflict in a family's generated files whose sources merged clean; no generated file holds a `[T; N]` or a version in its name.
- **K3:** tasks' module at most half its lines, with every byte, bound and fingerprint unchanged by the drift test and `every`, and neither compile slower.
- **K5:** each owner's hand-written translation holds only records marked `by hand` and its converters.
