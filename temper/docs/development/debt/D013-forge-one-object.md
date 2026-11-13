# D013: The forge's top is one object of 117 fields spread over 35 files

- **Kind:** complexity
- **Where:** `temper-engine-domain-forge/src/domain.rs` (`Domain`, `Domain::new`), `temper-engine-domain-forge`
- **Seen:** 2026-11-13 at `4d06c93e26`, designing milestone G5 of `temper/docs/design/draft/crate-graph.execution.md` (new here)
- **Checked:** 2026-11-13 at `4d06c93e26`
- **Value:** medium (a reader can find a concept's state in one place; the top can then split along its parts, under the size bound)
- **Size:** L
- **Status:** open
- **Paid when:** `Domain`'s state is grouped into owned parts by concept, each with its own `impl` in its own module, `Domain::new` builds them rather than every field, and the top splits along them so that `temper-engine-domain-forge` leaves the size check's exceptions (`tools/crates/design.md`, 3.1).
- **Measure:** `awk '/^pub struct Domain \{/,/^\}/' temper/crates/temper-engine-domain-forge/src/domain.rs | grep -cE '^\s+(pub(\(crate\))? )?[a-z_]+:'` (117 at base)

## What

`Domain` is the forge connector's top: the state machine of
`temper/docs/design/domain/forge/README.md`, section 3.

- `Domain` has 117 fields;
- `impl Domain` blocks sit in 35 files
  (`rg -l '^impl Domain' temper/crates/temper-engine-domain-forge/src | wc -l`);
- the crate is about 79,000 lines with its own tests, over the size
  check's 60,000 (`tools/crates/design.md`, 3.1).

## Why it matters

Every concept reads and writes the same fields, so a change to one
concept's state can reach code in any of 35 files. And `Domain` can't be
cut as it stands (`temper/docs/design/draft/crate-graph.md`, 5.2): G5
moves only the top's vocabulary out (`temper/docs/design/domain/forge/README.md`,
section 3), so the top stays over the size bound and every landing that
touches it recompiles all of it.

## What pays it back

Owned parts (adoption and resources, changes and their landing, tiers,
follows, findings and views) with their methods, then a split of the top
along them, each part a crate where it saves the landings that don't
touch it a recompile of several seconds (`crate-graph.md`, 5.2). D012's
split of `domain.rs` and `landing.rs` along the design's parts is the
same work at the file level.
