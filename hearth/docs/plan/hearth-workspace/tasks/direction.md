+++
id = "direction"
part = "rules"
kind = "feature"
needs = ["join-steps"]
files = [
  "tests/boundary/tests/direction.rs",
  "tests/boundary/tests/outward.rs",
  "AGENTS.md",
  ".config/gate/README.md",
]
effort = "high"
minutes = 35
+++

# The trees' direction states hearth's place: skein alone, and nothing names hearth

## Goal

`tests/boundary/tests/direction.rs` holds the trees as a line, each naming those before it. hearth
names skein alone and nothing names hearth, which a line can't say, and a package under `hearth/`
isn't in `TREES`, so today it isn't checked at all. Make the check state the rule, with a test that
catches a breach both ways.

Acceptance:

- `heavy cargo nextest run -p fabrig-boundary` passes, and reads hearth's members and their
  dependencies on skein in the real workspace.
- A new fixture test fails for each of: hearth naming smith, jig or temper; skein, smith, jig or
  temper naming hearth. hearth naming skein is no breach.
- The existing tests' assertions hold as they stand, their fixtures gaining hearth's manifest.

## Inputs

- `notes/common.md`, in this plan's directory (`hearth/docs/plan/hearth-workspace/`).
- `AGENTS.md`, 1; the module doc of `tests/boundary/tests/direction.rs`.
- This cut wrote no design: `AGENTS.md`, 1, is the rule.

**Departures from the implementer's text:** you edit root `AGENTS.md` and `.config/gate/README.md`,
as below.

## Owns and may change

**Owns:** `tests/boundary/tests/direction.rs`.

**May change, only as the steps say:** `tests/boundary/tests/outward.rs` (one entry), `AGENTS.md`
(one bullet), `.config/gate/README.md` (one list).

Pre-approved: if `TREES`'s shape below reads badly inside a function, reshape the helpers around it,
and keep the table.

## Steps

1. **The table replaces the line:**

   ```rust
   /// The trees, and the trees each one's packages may name besides its own:
   /// skein to temper in a line, and hearth on skein alone, which nothing names.
   const TREES: [(&str, &[&str]); 5] = [
       ("skein", &[]),
       ("smith", &["skein"]),
       ("jig", &["skein", "smith"]),
       ("temper", &["skein", "smith", "jig"]),
       ("hearth", &["skein"]),
   ];
   ```

   `tree` finds a directory's row. `breaches` holds a dependency on another tree's package a breach
   unless the row names that tree; its message stays `{name} ({directory}) depends on {dependency}, a
   {tree} package`. `the_trees_depend_upstream_only` loops over the rows: a tree with names has
   dependencies on them, and every tree has members. The module doc says the same in words.
2. **The fixture test:** `ROOT` gains a `hearth/f` member and `hearth-f`, and the existing fixtures its
   manifest. A new test, `hearth_names_skein_alone_and_nothing_names_hearth`, asserts the exact breaches
   of a fixture where hearth names smith, jig and temper, and each of the four names hearth, and that
   hearth naming skein adds none. Break the rule once by hand, as the implementer's text asks: remove
   hearth's row and see it fail.
3. **`tests/boundary/tests/outward.rs`:** `no_file_names_the_lab`'s prefixes gain `"hearth/"`.
4. **`AGENTS.md`, 1, `Dependencies point one way`:** skein names none of smith, jig, temper or hearth;
   smith names neither jig nor temper; jig never names temper; hearth names skein alone, and nothing
   names hearth. `A missing generic mechanism goes into skein before its smith, jig or hearth consumer.`
5. **`.config/gate/README.md`, the gate's item 2:** `hearth/clippy.toml` in its list of rules.
6. fmt, clippy for `fabrig-boundary`, the citations check; commit and push.

## Report

The implementer's report, and the exact breaches the new test pins.
