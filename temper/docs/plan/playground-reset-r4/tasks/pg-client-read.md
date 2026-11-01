+++
id = "pg-client-read"
part = "playground"
kind = "feature"
needs = ["pg-wire", "pg-project-page"]
files = [
  "temper/crates/temper-client-domain/src/forge/demo.rs",
  "temper/crates/temper-client-domain/src/forge/family.rs",
  "temper/crates/temper-client-domain/src/forge/vocabulary.rs",
  "temper/crates/temper-client-domain/src/forge.rs",
  "temper/crates/temper-client-domain/src/domain.rs",
  "temper/crates/temper-client-domain/src/route.rs",
  "temper/crates/temper-client-domain/src/forge/listed.rs",
  "temper/crates/temper-client-domain/src/tests/demo.rs",
  "temper/crates/temper-client-domain/src/tests/playground.rs",
  "temper/crates/temper-client-view/src/tests/fixtures.rs",
  "temper/crates/temper-client-view/src/tests/demo.rs",
  "temper/tests/client/src/front.rs",
  "temper/tests/client/src/front/account_acts.rs",
  "temper/tests/client/tests/suite/demo_start.rs",
]
effort = "high"
minutes = 70
+++

## Goal

temper's client reads the playground from its new read and runs it as
ordinary records (draft `temper/docs/design/draft/playground-reset.md`,
6.1 and 6.3, "What goes"): the forge's side reads `Read::playground`
(`pg-wire`) where it read `Read::demo`, starts it with
`StartPlayground` where it sent `StartDemo`, and drops the demo's two
halves, its end observed on another forge's connect or a real account,
its retirement, and the client's removal of the scripted account. While
the playground runs, it holds whole: nothing but its reset ends it
(6.1: "A real account alone does not end it").

Acceptance:

- the forge's side reads `Read::playground` at the moments it read the
  demo (client/README.md, 4.8, "Read once a link is live": link live,
  live again, a start answered), and keeps the decoded answer;
  `StartDemo` and `Read::demo` are sent by no client code (domain,
  view, tools, shells);
- `forge::Demo`'s `forge` and `project` are `Option<u16>` and
  `Option<u32>`, and `DemoState` is `NotStarted | Running | Resetting`
  (`Ended` goes);
- `Domain::demo()` is `Option<&forge::Demo>`: some while the playground
  runs (`demo_running`, as today: answered running, or its start out
  or failed) or resets; `forge::demo::Halves`, `halves`, `current`,
  `RetiredDemo`, `retired_demo`, the deployment watch, the end
  observation and the account removal are gone, with their readers;
- a connected real forge, or a usable account of another provider,
  ends nothing, and no `AccountChange::Remove` is sent;
- "Try the playground" sends `StartPlayground` and opens Chats; once
  the read names the playground's project, the session that started it
  selects it, as today's start selects `demo.project`;
- the tests below pass, and every existing test of the packages
  changed, its expectations updated where they pinned what goes
  (listed in the report).

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 5).
- `temper/docs/design/draft/playground-reset.md`, 6.1 to 6.3.
- `temper/docs/design/client/README.md`, 4.8 (the demo's read and its
  halves, which this task reduces to the read and the start) and 3.5
  (`Welcome`, the setup strip).
- `temper/docs/plan/playground-reset-r4/tasks/pg-wire.md` (the read's
  records) and `tasks/forge-asks.md` (`Request::StartPlayground`).
