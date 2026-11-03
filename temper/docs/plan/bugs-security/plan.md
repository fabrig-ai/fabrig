# bugs-security: the plan

From `inputs/request.md` (the debt backfill's "Bugs", routed by the director, 2026-11-03): bugs
that leak a secret, each fixed with a test in the lowest world that shows it, failing on the old
code. Every task names `notes/common.md`.

## Parts and tasks

1. **jig-debug** (`jig/docs/design/domain/hosts.md, 5.1`), slice 1.
   - `tasks/jig-resources-debug.md`, medium, ~30 minutes, needs nothing (request item 1):
     `jig-test-service`'s `program_sources::Resources` gets temper's count-only `Debug`; tests in
     a new `src/tests/program_sources.rs` format it, and a configured `Component`, with a canary.

Still being cut (later pushes): skein-io's `file::Request` and `file::Event` (item 2), `CryptoRng`
losing `Clone` and its users (item 3), and the other credential holders the search of item 4
finds, tree by tree.

Critical path so far: `jig-resources-debug` alone.

## What was decided, and why

- **Released first:** item 1 needs no survey: `Resources` is built through `from_paths` and
  `new` only, and nothing reads its `Debug` text (no test, trace or golden file holds it).

## Overlaps with the drafts beside this one

- `jig-resources-debug` edits `jig/testing/jig-test-service/src/program_sources.rs`, adds
  `src/tests/program_sources.rs` and one `mod` line to `src/tests.rs`, all inside
  `fabrig-scale/standard-scale`'s glob `jig/testing/jig-test-service/src/**`; that task's work is
  the standard profile's scale, not program sources, so only the `mod` line could meet a textual
  conflict.
