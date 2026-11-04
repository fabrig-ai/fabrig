+++
id = "ready-adopts"
part = "ready-adopts"
kind = "feature"
needs = ["ready-name", "project-addresses"]
files = [
  "temper/crates/temper-client-domain/src/pages/welcome.rs",
  "temper/crates/temper-client-domain/src/tests/pages/welcome.rs",
  "temper/crates/temper-client-domain/src/tests/pages/ready_adoption.rs",
  "temper/crates/temper-client-view/src/pages/welcome.rs",
  "temper/crates/temper-client-view/src/tests/welcome.rs",
  "temper/crates/temper-client-view/src/tests/model_forge.rs",
  "temper/tests/client/tests/suite/welcome.rs",
  "temper/tests/client/tests/suite/adoption.rs",
  "temper/crates/temper/tests/fake_backends/walk/open_default.rs",
]
effort = "high"
minutes = 120
+++

# Ready adopts the first repository and opens Home

## Goal

Once Ready's "Create your project" has made the project, it adopts the forge's first repository
into it, as `temper ctl forge setup --project` does, opens the project's `Home`, and waits for
nothing. Ready's lead names the repository before the press. Refused or unanswered, the project
stays made and the state is kept for Home to offer.

Acceptance:

- With a first repository, one press sends `MakeProject`, then, on `ProjectMade`, one `Adopt` of
  that repository, in that project's scope, under `first_repository_key(forge, false)`, with
  ctl's fields; never before the project is made, never twice; with none, no `Adopt`.
- The page shows `Home { project }` at `ProjectMade`, before any adoption's answer.
- `pages.adopting()` says it out; `Adopted` ends it with no `ended`; a refusal, or no answer, sets
  `ended`, and the project stays.
- The name taken opens the person's project of that name, its name compared ASCII-case-folded, as
  jig's `same_name` folds, and adopts into it; a refusal there (`HomeHeld`, `AnotherProject`) is
  an ordinary refusal.
- `pages::Act::AdoptFirst` sends the adoption again: one at a time, a new key after a refusal.
- `"+ New project"` from the switcher opens the project's `Home`.
- Ready's lead reads "Everything is connected. Create your first project, with work/hello in it,
  and start working.", and without a first repository as today; the Model step's forge line reads
  "ORG/NAME made".
- Every commit carries the request's trailer (`notes/common.md`).

## Inputs

- `temper/docs/plan/look-round-3/notes/common.md` and `notes/contracts.md` (The first repository,
  Ready's adoption, Numbers).
- `temper/docs/design/client/README.md`, 3.5, The welcome: *Ready*, *Made*, *The name taken*, *Its
  adoption*, and the Model step's forge line; 5.3, *A new project*.
- `temper/docs/design/protocol/setup.md`, 6.7.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-domain/src/pages.rs`: `make_default`, `default_told` (it takes
  `env`), `told`, `switcher_chosen`, the new field, its getter, `Act::AdoptFirst` and the acts'
  lists that name every act.
- `temper/crates/temper-client-domain/src/{acts.rs,forge.rs,pages/chat.rs}`: arms; in `forge.rs`
  `first_repository_again_key`, and `first_repository_key`'s doc comment, which says the page and
  ctl share it: Ready and ctl do.
- `temper/crates/temper-client-view/src/words.rs`: `WelcomeForgeDoneRepository`'s words; a new word
  for the lead, numbered as `notes/contracts.md` says, beside `WelcomeReadyLead`.
- `temper/crates/temper-client-view/src/frame.rs`: `decode`, for the new act, if the compiler
  asks.
- `temper/crates/temper-client-view/src/tests/frame.rs`, `tests/connect_forge.rs`, `tests/forges.rs`,
  `temper/tests/client/tests/suite/{connect_forge,first_repository,welcome_m3,welcome_m3_owned,
  welcome_edges,first_run_hooks}.rs`: assertions of a word this task changes, or of a landing on
  `Chats` after Ready or "+ New project", those alone.
- `temper/tools/gallery/src/tests.rs`, line 333: the lead.
- `temper/scripts/proof-live-github-managed.py`, 372, 374, 422, 706: only if it reads a word or a
  landing this task changes.
- `temper/docs/help/project.md`, line 33, and `temper/docs/help/forge.md`, line 47, departing from
  the implementer's text: the first says the welcome's Ready makes a project named for the first
  repository, or "My project"; the second that the client's welcome adopts it into your first
  project. These files are compiled into tests: run `temper-docs`'s.
- `temper/docs/design/client/README.status.md`: this task's part of the **Not yet** entry.

Options, each pre-approved:

- If `pages.welcome_first` is `None` at `ProjectMade` because rows moved, send nothing: Home offers.
- If `pages::Request`'s two exhaustive matches (`route.rs`, 142 and `suggest`, 342) force an arm for
  a new variant, avoid the variant: `Request::Client` carries the send.
- `FirstRun::Whole`'s redirect (`domain.rs`, 154) stays on `/chats`: leave it and its test.

## Steps

1. **`default_told`** gains `env`. After setting `default_project.project` from `ProjectMade`, or
   from the name-taken path, `adopt(pages, env, project, out)`, a new private function. The name's
   comparison folds ASCII case.
2. **`adopt`,** to `notes/contracts.md`: builds the `forge::Adoption`, `forge::ask(&adoption,
   &env.limits.forge)`, a new `Pages` key from `pages.serial`, pushes the send, sets
   `pages.adopting`. `told` reads its answer through `forge::answer(&env.limits, answered)` in a
   `default_told`-shaped function; `Told::Pending` with `Progress::Unknown` ends it `Unknown`, as
   `default_told` does.
3. **Landing:** `default_told`'s `Show` and `switcher_chosen`'s go to `Address::Home { project }`.
   `OpenProject` and `FirstRun::Whole` stay on `Chats`.
4. **`Act::AdoptFirst`:** on `Address::Home { project }`, with `pages.welcome_first` and no request
   out: `adopt`, the attempt moved on when `ended` is `Refused`.
5. **The view:** `ready` draws the lead with `root.pages().welcome_first()`. The word's `Name` hole
   takes `ORG/NAME`.
6. **Tests.** `tests/pages/ready_adoption.rs`, in `temper-client-domain`: the order of the two
   sends; none with no first repository; one `Adopt` across a double press and a replayed answer;
   `Adopted`; refused; unanswered; the name-taken path with "hello" against "Hello"; `AdoptFirst`'s
   key after a refusal, and its second attempt's; each new test bites. `temper/tests/client`,
   `suite/welcome.rs` 58, 59 and 62 and `suite/adoption.rs` 130 to 141: the engine adopts, and Home
   opens first. `open_default.rs`: the walk lands on `Home`, its project's button named by the
   offered name.
7. **Check,** `notes/common.md`. This changes what Ready means to its readers:
   `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-world -p
   temper-client-gallery -p temper-docs -p temper-client-tools`, then `heavy cargo check -p temper
   --tests` and the walk `open_default` if the profile runs it. No journey reaches Ready's press:
   `project-landing`'s recording starts at `/chats`, after it, and goes on replaying until
   `record-landing`. Reprint, and expect no change.

## Report

As `notes/common.md`, Your report.
