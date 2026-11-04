# D008: The agent shell maps endpoints to a one-variant program name

- **Kind:** complexity
- **Where:** `smith-agent-shell/src/config.rs`, `smith-agent-shell/src/config/programs.rs`, `smith-local-shell/src/fixture.rs`
- **Seen:** 2026-10-31 at `aa95945cc`, in the smith code-quality review (`reviews/2026-10-31-quality-areas.md`: report 08, finding 12)
- **Seen:** 2026-10-31 at `679eacb3c`, in the quality run's closing report (outside the repository): left, as the map is public and jig and temper build it
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (removes a lint exception at each site and a map that says nothing)
- **Size:** S
- **Status:** open
- **Paid when:** the configuration holds the program as `Option` (or a set of endpoints) while `ProgramName` has one variant, and no `zero_sized_map_values` expectation remains.
- **Measure:** `git grep -c 'zero_sized_map_values' -- '*.rs' | awk -F: '{s+=$2} END {print s}'` is 12

## What

`ProgramName` has one variant, `ClaudeCode` (`config/programs.rs:13`).
The configuration still holds `endpoint_programs:
BTreeMap<u32, ProgramName>` (`config.rs:50`, `:63`), and each such map
needs `expect(clippy::zero_sized_map_values)`: 12 across smith, jig's
`tests/channel` and `temper-playground`.

## Why it matters

The map is generality for a second program that does not exist, paid for
by lint exceptions in three trees.

## What pays it back

`Option<Program>` and the endpoints that use it, until a second program
arrives. The type is public: `smith-local-shell`, jig's test channel and
`temper-playground` change with it in one task.
