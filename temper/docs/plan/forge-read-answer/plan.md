# forge-read-answer: the plan

2026-11-07. From `inputs/request.md`: an agent's forge read never takes down `temper serve`.

## The cause

Read, not run (`notes/contracts.md`, section 1). `ui-qa-small`'s merge (`aaa951702a`) changed no forge read and no admission: client pages, the configuring stock and `agent.md` 8.1 only. The root names an agent's read `serial | 1 << 61` from its own serial; the connector names its change steps' and landings' reads `sequence | 1 << 61` from its own sequence, and claims an answer for its own reads before passing the rest to the root. When the two meet, a step takes the agent's `Busy` (the client refuses at once under load) and drops its owner; the step's own `Commits` (or a landing's `Tree`) then goes to the root, which answers the agent's `read_file` with it, and the rendering's `unreachable!` stops `serve`. That attempt's `Busy` is never noted, which fits the live log's two refusals and then the panic. The client never answers a kind its call didn't ask (`api::accepts`), so nothing else can bring such a kind.

## Parts and tasks

| Part | Task | Needs | Request item | Crates |
|---|---|---|---|---|
| `owners` | `read-owners` | none | 1, 2 | `temper-engine-domain-forge`, `temper-engine-domain` (`route/forge_route/reads.rs`), `temper-engine-domain-world` |
| `render` | `unasked-answer` | none | 3 | `temper-engine-smith`, `temper-service` (`route.rs`) |

- **`read-owners`:** a story in the engine domain's world, red first: a run's `read_file` attempts are refused `Busy` while a change step's read is held, sweeping the root's serial past the step's sequence; each must hear its own `Busy` and the run must get its own `File`. Then `root_read` and `is_root_read` (`forge.md`, 5.3, *Each answer reaches the read that asked*), the root minting by them, the router passing by them, and the step's and landing's owners moved to `1 << 62`. A table pins the range.
- **`unasked-answer`:** `Runs`, `Run`, `Commits`, `Tree` and `Queued` on a forge read render as a refused read naming the tool and the kind, `Problem::Unasked` in the rendering, and a `host-render` read refusal note for stderr (`agent.md`, 8). Rows in the existing `root_call_answers_become_bounded_smith_text_or_error`. It holds whatever the cause, including for a store that already recorded such an answer.

Both start at once: they share no crate and no file.

## Critical path

`read-owners`, about 120 implementer-minutes, alone on its path; `unasked-answer` (about 45) runs beside it. The plan lands when both have merged and the trunk's end check is green.

## Item 4: the live stories' fallout

Read, not run. `read-owners`' story reproduces the mechanism below the live tier: a run's `Busy` taken by a change step and the step's `Commits` on the run's `read_file`; its report says what the base's failure showed. The live stories need no change beyond the fix: their failing assertion and their 40-minute wait follow from `serve`'s death. The box restarts a `serve` that exits (`temper/docs/design/protocol/setup.md`, 7.2), so the referee's check for a stopped box (`temper/crates/temper/tests/support/forge_land.rs`, *live box stopped before its reads*) doesn't fire, and the story waits on its patience. Whether the restarted `serve` took up the run again is unknown without the log. This plan changes nothing the live stories compile; after-landing runs them.

## Decisions

- The owner range is the root's by a contract in `temper-engine-domain-forge`, the crate that routes, not by convention in two places: the root and the router both use `root_read` and `is_root_read`.
- The connector's change steps and landings move to `1 << 62`, which the lost-writer reads already use from the same sequence; one counter keeps their low bits apart.
- The log line reuses `Note::ReadRefused` at layer `host-render`, the layer the root already gives refusals made while rendering, so `temper serve`'s line format is unchanged. No stored format, protocol or wire changes.
- Nothing dropped: three of the four items are tasks, and item 4 needs no change.

## Overlaps

None with the drafts beside this one (`look-round-16`, `per-wake`: skein-ui, the client, jig). The request warns that other areas write `temper-engine-domain` and `forge_route.rs`: this plan's root hunk stays in `route/forge_route/reads.rs`, and `forge_route.rs` itself is untouched.
