+++
id = "frame"
part = "frame"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/address.rs",
  "temper/crates/temper-client-domain/src/domain.rs",
  "temper/crates/temper-client-domain/src/pages/welcome.rs",
  "temper/crates/temper-client-domain/src/tests/address.rs",
  "temper/crates/temper-client-domain/src/tests/pages/welcome.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/pages/welcome.rs",
  "temper/crates/temper-client-view/src/tests/welcome.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-shell-web/src/lib.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/journeys/src/print.rs",
  "temper/tools/gallery/**",
  "temper/docs/design/client/journeys/**",
]
effort = "high"
minutes = 110
+++

# The wizard's frame and the Welcome step

## Goal

The welcome's steps become links to their own steps, each ticked by a fact and the current one always
current; `/welcome?at=welcome` opens the Welcome step; the Welcome step knows a forge is begun; the brand
goes to `/welcome`; the tab's title is one for every step. Request, section 2.

Acceptance:

- **The steps,** in every state, are four links in a list named Steps, keeping today's look (number or
  tick in a ring, the label; below 760 only the current label): `.steps a` in `mock.css`, hover included.
  `pages/welcome.rs`'s `steps` no longer calls `flow::steps`; `ConnectForge` and `ConnectModel` keep it.
- **The tick's table** holds as `temper_client_domain::pages::welcome::step_marks` says (below): Ready
  stays current, black "4", when both are done.
- **`/welcome?at=welcome`** round-trips, opens the Welcome step whatever the store's state, and moves on
  to the chats page, replacing the address, only when the state is whole. Bare `/welcome` decides as it
  did.
- **The Welcome step with a forge begun:** the cards draw as ever, but "Set up temper"'s button reads
  "Go on setting up" and goes to Ready if a model is usable, else Model. The playground's card and the
  note under them are left out while `playground_offered` isn't `Some(true)`.
- **The brand** on the welcome links to `/welcome`, named "temper". Other pages' brands stay.
- **The tab's title** on every welcome step is "Welcome · first run · temper", in the shell, the journeys'
  print and the gallery's.
- A render beside the mock at 1440, 1000 and 420, steps hovered: `notes/common.md`, Render proofs.

## Inputs

- `temper/docs/plan/ui-welcome/notes/common.md`.
- `temper/docs/plan/ui-welcome/inputs/request.md`, section 2.
- `temper/docs/design/client/README.md`, 3.5: *The welcome*, *Its frame*, *Its steps*,
  *`/welcome?at=welcome`*, *The store's state*, *The playground is offered*, *The Welcome step*.
- `temper/docs/design/client/browser.md`, 2.3 (*The welcome's step is its query*), 2.5 (*The welcome's
  title*, *The welcome's look*) and 7.2.
- The mock: `temper/docs/design/client/greenfield/app.js`, `welcomePage()`'s `steps`, and `mock.css`'s
  `.steps`.

## Owns and may change

Owns the files in `files`.

May also change:

- `temper/docs/design/client/README.status.md`: delete the **Not yet** entries this closes: "3.5, The
  welcome's frame", and "The playground offered on Forge's and Model's foot while a real account alone is
  kept". `temper/docs/design/client/browser.status.md`: delete "2.3: `?at=welcome`..." and "2.5: the welcome's
  title...".
- Any `match` on `Step` in the client's crates and tests: the arm the compiler asks for.
- `temper/crates/temper-client-view/src/tests/{rail,frame,connect_forge,connect_model,model_forge}.rs` and
  `temper/tests/client/tests/suite/welcome*.rs`: an assertion that reads the steps as text or the brand as
  "temper chats" on a welcome page, changed with it. `temper/crates/temper/tests/support/person.rs` and the
  walks only if they press the brand or read the steps.

Options, pre-approved:

- If `crate::link` cannot hold the ring and label as the mock's `.n` and `.l` (it can hold `Run`s and
  children), keep the ring as today's `::before` counter on the link and the label its text, and say so.
  Default: today's `::before`, moved from `> span` to `> a`, so that nothing changes but the element.
- If a journey's print moves by this (it does: the steps and the brand), reprint and commit. Its recording
  should not go stale; if one does, run `again` for it, and if the walk changed, leave it to the last
  task, saying so.

Not this task's: Forge's, Model's and Ready's bodies; the `Step::Forge` and `Step::Model` foot's look.

## Steps

