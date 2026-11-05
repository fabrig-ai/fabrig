# forge-no-registration: an owned Forgejo that registers no one through temper

The user wants registration disabled on the deployment's forge (2026-11-05). `[service] DISABLE_REGISTRATION` is already true. But when `application.sign_in.forges` names an owned forge, its preparation writes `[oauth2_client] ENABLE_AUTO_REGISTRATION = true` and `[service] ALLOW_ONLY_EXTERNAL_REGISTRATION = true` (`temper/crates/temper-forge-setup/src/ini.rs`, 78; `sign_in.rs`, 13; `temper/docs/design/protocol/setup.md`, 6.2; `sign-in.md`, 9.4). So a member's first "sign in with temper" makes an account.

temper mints for a member alone, so a stranger makes none today (`sign-in.md`, section 3). That stays the default: a deployment's other people need it.

## The change

One environment variable read where `temper forge prepare` runs, say `TEMPER_FORGE_NO_REGISTRATION`, unset meaning today's bytes exactly. When set, the preparation writes:

- `[oauth2_client] ENABLE_AUTO_REGISTRATION = false`, with `USERNAME` and `ACCOUNT_LINKING = login` as today;
- no `ALLOW_ONLY_EXTERNAL_REGISTRATION = true`, so that `DISABLE_REGISTRATION` also closes the form Forgejo's link page would offer.

`sign_in.rs`'s `SETTINGS` for a managed Forgejo follows the same variable, if that is where it's read; otherwise say so. A later prepare must write the same, so that it survives.

## Tests

A golden of `ini.rs`'s text with the variable set and unset; the unset one unchanged. Then, in the process tier's Forgejo sign-in story, or the cheapest tier that holds a real Forgejo, prove what the settings do at Forgejo 17.0.1 (conformance is the check for what Forgejo does; nothing here has been run):

1. someone with no linked account who signs in through temper gets no new account;
2. an account already linked keeps signing in;
3. an existing account not yet linked is offered linking by its password, once, with no way to register.

If (1) or (3) doesn't hold, end with a `Question:` naming what Forgejo did. No test reaches a real host.

## Design

Add the entry to `temper/docs/design/draft/dogfood-hacks.md`, in section 2's form: what, why, cost and risk, what the pipeline does there (it has no such sign-in; its forge is a public one), and a real design: possibly a setting on the owned forge, with this as its value. `setup.md` and `sign-in.md` stay as they are; `dogfood-hacks.status.md` names the gap.

## Beside it

Leave fabrig/ui's files alone. One task, in temper. A stored format, a protocol or a public interface change is a `Question:`. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).
