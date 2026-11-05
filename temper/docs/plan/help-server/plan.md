# help-server: `temper help server`'s walk works as written

Four tasks, from ops's four findings at main `fadf954`. Each task confirms its finding on the trunk first.

## Parts and tasks

| Part | Task | Needs | What |
|---|---|---|---|
| `forge-sign-in` | `kept-metas` | | finding 1: an entry refuses no start; the page's metas come from the kept forges; the refusal says what is wrong |
| `owned-by-hand` | `no-group-file` | | finding 2: `fetch`, `prepare` and `run` need no group's file outside a box |
| `server-page` | `server-page` | | findings 3 and 4's page: the connect, `PrivateTmp=yes`, where a run's failure is read, and what the page had wrong |
| `walk-story` | `walk-story` | `kept-metas`, `no-group-file` | the process tier's Forgejo story runs the walk's commands, the meeting point |

Critical path: `kept-metas`, then `walk-story`. The three first tasks start at once.

## What was decided

- **Finding 1 is built, not patched.** `temper/docs/design/protocol/sign-in.status.md` already recorded the gap (9.7, **Not yet**). Stopping the refusal alone would leave no meta at all, and so no "Open Forgejo ↗". The start makes no forge's meta; `temper-service` decides each entry once the records load and when a forge is made or removed. ops's belief about the cause holds, from reading; `kept-metas`'s first test proves it.
- **A browser origin apart from `git`.** On the walk the forge is a managed record, reached on loopback, so its `git` can't match a `redirect_uri` at its public name. A Forgejo's `web`, which `forge connect` already takes and the record already keeps, now gives its browser origin (`sign-in.md`, 9.1 and 9.7). No store or wire change. Pull links still use `git`, so a Forgejo connected this way links pulls at its loopback address; that is not in the request, and is named here for a later draft.
- **Finding 2: the commands need no file.** A forge with none takes the machine's `owned` part (`setup.md`, 7.4, changed, with its reason), so neither the walk nor a redeploy needs an extra step. `temper forge remove` stays strict: it deletes a directory by its kind. A command that writes the file was rejected: a redeploy removes it, so the walk would need it again each time.
- **Finding 3: the page says so.** Making the engine connect a hand-run Forgejo at its start would change what `temper serve` is (`setup.md`, 6.5 and 7.4). The connect ops ran is already sound (`setup.md`, 5.5), and `TakeOver` never runs, so `admin_token` stays for the next redeploy.
- **Finding 4's hold needs a stored-format change,** so it is a question (`hold-last-failure` in `plan.toml`), recommending that the page says where the reason is, which `server-page` does. No task waits for it.
- **Beyond the findings, the page's other errors** go too, as the request says the design wins: `fetch` does print its progress (`setup.md`, 6.6), and the "forge not prepared" line and the handover at the start cannot be met on this walk. `sign_in.forges` itself gets a short paragraph, since the page never said how to set up what finding 1 is about.

## Overlaps

No task touches fabrig/ui's files. Unavoidable, each a line or two: `temper-service/src/tests.rs`'s `mod` list, beside other drafts' test modules; `temper-service/src/family.rs` only if it proves to drop `web`, where `ui-welcome/setup-intent` and others have merged. Between this plan's own tasks, `kept-metas` and `walk-story` touch `crates/temper/tests/**` in different files.

## Design written

`temper/docs/design/protocol/sign-in.md`, 7, 9.1, 9.7, 9.8; `setup.md`, 6.8, 7.1, 7.4; `forge.md`, 2.1; `README.md`, 4; `../domain/testing.md`, 2.6; with their status files.
