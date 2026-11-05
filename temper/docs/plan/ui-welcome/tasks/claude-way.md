+++
id = "claude-way"
part = "claude-way"
kind = "feature"
needs = ["model-step"]
files = [
  "temper/crates/temper-client-domain/src/pages/connect_model.rs",
  "temper/crates/temper-client-domain/src/pages/accounts/**",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/route.rs",
  "temper/crates/temper-client-domain/src/domain.rs",
  "temper/crates/temper-client-domain/src/tests/pages/connect_model.rs",
  "temper/crates/temper-client-view/src/pages/welcome_model.rs",
  "temper/crates/temper-client-view/src/accounts/**",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/welcome.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tests/client/tests/suite/connect_model.rs",
  "temper/tools/gallery/**",
  "temper/docs/design/client/journeys/**",
]
effort = "high"
minutes = 150
+++

# The Claude way

## Goal

The welcome's Claude way declares and checks, so that a program's account reaches Ready: **the happy path's first
walk.** Request, section 5.

Acceptance:

- The page as 3.5, *A way's page*'s Claude: "Your Claude subscription"; the panel's words; the `temper-welcome-cmd`,
  `temper program-login claude-owner`, NAME `program_default`'s, no Name field on the welcome; the muted line; the
  primary "I’ve signed in"; "All ways".
- "I’ve signed in" sends `DeclareProgram` under NAME if no account of the person's has that source, then
  `CheckProgram { source }`, in that order, the second on the first's answer. While either is out, the note, a spinner
  and "Checking Claude Code’s sign-in on this machine…".
- Once `model_usable` holds, after a press, the address moves to Ready, replacing the step's.
- Answered, settled, and the source not usable: the page stays, with the alert "Claude Code isn’t signed in here yet.
  Run the command above, then press I’ve signed in." A second press rechecks.
- Standalone `ConnectModel`'s program way keeps everything. While Claude Code is being fetched, the command does that
  itself: nothing to add.
- A render beside the mock's `welcome.html?at=model&way=claude` at 1440, 1000 and 420: checking, not yet.

## Inputs

- `temper/docs/plan/ui-welcome/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-welcome/inputs/request.md`, section 5.
- `temper/docs/design/client/README.md`, 3.5, *A way's page* (Claude); 5.5.
- `temper/docs/design/draft/claude-provider.md`, 1 and 3.
- `temper/crates/temper/tests/fake_backends/walk/program_here.rs`: the order.
- The mock: `temper/docs/design/client/greenfield/app.js`, `way === 'claude'`, `data-claude-check`; `mock.css`'s `.cmd`.

## Owns and may change

Owns the files in `files`.

May also change: `temper/crates/temper-client-view/src/pages/connect_model.rs` (`request_for_way`, `ready`, which know
only `DeclareProgram`: a pending `CheckProgram` must not make standalone misread); `temper/tests/client/src/front/account_acts.rs`,
for a driver of a host's check, if there is none; `temper/docs/design/client/README.status.md`: delete "3.5, *A way's
page*'s Claude".

`claude-always/fetching-words` touches `accounts/words.rs` and `connect_model`'s tests: keep both on a catch-up.

Options, pre-approved:

- `programs::check` refuses `Limit { Sources }` while a check of the source is out (jig's
  `jig-domain-accounts/src/programs.rs`): a second press waits for `settled`, the button being disabled while it's
  out. If the refusal still arrives, it reads as "still checking", not as an alert.
- `declare` refused `Issued`, for a name already issued to the person: find the source again and check it.
- Any other refusal of either: the words 5.5 has for it, through `accounts::outcome`, on the page.
- `Accounts.settled` is the document's: "answered" is the request answered, then `settled` true.

## Steps

1. **The domain,** `pages/connect_model.rs`:

   ```rust
   /// The welcome's "I've signed in" (client/README.md, 3.5, A way's page).
   #[derive(Clone, Copy, Debug, PartialEq, Eq)]
   pub enum ProgramCheck {
       Idle,
       /// `DeclareProgram` out.
       Declaring,
       /// `CheckProgram` out, or answered and not yet settled.
       Checking { source: u32 },
       /// Settled with the source not usable.
       NotYet,
   }
   ```

   `ConnectModel` gains `check: ProgramCheck`, reset when the way changes. `Act::SignedIn { name, way }`: finds the
   source (a loop over `accounts::document`'s accounts, its sources `kind == Program && name == Some(NAME)`, `holder ==
   Party(me.party.number)`; mind `cut`), and sends. A `told` on `AccountProgramDeclared { source, .. }` while `Declaring`
   sends the check under a key of its own, from `account_name`'s serial: the view binds a name at build time, so the
   chained send names its own. Send the check by `c::AccountChange::CheckProgram { source }`, as
   `pages/accounts/acts.rs`'s `ask` does, through `self.request`; `Act::Accounts` is guarded by `accounts_open` and isn't
   the way here.
2. **The move on,** `route.rs`, beside `welcome_progress`: at `Address::Welcome { Model { way: Some(Program) } }` with
   `check` not `Idle` and `model_usable == Some(true)`, `Show { Ready, replace: true }`, and `check` back to `Idle`.
   `NotYet` is set here too, from `settled`, called where `welcome_progress` is.
3. **The view,** `welcome_model.rs`'s `Program` arm: the note while `Declaring` or `Checking`; else the panel. Reuse
   `connect_model`'s `command`, styled `WELCOME_CMD`. Words: `WelcomeClaudeTitle`, `WelcomeClaudeBody`, `WelcomeClaudeNote`,
   `WelcomeClaudeSignedIn`, `WelcomeClaudeChecking`, `WelcomeClaudeNotYet`. On a usable model the step is `Usable`, as
   `model-step` has it.
4. **The look:** `.cmd` from `mock.css`, as `temper-welcome-cmd`.
5. **Tests.**
   - `temper-client-domain`'s `tests/pages/connect_model.rs`: no source: `DeclareProgram`, then on its answer
     `CheckProgram`; a source already held: `CheckProgram` alone; a second press while out does nothing; `NotYet` then
     a second press rechecks; the move on, its `replace`.
   - `temper/tests/client/tests/suite/connect_model.rs`: `CheckProgram` from the welcome, on `program_fixture`, the
     engine answering; through to Ready.
   - `temper-client-view/src/tests/welcome.rs`: each state, browser and terminal; no Name field.
   - Gallery: `welcome-model-claude`, `welcome-model-claude-checking`, `welcome-model-claude-not-yet`, in `names.txt`,
     `samples.rs`, `tests.rs` and `browser.md`'s 7.2 table.
6. **Check,** `notes/common.md`; reprint journeys; render proof; commit and push.

## Report

The order on the wire, from the world test; what a refused second press read as; the builder and directory.
