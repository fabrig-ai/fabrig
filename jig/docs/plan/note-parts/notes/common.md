# note-parts: the rules every task names

The request is `jig/docs/plan/note-parts/inputs/request.md`: read *The change*, and *For every task of this plan*, which binds you
and is not repeated here. Where this note and the implementer's text differ, this note wins.

## Departures from the implementer's text

- **Design text:** the cut wrote the design. A task may edit only the status entries it closes, in the design status files its spec
  names. If the code teaches that a design line is wrong, make the small edit and name it in your report.
- **Decide, don't ask:** a `Question:` is only for what truly blocks the task. A stored format, a protocol, a tool or a public
  interface may change freely, in place, with no version bump and no compatibility. A small edit just outside your files (a re-export,
  a `mod` line, a design line, a golden) is made and named in your report.
- **Generated files** your change makes stale are yours, by their own tool, never by hand, `docs/help/reference/*` included.
- **Another tree's lines:** a branch stays in its tree, but for what a changed type forces; your spec names those lines, each edited
  only for that.

## The contracts

`jig/docs/plan/note-parts/notes/contracts.md` holds the shapes that cross tasks. They bind: use its names, fields and words. What it
leaves open inside a module is yours.

## Your report

As the implementer's text says, and:

- each new test's bite: the rule you broke and the failure you saw;
- any file you changed outside your spec's lists, and why;
- anything `temper/crates/temper-client-domain/src/pages/chat/document.rs`, `ui-chat-draft`'s `note_write`, would need, if you saw it.
