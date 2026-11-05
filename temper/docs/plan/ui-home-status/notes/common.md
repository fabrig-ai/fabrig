# ui-home-status: the rules every task names

The request is `temper/docs/plan/ui-home-status/inputs/request.md`; the design this plan cut is
`temper/docs/design/domain/forge.md`, 5.6 and 5.7, `temper/docs/design/protocol/store.md`, sections 2 and 3,
`temper/docs/design/client/README.md`, 3.5 (*Home on any project*), 4.6 and section 8, and
`temper/docs/design/protocol/configuration.md`, 13.4. Read this note, then
`temper/docs/plan/ui-home-status/notes/contracts.md`, then your spec. Where this note and the implementer's
text differ, this note wins.

## Decide, don't ask

The user is asleep and takes the plan's judgement. There is no supported deployment: a stored format,
a protocol or a public interface may change freely, with no compatibility and no `Question:`. A small
edit your task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is
made and named in your report, not parked. A `Question:` is only for what truly blocks the task.

## Departures from the implementer's text

- **Design text:** the cut wrote the design. A task may edit the `docs/design/` status lines its spec
  names, as it closes them, and a design line a small edit needs; say which in your report. A draft's
  own text stays as it is.
- **`docs/help/` and generated files:** `docs/help/reference/*`, prints, goldens and counted baselines
  your change makes stale are yours to regenerate in the same task. Don't park on them.
- **A throwaway render script** is allowed in any task, untracked and gone before the push.

## `seen` is the count at `previous`

The request's first section defines `seen` as "the `landed` count at `previous`", then says a new sitting
sets it to the current `landed`, which would zero the count at the moment the person comes back. The
design keeps the definition: `next_seen` holds the count at `latest`, and `seen` takes it when `previous`
moves (`temper/docs/design/domain/forge.md`, 5.6). The tests hold the definition: a change landed between
two sittings reads as 1 at the second's first look.

## Reuse what `ui-changes` landed

The trunk holds main's Changes page. `queued`, `red` and the words of the foot are that page's, not new:

- the engine's `views/changes.rs` (`owns`, `changes_red`, `panel::ready`);
- `landing_log::landed`, the one place a landing is first seen;
- the view's `pages::changes::landing_line`, whose comment says "Home's too in time": one function serves both;
- `forge::changes::watch` and `pages/changes.rs`, the shape of a watch a page opens for its project.

## Reach for the mock

The target is `temper/docs/design/client/greenfield/` (`home.html`, drawn by `home` in `app.js`; `mock.css`;
the words in `data.js`). Open its pages from the checkout; nothing needs extracting. Reach for it, not the
old page. Where old tests walk a part the mock removes, change them with it.

## Things that conflict across plans

Other plans (`ui-needs`, `ui-chats`, `ui-welcome`, `ui-settings`, `ui-goal-page`) run beside this one and
touch the same files.

- **`temper.css`:** new rules go under a marker comment, `/* ==== ui: home ==== */`, never at the end of
  the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt` and `tests.rs`:** on a catch-up, regenerate rather than
  hand-merge, and keep `names.txt` sorted as it is.
- **The store's bytes:** this plan takes 36 and 37. If another plan has taken one first, take the next free
  and say so in your report; the design's table moves with it.
- **The intro:** `ui-away` chains after this plan and adds its words to the same intro. Its composition
  stays in one function (`notes/contracts.md`).

## Not this plan's

- Settings' "The brief on Home": the agent always runs for now.
- "Turn on drafts and goals": drawn only once `Settings` has that switch.
- Away's words: plan `ui-away`.
- Connected on Home: gone by the look request `home-drops-connected`.

## Render proofs

Chromium 147's new headless won't go below 500 px by `--window-size`: a 420 px proof uses CDP's
`Emulation.setDeviceMetricsOverride`. Handle `Page.javascriptDialogOpening`, or a pending request's
`onbeforeunload` hangs a CDP driver. The mock's Google Fonts link needs the network; the client's own fonts
are what's compared. Screenshots go to `~/src/fabrig/ui-home-status/` on your builder, outside any worktree,
never committed; your report names the builder.

## Your report

As the implementer's text, with: the status lines you closed; each small edit outside your file list; the
options you took.
