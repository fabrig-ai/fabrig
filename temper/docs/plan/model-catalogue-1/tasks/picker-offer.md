+++
id = "picker-offer"
part = "picker-offer"
kind = "feature"
needs = ["offer-surface"]
files = [
  "temper/crates/temper-client-domain/src/pages/choice.rs",
  "temper/crates/temper-client-domain/src/tests/pages/choice.rs",
  "temper/crates/temper-client-view/src/choice.rs",
  "temper/crates/temper-client-view/src/tests/choice.rs",
  "temper/crates/temper-client-view/src/pages/accounts/rows.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/tools/gallery/src/worlds.rs",
]
effort = "high"
minutes = 150
+++

# The chooser draws the offer, and works nothing out

## Goal

Draw the chooser, in today's look, from `me`'s offer: `fabrig/ui` lands the look `model-picker` on these facts. So a
provider with no usable account shows its name and a way to connect, and no model, stock ones included; the line names
what `resolved_default` says, never a guess from the catalogue; and a task given no choice with no account says so
without naming a model.

Acceptance, each a test in `temper-client-view/src/tests/choice.rs` unless it says otherwise:

1. The panel, by `me.providers`: a provider with offered rows, its name, then a button per row with `picker` true, in
   the offer's order, shown by `shown`, else its name. The chosen one's efforts follow, low to high.
2. A provider with no offered row, the person holding no account of it: its name and "Connect a plan or key ›", a link
   to `ConnectModel { way: None }`, and no model.
3. A provider with no offered row where the person holds an account of it needing attention: its name and that
   account's fix, as the accounts page words it ("Sign in again ›"), a link to `Accounts`, and no model. Readiness
   unknown, with no fix to show: `W::AccountReadinessUnknown`, as now.
4. The playground's provider has no line unless the offer holds its rows. "needs an account" (`W::NeedsAccount`) goes.
5. Pressing a model sets its `effort`, so a press leaves a valid draft and a chat can start at once (domain test
   in `tests/pages/choice.rs`). A template's or a task's own choice stays valid offered or not, and a new chat from a
   template naming an unoffered model still sends.
6. A new chat's line, with no choice: `resolved_default` of its template, "MODEL · EFFORT", with catalogue and charter
   orders opposed; with no preview, "Using the default model". Never the catalogue's first.
7. A task or draft holding a model the offer leaves out: "MODEL · needs an account" on its line. A model in the offer
   doesn't read as needing one, whatever `doc.readiness` says for the party, the playground's included.
8. A wait `Account { why: Empty }` with `task.llm` none says "No account can run this chat yet." and names no model,
   with "Connect a model"; with `task.llm` some, "No account can run MODEL yet." as now.
9. An account's row on the accounts page lists the models whose `accounts` holds it (`pages/accounts/rows.rs`, `models`).

## Inputs

- `temper/docs/plan/model-catalogue-1/notes/common.md`, `temper/docs/plan/model-catalogue-1/notes/contracts.md`
  (*The client*).
- `temper/docs/design/client/README.md`, 3.5, *The work's account and model*; 5.5, *The composer's line* and *A run's wait*.
- What not to build, the look's: `git show origin/look-requests --stat` holds it, for reading only: no menu, no "All
  models", no heading per account, no CSS.

## Owns and may change

- Owns its `files`. `words.rs` is a shared file: add `ConnectPlanOrKey`, change `WaitEmpty`'s words, remove
  `NeedsAccount`, add nothing else (`notes/common.md`).
- `temper-client-domain`: `choice::models` goes; `choice::offer` stands in; `Context::Template` gains `template: u32`, set
  here to `jig_client::shapes::CHAT` (`configure-picker` sets the configuring template). Call sites follow: `pages/chats.rs`.
- May change, each for its compile or its words alone: `temper/crates/temper-client-view/src/tests/{playground,chat,accounts,
  fixtures,home}.rs`; `temper-client-domain/src/tests/pages/chats.rs`; `temper-client-view/src/limits.rs`'s `choices`
  nodes if a provider's line passes the bound, with a hunk of its own.
- `temper/tools/gallery/src/worlds.rs`: its `Bare` and early `FirstRun` worlds list models with no account: empty them;
  `template_preview` fakes the preview through `llm`: give it `defaults`. Gallery names and samples stay. Journeys' prints
  are `fabrig/ui`'s catch-up: regenerate none.
- Rows whose `provider` is none, as `temper/tests/client/src/component.rs` has them: drawn last, under no line.
- `temper/docs/design/client/README.status.md`: removes the **Not yet** 3.4, 5.5 on the supplied default, and from the
  **Not yet** this plan added the chooser's sentence and the wait's. `configure-picker` closes the rest.
- Options, pre-approved:
  - A task funded by a project or another person shows the person's offer, as the draft has it (section 3): "its picker offers
    what is offered now". The run's check decides. Don't filter by payer.
  - The account the line names keeps today's translation (`selected_account`, `chosen_source`, `pinned`): the look takes the
    account off the line. Leave those functions, reading `ready` as usable for a row in the offer.
  - Two accounts of a provider draw one provider line here: the heading per account is the look's, from `accounts`.

## Steps

1. Domain, `pages/choice.rs`: `offer`, `Context::Template`, `Act::Model`, `valid`.
2. View, `choice.rs`: `panel` and a `provider_line`; `model_words`, its choice-none branch taking the preview; `using_line`;
   `byline_words`; `account_wait`. `model_shown` takes `shown` where a row names the model.
3. `rows.rs`, `models`. The gallery's worlds.
4. The tests. Break each rule once to see it bite.
5. Narrow tests, then the implementer's text's closing checks over `temper-client-domain`, `temper-client-view` and the
   gallery, with `heavy cargo nextest run -p temper-client-world -p temper-client-shell-terminal`. Commit and push.

## Report

As the implementer's text says; each shared file you changed, by name.

## Answers

### 2026-11-06: wait for offer-surface to finish, then resume picker-offer (recommended), or assign its missing domain-contract and fixture repair here? The current dependency branch prevents this task's acceptance tests from compiling. (model-catalogue-1/picker-offer-1, foreman model-catalogue-1/picker-offer/foreman-1)

Wait for offer-surface to finish, then resume picker-offer; its missing domain contract and fixture repairs remain offer-surface’s work.
The picker-offer spec declares `needs = ["offer-surface"]`.
Offer-surface’s goal explicitly owns “`MeModel`’s `effort`, `shown`, `accounts` and `picker`; `MeProject`’s `defaults`.”
Its acceptance 4 requires `jig_client::shapes::resolved_default`.
Its permitted `MeProject` literal repairs explicitly include `jig/testing/jig-client-test-records/src/owner.rs`, with “`defaults` empty”.
These changes fall outside picker-offer’s permitted files; do not reassign them here.
After the completed dependency is available, merge it, finish regeneration, acceptance tests, deliberate rule breaks and closing checks, then commit and push.
Picker-offer is not accepted as done: its report explicitly says acceptance is unverified.