- `temper/docs/plan/playground-reset-r4/notes/surveys/s2-playground.md`,
  section 4 (the client's demo modules and their readers).

## Owns and may change

Owned: the `files` above (`tests/playground.rs` is new).

Neighbour files, edits allowed, each only for the changed items
(`Demo`'s two options, `DemoState`'s variants, `Domain::demo()`'s
type, the halves and retirement gone):

- `temper-client-domain`: `pages.rs` (`retired_project` and
  `demo_model` go: `project_changed` drops the retired project's test;
  `demo_model`'s one reader takes "the playground runs"),
  `pages/welcome.rs`, `pages/connect_model.rs`, `forge/admin.rs`,
  `forge/connect.rs`, and the domain's other test files (fixtures that
  build a `w::DemoAnswer` build a `w::PlaygroundAnswer`; tests that
  pinned a half's end, a retirement or the removal are deleted, each
  named in the report).
- `temper-client-view`: every reader of `root.demo()`, `Demo.forge`,
  `Demo.project`, `retired_demo` and `DemoState` (`frame.rs`,
  `choice.rs`, `root.rs`, `forge/connect.rs`, `forge/forges.rs`,
  `pages/chats.rs`, `pages/chat.rs`, `pages/inbox.rs`, `pages/task.rs`,
  `pages/accounts.rs`, `pages/accounts/rows.rs`, `pages/connect_model.rs`,
  `pages/welcome.rs`, `pages.rs`, `accounts/words.rs`), edited so they
  draw what they drew with both halves holding: the banner keeps its
  `DemoBoth` words and both links until `pg-playground-banner`; and the
  view's other test files, as for the domain's.
- `temper-client-tools` (`forges.rs`, `demo.rs`, `src/tests/forges.rs`):
  the CLI's `playground start` sends `StartPlayground` through
  `forge::demo::ask`; its tests expect `w::Request::StartPlayground`;
  `demo.rs` prints `StartFailure::Project` as "the playground's project
  was refused". No other change to the CLI: its help and words stay.
- `temper-client-shell-terminal/src/tests/**`, `temper/tests/client/`
  (`src/translate.rs`, `src/translate/forge.rs`, its other suite and
  qa files), `temper/tools/gallery/`, `temper/tests/one-process/`: the
  fixtures that answer the demo's read answer `Read::playground` with a
  `PlaygroundAnswer`; expectations that pinned what goes are deleted or
  updated (each named in the report); a golden the banner's unchanged
  words keep stays as it is.

Pre-approved:

- the view fixture `demo_read_answer(root, read, project, state:
  w::DemoState)` becomes `playground_read_answer(root, read, project:
  Option<u32>, state: w::PlaygroundState)`, its callers updated; keep
  `forge: Some(wire::FORGE)` as the fixture's forge;
- the `Joint` (`temper/tests/client/src/front.rs`) keeps its scripted
  start (`demo_starts`, `demo_outcomes`) for `StartPlayground`, flipping
  its answer to `running` with forge and project; `demo_removals` goes;
- if a `Read::playground` answer arrives `running` with no forge,
  treat it as running with none (no forge row is the playground's);
  default: as said.

## Steps

1. **Decoder** (`forge/family.rs`): `pub fn playground(record: &[u8],
   limits: &w::Limits) -> Option<forge::Demo>` from
   `wire::decode_playground_answer`, replacing `demo`; `not_started` →
   `NotStarted`, `running` → `Running`, `resetting` → `Resetting`.
2. **Vocabulary** (`forge/vocabulary.rs`): `Demo { state, project:
   Option<u32>, forge: Option<u16>, provider, owner, name, cues }`, its
   doc citing draft 6.1.
3. **The side** (`forge/demo.rs`): `State` keeps `answer`, `pending`,
   `live`, `answered`, `start`, `failure`; the rest goes. `read` sends
   `w::Read::Playground`; `told` decodes through `family::playground`;
   `ask` builds `w::Request::StartPlayground`; `start` sends it while
   the answer is `NotStarted`, or `Running` with a failure (Try again),
   and never while `Resetting`. `progress` goes (and
   `route::demo_progress` with it, or it keeps only the read). The
   module doc says: the playground's read and start (draft 6.1); its
   reset is `pg-playground-banner`'s.
   `Side` (`forge.rs`): `demo_running` true for `Running`, or a start
   out or failed; a new `pub fn demo_resetting(&self) -> bool`;
   `serves_pages` compares `Some(forge)`; `demo_provider` the answer's
   provider.
4. **Root** (`domain.rs`, `route.rs`, `forge/listed.rs`):
   `Domain::demo(&self) -> Option<&forge::Demo>` while running or
   resetting; `route::playground` sends the start and opens Chats, and
   selects the project when the answer first names it (keep the
   started session's intent on `Side`, e.g. `select: bool`);
   `demo_facts` keeps the provider; `listed::playground(root) ->
   Option<(&Demo, &Row)>` finds the row by `demo.forge == Some(number)`;
   `listed::personal` excludes the playground's forge by the same test.
5. **Readers**: every neighbour above, mechanically.
6. **Tests** (the client domain's unit tests; the client world):
   - `temper-client-domain/src/tests/playground.rs` (new, with a `mod
     playground;` line in `src/tests.rs`):
     `the_read_follows_the_playgrounds_state` (answers `not_started`,
     `running` with forge 4 and project 7, `resetting`: `demo()` none,
     some running, some resetting; forge and project as answered);
     `try_the_playground_sends_start_playground_at_the_deployments_scope`
     (the `Send`'s ask decodes to `w::Request::StartPlayground`, scope
     `Deployment`; no second start while it is out);
     `a_connected_forge_and_a_usable_account_end_nothing` (a running
     answer, then a connected Forgejo row and a usable OpenAI account
     delivered: `demo()` still some, no `AccountChange::Remove` and no
     second read sent);
     `the_starting_session_selects_the_playgrounds_project_once_named`
     (start, answer `running` with project none: no project selected;
     then project 7: `pages().project() == Some(7)`).
   - `temper-client-domain/src/tests/demo.rs`: keep what still holds
     (the read's moments, a start's failure and Try again), delete the
     halves, retirement and removal cases.
   - `temper/tests/client/tests/suite/demo_start.rs`
     (`temper-client-world`): its cases on the playground answer; the
     ended-demo and halves cases deleted.
   - Narrow runs per file: `heavy cargo nextest run -p
     temper-client-domain --lib -E 'test(playground::) | test(demo::)'`,
     `-p temper-client-view --lib`, `-p temper-client-world --test suite
     -E 'test(demo_start::) | test(welcome::)'`.
7. Checks, once: `cargo fmt`; `heavy cargo clippy -p
   temper-client-domain -p temper-client-view -p temper-client-tools -p
   temper-client-world --all-targets -- -D warnings` (and any other
   package changed); focused tests of `temper-client-domain`,
   `temper-client-view`, `temper-client-tools`,
   `temper-client-shell-terminal`, `temper-client-world`; `heavy cargo
   check --tests` for `temper-client-shell-cli`,
   `temper-client-shell-web`, `temper-client-gallery`,
   `temper-one-process-world`, `temper`, one at a time. This changes
   what `Domain::demo()` means, so every reader package's tests run
   (listed). Commit, push.

## Report

fab's sections; `## Interfaces`: `forge::Demo`, `DemoState`,
`Domain::demo`, `Side::demo_resetting`, `family::playground`, the view
fixture's new name and signature; `## Left`-style list of every test
deleted or changed for what goes, by file and name.
