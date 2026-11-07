# limits-fuzzy: temper's fuzzy suite under its limit

The request: `inputs/request.md`. This is the second question of `skein/docs/foundation/testing-strategy.md`, 8.1, for temper's fuzzy suite: it reads 125 to 128 CPU s against 120, and the aim is about 15 out. Every sweep stays where it is, and draws what it drew. No budget, timeout, deadline or build setting changes, and no test goes.

## Parts and tasks

| Part | Task | Needs | What it makes run | Effort, minutes |
|---|---|---|---|---:|
| `codecs` | `tasks/public-wire.md` | | The decoder sweep's 2,000 seeds, at a cost a profile explains | high, 50 |
| `one-process` | `tasks/one-process-sweeps.md` | | The two crash sweeps' four seeds each, at the same cuts | high, 110 |
| `state` | `tasks/state-hash.md` | | The gate on the compare's cost, then `Hash` on skein's and jig's state types | medium, 80 |
| `engine` | `tasks/engine-sweeps.md` | `state-hash` | The walking, escalation and role sweeps, replaying by digest, then the world's next causes | high, 140 |
| `under` | `tasks/under-causes.md` | the three sweep tasks | What they left below their worlds, and the real loop's causes that clear the bar | high, 130 |
| `close` | `tasks/close.md` | all of the above | The account for the user: `temper/docs/development/limits-fuzzy.md` | medium, 60 |

**Critical path,** by the specs' estimates: `state-hash`, `engine-sweeps`, `under-causes`, `close`, about 410 minutes.

**Six tasks, against the request's "about five":** the digest's derive lines fall in three trees, and a branch stays in its tree. `state-hash` is the mechanical half, at medium effort, and it holds the gate, so that no line is written before a profile asks for it.

## Where the gain should come from

These are leads from reading, with the request's figures, and no profile stands behind them yet. Each task's profile decides, and the account will say what they really gave. If the old 84% still holds, the first row alone could reach most of the aim of 15 CPU s.

| Sweep | CPU s now | What may go |
|---|---:|---|
| Walking shards and escalation, and the roles sweep beside them | 21.5, and the roles sweep's unmeasured share | The text of two whole worlds at each iteration, which an old comment puts at 84% of the walk |
| The two crash sweeps | 9.6 | A setup that runs a whole story to learn a window, and per-step scans |
| Public records | 3.9 | Two inputs that don't vary with the seed, decoded 2,000 times |
| Real loop | 57 | Only what's small and clears 2 CPU s: `notes/real-loop.md` |

## What this plan decided

- **Tasks are cut by world.** A profile names the causes, and a lead doesn't wait for one. So each sweep task owns its world, fixes what its profile puts there and stops at the request's 2%, and a cause below its world goes under `## Left` for `under-causes`, which starts after them.
- **The digest is `Hash`, as the design says.** `skein/docs/design/lib.md`, 5.1, and `skein/docs/foundation/testing-strategy.md`, 6, already say a state is digested through its types' derived `Hash`, skein's containers implement it, and `temper/docs/design/domain/testing.status.md` lists it as not yet built. A survey found about 660 one-word derive lines and no blocker (`notes/contracts.md`). No text-side trick reaches the same place: a sink that hashes the `{:?}` text skips the strings' copies and still pays for the formatting.
- **A gate before the width.** The 84% is an old figure. `state-hash` profiles first, and writes no derive unless the compare is at least 25% of those sweeps, about 5.4 CPU s.
- **The test `Store` hashes what its `Debug` prints:** four fields. That's what's compared today, and it keeps the store component's own state types, 158 sites in seven crates, out of this plan.
- **A digest at each iteration, and the text once at each seed's end.** `Hash` leaves out by design what later answers don't depend on, such as a capacity (`skein/docs/design/lib.md`, 5.1), and `Debug` printed it. The last comparison keeps the text's whole reach for what lasts to the end of a seed, at the cost of one formatting per seed.
- **A cheap hasher may stand in for `StateHasher`.** SHA-256 unoptimized may cost more than the formatting. These digests are compared once, in one process, and never kept. `engine-sweeps` measures both.
- **The walking shards stay eight,** though their comment said they'd hold only until the digest. Merging them saves about 56 ms of processes and costs the gate's wall time.
- **Fixed inputs aren't draws.** In the public records' sweep, "at every bound" and "length past ceiling" read no random number, so checking each once per kind drops no draw. All 2,000 seeds keep their inputs, byte for byte.
- **This plan adds no gate entry.** The request's "once it holds, the limit fails merges" is a later act: its every-task rule says no budget changes, and `skein-gate-entries` shows the form, a plan of its own. `close` says what the figures support.

## Tests that go

None. A task that finds one must go adds its row here first, by test name, with what holds its behaviour now (`notes/common.md`).

## Overlaps this plan could not avoid

`notes/common.md`, *Beside this plan*, lists what other drafts hold.

- `public-wire` and `one-process-sweeps` meet none of it.
- `state-hash` meets `account-email/facts-record` on derive lines in `jig/crates/jig-domain-accounts/src/{changes,domain,restore}.rs` and `jig/crates/jig-domain-core/src/start.rs`. Those are one-word edits, which merge unless that draft changes the same line.
- **Any draft that adds a state type this closure reaches, once `engine-sweeps` lands, must derive `Hash` on it, or `temper-engine-domain-world` stops compiling.** `account-email`, `model-catalogue-3` and any other draft in flight may meet this at a merge, and the fix is one word. That's the design's rule (`skein/docs/design/lib.md`, 5.1) starting to bite.
- `under-causes` may meet `account-email/status-email` in smith, and `diagrams` in `temper/crates/temper-service/src/stock/**`, only if a cause sits there. It keeps its hunks small.
- `close` meets no other draft in `temper/docs/design/domain/testing.status.md` as far as the brief lists, and it runs last.
