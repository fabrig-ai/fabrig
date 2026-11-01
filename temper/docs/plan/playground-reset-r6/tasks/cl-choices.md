+++
id = "cl-choices"
part = "fs"
kind = "feature"
needs = ["cl-wire"]
files = [
  "temper/crates/temper-client-domain/**",
  "temper/crates/temper-client-view/**",
  "temper/tools/gallery/**",
  "temper/tests/client/**",
  "temper/docs/design/client/journeys/*/0*.html",
  "temper/crates/temper-client-shell-web/src/**",
  "temper/crates/temper-client-shell-terminal/src/**",
]
effort = "high"
minutes = 120
+++

## Goal

Connect a forge and the welcome's Forge step offer forges **by kind**, so that a deployment
with no forge record still offers GitHub, Forgejo on this machine and a Forgejo elsewhere
(draft `temper/docs/design/draft/playground-reset.md`, section 3, "Its client"). A choice
whose kind has a kept record that nothing has begun on targets that record, as today; a
choice with none sends `make_forge` first and, on `made { forge }`, the existing `SetUp`,
`Connect` or Details for that number.

- With zero rows, Choose draws GitHub, Forgejo on this machine and A Forgejo elsewhere, in
  that order; the welcome's Forge step draws GitHub and its one Forgejo card.
- "Run on this machine" (welcome, no record) sends `make_forge { owned }`, then on
  `made { forge: N }` the set up at once of N (`work/hello` into `Default`), and the
  welcome moves to Model as it does today on `setting_up`.
- Connect a forge's Forgejo on this machine with no record: Details' set-up form, its
  "Set up" sending `make_forge { owned }` then `set_up` of N with the form's values; the
  address then names `Forge(N)` (replaced).
- "Connect GitHub" with no record sends `make_forge { github }`, then the manifest's
  `Connect { N, github_manifest { back } }` it sends today.
- A Forgejo elsewhere's Details is a form (API address, git address, optional web address)
  whose "Add this Forgejo" sends `make_forge { forgejo { api, git, web } }`, then opens
  `ConnectForge { Forge(N) }`, whose Details are that row's (its admin form when the row
  says `admin`). The configuration snippets the Details showed go.
- `refused: full` and `not_offered` show on the card or form that sent them; nothing more
  is sent. One make at a time.
- With the committed journeys' recorded rows (forges 2 and 3 offered, nothing begun), every
  card and its act are what they were: `temper-client-journeys`' test passes with no
  recording changed.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T4 (MakeForge
  and its answer), T8; `notes/cl-wire.md` 1 to 3.
