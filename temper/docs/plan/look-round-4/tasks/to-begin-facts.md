+++
id = "to-begin-facts"
part = "to-begin-facts"
kind = "feature"
needs = ["home-needs"]
files = [
  "temper/crates/temper-client-domain/src/pages/home.rs",
  "temper/crates/temper-client-domain/src/pages/chats.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/forge/dismissed.rs",
  "temper/crates/temper-client-domain/src/tests/pages/home.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chats.rs",
  "temper/crates/temper-client-domain/src/tests/dismissed.rs",
]
effort = "high"
minutes = 80
+++

# To begin's steps, computed, and "Skip" sent

## Goal

`temper/docs/design/draft/to-begin.md`, 3 to 5, in the domain: each step's done from the project's
facts, when To begin shows, and `Act::SkipToBegin` sending the `Dismiss` that `to-begin-store` built.
The view is `to-begin-card`'s. Every commit carries `Look-request: project-home-onboarding`.

Acceptance, each by a test of `tests/pages/home.rs` or `tests/dismissed.rs`:

- `Facts` and `Skip` are `notes/contracts.md`'s, What To begin reads.
- **`configured`** is true once a line of the `settings` document follows a change, as today, or a
  row of the project's chats is configuring (`pages::chats::configuring`). With no template 3 in
  `me` no row is, so a setting's change alone does it.
- **`chat_asked`** is true once the project holds a chat other than a configuring one, live or
  ended, as `Chats` reads them. A read cut past what is delivered (`live.more`, `after`) counts as
  another chat, as `asked` counts it today; a configuring chat past the cut isn't seen.
- **`delivered`** is true once the `forges` rows, the chats' live document and first ended page and
  the `settings` document are each delivered, or refused or ended; a refused or ended one counts,
  its step left. `Repository::Unknown` is not delivered.
- **`to_begin()`** is `delivered && skip == Skip::NotSkipped && left() > 0`.
- **`skip`:** `root.forge().dismissed(Dismissal::ToBegin { project })`: `Some(true)` is `Skipped`,
  `Some(false)` is `NotSkipped`, `None` is `Unknown`; `Skipped` as well while this Home's request is
  out, and once it is answered *dismissed*, until the page is left.
- **`Act::SkipToBegin`** on `Home` sends `forge::dismissed::ask` once, keyed from `pages.serial` with
  `Owner::Pages`, `name: None`. A second press while one is out sends nothing. Elsewhere, or with
  `ask` refusing its bounds, it does nothing.
- **Its answer,** in `Home::told`: *dismissed* ends it, skip stays `Skipped`; *unknown* ends it,
  the subject says; anything else (forge refused, core refused, `Limit`, `Lost`, `NotKept`,
  `Unconfirmed`) ends it with `skip_refused` true, so To begin shows again. A later press clears it.

## Inputs

- `temper/docs/plan/look-round-4/notes/common.md`, `notes/contracts.md` (What Home knows, What To
  begin reads).
- `temper/docs/design/client/README.md`, 3.5, *To begin*, and 4.6, *`Dismiss`*, *A person's
  dismissals*.
- `temper/docs/design/draft/to-begin.md`, 3 to 5.
- `git show --format=%B -s 9fcbec1d32`: the request's message.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-domain/src/{acts.rs,pages/chat.rs,pages/task.rs}`: the new `Act`'s
  arms, compiler-forced.
- `temper/crates/temper-client-domain/src/forge/family.rs`: reading `w::Outcome::Dismissed`, if
  `dismissed.rs` can't do it through what exists.
- `temper/docs/design/client/README.status.md`: from its *3.5, Home* entry, the end of the sentence
  on `Dismiss` having no sender and `dismissed` no reader once they have, and the last clause on
  `configured` and `chat_asked`. Nothing else.

Options, each pre-approved:

- `configuring`'s fallback: with no template 3 in `me`, or with a template 3 whose executor is
  `Chosen`, it is false.
- If `Chats::delivered` needs to tell a read that failed from one still out, keep what you need
  beside `page`, private to `Chats`.
- A `home.rs` view test that fails because `Facts` gained a field: arms and fixtures only.

## Steps

1. **`pages/chats.rs`:** `configuring`; rows read once by one private walk over `live` and `page`,
   giving `Chats::chat_counts(&self, client, me, project) -> (configuring: bool, other: bool)`, or
   your shape; `delivered`. `CONFIGURING_TEMPLATE` stays private to `pages`.
2. **`pages/home.rs`:** `Facts`, `Skip`, `to_begin`, `facts`; the skip's key and state in `Home`.
3. **`pages.rs`:** the act, its arms, `told`'s route to `Home`, which takes `env`'s limits if it
   needs them to read the answer.
4. **Tests:** each acceptance line; these bite:
   - `a_configuring_chat_marks_the_first_step_and_not_the_third`, with a chat row of charter 3;
   - `a_stock_chat_marks_the_third_step_alone`;
   - `to_begin_waits_for_every_fact`, each withheld in turn;
   - `a_skipped_project_hides_to_begin_and_a_refused_skip_brings_it_back`;
   - `a_skip_answered_before_its_row_arrives_stays_hidden`;
   - `another_projects_dismissal_leaves_to_begin`.
5. **A change of meaning:** `configured` and `chat_asked` change what they say, so run their
   readers: `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p
   temper-client-gallery -p temper-client-world`.
6. **Check,** `notes/common.md`, clippy on those packages, then push.

## Report

As `notes/common.md`, Your report.
