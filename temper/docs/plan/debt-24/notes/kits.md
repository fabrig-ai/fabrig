# debt-24: a world's shared pieces move to a kit (skein D028, jig D031)

Every kit task names this note. A read-only survey at this plan's base found the modules and readers each spec lists: re-measure first, and find code by name.

## The recipe

`kit-packages` has made every kit: an empty `<tree>/testing/<kit>/`, on `harness`, with its `clippy.toml`, already a workspace member and a workspace dependency. A kit task then:

1. `git mv`s the modules its spec names from the world's `src/` to the kit's `src/`, with their `pub mod` lines. Git must see a rename: no reformatting in the same commit.
2. Leaves the world's `src/lib.rs` its remaining modules and adds `pub use <kit>::*;`. So the world's own `tests/**` and remaining `src/` keep their paths, `crate::reference` included. A `use` in that remaining code may name the kit where the compiler asks.
3. Gives the kit the dependencies its modules use, and drops from the world what it no longer uses.
4. For each reader its spec names: swaps the world's manifest line for the kit's, keeping dev or normal, and renames the crate path in that package's `.rs` files, with a one-shot untracked `sed`.
5. Regenerates `Cargo.lock` with `cargo metadata --offline --format-version 1 >/dev/null`, never by hand.

A reader its spec doesn't name stays as it is, in any tree: the re-export keeps it compiling until its own task.

## The rules

- **A kit's name ends in `-kit`,** never `-world`. No package is renamed to empty a search. No alias (`x_world = { package = "x-kit", .. }`).
- **No test changes what it asserts.** Test names and counts hold: `heavy cargo nextest list -p <world> -p <kit>` together count what the world counted alone, and a moved module's unit tests now belong to the kit. The tests move with the module; they aren't rewritten or retimed.
- **Seeds replay.** Nothing changes an order of draws or events.
- **A fixture the moved code reads stays where it is,** with its `include_bytes!` path following. If it has a row in `.config/fixture-readers.toml`, the row names the new package (`tools/tiers/design.md`, 9).
- **The package names in `tools/tiers/packages.toml` and `.config/nextest.toml` stay:** the world still exists. A moved test a tier's filter selects by binary stays selected, or stop and ask.
- **Cycles.** A kit depends on no `-world` package, but for the two `jig/docs/design/domain/testing.md`, 7 names. A crate under test may dev-depend on a kit only if the kit doesn't depend on that crate: two copies of its types won't meet.
- **Lints.** A kit's code holds its `clippy.toml`; a scoped `#[expect]` with a reason where a moved item trips one, never a rewrite.
- **Old forms.** If `<tree>/scripts/old_forms.py` counts by path, `--write` keeps it at or below its total. A raised total is yours to explain as a pure move in the report; if the tool reads it as a raise, stop and ask.
- **Lines.** A move adds no Rust beyond the re-export, so a kit task's report gives net Rust lines, manifests and `clippy.toml` apart. These two items can't end with fewer lines from moves alone, as the plan says.

## What runs

The world, the kit and each reader the spec names: `heavy cargo nextest run -p ...`. Every other package that still reads through the re-export: `heavy cargo check -p ... --tests` on one of them per tree is enough; the trunk's end check runs the rest.
