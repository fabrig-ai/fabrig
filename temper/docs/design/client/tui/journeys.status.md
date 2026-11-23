# journeys: status

2026-11-21. Where temper's code lags or departs from [journeys.md](journeys.md).

- **Not yet:** 4.2: the recording print's mismatch diagnostic in
  `temper/crates/temper-client-journeys-tool/src/run.rs` points to
  section 8, the box session, rather than sections 4 and 6.1's replay
  and recording print.
- **Not yet:** 6.2: the four journey directories still hold a
  `recording`, which is the writer's to remove. `launchpad-drawn`
  holds seven numbered page pairs but its walk marks five;
  `design-session` holds fifteen but marks ten. The writer's next
  successful print replaces those pages and removes the recordings
  (`../journey-writer.md`, 4).
- **Not yet:** 8.2: the drive session's save observer,
  `temper/crates/temper-client-journeys-tool/src/drive/io/tui.rs`, looks
  for `session/state/temper/tui/recording`. The recorder writes
  `recording-TIME`; the observer needs to follow the per-run filename
  for `see`, `mark` and approval to receive its saves [in flight: change 14084].
- **Not yet:** 8.3 and 9: `box-welcome` and `box-claude-sign-in` have no
  directories or walks. The drive session starts the TUI at a box's
  address, not in front of its child box; the world's TUI starts with
  no front facts or reports and refuses stop and terminal loans. The
  pair needs a front-host driver and walks; the second also needs a
  loan and return of the terminal. Wrapped-link redaction is built
  and is not what blocks them.
- **Not yet:** 9: `role-talk` has no directory, walk or scripts for its
  standing-role conversation and rule proposal. Its starting state
  and acts need client lines; the walk grammar refuses `ctl`.
