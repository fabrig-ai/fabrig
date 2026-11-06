+++
id = "login-command"
part = "login-command"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema",
  "temper/crates/temper-client-protocol-wire/src/lib.rs",
  "temper/crates/temper-client-protocol-wire/src/generated/v1.rs",
  "temper/crates/temper-client-protocol-wire/src/tests.rs",
  "temper/crates/temper-client-protocol-wire/golden/**",
  "temper/crates/temper/src/directories.rs",
  "temper/crates/temper/src/lib.rs",
  "temper/crates/temper/src/box_command.rs",
  "temper/crates/temper/src/fake_backends.rs",
  "temper/crates/temper/src/tests/**",
  "temper/crates/temper-engine-domain/src/boundary.rs",
  "temper/crates/temper-engine-domain/src/route/kinds.rs",
  "temper/crates/temper-engine-domain/src/tests/kinds.rs",
  "temper/crates/temper-service/src/family.rs",
  "temper/crates/temper-service/src/route.rs",
  "temper/crates/temper-service/src/clients.rs",
  "temper/crates/temper-service/src/settings.rs",
  "temper/crates/temper-service/src/startup.rs",
  "temper/crates/temper-service/src/system.rs",
  "temper/crates/temper-service/src/tests/family.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/connect_model.rs",
  "temper/crates/temper-client-domain/src/tests/connect_model.rs",
  "temper/crates/temper-client-view/src/pages/connect_model.rs",
  "temper/crates/temper-client-view/src/pages/welcome_model.rs",
  "temper/crates/temper-client-view/src/tests/connect_model.rs",
  "temper/crates/temper-client-view/src/tests/welcome.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tests/client/src/front.rs",
  "temper/tests/client/tests/suite/connect_model.rs",
  "temper/docs/plan/ui-walk-2/real-check.md",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 150
+++

## Goal

Finding R1: on the Model step, `temper program-login claude-owner`, run exactly as shown, refuses on a box with non-default config, data or cache directories, since it looks under the default config. Make the shown command run on the box being set up.

The request's decision, taken in its smaller form, as it asks: the box answers one string, composed where the facts are, and the page puts it before the command. The client holds no directory as a fact, doesn't quote, and doesn't decide what a default is. It stays a read of its own, beside the deployment's, and not a field of `DeploymentAnswer`, because that one is answered in `family::to_root` before any party is known, the same for every visitor.

Acceptance:

1. On a box whose directories are the defaults, the page shows `temper program-login NAME`, as it does now.
2. Otherwise the owner's page shows `TEMPER_CONFIG=… TEMPER_DATA=… TEMPER_CACHE=… temper program-login NAME`, all three, shell-quoted, at the Model step and on Connect a model. Words pasted into `sh` set exactly those bytes, with spaces, apostrophes, `$` and backticks in the paths.
3. Anyone but the deployment's owner gets a refusal, no path, and the plain command.
4. Under `temper box`, whose child `internal serve --box` gets its directories as flags, the default case stays short.
5. Tests for both cases, offline, in the tiers below; none needs a download or a real service.
6. The real command, verbatim, on a real box with custom directories, up to the sign-in prompt, signing in to nothing. Written in `temper/docs/plan/ui-walk-2/real-check.md`.
7. The long command wraps inside its panel at 1440 and at 420: no horizontal scroll, every character visible.

## Inputs

- `temper/docs/plan/ui-walk-2/notes/common.md`, and the request it names.
- `temper/docs/design/client/README.md`, 4.6, *The program's environment*; 3.5, *Claude*, in the Model step; 11, the two lines on the command.
- `temper/docs/design/protocol/configuration.md`, 12: the three directories.
- `temper/docs/design/protocol/credentials.md`, 3: the login.
- `temper/docs/plan/ui-final-walk/real-checks.md`, R1 and *R1 handoff*, and `temper/docs/plan/ui-final-walk/tasks/real-checks.md`, for how the real check builds and runs.

## Owns and may change

Owns the files in the header. Also:

- **Compiler-forced arms and fields for the new `Read`, `ForgeRead` and `ForgeReadAnswer` variants,** in any file of `temper/`, including `temper/crates/temper-service/src/clients/translation.rs` and its tests.
- **A `settings::Engine` literal's new field** (`temper-service/src/tests/fixtures.rs`, `temper/tests/client/src/component.rs`, ...).
- **`family::forge_read`'s new argument at its call sites** (`temper-service/src/tests/family.rs`, about three lines).
- **A wire version:** if skein's codegen check asks for `family temper-client-wire 14` to move, move it. There's no compatibility to keep.

Pre-approved options:

