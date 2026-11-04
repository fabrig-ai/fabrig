+++
id = "home-facts"
part = "home-facts"
kind = "feature"
needs = ["ready-adopts"]
files = [
  "temper/crates/temper-client-domain/src/pages/home.rs",
  "temper/crates/temper-client-domain/src/tests/pages/home.rs",
]
effort = "high"
minutes = 80
+++

# What Home knows

## Goal

The domain gives the view Home's facts as one pure reading, so that `home-page` draws: whether a
way of working is configured, the repository step's state, whether a chat was asked. `Home` keeps
the watches they read, and "Start" works from it.

Acceptance:

- `pages::home::facts(root, project)` and `Facts::left`, to `notes/contracts.md`, for each state of
  `Repository`: in, adopting, offered with and without a refusal, GitHub, none, unknown.
- On `Address::Home`, `Subject::Settings` is watched and unwatched on leaving, and a refused or
  ended watch reads not configured. The chats page's own reading is unchanged.
- `pages.chats` is kept on `Home`, so that `Act::FirstChat` starts the chat from there and its
  answer opens it.
- The first repository's precedence: `In` over `Adopting` over `Offer`.
- Every commit carries the request's trailer (`notes/common.md`).

## Inputs

- `temper/docs/plan/look-round-3/notes/common.md`, `notes/contracts.md` (Home's facts, Ready's
  adoption, A project's repositories).
- `temper/docs/design/client/README.md`, 3.5, *Home*, *The first chat*.
- `jig/crates/jig-client-domain/src/contract/settings.rs`, `SettingsDocument`.

## Owns and may change

Owns, as `files` lists them, with their `mod` lines.

May also change:

- `temper/crates/temper-client-domain/src/pages.rs`: `went`, where `Chats` and `Inbox` are made
  and left, and a `pub home: Option<home::Home>` field with its getter.
- `temper/crates/temper-client-domain/src/pages/chats.rs`: what lets a `Chats` serve `Home`.
- `temper/crates/temper-client-domain/src/forge/listed.rs`: read only, unless `switcher-rows` hasn't
  landed; then add `held` as `notes/contracts.md` says, and that task keeps it.
- `temper/crates/temper-client-domain/src/tests/**`: fixtures the new watch needs.
- `temper/crates/temper-client-shell-terminal/src/tests/fixtures.rs`, 294: a `Settings` delivery,
  where the fixture walks to `Home`.

Options, each pre-approved:

- If `Chats::configure` needs the page's address, widen its check to `Home`.
- If a `SettingFollows::Change` line shows on a fresh box, where no person accepted anything (temper
  writing its own), read the revision's `by` instead: done on a `RevisionDecider::Party`. Say so.
- `chat_asked` counts an undelivered document as not asked, as `offers_first_chat` does.

## Steps

1. `pages/home.rs`, to `notes/contracts.md`, as `pages/inbox.rs` keeps its watch.
2. `pages.rs`: `went` as `Acceptance` says.
3. Tests in `tests/pages/home.rs`: every `Repository` state; each order of rows and answers;
   `left`, 0 to 3; the watch's open and close; `FirstChat` on `Home`; a project that isn't the
   person's. Each bites.
4. **Check,** `notes/common.md`, then `heavy cargo nextest run -p temper-client-domain -p
   temper-client-view -p temper-client-shell-terminal -p temper-client-world`, clippy on the same.

## Report

As `notes/common.md`, Your report.
