# watch-room: the plan

From `inputs/request.md`. Ranked, with a ceiling of four tasks. Everything below was judged from reading the code at `bc3996bb40`. No test ran for this cut, so each figure is from reading until its task reports.

## Parts and tasks

| Request's item | Part | Task | Needs | Minutes |
|---|---|---|---|---|
| 1, the failing story | `watch-story` | `story` | | 75 |
| 2, the budget | `watch-rooms` | `size-budget` | `story`, `link-told` | 130 |
| 3, a client says so (jig) | `link-told` | `link-told` | | 30 |
| 3, a client says so (temper) | `ctl-told` | `ctl-told` | `link-told` | 40 |

**Nothing is dropped, and this is the ceiling.** Item 3 takes two tasks because a branch stays in its tree: the fact is jig's and the words are temper's.

**Critical path:** `story` then `size-budget`, about 205 implementer-minutes. `link-told` then `ctl-told` is about 70, and they run beside it.

## What was decided

- **The rooms follow `clients.streams`.** A stream's room stays 17, one shipped client's. The deployment's is `clients.streams` times 17: 136 for `temper init`'s 8, 34 for the small profile's 2. `clients_http.streams` adds no room, since it bounds the transport's side of the same streams.
- **A party's room is the deployment's.** The request says a person's tabs are one party, and one person may hold every stream. What a person can use up is the streams, and that refusal already names them. A fair share would lock a person's own tabs out again, so it goes under section 11's open questions, with no share as the default.
- **Not a setting, and the two names go.** The request asks which, so here it is: more room is more streams, and `views.watchers` and `views.watchers_per_party` are refused at the start, naming `clients.streams`.
  - **Why not make them mean what they say:** jig's small profile has them at 4, below one client's own 17, so honouring them would need a floor that takes the meaning away again.
  - **Why refuse, not ignore:** a silent no-op is what misled the investigation. It's a configuration name, not a stored format or a protocol, so it isn't a question first, and formats change in place before v0.1.0.
  - **jig keeps the names,** since jig's own root takes them. `views.`'s other names stay taken and ignored. Nobody was misled by them on record, so they go under section 11 too.
- **`VIEWS` keeps its name, as one stream's limits.** `core_limits` scales it through `views_for`. This keeps `size-budget` off `forge_route.rs`, which `update-recovers/update-hold-words` is changing, and leaves `reloads.rs` and the configuration test true as they stand.
- **Written-out worlds stay at `streams: 1`, so their room stays 17.** No existing story or ceiling changes meaning, and only stories that mean the shipped figure set 8.
- **The story lands failing, under `should_panic`.** The test states today's failure and runs on every focused run, where an `ignore` could rot. `size-budget` has to remove it: the moment the room is sized, the test fails until it does. Each task's branch stays green.
- **`watches_full` is a `bool` on jig's link.** A refusal carries no count, and a count would be a protocol change, a question first. So the line says "all the engine's watches", with no number. It covers an application's watch too, read from the watches themselves, so it can't go stale on the next `me` delivery as `streams_full`'s clearing would. Busy has rarer, passing causes (exchanges, a load slot). The words are the request's, and at a 30 s deadline a passing cause has passed.

## What reading found, for `size-budget` to measure

- **Capacities allocate nothing until used** (`Slab`, `Map`, `Set` in `skein-lib`), so a larger room costs nothing until it's held. No heap sum admits the root, so no startup figure moves.
- **A commit's pass builds one document per marked watch,** not per subject (`jig-domain-core`'s `routing/view/routes.rs`, `made_outputs`), and holds each until it's delivered. That's the real cost of a full room, and the new ceiling measures it.
- **`subjects_per_change` is safe.** The marks pass sends one subject at a time, and the forge's `MARKED` is checked against 32, so a larger room can't breach it.
- **A full room's 136 deliveries a commit, 408 over 3 commits, fit `root.journal.deliveries`' 512.** The task confirms this.
- **The store's bounds don't follow.** A watch holds no row.

## Overlaps with other drafts

- **`ctl-told` and `ctl-waits/deadline-words`, `ctl-waits/kept-answer`:** `host.rs`, `src/tests.rs`, `src/tests/fixtures.rs`, `docs/help/ctl.md`. Unavoidable, and the request expects the catch-up. It's kept to one `if`, one `mod` line, one bullet and a new test file. The spec gives the order to keep.
- **`ctl-told` and `ctl-words/key-words`:** `docs/help/ctl.md`, `tools.status.md`, each a line of its own.
- **`story` and `long-words/composer-view`, `ctl-waits/kept-answer`:** `temper/tests/client/tests/suite/main.rs`, one `mod` line.
- **`link-told` and `ui-pgup/live-kept`:** `jig/docs/design/client/domain/client.status.md`, each its own line.

## Left outside

- **The request names `temper/docs/help/reference/limits.md`.** That page holds `application.limits.overrides`. `clients.streams` and the two `views.` names are on `limits-engine.md`, generated by the same table, so that's the page that follows.
- **A person's page past the room** (`No chats yet`, `Behind`) is temper/ui's, being fixed apart. `size-budget` reports what it sees.
- **The browser re-check** is temper/forge's, by hand, after landing. No browser story is written.
