# ui-drafts-default

The request is `inputs/request.md`: the configuring chat offers "+ Draft" unconditionally, and a new project has `drafts` on.
Two tasks at most, focused and fuzzy alone.

## Parts and tasks

- **`configuring-offer`,** task `configuring-offers`, needs nothing: the two gates, through `pages::chat_offers_draft`.
- **`drafts-on`,** task `drafts-on`, follows in the next push: the default, what reads it, and Goals in the bar of a project
  holding a goal.

## Critical path

`configuring-offers`, then `drafts-on`. The second waits for the first because both write `temper-client-view` and its tests,
and the second regenerates what the two change together.

## Overlaps

`ui-model-line` changes `pages/chat.rs`, `ui-small-1` changes `frame.rs` and `words.rs`, `ui-proposal-2` and `ui-box-slots` the
status files and `tests/chat.rs`. None could be avoided: whichever lands second catches up.

A stored format, a protocol or a public interface may change freely here: none does in `configuring-offers`.