- If a journey goes stale, which the cut doesn't expect (no recorded journey's print holds the command, and the read goes only at a program's way), re-record it by the request's rules. The recording's box takes default directories under a fresh `HOME`, so that no machine's path lands in a print; change the driver's line if it passes flags. Say so.
- If a fake box's sandbox directories make a test or a walk see the long command, that's right: a fake box has non-default directories, and `setup.md`, 7, has no product path branch on the flag. Change only that test's expected words, and keep what it asserts of the name.
- If `Told::Link { Live }` comes after the address, on a reload, send the read then too.
- If `temper-forge-setup`'s `sign_in::quote` is hard to reach from `temper`, say so and write a small private `quote` in `directories.rs`. `temper` depends on it today.
- If `kinds::read`'s `Answer::None` reaches the client in a form other than a `Readout::Family` with a record, treat every other form as no words.

## Steps

**The string, in the binary** (`temper/crates/temper/src/directories.rs`):

```rust
impl Directories {
    /// `TEMPER_CONFIG=… TEMPER_DATA=… TEMPER_CACHE=…`, or empty where `temper
    /// program-login` finds these three by itself (temper/docs/design/client/README.md, 4.6).
    #[must_use]
    pub fn program_environment(&self, environment: &[EnvironmentEntry], working: &[u8]) -> Box<[u8]>;
}
```

Decide the default by **comparing paths**, never by `origins`: `box_command.rs` runs its child with `directories.flags()`, so a child's origins always say `Flag`. Resolve `settings::resolve(&SETTINGS, &[], environment)` with the three `TEMPER_*` variables taken out, and compare all three, as the absolute paths `resolve` makes. A refusal there (no `HOME`) counts as not the default. Each value goes through `temper_forge_setup::sign_in::quote`. A worst case is `3 * (4 * 4096 + 2)` plus the names and two spaces, 49199 bytes.

**To the service:** `settings::Engine.program_environment: Box<[u8]>` and `Clients.program_environment`, set beside `clients.deployment` in `system.rs`. `settings::Engine` is built in `startup::check` from a jig `Directories`, which holds no environment, so add `pub fn program_environment(&mut self, words: Box<[u8]>)` on `startup::Checked`, called by the binary on the paths that serve (`Command::Serve`, in `lib.rs`), from `Directories::program_environment` and the process's environment. `Box`'s child is a `Serve`, so it needs no line of its own. Check whether `BoxFakeBackends` starts a child the same way.

**The wire** (`client-wire-v1.schema`):

```
versioned record ProgramEnvironmentAnswer { environment: bytes 49216 }
enum Read { playground, setting: SettingRead, proposal_items: ProposalItemsRead, deployment, revision_setting: RevisionSettingRead, program_environment }
```

Regenerate `generated/v1.rs` and the goldens with the generator the crate's tests hold (skein's `skein-codegen`, `skein/docs/design/README.md`, never by hand); `wire::program_environment_answer` and `decode_program_environment_answer`, as `deployment_answer` and its decode are in `lib.rs`.

**The gate** (`temper-engine-domain`): `ForgeRead::ProgramEnvironment` and a unit `ForgeReadAnswer::ProgramEnvironment`, since the engine holds no paths. In `kinds::read_refusal`, this query is permitted when the scope is `ApplicationScope::Deployment` and `people.role(party, DEPLOYMENT)` is the owner's; under any project scope, `own_kind` is false. `family::to_root`'s read arm sends it through `forge_settings_query`, and `route.rs`'s `Delivery::ForgeRead` encodes it from `engine.clients.program_environment`.

**The client** (`temper-client-domain`, `pages/connect_model.rs`):

```rust
impl ConnectModel {
    /// The owner's `program_environment` (client/README.md, 4.6), empty until it answers.
    #[must_use]
    pub fn program_environment(&self) -> &[u8];
}
```

The read goes, once per page, when `pages.address()` names a `Way::Program` (`Address::ConnectModel { way }` or `Address::Welcome { step: Step::Model { way } }`), from where `pages.rs` handles `connect_model::open(&address)`, and with `Readable::Family` as `forge/deployment.rs` builds its read: `c::Scope::Deployment`, `c::Window { rows: 0, after: None }`. Its answer comes in through `ConnectModel::told`'s neighbour in `pages.rs`. Anything but a decoded record leaves it empty. Nothing goes at any other way, so no recorded journey's exchange changes.

**The view:** `connect_model::command(b, w, name, welcome)` gains the page's words, and writes one `Run::Plain`: words, a space, then `named(w, W::Command, name)`. All three of its callers (`welcome_model.rs:261`; `connect_model.rs:481`, `:547`) take them from `root.pages().connect_model()`. `accounts/words.rs` stays (`README.md`, 11).

**The sheet:** where `.welcome-cmd` and the flow's code are drawn, under `/* ==== ui: welcome ==== */`, the long line wraps (`overflow-wrap: anywhere`, as the mock's narrowest does not need it, so nothing else moves). Prove it with a throwaway render at 1440 and at 420, by the request's *Render proofs*.

**Tests, each must bite:**

- `temper/crates/temper/src/tests/`: `the_default_directories_answer_nothing`; `flags_naming_the_defaults_answer_nothing` (the box's child); `one_moved_directory_names_all_three`; `no_home_names_all_three`; `the_words_survive_a_shell` (runs `sh -c` over the answer, then `printf %s "$TEMPER_CONFIG"` and its two neighbours, with spaces, an apostrophe, `$`, a backtick and a newline; offline); `the_widest_directories_fit_the_wire` (4096 apostrophes each, against `w::CEILINGS`).
- `temper-client-protocol-wire/src/tests.rs`: its round trip and trailing refusal, beside the `DeploymentAnswer` lines (`tests.rs:1160`).
- `temper-engine-domain/src/tests/kinds.rs`: `only_the_owner_reads_the_program_environment`: the owner answered; another role at `DEPLOYMENT`, a signed-out sign-in and a project's scope refused.
- `temper-service/src/tests/family.rs`: `the_program_environment_read_goes_to_the_party` (beside `tests/family.rs:2288`); `forge_read_encodes_the_environment`; and `the_deployment_read_stays_answered_before_any_party`, if nothing holds it now.
- `temper-client-domain/src/tests/connect_model.rs`: `a_programs_way_reads_the_environment_once`; `another_way_reads_nothing`; `a_refusal_leaves_the_words_empty`.
- `temper-client-view/src/tests/connect_model.rs` and `welcome.rs`: `the_command_stays_short_without_words` (the existing assertions stand as they are); `the_command_takes_the_words_first`, the whole line, at both pages.
- `temper/tests/client/tests/suite/connect_model.rs`: `the_owner_sees_the_command_the_box_answers` and its empty case; `front.rs` gains `program_environment: Box<[u8]>`, empty by default, and answers the read as it does `Read::Deployment` (`front.rs:1383`).

**The real check.** As `temper/docs/plan/ui-final-walk/tasks/real-checks.md` builds and runs it, a real box, no fake: explicit empty config, data and cache directories, one with a space and one with an apostrophe in its name, a fresh `HOME`, a clean `PATH` and no `TEMPER_*` exported. Open its owner link; Set up temper, Forgejo on this machine, the empty `work/hello`, then Claude. Copy the command from the page, unedited, and run it with `script -qfc` in the same home and `PATH`. It reaches the pinned download and the first sign-in prompt, and stops there: **sign in to nothing,** with no provider and no model call. Then a plain `temper box` with default directories under another fresh `HOME`: the page shows the short command. Write in `temper/docs/plan/ui-walk-2/real-check.md` what ran, what the page showed and what the terminal said, with the machine's own paths shown as `D/`, `HOME/`: no machine's path, sign-in link or token. Evidence stays outside the repository.

**Checks.** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; clippy on every package changed. The focused tests of those packages and their direct dependents, through `heavy`, one command at a time: `temper-client-protocol-wire`, `temper-engine-domain`, `temper-service`, `temper-client-domain`, `temper-client-view`, `temper-client-shell-terminal`, `temper-client-shell-web`, `temper-client-world`, `temper`; and `heavy cargo check -p <dependent> --tests` for whatever else names `w::Read`, `ForgeRead` or `settings::Engine`. Then remove the **Not yet** entry `4.6, *The program's environment*` from `temper/docs/design/client/README.status.md`, run `python3 -I -B tools/citations/citations.py --base origin/main`, commit and push.

## Report

Each acceptance with its test or its evidence. What the real command printed, up to the prompt, paths generalised. Whether any journey went stale, and any test whose expected words moved, by name. Whether the fake box's walk sees the long form, and that you left `accounts/words.rs` plain.

## Rebase (2026-11-06, step merge, foreman ui-walk-2/login-command/foreman-1)

combine both status-entry removals when rebasing onto the integration tip
The merge conflicts only in `temper/docs/design/client/README.status.md`.
Commit `0a153c35ab` removes the program-environment gap; integration commit `cde18effbb` removes the adjacent “A press stays under the finger” gap.
Resolve by leaving both entries absent. Preserve all other incoming status changes; do not take either whole file wholesale.
Main commit `1c32704c0b`, brought into integration by `3e14c14c68`, also adds a now-stale **Not yet** paragraph beginning `3.5, *Claude*, 5.5: the command is the same words whatever the box's`.
Remove that paragraph too: it records the custom-directory command failure this task implements and verifies.
The CSS auto-merged successfully; the refusal reports no code conflict or test failure.
Run the required citations check after resolving the status file.
