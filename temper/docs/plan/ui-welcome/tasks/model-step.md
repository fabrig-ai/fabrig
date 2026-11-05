+++
id = "model-step"
part = "model-step"
kind = "feature"
needs = ["forge-step"]
files = [
  "temper/crates/temper-client-domain/src/pages/welcome.rs",
  "temper/crates/temper-client-domain/src/tests/pages/welcome.rs",
  "temper/crates/temper-client-view/src/pages.rs",
  "temper/crates/temper-client-view/src/pages/welcome_model.rs",
  "temper/crates/temper-client-view/src/pages/welcome.rs",
  "temper/crates/temper-client-view/src/pages/connect_model.rs",
  "temper/crates/temper-client-view/src/pages/accounts.rs",
  "temper/crates/temper-client-view/src/accounts/words.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/welcome.rs",
  "temper/crates/temper-client-view/src/tests/connect_model.rs",
  "temper/crates/temper-client-view/src/tests/model_forge.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/**",
  "temper/docs/design/client/journeys/**",
]
effort = "high"
minutes = 170
+++

# The Model step and its ways

## Goal

On the welcome alone, the Model step draws the mock's states: a model usable, a way's own page in one panel, the
sign-in under way, finishing a paste. Standalone `ConnectModel` keeps everything. The Claude way is the next task:
it stays as it draws today, in its panel. Request, section 4.

Acceptance, each in the browser and the terminal:

- **A model usable, at any way or none:** h1 "Connect a model", no lead; the status, ticked, "✓ ChatGPT plan",
  in the same words as Ready's Model line (the email isn't this plan's); the primary "Continue" to Ready; the foot.
  It replaces the choices and the "Signed in to ChatGPT as…" card with its facts, "Continue with ChatGPT", "Manage
  usage" and "…is on the Accounts page", on the welcome alone.
- **A way's page has its own heading,** no lead, one `temper-welcome-panel`. "‹ All ways" goes; "All ways", quiet,
  stands after the primary. Key and Paste as 3.5, *A way's page*: "Add key" and "Finish connecting" disabled until
  typed; the field the panel's width; Paste's refusals and endings keep their words, in the panel.
- **The sign-in under way, the tab not gone yet:** the note, in place of the card, "LLM account sign-in started" and
  the bare "Pending". Its disclosure, "Signed in, but the page couldn't be reached?", stays under it. No `Mockup:` line.
- **Finishing a paste:** the note, "Finishing your sign-in…".
- **Kept:** a sign-in's other endings: declined, held, refused, the port taken, every paste ending. Their words stay.
- **The forge's pane beside the column stays,** in every state.
- **The gallery's fixture offers two cards where a box shows two:** `welcome-model-*`'s providers.
- A render beside the mock's `welcome.html?at=model`, with its `way=key`, `way=paste`, away and finishing states, at
  1440, 1000 and 420: `notes/common.md`, Render proofs.

## Inputs

- `temper/docs/plan/ui-welcome/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-welcome/inputs/request.md`, section 4.
- `temper/docs/design/client/README.md`, 3.5: *Model*, *A model usable*, *A way's page*, *The sign-in under way*;
  3.5's `ConnectModel` (*Signing in*, *The paste*); 5.5, *A sign-in*.
- The mock: `temper/docs/design/client/greenfield/app.js`, `welcomePage()`'s `at === 'model'`; `mock.css`'s `.panel`,
  `.away-note`, `.numbered`, `.field-v`, `.facts`, `.btn.quiet`.

## Owns and may change

Owns the files in `files`.

May also change:

- `temper/tests/client/tests/suite/{connect_model,connect_model_edges,welcome,welcome_m3,welcome_m3_owned,welcome_m4,welcome_edges}.rs`,
  `temper/crates/temper/tests/support/person.rs`: a step on the welcome's Model step that presses "‹ All ways", reads the
  made card or the "Pending". Standalone's stay. `temper/crates/temper/tests/fake_backends/walk/sign_in.rs` is
  `ui-signin`'s: only a word it reads.
- `temper/docs/design/client/README.status.md`: delete "3.5, *A model usable*…", and "The Model step showing an account
  added before as ready".

