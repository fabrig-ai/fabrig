+++
id = "model-check-box"
part = "model-check-box"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/pages/model_check.rs",
  "temper/crates/temper-client-view/src/pages/welcome_model.rs",
  "temper/crates/temper-client-view/src/pages/welcome.rs",
  "temper/crates/temper-client-view/src/pages/connect_model.rs",
  "temper/crates/temper-client-view/src/pages.rs",
  "temper/crates/temper-client-view/src/flow.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/**",
  "temper/crates/temper-client-domain/src/pages/connect_model.rs",
  "temper/crates/temper-client-domain/src/tests/pages/connect_model.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tests/client/tests/suite/connect_model.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 100
+++

# The try's status box on the welcome and on `ConnectModel`

## Goal

The welcome's Model step and `ConnectModel` draw the request's status box in all its states, by one
function, from one function that says which state holds. Today's facts reach two states: the host's check
under way and the check settled not signed in, which replace `WelcomeClaudeChecking`'s note and
`WelcomeClaudeNotYet`'s alert. No trial runs (`notes/common.md`, The behaviour is not built here). Every
commit carries `Look-request: model-check`.

Acceptance:

- **Every rule** of the request's own `overlay.css` diff is in `temper.css`, selectors and declarations as
  the overlay has them, under `/* ==== ui: welcome ==== */`. `ConnectModel` carries no `temper-welcome`
  class, so the box's rules, and the `temper-welcome-forge-status`, `temper-welcome-spin` and
  `temper-welcome-ok` rules it takes, reach its page too: widen them by what its state needs, by `:is(..)` or
  a selector list. Say which in the report.
- **The markup** is `model-check/01.html`, `02.html` and `03.html`'s `main`: their roles, names, classes and
  order, the box's classes `temper-welcome-forge-status`, `temper-welcome-check`, with `temper-flow-done`
  answered and `temper-welcome-check-stopped` stopped. Two departures, each the design's: the list is
  `flow::progress`'s, named "Each step" with its hidden state words as Forge's has them, which the terminal
  reads; "Sign in again" is a button styled `temper-flow-secondary`, where 03 has a link, its address being
  the page it stands on.
- **The words,** `temper/docs/design/client/README.md`, 3.5, *The try*, curly apostrophes included.
  `WelcomeClaudeChecking` goes; `WelcomeClaudeNotYet` becomes "Claude Code isn’t signed in here yet. Sign in
  again, then try again.", since the box stands in the panel's place with no command above it.
- **Today's facts,** on the welcome's Program way: `ProgramCheck::Declaring | Checking` draws the box
  Running at its first step, the other two to come, with no "one short message", since no message goes;
  `ProgramCheck::NotYet` draws it Stopped at its first step, that why, then "Try again", "Sign in again",
  "Continue anyway". "Try again" sends `Act::SignedIn`, as the panel's button. "Sign in again" brings the
  panel back, command and "I’ve signed in". "Continue anyway" goes to Ready.
- **`ModelStep::Usable`** with no trial draws as it does today. `welcome_program_progress` stands: a model
  usable moves on to Ready.
- **All the other states are drawn and tested,** reached by nothing yet: Running at any step with its muted
  words; Answered, its muted "answered in 6 s", the list under the disclosure "Each step", and `usable`'s
  "Continue"; Stopped at a later step, those before it ticked; a reason with no act, whose second control is
  "All ways", "Continue anyway" standing after it (`temper/docs/design/draft/model-check.md`, 4: it keeps no
  one out); a plan's and a key's two steps; the box in `ConnectModel`'s card's place, its primary there as
  `ready_body`'s is; Ready's muted "didn’t answer when tried".
- **The terminal:** the same lines as text, with no spinner: "Trying your Claude subscription · one short
  message", "✓ Claude subscription · answered in 6 s", "Claude didn’t answer". A heading stays a heading.
- **The gallery:** `names.txt` keeps its names. `welcome-model-claude-checking` and
  `welcome-model-claude-not-yet` hold the new parts, in `tools/gallery/src/tests.rs`; `samples.rs` walks them
  as it does.
