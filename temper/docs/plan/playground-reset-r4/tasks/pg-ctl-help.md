+++
id = "pg-ctl-help"
part = "playground"
kind = "docs"
needs = ["pg-ctl"]
files = [
  "temper/docs/help/reference/ctl-playground.md",
  "temper/docs/help/reference/ctl-forge.md",
  "temper/docs/help/playground.md",
]
effort = "medium"
minutes = 20
+++

## Goal

temper's help says what `pg-ctl` built: the two generated reference
pages of `temper ctl playground` and `temper ctl forge` as the tool
table now renders them, and the playground's concept page as the lead
wrote it below (draft `temper/docs/design/draft/playground-reset.md`,
6.1 to 6.3).

**Departure from the implementer's text, stated:** `docs/help/` is
the lead's (`temper/AGENTS.md`). Here the lead gives every word: the
concept page's whole text is in Steps, to be written byte for byte; the
two reference pages are generated, and their bytes are what the test
`every_ctl_group_page_is_what_the_table_renders`
(`temper/crates/temper/src/tests/commands.rs`) prints after "the page
should be:", copied from its output. Write no word of your own; if the
concept page below fails a rule of temper-docs (a link, a heading, a
kind line), fix only what the rule names and say so in the report.

Acceptance:

- `temper/docs/help/reference/ctl-playground.md` and `ctl-forge.md` are
  byte for byte the pages the test renders, and the test passes;
- `temper/docs/help/playground.md` is the text in Steps, step 2;
- temper-docs' rules and CLI help tests pass (`heavy cargo nextest run
  -p temper-docs`; `heavy cargo nextest run -p temper --lib -E
  'test(/ctl_group_page/) | test(/help/)'`).

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` (its "Not
  yours" line is departed from as above, for these three files alone).
- `temper/docs/design/draft/playground-reset.md`, 6.1 to 6.3.

## Owns and may change

Owned: the three pages above. Nothing else.

## Steps

1. Run `heavy cargo nextest run -p temper --lib -E
   'test(=tests::commands::every_ctl_group_page_is_what_the_table_renders)'`
   (check the count is 1; the path as nextest lists it). For each page it
   names, write the bytes it prints after "the page should be:" to that
   page; run it again until it passes.
2. Replace `temper/docs/help/playground.md` with exactly this text:

   ````markdown
   # The playground

   Kind: concept

   The playground is temper with nothing of yours: a repository on an in-memory forge and a scripted model, made when you choose it, so that a chat and its change run through real tasks. Leaving it resets temper to where it was before.

   ## What it is

   - **Its parts:** the playground's forge, a fake Forgejo held in
     memory, with the repository `playground/widget`; the scripted
     model, an account the deployment holds; and the project
     `Playground`, which adopts the repository. They are made when the
     playground starts, and removed when it is reset.
   - **What is real:** the engine, the worker and the agents' processes;
     the forge, its git and the model are fakes.
   - **The scripted model** answers three requests, as typed: `Add CSV
     export to widget` delegates a change to the repository;
     `Add a dark theme to widget` and `Fix widget's empty search state`
     each explain that nothing changes. Any other request is answered
     that it is not one of these. Only the playground's project uses
     it, and the playground's project uses no other model.
   - **Under a box,** the fakes run as one of its groups of child
     processes; elsewhere, the deployment must reach fakes of its own.

   ## How it changes

   - **Started:** `playground start` makes the forge, the account and
     the project, waits for the forge, and adopts `playground/widget`.
     It prints `running` once the playground runs. It is offered while
     no forge of yours is kept.
   - **Running:** a chat in the project `Playground` asks for one of the
     three requests (temper help chat). Every page says it is the
     playground, and offers nothing to add.
   - **Reset:** `playground reset`, or "set up temper" on any page's
     banner, deletes the project with its chats and tasks, removes the
     forge and the scripted account, and leaves temper as it was before
     the playground started: the welcome opens at its Forge step. A
     reset cut by a restart goes on at the next start, and so does a
     playground the deployment finds running when it starts, since the
     in-memory forge kept nothing.

   ## What temper guarantees

   - **One call, again until done:** a start or a reset that fails
     prints the step that failed, and the same call again finishes what
     is not done, never repeating what is.
   - **No account of yours is used, nor changed:** the playground's
     project runs on the scripted model alone, and an account you added
     stays when the playground is reset.
   - **Its work is real work:** its chats, tasks and change have the
     states, acts and reads of any other (temper help task), until the
     reset erases them.

   ## Acting on it

   - **ctl:** `playground start` starts it and `playground reset` resets
     it, as a party with a role in the deployment; `chat start` in the
     project `Playground` asks it for a change; `task show` and `change
     show` follow the work.
   - `forge setup` makes the same repository, `playground/widget`, on
     the owned Forgejo of a box (temper help forge).

   ## Limits

   - `refused: not offered` (exit 2): the deployment has no playground,
     or a forge of yours is kept.
   - A failed start or reset exits 1, naming its step; make the same
     call again.
   - The scripted model does no work beyond its three requests.
   ````

   The reference pages' words are the tool table's (step 1); if the
   concept page names a ctl call the table does not have under that
   name, keep the page's words and say so in the report.
3. The tests of the acceptance, one at a time. Commit, push.

## Report

fab's sections; any rule of temper-docs that made you change a word,
with the word.

## Answers

2026-11-01. Who repairs the E0004 in `temper/tests/client/src/translate.rs`
(`unsupported_people` lacks `ProjectClosed`, `ProjectGone`, `TaskErased`,
added by `pd-people-gone`) that stops step 1 from compiling? The answer
call repairs it: three arms in the function's own pattern,
`people::Event::<Variant> { .. } => panic!("the client kit does not make
People::<Variant>")`, after `Act`. If your base already has them, own
nothing more. If it does not, add exactly those three arms and nothing
else in that file: this task owns `translate.rs` for that alone, and the
report names it under `## Interfaces` as a compile repair. Any other
compile error outside the three pages is a question again.
