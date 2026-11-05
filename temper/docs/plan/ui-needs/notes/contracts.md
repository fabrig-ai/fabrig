# ui-needs: the shapes that cross tasks

The run's binding calls. A task builds to what is here; a later batch adds to this note and changes
nothing above it.

## A question's words

`temper-client-domain`'s module `question` (created by `question-parser`), by
`temper/docs/design/domain/agent.md`, 8.2:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    /// What the button says, and the answer's words: no prefix, no " (recommended)".
    pub words: Box<[u8]>,
    pub recommended: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parsed {
    pub question: Box<[u8]>,
    /// Empty where there is none.
    pub context: Box<[u8]>,
    pub saw: Option<Box<[u8]>>,
    /// Empty on the fallback: answered by a reply alone.
    pub choices: Box<[Choice]>,
}

impl Parsed {
    /// The choice drawn primary: the recommended one, else the first.
    #[must_use]
    pub fn primary(&self) -> Option<usize>;
}

/// `cut` is the read's `question.words.cut`.
#[must_use]
pub fn parse(words: &[u8], cut: bool) -> Parsed;
```

## What the client keeps

`temper-client-domain`'s module `pages::needs` (created by `needs-session`), by
`temper/docs/design/client/README.md`, 3.5, *What the client keeps*. `Pages` gets a field `needs:
needs::Session` and `pub const fn needs(&self) -> &needs::Session`. The view reads these, and binds
`Act::Needs(..)`; nothing else crosses.

```rust
pub const HOLD: Duration;            // 5 s
pub const TOAST: Duration;           // 4.5 s
pub const ANSWERED_ON_NEEDS: Duration; // 10 min
pub const ANSWERED_ON_HOME: Duration;  // 2 min

/// A variant of `pages::Act`: `Act::Needs(needs::Act)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    /// A choice's button: `words` are `question::Choice::words`.
    Choose { object: ObjectKey, words: Box<[u8]> },
    /// "Reply…", or Home's "Reply": opens the form under the card.
    OpenReply { object: ObjectKey },
    /// The form's "Cancel".
    CancelReply,
    /// "Send reply": answers with the card's words (`pages::Act::CardWords`).
    SendReply { object: ObjectKey },
    /// "Later".
    Later { object: ObjectKey },
    /// "Undo", on an answered line or the toast.
    Undo { object: ObjectKey },
    /// A Home line: opens Needs you at this card, filtered by Home's project.
    Open { object: ObjectKey },
}

/// What an answered line and the toast say; the view words it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Label {
    Choice(Box<[u8]>),
    /// Already cut: 57 characters plus "…" past 60. The view wraps it in `Replied: “…”`.
    Reply(Box<[u8]>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answered {
    pub object: ObjectKey,
    pub project: u32,
    /// The task's title; may be empty, and the view then says "Task N".
    pub title: Box<[u8]>,
    pub label: Label,
    pub at: Wall,
    // private beside these: the held answer, its deadline and its key
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Toasted {
    /// "Answered: LABEL", with "Undo" while `Session::held(object)`.
    Answered { object: ObjectKey, label: Label },
    /// "Later: it stays in Needs you, after the others."
    Later,
}

impl Session {
    /// The open cards: `inbox.needs()` less the answered, in order, those marked later last.
    #[must_use]
    pub fn open<'a>(&self, inbox: &'a Inbox) -> Box<[&'a Card]>;
    /// The card shown on Needs you: `at`'s where it is open, else the first open.
    #[must_use]
    pub fn shown<'a>(&self, inbox: &'a Inbox) -> Option<&'a Card>;
    #[must_use]
    pub fn later(&self, object: ObjectKey) -> bool;
    /// The form is open under this card.
    #[must_use]
    pub fn replying(&self, object: ObjectKey) -> bool;
    /// Its answer is held: its line and the toast offer "Undo".
    #[must_use]
    pub fn held(&self, object: ObjectKey) -> bool;
    /// What came back for an answer that did not go through: the card is open again.
    #[must_use]
    pub fn refused(&self, object: ObjectKey) -> Option<&jig_client::Outcome>;
    /// The lines answered within `window` of `now`, in `project` (None: every project), oldest first.
    #[must_use]
    pub fn answered(&self, now: Wall, window: Duration, project: Option<u32>) -> Box<[&Answered]>;
    #[must_use]
    pub fn toast(&self) -> Option<&Toasted>;
}
```

- **The view's clock:** `Session::answered` takes the view's `times.now`; the view sets `times.next`
  to when the oldest shown line goes, as Home's kicker does for its minute.
- **The domain's clock:** `HOLD` and `TOAST` are deadlines on `env.now`, through `pages::fire` and
  `pages::next_deadline`; `Answered::at` is `env.wall`.

## The standing count

Added to `pages::needs` by `needs-frame`:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Count {
    pub count: u32,
    /// The document was cut: the view says "N+".
    pub cut: bool,
}

/// The standing watch, of the same asker as `me_handle` and `accounts::handle`.
#[must_use]
pub fn handle() -> Handle; // number u64::MAX - 1

impl Session {
    /// The needs' rows of `project` in the standing document, less the answered; zero before it is
    /// delivered or once its watch ended.
    #[must_use]
    pub fn count(&self, client: &Client, project: u32) -> Count;
}
```

## A card, as the view reads it

`temper-client-view`'s module `pages::ask` (created by `needs-page`, grown by `needs-kinds`, used by
`home-card`). It holds what Home and Needs you share, Home's `need_title`, `need_when` and `need_kind`
moved in:

```rust
pub(crate) enum Kind { Question, Proposal, Held, Funds { spent: u64, limit: u64 }, PersonTask }

pub(crate) struct Asked<'a> {
    pub card: &'a Card,
    pub kind: Kind,
    /// The task's title, or "Task N" where it is empty, or the kind's words with no task.
    pub title: Box<[u8]>,
    /// A question's words as read; `None` on the other kinds.
    pub parsed: Option<temper_client_domain::question::Parsed>,
    /// The design's SHORT.
    pub short: Box<[u8]>,
}

pub(crate) fn read(card: &Card, w: &Words) -> Asked<'_>;
/// The mono line: TITLE (a link), kind, age, " · later" where `later` asks, and the project's name
/// first where `project` names one.
pub(crate) fn meta(b: &mut Builder, asked: &Asked<'_>, project: Option<&[u8]>, later: bool, w: &Words, bindings: &mut Bindings, times: &mut Times);
/// The `.answered` lines: a tick by style, "TITLE:" a link, the label, "Undo" while held.
pub(crate) fn answered_lines(b: &mut Builder, root: &Domain, lines: &[&needs::Answered], w: &Words, bindings: &mut Bindings);
/// The reply form, drawn only while `Session::replying`.
pub(crate) fn reply_form(b: &mut Builder, root: &Domain, asked: &Asked<'_>, w: &Words, bindings: &mut Bindings);
```

The label's words, `Replied: “…”` around `Label::Reply`, are one function of `pages::ask`, which the
toast (`frame.rs`) calls too; `needs-frame` makes it in `frame.rs`, and `needs-page` may move it.