- **Reprinted:** `heavy cargo run -p temper-client-journeys`, its pages committed if any changed;
  `overlay.css` stays empty. No journey is known to reach these states, so expect none: say which.

## Inputs

- `temper/docs/plan/look-round-9/notes/common.md`.
- `temper/docs/design/client/README.md`, 3.5: the welcome's Model step, *A way's page*'s Claude, *The try*.
- `temper/docs/design/client/README.status.md`, its **Not yet** on *The try*.
- `temper/docs/design/draft/look-requests.md`, 3.
- `temper/docs/design/draft/model-check.md`, 3 and 4, for what is not built here.
- The request, as the note says.

## Owns and may change

Owns, as `files` lists them. `temper-client-view/src/pages.rs` takes the one `pub(crate) mod model_check;`
line, and `src/tests.rs` the one `mod model_check;` line.

May also change:

- `temper/crates/temper-client-view/src/limits.rs`: a count derived from the words.
- `temper/tools/gallery/src/samples.rs`: only if a state stops reaching its page.
- Pages under `temper/docs/design/client/journeys/*/`: what the reprint writes, never by hand.
- `temper/docs/design/client/README.md`, 3.5, *The try*, `README.status.md`'s entry on it and
  `browser.md`, 7.2's two rows: a line the code taught you, named in the report.

Readers already found, each edited for what this changes:

- `temper-client-view/src/tests/welcome.rs`, about line 2360: the status's new name, `WELCOME_CHECK` for
  `WELCOME_AWAY`; frame 3's assertions take the box where they read the panel's alert and command.
- `temper/tests/client/tests/suite/connect_model.rs`, about line 307: its first sentence stands; the second
  press is of "Try again".
- `temper/tools/gallery/src/tests.rs`, about line 1200: the two `Holds` above.

Options, each pre-approved:

- **`accounts::outcome` draws the answered check** once the panel is back: `Act::SignInAgain` also clears
  what `Act::Cancel` clears. If that would drop a fact a test reads, gate `outcome` on the request's
  being other than the check's, and say so.
- **`welcome_model::all_ways`** is private: widen it to `pub(super)` for `model_check`, or pass the way out.
- **A `Holds` part can't say absent text:** assert the command's absence by `Part::Absent` on what it can
  name, or in the view's test, and say so.

Leave: `route.rs`'s `welcome_program_progress`, `temper-engine-*`, `temper-service`, the wire, skein,
`temper/docs/design/draft/`, `temper/docs/design/client/greenfield/` (its README keeps look requests' changes
on top of it) and whatever `notes/common.md` gives `look-round-7`.

## Steps

1. **The domain,** `pages/connect_model.rs`: a new `Act::SignInAgain`, guarded as `Act::SignedIn` is by
   `matches_welcome_program(current)`, turning `ProgramCheck::NotYet` to `Idle`, with a test beside
   `checked`'s.
2. **The module,** `temper-client-view/src/pages/model_check.rs`:

   ```rust
   //! The try of a model, its status box (temper/docs/design/client/README.md, 3.5, *The try*).

   /// The steps' first line: what the way's account has already done.
   pub(crate) enum First { Subscription, Plan, Key }
   /// The act that answers a reason.
   pub(crate) enum Answer { SignIn }
   pub(crate) enum State {
       /// `at` counts from the first step; `message`, whether a message goes.
       Running { at: usize, message: bool },
       Answered { seconds: u64 },
       Stopped { at: usize, why: Box<[u8]>, answer: Option<Answer> },
   }
   pub(crate) struct Trial {
       /// "Claude subscription", "ChatGPT plan", "OpenAI API key".
       pub(crate) account: Box<[u8]>,
       /// "Claude", "ChatGPT".
       pub(crate) who: Box<[u8]>,
       pub(crate) first: First,
       pub(crate) state: State,
   }
   /// Which state the box draws: the host's check alone, no trial being a fact yet. The one
   /// function temper/docs/design/draft/model-check.md, 2 to 4, feeds.
   pub(crate) fn trial(root: &Domain, w: &Words) -> Option<Trial>;
   /// Whether the last try stopped: no fact yet, the same draft's 4.
   pub(crate) const fn last_try_failed() -> bool;
   /// The box.
   pub(crate) fn status(b: &mut Builder, w: &Words, trial: &Trial);
   /// A stopped try's controls, in a `WELCOME_ACTS` slot. `go_on` is where "Continue anyway" goes.
   pub(crate) fn stopped_acts(
       b: &mut Builder,
       root: &Domain,
       w: &Words,
       bindings: &mut Bindings,
       trial: &Trial,
       way: Way,
       go_on: Address,
   );
   /// Ready's muted line after a try that stopped.
   pub(crate) fn ready_note(b: &mut Builder, w: &Words);
   ```

   A step's three marks are `flow::Mark`'s, with a new `Mark::Stopped` for `look::WELCOME_CHECK_FAILED` and
   `Word::FlowFailed`: 03's selectors read that class, where `Mark::Failed` styles `temper-flow-failed`.
