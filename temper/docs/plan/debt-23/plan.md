# debt-23: the twenty-third debt round

Two independent parts (`inputs/request.md`): the spot review of debt-21, and the items skein D023, jig D016 and hearth D006. No temper client view, no `temper.css`, no `jig-client-view`. This file grows with each push.

## Parts and tasks

| Source | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| review 1 | `spot-review` | `connect-close` | | medium | 15 | `connect` keeps a close; the pin fails first |
| review 8 | `spot-review` | `settings-name` | | medium | 30 | the value-mismatch abort; `Name`; closing gates before it clones |
| review 3, 7 | `spot-review` | `connection-deadline` | `connect-close` | medium | 30 | the deadline is derived from the state |
| review 4 | `spot-review` | `connection-states` | `connection-deadline` | high | 45 | `Closed` and `Idle` hold no exchange, `Closed` no deadline |
| review 2, 5 | `spot-review` | `hook-slack` | | high | 45 | the pin across `Done`, then the slack |
| review 6 | `spot-review` | `oauth-secret-fields` | | medium | 60 | five fields become `Secret`; their records derive `Debug` |
| review 7 | `spot-review` | `oauth-noise` | `oauth-secret-fields` | medium | 15 | hand-written clones and a duplicate helper |
| jig D016 | `payloads` | `args-core-party` | | medium | 60 | ten core sites, party and settings |
| jig D016 | `payloads` | `args-core-turns` | `args-core-party` | medium | 50 | seven core sites, turns and views |
| jig D016 | `payloads` | `args-core-calls` | `args-core-turns` | high | 70 | ten core sites, named calls and effects |
| jig D016 | `payloads` | `args-core-checks` | | medium | 30 | settings' checks split; `Creator` |
| jig D016 | `payloads` | `args-tasks` | | medium | 90 | tasks' 16 sites |
| jig D016 | `payloads` | `args-fleet-host` | | medium | 55 | fleet's five, host's three |
| jig D016 | `payloads` | `args-people-views` | | medium | 45 | people's four, views' three |
| jig D016 | `payloads` | `args-small` | | medium | 25 | authority, accounts; the two seams' comments |
| jig D016 | `payloads` | `args-store` | | medium | 20 | `manifest`, `seal` |
| jig D016 | `payloads` | `args-ops` | | medium | 25 | ops' three |
| jig D016 | `payloads` | `args-close` | the ten above | medium | 10 | the item goes |
| skein D023 | `llm-states` | `llm-connection-phase` | | high | 40 | `Phase` holds the call; `idle_at` goes |
| skein D023 | `llm-states` | `llm-client-states` | | high | 120 | `Client`'s states hold their data; a pin first |
| skein D023 | `llm-states` | `llm-states-close` | the two above | medium | 15 | the item goes; statuses keep what's left |
| hearth D006 | `builder-lifecycles` | `builder-child` | | high | 60 | `git::Child`; `Held` |
| hearth D006 | `builder-lifecycles` | `builder-resolve` | `builder-child` | high | 75 | `Resolve`'s stage; a pin first |
| hearth D006 | `builder-lifecycles` | `builder-read` | `builder-child` | high | 75 | a read's current work and feed |
| hearth D006 | `builder-lifecycles` | `builder-client` | | high | 45 | a client's output and stream |
| hearth D006 | `builder-lifecycles` | `builder-current` | `builder-child`, `builder-resolve`, `builder-read` | high | 105 | `Protocol` holds one current call |
| hearth D006 | `builder-lifecycles` | `store-states` | | high | 90 | `Store`'s phase, standing, flight; a pin first |
| hearth D006 | `builder-lifecycles` | `lifecycles-close` | the six above | medium | 15 | the item goes |

1360 implementer-minutes in all: 240 in `spot-review`, 480 in `payloads`, 175 in `llm-states`, 465 in `builder-lifecycles`. No debt item closes in `spot-review`, by the request.

## What was decided

- **Finding 1 is right.** `Link::connected` answers `Start::Closing` only once `Link::close` has run, which only `Connection::close` and `abort` call. `ended` survives the notice now as well as `closing`; the spec makes a test that sees it a question.
- **Finding 8's abort is the value mismatch only.** `drop_project` and `restore` skipping a setting that isn't an application's isn't a value mismatch, and the request brings back only that. `core.status.md` reads "No gap" and no status under `jig/docs/design/domain/` recorded otherwise, so none changes. The abort guards `check`'s promise, which `D017` says stays, so that item doesn't change either.

