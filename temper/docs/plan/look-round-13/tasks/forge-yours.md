+++
id = "forge-yours"
part = "forge"
kind = "feature"
needs = ["forge-cards"]
files = ["temper/crates/temper-client-view/src/forge/connect.rs", "temper/crates/temper-client-domain/src/forge/admin.rs", "temper/crates/temper-client-domain/src/forge/tokens.rs", "temper/crates/temper-client-view/src/tests/connect_forge.rs", "temper/crates/temper-client-domain/src/tests/admin.rs", "temper/crates/temper-client-domain/src/tests/forgejo_tokens.rs", "temper/tests/client/tests/suite/forgejo_tokens.rs", "temper/tools/gallery/**"]
effort = "high"
minutes = 60
+++

## Goal

"Your Forgejo" (pages 13, 14): one panel, its address first, its rare fields folded, and "How temper connects" a choice between "As its admin" and "With its bot’s tokens" that shows one way's fields at a time. Each button says what it does, "Connect as its admin", "Connect with its tokens"; beside it a quiet "Other forges".

Acceptance:
- page 13 and 14 draw as the request's, in `CONNECT_PANEL`, `CONNECT_HOW`, `CONNECT_PAIR`, with the folds "Its git or web address, if they differ", "Its runner label: docker" (the summary says the label typed), "A webhook secret, if you made hooks to temper’s listener";
- each way's scopes as the request's: admin's "Scopes: write:admin, read:user, write:organization, write:repository"; API token's and git token's as page 14;
- today's checks, fields and connect acts are unchanged: a refusal shows in the panel with today's words;
- the managed Forgejo's admin details and the connected Forgejo's tokens details are this one page, the choice being which address it's at: a forge whose row is already one kind shows that way alone;
- focused tests of `temper-client-view`, `temper-client-domain` and the client world's `forgejo_tokens` pass.

## Inputs

- `temper/docs/design/client/forge.md`, 5.4, *Connecting a forge*, and its Your Forgejo fields.
- The request's pages 13, 14.
- `temper/docs/plan/look-round-13/notes/common.md`; `notes/contracts.md`, *Look tokens*.

## Owns and may change

- Owns the `files` above; in `forge/connect.rs`: `managed`, `admin_details`, `admin_form`, `tokens_details`, `tokens_form`, `github`, `here` and their helpers.
- May change: tests of the changed words in `temper-client-view/src/tests/{forges,settings}.rs`, `temper/tests/client/tests/suite/{forges_edges,github_connect}.rs`: words alone. `temper/tests/client-browser/**`: edit and compile only.
- Option, the choice: a `ChoiceGroup` of `Role::Choice`s with a domain field `admin::Way`; if the two forms' drafts can't share the address and the Details' fields, each keeps its draft, and only the address field is copied across on a switch.
- `Word::Forge`'s "Forge number" isn't here; that's the sheet's.

## Steps

1. Domain: the way's choice, a unit test: a switch keeps what was typed in each way.
2. View; tests; goldens, adding `connect-forge-yours-admin` and `connect-forge-yours-tokens`; journeys reprint.
3. Compare at 1440, 1000, 420 against pages 13, 14 by a throwaway driver.
4. `cargo fmt`, old forms, clippy, focused tests, commit with the trailer, push.

## Report

As the implementer's text has it, with the differences from the request's pages named.
