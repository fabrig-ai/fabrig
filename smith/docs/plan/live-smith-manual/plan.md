# live-smith-manual: the cut

The request (inputs/request.md): smith's API-key live story leaves the automatic `live-smith` run
and is the owner's, and smith's testing design says what runs after landing and what the owner
runs. One direct part, one task; the whole draft is about twenty lines.

## Parts and tasks

| part | task | needs | files | effort |
| --- | --- | --- | --- | --- |
| key-story-manual | key-story-manual | none | `smith/tests/end-to-end/live.rs`, `smith/docs/design/testing.md` | medium, 25 min |

Slice 1: `cargo nextest list --profile live-smith` holds every live story but
`an_anthropic_key_delivers_a_commit_in_place`, which the owner runs with `--run-ignored only`.

## What was decided

- **`#[ignore]`, not the profile's filter.** The request allows either. The ignore matches temper's
  owner-run live stories (`temper/crates/temper/tests/live.rs`), keeps the owner's command a
  plain `--run-ignored only`, and keeps this draft off `.config/nextest.toml`, which live-temper's
  `pair-profile` changed in its trunk.
- **One task, code and design together.** The attribute and the design text say the same thing;
  split, they would be two slots for one line each. The spec gives the design text word for
  word (a stated departure: implementers do not write `docs/design/` otherwise).
- **The check is a list, never a run.** The task lists the `live-smith` selection before and after
  and the owner's selection; no live test runs (no real-service probes, no credentials).
- **Survey:** the story's name appears only in `live.rs` and the request; the `anthropic-key`
  setup in `support/live.rs` and its offline control in `live_configuration.rs` stay, since the
  owner's run uses them; no test reads `smith/docs/design/testing.md` (smith-bench's design
  checks read only the sections its benchmark tasks cite, none in testing.md); `tools/tiers`'
  package sets do not change (the `smith` package keeps other live stories).

## Overlaps with drafts beside this one

- live-smith-reds (cut in flight): its tasks so far own `smith/tests/fake-claude/**`,
  `smith/tests/agent-process/tests/live/program.rs` and two `smith-protocol-program` test files,
  none of this task's. It may later touch `smith/tests/end-to-end/live.rs` or
  `smith/docs/design/testing.md` for the Claude story; this task's edits are one attribute and
  two places in 2.3, so a textual conflict is unlikely but not ruled out.
- sign-in-wait owns `smith/docs/design/shell.md`; live-temper owns `.config/nextest.toml` and
  temper's files; v0.1.0-ladder's files are not listed. No overlap known.

## Left outside

- `smith/docs/design/testing.status.md` still says the own-account live checks "have not run";
  the after-landing run makes that stale. Its designer's, not in this request.

## Critical path

`key-story-manual` alone: one build of `smith`'s and `smith-agent-process-world`'s test binaries
for the list before, one incremental after, and clippy.
