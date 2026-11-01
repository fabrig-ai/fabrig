+++
id = "cl-play-domain"
part = "client"
kind = "feature"
needs = ["pg-wire"]
files = ["temper/crates/temper-client-domain/**", "temper/tests/client/src/**"]
effort = "high"
minutes = 120
+++

## Goal

Move temper's client domain from the demo to the playground (draft
`temper/docs/design/draft/playground-reset.md`, 6.1 to 6.3): it reads
`Read::Playground`, starts by `StartPlayground`, ends by `EndPlayground`,
and drops the demo's two halves, its retired identity, its deployment
account watch and its automatic account removal, and the setup strip's
facts. `Demo` becomes `Playground` with no alias left. The view's new
pages are task `cl-play-view`'s; here the view, the shells' fixtures, the
gallery and the client world only follow the domain's names.

Acceptance:

- `temper_client_domain::forge::Playground` and `PlaygroundState` as
  `notes/playground.md`, section 6, has them; `forge/playground.rs`
  replaces `forge/demo.rs`; `Demo`, `DemoState`, `RetiredDemo`, `Halves`,
  `halves`, `current`, `usable_account`, `other_model_usable`,
  `deployment_model`, `owner_deployment`, `progress`, `observe_end`,
  `watch_deployment` and `remove` are gone, with their readers.