`claude-always/fetching-words` also touches `accounts/words.rs` and `connect_model`'s tests: keep both on a catch-up.

Options, pre-approved:

- **The sign-in's address link** on the welcome: the tab leaves at once, so the note holds neither the link nor
  "Address". If a test holds them on the welcome, change it. Standalone keeps them.
- **`welcome-model-signin-ready`** keeps its name, and draws the connected state; `welcome-model-renewed` likewise if
  its account is usable.
- A way's sign-in before it is asked keeps its card with "Continue with ChatGPT" and "This browser isn't on temper's
  computer ›", in the panel, under its heading: this is how the paste is reached.

Not this task's: Claude's page (`claude-way`), Ready's words (`ready-step`).

## Steps

1. **The state, pure,** in `temper-client-domain/src/pages/welcome.rs`:

   ```rust
   /// What the welcome's Model step draws (client/README.md, 3.5, Model).
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub enum ModelStep {
       /// A model usable, at any way or none.
       Usable,
       /// No way: the plans' cards.
       Choose,
       /// A sign-in's way, not yet asked or ended with no model.
       SignIn { way: Way },
       /// A sign-in asked, its tab not gone yet.
       Away { way: Way },
       /// A paste accepted.
       Finishing { way: Way },
       /// A sign-in `elsewhere`.
       Paste { way: Way },
       Key { way: Way },
       Program { way: Way },
   }
   /// None until the page and `me` are in.
   #[must_use]
   pub fn model_step(root: &Domain) -> Option<ModelStep>;
   ```

   From `model_usable`, `connect_model::way`, the request's `sign_in_state`, `paste_state`, `elsewhere()`, as the view's
   `body` and `outcome` read them today. A table test: each row of the enum.
2. **The view.** A new module `pages/welcome_model.rs`, one `step(b, root, w, accounts, text_per_node)`, called from
   `welcome.rs`'s `Step::Model` arm in place of its heading, lead and `connect_model::body`; the pane and foot stay
   there. `Choose` keeps today's heading, lead and `choose_body` (make it `pub(super)`). `Program` keeps today's body,
   in the panel, for the next task.
3. **`accounts.rs`:** `paste_body` splits into the ending, returning whether it ended, and its field and finish, so
   the panel can place its numbered list first. The accounts page and standalone `ConnectModel` print as they did. A
   `Meaning`'s disabled-until-typed is what `Waiting { ready }` says.
4. **`connect_model.rs`'s `welcome(root)` branches** that this replaces go; `body` is then standalone's alone, and
   what stays for the welcome is named `pub(super)`. `key_field` takes the field's word.
5. **Words:** new, beside their neighbours, as the design has them: `WelcomeKeyTitle`, `WelcomeKeyBody`,
   `WelcomeKeyField`, `WelcomeKeyFacts`, `WelcomePasteTitle`, `WelcomePasteOpen` (the link) with `WelcomePasteOpenRest`,
   `WelcomePasteCopy`, `WelcomePasteHere`, `WelcomePasteField`, `WelcomePasteFinish`, `WelcomeAllWays`,
   `WelcomeAwayTitle`, `WelcomeAwayBody`, `WelcomeFinishing`. A word with the plan's name in it takes a hole, as
   `paste_plan` gives it.
6. **The look,** under `/* ==== ui: welcome ==== */`, by `notes/contracts.md`. Keep the pane's rules as they are. The
   spinner is `forge-step`'s.
7. **The gallery.** `worlds.rs`'s `providers(w)`: `welcome-model-*` offers ChatGPT and Claude, as a box does. New states
   `welcome-model-connected`, `welcome-model-away`, in `names.txt`, `samples.rs`, `tests.rs` and `browser.md`'s 7.2 table.
   `welcome-model-signing-in` and `welcome-paste-*` redraw: update their `Holds`.
8. **Tests,** in `temper-client-view/src/tests/welcome.rs`, browser and terminal: each state above; the pane beside
   each; a sign-in's declined, held and refused endings as they were; `tests/connect_model.rs` untouched in what it
   asserts of standalone, which proves it kept everything.
9. **Check,** `notes/common.md`; reprint journeys; render proof; commit and push.

## Report

For each state the words and where; what the terminal prints for the notes; the builder and directory of the screenshots.
