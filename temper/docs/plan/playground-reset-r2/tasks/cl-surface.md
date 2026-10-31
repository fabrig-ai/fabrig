+++
id = "cl-surface"
part = "client"
kind = "interface"
needs = ["cl-wire"]
files = [
  "temper/crates/temper-client-domain/**",
  "temper/crates/temper-client-view/**",
  "temper/crates/temper-client-shell-web/**",
  "temper/crates/temper-client-shell-terminal/**",
  "temper/crates/temper-client-shell-cli/**",
  "temper/crates/temper-client-tools/**",
  "temper/tests/client/**",
  "temper/tools/gallery/**",
  "temper/docs/design/client/journeys/**",
]
effort = "high"
minutes = 75
+++

## Goal

Give the client's domain and view the vocabulary every later client task
builds on, compiling, with today's behavior: the new forge choices and
the project settings address, the forge acts for removal, start over and
the playground's end, the removal's and the end's asks and answers
(real), a `playground()` read bridged from today's demo, the forge row's
`finished`, a project settings page and its link (empty), and the hooks
the later tasks fill in files of their own. After it, `cl-connect`,
`cl-remove`, `cl-play-domain`, `cl-welcome` and `cl-project` run at once
on disjoint files.

Acceptance (new tests, each in the package named):

- `temper-client-domain`, `src/tests/address.rs`:
  `new_forge_choices_and_project_settings_round_trip` (print then parse
  gives back `ConnectForge { forge: Some(Choice::New(NewForge::GitHub)) }`,
  `Here`, the welcome's `Step::Forge` with each, and
  `Address::Project { project: 7 }` at `/projects/7/settings`).
