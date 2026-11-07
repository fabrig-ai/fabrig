+++
id = "models-tab"
part = "page"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-view/src/pages/models.rs", "temper/crates/temper-client-view/src/tests/models.rs", "temper/crates/temper-client-domain/src/address.rs", "temper/crates/temper-client-domain/src/tests/address.rs"]
effort = "high"
minutes = 90
+++

# Settings has a Models tab, drawn from `me`

## Goal

The tab between Accounts and Builders, read-only: a section per provider saying what pays, a row per offered model with its name, default effort, whether it's the default and "On" or "Off", the provider's "Connect" line or its fix where no model is offered, and the line to Configure temper. It reads what `me` already gives, so it needs no other task and changes no jig crate. The switch and "Add a model by name" are later tasks of this part, on the settings this plan adds.

Acceptance:

- `/projects/1/settings/models` opens `Address::Models { project: 1 }` and round-trips; Settings' tabs read How work runs, Repositories, Forges, Accounts, Models, Builders.
- With two accounts, two sections and every offered model; with a Claude subscription alone, "OpenAI" has "Connect a plan or key ›" and no row: no name of an OpenAI model is on the page.
- An account that stopped being usable has its own fix and no row.
- A row reads "claude-opus-5-5 · high · the default" on the chat template's `resolved_default`, and "claude-sonnet-5-5 · high" elsewhere; a model that declares no efforts reads its name alone.
- The terminal's print holds the same words, in the same order.
- New gallery states `models`, `models-one-account`; focused tests pass.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *Settings*: its table of tabs, *Its tabs*, *The `Models` tab* (its rows' words; the switch, "added by you", "estimated prices", left-out and Add are not this task's), *The rows*, *Builders*.
- `temper/docs/design/draft/model-catalogue.md`, 3.
- The look: `git show 47c23ff6ed:temper/docs/design/client/journeys/model-picker/07.html`, and the `/* Settings' list of models. */` rules of `git show 47c23ff6ed:temper/docs/design/client/journeys/overlay.css`, which you land under `/* ==== ui: settings ==== */` for `.temper-models-source` and `.temper-models-ask`; `.temper-models-add` is `models-add`'s. Its "Read again", "Read from OpenAI 2 hours ago", "new" and "Couldn't read" wait for step 3: draw none.
- `temper/docs/plan/model-catalogue-2/notes/common.md`.

## Owns and may change

Owns `temper-client-view/src/pages/models.rs` and its test file, both new.

May change, and how:

- `temper-client-domain/src/address.rs`, `tests/address.rs`: the new variant, path and rows, beside `Builders`.
- `temper-client-domain/src/pages.rs`, `temper-client-view/src/frame.rs`, and any other file with a match over `Address`: the arm beside each `Address::Builders { .. }`, the same but `Models`. The project switcher keeps the tab (`Show { address: Address::Models { project } }`).
- `temper-client-view/src/pages/settings.rs`: `Tab::Models`, its heading and lede, its place in the tabs' array and arms.
- `temper-client-view/src/pages.rs`, `lib.rs`: the `mod` line, and the page's dispatch beside `Builders`'.
- `temper-client-view/src/words.rs`: its words, under names beginning `Models`, numbered as the file numbers them.
- `temper-client-view/src/choice.rs`: `provider_line` and `provider_shown` become `pub(crate)` and nothing else. If `look-round-11` has moved them when you start, call them where they are.
- `temper-client-shell-web/temper.css`, under the marker.
- `temper/tools/gallery/`: `names.txt`, `src/samples.rs`, `src/tests.rs`, `src/worlds.rs` for the two states; its `Holds` beside `builders`'. `temper-client-shell-terminal/src/tests/render.rs`: a row beside `builders`.

Options, each pre-approved:

- `me` has no project yet, or no row for this one: draw the frame's `settings::missing`, as Builders does.
- The accounts document isn't read yet, or the person's accounts aren't in it: the section's first clause goes, leaving "These come with temper."
- `All models`' link stays as it is: it moves when `look-round-11` is on main (`models-close`).

## Steps

`pages/models.rs`, in the shape of `pages/builders.rs`:

```rust
//! Settings' Models tab (temper/docs/design/client/README.md, 3.5, The `Models` tab).
pub(crate) fn main(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32);
fn section(b, root, w, bindings, provider: &c::ProviderOffer, models: &[&c::MeModel], default: Option<&[u8]>);
fn pays(root: &Domain, provider: u32, w: &Words) -> Box<[u8]>;   // "Your Claude subscription." and so on, one clause per usable account
fn row(b, w, model: &c::MeModel, is_default: bool);
fn ask(b, w, bindings, project: u32);                             // the last line
```

It reads `pages::choice::offer(me, project)` for the models, each one's `provider` for its section, and `me.providers` for the sections' order; `MeDefault` of the chat template (template 1) for "the default". `pays` reads `pages::accounts::document`: a program source or an `anthropic` plan is a subscription, another provider's plan its "ChatGPT plan" as `accounts::words::provider_words` names it, a key "Your API key." The row's "On" or "Off" is `model.picker`, plain words.

Tests in `tests/models.rs` through the view's existing fixtures, each breaking its rule once by hand:

- `tabs_put_models_between_accounts_and_builders`
- `a_row_says_name_effort_and_the_default`
- `a_model_with_no_efforts_says_its_name_alone`
- `no_model_of_a_provider_with_no_account_shows`
- `an_unusable_account_keeps_its_fix_and_no_row`
- `each_section_says_what_pays`

`tests/address.rs`: rows for `Models { project: 0 }`, `3` and `u32::MAX`, as `Builders`' are.

`cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain -p temper-client-view -p temper-client-shell-terminal -p temper-gallery --all-targets -- -D warnings` (`rg -m1 '^name' temper/tools/gallery/Cargo.toml` for the gallery's name); `heavy cargo nextest run` on the same packages. Regenerate what the gallery and the terminal's prints make stale. Commit and push.

## Report

The rules each test broke and what failed; screenshots aren't this task's (`models-proof`). Whether `look-round-11` had landed when you started.
