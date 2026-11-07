+++
id = "welcome-step"
part = "welcome"
kind = "feature"
needs = ["stories-named"]
files = ["temper/tests/client-browser/tests/browser_all/browser_accounts.rs", "temper/tests/client-browser/src/issuer.rs", "temper/tests/client/tests/suite/second_sign_in.rs", "temper/tests/client/src/front/account_acts.rs"]
effort = "medium"
minutes = 55
+++

## Goal

The welcome's sign-in becomes the first story of test 9, on a second provider of the same issuer, so the first thing a person meets stays in the tier. The path that merge newly takes, a second provider's sign-in after the welcome's, runs in the client world first, because the browser test isn't run here.

Acceptance:

- `suite::second_sign_in::a_second_providers_sign_in_reaches_ready_after_the_welcomes` passes in the focused suite, and is seen to fail on the same provider (below).
- `the_welcome_sign_in_opens_a_tab_and_moves_on_to_ready` is gone. Its steps are the story `welcome` of `the_sign_in_opens_a_tab_and_the_page_moves_on_to_ready`, including `/welcome?at=model`, `/welcome?at=ready`, and the authorization and code exchange it counted.
- A focused unit test holds that the two-provider deployment passes startup validation.
- A failed step names its story, in the round trip's second script too.
- `heavy cargo check -p temper-client-browser-world --tests` passes. **The browser test isn't run**; the report says so.
- `heavy cargo nextest run -p temper-client-browser-world --lib` and `heavy cargo nextest run -p temper-client-world --test suite -E 'test(second_sign_in)'` pass, each selecting what it names. Check the count.

## Inputs

- `temper/docs/design/client/browser.md`, 4.2: *`ConnectModel`'s sign-in, in a new tab*, and in it *The welcome's Model step*; 4.3, *A test of several stories*; 4.6, 9.
- `temper/docs/plan/browser-cap/notes/common.md` and `notes/contracts.md`.
- `temper/docs/plan/browser-cap/plan.md`, *What was decided*: why the welcome goes first, on a second provider.
- `temper/tests/client/tests/suite/connect_model.rs`: `welcome_sign_in`, `welcome_signed_in`, `welcome_moves_on`, and the helpers `view`, `text`, `go`, `press`, `visited`, `visit`.
- `temper/tests/client/tests/suite/sign_in.rs`: `renewed_sign_in_explains_the_same_account_on_connect_model`, what one provider would do.

## Owns and may change

Owns the header's files.

May change, for these and nothing else:
- `temper/tests/client-browser/src/lib.rs`: `Tier::accounts_sign_in`, one new private function beside `account_provider`, and one test in its `tests` module. `Tier::accounts` and `account_provider` keep what they do for test 8. Everything else in this file is `reading-one-test`'s.
- `temper/tests/client/tests/suite/main.rs`: `mod second_sign_in;`.

Not yours: `browser_pages.rs`, `connect_model.rs`.

Options, pre-approved:
- If startup validation refuses the second provider for socket or account room, lower another of the fixture's `limits.overrides` the way `account_provider` does, with its reason in a comment, and say so. If it refuses for another reason, give the second model or provider what validation asks, and say so.
- The client-world fixture's route into `fixture_root` (a new `Tools` variant, a parameter, or editing its config after) is yours; the smallest that reads well.
- If `model-cards` or `model-end` has reached your base and the words below differ, read them as the page now has them, and say so.

## Steps

### The lower story first: `temper/tests/client/tests/suite/second_sign_in.rs`

A new file, so as to stay off `connect_model*.rs`, which other drafts are changing. It copies the few helpers it needs. Its module comment cites `temper/docs/design/client/browser.md`, 4.2.

A fixture beside `empty_plan_fixture` in `front/account_acts.rs`:

```rust
/// `empty_plan_fixture`, with a second provider offering a sign-in: `playground_providers`' `other`.
#[must_use]
pub fn two_sign_ins_fixture(seed: u64) -> (Joint, Root);
```