- `temper-client-domain`, new `src/tests/remove.rs`:
  `a_removal_asks_for_its_forge_in_the_deployment_scope` and
  `each_removal_answer_decodes` (removing, not offered, unknown; another
  outcome is not this ask's answer).
- `temper-client-domain`, new `src/tests/playground.rs`:
  `the_end_asks_in_the_deployment_scope_and_its_answers_decode` and
  `a_running_demo_reads_as_a_running_playground` (the bridge).
- `temper-client-view`, `src/tests/frame.rs`:
  `a_chosen_project_offers_its_settings_beside_adoption`.
- Every focused test of the changed packages passes unchanged in meaning.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 5.
- `temper/docs/plan/playground-reset-r2/tasks/cl-wire.md` (the wire as it
  merged: `remove_forge`, `end_playground`, the outcomes).
- `temper/docs/design/draft/playground-reset.md`, 3 ("Its client"), 4
  ("Its client"), 5 ("Its client"), 6.1, 6.2, 6.4.
- `temper/docs/design/client/README.md`, 3.5 (addresses), 4.6 (the
  family's requests).
- `temper/docs/design/client/journeys.md`, 6 and 8 (a reprint; a stale
  journey).

## Owns and may change

Owns, for this task's run, the client crates in the header. No other
task writes them while it runs. Everything below is additive or a move;
behavior stays today's except where a step says.

Readers of the items it adds or moves (forced arms, scope.md):
`temper-client-domain/src/{departure.rs,pages/connect_model.rs,forge.rs,route.rs,acts.rs,pages/task.rs,pages/chat.rs}`,
`temper-client-view/src/{root.rs,frame.rs,choice.rs,pages/connect_model.rs}`,
`temper-client-shell-web/src/host.rs`, `temper-client-shell-terminal/src/route.rs`,
`temper-client-tools/src/root.rs`, `temper/tools/gallery/src/*.rs`,
`temper/tests/client/src/**`; and any other exhaustive match on
`Address`, `forge::Act`, `pages::Act` or `Meaning` the compiler names.

## Steps

Paths below are under `temper/crates/temper-client-domain/src` (D) and
`temper/crates/temper-client-view/src` (V). Module docs of new files cite
`temper/docs/design/draft/playground-reset.md, N`.

1. **Addresses (D/address.rs).**
   - `pub enum Choice { Forge(u16), New(NewForge), Elsewhere }` and
     `#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum NewForge {
     GitHub, Here }`: a forge to make, of a kind, GitHub's App or an
     owned Forgejo on this machine (draft 3, "Its client"). Printed as
     `forge=github` and `forge=here` wherever `print_choice` prints a
     number today (`/forges/connect?forge=github`,
     `/welcome?at=forge&forge=here`); parsed back by `forge_choice`.
     `Elsewhere` stays as today (`cl-connect` decides its fate).
   - `Address::Project { project: u32 }`: the project's settings,
     `/projects/N/settings`, parsed beside `/projects/N/adopt` with the
     same `project_decimal`.
2. **Forge acts (D/forge/form.rs).** `Act::StartDemo` becomes
   `Act::StartPlayground` (same meaning); add `EndPlayground`,
   `RemoveForge { forge: u16 }` (ask to remove: opens its confirmation),
   `StartOver { forge: u16 }` (the same, from the Forge step: draft 6.4),
   `ConfirmRemove`, `CancelRemove`. Existing payloads stay
   (`cl-connect` changes them).
3. **The removal (new D/forge/remove.rs, `pub mod remove` in D/forge.rs).**
   ```rust
   #[derive(Debug, Default)]
   pub(super) struct State { pub removal: Option<Removal> }
   #[derive(Clone, Debug, PartialEq, Eq)]
   pub struct Removal { pub forge: u16, pub start_over: bool, pub phase: Phase }
   #[derive(Clone, Debug, PartialEq, Eq)]
   pub enum Phase { Confirming, Sending { request: Token, progress: RequestProgress }, Refused(Refusal), Removed }
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub enum Refusal { NotOffered, Unknown }
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub enum RemoveAnswer { Removing, Refused(Refusal), Unknown }
   impl Side { pub fn removal(&self) -> Option<&Removal> }
   pub fn ask(forge: u16, limits: &Limits) -> Option<jig_client::Ask>;   // real
   pub fn answer(value: &jig_client::Answered, limits: &Limits) -> Option<RemoveAnswer>; // real
   pub(crate) fn act(side: &mut Side, limits: Limits, act: &Act, out: &mut Queue<Request>); // stub: nothing
   pub(crate) fn told(side: &mut Side, told: &jig_client::Told, limits: Limits, out: &mut Queue<Request>) -> bool; // stub: false
   ```
   `ask` encodes `w::Request::RemoveForge` in the deployment's scope as
   `demo::ask` encodes the start; `answer` decodes `Removing`,
   `RemoveRefused` (each `why`) and `Unknown` as `demo::answer` decodes a
   start, an entrance refusal as `Unknown`, any other outcome `None`.
   `Side` gains `remove: remove::State`. `forge::step`'s `act_on` sends
   `RemoveForge`, `StartOver`, `ConfirmRemove` and `CancelRemove` to
   `remove::act`. In D/route.rs, a `forge_remove` hook beside
   `forge_demo` offers each `Told` to `remove::told` first (a `true`
   consumes it), as `forge_demo` does.
4. **The playground (D/forge/demo.rs).** Move the `impl Side` demo
   accessors from D/forge.rs (`demo_answered`, `demo`, `demo_running`,
   `demo_starting`, `demo_failure`) into an `impl Side` block in
   demo.rs, unchanged. Add:
   ```rust
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub enum Playground<'a> { Running(&'a Demo), Leaving }
   impl Side { pub fn playground(&self) -> Option<Playground<'_>> } // bridge: Running(answer) when demo_running() and the answer is held, else None
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub enum EndAnswer { Resetting, NotOffered, Unknown }
   pub fn end_ask(limits: &Limits) -> Option<jig_client::Ask>;                       // real: w::Request::EndPlayground, deployment scope
   pub fn end_answer(value: &jig_client::Answered, limits: &Limits) -> Option<EndAnswer>; // real
   pub(crate) fn end(side: &mut Side, limits: Limits, out: &mut Queue<Request>);      // stub: nothing
   ```
   `Act::EndPlayground` goes to `demo::end`. In D/domain.rs add
   `pub fn playground(&self) -> Option<forge::demo::Playground<'_>>`
   delegating to the side. `Playground::Leaving` is never made here
   (`cl-play-domain` makes it).
5. **The welcome's decision moves:** D/domain.rs's private `fn welcome`
   moves unchanged to D/pages/welcome.rs as `pub(crate) fn decide(root:
   &mut Domain, out: &mut Queue<Request>)`; `step` and `fire` call it.
   (D/pages/welcome.rs is `cl-welcome`'s next; D/domain.rs
   `cl-play-domain`'s.)
6. **The forge row's `finished`:** `forges::Row` gains `pub finished:
   bool`, decoded from the wire row in D/forge/family.rs; every literal
   gets it (scope.md).
7. **The project page (new D/pages/project.rs).**
   ```rust
   #[derive(Debug)]
   pub struct Project { pub project: u32 }
   pub(crate) fn act(pages: &mut Pages, env: &Env<Limits>, client: &jig_client::Client, act: &Act, out: &mut Queue<Request>); // stub: nothing
   pub(crate) fn told(pages: &mut Pages, told: &jig_client::Told) -> bool; // stub: false
   ```
   (adjust the parameters to what `pages.rs` holds at the call; keep the
   names.) `pages::Act` gains `DeleteProject`, `DeleteWords { words:
   Box<[u8]> }`, `ConfirmDelete`, `CancelDelete`, each sent to
   `project::act`. `Pages` gains `project_page: Option<project::Project>`
   and `pub const fn project_page(&self) -> Option<&project::Project>`;
   `went` makes it for `Address::Project { project }` and drops it on
   leaving, as it does `welcome`; the pages' told routing offers each
   `Told` to `project::told` first.
8. **View bindings (V/forge/bindings.rs).** `Meaning` gains
   `RemoveForge(u16)`, `StartOver(u16)`, `ConfirmRemove`, `CancelRemove`,
   `EndPlayground`; `decode` makes the forge acts of step 2 of them on a
   press.
9. **View hooks.**
   - New `V/forge/remove.rs` (`pub(crate) mod remove` in V/forge.rs):
     `pub(crate) fn control(b: &mut Builder, root: &Domain, words:
     &Words, bindings: &mut Bindings, row: &Row, start_over: bool)`,
     drawing nothing. V/forge/forges.rs calls it after each personal
     row's own actions with `start_over: false`.
   - New `V/pages/project.rs`: `pub(crate) fn main(b: &mut Builder,
     root: &Domain, words: &Words, bindings: &mut Bindings,
     text_per_node: u32)` drawing an empty `Role::Main` with
     `crate::look::MAIN`; V/root.rs draws it for `Address::Project`,
     after the banners as `Adopt` is.
   - V/frame.rs's navigation: after the Adoption link, a link "Project
     settings" (`Word::ProjectSettings`, new in V/words.rs, the one
     phrase this task adds there) to `Address::Project { project }`,
     current on that address.
10. **Forced arms** in every reader above (scope.md): a new `Address` is
    excluded wherever `Adopt` is excluded, and drawn as `Adopt`'s kin
    (frame, banners, departure, the shells' first address).
11. **Words of later tasks** (a rule for them, written here once): each
    later client task keeps its new phrases in a words module of its own
    after `V/accounts/words.rs`'s pattern (its own `Word`, `say`, phrase
    numbers under a free owner), not in V/words.rs, so they never edit
    one file at once. Write that rule as a module doc line in
    `V/forge/remove.rs` and `V/pages/project.rs`.
12. **Checks:** the new tests; `cargo fmt`; `heavy cargo clippy -p
    temper-client-domain -p temper-client-view -p temper-client-tools
    -p temper-client-shell-cli -p temper-client-shell-terminal -p
    temper-client-shell-web -p temper-client-world -p
    temper-client-gallery --all-targets -- -D warnings`; one `heavy
    cargo nextest run` over the same packages and
    `temper-client-journeys`.
13. **Journeys:** the frame's new link changes the prints, not the
    recordings: `heavy cargo run -p temper-client-journeys` reprints;
    commit the reprinted pages. If the journeys test reports one stale,
    an act's Debug text changed (journeys.md 3.3): put the recording's
    `act` line's event text to the new print, then `heavy cargo run -p
    temper-client-journeys -- again NAME`; if `again` refuses, stop and
    ask (`Question:`).
14. Old forms from `temper/`; commit and push.

## Report

- The final signatures of steps 3, 4 and 7, as built (the later client
  tasks read them).
- Every forced arm outside the client domain and view.
- Whether the journeys were reprinted or recorded again.
