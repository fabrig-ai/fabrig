+++
id = "picker-menu"
part = "picker"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/pages/choice.rs",
  "temper/crates/temper-client-domain/src/tests/pages/choice.rs",
  "temper/crates/temper-client-view/src/choice.rs",
  "temper/crates/temper-client-view/src/choice/picker.rs",
  "temper/crates/temper-client-view/src/pages/session.rs",
  "temper/crates/temper-client-view/src/pages/new_chat.rs",
  "temper/crates/temper-client-view/src/pages/configure.rs",
  "temper/crates/temper-client-view/src/tests/choice.rs",
  "temper/crates/temper-client-view/src/tests/configure.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/**",
]
effort = "high"
minutes = 240
+++

# The model and effort at the composer: the view, the stylesheet and the draft behind them

## Goal

Land the look request `model-picker`'s pages 01 to 06 and its states in words, in the view and `temper.css`, on every chat,
and remove what it replaces, rules and view both. Every commit carries `Look-request: model-picker`.

Acceptance:

1. **Closed:** on the hint's row, at the composer's right edge, on a new chat, Configure temper, a chat and a configuring
   chat, a disclosure named "Model" described by the closed line: "Claude Opus 5.5 · high", or "GPT-6.1 Sol · high".
2. **Open:** a heading per account, models by shown name, a tick on the chosen, "default" muted, a connect or fix line for a
   provider with no model, "Effort" following the chosen model and absent where it declares none, "All models" and how
   many where the picker shows fewer than the offer.
3. **All models:** "‹ Models", "Find a model" filtering, every offered model by account, "Accounts, in Settings" linking to
   `Accounts`, no "new".
4. **A started chat:** "A change applies from the next turn."; one press sends one amendment, with no "Apply"; "Changing…"
   while sent; refused, why, and the choice is the chat's own again.
5. **The other states:** two accounts of one provider, a heading for each and a press preferring its account; no model yet,
   5.5's words above the composer and no picker; a choice that can't be changed, the same words, not a button; an account
   stopped being usable, its fix and no model.
6. **Removed:** the message's "What it replaces" list, each item, view and rules, and a chat's bar keeps "T2" and its status.
7. **The terminal:** the print holds "Model: LINE" with the list as lines and "Effort:" with its efforts beneath; headings
   and order keep their meaning.
8. **The gallery** holds `temper/docs/design/client/browser.md`, 7's ten `chat-model-*`, `chat-started-model-*` and
   `configure-model-open` names, by those names.
9. Focused tests of the packages below are green but for `temper-client-journeys`, whose prints are `picker-proof`'s.

## Inputs

- `temper/docs/plan/look-round-11/notes/common.md`, first; the request's message and pages, as it says how.
- `temper/docs/design/client/README.md`, 3.5, *The work's account and model*, *A chat's bar*; 5.5, *The composer's line*; 5.9.
- `temper/docs/design/draft/model-catalogue.md`, 3.
- `jig/docs/design/client/domain/contract.md`, 5.1: `MeModel`, `resolved_default`.
- `temper/docs/plan/model-catalogue-1/plan.md`, *For `fabrig/ui`'s landing of `model-picker`*.

## Owns and may change

Owns, as `files` lists them.

May also change, each for what this names:

- `temper/crates/temper-client-view/src/pages/chat.rs`: the `choice::byline` line becomes `session::number`; the `hint` call
  its new name. Small hunks: `ui-proposal-card` is in this file.
- `temper/crates/temper-client-view/src/{look.rs,words.rs,limits.rs}`: classes, words, node bounds and their comment.
- `temper/crates/temper-client-view/src/pages/welcome_model.rs`: nothing unless `new_chat_model`'s reader needs it.
- `temper/crates/temper-client-domain/src/pages/chats.rs`: making `configuring` reachable from `choice.rs`, if it isn't.
- `temper/crates/temper-client-domain/src/{pages.rs,acts.rs}`, `pages/chat.rs`, `pages/task.rs`: arms for `choice::Act`, if any
  name its variants.
- Reader tests, changing the words and the walk and keeping what each checks:
  `temper/crates/temper-client-view/src/tests/{chat,delegation,home,frame,playground,fixtures}.rs`,
  `temper/crates/temper-client-shell-terminal/src/tests/**` (goldens by their tool: `first_run.txt` prints "Change model"),
  `temper/tests/client/tests/suite/{choice,configuring_chat}.rs`, `temper/tests/client/tests/live_text.rs` ("Reset choice" becomes
  the row marked "default"), `temper/tests/client/src/{component,fixtures}.rs`.
