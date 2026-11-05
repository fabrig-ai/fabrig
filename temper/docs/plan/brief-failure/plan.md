# brief-failure: a brief that can't be gathered says why, ends, and survives one section

From the first real goal on a deployment: a CI repair `Retrying`, `retry: refused`, count 25 and rising, with no reason anywhere and no end. The request (`inputs/request.md`) has the route, confirmed by reading: a forge read error falls to `BriefRead::Failed`, a required section's `Missing` fails the brief, `tag_brief` sends `PreparationFailed`, and `run::pause` has no bound.

## Parts and tasks

| Part | Task | Needs | What |
|---|---|---|---|
| `brief-words` | `brief-words` | | `Fate`, `Failure`: each ending of the brief child says why and how it counts |
| `retry-reason` | `retry-reason` | | the retry notice's reason, the counted event, the hold's words, `tasks-v9` |
| `brief-counts` | `brief-counts` | `brief-words`, `retry-reason` | the core routes each failure by 9.1's table; the stories in `jig/tests/core` |
| `reason-shown` | `reason-shown` | `retry-reason` | the client's wire, the chip, `temper ctl task show` |
| `forge-sections` | `forge-sections` | `brief-words`, `brief-counts` | the forge's REASONs, a repair's pair, the log lines, the stories in the forge world; the walk |
| `forge-answer` | `forge-answer` | | the cause, from the captured answers in `notes/pr7/`; nothing waits for it |

Critical path: `retry-reason`, `brief-counts`, `forge-sections`, then the walk. `reason-shown` and `forge-answer` run beside it.

Six tasks where the request guessed three or four: one writer per crate keeps the kit's surface, the core's routing, the client's wire and the forge apart, and the first two are small so the rest aren't held.

jig is shared with hearth, whose manifests name no jig crate, so this change doesn't reach it. `skein-forge-client` isn't on hearth's surface.

## What was decided

- **A failure carries `Fate`: capacity, transient or permanent,** and its words. `jig/docs/design/domain/engine.md`, 9.1 names each. Capacity is a brief slot, a connector's room, a client's admission, the core's own tables and a claim.
- **A forge that isn't connected pauses.** `NotConnected` is no API call made, and `temper/docs/design/domain/forge.md`, 8.4 already says a wait for access doesn't count toward a stall. Holding every repair through an outage would make the person release each one. `Busy` is the same: no call made. Both still say why.
- **A forge that can't be reached counts as transient.** `Unavailable` and `Timeout` are `tasks.md`, 5.5's "a peer unavailable", and the client already waits a read out before it says so. A long outage does hold a repair, and the hold names why; the allowance is the user's.
- **A late required section counts as transient,** even where a full client queue made it late: only a refusal at admission is capacity by the client's own word.
- **Reasons are `&'static [u8]` in the kit's types.** "No LLM-written or secret words" then holds by construction, and a forge's answer can't leak into one. Only the stored form is bytes.
- **The reason sits inside `RetryNotice`, not beside it,** so a later notice can't stand with an earlier reason. `RetryNotice` loses `Copy` for it.
- **A hold's reason is a new `HoldSource::Preparation`, not `Account`.** `Account` makes the document name a connector's hold part to read, which a brief's failure hasn't got, and a core section has no connector at all.
- **The pair is the forge connector's to decide,** not jig's or the root's. "A repair with neither" is a forge rule (8.4), the root only routes (`jig/AGENTS.md`), and a jig "one of these" would be vocabulary for one user. The reason's plan names its partner.
- **A lone section stays required.** A review's or a resolution's pull section is what the run is for (`engine.md`, 9), so only a repair's pair becomes words. A capacity refusal never becomes words, so a run isn't started short for want of room.
- **The core's other preparation failures are in the table too,** not only briefs': a dependency's result gone, a transcript's window, the notes index and a spent attempt number paused without bound by the same route, so leaving them would leave the same fault.

## Overlaps that could not be avoided

- `jig-domain-core`'s `routing/run/preparation.rs`, `routing/mod.rs` and `routing/effect/note.rs` (`ui-chat-draft`, `note-parts`): `brief-counts` keeps to its call sites, two lines in `note.rs`.
- `client-v19.schema`, its generated file and goldens (`ui-chat-draft`, and others before it): `reason-shown` adds three fields; a merge regenerates. It starts after `retry-reason` alone, so it needn't wait for the routing.
- `temper-engine-domain-forge`'s `boundary.rs`, `domain.rs` and `temper-engine-domain`'s `boundary.rs`, `route/forge_route.rs`, `assemble.rs` (`ui-away`, `ui-chat-draft`, `adopt-access`): `forge-sections` goes last and keeps each hunk small; its new work is in `brief_words.rs`, `brief.rs` and `held.rs`, which no draft beside this one lists.
- `temper-client-view/src/tests/chats.rs`: one forced literal (`reason-shown`), the only line under fabrig/ui's crates.
