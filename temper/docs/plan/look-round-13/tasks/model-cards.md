+++
id = "model-cards"
part = "model"
kind = "feature"
needs = ["connect-look", "welcome-ways"]
files = ["temper/crates/temper-client-view/src/pages/connect_model.rs", "temper/crates/temper-client-view/src/pages/welcome_model.rs", "temper/crates/temper-client-view/src/pages/sign_in_wait.rs", "temper/crates/temper-client-view/src/accounts/words.rs", "temper/crates/temper-client-domain/src/pages/connect_model.rs", "temper/crates/temper-client-domain/src/pages/accounts/ways.rs", "temper/crates/temper-client-view/src/tests/connect_model.rs", "temper/crates/temper-client-domain/src/tests/pages/connect_model.rs", "temper/tests/client/tests/suite/connect_model*.rs", "temper/tools/gallery/**"]
effort = "high"
minutes = 90
+++

## Goal

`ConnectModel` takes the welcome's cards, panels and words, under Settings' path and in its column: pages 02, 03, 05, 06, 07 of the request. One set of cards serves it and the welcome. The steps bar, the "Choose" card, the demo API key and the two filled holder buttons go. Its end is `model-end`'s: until then the way's Ready may stand as it is.

Acceptance:
- at Choose the page is: `temper-connect` column, path, "Connect a model", its lede, two cards (`CONNECT_CARD`, the default also `CONNECT_DEFAULT` with the primary button), "Or an API key" with a row per key way, an owner's one quiet "Use deployment accounts" or "Use your accounts";
- a plan the person has an account of says "✓ Connected" in `CONNECT_YES` in its note's place, its button "Add another plan"; the default moves to the first way not connected (page 05); with the deployment's accounts chosen, no sign-in or program card, keys alone;
- no way of the scripted provider `demo` is offered, here or in the Accounts' rows' acts;
- each way's page is the heading, one `CONNECT_PANEL`, its words as the request's, a quiet "Other ways" beside its main button, no "‹ All ways"; the sign-in's wait is page 03: "Waiting for ChatGPT’s sign-in, in the new tab", "Open it again ›", "Other ways". "In a browser on the engine's machine" goes for "Sign in, in a new tab";
- `welcome_choices`/`choices` and `choose_body`/`standalone_choose_body` are one path; the welcome's own pages change only by what `CONNECT_*` replaces where their prints stay equal, or by a reprint that shows no change a person would see;
- the terminal keeps each heading and region's meaning (`temper/docs/design/draft/look-requests.md`, 3, step 5);
- focused tests of `temper-client-view`, `temper-client-domain`, `temper-client-shell-terminal`, the client world's `connect_model*` and `the_committed_journeys_are_their_print` pass.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, `ConnectModel`; 5.5, *The ways*; 5.6.
- `temper/docs/design/draft/look-requests.md`, 3 to 5; the request's `README.md`, and pages 02, 03, 05, 06, 07.
- `temper/docs/plan/look-round-13/notes/common.md`; `notes/contracts.md`, *Look tokens*.

## Owns and may change

- Owns the `files` above.
- May change: `temper-client-view/src/flow.rs` (`steps`, `choice_card` and the like: remove what nothing calls once this and `forge-rows` have run; leave what `forge/connect.rs` still calls); `pages/settings.rs`, `path`'s word for this page only; `Word::PathConnectAccount` to "Connect a model"; tests that walk the removed parts, in `temper/tests/client/tests/suite/{sign_in,sign_in_paste,accounts_edges,welcome_*}.rs`, `tests/client/src`, `temper-client-view/src/tests/{accounts,model_forge,home,rail,settings}.rs`, `crates/temper/tests/support/person.rs`: only for the words and controls that changed. `temper/tests/client-browser/tests/browser_all/browser_accounts.rs`: edit and compile only.
- Option, the demo way: if `accounts::ways`' `playground: Option<u32>` is `None` whenever the playground is not read, skip an offer whose name is `demo` in `ways` itself, naming the reason in its doc. If a test seeds a provider named `demo` to stand for a real one, rename it there.
- Option, the holder: `Act::Holder` stays the act; its two buttons become one link (`Role::Link` or a `Role::Button` in `CONNECT_QUIET`), toggling. Its tests press the new name.
- "Connected" reads the person's accounts in `accounts::document`, by provider and kind; if a way's kind can't be told apart from a row, by provider alone, and say so.

