+++
id = "only-story"
part = "story"
kind = "feature"
needs = ["only-read"]
files = ["temper/tests/client/tests/suite/only_you.rs", "temper/tests/client/tests/suite/main.rs", "temper/tests/client/tests/suite/fuzzy/density.rs", "temper/tests/client/src/front/account_acts.rs", "temper/tests/client/tests/suite/accounts_page.rs"]
effort = "high"
minutes = 100
+++

## Goal

Prove Only you through temper's engine and the client's domain together:
a person's ChatGPT sign-in, signed in, then read as expiring, then signed
out, then signed in again, is listed while it needs them and gone after
(`temper/docs/design/client/README.md`, 3.5, **Only you**). And the
density fuzz holds Only you's part to its rule across random accounts
documents (*A page's parts*).

Acceptance:

- `tests/suite/only_you.rs` (its `mod only_you;` in `tests/suite/main.rs`,
  sorted), one story, `a_login_is_listed_only_while_it_needs_its_person`,
  in the joint world with only the issuer faked:
  1. a signed-in ChatGPT plan, as `tests/suite/accounts_page.rs`'s
     `signed_in` makes it, then `Address::Inbox` unfiltered, settled:
     `pages::only_you::read(root.person())` holds no item, and
     `parts::needs_facts(..).only` is 0;
  2. the engine hears `jig_domain_accounts::FromComponent::Expiring` for
     the source at its grant's generation: one item, `Why::Expiring`,
     kind `SignIn`, its account and source the plan's, and `parts::needs`
     shows `only`;
  3. the engine hears `FromComponent::Failed { failure: SignInAgain }`:
     one item, `Why::SignedOut`;
  4. signed in again, the same subject (`Event::SignedIn`, as step 1's
     sign-in, renewing the account): no item, and `only` hidden.
- `tests/suite/fuzzy/density.rs`: the world delivers, at random steps
  among its others, an `accounts` document (`Watchable::Jig(c::Subject::Accounts)`,
  `c::Delivery::Accounts`) of zero to three accounts, each the person's or
  the deployment's, its deciding source a sign-in, a program's login or a
  key, signed out, expiring, both or neither, sometimes `cut`, and keeps
  the last one; `check` asserts `needs_facts.only` equals the count its
  own oracle takes from that document (the person's accounts whose first
  source is a sign-in or program's login signed out or expiring), and
  `check_rules` and `check_content` hold `only` to `known && only != 0`.
  `observed` gains "an Only you item" and "no Only you item", both
  required. `SEEDS` and `STEPS` stay as they are.
- The rules bite: count a key's signed-out source in the read once, see
  the fuzz fail on its seed, restore; the report names it and the seed.

## Inputs

- `temper/docs/plan/ui-n3a-only/notes/common.md`;
  `temper/docs/plan/ui-n3a-only/notes/contracts.md`, sections 1 and 2.
- `temper/docs/design/client/README.md`, 3.5, **Only you** (*Its read*,
  *It shows*, *Its acts' end*), *A page's parts*.
- `jig/docs/design/domain/accounts.md`, 3.3 (a sign-in renewing the
  account of the same subject).
- Patterns: `tests/suite/accounts_page.rs:33` (`signed_in`), `:70`
  (`FromComponent::Failed` sent to the engine and settled);
  `temper/tests/client/src/front/account_acts.rs` (`plan_fixture`,
  `settle`); `jig/crates/jig-domain-accounts/src/domain.rs:991` (the
  engine's `Expiring`, which needs the grant's generation);
  `tests/suite/fuzzy/density.rs:707` (a delivery), `:490` (`check`).

## Owns and may change

- **Owns:** the files in this spec's header.
- **May change:** `tests/suite/accounts_page.rs`'s `signed_in`, moved into
  `front::account_acts` as `pub fn signed_in_plan(seed: u64) -> (Joint,
  Root, u32)` with `accounts_page.rs` calling it, its behaviour the same;
  that is the default, so that the story shares the fixture rather than
  copying it.
- **Not yours:** `temper-client-domain` and `temper-client-view`; the
  density world's other facts and their checks, but the literal of
  `NeedsParts` `only-read` wrote.
- **Options, pre-approved:** if no grant is held after the sign-in, send
  the engine `FromComponent::Granted` for the source first, as the
  component would, and use its generation. If the renewal in step 4 needs
  the issuer's visit again, do it as step 1 does. If the density world's
  random accounts need the person's party, take it from its `me` frame.

## Steps

1. The story; break nothing yet.
2. The density world's deliveries, oracle and checks; break the rule once.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-world --all-targets -- -D warnings`; `heavy
   cargo nextest run -p temper-client-world --test suite -E
   'test(=only_you::a_login_is_listed_only_while_it_needs_its_person) |
   test(/^accounts_page::/)'`; `heavy cargo nextest run -p
   temper-client-world --profile fuzzy --test suite -E
   'test(=fuzzy::density::random_facts_and_settings_keep_each_part_and_card)'`,
   its time against the fuzzy suite's 120 CPU seconds (the report gives
   it before and after). Commit and push.

## Report

As the implementer's text asks; the density test's time before and after.
