+++
id = "ready-pane"
part = "ready-pane"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/pages/connect_model.rs",
  "temper/crates/temper-client-domain/src/pages/welcome.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/route.rs",
  "temper/crates/temper-client-domain/src/tests/pages/connect_model.rs",
  "temper/crates/temper-client-domain/src/tests/pages/welcome.rs",
  "temper/crates/temper-client-view/src/pages/model_check.rs",
  "temper/crates/temper-client-view/src/pages/welcome.rs",
  "temper/crates/temper-client-view/src/pages/welcome_model.rs",
  "temper/crates/temper-client-view/src/pages/connect_model.rs",
  "temper/crates/temper-client-view/src/forge/connect.rs",
  "temper/crates/temper-client-view/src/flow.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/**",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 150
+++

# The model's line in Ready's pane

## Goal

"I've signed in" and a key added go on to Ready at once. Ready's checklist goes: its two lines go to the pane the
Model step has, Forge's then Model's, in the same parts, and the check's states are Model's line. The try's status box
in the panel's place, "Continue anyway" and the box's answered "Continue" go, rules and view both. Every commit carries
`Look-request: model-check-pane`.

Acceptance:

- **At once:** on the welcome's Claude way, pressing "I’ve signed in" sends the request and replaces the address with
  `Welcome { step: Ready }` in the same step. A key's way moves on to Ready, replacing, once the key is added and a model
  usable, and once only; refused, it stays with the refusal's words.
- **"Ready" from the first frame:** from the press until the host's check says otherwise, the heading is "Ready", the
  Model step is ticked, the act is enabled and pressing it makes the project. Settled not usable: "Almost ready", the act
  disabled, Model's line Stopped.
- **The pane, every state,** in browser and terminal, node for node as the request's pages: 01 Running with its bar, 02
  "Each step" holding its marks, 03 ticked with no "Each step", 04 Stopped with its why, "Try again" (a secondary
  button, which checks the host again from Ready), "Sign in again" (a quiet link to the way's page, the panel back and the
  stop gone) and "Each step".
- **Nothing yet:** "Forge none yet · Connect a forge", "Model none yet · Connect a model" with the last sign-in's ending
  under it, and "Model PLAN plan · signing in…" in the pane as they are. "Open Forgejo ↗" follows Forge's line.
- **No check running:** a person on a usable account goes from Model to Ready to Create, the line ticked with no
  "answered in N s". The Model step reached with a usable model (‹ Back) keeps its ticked status and "Continue".
- **A refusal stays said:** a refused `DeclareProgram` or `CheckProgram` shows the refusal's words under Model's line.
- **Gone:** `model_check::status`, `stopped_acts`, `ready_note`, `last_try_failed`; `Act::SignInAgain`; the trial's branch
  in `welcome_model.rs` and `connect_model.rs`; `WELCOME_CHECK*`, `WELCOME_CHECKLIST`, `WELCOME_CURRENT`; their rules; the
  words no page says.
- Tests green; each new one bites.

## Inputs

- `temper/docs/plan/look-round-12/notes/common.md`.
- `temper/docs/design/client/README.md`, 3.5: *A model usable*, *A way's page*, *The try*, *Ready*.
- `temper/docs/design/client/README.status.md`: its **Not yet** and **Design departs** on 3.5, *The try*.
- `temper/docs/design/client/browser.md`, 2.5, Ready's pane; the gallery's table, 7.2.
- `jig/docs/design/domain/accounts.md`, its table's "declare a program's login": each host reports once declared, so a
  page left mid-declaration loses no check.
- The request, as the note says.

## Owns and may change

Owns, as `files` lists them.

May also change, call sites and expectations only, each named in the report:

- **Worlds that walk the way or read the lines:** `temper/tests/client/tests/suite/connect_model.rs`,
  `connect_model_edges.rs`, `forges_edges.rs`, `sign_in.rs`, `temper/tests/client/src/**`;
  `temper/crates/temper/tests/support/person.rs`, `sign_in_welcome.rs`, `temper/crates/temper/tests/fake_backends/walk/**`;
  `temper/crates/temper-client-shell-web/tests/suite/host.rs`, `host/walk.rs`, `src/host_tests_*.rs`;
  `temper/tests/client-browser/tests/browser_all/browser_public.rs`; `temper/tests/real-loop/src/case/**`. They find
  "Model ChatGPT plan", "Forge Forgejo on this machine", or press "Continue" after a way: Forge's and Model's usable
  lines lose their label, and the line is `Role::Paragraph`.
- `temper/crates/temper-client-view/src/limits.rs` and `temper/crates/temper-client-shell-*/src/limits.rs`: bounds derived
  from the word list.