## Steps

1. Domain: `model::connected(root, way) -> bool` and `model::default_way(ways, ...)` in `pages/connect_model.rs`, each with a unit test on `temper-client-domain/src/tests/pages/connect_model.rs`: the first not connected is the default; all connected, none.
2. View: one `cards` function, its callers both; the column and tokens; the panels; "Other ways". Sign-in's wait from `sign_in_wait.rs`, the welcome's words.
3. Repair the tests; regenerate the gallery's goldens and names (`connect-model-*` stay named; add `connect-model-connected` for page 05); reprint journeys if they change; the terminal's goldens.
4. Compare at 1440, 1000, 420 against the request's pages by a throwaway driver; report the differences left.
5. `cargo fmt`, old forms, clippy, focused tests, commit with the trailer, push.

## Report

As the implementer's text has it, with the differences from the request's pages named.

## Rebase (2026-11-06, step merge, foreman look-round-13/model-cards/foreman-1)

resolve the three merge conflicts while preserving model cards and landed forge/proposal changes
The merge failed in three shared files; rebase onto the current landing target.
Relevant incoming commits are `479f51f772` (forge cards) and `d6820c4ad4` (main’s proposal updates).

- `temper/crates/temper-client-shell-web/temper.css`: combine both branches’ selector fixes. Preserve Settings support for the forge sheet and model sign-in/paste panels, shared headline/lede spacing, and model waiting styles. For connect rows, retain the incoming strong label’s `font-size: 15px` and exclude spans containing a strong label from muted description styling.
- `temper/crates/temper-client-view/src/words.rs`: keep the incoming forge/proposal vocabulary and IDs, plus this task’s `PathConnectAccount` wording, “Connect a model”. Recount `PHRASES`, `SEGMENTS`, and `ENGLISH_BYTES` for the combined code; neither side’s constants alone suffice. Preserve the incoming `ALL` entries.
- `temper/tools/gallery/src/samples.rs`: retain all incoming forge/proposal samples and this task’s `connect-model-connected` sample and account setup in `domain()`. At the inspected integration tip, the combined sample count is 282; derive it again if the target advances.
- Check the automatically merged gallery tests/worlds for both sets of behavior. Regenerate gallery names and affected prints/goldens through their tools; keep names sorted. Reprint journeys if the combined rendering makes them stale.
- Run the spec’s focused checks through `heavy`, including the gallery and committed-journey checks. Browser stories are compiled only.

## Rebase (2026-11-06, step merge, foreman look-round-13/model-cards/foreman-2)

resolve the words and gallery conflicts while preserving the landed Your Forgejo changes
Rebase onto the current landing target. The merge conflicts are in `temper/crates/temper-client-view/src/words.rs` and `temper/tools/gallery/src/samples.rs`.
The new incoming change is `1cfcc37274` (“Merge Your Forgejo with the Settings proposal vocabulary”), landed by integration merge `d1190606f7`.

- In `words.rs`, preserve incoming `ConnectHow` through `ConnectFirst`, their IDs, English phrases and `ALL` entries, and the removal of `TokensTitle`, `TokensWebhookHint` and `AdminFirst`. Keep this task’s `PathConnectAccount` wording, “Connect a model”.
- Recount `PHRASES`, `SEGMENTS` and `ENGLISH_BYTES` using the combined vocabulary’s declaration checks; neither branch’s constants suffice.
- In `samples.rs`, retain incoming `connect-forge-yours-admin` and `connect-forge-yours-tokens`, including their setup in `domain()`. Also retain `connect-model-connected` and this task’s account setup for both model choice samples.
- The combined sample count at `d1190606f7` is 284, replacing the previous rebase instruction’s 282. Recount if the target advances.
- Check the automatically merged CSS, gallery names and tests: preserve both forge samples’ assertions and model-card assertions. Keep names sorted and consistent with sample order.
- Regenerate any stale prints, goldens or journeys through their tools.
- Run the spec’s focused checks through `heavy`, including gallery checks and `the_committed_journeys_are_their_print`. Browser stories are compiled only; run no walks or render checks.
