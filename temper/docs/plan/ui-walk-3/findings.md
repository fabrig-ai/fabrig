# Consolidated third-walk findings

The original walk/additions/real-login observations use client
`fa5b518db6093a96a90d3507a4e18ee1ed27eea6`; the focused rewalk uses
`7bb001fc71`. [report.md](report.md) names every source builder and folder,
selects the current pair for each walked state, and preserves the real steps.
Untouched W3/W6/A4/R1 evidence is carried, not represented as another run.
B1 is a new unresolved configuring-chat acceptance failure. N3 remains an unexplained original observation;
passing replays do not establish a fix.

| id | where (step, page, state, width) | seen | the mock or rule says | kind | size | owner | state |
|---|---|---|---|---|---|---|---|
| W3 | 5, Configure, project Needs you, Home and acceptance, 1440 and 420 | The exact fresh-box prompt produces a live proposal. The inbox row retains `project: 0` and now also has `projects: [1]`; the project projection and count use that membership. Accept changes the ways and clears the count. | Project settings proposals appear in each affected project's inbox, count and Home; acceptance updates the original chat card. | other | large | fabrig/ui | fixed, seen at both widths: exact cue on a fresh revision-1 box; `final-1440/05-configure-proposal-accepted-1440-pair.png`, `final-420-v3/05-configure-proposal-accepted-420-pair.png`; inbox JSON before and after acceptance. Focused rewalk: Needs you/Home decisions and cleared counts are seen again. The original accepted configuring card after Needs you acceptance remains carried. Direct Accept in that chat is separately not fixed, B1; the project-membership fix is not a claim that entry point works. |
| W6 | 3, Home after a real landing and 31 minutes away, 1440 and the same live box at 420 | Stock sitting and personal ChatGPT plan, without a funding edit. Separate unfunded box uses a shortened sitting and ctl period cut. | A funded return has sentence, lede and byline; a refused run leaves the computed landing sentence without brief prose, byline or error. | other | large | fabrig/ui | fixed, seen: `final-1440/03-home-brief-1440-pair.png` and the same live box at 420 after 1861.756 seconds; stock sitting and personal plan, no funding step. Separate unfunded rows at both widths have the minute-old computed landing sentence and no brief/byline/error. |
| N1 | 5, Accept proposal sheet; 4, Stop goal sheet, 1440 and 420 | The confirmation is pinned to the viewport's upper-left corner. At 1440 Accept is x=0, y=0, width=460, height=214.09375; computed margin is 0px, position fixed, inset 0px. At 420 it is x=0, y=0, width=388, height=242.5625. | `temper/docs/design/client/README.md`, 3.6, The sheet: the mock's modal sheet, narrower. The mock's start sheet is centred. No decision moves a confirmation to the corner. | look | small | fix-look | fixed `f5011fb0b6` (`N1: Centre confirmation sheets over every page`); red-first `confirmations::confirmations_keep_auto_margins_in_every_page` (missing auto margin), then green. Touched: Needs you and Home, Accept/Reject proposal sheets; Chat, Close/Start over sheets; Goal, Stop sheet; Task, Cancel/Stop/Release sheets, all at 1440 and 420. The original Goal/Stop pairs and JSON were rechecked at both widths: those sheets were already centred; the corner fault was reproduced in Needs you. New live Accept rectangles are x=490, y=392.953125, width=460 at 1440 and x=16, y=378.71875, width=388 at 420. Rewalk 7bb001fc71: centred live sheets at both widths (centres 720/500 and 210/500, or 712.5/500 and 202.5/500 on Home with its 15px layout scrollbar), all five cases and proposal entry points; task variants rechecked as static galleries, live Task acts unobserved. Fresh pairs and sheet rectangles: walk-again/; native-modal reference: walk-again-position/. |
| N2 | 2, Settings, How work runs, Status, 1440 and 420 | The disabled “The brief on Home” select says “Off”; the row describes what temper does today. The view hardcodes `SettingsOff`. | D11 disables these controls while describing current behaviour. The stock status agent is enabled and may write a brief; “Off” does not describe that behaviour. | word | small | fix-look | fixed `ba42299c12` (`N2: Show the enabled Home brief in Settings`); red-first `tests::settings::disabled_sections_say_so_once_and_their_controls_are_disabled` (Off instead of On), then green in both shells. Touched: Settings → How work runs, Status, The brief on Home disabled select, at 1440 and 420. Rewalk 7bb001fc71: seen On and disabled at both widths in walk-again-status/settings-status pairs and status JSON. |
| N3 | 4, Chat/Draft after working-note restoration, 420 | Start goal… is offered on the first document turn, then absent after the working-note round trip although the restored document is visible. No error explains it. Two earlier fresh phone boxes reached the sheet. | The same chat still has a nonempty document, enabled drafts and the home repository that made the first offer valid (`temper/docs/design/client/README.md`, 3.5). Its goal entry must stay available. | link | small | fix-flow | not reproduced: four fresh fake boxes, three at 420 and one at 1440, using the exact working-note cues and reopening T1 through Chats after each. Start goal… stayed enabled after restoration and opened the sheet in all four; no N3 code change. Existing tests `tests::pages::chat::a_page_of_working_notes_does_not_hide_a_later_document` and `tests::chat_sheet::start_goal_shows_when_it_may` pass. Rechecked: Chat/Draft, first document → working notes only → restored document after three notes pages → Start goal sheet, at 420 and 1440. Evidence on debian-32gb-hel1-14: `~/src/fabrig/ui-walk-3/fix-flow/draft-420/`, `draft-420-repeat/`, `draft-420-third/`, `draft-1440/`; each has `04-draft-*` and `04-start-goal-sheet-*` pairs and public notes JSON. The original failed observation remains above; these replays establish no cause. Rewalk 7bb001fc71: not reproduced at either width; document and Start goal sheet captured after all working-note pages. The original failed observation remains unresolved; no N3 fix claimed. |
| A2 | Rename form, 1440/420 | Row replaced by 38px field, min-height 34px, primary Save; no visible Cancel or Rename act. | Selected-row form, `temper/docs/plan/ui-walk-3/notes/decisions.md`, final section; mock app.js 229–240. | look | small | fix-look | not reproduced (already fixed, seen): fresh fake-box replay and mock/client pairs rechecked on this branch at 1440 and 420; selected-row field 38px high, min-height 34px, primary Save, no visible Cancel or Rename act. Save/Enter save, Escape cancels, empty name retains the name; focus returns to summary with real-control rings retained. Evidence: `~/src/fabrig/ui-walk-3/fix-look/rename/`, `focus-metrics-1440.json`, `focus-metrics-420.json`. Rechecked: Project switcher, rename form and Save/Enter/Escape/empty-name return states, 1440 and 420; no code change. Rewalk 7bb001fc71: both widths, fresh selected-row form pair and focus-metrics JSON under walk-again/. |
| A3 | Rename Save/Enter/Escape, 1440/420 | Settled focus returns to SUMMARY; main/h1 outline none; next Tab reaches BUTTON with solid 2px ring. Empty name retains old name. | Return focus to opener/summary, retain real-control rings; same decisions section. | other | small | fix-flow | not reproduced (already fixed, seen): two fresh fake boxes rechecked Save, Enter, Escape and empty-name Save at 1440 and 420. Every settled sample returns focus to SUMMARY; main/h1 have no outline, and next Tab reaches a real project-row BUTTON with a solid 2px focus-visible ring. Escape and empty Save retain Entered project. No code change; existing `tests::frame::the_rename_form_binds_its_field_save_and_cancel` and `programmatic_page_targets_do_not_draw_control_focus_rings` pass in the focused suites. Rechecked: Project switcher, rename form → Save/Enter/Escape/empty-name return → next Tab, at 1440 and 420. Evidence on debian-32gb-hel1-14: `~/src/fabrig/ui-walk-3/fix-flow/rename/`, `focus-metrics-1440.json`, `focus-metrics-420.json`, immediate and settled samples, and `08-rename-*-rename-*` pairs. Rewalk 7bb001fc71: Save/Enter/Escape/empty Save return to summary; next Tab preserves 2px ring; main/h1 have none. Fresh both-width pairs and focus-metrics under walk-again/. |
| A4 | Needs you, immediate choice after Undo, 1440/420 | Ten answers sent; green receipts, no question/count, empty inbox and completed goal after answer. | Every choice sends after D5’s hold. | other | large | fabrig/ui | fixed, seen in 10 repeats; all event logs, inbox JSON, goal JSON and pairs retained. |
| M1 | close-chat, Escape on Close/Start over/Stop goal/Accept/Reject, 1440/420 | Escape leaves the sheet open. Keep dismisses it without action. | The additions task requires Escape/keep cancellation with no work changed; `temper/docs/design/client/README.md`, 3.6, The sheet. | other | small | fix-look | fixed `837716da2b` (`M1: Bind confirmation keep acts to Escape`; test subset repair `b0803db3ab`); red-first `tests::confirm::escape_cancels_a_confirmation_without_sending_its_action` (missing Escape shortcut), then green; all eight sheet variants check Escape on keep alone. Touched: Chat, Close/Start over; Goal, Stop; Needs you/Home/Chat, Accept/Reject; Task, Cancel/Stop/Release including uncertain release; sheet open → Escape/keep dismissed without action, at 1440 and 420. Live proof: 10/10 sheets across the five reported cases at both widths, task lists equal before/after Escape and keep; `fix-look/live-summary.json` on the evidence builder. Rewalk 7bb001fc71: five reachable cases plus proposal Home/Chat entry points (Chat Accept cancellation only; primary fails as B1), Escape and keep leave task lists equal at both widths; sheet JSON under walk-again/. Live Task cancellation is unobserved, not claimed fixed by a static gallery; shared-rule unit test and gallery look proof retained. |
| M2 | The two scroll bars, Older, 1280×720/1000×600 | First read remains Loading earlier messages after 40 seconds; zero completed loads and no second press. | The requested two completed Older loads; loading status only while a read is out (`temper/docs/design/client/README.md`, 3.5, Reading back). | other | large | fabrig/ui | large, moved; still open. No fix touched this path; prior observation and reproduction retained below. |
| M3 | close-chat, reject W3 settings proposal with reason, 1440/420 | Reject leaves the sheet open with Malformed operation and Needs you 1. Accept succeeds on the same fresh proposal shape. | The primary act sends the rejection and completes; refusal must describe a valid reason (`temper/docs/design/client/README.md`, 3.6). | other | small | fix-flow | fixed `9cd2f1ca00` (`M3: Omit the running-work choice when rejecting settings`); red-first `tests::acts::settings_rejection_sends_its_reason_without_a_running_choice` sent `Some(Stays)` on Reject instead of `None`, then green; Accept still sends `Some(Stays)`. Touched: Needs you, Home and configuring Chat, settings-proposal Reject reason → confirmation sheet → completed rejection/card marked Rejected and need cleared, at 1440 and 420. Live Needs you proof at both widths: fresh revision-1 proposal, reason retained in the sheet, Escape/keep leave tasks unchanged, primary Reject closes the sheet and marks the card Rejected with no Malformed operation or rail count. Evidence on debian-32gb-hel1-14: `~/src/fabrig/ui-walk-3/fix-flow/reject/`, `sheet-127-reject.json`, `sheet-147-reject.json` and `08-close-chat-*-reject-*` pairs. Rewalk 7bb001fc71: Direct configuring-Chat Reject is not offered under client/README.md, 5.13; it is unobserved and not claimed fixed there, with new unreachable state rows in walk-again-chat/. Reject from Needs you and Home completes at both widths on fresh revision-1 boxes; reason retained, card Rejected, sheet closed and proposal gone from the retained first needs page. walk-again/sheet-*-reject.json and proposal-*-{before,after}.json. |
| R2 | Real check; sheet's Run on this machine; fresh custom or plain cache; 1440 and 420 | The required UI action starts fetching Forgejo as well as the later Claude Code fetch. The screenshot's Forgejo progress advances; the custom box subsequently logs its Forgejo group started. | The real-login spec permits the pinned Claude Code download as its one real service. An empty cache plus the required Forgejo setup does not keep that boundary. | other | large | fabrig/ui | large, moved; still open. No fix touched this path; prior observation and reproduction retained below. |
| R1 | Real Model/Connect a model command, custom directories, 1440/420 | Shown command fetches the pinned Claude Code, reaches sign-in, then reuses the cached binary. | Command must operate on the serving box; every character must wrap inside the panel. | other | large | fabrig/ui | fixed, seen in real-login at fa5b518db6: correct custom-directory command, pin SHA-256, first prompt and cache reuse; no credentials or model call. No fix touched command/fetch, so this evidence is carried; R2 separately limits the service-boundary claim. |
| B1 | Configure, settings proposal Accept in its originating Chat, 1440/420 | Accept opens its sheet; primary returns No visible object with that name, leaves the sheet open and Needs you 1. Needs you/Home acceptance works. | client/README.md, 5.13: the card Accept sends the proposal decision; 3.6: a successful primary completes. | other | large | fabrig/ui | large, moved: cause and valid Chat decision scope/identity not established; requires domain repair beyond the permitted word/link/look fix. Repeated at both widths after Escape/keep; direct-primary follow-up and public inbox evidence retained in walk-again-chat/. |

