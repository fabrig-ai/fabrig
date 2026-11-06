# configuring-words: plan

The configuring chat speaks the person's terms and never sends them to a terminal. The request is `inputs/request.md`.

## What is words, and what is more

| The user's want | What it takes | Beyond the stock's words? |
|---|---|---|
| 1. Never a terminal | words; the refusal's words | **Refusal text:** `approval_words` named `repository adopt`, `project show` and `temper help configure` |
| 2. The person's terms | words | no |
| 3. Approvals start from the person | words, **a wider `settings` inventory, a wider `read_permission`** | **yes: no new tool, no new brief section, nothing in jig; one skein type and one stored record** |
| 4. Ways and questions read as settings | **a fix in the run's read**, the tool's words, words | **yes** |
| 5. "Nothing has changed" once | words | no |

No new tool. No brief section: the inventory is what the instructions read first, and a brief section would spend every chat's room for one charter's need (`temper/docs/design/domain/agent.md`, section 10 keeps it open).

## Why ways and questions said `unknown`

A real fault of the read, not a wrong key. The agent's key was right: `{"kind":"project-ways","project":1}`. The stock holds no row for a project's kinds 6 and 7, and their value with none stored is a constant that "who reads it takes" (`temper/docs/design/protocol/configuration.md`, 13.4). The root and the client took it, and the run's `settings` read did not, so jig's core answered `Unknown` and the tool printed it. `settings-reads` makes the read take the stock. What's left of `unknown` (a project that isn't one, an older revision) now says it wasn't read and says nothing about the settings, and the words tell the agent never to report it as a setting.

## What is newly readable, and by whom

Only a run whose authority holds the settings family, in the stock charter 3's alone, and of its own project alone, in the inventory, as the repositories already are:

- **Each person of its project:** their number, login, role number and forge user id. The first three are what that project's `project show` already prints. **The forge user id is new.** Marks: the person this chat is with, and temper's own forge writer. No email, no display name, no services.
- **An owned forge's person:** the login from `application.machine.owned.person`. **New to any run.** It's how a box's chat can say "you (ada on the forge)" when the deployment login differs from the forge person.
- **`read_permission`'s login:** **new to every run with the forge reads,** in the stock charters 1, 2, 3, 4 and 6, for a user id it names. This is the one widening not held to the settings family: it's the forge's own answer about a user of a repository that run may already read. If you want it held to the settings family, that's a condition in `temper-engine-domain`'s render.

## The skein task

**`permission-login` goes beyond the request's "in temper".** The user asked that `read_permission` name the login. temper can't do that alone: a person seeded by adoption has an empty login in jig's people records, and skein's answer drops the login both dialects read. So `api::Answer::Permission` carries a `UserPermission`, and temper's stored `PermissionValue` gains its fields in place ("Formats change in place before v0.1.0"). I read this as a wider answer rather than a protocol change, since the forge's wire and the client's are untouched, so it's no `Question:`. It's the first thing to look at if you disagree: without it, `read_permission` names a login only for someone who has signed in, and the stored record and skein stay as they are.

Left as they are, and in `agent.md`, section 10: `read_collaborators` lists ids, and `read_permission` names each.

## What the agent still can't do

On a fresh box the person is often no collaborator, so adoption lists only temper. Their forge person is then known by login with no id, and acceptance refuses a person's approval. That's what the request asks for: it proposes, says what acceptance checks, and on a refusal says who must be added and the repository adopted again. Its refusal no longer names a command.

**Not confirmed:** the browser has an adoption form (`Adopt { project }`, `temper/docs/design/client/README.md`, 4.4), but I couldn't confirm from the design that it adopts a repository already adopted. If it can't, "adopt the repository again" has no browser path on such a box. That's for the walk to see, and it's in the `ui-*` plans' files, so this plan leaves it.

## Tasks

| Task | Part | Needs | Owns |
|---|---|---|---|
| `stock-words` | words | | `configuring.txt`, its test |
| `settings-reads` | reads | | the run's settings read, `unknown`'s words, the refusal's words |
| `permission-login` | login | | skein's answer, the stored record, the tool's text |
| `project-people` | people | `settings-reads` | the inventory's two blocks |

Three go out at once. **Critical path:** `settings-reads` (70 minutes), then `project-people` (80 minutes), 150 implementer-minutes.

`permission-login` and the two `reads` tasks share `temper-engine-smith`, in different files (`answers.rs` and `calls.rs` against `settings.rs`), so they run side by side.

## Words and byte figures

`configuring.txt` goes from 4,807 to 6,009 bytes, up 1,202 (`notes/configuring.txt`). `stock-words` holds the figure in `the_configuring_agent_is_charter_3_and_template_3`, and the measured rows in `stock_brief.rs`. `permission-login`'s two descriptions add 68 bytes to every charter with the forge reads (1, 2, 3, 4, 6), and the words 1,202 to charter 3: its non-brief goes to 42,103 and its room to 358,032, from 40,833 and 359,302; charter 1's to 42,114 and 358,021, from 42,046 and 358,089. `brief_bytes` stays 356,270, and the 68 come out of `stock_brief.rs`'s margin, 279 from 347. The ceiling is 6,020 bytes, where charter 3's non-brief reaches charter 1's.

How a model takes the words is unprovable below a walk. What's tested is mechanical: the phrases present and absent, each tool's answer, the refusal naming no command.

## Walks and rungs

None moved. Checked against `temper/docs/design/draft/configuring.execution.md`, `temper/docs/design/draft/v0.1.0.md` and `temper/docs/design/domain/testing.md`, 6.4 and 6.5:

- **No story reads the old words.** The engine's world reads `configuring.txt` by `include_bytes!`. `Never promise` and `establish eligible people` appear in no test or walk.
- **Stage 1's pass plays the person through `temper ctl` itself,** and never needs the agent to point at it. The sentences its judge and golden read stay: the workflow described when asked, the proposal number and its wait for an owner, what settings can't express.
- **The one test that pinned old refusal words** is `approval_identity.rs`, which `settings-reads` moves.

`testing.md`, 6.4 gains a row for the agent's reads. `configuring-walks` is in the queue and its plan isn't on this branch, so I couldn't check it; it should read `agent.md`, 8.1 as it now stands.

## Overlaps

- **`fabrig/ui`:** none. No `temper-client-*` or `temper-fake` file is touched.
- **`box-link-first`:** none in these files. `temper-service/src/system.rs`, which `project-people` touches, isn't among its files, though `startup.rs` and `settings.rs` beside it are.
- **`debt-25`:** `permission-login` touches `skein-forge-connection/src/component/github_calls.rs`, beside its `connection.rs`, not the same file. `temper/tests/conformance/*` isn't touched.
- **`ui-rename-focus`:** it edits `skein/docs/design/STATUS.md`; this plan doesn't.

## Walk

`walk = false`: a box walk of the configuring chat by fabrig/qa follows the landing.
