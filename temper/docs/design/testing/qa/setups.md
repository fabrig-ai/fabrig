# The setups' catalogue

Provisional, 2026-11-12, from `../../draft/landed-qa.md`, sections 5
and 6, milestones Q3 and Q4 of `../../draft/landed-qa.execution.md`.
The recipes kept in `temper/docs/development/qa/setups/`, what each
reaches and where its scripts come from. Their format and walk are
`../../client/recipe.md`; how a pass restores, makes again and adds to
them is `places.md`.

## 1. In one page

- **The first recipes** (section 2), from the journeys' setups: the
  welcome, `project-landing` and `chat-draft`, on the box's generated
  scripts.
- **The TUI's** (section 3): the TUI's journeys start from
  `project-landing`, which serves their starting screens; `tui-chat`
  adds a chat asked and answered from the TUI's launchpad.
- **The twins** (section 4): B0's walk replayed as setups, its
  scripts its twin's with takes, so that its numbers and heads are the
  box's own: `widget`, `b0-plan` and `b0-landed`. B1's waits on how its
  gate's commit is made (section 4.4).
- **Grown by passes** (`places.md`, 6): a recipe a pass adds lands with
  its request; this document names the catalogue's roots, not every
  recipe.

## 2. The first recipes

Milestone Q3's, each with no `scripts.json`, so the box's generated
scripts answer, strict (`../../protocol/setup/box.md`, 5):

| Recipe | After | Reaches |
|---|---|---|
| `welcome` | none | a fresh box's welcome, nothing connected |
| `project-landing` | `welcome` | Hello, with `work/hello` on the box's own Forgejo and the fake ChatGPT plan signed in, as the web journey's setup reaches it (`../../client/web/journeys.md`, 8.2) |
| `chat-draft` | `project-landing` | Hello's first chat with its first Away mode draft written |

## 3. The TUI's

- **Their starting screens** are `project-landing`'s, as the TUI's
  journeys start from it (`../../client/tui/journeys.md`, 9): a TUI
  case's lines start the TUI at its first `tui` line, signed in by the
  drive session (`../../client/drive.md`, 3.3), so a case on the
  launchpad starts from `project-landing` with no recipe of its own.
- **`tui-chat`,** after `project-landing`, with no `scripts.json`: the
  steps of the `launchpad-chat` journey's walk past its approval
  (`../../client/tui/journeys/launchpad-chat/walk`), as drive lines, from
  `web reload "/projects/1"`, as the journey's setup ends, to the fake
  model's answer, a `tui see` of it that waits, then `web reload "/"`,
  a step that is no `see`, so that the answer's `see` stays a step
  (`../../client/recipe.md`, 3.1); its check `tui see "/chats 1"`, the
  chat's row on the launchpad of the TUI the check starts afresh after
  the walk's `tui stop` (`../../client/recipe.md`, 5.1;
  `../../client/tui/pages/launchpad.md`, 4). It reaches Hello
  with an Ask temper chat asked and answered from the TUI, and serves
  the TUI's chat screen (`../../client/tui/pages/chat.md`) and the web
  client's view of a chat the TUI started.
- **A TUI state** a later case needs is a pass's to add, as any other
  (`places.md`, 6): its lines are `tui` lines, and its check `tui see`
  or `web see` lines, read on the root page and a TUI started afresh
  (`../../client/recipe.md`, 5.1).

## 4. The twins

### 4.1 What a twin setup is