3. **The look,** `look.rs`: `WELCOME_CHECK`, `WELCOME_CHECK_STOPPED`, `WELCOME_CHECK_FAILED`,
   `WELCOME_CHECK_WHY`.
4. **The words,** `words.rs`, each as *The try* has it: `WelcomeTrying`, `WelcomeTryShort`, `WelcomeTryAnswered`,
   `WelcomeTryNoAnswer`, `WelcomeTrySubscription`, `WelcomeTryPlan`, `WelcomeTryKey`, `WelcomeTryStarts`,
   `WelcomeTryAnswers`, `WelcomeTryAgain`, `WelcomeSignInAgain`, `WelcomeContinueAnyway`, `WelcomeTryFailed`.
   "Each step" is `Word::ForgeProgress`.
5. **The callers:** `welcome_model::program` draws its heading, then, with `trial` some, `status` and, stopped,
   `stopped_acts`, returning before the panel. `welcome_model::usable`, with `trial` some, draws `status` in
   its status's place; its "Continue" stands. `connect_model::body`, above `if ready(root, way)`: with `trial`
   some, `flow::details` under `ending_heading`, `status`, then for Answered `ready_body`'s primary (extract
   its few lines, no more) and for Stopped `stopped_acts`, `go_on` that primary's address; closed, return.
   `welcome::model_line`: after the usual line, `if model_check::last_try_failed() { ready_note }`.
6. **Tests, each biting,** in a new `tests/model_check.rs`:
   - from `ProgramCheck`: Checking is Running at 0 with no muted words, NotYet is Stopped at 0 with its why and
     `Answer::SignIn`, Idle is `None`;
   - `status` for each `State`, on a `Trial` built by hand: 01, 02 and 03's roles, names, classes and order;
     the marks of the steps before, at and after `at`; the spinner in the browser alone;
   - the steps of `First::Plan` and `First::Key`: two, the first ticked at once;
   - `stopped_acts`: the order, with and without an `Answer`; each control's act or address;
   - the terminal's three lines;
   - `ready_note`'s words.

   In the domain's and the world's tests: "Sign in again" brings the panel back and a press of "I’ve signed
   in" checks again; "Try again" sends the check; "Continue anyway" reaches Ready with no model. `tests/look.rs`
   names `temper-welcome-check` and `temper-welcome-check-why`.
7. **A change of meaning runs its readers:** `heavy cargo nextest run -p temper-client-domain -p
   temper-client-view -p temper-client-shell-terminal -p temper-client-shell-web -p temper-client-gallery -p
   temper-client-journeys -p temper-client-world`, and `heavy cargo check -p temper-one-process-world -p
   temper-client-browser-world --tests`.
8. **Reprint,** and read the diff.
9. **Fmt, clippy, the old forms, the citations;** commit and push.

## Report

Each overlay rule and page change you landed, one line each; the selectors you widened; which gallery
states changed; which pages reprinted; the tests' count; and what `notes/common.md`, Your report, asks.
