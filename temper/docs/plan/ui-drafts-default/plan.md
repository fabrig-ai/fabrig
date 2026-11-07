# ui-drafts-default

The request is `inputs/request.md`: the configuring chat offers "+ Draft" unconditionally, and a new project has `drafts` on.
Two tasks, focused and fuzzy alone, complete when they land.

## Parts and tasks

- **`configuring-offer`,** task `configuring-offers`, needs nothing: the two gates, through a new
  `pages::chat_offers_draft`. Another chat keeps following `drafts`.
- **`drafts-on`,** task `drafts-on`, needs `configuring-offers`: the stock, its readers, the words, the prints, and Goals in the
  bar of a project holding a goal.

## Critical path

`configuring-offers`, then `drafts-on`. The second waits for the first because both write `temper-client-view` and its tests,
and the second regenerates what the two change together.

## Can a goal started there be found?

**Not from Home, so the request's small fix is in the plan.** The request asked to hear which it was.

- **From its chat: yes.** `chat_sheet::bar` draws "Goal TITLE →" from the person's `GoalFrom` fact, which reads no setting, and
  the chat's line says it too.
- **By its address: yes.** `Task { task }` and `Goals { project }` draw whatever kind 7 says, and on them Goals stays in the bar
  (`goals_current`).
- **From Home: not while To begin shows.** `pages/home.rs` draws Goals only when `!facts.to_begin() && root.has_goals(project)`,
  and the test `to_begin_keeps_chats` holds that. To begin shows while a step is left and it isn't skipped. A goal needs a
  repository in and a chat asked, which leaves step 1, `configured`: a configuring chat, or a setting changed. Until now a
  person had to turn `drafts` on before "Start goal…" appeared. On by default, a person who goes straight to a chat, writes a
  draft and starts a goal has neither. Home then shows To begin and Chats, with no Goals section and, Goals being hidden, no
  entry in the bar. The goal is reachable only by way of its chat's row.

**The fix,** as the request gives it: `frame.rs`'s `goals_shown` also reads `Domain::has_goals`, which holds on every page of the
open project, so the rail and the tab bar both show Goals. The setting hides an offer and not a person's own work. Home's
To begin keeps its shape.

## What was decided

- **The stock has a fourth writer** the request doesn't name: `temper-client-tools`' `settings.rs`, `stock`, which `settings show`
  prints. `drafts-on` changes it, and `docs/help/settings.md` and the `project-ways` form's line with it.
- **Until kind 7 is read,** Settings' controls draw the stock, disabled, so a fresh project's switch doesn't flip from Off to On.
  `drafts_offered` and the bar's entries stay absent until then. No code changes for it; 5.13 says so.
- **"Turn on drafts and goals",** a link in the design that nothing draws, is now "Show goals", since drafts are on.
- **`chat-draft` loses its toggle.** A recording holds no value for kind 7, so the client takes the new stock and the recorded
  switch has no control. Its first steps are cut from the recording and `again` records it, in place of a person's new walk
  (`temper/docs/design/client/journeys.md`, 8); a fallback is pre-approved in the spec. Other journeys are reprinted.
- **The mock departs:** its fresh projects have `drafts` false and its "+ Draft" follows that on every chat. That's a **Design
  departs** entry on the user's word, and the mock stays as drawn.
- **No format changes:** a `ProjectWays` record is the same size either way, and nothing stored is read differently. A project
  that stored `drafts` false keeps it; only a project with none stored changes.
- **No browser, walk or render check is named,** and no story is known to read the old default: the fake box's walks name their
  kind 7 outright. `again` is the one thing that starts a browser, and the request asks for it.

## Overlaps

None could be avoided; whichever lands second catches up, regenerating prints rather than merging them.

- `temper-client-view`'s `pages/chat.rs` and `tests/chat.rs`: `ui-model-line/model-line`, `ui-box-slots/box-words`.
- `frame.rs`, `tests/frame.rs`, `words.rs`: `ui-small-1/link-why`, `ui-small-1/small-words`.
- `temper-settings-form` and `temper-client-tools`: `model-catalogue-2/configuring-learns`.
- `temper/tools/gallery/**`: `look-round-13`'s tasks, `ui-small-1/small-words`.
- `temper/tests/client/tests/suite/configuring_chat.rs`, `forge/ways.rs` and the status files: `ui-proposal-2`, already merged on
  its trunk.
- `temper/docs/design/client/journeys/**`: `look-round-13/look-proof`, `look-round-13/welcome-ways`.
- `temper/docs/design/client/README.md` and `README.status.md`: `ui-proposal-2`, `ui-small-1`, `ui-model-line`, `ui-box-slots`.