- `temper/tests/client-browser/tests/browser_all/browser_pages.rs`: `byline_steps` to the new name and `details.temper-model`,
  compiled, never run.
- `temper/docs/design/client/README.status.md`: remove the **Not yet** on *The work's account and model* when this lands.

Options, each pre-approved:

- **`.temper-choice`'s rules:** `above_composer` and an agent's composer keep it. Remove `.temper-rows` and `.temper-row`
  under it, and `.temper-choice > button`, if nothing draws them once `panel` goes; if something does, keep them and say so.
- **An agent's task's page** (`pages/agent.rs`, `pages/task.rs`, through `choice::composer`) draws the same picker beside its
  line. Its class isn't `.temper-session`: widen the rules' prefix to `#app > :is(.temper-session, <its class>)` rather
  than copying them, and say so. If the absolute menu can't work there, give it `position: static` on that page alone.
- **A provider with nothing to connect,** the playground's where it isn't running: no line. Today's test is
  `playground_provider`, which is none until its answer is read; if `me.providers` then shows it, also pass over a
  provider `accounts::ways` gives no way for.
- **Enter in "Find a model"** sends nothing. If the composer's form would submit, stop it in the view; if only skein could,
  say so in the report and `picker-proof` checks it.
- **A shown name a test pins:** a closed line changes `new_chat_model`, so Ready's line follows, as its comment says. If
  that breaks a pinned Ready print, update the print.

Leave: `temper/docs/design/client/journeys/**` (`picker-proof`), `overlay.css`, what `notes/common.md` says isn't landed, and
every jig crate: `me` already gives all this needs.

## Steps

### The draft, `temper-client-domain/src/pages/choice.rs`

```rust
pub enum Context {
    Template { project: u32, template: u32, executor: c::TemplateExecutor, model_index: Option<u8> },
    /// `template` is the chat's: the configuring template for a configuring chat, else `CHAT`.
    Task { project: u32, task: u64, template: u32, revision: c::Revision, model_index: Option<u8> },
}
pub struct Draft {
    pub context: Option<Context>,
    pub original: Option<c::LlmChoice>,
    pub requested: Option<c::LlmChoice>,
    pub pending: Option<Key>,
    pub outcome: Option<Outcome>,
    /// The menu is on "All models".
    pub all: bool,
    /// "Find a model"'s words, at most `FIND` bytes.
    pub find: Box<[u8]>,
}
pub enum Act {
    /// A row's press. `account` is the heading's, where its provider has two; on a task it sends at once.
    Model { index: u32, account: Option<u32> },
    Effort { index: u32 },
    All,
    Back,
    Find { words: Box<[u8]> },
}
pub const FIND: u32 = 256;
impl Draft {
    /// The request, else the template's `resolved_default`: what the picker says is chosen.
    pub fn chosen<'a>(&'a self, me: &'a c::Me) -> Option<&'a c::LlmChoice>;
    /// The row `chosen` names, by `model_index` where a person picked one.
    pub fn selected(&self, me: &c::Me) -> Option<u32>;
    /// The row `resolved_default` names.
    pub fn default_row(&self, me: &c::Me) -> Option<u32>;
    /// Whether "Find a model"'s words match the row, in any case, anywhere in its shown name or name.
    pub fn finds(&self, model: &c::MeModel) -> bool;
}
```

- **`Act::Default` and `Act::Apply` go.** `Model` and `Effort` edit the draft, and on a `Context::Task` then call `amend`,
  which sends only a change it can (`valid`, and differing from `original`).
- **A press of `default_row`** returns `requested` to `original`, unselected, where `original` names that model or none;
  otherwise it is as any press. So a new chat can go back to no choice, and a started chat with none sends nothing. A
  press with a different `account` under a provider's second heading isn't the default row's press.
- **A press** sets `choice.account` to the act's, none where its provider has one account, keeping `subagents`.
- **`Effort` with no request** starts one from `chosen`'s model.
- **`told`:** a refusal puts `requested` back to `original`, unselected, keeping `outcome` for the menu. So does a
  `Propose`'s answer: the change isn't the chat's until it's decided. An amendment answered `Done` keeps `requested` until the
  document moves it.
