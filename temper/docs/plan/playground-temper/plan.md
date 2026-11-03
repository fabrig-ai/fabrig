# playground-temper: the plan

From `inputs/request.md`: temper's playground world, natively: `temper/docs/design/playground.md`,
sections 3 (the crates), 4 (the world), 5 (stories as data, their pace; story A), 8 (the
documents' bounds, measured) and 9's native part. The design exists and needed no new section;
the shapes that cross tasks are `notes/contracts.md` (C1 the crate, C2 the harness's seam, C3 a
world's start, C4 stories as data). Every task names `notes/common.md`.

## Parts and tasks

1. **hosted-git** (`temper/docs/design/playground.md, 3`), slice 1.
   - `tasks/hosted-git-moves.md`, migration, medium, ~60 min, needs nothing:
     `temper-fake-git-hosted` from `testing/` to `crates/`, its `[dependencies]` product crates
     alone, its fixtures moved word for word to the checkout world (`hosted`); the real loop's and
     the one-process world's paths edited; three design lines and two status entries.
2. **fake-pace** (`temper/docs/design/protocol/setup.md, 8.4`), slice 2.
   - `tasks/fake-pace.md`, feature, high, ~60 min, needs nothing: temper-fake's documents of
     their own read `pace` (`reply_ms`, `ci_ms`, at most 60000), the domains' latencies least and
     most alike; `llm::LIMITS` and `forge::checks` unchanged for their readers.
3. **composition** (`temper/docs/design/playground.md, 3`), slice 3.
   - `tasks/playground-documents.md`, interface, high, ~90 min, needs nothing: the crate made,
     with `documents` and `fakes`, the composition's values and the test kits' copied as its own.
   - `tasks/playground-processes.md`, feature, high, ~150 min, needs `playground-documents` and
     `hosted-git-moves`: `machine`, `service`, `programs`, `story` (C2, C3), `world(start)`
     harness-free with the trace off, handover reads from the disk, `Links::Print`.
   - `tasks/one-process-documents.md`, migration, medium, ~60 min, needs `playground-documents`
     and `hosted-git-moves`: the one-process world reads its composition's documents from the
     playground and drops the test kits they came from.
   - `tasks/one-process-composed.md`, migration, high, ~150 min, needs `playground-processes` and
     `one-process-documents`: `Process<OneProcess>`, its captures as `Observe`, every world builder
     on `programs()`; no composition copy left; testing.status.md's and playground.status.md's 3.
   - `tasks/playground-wasm32.md`, feature, high, ~90 min, needs `story-a`: the crate checks and
     clippies for wasm32, temper's crates gated where native-only; other trees' failures listed.
4. **story** (`temper/docs/design/playground.md, 5.2`), slice 4, **walked** (rung A's story through
   `temper ctl`).
   - `tasks/story-a.md`, feature, high, ~90 min, needs `playground-processes` and `fake-pace`:
     `stories/a/` as data (scripts from temper-init's template, `otherwise` for the browser, pace
     1200 ms and 4000 ms), `Story::start`, the checks as data; playground.status.md's 5.
   - `tasks/one-process-story-a.md`, feature, high, ~120 min, needs `one-process-composed` and
     `story-a`: story A walked in the one-process world by its fake person through the terminal
     client, from the welcome to the CSV change landed and two answers, under its referee.
5. **world** (`temper/docs/design/playground.md, 4`), slice 5.
   - `tasks/playground-world.md`, feature, high, ~90 min, needs `story-a` and
     `one-process-composed` (the same status line): `tests/playground`, story A's world with no
     harness: no box, calm, one seed, the link on standard output, the listener at the document's
     address.
6. **bounds** (`temper/docs/design/playground.md, 8`), slice 5.
   - `tasks/bounds-measured.md`, feature, high, ~150 min, needs `playground-world` and
     `one-process-story-a`: memory at the first step (harness-free) and through story A (the
     one-process world), per process, each knob's effect and floor, in
     `temper/docs/development/playground-memory.md`; no bound set, the bar being the user's.

## Critical path

`playground-documents` (90) → `playground-processes` (150) → `one-process-composed` (150) →
`one-process-story-a` (120) → `bounds-measured` (150): about 660 implementer-minutes. Beside it:
`hosted-git-moves` and `fake-pace` at once; `one-process-documents` once the documents land;
`story-a` beside `one-process-composed`; then `playground-world` and `playground-wasm32`.

## Not in this plan

- **The walk through the web client's host and a log replayed** (`playground.md`, 9): they need
  skein's visitor, its in-memory bridge `Platform` and its session log and replay feed
  (`skein/docs/design/playground.md`, 4 to 6), which plan `playground-skein` builds and whose
  cut has pushed no contracts yet. The report's question.
- **The reset without the box** (`playground.status.md`, 4): it does not hold. The fake forge's
  removal still asks the box link whether or not a pipe is there
  (`temper-engine-domain-forge/src/removal.rs`, ~38; `domain/forge.status.md`, 4.1), so it stays
  named, as the request says.
- **B0's twin documents** (`domain/testing.md`, 2.4: moved into `temper-playground`'s stories when
  it lands): `temper/tests/real-loop/tests/fixtures/b0` stays until B0 becomes a story (5.3).
- Out of scope by the request: the page, starter and frame (6), the served files (7), the wasm32
  builds in the gate, the browser tier (9's browser part), the bar (10).

## Overlaps with drafts beside this one

- The root `Cargo.toml` and `Cargo.lock`: `hosted-git-moves`, `playground-documents` and
  `playground-world` each edit a member line, placed apart (C1).
- `temper/docs/design/playground.status.md`: five tasks of this plan remove words from its one
  **Not yet** entry, ordered by `needs` where they touch one line, never reflowed
  (`notes/common.md`).
- `playground-wasm32` may gate a native-only item in a temper crate that `reference-m10`
  (`temper-service/src/limits.rs`, `temper-engine-domain-forge`) or `configuring-p2-build`
  (`temper-service/src/startup/problems.rs`, `temper-settings-form`, `temper-engine-smith`)
  changes; its spec makes any change there beyond a gate a question.
- None with `playground-skein` (skein's `skein-playground`, `skein-ui-web`, `skein-ui-bundle`):
  this plan touches no skein file.
