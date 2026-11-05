# debt-23: the twenty-third debt round

Two independent parts (`inputs/request.md`): the spot review of debt-21, and the items skein D023, jig D016 and hearth D006. No temper client view, no `temper.css`, no `jig-client-view`. This file grows with each push.

## Parts and tasks

| Source | Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---|---:|---|
| review 1 | `spot-review` | `connect-close` | | medium | 15 | `connect` keeps a close; the pin fails first |
| review 8 | `spot-review` | `settings-name` | | medium | 30 | the value-mismatch abort; `Name`; closing gates before it clones |

The rest of the part and the three items follow in later pushes.

## What was decided

- **Finding 1 is right.** `Link::connected` answers `Start::Closing` only once `Link::close` has run, which only `Connection::close` and `abort` call. `ended` survives the notice now as well as `closing`; the spec makes a test that sees it a question.
- **Finding 8's abort is the value mismatch only.** `drop_project` and `restore` skipping a setting that isn't an application's isn't a value mismatch, and the request brings back only that. `core.status.md` reads "No gap" and no status under `jig/docs/design/domain/` recorded otherwise, so none changes. The abort guards `check`'s promise, which `D017` says stays, so that item doesn't change either.

## Overlaps

- `settings-name` and `connect-close` meet no other draft's files.

## Critical path

So far: no task waits on another.
