# hearth-memory-6: the plan

Round 6, the last of hearth's move to the dynamic memory model: the service, and the end of D001 and D004. The design is `hearth/docs/design/domain/render.md`, 8.4 and 8.5, which this cut wrote first, with `hearth/docs/design/domain/testing.md`, 8.4, 8.5 and 11, and their status files. The shapes that cross tasks are in `notes/contracts.md`, and the standing rules in `notes/common.md`.

## Parts and tasks

| Part | Tasks | Needs |
|---|---|---|
| `before` | `figures-before` | nothing |
| `lists` | `hand-lists` | nothing |
| `stage` | `stage` | nothing |
| `orders` | `store-step`, `stuck-client` | nothing |
| `queues` | `uncap` | `stage`, `store-step`, `stuck-client` |
| `constants` | `constants` | `uncap` |
| `resumed` | `resumed-order` | `uncap` |
| `close` | `close` | `constants`, `resumed-order`, `hand-lists` |
| `after` | `figures-after` | `close` |

The critical path is `stuck-client`, `uncap`, `resumed-order`, `close` and `figures-after`: about 210 implementer-minutes. Five tasks start at once, and `hearth-service-builder/src/lib.rs` has one writer at a time: `stage`, then `uncap`.

## What was decided

- **The backlog is `told`, `file_told`, `answers` and `asked`, and only `domain::resume` waits on it.** This is 7.1's form. `calls` stays out: while render is ready the domain returns before taking events, as it does today, so a `calls` counted in the backlog could hold the watermark shut for good, at any watermark `calls` can reach. Left out, the backlog drains with no help from `calls`: `answers` goes down as the printer takes it, and `asked` goes to io. So what waits at a stuck client waits on a real sink. An event in `calls` still waits behind the domain's own next resume, as today, and `render.md`, 10 keeps that question for the first build step that streams.
- **`limits.queue` stays,** because the two queues shared with the kernel use it. `Service::new` asks only that it and the watermark aren't zero. Nothing else explained 3.
- **The watermark is also a pass's bound on a layer's resumes.** Other trees pace a queue by what it held when the pass reached it, which works here for every queue, but a resume has no queue, and render's has no natural stop. A separate number would be a second knob with no use, and the request asks for one. It's 30, near today's 32, so the pass stays about the size it was. `a_pass_stops_short_of_a_whole_render` holds it, by counting passes that ended with render ready under the watermark: only the bound can have stopped those.
- **`Asked` keeps one order.** `stage::down` takes `Io`'s requests alone, so `io_asked` holds at most one, and the next request isn't popped until it's gone. Splitting `asked` into two lanes would have let a file request pass an `Io` one, which no document asks for.
- **`stage` also uncaps `io_submissions`, `file_told` and `file_submissions`,** since only `io_up` and `io_down` push to them. `io_up` and `io_down` then end in their final form in one task, and `uncap` needn't reopen them. `stage::up`'s own room check then never binds, and `iterate`'s last loop is the one edge, which is what lets `limits.queue >= 3` go.
- **`stuck-client` comes before `uncap`, and passes on today's code,** as 7.2 asks. It holds the output in the world's own `Shared`, with no change to skein. Its first bite raises `limits.queue`, and `uncap`'s bite takes `admits` out.
- **`hand-lists` is a task because D001 goes.** The request doesn't name its rows, which keep nothing, but each `.room() == 0` there would lose the only line that explains it.
- **`resumed-order` runs after `uncap`,** so the search is of the guard in its final form, with no room check beside it. It can't hold back anything but `close`.
- **Three test drivers keep a queue of 1,** where a second output panics, as `hearth-domain-builder/src/tests.rs` already does. The protocol world's four queues go uncapped, because they stood for the service's.
- **`hearth/docs/design/README.md` and `domain/steward.md` don't change.** The request names them where this changes what they say. Read for it, neither names a queue between the builder's layers, a room check or a `MAX_OUT`. `steward.md`, 3.4's queue is of jobs, between processes, and stays. Their status files don't change either. The draft `brainstorm.md` says nothing on this, so no draft's status lags.

## The survey

Every reader of what changes, from the trunk at this cut:

- **`MAX_OUT`, `MaxOut`:** the table in `tasks/constants.md`, and `hearth-service-builder/src/lib.rs`, which `stage` and `uncap` clear first.
- **`tag`:** the request says `hearth/tests/` knows about it where it reads tokens. A search found nothing reading a token's parity or shift, so `stage` says what it found and pre-approves following any it meets. `fixtures::STARTUP_CLOSE` reads as the file lane with `tag` gone, but the world's wrapper takes it before the service sees it.
- **`limits.queue`:** `hearth/tests/builder/tests/suite/bounds.rs`, `orders.rs` and `simulated/client.rs`. No stored format or deployment holds it. `Limits` is built only in `limits.rs`, so `watermark` touches no other literal.
- **`Markdown read by tests`:** none of these files is in `.config/gate/read-by-tests.txt`.
- **The citation check skips `docs/plan/`,** so `hearth-memory-5`'s citations of D001 survive its deletion.
- **`fixtures::CUTS`** counts the client's submissions. Routing and pacing change none, so a change to it is a finding.

## Overlaps

`debt-26`, on its trunk and not on main, changes `hearth-protocol-builder/src/lib.rs`, `keep.rs` and `read.rs` (`debt-26/hearth-spot`): `Current`'s accessors and `finish`. Two tasks here change that `lib.rs`: `store-step`, in `up` alone, and `constants`, the `MaxOut` lines at the top. Neither meets those hunks, though they sit close enough that the merge may still need a look. `store-step`'s spec keeps it to `up`.

No other draft beside this one lists a file under `hearth/`.

## Figures that don't agree

The request gives hearth's focused suite as about 32 CPU s. `hearth/docs/design/domain/testing.status.md`, 9.1 records 44.886618 CPU s by `tools/tiers/cpu.py` on 2026-11-05, and says which of the two measures the suite is unknown. `figures-before` measures it first, on the code as it stands, and `figures-after` the same way, so that the comparison holds even if neither matches 32. This round doesn't aim at that figure: it adds a few small tests, one of up to 0.3 CPU s, and takes out checks that cost little. `figures-after` answers the request's question plainly.