## Reproductions and measurements

These retain the original failed observations. The state column above gives the current rewalk disposition.

N1 reproduction: fresh fake box, finish first run, open Configure temper before
changing settings, send “Show drafts and goals for this project.” Open the
project's Needs you and press Accept. The sheet's Not yet and Accept work, but
the sheet starts at the screen's top-left. The initial walk also reported it on Goal/Stop; fix-look's reinspection found that original sheet already centred. Fresh rewalk sheets are centred. Adjacent client JSON records its rectangle and
computed styles. This is a look fault, not a failed proposal decision.

N2 reproduction: finish first run, open Settings → How work runs and read the
Status section. Compare its “Off” with the funded Home return in the report.
The decision to keep the select disabled is retained; the finding is its word.

Rename, focus, A4 and account-flow evidence is consolidated below and in the result table.

W3 replay: a fresh box, ChatGPT first run, Configure from Settings, exact prompt
“Show drafts and goals for this project.” before any setting edit. Follow the
card through Needs you and Home; Accept, then the sheet's Accept. The count
disappears, Goals is in navigation, Drafts is in Chats, and the original chat
card reads accepted. The new `projects` membership explains why the old
`project: 0` envelope alone is no longer evidence of W3.

W6 replay: fresh box, personal ChatGPT sign-in, enable ways, first chat
“Land a local hello status change.”; Changes confirms the landing. Home,
Skip To begin, close its tab, keep the box running for at least 31 minutes
with stock `forge.sitting_ms`, reopen the observed Home address. The final
phone brief is the same live desktop state resized, via `--brief-from`.
For the unfunded variant use another fresh box, set only its sitting to
3000 ms before first run, land the same change, cut each role's period
allowance and the period through ctl before the post-landing Home visit, then
look at Home, close it and return once the landing is a minute old. The kept public project read has period left 0.