1. **The address.** In `temper-client-domain/src/address.rs`:

   ```rust
   pub enum Step {
       /// `/welcome`: decides by the store's state (client/README.md, 3.5).
       Welcome,
       /// `/welcome?at=welcome`: the same step, asked for by its link.
       WelcomeAsked,
       Forge { forge: Option<Choice> },
       Model { way: Option<Way> },
       Ready,
   }
   ```

   A new variant, not a field on `Welcome`, because a recording's events print `Step` and must not go
   stale. `/welcome?at=done` still opens `/welcome`. Add both to the round trip in `tests/address.rs`.
2. **The guard.** `domain.rs`'s welcome guard takes both: at `WelcomeAsked` a state `Unfinished` or `Empty`
   stays and is decided; `Whole` shows the chats page, replacing, as at `Welcome`.
3. **The facts,** pure, in `temper-client-domain/src/pages/welcome.rs`:

   ```rust
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub struct StepMark { pub current: bool, pub ticked: bool }
   /// Welcome, Forge, Model, Ready, in order (client/README.md, 3.5, Its steps).
   #[must_use]
   pub fn step_marks(at: Step, forge_kept: bool, model_usable: bool) -> [StepMark; 4];
   /// A real forge is kept, even under way or cut short: what `decision` calls kept.
   #[must_use]
   pub fn forge_kept(root: &Domain) -> bool;
   /// Where "Go on setting up" and Forge's and Model's "Continue" go: Ready with a usable model, else Model.
   #[must_use]
   pub fn go_on(root: &Domain) -> Step;
   ```

   `done = { welcome: at isn't Welcome or WelcomeAsked, or forge_kept; forge: forge_kept; model:
   model_usable; ready: false }`; `ticked = done && !current`; a choice or a way is still its step.
4. **The view.** `pages/welcome.rs`: `steps` draws, per step, a `Role::Item` styled `FLOW_STEP`, with
   `FLOW_DONE` when ticked and `State::Current` when current, holding `crate::link` to its address
   (`WelcomeAsked`, `Forge { forge: None }`, `Model { way: None }`, `Ready`) and the label's text. The list
   keeps its name and `FLOW_STEPS`. `main`'s `Step::Welcome | Step::WelcomeAsked` arm draws the cards once
   `page.decided`, with `forge_kept`'s button (`Word::WelcomeGoOn`, "Go on setting up", to `go_on`). The
   `playground` local reads `playground_offered(root) == Some(true)`.
5. **The frame.** `frame.rs`: on `Address::Welcome`, the brand is `Act::Go` to `Step::Welcome`, named
   `Word::Brand`. Other addresses keep `BrandChats`.
6. **The title.** `temper-client-shell-web/src/lib.rs`, next to the Home rule: `main.temper-welcome` gives
   "Welcome · first run"; the same in `tools/journeys/src/print.rs` (a test beside `home_tests`) and
   `tools/gallery/src/lib.rs`.
7. **The look.** `temper.css`, in place: `#app:has(> .temper-welcome) .temper-flow-step > span` and its
   `::before`s become `> a`, with `display: flex; color: inherit; text-decoration: none`; the small-screen
   rules likewise; `.temper-flow-step > a:hover { color: var(--gf-ink); }` under `/* ==== ui: welcome ==== */`.
8. **Tests.**
   - `tests/address.rs`: the round trip.
   - `tests/pages/welcome.rs`: `step_marks`'s table: at each of the four steps, with `forge_kept` and
     `model_usable` each way, 16 rows; `at=Ready` both done keeps Ready current and unticked. The guard at
     `WelcomeAsked` for empty, unfinished and whole.
   - `temper-client-view/src/tests/welcome.rs`: `the_steps_mark_the_current_one` becomes links, one
     current; a tick per state, in the browser's and the terminal's print; the brand's `Go`; "Go on setting
     up" and where it goes, with a usable model and without; the playground's card and note left out with a
     forge kept, and the foot's "Try the playground instead" with a real account alone.
   - Gallery: `welcome-go-on`, in `names.txt`, `samples.rs`, `tests.rs` and `browser.md`'s 7.2 table;
     `welcome-first-run-0`'s `Holds` still pass.
9. **Check,** `notes/common.md`; reprint journeys; render proof; commit and push.

## Report

The tick's table, the goldens moved, the builder and directory of the screenshots.
