+++
id = "home-everyday"
part = "home-everyday"
kind = "feature"
needs = ["home-needs", "everyday-look"]
files = [
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/names.txt",
]
effort = "high"
minutes = 170
+++

# Home on any project

## Goal

Home draws what the request `project-home-everyday` asks, where the facts reach: the headline by what
needs the person, Needs you and Chats. To begin stays as it stands; `to-begin-card` changes it. Every
commit carries `Look-request: project-home-everyday`.

Acceptance:

- The markup follows `git show 8bbc1bc964:temper/docs/design/client/journeys/project-at-work/01.html`,
  names, classes and order, where the facts reach. Words are
  `temper/docs/design/client/README.md`, 3.5, *Home on any project*, each state of it.
- **The order:** the intro; To begin while `facts.left() > 0`, as today; Needs you; Chats; Connected,
  as today. With To begin drawn, Needs you and Chats are left out when empty.
- **The headline,** with no To begin drawn: "Nothing needs you.", "One thing needs you.", "N things
  need you.", "N+ things need you." with a cut read, and the project's name while `Needs::Unknown`.
  With To begin drawn it stays as today. No lede with no To begin drawn; the kicker as today.
- **Needs you:** `section.temper-home-needs`, named "Needs you", `.temper-home-head` with `h2`,
  "Needs you · N" and `.temper-home-signal` while something does, "Needs you" otherwise; with more
  than one, `a.temper-home-in-a-row` "Answer in a row" to `Address::Inbox { filter: Filter { project:
  Some(project), kind: None, goal: None } }`, the browser's alone. Empty, `p.temper-home-empty`.
- **The first need,** `.temper-ask`, from `Inbox::needs`: `p.temper-ask-meta`, the task's link as the
  inbox's card gives it (`Address::Task`), named and showing the task's title, or its number with no
  title, then " · a question · 4 min", its kind in lower case and its age in `TimeForm::Span`;
  `p.temper-ask-question`, the words the inbox's card draws; `.temper-ask-choices` holding one button,
  `.temper-ask-quiet`, "Reply" on a question and "Open" on any other kind, `Act::Go` to that filtered
  inbox. With its page not read yet, the meta line alone.
- **On a phone,** in the browser alone (`Shell::Browser`), after the card, `a.temper-home-cta` to that
  inbox, named "TITLE: WORDS Answer", its first span "TITLE: WORDS", its second
  `span.temper-home-cta-go` "Answer". The stylesheet shows one or the other.
- **The others,** at most five, `ul.temper-home-rows`, each a link to that inbox: a span of the task's
  title, `span.temper-home-when` of its kind and age.
- **Chats:** `section.temper-home-chats`, named "Chats"; `.temper-home-head` with `h2` "Chats" and,
  with any chat, a link "All N", named "All N chats", "All N+" where `live.more` or the ended
  `after` cuts the rows, to `Address::Chats { filter: ChatsFilter::All, start: false }`.
  `ul.temper-home-rows`, at most five, live first as `Chats` delivers them, then ended; each `li > a`
  to `Address::Chat { task }`, named "TITLE, STATE, AGE": a span of the title and
  `span.temper-home-when` "STATE · AGE": "waiting on you" for `ChatStatus::WaitingForYou`, "closed"
  once ended, otherwise the words `Chats`' status says; AGE in `TimeForm::Span`, from `ended_at` or
  `last_activity`. Empty, `p.temper-home-empty` "No chats yet."
- **The terminal** keeps the headings, and each line one readable run: the print test holds "Needs
  you · 1", the question's words, "Reply" and "Chats".
- **Not drawn** (`README.md`, 11; the account says so): the kicker's "you last looked", what landed,
  the lede, the byline, choices, the context line, "Later", the answer's green line and "Undo",
  Goals, the landing's line.
- **Gallery:** `home-at-work` (the request's page: one question, three chats, the words as drawn, but
  the facts no client is given), `home-needs-several` (three needs), `home-nothing-needs` (every step
  done, nothing needs, one chat). `home-ready` now draws the plain Home.

## Inputs

- `temper/docs/plan/look-round-4/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5, *Home*, *Home on any project*; 5.12; 7.2 of `browser.md`.
- `git show --format=%B -s 8bbc1bc964`; the pages at that commit.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-view/src/words.rs`: words as `notes/contracts.md` says; `limits.rs`: a
  count that follows them.
- `temper/crates/temper-client-view/src/pages/inbox.rs`: `source_task`, `task_title`, `waiting_since`
  made `pub(crate)`, a words-only `card_words(page)` split from `body`.
- `temper/crates/temper-client-view/src/pages/chats.rs`: `status` split so that a words-only
  `status_name` serves Home; `status` draws what it drew.
- `temper/crates/temper-client-view/src/tests/{fixtures.rs,frame.rs}`: fixtures.
- `temper/tools/gallery/src/worlds.rs`: what the new states need.
- `temper/docs/design/client/browser.md`, 7.2: the gallery's table gains its three states.
- `temper/docs/design/client/README.status.md`: "no Needs you or Chats" goes from its *3.5, Home*
  entry.

Options, each pre-approved:

- If a phrase's bound fails, it is derived: follow the failure, with its comment, and say so.
- If the summed node bound of `limits.rs` is too small for Home's ten rows, raise the term that adds
  them with a comment; it is not a budget.
- `home_outline_matches_the_requested_main_and_terminal_lines_join` and its neighbours follow the new
  order; a test that walked `to_begin` keeps walking it.

## Steps

1. **`pages/home.rs`:** `needs` and `chats`, each a function; `main` calls them. Read
   `root.pages().inbox()` and `root.pages().chats()`.
2. **Tests,** `tests/home.rs`: every state above, in both shells where they differ; each act's event;
   the node order against the request's; the terminal's print. Each rule-pinning test bites.
3. **Gallery,** then render at 1440, 1000 and 420 by eye, with a throwaway page; `account-everyday`
   does it by the book.
4. **Check,** `notes/common.md`: `heavy cargo nextest run -p temper-client-domain -p
   temper-client-view -p temper-client-shell-terminal -p temper-client-gallery -p
   temper-client-world`, clippy on the same, push. No journey draws Home, so nothing reprints.

## Report

As `notes/common.md`, Your report, every state above with where it renders.