N3 reproduction: replay `walk.py --width 420` on a fresh box, enable the four
ways, and send “Let us design Away mode.” in its first chat. Press + Draft.
`final-420-v3/04-draft-first-turn-420-client.json` contains enabled Start goal…
and the first document. Add “Add what happens to a running goal.”, then the
two working-note cues from `notes/walk.md`, reopening T1 through Chats after
each. The restored document is visible and all three ctl notes pages are kept,
but `final-420-v3/04-draft-420-client.json` has no Start goal…; its PNG and
`errors.json` retain the stop. No setting, repository or goal changed in that
box between the two captures. The cause is unproved. The later goal and
Settings states use the earlier fresh `final-420-v2` box 2, explicitly named
on those rows; this failed observation is retained rather than erased by the
passing box.

## M2: the unresolved Older read

On a fresh fake box, enable the features and open a chat through the composer. Send 66 history messages with ctl (24 repeated reading sentences each), reopen T1 through Chats, and send six bounded composer paragraphs (four repeated sentences, as the browser reading-back story does). At 1280×720 and 1000×600, scroll to the top and press Older. The status remains Loading earlier messages throughout the 40-second observation. There is no second press while that read is outstanding. Both failing pairs, public activity and exact history counts are retained in `scroll-landed/`; replay with `driver/additions.py --section scroll` and fresh output.

