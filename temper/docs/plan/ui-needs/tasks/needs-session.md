+++
id = "needs-session"
part = "needs-state"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/pages/needs.rs",
  "temper/crates/temper-client-domain/src/tests/pages/needs.rs",
]
effort = "high"
minutes = 70
+++

# What the client keeps of what needs the person

## Goal

`Pages` keeps Later, Answered, At, the reply form's card, the held send with its Undo, and the toast,
for a session, with the deadlines and the acts that move them. Domain only: no view reads it until
later tasks. The held send goes in with the state, ahead of the pages, so that the pages draw it from
their first commit and nothing sends at once and then changes.

Acceptance:

- `pages::needs` exists as `temper/docs/plan/ui-needs/notes/contracts.md`, *What the client keeps*,
  has it, and `pages::Act` has `Needs(needs::Act)`.
- Every rule of `temper/docs/design/client/README.md`, 3.5, *What the client keeps*, but the rail's
  count and the drawing, holds in step tests, each named in the report, each biting.
- Home's `Facts.needs` counts the open cards, not `inbox.needs()`.
- The crate's existing tests pass unchanged, but the arms the compiler asks for.

## Inputs

- `temper/docs/plan/ui-needs/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5, *What the client keeps*, and 3.6.

## Owns and may change

Owns the two files in `files`. May also change, for these edits alone:

- `src/pages.rs`: the `mod`, the field and its accessor, `Act::Needs`, its route in `step`, the session
  in `fire`, `next_deadline` (it stops being `const`), `told`, `went`, and `link_changed`.
- `src/acts.rs`: `allowed` to `pub(crate)`, and the arm the compiler asks for.
- `src/pages/home.rs`: `facts`' count.
- `src/tests/pages.rs`: the `mod needs;` line; `src/tests/pages/home.rs`: a test of the count.
- `temper/docs/design/client/README.status.md`: take "`Pages`' session state and its held send, the
  toast" out of its 3.5 entry's words, as far as this task closes them; the drawing of the toast is
  `needs-frame`'s, so leave "the toast" if you like and say so.
- Any `temper-client-view` file, for the arm the compiler asks for in a `match` on `Act`, if there is
  one; that view draws nothing new.

## Steps

1. `src/pages/needs.rs`, its module comment citing the design. Sketch of its behaviour, binding:

   - **`Choose`:** only on an inbox card (`pages.inbox`, never a task's or a chat's) that is a
     `ReadValue::Question` with `ObjectKey::Question { task, message }`, a project, no `card.key`, and
     `acts::allowed(page, ActKind::Answer)`, and only with words `Text::new` takes at `CUE_BYTES`, as
     `Act::CardWords` does. Otherwise it does nothing. It pushes an `Answered` with its held answer
     and the deadline `env.now + HOLD`, `at` `env.wall`, title `question.task.title`; sets the toast
     (`Toasted::Answered`, `env.now + TOAST`); and takes the object out of Later, `at`, the reply
     form and `refused`.
   - **`OpenReply`, `CancelReply`:** one card's form at a time; `CancelReply` closes it and clears
     that card's words.
   - **`SendReply`:** as `Choose`, with the card's `words` (bytes trimmed of ASCII whitespace, none
     left: nothing happens) and `Label::Reply`; the card's words are cleared, and kept in the held
     answer. The cut counts characters by UTF-8 lead bytes, so it never splits one: at most 60 whole,
     past that the first 57 and "…".
   - **`Later`:** marks it; takes it out of `at`, the form and `refused`; sets `Toasted::Later`.
   - **`Undo`:** only while held: drops the line and the toast if it is this object's, sends nothing;
     a reply's words go back to the card's words where the card is on the page.
   - **`Open`:** sets `at`, then `Request::Show` of `Address::Inbox` filtered by the project of
     `Address::Home { project }`, or `Filter::ALL` off Home.
   - **`fire`:** at each deadline passed: the toast goes; a held answer goes as `Ask::Answer { project,
     on: AnswerOn::Question { task, message, words } }` with `Key { asker: Owner::Pages.asker(),
     number }` from `pages.serial`, as `acts::send` makes one, built from what the line kept, since
     the person may be on a page with no card; its line stays and is no more held.
   - **`told`:** a `Told::Answered` for such a key is taken before `acts::told`. `Done` leaves the
     line. Any other outcome removes the line and keeps the outcome in `refused`, the card open again.
   - **`went`:** leaving for any address but `Inbox` clears `at`; any change of address closes the form.
   - **`link_changed`,** on `SignedOut` or `OutOfDate`: the session is `Session::default()`, a held
     answer dropped unsent.
   - **Bounds:** `Answered`, Later and `refused` are `skein_lib::List`s of a const capacity, 64, with
     its comment. `Answered` full: the oldest line no more held goes; every line held: the act does
     nothing. Later or `refused` full: the oldest goes.

2. `src/tests/pages/needs.rs`, in the shape of `src/tests/pages/inbox.rs` and `src/tests/acts.rs`, at
   the least:
   - a choice: no `Send` before 5 s, the card out of `open`, the line and toast there, `held` true,
     `next_deadline` the earlier of the two; at 4.5 s the toast goes; at 5 s one `Ask::Answer` with the
     choice's words, `held` false;
   - the send goes though the person has gone to `Chats`, with no inbox on the page;
   - Undo inside 5 s: nothing sent, ever, the card open again in its place; Undo at 5 s: nothing;
   - a reply: the words sent, the label's cut at 60 and 61 characters and across a multi-byte
     character; blank words do nothing; Undo returns its words to the card;
   - two choices on two cards inside 5 s: two deadlines, two sends, in order;
   - refused: the line gone, the card open, `refused` the outcome; a second answer clears it; `Done`
     keeps the line;
   - Later: last in `open`, `shown` moves on, `later` true, the toast; marked alone, still shown; it
     still counts in `open`;
   - `at`: `shown` is that card until answered or marked; `went` away clears it; an `at` no longer open
     falls to the first;
   - `answered`: the window at its edge, the project's filter, the order;
   - sign-out drops a held answer unsent and every field; `next_deadline` is then none;
   - a choice on a proposal, on a task page's card, on a card with `key`, or with words past
     `CUE_BYTES`: nothing happens;
   - the 65th answered line: the oldest no more held goes, none past 64;
   - Home's `Facts.needs` falls as a card is answered.

3. `heavy cargo nextest run -p temper-client-domain --lib -E 'test(needs)'`; the whole `--lib` of the
   crate; `heavy cargo check -p temper-client-view --tests`, `-p temper-client-shell-web`, `-p
   temper-client-gallery`. Then fmt, clippy and the checks of the implementer's text. Commit and push.

## Report

As the implementer's text asks, and which `Outcome` you met that the design's words did not name.
