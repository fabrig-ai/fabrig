+++
id = "question-whole"
part = "question"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/pages/inbox.rs",
  "temper/crates/temper-client-domain/src/pages/inbox/whole.rs",
  "temper/crates/temper-client-domain/src/question.rs",
  "temper/crates/temper-client-domain/src/words.rs",
  "temper/crates/temper-client-domain/src/tests/pages/inbox.rs",
  "temper/crates/temper-client-domain/src/tests/question.rs",
  "temper/crates/temper-client-domain/src/tests.rs",
  "temper/crates/temper-client-view/src/pages/ask.rs",
  "temper/crates/temper-client-view/src/pages/needs.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/needs.rs",
  "temper/crates/temper-client-view/src/tests/cards.rs",
  "temper/crates/temper-client-tools/src/tests/jig/reads.rs",
]
effort = "high"
minutes = 100
+++

## Goal

A goal's question past 512 bytes ends mid-word under Needs you, with no sign, and shows `Choice:` and `Saw:` as plain text: the engine's person-task read cuts its words at `documents.text_bytes`, and `question::parse` treats cut words as no line form. The design has buttons for these choices, so they aren't plain by design. Read the whole through `TaskText`, the plainer of the request's two ways: `pages/goal/specification.rs` and `pages/chat/request.rs` already do, and carrying it would change the wire. Then the card and the goal's band draw the question whole with its choices, a question that stays cut says so, and the funds card says what a person can do.

Acceptance:

