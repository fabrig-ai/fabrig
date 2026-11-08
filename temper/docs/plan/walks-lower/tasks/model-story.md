+++
id = "model-story"
part = "model"
kind = "feature"
needs = []
files = ["temper/tests/client/tests/suite/connect_model.rs"]
effort = "medium"
minutes = 45
+++

# A welcome sign-in's account reads usable from the engine, in the joint world

## Goal

Of the process walk `sign_in::a_chatgpt_sign_in_on_the_fake_box_leaves_a_usable_model_and_ready_can_create`
(`temper/crates/temper/tests/fake_backends/walk/sign_in.rs`), the box's
`a_chatgpt_browser_tab_on_the_fake_box_leaves_ready_able_to_create` walks
the same path to Ready and stays; the one check only the first makes is
that the account document, read fresh after the sign-in, says the
account is usable. It moves to `temper/tests/client/tests/suite/connect_model.rs`,
into the held welcome sign-in's shared check. The walk itself is
`walks-go`'s; you never touch it.

Acceptance:

- `welcome_moves_on` (shared by
  `a_held_welcome_sign_in_moves_on_once_when_the_watch_makes_its_account`
  and `a_reloaded_welcome_sign_in_moves_on_once_when_the_watch_makes_its_account`)
  checks, from the joined engine's account facts as the person's client
  holds them (the account document `model_usable` reads; never a fixture
  value): before `SignedIn`, no account is usable; after it, the account
  the sign-in made is usable, of the ChatGPT plan's provider, with one
  sign-in source; and a fresh read of the Accounts page (`reopen`) says
  the same. Both stories pass.
- **Bite:** send `SignedIn` with `declined: true` in place of `false`
  in `welcome_moves_on` by hand, or skip it, and see the new usability
  assertion fail for that reason; restore.
- Clippy clean, fmt clean, `temper-client-world`'s focused suite green.

## Inputs

- `temper/docs/plan/walks-lower/notes/common.md` (read first).
- `temper/docs/plan/walks-lower/inputs/request.md`, the
  `sign_in::a_chatgpt_sign_in_on_the_fake_box_leaves_a_usable_model…` ruling.
- `temper/docs/design/domain/testing.md`, 2.6, the bullet **The fake box
  keeps two walks**, its **A sign-in's usable model** item.

What the survey found, for your bearings (verify before relying):

- `welcome_sign_in`, `welcome_signed_in`, `welcome_moves_on` are local to
  `connect_model.rs`; `accounts::empty_plan_fixture` starts with no
  usable model and no deployment account
  (`temper/tests/client/src/front/account_acts.rs`, about 268).
- `model_usable` reads the account facts' `usable`
  (`temper/crates/temper-client-domain/src/pages/welcome.rs`, about 283;
  `pages/accounts.rs`, about 36).
- `a_typed_welcome_sign_in_way_with_a_usable_model_keeps_continue`
  checks usability on an already-usable fixture, which proves nothing of
  a sign-in: don't lean on it.

## Owns and may change

Owns:

- `temper/tests/client/tests/suite/connect_model.rs`: `welcome_moves_on`
  and any private helper it needs.

Pre-approved options:

- **A new story instead:** if extending `welcome_moves_on` would break
  another of its callers' checks, write a story of its own,
  `a_welcome_sign_in_makes_its_account_usable`, beside them, and say so.

## Steps

1. Read the inputs and `connect_model.rs` around `welcome_moves_on`.
2. Extend it; run both stories:
   `heavy cargo nextest run -p temper-client-world --test suite -E
   'test(/^connect_model::a_(held|reloaded)_welcome_sign_in_moves_on/)'`.
3. Bite it as above; restore.
4. Once, at the end: `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-world --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-client-world` (check the count). Commit
   (`walks-lower: a welcome sign-in's account reads usable in the joint
   world`, the body saying why) and push.

## Report

As the implementer's text says: the first line; the commit pushed; each
run with its count; the bite and its failure; the minutes.
