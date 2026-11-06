+++
id = "render-repeats"
part = "render"
kind = "feature"
needs = ["render-lookups"]
files = ["hearth/tests/render/src/**", "hearth/tests/render/tests/suite/**", "hearth/testing/hearth-test-render/src/**"]
effort = "medium"
minutes = 40
+++

# What the render world's harness repeats whatever its seed

## Goal

Repair what the render suite's harness repeats, if the aim needs it: an expected directory read again per seed, a template compiled per render, a repository decoded twice. These are test code. The start check may end this task at once.

Acceptance:

- One commit per repair, each with what it saved, largest share first, until the options end.
- Every seed still renders and is checked, on the same expected values. No state crosses seeds, and what's memoized is never written after it's made.
- Every test of `hearth-render-world` and `hearth-builder-world` passes in `default` and `fuzzy`, with every ceiling standing.

## Inputs

- `hearth/docs/plan/hearth-speed-2/notes/common.md` and `hearth/docs/plan/hearth-speed-2/notes/repairs.md`
- `hearth/docs/plan/hearth-speed-2/figures.md`: `### Render suite` and `## What it found`
- `hearth/docs/design/domain/testing.md`, 9.1: "A world computes once what its seed doesn't change"

## Owns and may change

- **Owns:** `hearth/tests/render/src/**`, `hearth/testing/hearth-test-render/src/**`, and in `hearth/tests/render/tests/suite/**`, helpers and loop shape alone: every `assert` stays, on the same values.
- **`hearth-test-render` is also read by `hearth-builder-world`.** Its public signatures don't change.
- **Options, pre-approved, in the order the profile's shares put them:**
  1. **The expected directory per seed:** `golden.target_digests("")` (`hearth/tests/render/src/fixtures/goldens.rs:19-20`) re-reads the directory and every target's file inside the seed loop, 4 seeds by 5 repositories in 4 tests. It moves above the loop; the assertion stays in it. This saves system time too.
  2. **The template and the blob per render:** the parent compiles a template per read (`hearth/testing/hearth-test-render/src/parent.rs:202-224`) and hashes a static blob per render (`:186-187`). A `thread_local!` memo keyed by body and limits, handing out a clone. **Bypassed under `Memory::Within`,** because `blob_event` runs inside the meter's span (`:91-157`): a memory test computes as today, and its peak doesn't move.
  3. **A repository decoded twice:** `repository_of` then `inputs::run` decode the same leaves (`hearth/tests/render/tests/suite/properties.rs:93-94`). `render(&repository, &[], case)` is the identical call.
  4. **Seed-free fixtures in a seed loop:** `declarations::decode(b"{}")` and the like inside `for seed` in `thin.rs`, `compose.rs`, `errors.rs`, `schema.rs` and `debian.rs` of `hearth/tests/render/tests/suite/`. Move above the loop what depends on no seed, confirming it by reading. The gain is small, so only with a measured fall in `Ir`.
- **Seed-dependent, so it stays:** `properties::{generated,edit,mutate}`, the 200-case suites (`oracle`, `schema`, `templates`, `entries`, `evaluate`), `laws::parts`.
- **Not options:** anything in `hearth/crates/`. A cost there goes under `## Left`, for `last-round`.

## Steps

1. The start check. Stop if met.
2. `Ir` of the costliest test `figures.md` samples from `hearth-render-world`, and that binary's figure, twice.
3. Each repair, then the same figures, then a commit with what it saved. Take out what saves nothing.
4. `heavy cargo nextest run -p hearth-test-render -p hearth-render-world -p hearth-builder-world`, then `--profile fuzzy -p hearth-render-world -p hearth-builder-world`.
5. The end of `notes/common.md`. Push.

## Report

The ratio at your start, each repair and its saving, the figures on both sides, the machine, and `## Left`.
