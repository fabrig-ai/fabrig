+++
id = "joint-world"
part = "work"
kind = "feature"
needs = ["rail", "work-page", "readers"]
files = ["temper/tests/client/tests/suite/work.rs", "temper/tests/client/tests/suite/main.rs"]
effort = "high"
minutes = 120
+++

# The client's joint world: Work across two projects

## Goal

One story in `temper-client-world`, the client joined to its real engine root: a
person in two projects, each with work of its own, opens Work on the first, sees
that project's work alone, moves among Work's filters without a read again,
switches to the second project and sees its work alone, Work's filter kept.

Acceptance:

1. `temper/tests/client/tests/suite/work.rs` holds the story, a focused test, beside
   its siblings' fixtures.
2. The person is a member of two projects; each project holds a tracked goal and a
   chat of the person's with a document, and the first a change on its way to
   landing as well, where the world's fixtures make one (`change_panel.rs`'s), each
   made through the engine as the sibling stories make them.
3. On the first project's Work, the person finds Work's head and filters, the goal's
   and the change's cards and the draft's, and nothing of the second project's.
4. Pressing "Drafts", then "Goals", then "All", each a move, sends no read again
   (the world's request count, or the domain's open handles, the same before and
   after); "Chats" opens the Chats page.
5. Choosing the second project in the switcher shows its Work under the same
   filter, its own goal and draft, and none of the first's; the first project's three
   reads are released.
6. `heavy cargo nextest run -p temper-client-world -E 'test(/work::/)'` passes and
   selects the story; its CPU time is said.

## Inputs

- `temper/docs/plan/ui-p1-rail/notes/common.md`; `contracts.md`, C2 and C4.
- `temper/docs/design/client/README.md`, 3.5, *Work* (*Its reads*, *Its filters*,
  *All*); 5.3, *Projects in the rail*.
- `temper/docs/design/domain/testing.md`'s worlds; the sibling stories that make a
  second project, a goal and a change through the engine (`temper/tests/client/tests/suite/`:
  `settings.rs`, `task.rs`, `change_panel.rs`, `chat_draft.rs`), and
  `temper/tests/client/src/fixtures.rs`.

## Owns and may change

Owns: the new suite file; `mod work;` in `tests/suite/main.rs`, in order.

Reader and neighbour files: `temper/tests/client/src/fixtures.rs`, a fixture the
story needs that two siblings could share, added beside its kind.

Not this task's: product code. A behaviour the story finds otherwise than 3.5 says
is a question with the story's output.

Pre-approved options:

- **If the world can't put a change on its way in one project cheaply:** keep the
  goal and the draft, check the change's card in the view's tests instead, and say
  so.
- **If the switcher's press can't be found by the fake person by its name:** open
  the second project's Work by its address, as the world's other stories open pages,
  and check the switcher's summary names it.

## Steps

1. Build the world: two projects, the person in both, their goals, drafts and the
   change.
2. Write the story's steps and checks as acceptance 3 to 5.
3. Break one rule by hand (a move between Work's filters reopening a read) in a
   scratch edit of `pages.rs`, not committed, and see the story fail; restore.
4. **Check** as `notes/common.md` says. Commit and push: the message says Work holds
   one project's work and keeps its reads across its filters.

## Report

Follow the implementer's text. Also give the story's CPU time and the rule you
broke and the failure you saw.
