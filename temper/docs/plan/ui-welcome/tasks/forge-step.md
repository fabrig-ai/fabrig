+++
id = "forge-step"
part = "forge-step"
kind = "feature"
needs = ["frame"]
files = [
  "temper/crates/temper-client-view/src/forge/connect.rs",
  "temper/crates/temper-client-view/src/pages/welcome.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/welcome.rs",
  "temper/crates/temper-client-view/src/tests/connect_forge.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/**",
  "temper/docs/design/client/journeys/**",
]
effort = "high"
minutes = 90
+++

# The Forge step with a forge

## Goal

With a personal forge under way or connected, the Forge step says it as the mock does, and moves on with
"Continue". "View" goes. Request, section 3.

Acceptance, on the welcome's Forge step alone:

- **Under way, or connected (GitHub with an accepted installation counts):** h1 "Connect a forge"; lead
  "One forge is enough to begin; add others later, from Settings."; `temper-welcome-forge-status`, in the
  column, not the pane; the primary "Continue" to `go_on(root)`; the foot.
- **The status:** under way a spinner, the forge's name, strong, a muted line, and the disclosure "Each
  step"; connected a tick, the name, and "connected as temper · work/hello made". On GitHub "temper’s App
  on ACCOUNT", ACCOUNT the first accepted installation's. No repository count.
- **Stay as they are:** a failed set up, a forge cut short ("not finished"), GitHub's install and Forgejo's
  admin steps. "View" is drawn on none of them, on the welcome.
- `ConnectForge` keeps everything.
- A render beside the mock's `welcome.html?at=forge` at 1440, 1000 and 420, under way and connected, with
  "Each step" open: `notes/common.md`, Render proofs.

## Inputs

- `temper/docs/plan/ui-welcome/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-welcome/inputs/request.md`, section 3.
- `temper/docs/design/client/README.md`, 3.5: *Forge*, *A forge cut short*, *The Forge step with a forge*.
- `temper/docs/design/client/forge.md`, 5.4.
- The mock: `temper/docs/design/client/greenfield/app.js`, `welcomePage()`'s `at === 'forge'`, branch
  `else if (f)`, and `forgeStatus`, `forgeSteps`; `mock.css`'s `.forge-status`, `.spin`, `.ok`, `.sep`,
  `.each`, `.acts`.

## Owns and may change

Owns the files in `files`.

May also change:

- `temper/docs/design/client/README.status.md`: delete "3.5, The Forge step with a forge".
- `temper/tests/client/tests/suite/{welcome,welcome_m3,welcome_m3_owned,welcome_m4,welcome_edges,github_connect}.rs`
  and `temper/crates/temper/tests/support/person.rs`, `tests/fake_backends/walk/*`: a step that presses
  "View" or reads the old line on the welcome's Forge step, changed with it. `ConnectForge`'s stay.

Options, pre-approved:

- The status's muted line under way uses `forge::words::forge_connection(row)`, so a fetch's figure stays
  ("Setting up: fetching Forgejo · 42%"). `welcome_line`'s pane keeps what it draws.
- A `forge=N` address on a connected forge keeps moving on to Model, as 3.5 says; a test that holds it stays.

Not this task's: Model's pane, Ready's lines.

## Steps

1. **The predicate,** in `temper-client-domain`'s `forge/connect.rs` beside `underway`:
   `pub fn status_shown(row: &Row, playground: Option<u16>) -> bool`: not failed, not cut short, not an
   unaccepted GitHub, not an admin's Details; under way or connected. Decided by what `welcome` in the
   view falls to today; a table test beside its neighbours.
2. **`forge/connect.rs`'s `welcome`:** that branch calls `forge_status(b, root, w, row)`, then
   `temper-welcome-acts` holding the primary "Continue" (`Word::WelcomeContinue`), `Act::Go` to
   `temper_client_domain::pages::welcome::go_on(root)`. Remove `link(.., Word::ForgeView, ..)` from
   `welcome`; `ForgeView` stays for what else reads it, gone from `words.rs` if nothing does.
3. **`forge_status`:**

   ```rust
   /// The Forge step's line in the column (client/README.md, 3.5, The Forge step with a forge).
   fn forge_status(b: &mut Builder, root: &Domain, w: &Words, row: &Row);
   ```

   A `Role::Slot` styled `look::WELCOME_FORGE_STATUS`, plus `look::FLOW_DONE` connected; a paragraph named
   by its words, "FORGE · line", holding in the browser the spinner (`WELCOME_SPIN`, a textless `Role::Slot`,
   aria-hidden as the look's are) or the tick, the name strong, " · ", the line in `FLOW_MUTED`; in the
   terminal one run, "✓ " first once connected, as `stacked` does. Under way, then `each_step` (not for
   GitHub, which has no steps here). Connected words: `WelcomeForgeDoneDetail`, then
   `WelcomeForgeDoneRepository` when a first repository was made; GitHub's new word `WelcomeForgeGitHubApp`,
   "temper’s App on {name}".
4. **The lead.** `pages/welcome.rs`'s Forge arm: `WelcomeForgeLeadKept`, "One forge is enough to begin; add
   others later, from Settings.", when `status_shown` holds for the personal row; else `WelcomeForgeLead`.
5. **The look,** under `/* ==== ui: welcome ==== */`, from `mock.css`: `.forge-status` (and `.done`, `> p`,
   `b`, `.sep`), `.spin` and its keyframes if it has none yet, `.ok`, `.acts`, with `--gf-*` tokens. New names
   per `notes/contracts.md`, in `look.rs`'s list and `look::NAMES`. Reuse `.temper-welcome-status`'s "Each
   step" rules where they read the disclosure; scope a copy if they don't.
6. **Tests,** in `temper-client-view/src/tests/welcome.rs`, browser and terminal: under way, connected with
   and without a first repository, GitHub accepted; a failed set up, a forge cut short and an unaccepted
   GitHub unchanged; no `View` on any; "Continue" to Model, and to Ready with a usable model. Gallery:
   `welcome-forge-under-way`, `welcome-forge-connected`, `welcome-forge-github`, in `names.txt`, `samples.rs`,
   `tests.rs` and `browser.md`'s 7.2 table; worlds as `welcome-model-forge-*` use, GitHub's from
   `worlds/github.rs`.
7. **Check,** `notes/common.md`; reprint journeys; render proof; commit and push.

## Report

The words used for each state; the goldens moved; the builder and directory of the screenshots.