- **A real run replayed** (`../../draft/landed-qa.md`, 6): the run's
  agents say what the real model said, from its twin's scripts
  (`../real-loop.md`, the ladder's twins), and the person's acts are
  its recipes' lines, in the walk's words, through ctl as the walk made
  them (`../../../development/acceptance.md`, 6 and 7). Its parent is
  the state the walk started from.
- **On the box's own Forgejo,** real, with its runner: the walk's
  backends, ChatGPT with an owned Forgejo
  (`../../../development/acceptance.md`, 6), so the repository, its
  CI and its pull requests are the box's.
- **Strict:** every turn the box asks for is the twin's or fails
  (`../../protocol/setup/box.md`, 5), so a twin that no longer covers a
  run blocks the cases on it, naming its scripts, and is never a
  finding against temper (`../../draft/landed-qa.md`, 6).

### 4.2 What differs on the box

- **Numbers and ids the walk's box gave and this one does not:** the
  tasks' numbers, since the parent's state holds tasks of its own; the
  pull request's, numbered by the box's Forgejo; and every head and
  merge, made by real git. Each is a take (skein's
  `testing/fakes/llm.md`, 2.6): the twin's calls and texts name what
  the box's conversation holds, from a tool's output or the agent's
  brief, rather than what the walk's held.
- **Made by `temper-twin`,** never by hand: its `--takes FILE`
  (`../fakes.md`, 1) writes each take into the turns that use it, from
  a `takes.json` kept beside the twin's fixture, and the fixture's
  `regenerate.sh` passes it, so a twin made again keeps its takes.
- **Proven by the twin's fuzzy story:** the fixture, with its takes,
  is what `suite::fuzzy::green_light` replays (`../real-loop.md`, the
  ladder's twins), so each take is found in the conversation smith
  writes and the story reaches its end as before. Whether the takes
  find the box's values is the walk after landing's
  (`../../draft/landed-qa.execution.md`, 1).
- **One source:** a twin setup's `scripts.json` is its fixture's
  `cued` and `asked`, member for member, with no `otherwise`, so that
  under strict a call no cue selects fails; a focused test holds the
  two equal.

### 4.3 B0's

From `temper/tests/real-loop/tests/fixtures/b0/`, the rung's walk
(`../../../development/acceptance.md`, 6):

| Recipe | After | Its lines | Reaches |
|---|---|---|---|
| `widget` | `project-landing` | `ctl forge setup` with `--playground` on the box's Forgejo, forge 1, waited until adopted; its check a `web see` of widget's project | the project widget, `playground/widget` its home with the starter CI, beside Hello: the repository the rungs start from (`../../../development/acceptance.md`, 5) |
| `b0-plan` | `widget` | the person's request, `person.json`'s `request`, in a chat of widget's project through ctl, waited until the work's question is in the inbox; its check a `web see` of the question's words | the work's plan asked and waiting for its go-ahead, nothing pushed |
| `b0-landed` | `b0-plan` | the person's answer, `person.json`'s `answer`, through ctl's inbox, waited until the chat's last word; its check a `web see` of `person.json`'s `last_word` | the change landed on `main` with CI green and an agent's review passed, the chat saying so |

- **`b0-plan`'s `scripts.json`** is B0's twin's; `b0-landed` has none
  of its own, since its chain gives its parent's.
- **Their pages:** the chat, Needs you and its question, the goal and
  its change, Work and the project's Home at each state; a case on a
  later page of the same goal starts from the nearest.

### 4.4 B1's

B1's walk (`../../../development/acceptance.md`, 7) starts from
`widget` with its person's gate committed to `main` through the
Forgejo's write API, which no drive line makes
(`../../client/drive.md`, 3). How that commit is made is open
(`../../draft/landed-qa.status.md`); until it is settled B1's twin is
no setup.

## 5. Their tests

Focused, with no box: every recipe of the catalogue reads, its chain
and hash, as `recipe list` gives them (`../../client/recipe.md`, 7); a
twin setup's `scripts.json` reads in the box's `--scripts` form and
equals its fixture's `cued` and `asked`. The walks are the walk's after
landing (`../../draft/landed-qa.execution.md`, 1).

## 6. Open questions

- **A twin's later states:** B0's plan and landing are its setups. A
  state between, CI running or a review under way, is a stopped box mid-run.
  Default: none until a case needs one, then the pass's own (`places.md`, 6).
