+++
id = "client-names"
part = "client"
kind = "migration"
needs = []
files = ["temper/crates/temper-client-domain/**", "temper/crates/temper-client-view/**"]
effort = "medium"
minutes = 150
+++

# client-names: the client's code says Ask temper

## Goal

The client already draws Ask temper where the configuring chat was (commit
`2cfda766be`): its words say "Ask temper", it starts template 5, and `Address::Ask`
replaced `Address::Configure`. Its code still names the old chat: the view's
`pages::configure` module, `Word::Configure*`, `configuring()` helpers and locals,
comments citing `Configure`, and test fixtures that keep a "former" template or charter
3. This task renames them and removes those fixtures, changing no word a person reads
and no behaviour.

Acceptance:

- `rg -n -i 'configur(e|ing)' temper/crates/temper-client-domain temper/crates/temper-client-view`
  finds no name, comment or test name for the old chat, Configure or Configure temper:
  what remains is configuring as an act on settings ("configured", `configure(&mut self, ..)`
  page setup methods, a negative assertion that old words are absent) and the
  negative route test `tests::address::a_path_that_is_no_page_is_missing`'s
  `/projects/3/settings/configure`, which stays.
- No test fixture in these crates holds a template or charter 3 standing for the
  configuring chat; where one discriminated Ask temper from another conversation
  template, a custom conversation template of the fixture's own does it, numbered 7 and run by
  a charter of its own numbered 38.
- Every rendered word, gallery print and terminal golden is unchanged: the gallery
  (`heavy cargo nextest run -p temper-client-gallery --lib`) and the terminal goldens
  (`heavy cargo nextest run -p temper-client-shell-terminal --lib -E 'test(/render/)'`)
  pass with no regeneration; a words declaration the renames regenerate (the view's
  ALL and English declarations, by their own test) changes in names alone.
- `heavy cargo nextest run -p temper-client-domain -p temper-client-view --lib` passes
  with the base's count.

## Inputs

- `temper/docs/plan/ui-ask-drop/notes/common.md` (its *Words*) and `notes/contracts.md`
  (*Words*).
- `temper/docs/design/draft/ask-temper.md`, section 9.
- The commit `2cfda766be`'s message (`git log -1 2cfda766be`): what the client landed.

## Owns and may change

- Owns `temper/crates/temper-client-domain/**` and `temper/crates/temper-client-view/**`.
- Reader files, call sites only: `temper/tools/gallery/src/**` and
  `temper/crates/temper-client-shell-*/src/**` where a renamed public item is named
  (`pages::configure::..`, `Word::Configure*`, a fixture helper); a gallery **sample
  name** (`configure-*` in `names.txt`) is not renamed here: the task `gallery` does it.
- Renames, pre-approved defaults (on a clash with a name that exists, add `Page`:
  `AskPageTitle`):
  - `temper-client-view/src/pages/configure.rs` → `pages/ask.rs`, `pages::ask`;
    `src/tests/configure.rs` → `src/tests/ask.rs`, with its `mod` lines.
  - `Word::Configure*` → `Word::Ask*` (`ConfigureTitle` → `AskTitle`, `ConfigureOwnSee`
    → `AskOwnSee`), `Word::Configuring` → `Word::Asking`, `Word::ModelsConfigure` →
    `Word::ModelsAsk`, `Word::SettingsConfigureHelp` → `Word::SettingsAskHelp`; their
    words, numbers and slots kept.
  - `configuring()` in `pages/chat.rs` → `asking()`; locals and fields named
    `configuring`/`configured` for Ask temper's chat → `asking`; `chats::open_configuring`
    and the like → `open_asking`.
  - Comments: "the configuring chat", "Configure", "Configure temper", "the configuring
    agent" for what is now Ask temper's say "Ask temper" or "Ask temper's page", with
    the same citation; `look.rs`' "configuring opening" → "Ask temper's opening".
  - Test functions and fixture helpers named `configur*` for Ask temper → `ask*`/`asking*`
    (`configuring_me` → `asking_me`, `settings_configure_link_opens_the_delivered_configuring_chat_in_both_shells`
    → `settings_ask_link_opens_the_delivered_asking_chat_in_both_shells`). A test name
    a design or status cites (`rg` it in `temper/docs/design/`) is renamed too and
    listed in the report, for the lead.
  - `tests/fixtures.rs`' custom Ask charter 37 stays (it checks the delivered executor);
    `tests/rail.rs` ~350 and ~489, where template 5 is modelled with charter 3, use
    charter 11.
  - `tests/settings.rs` ~127, which accepts "Configure temper" or "Ask temper", accepts
    "Ask temper" alone.

## Steps

1. Rename module by module, `heavy cargo check -p temper-client-view --tests` while
   errors last, then the gallery and shells' `--tests` checks.
2. Once: `cargo fmt`; `heavy cargo clippy -p temper-client-domain -p temper-client-view -p temper-client-gallery --all-targets -- -D warnings`;
   the acceptance's runs; `python3 -I -B temper/scripts/old_forms.py`. Renames need no
   bite check; `tests/settings.rs`' narrowed assertion does (make the page say
   "Configure temper" by hand, see it fail, restore).
3. Commit ("Name the client's Ask temper code for it") and push.

## Report

The renames (old → new), the fixtures removed or replaced, the generated declarations
that moved (names only), renamed tests a design cites, and the acceptance's `rg` output
with each remaining line's reason.
