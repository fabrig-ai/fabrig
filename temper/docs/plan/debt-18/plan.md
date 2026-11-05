# debt-18: the eighteenth debt round

Two independent parts (`inputs/request.md`): the spot review of debt-14's smith part, and four items that exist to remove
a copy. This file grows with each release.

## Parts and tasks

| Source | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| review 1 | `smith-review` | `effort-pin` | | high | 25 | a step test on the effort rule, a no-newline case; the seam goes |
| review 2 | `smith-review` | `session-fake-strict` | | high | 40 | the fakes panic on what no fixture sends; `feedback_text` goes |
| review 3, 4 | `smith-review` | `local-residue` | `effort-pin` | medium | 35 | notices read by type; residue goes; a row for the clips |
| review 5, 6 | `smith-review` | `terminal-words` | `local-residue` | high | 50 | the person's words move; one UTF-8 helper; forwarders go |

## Critical path

So far: `effort-pin`, `local-residue`, `terminal-words`, 110 minutes.

## Decisions: the review, checked against the code

Every finding was read in the code at this plan's base. They hold, but for these:

- **Finding 3, "one `Notice` arm":** merged carelessly, it changes what the local world's referee sees, since only the
  second arm reaches `self.notices` and `self.saw`. The spec guards that.
- **Finding 4, the clips:** of "drop `cap`, or add a row", a row. Dropping `cap` makes a notice copy a whole segment.
- **Finding 4, `const ALL` and `exited`:** the copy is in `smith/tests/tools/tests/suite/simulation.rs`, another test crate.
  They meet in `smith_tools_world::fixture`, which `stories.rs` already reads.
- **Finding 6, the status counts: skipped, already right.** `testing.status.md` and `STATUS.md` say three domain worlds call
  protocol crates, the review's own count; debt-14's retire corrected them after the merge the review read. `tests/local`
  also renders with `smith-local-protocol`, but `smith/docs/design/testing.md`, 2 makes it a system world.
- **Finding 6, the words:** the request names `service.rs` and `headless.rs`, and that's the scope. `service/accounts.rs`,
  `service/choices.rs`, `front/interactive.rs` and `front/remote.rs` hold more of a person's words: `terminal-words` reports
  them as left, and they're worth a smith item if fabrig/debt wants them.

## Overlaps

None for this part: no draft beside this one touches smith.