The history fallback is pre-approved. The first large composer attempts and an early check that counted a still-pending read are retained as driver attempts, excluded from the final verdicts. The second completed load is unobserved, rather than passed. The ten-page `window_page` fixture in `temper/crates/temper-client-domain/src/tests/pages/chat.rs`, test `a_full_chat_window_offers_older_and_reads_it_when_asked`, fills eight pages and leaves two earlier pages; `chat-work-reading` is the named gallery for the loading appearance.

## Other reproductions

**M1:** open any live confirmation in the close-chat traces, bring the client tab to the front and press Escape. Its title and acts remain. Press the keep act: it closes. Public task lists remain unchanged by cancellation. `temper/crates/temper-client-view/src/confirm.rs`, `sheet_acts`, binds Cancel to the keep button without an Escape shortcut; that is the small view fix candidate.

**M3:** fresh box; Settings → Start; send “Show drafts and goals for this project.”; Needs you → Reject disclosure; fill the reason “Keep the current project settings.”; press its Reject button, then the sheet’s primary Reject. The reason is shown, but the sheet says Malformed operation. Reproduces at both widths in `close-chat-narrow/`, boxes 127 and 147. Passing Accept traces are in `close-chat-complete/`, boxes 126 and 146. Candidate: `temper/crates/temper-client-domain/src/acts.rs` attaches `Running::Stays` to both accepting and rejecting a settings proposal. The rejection’s running-choice field needs investigation; no protocol or client fix was made here.