```rust
#[test]
fn a_second_providers_sign_in_reaches_ready_after_the_welcomes() {
    let (mut front, mut root) = accounts::two_sign_ins_fixture(fixtures::SEED);
    // The welcome's: reopen at Address::Welcome { step: d::Step::Model { way: None } }, press its sign-in,
    // visit, SignedIn with SUBJECT. The address is Welcome { step: Ready }.
    // ConnectModel's: go to Address::ConnectModel { way: None }; the second provider's way is offered there
    // though the model is now usable. Press it, by the label the view gives; visit; a new source.
    // front.tabs == [Open { PROVIDER }, Point { ISSUER }, Open { PROVIDER + 1 }, Point { ISSUER }].
    // The address is ConnectModel { way: Some(ModelWay { provider: PROVIDER + 1, kind: SignIn, source: Some(second) }) }.
    // SignedIn with the same SUBJECT: the page shows "Signed in to <the second provider>",
    // not "the same account again". front.leaves is empty.
}
```

**It must bite:** run the second sign-in on `PROVIDER` itself. It fails on the renewal's words, the reason this merge takes a second provider. The report says what it saw.

### The deployment: `src/lib.rs`

`Tier::accounts_sign_in` calls `account_provider`, then this:

```rust
/// The welcome's own provider on the same issuer, whose one subject would renew on the first
/// provider (temper/docs/design/client/browser.md, 4.2). A provider claims its models alone.
fn welcome_provider(value: &mut serde_json::Value, sign_in: &serde_json::Value);
```

It pushes a clone of `engine.llm.models[0]` named `welcome-fake`, and a provider `{"name":"Welcome provider","number":18,"signed_in_plan":false,"models":["welcome-fake"],"sources":{"keys":true,"sign_in":sign_in}}`.

**Its test,** in `lib.rs`'s `tests`, on the shape `the_tier_passes_startup_validation_with_stock_defaults` has:

```rust
#[test]
fn the_sign_in_tiers_two_providers_pass_startup_validation() {
    // `documents(..)`'s deployment, with account_provider and welcome_provider applied to a loopback
    // issuer's registration, through temper_service::startup::shape.
}
```

**It must bite:** with `"models":["fake"]` on the second provider, validation refuses. The report says so.

### The issuer: `src/issuer.rs`

`codes: 1` becomes 2, since a redeemed code keeps its place, and `connections: 2` becomes 4, since each tab may hold one beside the engine's. Its header comment says two sign-ins of one synthetic `OpenID` grant. These are the fixture's room, no budget or deadline.

### `browser_accounts.rs`

```rust
let (tier, mut issuer) = Tier::accounts_sign_in("accounts-round-trip");
let mut stories = Stories::of_person(0);
let mut steps = vec![];
stories.begin("welcome", &steps);
steps.extend(welcome_steps());
stories.begin("connect model", &steps);
steps.extend(/* the first script's steps of the test as they stand */);
```

**`welcome_steps()`** are the retired test's steps as they stand, with its button `Sign in to Welcome provider`: the see of "Set up temper", `Go` to `/welcome?at=model`, see the button, `Address`, press, `sign_in_tabs(1)`, see "Almost ready", `Address` `/welcome?at=ready`.

**`connect model`** drops its own first step, the see of "Set up temper": the page is no longer where it landed, and `welcome`'s see of it stands for both. Everything else stays, in order, from `Go` to `/chats` on.

**Tabs:** `sign_in_tab()` becomes `sign_in_tabs(count: usize)`, a `Step::Tabs` of `count` alike. Each sign-in's tab stays open, so both of `connect model`'s, the first script's and `RoundTrip`'s, expect 2.

**`RoundTrip`** replaces its script partway, so a later step's number starts over. A new field `before: u32` takes the first script's `at` when it does; `next` adds it to a `Next::Failed`'s `step`, and `at` adds it too. That keeps `Stories` naming `connect model` for a failure in the second script.

**After the run:** `stories.assert_done(outcome)`, then:

```rust
assert_eq!(/* Authorize count */, 2, "one authorization for each sign-in: a reload makes no second departure");
assert_eq!(/* Post count */, 2, "one code exchange for each sign-in");
```

Update the module comment if it needs it. Remove what the merge leaves unused.

### Then

`cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-browser-world -p temper-client-world --all-targets -- -D warnings`; the acceptance's commands. Commit and push.

## Report

As the implementer's text has it. Also:
- Each of the retired test's steps and where it now stands, and the one see that stands for both.
- What the lower story broke on, and the same for the startup test.
- Any option taken.
- That no browser test ran.
- Anything read in the code that makes a merged step look wrong, such as what `/chats` or `/accounts/connect` draw once a model is usable. After-landing proves these, so what you suspect is worth a line.
