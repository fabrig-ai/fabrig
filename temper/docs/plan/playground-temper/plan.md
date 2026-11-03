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

2. **fake-pace** (temper, `temper/docs/design/protocol/setup.md, 8.4`), slice 2.
   - `tasks/fake-pace.md`, feature, high, ~60 minutes, needs nothing: temper-fake's LLM and
     forge documents of their own read `pace` (`reply_ms`, `ci_ms`, at most 60000), the
     domains' latencies least and most alike; `llm::LIMITS` and `forge::checks` unchanged for
     their readers.
3. **composition** (temper, `temper/docs/design/playground.md, 3`), slice 3; its shapes are
   `notes/contracts.md`, C1 to C3.
   - `tasks/playground-documents.md`, interface, high, ~90 minutes, needs nothing: the crate
     `temper-playground` made, with `documents` and `fakes`, the one-process world's
     composition values and the test kits' copied as its own; nothing reads it yet.

More tasks follow in later pushes (the composition's processes, the one-process world rewired,
story A, the harness-free world, the bounds, the wasm32 build).

## Critical path (as it stands)

`hosted-git-moves`, then the composition's crate, then the one-process world rewired on it.

## Overlaps with drafts beside this one

- The root `Cargo.toml` and `Cargo.lock`: every task that adds or moves a crate edits them.
- None with `reference-m10` or `configuring-p2-build`'s files.
