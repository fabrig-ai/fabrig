+++
id = "worker-room"
part = "worker-room"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-worker-link/**", "jig/tests/worker-link/**"]
effort = "medium"
minutes = 80
+++

## Goal

The worker's side of the link is given its stream's room and is made
only when every frame it sends fits it, so no turn, call, answer or
hello ever waits below unanswered and silences the link
(`jig/docs/design/protocol/channel.md`, 9, *Every frame fits the stream
below*; 11). The worker's frames are bounded by its own limits, so a
check when it is made is the whole guarantee; nothing at runtime
changes.

Acceptance:

- `Limits.room` and `valid`'s new rules exist as
  `temper/docs/plan/worker-link-quiet/notes/contracts.md`, section 3 has
  them; `Component::new` refuses limits that break them, as it refuses
  unusable limits now.
- The rows of Steps, 2 pass and bite.
- Every existing test of the component and of `jig-worker-link-world`
  passes, edited only where it builds `Limits`.
- temper's stock worker settings, both profiles, still make the link:
  `heavy cargo nextest run -p temper-service -E 'test(/worker/)'` passes.
- The **Not yet** entry `9, 11, 12: the worker's side has no room either`
  in `jig/docs/design/protocol/channel.status.md` is removed, its lines
  alone (`notes/common.md`, *Status lines*).

## Inputs

- `jig/docs/design/protocol/channel.md`: 9 (*Every frame fits the stream
  below*), 10.1, 11 (*The component's own limits*, *The room is the
  stream's, not the domains'*), 12 (*A frame past the room, in the
  machine worlds*: the worker's world's row).
- `skein/docs/design/lib.md`, 7.1.
- `temper/docs/plan/worker-link-quiet/notes/contracts.md`, sections 1
  and 3.
- `temper/docs/plan/worker-link-quiet/notes/common.md`.

## Owns and may change

- Owns the files in `files`, but `jig/tests/worker-link/src/joined.rs`
  and `jig/tests/worker-link/tests/suite/joined.rs`, which are
  `engine-room`'s: in `src/joined.rs` you change one line alone, the
  worker's `Limits` literal's `room:` (or none, if it builds the
  worker's limits from `fixtures`).
- Reader files, the `room:` line alone:
  - `temper/crates/temper-service/src/worker_settings.rs`, `link_limits`:
    `room: worker.limits.io.largest_room(),` (`notes/contracts.md`,
    section 4).
  - `jig/testing/jig-test-service/src/limits.rs`, `worker_link`: `room:
    worker.limits.io.largest_room(),`.
  - Any other literal the compiler names: that site's io
    `largest_room()`, or in a world without io the room its scripted
    stream grants, named in the report.
- Pre-approved:
  - **A world or fixture whose own limits fail the new rule** (a small
    test io beside large sizes): give its `room` the largest frame its
    sizes give, computed with `sizes::largest_body`, and say which.
  - **If a stock profile fails it** (temper's `test(/worker/)` run, or
    `jig-test-service` on jig's standard or small profile): stop with a
    `Question:` naming the profile, the kind, its largest frame and the
    room. That would be a figure; push the work you have first.
- Not yours: `jig/crates/jig-protocol-engine-workers/**`,
  `jig/tests/engine-workers/**` (`engine-room`'s), skein, `temper/**` but
  the line above, any `docs/` but the one status entry.

## Steps

1. **The shape,** `notes/contracts.md`, section 3: `room` in `Limits`;
   `valid`'s rules beside the ones there. `channel_limits` is unchanged.
   The world's fixtures' limits take a room that keeps every row as it
   is.
2. **The rows,** one table in `jig/tests/worker-link/tests/suite/link.rs`
   (or the component's `--lib` limits tests, where `valid` is tested
   now): `a_room_below_a_frame_the_worker_sends_is_not_made`. Rows: the
   room at the largest of the worker's frames, made; one byte below the
   largest turn frame, one below the largest call frame, one below the
   largest answer frame, one below the hello's, each not made
   (`Component::new` is `Err(Unusable::Limits)`, `valid` false); a room
   of 269, not made. Break the rule once by hand (drop the kinds loop),
   see the rows fail, restore, and name it in the report.
3. **The readers,** as *Owns and may change* says; then `heavy cargo
   check -p temper-service --tests` and `heavy cargo check -p
   jig-test-service --tests`.
4. **Runs.** `heavy cargo nextest run -p jig-protocol-worker-link`,
   `heavy cargo nextest run -p jig-worker-link-world`. The component
   refuses limits it took before, which its readers see: also `heavy
   cargo nextest run -p jig-test-service`, `heavy cargo nextest run -p
   jig-channel-world` and `heavy cargo nextest run -p temper-service -E
   'test(/worker/)'`. Check each run's count.
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and
   `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-protocol-worker-link -p jig-worker-link-world -p jig-test-service
   -p temper-service --all-targets -- -D warnings`; remove the status
   entry; commit saying why (a worker whose frames could pass its
   stream's room is not made); the citations check; push.

## Report

As the implementer's text has it, and: the largest frame of each kind
the worker sends under temper's two stock profiles beside their room,
read from a test's numbers or computed with `sizes::largest_body`; each
fixture whose room you set otherwise, and why; the rule broken for the
bite and the failure seen.

## Answers

2026-11-09 (temper/migration). Increase the owned example's worker io.output so its largest Turn fits, and keep its turn contract. Size it from the turn the example admits, with no more headroom than the stock profiles keep. temper/docs/examples/owned.json is authorized for that one figure, plus whatever line the example's own check needs to say so. Change no other figure. (temper/migration's answer of 11:52Z, which landed on the part in plan.toml where fab doesn't read it; moved here by agents/loop)

2026-11-09 (agents/loop). Yes, the recommended option: worker-room adds the minimal named io.output override support in jig-protocol-config, with its registry entry, checks and generated reference updates, so the owned example can carry the authorized io.output figure. Keep both stock profiles and every other figure unchanged, add no other override name, and keep the change to what the owned example's figure needs. jig-protocol-config is opened to this task for exactly that. (agents/loop, on the user's word at 17:14Z to land this plan fast; temper/migration's 11:52Z answer authorized the figure)
