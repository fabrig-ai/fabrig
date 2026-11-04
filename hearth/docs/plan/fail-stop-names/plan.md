# fail-stop-names: the plan

Two small direct parts, independent of each other, from `inputs/request.md`. No behaviour of a
product changes.

## Parts and tasks

- **`fail-stop`:** `hook-binaries` (temper's two binaries take the hook), then `fail-stop-test` (the
  check in `fabrig-boundary`, the gate's list, the script's removal). The test needs the fix: over
  the workspace it is red without it.
- **`names`:** `transcript-host` (skein) and `probe-account` (temper's design), each alone.
- **`grader`:** `grader-url`, alone. A survey read how the grader builds: nothing fetches the URL,
  but it is the source's identity in the frozen bundle, so a rename breaks the offline build. The
  first grade now copies the two lines from the frozen workspace's own `Cargo.toml`. The grade
  command changes, so the task goes to version 3 (`smith/docs/design/benchmarks.md`, 5.2).

## Design

The cut closed the one Later entry the check settles: "The fail-stop check" in
`hearth/docs/design/domain/testing.md`, the only other file naming the script. A draft's own text
stays with a task: `probe-account` edits `temper/docs/design/draft/github.md`, section 11, because
the request names it, and says so as a departure.

## Overlaps

- `hearth/docs/design/domain/testing.md` is also `hearth-docs`'s, whose request changes Later
  headings and citations. This cut removes one bullet: the request says what names the script goes
  with it, so that could not be avoided. The neighbouring lines are unchanged, so a conflict is
  unlikely and small.
- `temper/tests/client-browser/**` and `Cargo.lock` are touched by `look-round-3` and `debt-13`:
  `look-round-3`'s branch changes only `tests/browser_all/` there, not the manifest or the binary,
  and `Cargo.lock` differs in other packages' entries.

## Questions

`oracle-valid`: `grader/src/lib.rs`, 40 very likely doesn't compile against the bundle. It is outside
the request, so the line stays by default. No task waits.

## Unobserved

The bundle is on no builder, so `grader-url` proves its command on a rehearsal of stub crates. Only
`smith-bench validate repository/shell-cwd` on a machine holding the bundle proves it on the real
one.

## Critical path

`hook-binaries`, then `fail-stop-test`: about an hour of implementer time. The others run beside it.