- **Finding 4 is right, and its two tests go.** Each claim was checked against the code. `Closed.exchange` is `Some` only from a raw overflow, and all three readers of `Moved::Finished` call `finish` in the same arm. `Idle.exchange` needs `connect` while an exchange runs, which only its test does. `Closed.until` is cleared by the sweep in the same `drive`. Removing state nothing outward reads is what the finding asks, so the request's "don't rewrite an assertion" doesn't stop those two tests going; `connection-states` is told so, and to stop if another test moves. `Receiving::Json { body, next }` is the one conditional clause: `discard` sets `Empty` without clearing `json_down`, so the task takes it only if a reading finds it safe.
- **Finding 3's trap is named in its spec.** `self.deadline` holds the old value after `self.state.take()`; a derived one doesn't. `State::idle` reads its own `until`, or `self.state.take().idle(..)` would read `None`.
- **Finding 5 keeps `Hint { value, offer }`.** All six pairs are reachable: a `Done` with no hint leaves `Ready` with no value, and a `204` over a refused hint can leave `Refused` with none. One enum would list the same pairs, so finding 5's "only if" fails. `pump`'s guard goes, since its one caller asks the same and `stream` feeds nothing once closing.
- **Finding 6's caveat is recorded, not redacted.** `subject`, `metadata`, `granted_scopes`, `issued_client_id` and `Redirect.error` print as they came. Redacting them would add hand-written `Debug` back, the thing finding 6 removes. `skein/docs/design/oauth.md`, 4 says so, in this plan's commit, and `lib.status.md`'s **Departs** on 6 holds lib.md's wider sentence against it. `oauth::Event::Redirected`'s `uri` and `oauth::Redirect`'s stay: an address without its query, no credential.
- **Finding 7 loses one clause.** `record.rs`'s `secret` helper has nothing to share now: its fields are `Secret`. Its `component/github.rs` line goes to `connection-deadline`, which holds that file. `expose_optional` goes through an exported `secret_ref`.

