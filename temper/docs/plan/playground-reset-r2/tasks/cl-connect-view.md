+++
id = "cl-connect-view"
part = "client"
kind = "feature"
needs = ["cl-connect", "cl-remove"]
files = [
  "temper/crates/temper-client-view/src/forge/{connect,forges,page,words,connect_words}.rs",
  "temper/crates/temper-client-view/src/tests/{connect_forge,forges}.rs",
  "temper/docs/design/client/journeys/welcome-set-up/**",
]
effort = "high"
minutes = 85
+++

## Goal

Connect a forge, the welcome's Forge step and Forges drawn on records
(draft 3, "Its client"; 6.1; 6.4): the choices are kind cards that make a
forge (GitHub; Forgejo on this machine), a kept forge not finished shows
"FORGE · not finished" with Start over and, for GitHub, Go on before it;
Forges lists the kept records with Remove on each row (`cl-remove`'s
control) and always offers "Connect a forge", but nothing to add while
the playground runs, where it lists the playground forge's row alone.

Acceptance (`temper-client-view`, `src/tests/`):

- `connect_forge.rs`: `the_choices_are_cards_for_kinds_to_make` (a fresh
  store: GitHub and Forgejo on this machine, no Elsewhere);
  `githubs_card_connects_a_new_github_without_a_number`;
  `the_welcomes_forgejo_card_sets_up_at_once_without_a_number`;
  `an_unfinished_forge_shows_not_finished_with_start_over`;
  `githubs_unfinished_forge_offers_go_on_before_start_over`;
  `an_underway_set_up_shows_its_progress_as_before`.
- `forges.rs`: `forges_lists_each_kept_record_with_remove`;
  `connect_a_forge_is_always_offered_outside_the_playground`;
  `the_playground_lists_its_forge_and_offers_nothing_to_add`.
- The existing tests of both files pass, rewritten where they named a
  configured slot (each listed in the report).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/tasks/cl-connect.md` and
  `cl-remove.md` (the domain items and the `control` hook, as merged; read
  their reports in the commits' messages if they name changes).
- `temper/docs/design/draft/playground-reset.md`, 3, 4, 6.1, 6.4.
- `temper/docs/design/client/forge.md`, 5.4 (connecting) and the Forges
  page's section.
- `temper/docs/design/client/journeys/welcome-set-up/README.md`.

## Owns and may change

Owns the files in the header. May add `mod connect_words;` in
`temper/crates/temper-client-view/src/forge.rs` (the line only). Keeps
the signatures of `V/forge/connect.rs`'s items that files outside
`V/forge/` call (`welcome`, `app_control`, `setup_control`,
`progress_in`, `setting_up`, `ready_card`, `status`, `first_line`,
`name`, `personal`, `underway`, and any other the compiler names): their
callers (`V/pages/welcome.rs`, `V/frame.rs`) are other tasks' now. A
new parameter there is a forced edit at each caller (scope.md), the
argument only. Calls `crate::forge::remove::control` as `cl-remove`
built it.

## Steps

V = `temper/crates/temper-client-view/src`.

1. **Choose (V/forge/connect.rs `choose`).** One card per
   `connect::choices` entry: `New(GitHub)` the GitHub card (its control
   `app_control` for a new GitHub: `Meaning::ConnectManifest { forge:
   None, .. }` / `ConnectApp(None)`, as `cl-connect` reshaped them);
   `New(Here)` the Forgejo card (`welcome_forgejo` on the welcome,
   `Meaning::SetUpAtOnce`; its Details form on Connect a forge, its set
   up `Meaning::SetUp`); a kept `Forge(n)` as today. The `Offer` note
   (`ForgeCommandsTime`) shows only if `cl-connect` kept an offer fact;
   else the card's note is `ForgeHereTime`.
2. **Details without a row:** `github` and `here` take `Option<&Row>`
   (none: the new forge's form, as today's not-connected row draws it).
3. **Unfinished (`Shown::Unfinished(n)`), on both flows and in
   `welcome`:** a line "FORGE · not finished" (FORGE the row's name), the
   row's failure or stop line when it has one (`failure`,
   `admin_stop`), then for GitHub "Go on" (installing when its App is
   connected and not accepted: `Meaning::Install { forge, back }`; the
   manifest again otherwise: `ConnectManifest { forge: Some(n), back }`)
   before `remove::control(.., row, true)` (Start over); for Forgejo the
   Start over alone. Words in a new `V/forge/connect_words.rs` after
   `V/accounts/words.rs`'s pattern (its own phrase owner).
4. **Forges (V/forge/forges.rs).** The rows `listed::rows` gives (the
   kept records less the playground's); "Connect a forge" always (the
   empty list's paragraph as today); while `root.playground()` is
   `Running`, no Connect link and no add control anywhere on the page,
   the playground section as today (draft 6.1). Each personal row keeps
   `remove::control(.., false)` (surface placed it).
5. **What goes:** `elsewhere` and the document-edit instructions, if
   `cl-connect` left any drawing; the "next number" words.
6. **Checks:** the tests above; `cargo fmt`; `heavy cargo clippy -p
   temper-client-view -p temper-client-gallery --all-targets -- -D
   warnings`; one `heavy cargo nextest run -p temper-client-view -p
   temper-client-shell-web -p temper-client-shell-terminal -p
   temper-client-gallery -p temper-client-world -p
   temper-client-journeys`.
7. **Journeys:** the Forge step's cards change the prints of
   `welcome-set-up` (02.html): reprint (`heavy cargo run -p
   temper-client-journeys`) and commit; its README still says what the
   pages show (if a sentence there is now wrong, change that sentence:
   the journey's README is this task's).
8. Old forms from `temper/`; commit and push.

## Report

- Each test rewritten and why.
- What the code taught for `client/forge.md` 5.4 (choices by kind; the
  unfinished forge; Forgejo elsewhere later).
