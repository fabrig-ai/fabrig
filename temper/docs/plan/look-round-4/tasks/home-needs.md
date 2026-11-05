+++
id = "home-needs"
part = "home-needs"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/home.rs",
  "temper/crates/temper-client-domain/src/pages/inbox.rs",
  "temper/crates/temper-client-domain/src/tests/pages/home.rs",
  "temper/crates/temper-client-domain/src/tests/pages/inbox.rs",
]
effort = "high"
minutes = 75
+++

# Home reads what needs the person

## Goal

Home holds the project's needs beside its chats, as `Inbox` reads them, and `Facts` says how many.
Nothing is drawn yet: the domain only. Every commit carries `Look-request: project-home-everyday`.

Acceptance:

- On `Address::Home { project }`, `Pages::inbox()` is `Some`, its filter `Filter { project:
  Some(project), kind: None, goal: None }`, and `Pages::chats()` is `Some` as it is today.
- `Inbox` reads a project's filter from the engine, as it reads a goal's: the `Read::Inbox` branch of
  `refresh` runs for `filter.project.is_some() || filter.goal.is_some()`, so that a project's need
  past the party's window is read. The `Inbox` page with a project filter takes the same path.
- `Facts.needs` is `Needs::Unknown` until the inbox's first delivery and the first answer of its
  read, then `Needs::Count { count, cut }`: `count` the cards of the needs' groups, whatever the
  updates hold, `cut` the read's cut for needs (`Inbox.cut[0]`). `Inbox::needs` lists them
  (`notes/contracts.md`).
- Home's handles never collide: `Home` one, `Chats` two, `Inbox` from there, and leaving leaves
  `pages.next` past all of them, whichever page leaves last (`pages.next` is the largest of what
  each `leave` returns; today the last write wins, and it would go backwards).
- `page_told` lets both pages hear: with `chats` and `inbox` both set, what `Chats` does not take
  goes to `Inbox`. The arms that have one page behave as they do.
- Leaving Home releases the inbox's watches, on a new address and on `Event::Left`.

## Inputs

- `temper/docs/plan/look-round-4/notes/common.md`, `notes/contracts.md` (What Home knows).
- `temper/docs/design/client/README.md`, 3.5, *Home* and *Home on any project*.
- `git show --format=%B -s 8bbc1bc964`: the request's message.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-domain/src/acts.rs`, `src/tests/fixtures/*`: what `cards_mut` or a
  fixture needs for an `Inbox` living beside `Chats`.
- `temper/crates/temper-client-view/src/tests/{inbox.rs,fixtures.rs}`, `temper/tests/client/**`: a
  fixture's answer to the project's read, where a test of the `Inbox` page with a project filter
  pinned the client-side filter; nothing else in them.

Options, each pre-approved:

- If `Told` isn't `Clone`, `Chats::told` may take `&Told`, or take it by handle first; your call.
- If a test of `Event::Left` pins that the inbox stays watched, it pinned what Home now breaks:
  change it and say so.
- If the read's `Updates` group costs Home a second read it never draws, keep it: the `Inbox` is one
  shape, and a Home that drops it is a change later.

## Steps

1. **`pages/inbox.rs`:** `refresh`'s condition; `Inbox::needs`; `Inbox::delivered(&self, client:
   &Client) -> bool`, the entries' document delivered and every read this refresh sent answered.
2. **`pages.rs`:** `went`, `Event::Left`, `page_told`, the handles. Read `redirect_chat` and every
   `pages.inbox` reader: nothing may take Home for the inbox's page (the rail's current entry is the
   address's, not this state).
3. **`pages/home.rs`:** `Needs`, `Facts.needs`, `facts`.
4. **Tests, `tests/pages/home.rs`:**
   - `home_reads_its_projects_needs_and_releases_them`: the `Read::Inbox` names the project; leaving
     unwatches every handle, and a later page's handles are new;
   - `needs_are_unknown_until_delivered_then_count_the_needs_alone`: an update isn't counted, and a
     cut read says `cut`;
   - `home_hears_its_chats_and_its_inbox_alike`: a chats' answer and an inbox's answer, either order;
   - `home_owns_and_releases_its_settings_and_chat_reads` keeps what it says.
5. **A change of meaning:** the `Inbox` page with a project filter reads the engine, so run its
   readers' tests: `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p
   temper-client-world`.
6. **Check,** `notes/common.md`, clippy on those packages, then push.

## Report

As `notes/common.md`, Your report, with what `Inbox::delivered` waits for.
