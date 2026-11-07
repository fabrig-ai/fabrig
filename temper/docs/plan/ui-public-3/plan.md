# ui-public-3: the plan

The request: making a project public says what will show, and the public pages lose their last loose words (`inputs/request.md`). Titles stay; public reading stays off, and nothing here turns it on.

## Parts and tasks

| Part | Task | Needs | What it does |
|---|---|---|---|
| says | `ctl-and-pages` | | ctl's answer and help say what may show; a landing draws no zero; the board's Landings reads as a link |
| says | `chat-and-help` | | the configuring chat's instructions and the stock's fit figures; the walk and the project page |

## Critical path

None to speak of: the two run side by side, each alone. They share only the words, fixed in `notes/contracts.md`.

## Where a project goes public

The survey the request asks for, by `SetVisibility` and `visibility` across temper and jig:

- **`temper ctl project amend --visibility public`** is the only client that sends it (`temper-client-tools/src/jig/runner/acts.rs`). It gets the words in its help and its answer.
- **Nothing stands between that command and the act.** ctl has no confirmation, so a person who hasn't read its help meets the words only in its answer, once the project is public. The request allows this and asks that it be said. `temper/docs/design/client/tools.md`, section 14 records it as Later, with a flag as the way to close it.
- **The configuring chat can't do it.** Its tools are the settings, read, note and recall families, and it is offered no such act. Its instructions said nothing about it, so they gain a paragraph: never propose it, say its owner does it outside this chat, and say the words.
- **A proposal can't carry it.** No proposal's item holds visibility, so there is no proposal to put the words in.
- **No page offers it.** Neither `temper-client-view` nor `temper-client-domain` sends it.
- **jig's request itself** can be sent by any client an owner signs in with. Only ctl does so among temper's.

## Decisions

- **The words go on ctl's stderr, not stdout.** stdout stays the answer scripts read, and `browser_public`, under repair by `ui-public-story`, asserts nothing follows the page line. Under `--json`, stderr is one JSON object a line (`tools.md`, 6.2), so the words go in `{"may_show":...}`. That key is a public interface, free to add under the request.
- **The chat gets the words even though it can't act.** The request names its instructions, and it is where a person is likeliest to ask. It may not send anyone to `temper ctl`, so it says the owner does it outside this chat and names no command.
- **The link keeps its word and its heading.** `browser.md`, 8.3 has "Landings" as the link, so only the sheet changes, to the mock's `.link`. No browser story's words change, so `browser_public` stays untouched.
- **A landing draws no zero, on any landing,** not only a stock queue's: a landing by turn with none has nothing to explain either. This settles `forge.md`'s open item, whose default was to show it. A count above zero stays as it is.
- **The chat's 305 bytes move `standard`'s brief** from 356,270 to 355,965, because charter 3 is the stock's largest and sets the room. Nothing else in the fit moves by design, and a whole spec still fits.
- **`people.md`, 13.2 is corrected in passing:** it said making a project public shows "never its name", against 13.3, where the name crosses. `configuration.md`, section 11 said "charter 1" for a figure charter 3's room sets.
- **`browser.status.md` never listed titles as open,** so nothing leaves it. The decision stood in `first-release.status.md` as a **Design departs**, which now records the reaffirmation and its condition.
- **The walk's summary changes:** "without exposing its private words" promised too much, now that a title is known to carry a chat's words.

## Overlaps with the drafts beside this one

Neither task could avoid these. No order helps, since both need to land together and the others are in flight:

- **`configuring.txt`, `temper-service`'s fit figures:** `model-catalogue-2`'s `configuring-learns` (the same file) and `stock-models` (`tests/stock*.rs`). Whoever lands second recomputes the figures from the tests, as `chat-and-help` says. The numbers conflict; the paragraphs don't.
- **`temper.css`:** `look-round-13` and others, in other markers; this replaces one rule under `ui: public`.
- **`browser.md`, `tools.md`, `agent.md` and their status files:** `look-round-13/look-proof`, `model-catalogue-2/models-proof`, `goal-delivery/goal-walks`, in other lines.
- **`temper/docs/help/`:** `look-round-13/look-proof`, `model-catalogue-2/models-help`. Neither lists these two pages by name, though `look-proof`'s glob covers them.

`browser_public.rs`, which `ui-public-story` is repairing, is untouched.