- The read: `w::Read::Playground`, once per live link and again after
  each start or end answer; its answer decoded by
  `decode_playground_answer`, replacing the last (an undecodable answer
  counts as none, as the demo's does).
- The acts (`forge::form::Act`): `StartPlayground` (was `StartDemo`,
  one start at a time, a failure kept until retried with a new key) and
  `EndPlayground`: sent once, kept as `Leaving { request }` until its
  answer; on `reset` (or `unknown`) the session reads the playground and
  the forges again, and the root shows
  `Address::Welcome { step: Step::Forge { forge: None } }` (replace:
  false). A read saying `resetting` with no `Leaving` sends
  `EndPlayground` with a new key (a reset a restart cut).
- Accessors on `forge::Side` (names binding, for the view):
  `playground_answered() -> bool`, `playground() -> Option<&Playground>`,
  `playground_running() -> bool` (running, or a start pending or
  failed), `playground_starting() -> bool`, `playground_failure() ->
  Option<&StartFailure>`, `leaving() -> bool` (an `EndPlayground` in
  flight or a `resetting` read), `playground_offered() -> bool` (the
  read says `offered` and no start is pending), `serves_pages(forge)`
  (false for the playground's forge while one is read). On `Domain`:
  `playground() -> Option<&forge::Playground>` while running (no halves).
  `pages::Act::RetryDemo` becomes `pages::Act::RetryPlayground`;
  `pages::Act::Playground` stays (Try the playground); a new
  `pages::Act::LeavePlayground` sends `EndPlayground`.
- Try the playground: `StartPlayground`, then the chats page; the
  project is the read's `project` once the start answers `running` and the
  read is answered again (today's configured project is gone).
- `forge::listed::setup` and `Setup` go (the setup strip); `listed::playground`
  returns the playground's row while the playground runs, without halves.
- `pages::welcome::decision`: a running playground moves on (as today);
  otherwise the owner's run is over when a forge other than the
  playground's is connected and a model is usable; `model_usable` reads
  a usable account line of a provider other than the playground's
  (`Playground::provider`), the person's or, for an owner, the
  deployment's, as today's `other_model_usable` does without the halves'
  state. The welcome's three states and its Forge
  step are task `cl-welcome`'s: change no more here than this.
- Domain tests: `src/tests/demo.rs` becomes `src/tests/playground.rs`,
  the halves' and removal cases deleted, and these added:
  `the_playground_is_read_once_per_live_link_and_after_each_answer`,
  `a_start_answers_running_and_the_reread_names_its_project`,
  `end_playground_is_sent_once_and_reset_opens_the_forge_step`,
  `a_resetting_read_sends_end_playground_with_a_new_key`,
  `not_offered_and_offered_are_told_apart`; `src/tests/pages/welcome.rs`
  and `src/tests/forges.rs` cases on the demo moved to the playground's
  names, the strip's cases deleted.
- The client world (`temper/tests/client`): its scripted front
  (`src/front.rs`, `src/front/account_acts.rs`, `src/translate/forge.rs`,
  `src/translate.rs`) answers `Read::Playground` and `StartPlayground`/
  `EndPlayground` as it answered the demo's (fixtures: offered, running
  with a project, resetting; `end_playground` answered `reset` after which
  reads answer `offered`), so the world's suites still find a playground.
  Its suite and QA tests that only name the demo are moved to the new
  names here; those asserting the halves, the strip or the banner's
  Connect links fail until `cl-play-view`: list them under `## Left`.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 3),
  `temper/docs/plan/playground-reset-r5/notes/playground.md` (sections 5
  and 6, binding).
- The draft, 6.1 to 6.3. `temper/docs/design/client/README.md`, 4.8 (the
  demo's halves, which go) and 3.5 (the welcome).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s3-playground-today.md`,
  section 3 (every client reader, file:line) and section 5's
  `temper-client-domain` and `temper-client-world` lines.

## Owns and may change

- Owns: `temper/crates/temper-client-domain/**`;
  `temper/tests/client/src/**` (the scripted front and translations).
- May change, call sites and fixtures only (the domain's new names and
  types, no new behaviour): `temper/crates/temper-client-view/src/**`
  (where it reads `root.demo()`, `Halves`, `demo_*` accessors and the
  strip: read `playground()`/`playground_running()`, treat the
  model and forge halves as "the playground runs", drop the strip's draw
  call), `temper/crates/temper-client-view/src/tests/fixtures.rs` and the
  view's tests' fixtures; `temper/crates/temper-client-shell-terminal/src/tests/**`,
  `temper/crates/temper-client-shell-cli/src/tests/**`,
  `temper/tools/gallery/src/**`, the
  client world's `tests/**` (names only).
- Not yours: the view's words and pages' new drawing (`cl-play-view`),
  `temper-client-tools` (`cl-tools`), the wire (`pg-wire`, merged).
- Options: if a view page needs a fact the halves gave (the model half
  showing "Scripted model" on a chat's bar), take it from
  `playground_running()` and the chat's project being the playground's;
  if a view test fails only because the strip or a Connect link is gone,
  leave it failing for `cl-play-view` and list it.

## Steps

1. `forge/vocabulary.rs` (`Playground`, bounds from the new wire
   ceilings), `forge/playground.rs`, `forge/family.rs` decoding,
   `forge/form.rs` acts, `forge.rs` accessors and dispatch, `route.rs`
   (`forge_demo` → `forge_playground`, `demo_facts` → `playground_facts`,
   no `demo_progress`), `pages.rs` facts, `domain.rs`, `listed.rs`,
   `pages/welcome.rs`, and the other readers the survey lists.
2. Domain tests; `heavy cargo nextest run -p temper-client-domain --lib`.
3. The view's, shells', gallery's call sites: `heavy cargo check -p
   temper-client-view --tests`, `-p temper-client-shell-terminal`, `-p
   temper-client-shell-cli`, `-p temper-client-gallery`.
4. The client world's front; `heavy cargo nextest run -p
   temper-client-world --test suite -E 'test(/welcome|demo|playground/)'`
   then the whole `suite` binary once.
5. `cargo fmt`; `heavy cargo clippy -p temper-client-domain -p
   temper-client-view -p temper-client-world --all-targets -- -D
   warnings`; the focused tests of `temper-client-domain`,
   `temper-client-view`, `temper-client-shell-terminal`,
   `temper-client-shell-cli`, `temper-client-world`. Old-forms from
   `temper/`. Commit and push.

## Report

As `notes/common.md` says; under `## Left` every view and world test
left failing for `cl-play-view`, by name.
