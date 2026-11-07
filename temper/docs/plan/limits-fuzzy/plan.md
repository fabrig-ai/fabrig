# limits-fuzzy: temper's fuzzy suite under its limit

The request: `inputs/request.md`. This is the second question of `skein/docs/foundation/testing-strategy.md`, 8.1, for temper's fuzzy suite: it reads 125 to 128 CPU s against 120, and the aim is about 15 out. Every sweep stays where it is, and draws what it drew. No budget, timeout, deadline or build setting changes.

## Parts and tasks

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `codecs` | `tasks/public-wire.md` | | The decoder sweep's 2,000 seeds, at a cost a profile explains |
| `one-process` | `tasks/one-process-sweeps.md` | | The two crash sweeps' four seeds each, at the same cuts |

Still to come, with the survey they need: `engine-sweeps`, `under-causes` and `close`.

## What this plan decided

- **Tasks are cut by world.** A profile names the causes, and a lead doesn't wait for one. So each sweep task owns its world, fixes what its profile puts there and stops at the request's 2%, and a cause below its world goes under `## Left`.

## Overlaps this plan could not avoid

`notes/common.md`, *Beside this plan*, lists what other drafts hold. `public-wire` and `one-process-sweeps` meet none of it.