## A4 repeats

Every row used a fresh box and the exact Later → Reply… → fill Your reply → Send reply → Undo → immediate offered choice sequence while the goal was active. No screenshot, mock action or deliberate wait occurred between Undo and choice. Intervals are measured from mouse release to mouse release. After the hold, public inbox and goal JSON were retained for every repeat.

| repeat | width | Undo → choice (seconds) | what it showed |
|---:|---:|---:|---|
| 1 | 1440 | 0.017028 | Green receipt; question/count gone; inbox empty; goal ended after answer. |
| 2 | 1440 | 0.015867 | Green receipt; question/count gone; inbox empty; goal ended after answer. |
| 3 | 1440 | 0.010441 | Green receipt; question/count gone; inbox empty; goal ended after answer. |
| 4 | 1440 | 0.017922 | Green receipt; question/count gone; inbox empty; goal ended after answer. |
| 5 | 1440 | 0.015398 | Green receipt; question/count gone; inbox empty; goal ended after answer. |
| 6 | 1440 | 0.016745 | Green receipt; question/count gone; inbox empty; goal ended after answer. |
| 7 | 1440 | 0.015297 | Green receipt; question/count gone; inbox empty; goal ended after answer. |
| 8 | 420 | 0.010029 | Green receipt; question/count gone; inbox empty; goal ended after answer. |
| 9 | 420 | 0.008202 | Green receipt; question/count gone; inbox empty; goal ended after answer. |
| 10 | 420 | 0.013328 | Green receipt; question/count gone; inbox empty; goal ended after answer. The direct 5.5s text sample caught a render gap; the settled screenshot/JSON shows the receipt. |

No “That answer wasn’t sent. Press it again.” toast was observed. The initial `a4/` run stopped on a driver-only ctl usage error; it is excluded from these ten.

## Focus numbers