- `temper/crates/temper-client-shell-terminal/src/tests/**`: prints of Ready.
- `temper/docs/design/client/README.md`, 3.5, and `README.status.md`'s entries: what the code taught you, a line.

Leave:

- **The journeys' prints** (`temper/docs/design/client/journeys/**`, `temper/tools/journeys/**`): `pane-proof`'s. Their
  tests go red here: report it under `## Left`.
- `temper/docs/design/draft/**`, `temper/docs/design/client/greenfield/**`, `temper/docs/design/client/mockups/**`.
- A sign-in's end moving on: `sign-in-new-tab`'s (the note). Home's line and the accounts row: deferred, no fact.

Options, each pre-approved:

- **Keeping the page on Ready bites a reader** of `pages.connect_model` (`accounts.rs`'s `connect_model().is_some()`, the
  `Act::SignIn` branch of `pages.rs`): make that reader ask for a way's page or `ConnectModel`, and say so.
- **`decision` counting a check out moves a whole store on:** if a test shows it, count it in `make_default` and the view
  alone, and say so.
- **The pane's "none yet" line:** the overlay draws no such state. Draw it with the pane's grid and ring, its link
  underlined, 18 px under the line above, and name the rule in the report.
- **A hidden state word:** the request's "stopped" is a new word `FlowStopped`; Forge's stays "failed".

## Steps

### The domain (`temper-client-domain`)

```rust
// pages/connect_model.rs
pub struct ConnectModel {
    // ...
    pub check: ProgramCheck,
    /// The way "I've signed in" was last pressed on: "Try again" and "Sign in again", on Ready, act on it.
    pub tried: Option<Way>,
}
// Act::SignInAgain goes.
```

- **`Act::SignedIn`:** taken on the way's page as today, or on `Welcome { step: Ready }` with `tried == Some(way)`. It sets
  `tried`; once it has sent, from the way's page, `out.push(Output::Replace { address: Address::Welcome { step: Step::Ready } })`.
  A press that sends nothing moves nowhere. A second press while one is out sends nothing, as today.
- **`changing_way`:** the check stays going to Ready and from Ready to `Model { way: None }`. Ready to a way's page clears
  it, `check = Idle` and `request.cancel()`, as `SignInAgain` did. Any other way change clears it, as today.
- **`went` (`pages.rs`):** the page stays on `Welcome { step: Ready }` while its `check` isn't `Idle`; otherwise it goes
  as today. Leave `connect_model::open` as it is, so that a held sign-in's return drops the page as today. Leaving
  Ready for anywhere else drops it: the check is the engine's.