- `temper/docs/design/client/forge.md`, 5.4 ("Connecting a forge", Choose and Details);
  `temper/docs/design/client/README.md`, 3.5 (the welcome's Forge step), 4.6.
- `temper/docs/design/client/journeys.md`, 1 and 8 (what stales a journey).
- The draft, section 3.

## Owns and may change

Owned: `temper/crates/temper-client-domain/` and `temper/crates/temper-client-view/`
(src and tests), `temper/tools/gallery/`.

May change: the shells (`temper-client-shell-web`, `temper-client-shell-terminal`) only
where the compiler requires for the new `Choice` variants and acts; the joined client world (`temper/tests/client/`) where the compiler requires or an expectation the change moves (each listed in the report); the journeys' printed
pages under `temper/docs/design/client/journeys/` only by the reprint (step 7), never a
recording.

Not yours: `temper-client-tools` (cl-ctl's, in parallel): if the compiler forces an edit
there, make only that edit and list it.

Options, pre-approved:

- A kept record that "nothing has begun on" is a non-fake, non-playground row whose
  connection is `NotSetUp` or `NotConnected` with no `first`, no `app`, no installations
  and no repositories. If the code reads better with a predicate of another name, keep this
  meaning.
- If a `made` answer arrives after the person left the page that sent it, the follow-up is
  still sent (the record is the person's choice) and no navigation is pushed.
- If the welcome's Forgejo card can't tell whether the machine offers its Forgejo (no
  record, so no `offer`), draw it as offered ("Here · about a minute") and let a
  `not_offered` answer say so on the card.

## Steps

1. **Address** (`address.rs`): `Choice` gains `GitHub` and `Here`, printed and parsed as
   `forge=github` and `forge=here` (on `/forges/connect` and `/welcome?at=forge`); a number
   and `elsewhere` as today. Tests in `tests/address.rs`:
   `the_kind_choices_print_and_parse_on_both_flows`.
2. **Cards** (`forge/connect.rs`): `pub enum Card { GitHub { forge: Option<u16> }, Here {
   forge: Option<u16> }, Admin { forge: u16 }, Elsewhere }` and `pub fn cards(rows: &State,
   demo: Option<&Demo>) -> List<Card>` replacing `choices`: GitHub's card targets the first
   GitHub row offered as today (not connected or stopped), and with no GitHub row at all is
   `forge: None`; Here's targets the first owned Forgejo row `NotSetUp` whose offer is not
   `NotOffered`, and with no owned row is `None`; a row of either kind in any other state
   draws no card of that kind; `Admin` as today's admin cards; `Elsewhere` always last.
   `step` and `welcome_step` read `Choice::GitHub`/`Here` as `Forge(N)` when the card has a
   target, else: `GitHub` → `Choose`, `Here` → `Details(Choice::Here)`.
3. **Making** (new `forge/making.rs`), the attempt kept on `Side` across pages:

   ```rust
   pub enum MakeChoice { Here, GitHub, Forgejo { api: Box<[u8]>, git: Box<[u8]>, web: Option<Box<[u8]>> } }
   pub enum Then { SetUp, SetUpAtOnce, ConnectManifest { back: Box<[u8]> }, Open }
   pub enum MakeAnswer { Made { forge: u16 }, Full, NotOffered, Unknown, EntranceRefused(c::EntranceRefusal) }
   pub struct Making { pub choice: MakeChoice, pub then: Then, pub request: Token,
       pub progress: Option<RequestProgress>, pub answer: Option<MakeAnswer> }
   pub struct Draft { pub api: Box<[u8]>, pub git: Box<[u8]>, pub web: Box<[u8]>, pub written: [u64; 3], pub checked: bool }
   pub enum Field { Api, Git, Web }
   ```

   `ask(choice, limits) -> Option<Ask>` (deployment scope, `w::Request::MakeForge`) and
   `answer(&Answered, limits) -> Option<MakeAnswer>` in `forge/family.rs`'s style; told
   handling in `making::told` called from `route::forge_forges` beside the others: pending
   progress as `forges::told` keeps a set up's; on `Made { forge }` run `then` for `forge`
   (`SetUp`: `forges::set_up` with the drafted values for N; `SetUpAtOnce`:
   `forges::set_up_at_once`; `ConnectManifest`: `github::manifest` for N with `back`;
   `Open`: nothing) and, when the address is the choice's (`ConnectForge { GitHub | Here |
   Elsewhere }` or the welcome's Forge step at it), `Request::Show` of the same place at
   `Forge(N)`, `replace: true`. New `forge::Act` variants: `MakeAndSetUp`,
   `MakeAndSetUpAtOnce`, `MakeAndConnectGitHub { back: Box<[u8]> }`, `ElsewhereWrite {
   field: making::Field, text: Box<[u8]> }`, `MakeElsewhere` (empty api or git marks the
   form `checked` and sends nothing). Accessors on `Side`: `making()`, `elsewhere_form()`.
   Signed out clears both, as `set_up` is cleared.
4. **Domain tests** (`src/tests/making.rs`, with `tests/connect.rs` for cards):
   `with_no_rows_the_cards_are_github_here_and_elsewhere`,
   `a_kept_record_nothing_begun_on_is_its_cards_target`,
   `a_begun_record_of_a_kind_draws_no_card_of_that_kind`,
   `here_with_no_record_makes_then_sets_up_the_made_number`,
   `the_welcomes_run_here_makes_then_sets_up_at_once_and_moves_to_model`,
   `github_with_no_record_makes_then_sends_its_manifest_connect`,
   `elsewhere_makes_a_forgejo_from_its_fields_and_opens_its_number`,
   `an_empty_elsewhere_address_sends_nothing`,
   `full_and_not_offered_are_kept_and_send_nothing_more`,
   `one_make_at_a_time`. `heavy cargo nextest run -p temper-client-domain --lib`.
5. **View** (`forge/connect.rs`, `forge/bindings.rs`, `words.rs`): `choose` draws from
   `cards`; a card with a target draws exactly what it draws today (words, controls and
   their acts); with none: GitHub's "Connect GitHub" binds `MakeAndConnectGitHub { back }`
   (the back address today's), the welcome's "Run on this machine" binds
   `MakeAndSetUpAtOnce`, Connect's Forgejo on this machine "Choose" links to
   `ConnectForge { Here }`, whose Details are today's set-up form with "Set up" bound to
   `MakeAndSetUp`. While a make is in flight its control is disabled; a refusal shows
   under it: full, "This deployment keeps as many forges as it may: remove one first.";
   not offered, "Not offered here: this deployment runs no Forgejo of temper's." for here,
   "Not offered on this deployment." otherwise. Elsewhere's Details: heading "A Forgejo
   elsewhere", "Your own Forgejo server. temper connects as its site admin, makes its bot
   and your first repository.", fields "API address" (hint "HOST:PORT, where temper reaches
   its API"), "Git address" (hint "The URL git clones from"), "Web address" (hint "Its URL
   in a browser, when not the git address"), and the button "Add this Forgejo"; its card's
   muted line becomes "Its admin's token · a few minutes". Remove the Words the snippets
   used (`ForgeConfig`, `ForgeConfigCode`, `ForgeIdentity`, `ForgeIdentityCode`,
   `ForgeSetupEntry`, `ForgeSetupCode`, `ForgeBotOnly`, `ForgeRestart`, and
   `connect::next_number` if nothing else reads it).
   View tests (`tests/connect_forge.rs`, `tests/welcome.rs`):
   `choose_with_no_rows_draws_github_here_and_elsewhere_in_order`,
   `the_welcomes_forge_step_with_no_rows_makes_on_run_here`,
   `here_details_with_no_record_set_up_by_making`,
   `elsewhere_details_draw_the_address_form_and_send_its_make`,
   `a_make_refusal_shows_under_its_control`,
   `a_targeted_card_draws_as_before` (rows 2 and 3 of `connect_forge::rows`: the same nodes
   and acts as `choose_draws_a_card_per_choice_in_order` expects).
   `heavy cargo nextest run -p temper-client-view --lib`.
6. **Gallery:** a state `connect-forge-no-records` (Choose with zero rows) and
   `connect-forge-elsewhere` (its form), in `tools/gallery/src/worlds.rs` and
   `names.txt`, as `connect-forge-choose` is made. `heavy cargo nextest run -p
   temper-client-gallery --lib`.
7. **The journeys:** `heavy cargo nextest run -p temper-client-journeys --lib`. It must not
   report a journey stale: if it does, a card's act or an ask changed for the recorded rows,
   which this task must not do; fix that. If it reports only a page's bytes differing,
   reprint with `heavy cargo run -p temper-client-journeys` and commit the reprinted pages
   (no `recording` changed), saying which in the report.
8. Then: `heavy cargo nextest run -p temper-client-shell-web --lib`, `-p
   temper-client-shell-terminal --lib`, `-p temper-client-world --test suite` (the joined
   world reads the domain), `-p temper-client-tools --lib` (it reads the domain). `cargo
   fmt`; `heavy cargo clippy` of each changed package `--tests`, and `heavy cargo clippy -p
   temper-client-domain -p temper-client-view --target wasm32-unknown-unknown`. Old forms;
   commit and push.

## Report

- Whether the journeys needed a reprint, and which pages.
- Every file edited outside the domain, view and gallery, and why.
- Under `## Left`: `temper/docs/design/client/forge.md` 5.4's words for Choose, Details and
  a Forgejo elsewhere that this changes.