`focus-metrics.json` combines the two final widths. Save/empty-name Save return to Saved project/Entered project SUMMARY with no mouse outline; Enter and Escape return to Entered project SUMMARY with a solid 2px keyboard ring. All next-Tab targets are project-row BUTTONs, focus-visible with a solid 2px ring. Main and h1 outline styles are none at every settled sample. The immediate Enter sample can precede the asynchronous focus return; both immediate and 400ms settled measurements are kept. A2’s input is 38px high with min-height 34px.

## Scroll numbers

`scroll-landed/scroll-metrics-WIDTH.json` holds document/body height, width and client dimensions at both ends. Each observed state has exactly one visible pane with overflow-y auto and scrollHeight greater than clientHeight. The draft pane is open but empty while its cue waits. The zero completed loads are M2’s limit.

| window | mode | document H/W | body H/W | visible scrollers | log scrollHeight/clientHeight | top at end / at top |
|---|---|---|---|---:|---|---|
| 1280×720 | Chat | 720/1280 | 720/1280 | 1 | 1130/550 | 580 / 0 |
| 1280×720 | Split | 720/1280 | 720/1280 | 1 | 1346/550 | 796 / 0 |
| 1000×600 | Chat | 600/1000 | 600/1000 | 1 | 1130/397 | 732 / 0 |
| 1000×600 | Split | 600/1000 | 600/1000 | 1 | 1634/397 | 1237 / 0 |

## Scope and limits

The live landing/agent fake paths expose no grouped call disclosure. `chat-work-done` and `task-solo-T9` supply the folded/open/folded gallery proofs, with native details toggled at both widths. The growing answer uses `chat-work-writing`; the live fake did not hold it through the poll. Latest is live: top stays 0 as new words arrive, then returns to 410/873 at 1440/420, equal to scrollHeight minus clientHeight, and the pill disappears. Loading earlier messages was captured live at both widths.

All nine close-chat rows are present at both widths. Close, Start over, goal Stop, Accept and Reject use live boxes; task Cancel, agent task Stop and release/uncertain release use named galleries after their fake/ctl attempts. Static confirmation buttons are look evidence; their effects are unobserved. A real owner hold was placed by ctl before the release fallback.

The Model page’s live choices, Claude check failure and ChatGPT Ready are retained. The remaining welcome/model/connection/paste endings are named galleries, not successful live trials. Own-way’s card, Not sure yet?, reference and all offered modes were walked on Hello and Fresh project at both widths. Start from it sends the project-specific words shown in the pairs; Not sure yet sends “Not sure yet. What can temper do?”. The fake’s “Hello from the local fake model.” answer is not a finding.


### R2: the check's service boundary conflicts with its required route

Start a real box with empty custom directories, open its owner link at 1440,
press Set up temper, Forgejo's Run on this machine, then the sheet's
Run on this machine. Before running the Claude command, Model already says
`Setting up: fetching Forgejo`; `custom-model-1440.png` shows 30%, and
`custom-model-420.png` shows 42%. The plain box repeats this in
`plain-model-{1440,420}.png`. `box-1.log` records Forgejo's group started at
09:02:27Z. No extra service probe was directed by the driver; the extra fetch
followed the required page action. This is a conflict in the check's scope,
not evidence that the product's Forgejo setup is wrong. Resolving which
service downloads this real check permits, or changing its required route,
needs the plan owner's decision; this observation-only task changes neither.


## B1: configuring Chat Accept

Fresh fake box, ChatGPT first run, Settings → Start, exact proposal cue before any settings change. Stay in the chat, Accept → Escape → Accept → Not yet → Accept → primary Accept. The sheet stays open with No visible object with that name, and Needs you remains 1, at both widths. The first command stopped on that refusal; its faults and pairs remain. The follow-up presses primary directly without earlier cancellation and records the result. Configure has no direct Reject under the written card contract; rejection is tested through Needs you and Home instead. No domain source change was made.

R1's conditions, command steps, measurements and limits are retained in [report.md](report.md), Real check — R1. Owed draft work is there in its own section, outside the findings table.
