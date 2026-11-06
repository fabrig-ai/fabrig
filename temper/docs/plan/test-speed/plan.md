# test-speed: the slow tiers' bottlenecks that need no cache

The request is `inputs/request.md`. Its survey read code and logs and ran nothing, so each task measures first and its figures go in `figures/<task>.md`.

## Parts and tasks

| Part | Task | Needs | What it does |
|---|---|---|---|
| `stopped-box` | `ctl-stopped` | none | `temper ctl` says at once that nothing listens (`client/tools.md`, 6.3), ending the panicking story's 30 s |
| `walk-waits` | `walk-waits` | none | timelines from `box.log`, then each walk wakes on its event or the short paces |
| `tier-threads` | `tier-threads` | none | a fake box on the kernel's ports (`protocol/setup.md`, 7.5), `process-temper` at three threads |

## Critical path

None yet: the three stand alone, about 150 implementer-minutes at the longest, most of it waiting on tier runs.

## What this cut decided

- **A person's interfaces changed, with no compatibility:**
  - `temper ctl`'s new line and exit at a refused connection (`client/tools.md`, 6.3).
  - `temper box --fake-backends` on a fresh sandbox takes the kernel's ports, where it preferred 8367 to 8371. The request says "give those walks a free port as the other tests have". A walk's first run is a person's, on empty directories, so the free port belongs in the product: a fake box then leaves the fixed ports to the real box a person starts after it. `temper init` and a person's `temper box` are unchanged.
- **Grouped, expected:** walks signing in to ChatGPT, whose redirect comes back to the fixed port 1455.

## Overlaps not avoided

- `box-small/link-hours`: `tests/process.rs`, `tests/fake_backends/box_link.rs` and `walk.rs`, small hunks on each side.
- `sign-in-new-tab/sign-in-holds`: `tests/fake_backends/walk/sign_in.rs`, `tier-threads` touching only lines that bind.
- `walk-waits` and `tier-threads` share `tests/fake_backends/walk.rs`, each with its own hunk.