- **`welcome_program_progress` (`route.rs`):** no move any more for the program, and no address guard: on a page with
  a check out, usable sets it `Idle`, settled sets it `NotYet`, as today. Add the key's move: on `Welcome { step: Model {
  way: Some(key's) } }`, a request of this page pending and the model usable, `Request::Show { address: Ready, replace:
  true }`.
- **`pages/welcome.rs`:**

  ```rust
  /// `model_usable`, or an account just declared whose host's check is out (client/README.md, 3.5, *The try*).
  #[must_use]
  pub fn model_counts(root: &Domain) -> Option<bool>
  ```

  `decision` and `go_on` read it; `model_step` keeps `model_usable`.

Tests in `tests/pages/connect_model.rs`: replace `welcome_claude_ready_replaces_the_pressed_step_and_clears_the_check`
and the `SignInAgain` lines; add, each to bite: the press replaces at once, declare and check alike; the check goes on
across `Went` to Ready and ends `NotYet` or `Idle`; "Try again" on Ready sends a fresh check and a press elsewhere sends
nothing; a way's page clears a stop; a key moves on once; `make_default` makes the project while the check is out.

### The view (`temper-client-view`)

`pages/model_check.rs` is Model's line:

```rust
//! Model's line in Ready's pane: the try's states (temper/docs/design/client/README.md, 3.5, *The try*).
pub(crate) enum First { Subscription, Plan, Key }
pub(crate) enum Answer { SignIn { way: Way } }
pub(crate) enum State {
    /// `at` counts from the first step.
    Running { at: usize },
    /// A usable account, no try being a fact yet: ticked, with no "answered in N s".
    Usable,
    Answered { seconds: u64 },
    Stopped { at: usize, why: Box<[u8]>, answer: Option<Answer> },
}
pub(crate) struct Trial {
    /// `Claude subscription`, `ChatGPT plan`, `OpenAI API key`.
    pub(crate) account: Box<[u8]>,
    /// `Claude`, `ChatGPT`: WHO in "WHO didn't answer" and "WHO answers".
    pub(crate) who: Box<[u8]>,
    /// The Model line's detail, `claude-owner, signed in on this machine`.
    pub(crate) detail: Option<Box<[u8]>>,
    pub(crate) first: First,
    pub(crate) state: State,
}
/// Which state the line draws: the host's check and the `accounts` document alone, no trial being a fact yet.
/// The one function temper/docs/design/draft/model-check.md, 2 to 4, feeds. None: a line with nothing yet.
pub(crate) fn trial(root: &Domain, w: &Words) -> Option<Trial>;
/// The line, in the pane. `model_counts` decides the page's heading and act, not this.
pub(crate) fn line(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, trial: &Trial);
```

- The line uses `forge::connect`'s `stacked` (now `pub(crate)`), with its mark widened from `done: bool` to ring, ticked
  or failed (the last `FLOW_FAILED`), its `Role::Paragraph`, its strong name, its muted details; its terminal branch says
  the line. The bar is `Role::Progress`, `FLOW_BUSY`, browser only, as `welcome_line`'s. "Each step" is a `Role::Disclosure`
  over `flow::progress`, as `each_step`.
- Words, as the pages have them: the muted "Trying: STEP"; the line's name "ACCOUNT · trying: STEP"; "ACCOUNT · WHO IT IS ·
  answered in 6 s"; "ACCOUNT · WHO didn’t answer". Keep `WelcomeTryNoAnswer`, `WelcomeTryAnswered`, the steps' words,
  `WelcomeTryAgain`, `WelcomeSignInAgain`, `WelcomeClaudeNotYet`. `WelcomeTrying`, `WelcomeTryShort`,
  `WelcomeContinueAnyway`, `WelcomeTryFailed` go. The region's name is a new word, "Forge and model".
- Stopped's `why` is a paragraph, `WELCOME_STATUS_WHY` (`temper-welcome-status-why`); its acts a slot, `WELCOME_STATUS_ACTS`
  (`temper-welcome-status-acts`): "Try again", `connect_model::button`, not primary, its act `Act::SignedIn { name:
  root.pages().account_name(), way: tried }`; "Sign in again", `flow::control`'s `Control::Link` on `Meaning::Go(Address::Welcome
  { step: Step::Model { way: Some(model_way(way)) } })`; with no answer, "All ways", `Control::Link` too.
- `flow::Mark::Stopped` styles `FLOW_FAILED` and says the request's "stopped".

`pages/welcome.rs`:

- `ready`: heading and lead from `connected(root) && model_counts(root) == Some(true)`; `setup_ending`, then
  `first_project`, then "Forgejo is still setting up…", then the pane. `steps` ticks Model by `model_counts`.
- The pane: `Role::Complementary`, `WELCOME_STATUS`, named by the new word: Forge's line (`welcome_line`, or the line
  with nothing yet), "Open Forgejo ↗" (`open_link`), then Model's (`model_check::line`, or the signing and none-yet lines
  as they stand, `welcome_end` under the last, and `accounts::outcome` under it where the page keeps a refusal). `main`
  passes `text_per_node`.
- `labelled`, `model_line`'s old body and the checklist go.

`welcome_model.rs`'s `usable` and `program`, and `connect_model.rs`'s `body` and `trial_body`, lose the trial.

Tests: `tests/model_check.rs` rewritten to the line's states, browser and terminal, its plan's and key's two steps and the
"All ways" reason; `tests/welcome.rs`'s frames, `ready_orders_forge_then_model_for_every_readiness_combination` (the pane's
two lines in order, after the form) and the rest that read the checklist; `tests/look.rs`, `tests/flow.rs`,
`tests/model_forge.rs`, `tests/connect_model.rs`.

### The stylesheet

Under `/* ==== ui: welcome ==== */`, the overlay's eight rules as they stand (`git diff ba044d7413^ ba044d7413 --
temper/docs/design/client/journeys/overlay.css`). Remove `temper-welcome-checklist`'s and `temper-welcome-check`'s rules
where they stand, the `@media` lines with them and `.temper-welcome-check` in the `:root:has(...)` at the head of the
greenfield block.

### The gallery

`welcome-model-claude-checking` and `welcome-model-claude-not-yet` become `welcome-ready-checking` and
`welcome-ready-not-yet`, in `names.txt`, sorted as it is, `samples.rs` and `tests.rs`'s `Holds`, as `browser.md`'s table has
them; `welcome-model-connected` and `welcome-model-signin-ready` stay; `welcome-ready-*`'s parts follow the pane.

### The checks

fmt, `old_forms.py`, clippy on what you changed, the focused tests of `temper-client-domain`, `temper-client-view`,
`temper-client-shell-web`, `temper-client-shell-terminal`, `temper-client-gallery`, and the worlds above. This changes what
a shared item means, `decision` reading `model_counts`: run the tests of its readers, those two crates and `temper/tests/client`.
The citations. Push.

## Report

What `notes/common.md`, Your report, asks: one line per state and rule landed. Reds left to `pane-proof`, under `## Left`.
