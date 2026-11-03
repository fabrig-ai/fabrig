# playground-temper: the plan

From `inputs/request.md`: temper's playground world, natively: `temper/docs/design/playground.md`,
sections 3 (the crates), 4 (the world), 5 (stories as data, their pace; story A), 8 (the
documents' bounds, measured) and 9's native part. Every task names `notes/common.md`; the shapes
that cross tasks are `notes/contracts.md` (written with the second batch).

## Parts and tasks (as released)

1. **hosted-git** (temper, `temper/docs/design/playground.md, 3`), slice 1.
   - `tasks/hosted-git-moves.md`, migration, medium, ~60 minutes, needs nothing:
     `temper-fake-git-hosted` from `testing/` to `crates/`, its `[dependencies]` product crates
     alone, its fixtures moved word for word to the checkout world (`hosted`), the real loop's and
     the one-process world's paths edited; three design lines and two status entries say
     `crates/`.

More parts follow in later pushes (the composition, story A, the fakes' pace, the bounds).

## Critical path (as it stands)

`hosted-git-moves`, then the composition's crate, then the one-process world rewired on it.

## Overlaps with drafts beside this one

- The root `Cargo.toml` and `Cargo.lock`: every task that adds or moves a crate edits them.
- None with `reference-m10` or `configuring-p2-build`'s files.
