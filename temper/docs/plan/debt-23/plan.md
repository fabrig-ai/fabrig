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

The three items follow in later pushes. No debt item closes in `spot-review`, by the request.

## What was decided

- **Finding 1 is right.** `Link::connected` answers `Start::Closing` only once `Link::close` has run, which only `Connection::close` and `abort` call. `ended` survives the notice now as well as `closing`; the spec makes a test that sees it a question.
- **Finding 8's abort is the value mismatch only.** `drop_project` and `restore` skipping a setting that isn't an application's isn't a value mismatch, and the request brings back only that. `core.status.md` reads "No gap" and no status under `jig/docs/design/domain/` recorded otherwise, so none changes. The abort guards `check`'s promise, which `D017` says stays, so that item doesn't change either.

- **Finding 4 is right, and its two tests go.** Each claim was checked against the code. `Closed.exchange` is `Some` only from a raw overflow, and all three readers of `Moved::Finished` call `finish` in the same arm. `Idle.exchange` needs `connect` while an exchange runs, which only its test does. `Closed.until` is cleared by the sweep in the same `drive`. Removing state nothing outward reads is what the finding asks, so the request's "don't rewrite an assertion" doesn't stop those two tests going; `connection-states` is told so, and to stop if another test moves. `Receiving::Json { body, next }` is the one conditional clause: `discard` sets `Empty` without clearing `json_down`, so the task takes it only if a reading finds it safe.
- **Finding 3's trap is named in its spec.** `self.deadline` holds the old value after `self.state.take()`; a derived one doesn't. `State::idle` reads its own `until`, or `self.state.take().idle(..)` would read `None`.
- **Finding 5 keeps `Hint { value, offer }`.** All six pairs are reachable: a `Done` with no hint leaves `Ready` with no value, and a `204` over a refused hint can leave `Refused` with none. One enum would list the same pairs, so finding 5's "only if" fails. `pump`'s guard goes, since its one caller asks the same and `stream` feeds nothing once closing.
- **Finding 6's caveat is recorded, not redacted.** `subject`, `metadata`, `granted_scopes`, `issued_client_id` and `Redirect.error` print as they came. Redacting them would add hand-written `Debug` back, the thing finding 6 removes. `skein/docs/design/oauth.md`, 4 says so, in this plan's commit, and `lib.status.md`'s **Departs** on 6 holds lib.md's wider sentence against it. `oauth::Event::Redirected`'s `uri` and `oauth::Redirect`'s stay: an address without its query, no credential.
- **Finding 7 loses one clause.** `record.rs`'s `secret` helper has nothing to share now: its fields are `Secret`. Its `component/github.rs` line goes to `connection-deadline`, which holds that file. `expose_optional` goes through an exported `secret_ref`.

## Overlaps

- The spot review's tasks meet no other draft's files. `temper-fake` and `temper-service` are other drafts' too (`ui-away`, `ui-chat-draft`), in other files: `oauth-secret-fields` changes `temper-fake`'s `oauth` tests and `temper-service`'s `clients/google.rs` and its `sign_in` and `accounts_host` tests.
- Inside this plan: `connect-close`, `connection-deadline` and `connection-states` hold `connection.rs` in order. `hook-slack` holds `hooks/connection.rs` alone, beside them. `oauth-secret-fields` and `oauth-noise` hold `skein-fake-oauth/src/lib.rs` in order.

## Critical path

So far: `connect-close`, `connection-deadline`, `connection-states`, 90 minutes.