1. Under Needs you and on its goal's band, a person task in words whose document cuts its words draws the whole once read: QUESTION, context, "What the agent saw" and its choices as buttons, each answering on a press as 3.5 has it. The answer goes out as `AnswerOn::Person`, unchanged.
2. Until the whole has come, it reads as words in no line form, ending in `…`, with "Reply…" alone: no choice is pressed on a part.
3. A read that ended short: the card draws "This question is cut here." and the link "Read the rest" to `Requester::Task`'s page, and the next refresh of the inbox asks again. A `message` question whose `words.cut` is set draws the same, linking to its task's page. No card draws cut words with neither (`temper/docs/design/client/README.md`, 3.5, *Cut*).
4. Words whose document isn't cut send no `TaskText`, and a person task not in words never does. A role's, a verdict's and an approval's card draw as they do now.
5. A spec whose parameters alone are cut (`task.spec.cut`, its words' `cut` false) parses by the line form: its words are whole.
6. SHORT, on Home and next-up, stays the first line of the document's bounded words. Needs you's heading and the band's QUESTION take `parsed.question`.
7. A goal's page draws one question whole, its first; a later one draws its QUESTION from the bounded words and "Answer". More than 16 `Choice:` lines read as words in no line form, whole.
8. The funds card reads as `notes/contracts.md`, *The funds card's hold*, has it, in the browser and in ctl.
9. A person task's question whose words pass `documents.text_bytes` runs through the real engine in the client world: `a_goal_from_the_sheet_asks_its_person_who_answers_under_needs_you` asks one and answers it by a choice's press.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *Needs you*: *By its kind*, *Whole*, *Cut*, *SHORT*; 5.4, the lines on a chat's first message and what the view prices; 5.8, *Needs you bands*.
- `temper/docs/design/domain/agent.md`, 8.2, *A question's line form*.
- `temper/docs/plan/ui-goal-asks-2/notes/contracts.md`: *New words*, *The funds card's hold*.
- `temper/docs/plan/ui-goal-asks-2/notes/common.md`, which gives you your **Not yet** entry in `temper/docs/design/client/README.status.md`, the first of the two naming this work: remove it.

## Owns and may change

**Owns:** the files in the header.

**Readers and neighbours, each for the stated edit only:**
- `temper/crates/temper-client-view/src/words.rs`: your two words, as `notes/contracts.md` has them.
- `temper/crates/temper-client-view/src/pages/goal.rs`: `band`, for acceptance 1, 3 and 7 alone. `goals-start` changes this file's Tasks section at the same time, so keep to `band` and `band_controls`.
- `temper/crates/temper-client-view/src/pages/home.rs`: call sites of what `ask.rs` changes.
- Every `Card { .. }` literal gains the new field: `temper-client-domain/src/pages/{task,chat}.rs`, `src/tests/pages/{goal,chat}.rs`, `src/tests/fixtures/{task,acts}.rs`, `src/tests/forge_ways.rs`.
- `temper/tests/client/tests/suite/chat_draft.rs`: that story's question and its press alone. `goals-start` adds its own test at this file's end.
- `temper/crates/temper-client-shell-web/temper.css`, under `/* ==== ui: goals ==== */`, only if `.ctx` isn't drawn inside a band without it.
- Goldens and gallery prints a focused test reads, regenerated, if your words move them. `rg 'deciding its budget proposal'` found only the three tests in your header.

**Options, each pre-approved:**
- If `Inbox` needs `long_text`, hold it from `limits` at `Inbox::new`.
- If `Inbox::cards` would read many whole questions at once, leave it: the document reads it already holds are no smaller. Size a `List` by the first page's `total`, never `long_text` up front.
- If a page's whole read answers something other than a `TaskText` page, it ended short: acceptance 3.
- If the terminal draws a card by a path `ask::read` doesn't reach, leave that path and name it in the report.

## Steps

**1. Failing tests first,** as rows of what exists:

- `temper-client-domain/src/tests/question.rs`: `parse` on 16 and 17 choices.
- `temper-client-domain/src/tests/pages/inbox.rs`, on `the_inbox_watches_inbox_and_reads_each_entrys_object`'s fixture: a cut person task in words sends one `TaskText` from 0; paging to the end gives the whole; a page that doesn't fit the card (another task, a wrong `from`, past `long_text`) or a read that ends leaves it short, and a refresh asks again; a refresh with the whole in hand asks nothing; uncut words and a verdict's person task ask nothing.
- `temper-client-view/src/tests/needs.rs`: `example_parts_are_in_order_and_choices_keep_their_names_and_acts` and `plain_and_cut_words_have_no_choices_or_saw` gain rows: cut and reading; cut and whole, with the choices; cut and ended short, with the cut line and its link; a `message` question cut.
- `temper-client-view/src/tests/goal.rs`, on `bands_root`: the first question whole, a second by "Answer".
- `tests.rs` near 166, `tests/cards.rs` near 385, `temper-client-tools/src/tests/jig/reads.rs` near 757: the funds words.

**2. `temper-client-domain/src/pages/inbox/whole.rs`,** after `pages/goal/specification.rs`:

```rust
//! A person task's question read whole (temper/docs/design/client/README.md, 3.5, Whole).
#[derive(Clone, Debug)]
pub struct Whole { /* its handle, bytes, total, whether it reads */ }
impl Whole {
    pub(super) fn start(task: u64, handle: Handle, out: &mut Queue<Request>) -> Self;
    /// Takes one page, asking for the next; false for a handle not its own.
    pub(super) fn told(&mut self, task: u64, bound: u32, page: Readout, out: &mut Queue<Request>);
    /// The whole, once it has it all.
    #[must_use]
    pub fn words(&self) -> Option<&[u8]>;
}
```

Take `specification.rs`'s page checks as they are.

**3. `pages/inbox.rs`:** `Card` gains `pub whole: Option<Whole>`. `receive`, on a `ReadValue::PersonTask` where `question::in_words` and `specification.words.cut`: starts a read with a new handle from `self.next`, unless the card's `whole` holds the whole already. `add_card` carries `whole` across a refresh as it does `page`, restarting one that ended short. `leave` unwatches its handle. Route `Told::Page` to the card by that handle, and keep `known` and `pending` meaning what they do: a whole read isn't awaited by `known`.

```rust
/// What a card's question reads (temper/docs/design/client/README.md, 3.5, Whole, Cut).
pub enum QuestionWords<'a> { Whole(&'a [u8]), Reading(&'a [u8]), Cut(&'a [u8]) }
impl Card { #[must_use] pub fn question_words(&self) -> Option<QuestionWords<'_>>; }
```

A person task in words uncut, and a `message` question uncut, are `Whole`. A `message` question cut is `Cut`.

**4. `question.rs`:** `pub const CHOICES: u32 = 16`; past it, `fallback`.

**5. `temper-client-view/src/pages/ask.rs`:** `read` parses by `card.question_words()`, `cut` false for `Whole` alone. `Asked` gains what acceptance 3 needs, and `Asked.short` stays from the document's words. A shared function, `ask::whole_parts`, moves the context and Saw drawing out of `pages/needs.rs`, and adds the cut line; `needs.rs` and `goal.rs`'s `band` call it. The `…` is added by the view while `Reading`.

**6. `temper-client-view/src/limits.rs`:** one more `long_text` in `text_bytes`, and the nodes of 16 choices and the cut line, commented as "one question read whole". `the_browser_words_fit_the_views_limits` and its siblings still pass.

**7. `temper-client-domain/src/words.rs`:** the funds words, its segments split around the hole.

**8. The story,** `temper/tests/client/tests/suite/chat_draft.rs`: its question grows past the world's `documents.text_bytes`, its `Choice:` lines last, and it pumps until the buttons draw before pressing.

**9.** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain -p temper-client-view -p temper-client-tools -p temper-client-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-tools -p temper-client-shell-terminal -p temper-client-world`. The funds words change what shared `phrasebook` says, so `temper-client-tools` runs. Commit and push.

## Report

Say that it reads the whole rather than carrying it. Name the rules broken by hand and what failed, whether the band needed a CSS rule, and any terminal path left.
