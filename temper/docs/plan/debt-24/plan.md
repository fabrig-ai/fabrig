# debt-24: the twenty-fourth debt round

Two independent parts (`inputs/request.md`): the spot review of debt-22, and the test items skein D028, D030 and jig D029, D031, D035. No temper client view, no `temper.css`, no `jig-client-view`. This file grows with each push.

## Parts and tasks

| Source | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| review 1, 7 | `spot-review` | `answer-rows` | | medium | 25 | `answer_rows` moves to `skein_forge::measure`; `within` and two tests go |
| review 6 | `spot-review` | `forgejo-repository` | | medium | 35 | `Repository::at`, `id`; one `refused`; `limit` goes |
| review 6, 7, note | `spot-review` | `forgejo-written` | `forgejo-repository` | high | 40 | `Writing` holds a small `Written`; explicit rows |
| review 3 | `spot-review` | `oauth-sign-in` | | high | 45 | the sign-in's fields move into `Grant::SignIn` |
| review 5, 7, note | `spot-review` | `accounts-lending` | | high | 50 | `Notice` splits; `has_work` asks what `fire` asks, pin first |
| review 4 | `spot-review` | `accounts-entry` | `accounts-lending` | high | 75 | `AccountEntry`'s methods; one `let entry` per function |
| review 8 | `spot-review` | `jig-tables` | | medium | 25 | two tables lose what can't fail |

295 implementer-minutes so far, all in `spot-review`. No debt item closes in it, by the request.

## What was decided

- **Each finding was read against the code.** All hold, but for the sub-finding below and the small drifts the tasks name (92 `expect("account entry")`, not 100; 22 `entry.name.number()`, not 24).
- **Finding 2 stays alone,** by the request, and each spec's common note says so.
- **Finding 1 keeps a public path.** `skein_forge_client::measure::answer_rows` stays as a re-export, because `temper/tests/engine/forge` reads it; so no other tree changes and no public interface breaks.
- **Finding 8's third copy is skipped.** `settings::moves`, `codec/tasks/models.rs`'s `moves` and `codec/people.rs`'s `model_moves` read three different generated `ModelName` types, with a `name` field or a `value()` accessor, and step code holds no generic item (`jig/AGENTS.md`). A fold would allocate a list per decode to save six lines, so the -14 isn't there. The review was wrong to call these a copy that can go.
- **Finding 5's `held` is the implementer's, on lines.** Returning `Lending` removes two `if`s and adds a comparison at eight sites, so it's taken only if net lines fall.
- **Finding 6's `Written` has a gate.** The review's -15 looks high: the enum and its builder cost about 30 lines, against `conflict`'s and `expected`'s matches. It stops `initial` cloning a whole write and keeping its body through the exchange, so it's worth taking at zero. Past that, it's skipped with the measured size.
- **The first correctness note holds: not reachable.** `writes::Plan::errors` panics in `Ended`, and `begin` and `heard` leave `Ended` only by returning `Err` or `Step::Done`, on which `skein-forge-connection/src/component/calls.rs` finishes the call. `forgejo-written` reads it again as it changes those paths.
- **The second correctness note looks real, and gets a pin.** `has_work` says a due record with no `earliest` has work; `fire` also needs the account live, no binding and a refresh token. A sign-in over a record past its lead, or a token-less rejected record, reads as work `fire` won't do, and `jig-protocol-accounts`'s `work_pending` reads it too. No visible event changes, since `fire` does nothing there. `accounts-lending` writes the pin first: if the state can't be reached, the note is wrong and nothing changes.
- **`ask_classes` keeps `SetRoles` and `RemoveProject`.** `project`, `capacity::project`, `deletion::ask_project` and `reserves_answer` all read `ask.family()`, so a further `F::Project(7)` row repeats one arm of `Ask::family`. `SetRoles` is what the roster test needs. Rows `models::selectors` reads go only after a hand break shows another test holds them.
- **`collaborators_finish_without_measuring_the_rows_again` goes:** it pins `rows: 0`, a bound no client builds.

## Overlaps

- The spot review's tasks meet no other draft's files. `jig-domain-people/src/tests.rs` and `jig-domain-tasks/src/tests.rs` aren't in any listed task's files.
- Inside this plan: `forgejo-repository` and `forgejo-written` hold `writes.rs` in order, and `accounts-lending` and `accounts-entry` hold `skein-oauth-accounts`' `component` in order.

## Critical path

So far: `accounts-lending`, then `accounts-entry`, 125 minutes. The test items are still being cut.
