# ctl-words: the plan

From `inputs/request.md`: what `temper ctl` tells a person, five small things. Everything below was judged from reading the code on 2026-11-06. A cut builds nothing, so no test ran: each task reproduces first and says if its cause is wrong.

## Parts and tasks

| Request's item | Part | Task | Needs | Minutes |
|---|---|---|---|---|
| 1, a refused key | `key-words` | `key-words` | | 25 |
| 2, a task waiting for an account | `account-story` | `account-story` | | 40 |
| 2, what `move` does then | `move-own` | `move-own` (jig) | | 25 |
| 3, `forge setup`'s last line | `setup-words` | `setup-words` | | 25 |
| 4, out of tries | `hold-look` | `hold-look` | | 20 |
| 5, an effort not offered | `effort-refusal` | `effort-refusal` | | 60 |

Six tasks, under the ceiling of eight, and **nothing is dropped.** One question is open, `hold-failure-text`, and no task waits for it.

**Critical path:** none of them needs another, so the longest is `effort-refusal`, about 60 implementer-minutes.

## What was decided

- **1. Only a bad name changes.** The new `Usage::KeyName` covers a name outside the rule and `--key` with no name. A missing required key, a key on a tool that takes none, and `--key` twice keep the words they have: the request doesn't name them.
- **2. The request's reading of `move` looks wrong.** `move` refuses `not_waiting` when the party is already the task's requester, whatever the task waits on (`jig-domain-tasks`' `moving::state`, turned into `NotWaiting` by `routing/refusals.rs`). `jig-domain-core/src/tests/acts.rs` pins that. The walker's task was presumably their own.
  - **What `move` should do then:** the same as on any task. It's offered on a task another party requests, and moving gives no task a run. On your own it's now `not_offered`, which says what's true. That's a small jig task, with no format change.
  - **One story, from one check** (`llm::waits_for_account`): the header says `Waiting for an account`, the plain line stays, and `--json`'s `waits_on` is `"account"`. That's a new value in what `waits_on` can hold, and this cut found no reader it breaks.
  - **What stays:** `chip.phase` stays `due` and `conditions.rs` is untouched, as the request asks, so `--until phase:due` still holds. `task list`'s lines carry no next-run wait, so they keep `nothing`. A held task's JSON keeps `nothing`, and its `hold` member says why.
  - **Left for later:** a task pinned to an account with no source keeps `Nothing awaited`, as `tools.md`, 14 has it, until QA meets one.
- **3. The line is true on an owned forge, and it goes from `forge setup` anyway.** The bot is a write collaborator there as anywhere, and Forgejo shows protection to admins alone. So the request's first branch, that it isn't true, doesn't hold.
  - **Why temper doesn't do what it implies:** granting the bot admin would let whatever holds its tokens change or remove the branch protection that is temper's second net. That's a security decision, and too large for a words fix.
  - **What this does:** a person can't act on the line on a repository temper has just made, so `forge setup`'s ending drops it. `--json` keeps `protection`, and `repository adopt` keeps the line.
  - **What's left:** `tools.md`, 14 records that a rule set later on an owned forge stays unread, and that reading it with the site admin's token would widen nothing. **Worth your look:** the request's either/or didn't quite fit, so this is the call in the plan most likely to be overruled.
- **4. A question, and the words now.** `plan.toml`'s `hold-failure-text` carries the cost: what it stores, its bound and who may read it. The recommendation is no. `hold-look` stores nothing and holds on either answer, so nothing waits. On a yes, the text is a plan of its own.
- **5. The engine's silence belongs to `ctl-waits/settings-answer`,** which is in flight on exactly this, at the clients' door. It isn't duplicated here.
  - **Not reproduced by this cut:** from reading, the door already refuses a template's lacking effort (`translate/llm.rs`, `refused`), so the silence isn't explained, and `effort-refusal` reports what its base does. A cut can't run it, so the request's "reproduce first" falls to that task.
  - **Why the item isn't dropped:** even a working door says only `This model cannot be chosen: M`, and the wire's `Refusal::Llm` has no effort in it. Adding one is a protocol change. So ctl checks the item against `me` before sending, as `--effort` already does.
  - **The limit:** a model `me` doesn't show still goes to the engine and gets the words that name the model alone.

## Overlaps not avoided

- `key-words`, `account-story`, `setup-words`, `hold-look` and `effort-refusal` all change `temper-client-tools`, in different files. Their tests share `tests/jig/task_words.rs` (`account-story`, `hold-look`) beside different stories, and each deletes its own bullet of `tools.status.md`, placed apart.
- `ctl-waits/kept-answer` glob-owns `temper-client-tools/src/tests/**`, and `ctl-waits/deadline-words` changes another line of `temper/docs/help/ctl.md`, as `key-words` does.
- `effort-refusal` changes `runner/acts.rs` and `runner/acts/choice.rs`, which `model-catalogue-1/ctl-offer` changes on its trunk, and may meet `me`'s offer when that lands.
- `account-story` and `hold-look` change `temper/docs/help/task.md` and `hold.md`, inside `model-check/trial-close`'s `temper/docs/help/**` glob.
- `move-own` changes `jig-domain-core/src/admission/roles.rs`, and `tests/acts.rs` beside `model-check/person-task` and `ui-proposal-card/proposal-facts`, whose globs cover `jig-domain-core/src/tests/**`.