- **D016 reads 2, not 0, when it's paid.** Its **Paid when** allows a seam a comment names as kept whole. `Domain::select` and `sources::Component::start` are the two public signatures among 71 sites, and the request stops at a breaking public interface; `select` also takes borrowed slices no owned fold may hold. So the item's target is those two, and `args-close` checks for it. This is the item's own clause, so it isn't a question.
- **No `Event` variant changes.** D016 proposes `Event::Turn(Turn)`, which would break readers across jig and temper, up to 14 packages for `people::Event::Ask` alone. A private struct at the helper boundary pays every site, as routes unpack a variant anyway. The item's worry, a field threaded by hand, holds for the variant's own arms still. That's what the **Paid when** leaves.
- **Folds own their values.** `skein/docs/foundation/programming-model.md`, 10.3 rules out references in step code's fields, so `core, env, work` stay loose and no `Route<'a>` comes in. `notes/arguments.md` holds the rules and every site's fold.
- **The survey's 13 tasks are 10 here.** Fleet with host, people with views, and authority, accounts and worker-agents go together, as each is a few sites in a crate no other task holds.
- **D016's item was stale in its crates.** It lists no accounts, authority, host, store or worker-agents, which hold ten of the 71; 71 is today's count, not its 68.
- **These folds add lines,** about +200 in all, as a struct costs its fields. The request asks net lines to fall only of the spot review.

- **D023's "call" is `Connection`'s `Phase`.** `call.rs` has no `Call`, and both statuses say so. `Phase` takes the token; `idle_at` is write-only and goes, and `waiting` and `reservation` stay, for the reasons the status now gives.
- **D023's "then follow" has one site, not two.** `call.rs` has `sync_deadlines`, which stays. `Client` mirrors nothing: `resume` derives its read demand from the state on each entry, so it has no `follow`, and a function with nothing to do would be noise. This is the cut's reading of the item's words, recorded so fabrig/debt can differ. The acceptance is the item's **Measure**, and no comparison decides a transition.
- **`Client`'s machines stay on the struct.** `close_sse` closes them from any state, and `has_work` spends one route on the `Closed` it leaves. `llm.md`, 3's table has the SSE reader under Streaming, so the status keeps that as a departure, with this reason.
- **One pin for D023, none of its transitions.** A rejected head reporting its request id is the one path a split loses silently that no test holds. The rest have tests already, and the survey's traps are in the spec.
- **D006 goes ahead of D001.** The item says no plan should take it first, and D001 is open at `max_out 53`. But the request names D006, and the overlap is adjacent lines, not shared logic: `MaxOut`, `up`'s `!above.is_empty()`, the store's `MAX_OUT` and `client.rs`'s two queue declarations stay untouched, and `git.rs`, `resolve.rs` and `read.rs` have no capped queues left. Neither `hearth-service-builder/src/lib.rs` nor any `MAX_OUT` form is needed, so fabrig/debt isn't asked.
- **D006's item is wrong in four small ways.** Its "about 24 `self.` sites" are 22 plus 28 `protocol.` lines. `failed`, `refused` and `ended` are facts read after a lifecycle, so they stay, and **Paid when** doesn't ask otherwise. Its **Where** omits `events.rs`, `keep.rs`, the protocol's `store.rs` and five of the store's modules, all of which change. Its **What** is silent on `read.rs`, which has the pattern whole, so it gets a task.
- **`ran` may stay a field.** Nothing proves `Run` excludes other work. `builder-current` asserts it by hand first, and if they coexist, `ran` isn't an `Option` that holds in turn, so **Paid when** still holds. That's pre-approved.
- **Two pins for D006:** a deadline during `Resolve`'s indices, and a failed entry's later finds answering `Unusable { List }`. Both paths would change silently, and no test holds them.
- **`lifecycles-close` can't be a Measure.** The item has none, so its searches stand in for the reviewer, with a read of the six types by eye.

## Seen, left

- **Possible hearth bug, read and not run, for fabrig/debt.** If standard error breaks and closes during a lock wait, and the store then answers `Unopened`, `progress.stop()` queues `Down::Finish`, and `resume` then hits `self.output.expect("bound output")` with no output. Behaviour stays unchanged here, so no task fixes it.
- `hearth-protocol-builder`'s `work_pending` is effectively unpinned, as D001 records. `builder-current` reviews it by hand; a test of it is D001's.
- `read.rs`'s `take_answer` `Cached` arm looks unreachable. `builder-read` keeps it.
- `State::Waiting` in fleet could hold `Claiming`. `args-fleet-host` takes it only if shorter.

## Overlaps

- The spot review's tasks meet no other draft's files. `temper-fake` and `temper-service` are other drafts' too (`ui-away`, `ui-chat-draft`), in other files: `oauth-secret-fields` changes `temper-fake`'s `oauth` tests and `temper-service`'s `clients/google.rs` and its `sign_in` and `accounts_host` tests.
- `routing/run/calls.rs` (`note-parts/note-range`), `effects.rs` (`ui-chat-draft/note-answer`), `routing/mod.rs` and `routing/run/preparation.rs` (`ui-chat-draft/note-scope`) couldn't be avoided: D016's sites are there. `args-core-calls` goes last of core so its two files come latest, and `Call` and `Proposing` live in `routing/run.rs`, which no draft holds. `jig-domain-people`'s `batch.rs` and `limits.rs` (`ui-chat-draft/person-words`) are named off limits to `args-people-views`.
- D023 and D006 meet no listed draft. D006 is the one most likely to meet fabrig/hearth's memory migration, which isn't a listed draft: `builder-client` keeps `client.rs`'s queue declarations as they are for that, and `builder-current`, `store-states` keep `MaxOut` and `MAX_OUT`. `skein/docs/design/link.md` is touched by `llm-states-close` alone.
- Inside this plan: `builder-child` and `builder-current` hold `hearth-protocol-builder/src/lib.rs` in order. `builder-resolve` and `builder-current` hold `resolve.rs` in order, the second in its tail. `builder-read` and `builder-child` hold `read.rs` in order. `builder-client` and `store-states` run beside anything. `llm-connection-phase` and `llm-client-states` share no file, and the second only uses the first's public API.
- Inside this plan: `routing/mod.rs` is the one caller of 14 core sites, so `args-core-party`, `args-core-turns` and `args-core-calls` run in order. `args-core-checks` shares nothing with them. `settings-name` and `args-core-party` both touch `settings/application.rs`, one call in the second; `settings-name` and `args-tasks` touch `jig-domain-tasks`, in `closing.rs` and elsewhere.
- Inside this plan: `connect-close`, `connection-deadline` and `connection-states` hold `connection.rs` in order. `hook-slack` holds `hooks/connection.rs` alone, beside them. `oauth-secret-fields` and `oauth-noise` hold `skein-fake-oauth/src/lib.rs` in order.

## Critical path

`builder-child`, `builder-resolve` or `builder-read`, `builder-current`, `lifecycles-close`: 255 minutes. Then `args-core-party`, `args-core-turns`, `args-core-calls`, `args-close`: 190 minutes. No part waits for another.
