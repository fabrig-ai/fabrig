+++
id = "builder-repeats"
part = "builder"
kind = "feature"
needs = ["machine-call"]
files = ["hearth/tests/builder/src/**", "hearth/tests/builder/tests/suite/**", "hearth/testing/hearth-test-repository/src/resolution.rs"]
effort = "high"
minutes = 45
+++

# What the builder world repeats whatever its seed

## Goal

With the fake machine's call repaired, repair what the builder world repeats, if the aim needs it: above all the oracle's whole world run per seed for a value no seed changes. The start check may end this task at once.

Acceptance:

- One commit per repair, each with what it saved, largest share first, until the options end.
- Every seed of every test still runs its build and checks it, every assertion on the same values. No state crosses seeds.
- **If `rendered_keys` is computed once,** each seed's build still compares against it, so a seed whose keys differ fails loudly. Before relying on that, run the eight seeds once by hand, asserting they give one value, and say so in the report.
- Every test of `hearth-builder-world` passes in `default` and `fuzzy`.

## Inputs

- `hearth/docs/plan/hearth-speed-2/notes/common.md` and `hearth/docs/plan/hearth-speed-2/notes/repairs.md`
- `hearth/docs/plan/hearth-speed-2/figures.md`: `### Builder suite` and `## What it found`
- `hearth/docs/design/domain/testing.md`, 9.1: "A world computes once what its seed doesn't change"

## Owns and may change

- **Owns:** `hearth/tests/builder/src/**` and `hearth/tests/builder/tests/suite/**`; in `hearth/testing/hearth-test-repository/src/resolution.rs`, `hex_digest`'s body alone, because `scratch-objects` owns the rest of that crate. In a test file, helpers alone: every `assert` stays, on the same values.
- **Options, pre-approved, in the order the profile's shares put them:**
  1. **`rendered_keys` once in a test's process** (`hearth/tests/builder/tests/suite/domain.rs:137`). By reading, THIN's keys hash only name, architecture, version and sha256 of `CORE` packages, whose hashed fields are constants, so no seed changes them. `domain.rs` runs a whole world per seed and per failure point for it, 54 runs where 6 tests need 6. A `static KEYS: LazyLock<(Keys, Digest)>`, computed at seed 0, is `Send + Sync`, since it holds only digests; the `Run`, which holds `Rc`, stays inside the initializer. The plan rules this within the request: the request names it, and every seed's build still runs and compares. **If the hand check finds keys differ by seed:** a `thread_local!` map by seed, saving only the repeats, as `hearth/tests/builder/src/simulated.rs:74` does, and say so.
  2. **The protocol world's mirror:** `Below::new` always serves (`hearth/tests/builder/src/io.rs:44`), deep-cloning and indexing the universe, while `hearth/tests/builder/src/simulated.rs:120-127` serves only for resolving calls. The same guard. Nothing digests the protocol world's machine (by `rg`).
  3. **`hearth::limits()`** copied per demand check (`hearth/tests/builder/src/io/stream.rs:94`): a `const` of what it reads.
  4. **`hex_digest`** (`hearth/testing/hearth-test-repository/src/resolution.rs:93-99`) by a table, not `write!` per byte: the same signature and text.
  5. **The fake layers:** `snapshot` copying byte by byte and `survey` rereading what it copied (`hearth/tests/builder/src/domain/layers.rs:211-249`): `extend_from_slice` and the like. What the fake reports doesn't change.
  6. **`files_of`** runs twice per golden check (`hearth/tests/builder/src/fixtures/goldens.rs:40,113`).
- **Not options, by reading:** caching the fake repository or the universe. Each is under 1-2% of a run, and a clone costs as much. If `figures.md` shows otherwise, it's option 7, on `notes/repairs.md`'s terms. `Behind::list`'s calls stay: `hearth/tests/builder/tests/suite/budgets.rs` pins them.

## Steps

1. The start check. Stop if met.
2. `Ir` of `domain::a_step_that_fails_in_any_layer_leaves_no_staging_and_no_temporary_and_keeps_the_layers_beneath`, and `hearth-builder-world`'s `suite` figure, twice.
3. Each repair, then the same figures, then a commit with what it saved. Take out what saves nothing.
4. `heavy cargo nextest run -p hearth-builder-world -p hearth`, then `--profile fuzzy -p hearth-builder-world`.
5. The end of `notes/common.md`. Push.

## Report

The ratio at your start, each repair and its saving, the figures on both sides, the machine, and `## Left`.
