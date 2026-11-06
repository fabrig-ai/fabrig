# ctl-waits: `temper ctl` says what it waited for, and a kept message gets its answer

The request: `inputs/request.md`. Three things, ranked, in at most four tasks.

## Parts and tasks

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `deadline-words` | `tasks/deadline-words.md` | | Item 2: the deadline's four lines, and what's true of an act (`temper/docs/design/client/tools.md`, 6.3) |
| `long-pass` | `tasks/long-pass.md` | | Item 3: one line for a pass past 1 s (`temper/docs/design/protocol/README.md`, 8.1) |
| `kept-answer` | to come | | Item 1, ranked first and released last: a survey of the kept act's path is running, and its tasks follow it |

Critical path, as it stands: each of these alone, in parallel.

## What this plan decided

- **Item 3's bound is 1 s, one line in 10 s at most.** A thirtieth of ctl's deadline, and the walk's longest whole read under load
  was 1.95 s. Long passes inside the 10 s are held as the longest and a count, so a long stall never hides behind a short one.
- **Item 3 costs seven clock reads a pass when nothing's slow,** with no allocation and no write. `long-pass` measures a read.
- **Item 3's stages are temper's own,** so nothing generic goes to skein. A step crate takes no closure or clock (`skein`'s
  programming model, 10), so `iterate` splits into stages and the binary's adapter reads the clock between them; a world has no watch.
- **Item 2 stays in `temper-client-shell-cli`,** with its one line of help. A sign-in's and a sign-out's line stay as they are.

## Overlaps

- `temper/docs/help/ctl.md` sits in `model-check/trial-close`'s glob, `temper/docs/help/**`. One line of Limits changes.
- `temper/crates/temper/src/host.rs`, `temper/crates/temper-service/src/lib.rs` and `protocol/README.status.md`: `landing-fixes-2/refusal-words` touched each and has merged.