- **`reset`** clears `all` and `find` only when the task or template differs, not on a revision: the menu holds across the
  document's moves.
- **`refresh`** fills `Context::Task.template` from `chats::configuring`.

### The view

`temper-client-view/src/choice/picker.rs`, a `mod picker;` in `choice.rs`, its module comment citing
`client/README.md`, 3.5:

```rust
/// The disclosure on the composer's row, or the same words where the choice can't change, or nothing with no model.
pub(crate) fn picker(b: &mut Builder, root: &Domain, task: Option<&c::Task>, w: &Words, bindings: &mut Bindings);
/// "Claude Opus 5.5 · high": the chosen model, where the offer holds it or a task does.
pub(crate) fn closed_line(root: &Domain, task: Option<&c::Task>, w: &Words) -> Box<[u8]>;
fn menu(...);        // the note or Changing… or why; headings and rows; connect lines; effort; All models
fn all_models(...);  // back; the field; every row `finds` holds; the link
fn heading(...);     // provider, what pays
fn row(...);         // the button; tag
fn effort(...);      // the group
```

`choice.rs` loses `byline`, `panel`, `model_button`, `provider_line` (into `picker.rs`, its fix's logic kept), `LineForm`, the
`Byline*` mapping in `selected_line`, the `ModelNeedsAccount` branch of `using_words` (which says "Using MODEL") and
`usage_link`'s use outside `using`. `offered` becomes `changeable`: as today but true while `draft.pending` is set, so that the
menu stays open with "Changing…". `feedback` loses the draft's pending and outcome, which are the menu's. `composer` is
`using`, then `picker`, then its waits. `new_chat_model` returns `closed_line`'s words.

`pages/session.rs`:

```rust
/// Closes the entry and draws its row: the hint and the picker (client/README.md, 3.5).
pub(crate) fn foot_row(b: &mut Builder, root: &Domain, task: Option<&c::Task>, w: &Words, bindings: &mut Bindings, hint: Word);
/// The chat's number in the bar, where the browser shows the title alone (client/README.md, 3.5).
pub(crate) fn number(b: &mut Builder, w: &Words, task: u64);
```

`foot_row` replaces `hint`, opening `look::FOOT_ROW` with the hint's paragraph and `picker`. `number` is a `look::BYLINE`
paragraph of `Word::Chat`, on `Shell::Browser` alone. `new_chat.rs` and `configure.rs` drop their bylines; `configure.rs`'s `byline` fn
goes, and its `session::right` stays, for its modes.

The nodes, as the pages have them. The disclosure is `Role::Disclosure`, named "Model" and described by `closed_line`, with
`State::FoldsAway`; its body is one `look::MODEL_MENU` slot. A heading is a slot, not a paragraph, since paragraphs flatten
controls in the terminal: `Run::Strong` the provider, then the clause or the link. Rows are `Role::List`/`Role::Item`/`Role::Button`,
the button named by the shown name alone with `State::Pressed` on the chosen, its "default" a child in `look::MODEL_TAG`, so
the fake person finds "Opus 5.5" and a browser shows both. In the terminal a row's line reads "Opus 5.5 · default". "Effort" is
`Role::Group`, its word then a slot of buttons. A choice that can't change is a paragraph in `look::MODEL` and
`look::MODEL_FIXED`.

- **Headings:** an account's provider word is `anthropic` → "Claude", else `provider_words`'s brand. What pays reads the
  accounts document: a program source or an `anthropic` plan, "your subscription"; another provider's plan, "your PLAN plan";
  a key, "your API key"; an account it doesn't hold, nothing. Two of one provider reading alike say `display`. Accounts come in
  the offer's order, `me.providers` then giving the connect and fix lines where it has none.
- **The closed line:** `w.say(ModelEffort, ...)` of the name and effort, with `shown`'s name; "Claude " before a shown name
  where the provider is `anthropic`. `shown` is `w.model_name` for the playground's, and that model has no effort. No chosen
  model at all: `Word::DefaultModel`.
- **Rows use `Act::Model`'s `account`** only where the row's provider has more than one account in the offer.
- **The default tag** goes on `default_row` under its first account alone. The tick goes on `selected`, under the choice's
  account, or else its model's first.

`look.rs`: `FOOT_ROW` `temper-foot-row`, `MODEL` `temper-model`, `MODEL_FIXED` `temper-model-fixed`, `MODEL_MENU`,
`MODEL_TAG`, `MODEL_FROM`, `MODEL_NOTE`, `MODEL_EFFORT`, `MODEL_ALL`, `MODEL_BACK`, `MODEL_FIND`, `MODEL_LONG`, by the overlay's
names. `BYLINE` stays.

`words.rs`, new: "Model"; "default"; "Effort"; "All models"; "‹ Models"; "Find a model"; "Accounts, in Settings"; "A change
applies from the next turn."; "Changing…"; "your subscription"; "your ", " plan"; "your API key". Gone: `ChangeModel`, `BylineTask`,
`BylineModel`, `BylineKey`, `BylinePlan`, `BylineProgram`, `BylinePlayground`, `ModelNeedsAccount`, `ChoiceReset`, `ChoiceApply`, `ChoiceDraft`,
`ChoiceDraftDefault`, `ChoicePending`, `ChoiceProvider`.

### The stylesheet

Under `/* ==== ui: chats ==== */`, the overlay's `model-picker` rules, its `@media (max-width: 720px)` with them; not `/* Settings'
list of models. */`'s. Besides them:

- `summary`'s name hides as the byline's did: `font-size: 0` on `.temper-model > summary`, its `> span` at 12.5 px.
- `.temper-model-fixed`: the summary's padding, size and colour, with no chevron.
- `.temper-model-all` is a button here, a link on the page: no border or background, the row's whole width.

Remove: `.temper-session-foot`'s `overflow: hidden` and `scrollbar-gutter`, the phone's `scrollbar-gutter: auto`; every `details.temper-byline`
rule; rules selecting `.temper-session-foot > .temper-session-hint` move to the row's child, keeping `.temper-hint`'s. The
`.temper-byline` paragraph's rule stays, for "T2".

### The tests

In `temper-client-view/src/tests/choice.rs`, reading the tree and the print, as its neighbours:

- `a_new_chats_picker_says_what_create_would_use`; `configure_temper_says_its_own_default`.
- `a_heading_says_what_pays`, with a subscription, a plan, a key and an account the document doesn't hold.
- `a_provider_with_no_account_is_one_line_and_no_model`; `an_account_stopped_being_usable_keeps_its_fix_and_no_model`.
- `two_accounts_of_one_provider_head_each_and_a_press_prefers_its_account`.
- `efforts_follow_the_chosen_model_and_none_has_no_row`.
- `all_models_shows_where_the_picker_shows_fewer`, `all_models_finds_by_shown_name_and_name`,
  `all_models_links_to_accounts_and_says_no_new`.
- `a_started_chat_says_when_a_change_applies`, `changing_is_said_in_the_menu`, `a_refusal_says_why_in_the_menu`.
- `a_choice_that_cant_change_is_words_not_a_button`; `no_model_yet_has_no_picker`.
- `a_chats_bar_keeps_its_number_and_no_account`.
- `no_model_of_a_provider_with_no_account_shows`: one account, closed and open and on All models.
- `the_terminal_prints_the_list_as_lines`.

In `temper-client-domain/src/tests/pages/choice.rs`: `one_press_on_a_task_sends_one_amendment`, `the_same_press_sends_nothing`,
`the_default_row_goes_back_to_no_choice`, `effort_with_no_request_starts_from_the_default`, `a_refusal_returns_the_chats_own_choice`,
`a_proposal_returns_the_chats_own_choice`, `a_second_press_while_changing_does_nothing`, `the_menu_holds_across_a_revision`.

### The checks

`cargo fmt`; `heavy cargo clippy -p temper-client-domain -p temper-client-view -p temper-client-shell-terminal -p temper-client-gallery
-p temper-client-world -p temper-client-browser-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-domain -p
temper-client-view -p temper-client-shell-terminal -p temper-client-shell-web -p temper-client-gallery -p temper-client-world`. This changes what
`choice::Draft` and `session`'s row mean, so those are its readers. `heavy cargo check -p temper-client-browser-world --tests`. Then
`python3 -I -B temper/scripts/old_forms.py`, the citations, push.

`temper-client-journeys`' tests stay red on this branch, on `chat-draft` and `welcome-playground`: `picker-proof` reprints them.
Say so under `## Left`.

## Report

What `notes/common.md`, Your report, asks, and each rule that had to differ from the overlay, with its reason.
