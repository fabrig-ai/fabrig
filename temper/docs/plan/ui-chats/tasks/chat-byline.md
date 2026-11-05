+++
id = "chat-byline"
part = "chat-byline"
kind = "feature"
needs = ["session-layout", "folds-away"]
files = [
  "temper/crates/temper-client-view/src/choice.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/new_chat.rs",
  "temper/crates/temper-client-view/src/pages/session.rs",
  "temper/crates/temper-client-view/src/tests/choice.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/src/lib.rs",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tools/gallery/names.txt",
]
effort = "high"
minutes = 75
+++

# The byline and its panel

## Goal

The composer's line and "Change model" move into the bar: the byline is the disclosure's summary, its body a panel under the
bar.

Acceptance, the request's section 3, item 1 and its phone's paragraph, and section 2's last line:

- `choice::byline` and `choice::above_composer`, as `notes/contracts.md`. A chat and `NewChat` read both; a configuring chat
  keeps its line with nothing behind it; `pages/agent.rs` keeps `composer`, unchanged.
- **The words:** "T4 · ", then account, " · ", model, from what `using_line` decides, with its "Using " and "Manage usage" left
  out: a plan "ChatGPT plan · gpt-6-luna", a key "OpenAI API key · MODEL", the playground's "Scripted model · Default".
  `ModelNeedsAccount`'s and the default's own words stay. `NewChat` has no "T4 · " and reads `new_chat_model`'s model.
- **The disclosure** keeps its name "Change model", its description the byline's words, `State::FoldsAway`, `BYLINE`. The browser
  shows the description alone, the same pixels as the mock's `.byline`, its hover the ink colour.
- **The panel:** the disclosure's body, under the bar, 320 px wide, shadow as `.temper-projects-menu`'s, closing on a press
  outside or Escape. "Manage usage" is its last control when the line named it.
- **Above the composer,** not in the panel: a pending request, its outcome, a task's wait for its account, a refused next
  run; with no usable model, "No model yet · Connect a model".
- **While the chooser isn't offered** (no models, a request out, a chat that can't change): `BYLINE` as a line, same words.
- **Phone:** `.session-top .right .byline` goes, and the disclosure's name shows instead, `.btn.quiet.small`, its panel still
  reachable and within the screen.
- **The terminal** prints the name, the byline's words and the content whole: the goldens change by that alone.
- **Gallery:** `chat-byline-open`, by the sample's way to open a disclosure in `notes/contracts.md`, *The gallery*.
- `README.status.md`: the remaining entry for 3.5, A chat's bar, and 5.5 goes.

## Inputs

- `temper/docs/plan/ui-chats/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chats/inputs/request.md`, section 3.
- `temper/docs/design/client/README.md`, 3.5, *The work's account and model*; 5.5, *The composer's line*.
- `skein/docs/design/ui-web.md`, 7, *Menus and disclosures*; how `.temper-projects-menu` is positioned, `temper.css`.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-view/src/tests/{chat,delegation,new_chat,playground,playground_start,welcome,connect_model}.rs`,
  `temper/tests/client/tests/suite/{choice,model,connect_model,accounts,accounts_edges}.rs`,
  `temper/tests/client-browser/**`, `temper/tests/one-process/**`, `temper/tests/real-loop/**`,
  `temper/crates/temper/tests/**`: a test that reads "Using …" under the composer. **A change of meaning runs its readers' tests.**
  Keep what each proves: that the line says the account and model.
- `temper/crates/temper-client-view/src/words.rs`, `limits.rs`: byline forms of the `Using*` words.
- The terminal's goldens; the journeys' prints.

Options, each pre-approved:

- The byline's forms are new words beside `UsingKey`, `UsingPlan`, `UsingProgram`; a program's reads "your Claude subscription,
  through Claude Code · MODEL".
- skein-ui's disclosure draws its name before its description in the summary: hide the name by css, visually, for readers keep it.

## Steps

1. Split `using_line` so that one function decides the words; `composer` still reads "Using …" from it for `pages/agent.rs`.
2. `byline`, `above_composer`; the readers. The panel's rules.
3. Renders of the byline closed and open beside the mock's byline at 1440, 1000 and 420. In a real browser, or the browser-tier
   story, press outside and Escape.
4. **Tests,** `tests/choice.rs`: each form of the words; the disclosure's name, description and state; a line when not offered;
   the feedback above the composer and not in the panel; no model. Each bites. Gallery, goldens, prints.
5. **Check,** `notes/common.md`, clippy, push.

## Report

As `notes/common.md`, *Your report*.
