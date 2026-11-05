# The first client walk

Client commit: `a040302420` (the client was not edited). The fake adds the three
walk-specific scripts listed below. Builder: `debian-16gb-hel1-8`.
Evidence folder: `~/src/fabrig/ui-final-walk/walk/`, outside all worktrees.
`index.html` is the contact page. Pair paths in this table are relative to that
folder, where a copy of this report sits. Every selected PNG was checked for
1440 or 420 CSS pixels; pairs are twice that width, at device scale 1.

The final one-command 420 replay passed in 262.120 seconds, including the
reused first-width Home evidence. Its draft/Settings box took 78.665 seconds,
Needs you 67.694 seconds and Away 63.116 seconds. Those per-box times overlap
because the landing box stays alive. The initial manual 1440 walk includes
31 minutes away from Home and driver development; its detailed elapsed times
are in `manual-times.jsonl`. All boxes are fresh. Total boxes started: 24,
including exploratory and interrupted driver attempts; 25 Chromium starts.
The table's sources are:

| box or source | steps / purpose |
|---|---|
| manual 3 | 1, 2, 3; live 31-minute Home return at 1440, then that same state at 420 |
| manual 4 | Configure introduction / answer and Settings' other tabs at 1440; W1 fast path |
| manual 5 | Needs you at 1440; W1 and real ctl proposal |
| manual 6 | Away at 1440 |
| replay-1440 / 1 | final scripted draft → goal, Pause, Resume, Stop and Draft return |
| replay-420-verified / 1–4 | complete one-command seven-step phone replay, with the original Home return reused |
| manual 8 | additional real temper-owned held card on the earlier walk cue |
| gallery and git 727f3523ee | explicitly named substitutes and Configure look-request reference |

Verdicts: **25 matches, 72 differs, 30 unreachable**, across 127 rows. `differs` names a carried decision or a finding. `unreachable` names the missing side/state and its substitute; it does not claim that the whole client page is missing. Findings are W1–W6 in `findings.md`.

| step | page | state | width | verdict | why | box or way | pair |
|---|---|---|---|---|---|---|---|
| 1 | Forge | forge | 420 | matches | Same structure, controls and look; scripted data differs. | replay-420-verified / box 1 | `replay-420-verified/01-forge-420-pair.png` |
| 1 | Forge | forge | 1440 | matches | Same structure, controls and look; scripted data differs. | box 3 | `01-forge-1440-pair.png` |
| 1 | Forge | forge-empty-hello | 420 | matches | Same structure, controls and look; scripted data differs. | replay-420-verified / box 1 | `replay-420-verified/01-forge-empty-hello-420-pair.png` |
| 1 | Forge | forge-empty-hello | 1440 | matches | Same structure, controls and look; scripted data differs. | box 3 | `01-forge-empty-hello-1440-pair.png` |
| 1 | Home | home-to-begin | 420 | differs | By page, First run: ready headline, Skip and no Connected; W2 To begin copy. | replay-420-verified / box 1 | `replay-420-verified/01-home-to-begin-420-pair.png` |
| 1 | Home | home-to-begin | 1440 | differs | By page, First run: ready headline, Skip and no Connected; W2 To begin copy. | box 3 | `01-home-to-begin-1440-pair.png` |
| 1 | Model | model-chatgpt-signed-in | 420 | differs | Fake issuer returns to Ready without credentials; ChatGPT plan is usable. | replay-420-verified / box 1 | `replay-420-verified/01-model-chatgpt-signed-in-420-pair.png` |
| 1 | Model | model-chatgpt-signed-in | 1440 | matches | Fake issuer automatically returns to Ready; ChatGPT plan is usable. | manual box 3; Ready is the automatic sign-in return | `01-model-chatgpt-signed-in-1440-pair.png` |
| 1 | Model | model-claude | 420 | differs | By page, First run: Claude alert and fetch sentence. | replay-420-verified / box 1 | `replay-420-verified/01-model-claude-420-pair.png` |
| 1 | Model | model-claude | 1440 | differs | By page, First run: Claude alert and fetch sentence. | box 3 | `01-model-claude-1440-pair.png` |
| 1 | Model | model-offers | 420 | differs | By page, First run: the forge pane stays on Model. | replay-420-verified / box 1 | `replay-420-verified/01-model-offers-420-pair.png` |
| 1 | Model | model-offers | 1440 | differs | By page, First run: the forge pane stays on Model. | box 3 | `01-model-offers-1440-pair.png` |
| 1 | Ready | ready | 420 | differs | By page, First run: Hello and ChatGPT plan, no email; fake forge offers no browser sign-in. | replay-420-verified / box 1 | `replay-420-verified/01-ready-420-pair.png` |
| 1 | Ready | ready | 1440 | differs | By page, First run: Hello and ChatGPT plan, no email; fake forge offer is not_offered. | box 3 | `01-ready-1440-pair.png` |
| 1 | Welcome | welcome | 420 | matches | Same structure, controls and look; scripted data differs. | replay-420-verified / box 1 | `replay-420-verified/01-welcome-420-pair.png` |
| 1 | Welcome | welcome | 1440 | matches | Same structure, controls and look; scripted data differs. | box 3 | `01-welcome-1440-pair.png` |
| 2 | Configure | configure-answer | 420 | unreachable | The configuring fake has no settings/document script; gallery configure-chat-proposal is shown in step 5. | replay-420-verified / box 2 | `replay-420-verified/02-configure-answer-420-pair.png` |
| 2 | Configure | configure-answer | 1440 | unreachable | Fake has no configuring document/proposal script; gallery configure-chat-proposal shown in step 5. | box 4; ctl project-ways after W1 | `02-configure-answer-1440-pair.png` |
| 2 | Configure | configure-look-request | 420 | differs | W5: composer hint is present, lifting the field 25.1875px above the reference. | git 727f3523ee / 01; saved live Configure introduction | `02-configure-look-request-420-pair.png` |
| 2 | Configure | configure-look-request | 1440 | matches | Compared with the configure-chat look request, including its overlay. | git 727f3523ee / 01; saved live Configure introduction | `02-configure-look-request-1440-pair.png` |
| 2 | Configure | configure-temper | 420 | differs | By page, Settings: full page; configure-chat look request. | replay-420-verified / box 2 | `replay-420-verified/02-configure-temper-420-pair.png` |
| 2 | Configure | configure-temper | 1440 | differs | By page, Settings: full page; configure-chat look request. | box 4; ctl project-ways after W1 | `02-configure-temper-1440-pair.png` |
| 2 | Settings | rail-features-on | 420 | differs | D11; Ruling 9; By page, Phone: Changes is absent from tabs. | replay-420-verified / box 1 | `replay-420-verified/02-rail-features-on-420-pair.png` |
| 2 | Settings | rail-features-on | 1440 | differs | D11; Ruling 9; By page, Phone: Changes is absent from tabs. | box 3 | `02-rail-features-on-1440-pair.png` |
| 2 | Settings | settings-how-work-runs | 420 | differs | D11; By page, Settings: Configure temper first. | replay-420-verified / box 1 | `replay-420-verified/02-settings-how-work-runs-420-pair.png` |
| 2 | Settings | settings-how-work-runs | 1440 | differs | D11; By page, Settings: Configure temper first. | box 3 | `02-settings-how-work-runs-1440-pair.png` |
| 2 | Settings | settings-switches-stalled | 1440 | differs | W1: feature switches remain disabled. | box 5 | `02-settings-switches-stalled-1440-pair.png` |
| 3 | Changes | changes-landed | 420 | matches | Same structure, controls and look; scripted data differs. | replay-420-verified / box 1 | `replay-420-verified/03-changes-landed-420-pair.png` |
| 3 | Changes | changes-landed | 1440 | matches | Same structure, controls and look; scripted data differs. | box 3 | `03-changes-landed-1440-pair.png` |
| 3 | Changes | changes-queue | 420 | matches | Same structure, controls and look; scripted data differs. | replay-420-verified / box 1 | `replay-420-verified/03-changes-queue-420-pair.png` |
| 3 | Changes | changes-queue | 1440 | matches | Same structure, controls and look; scripted data differs. | box 3 | `03-changes-queue-1440-pair.png` |
| 3 | Gallery | changes-queued-row | 420 | unreachable | Landing finished before queue capture; gallery shown at 420. | gallery changes-one; box 3 attempt | `03-changes-queued-row-420-pair.png` |
| 3 | Gallery | changes-queued-row | 1440 | unreachable | Landing finished before the nonempty queue was observed; gallery shown. | gallery changes-one; box 3 attempt | `03-changes-queued-row-1440-pair.png` |
| 3 | Chat | chat-change | 420 | differs | D7 and By page, Chats: retained rail and Stop/Close; scripted conversation differs. | replay-420-verified / box 1 | `replay-420-verified/03-chat-change-420-pair.png` |
| 3 | Chat | chat-change | 1440 | differs | D7 and By page, Chats: retained rail and Stop/Close; scripted conversation differs. | box 3 | `03-chat-change-1440-pair.png` |
| 3 | Home | home-after-change | 420 | differs | D2; By page, Home: brief waits for a new sitting. | replay-420-verified / box 1 | `replay-420-verified/03-home-after-change-420-pair.png` |
| 3 | Home | home-after-change | 1440 | differs | D2; By page, Home: brief waits for a new sitting. | box 3 | `03-home-after-change-1440-pair.png` |
| 3 | Home | home-brief | 420 | unreachable | W6: no brief after over 31 minutes; computed Home shown live, intended brief in the gallery row. | manual box 3; same state resized from 1440 to 420 | `03-home-brief-420-pair.png` |
| 3 | Home | home-brief | 1440 | unreachable | W6: no brief after over 31 minutes; computed Home shown live, intended brief in the gallery row. | box 3 | `03-home-brief-1440-pair.png` |
| 3 | Gallery | home-brief-gallery | 420 | unreachable | W6: gallery home-morning shows the requested brief; the live default path did not deliver one. | gallery home-morning; box 3 attempt | `03-home-brief-gallery-420-pair.png` |
| 3 | Gallery | home-brief-gallery | 1440 | unreachable | W6: gallery home-morning shows the requested brief; the live default path did not deliver one. | gallery home-morning; box 3 attempt | `03-home-brief-gallery-1440-pair.png` |
| 3 | Chat | tool-lines | 420 | differs | D7 and By page, Chats: retained rail and Stop/Close; scripted conversation differs. | replay-420-verified / box 1 | `replay-420-verified/03-tool-lines-420-pair.png` |
| 3 | Chat | tool-lines | 1440 | differs | D7 and By page, Chats: retained rail and Stop/Close; scripted conversation differs. | box 3 | `03-tool-lines-1440-pair.png` |
| 4 | Draft | draft | 420 | differs | By page, A chat and its draft: composer stays on white; W4 section-heading rhythm. W4: section-heading line height. | replay-420-verified / box 2 | `replay-420-verified/04-draft-420-pair.png` |
| 4 | Draft | draft | 1440 | differs | By page, A chat and its draft: composer stays on white. W4: section-heading line height. | replay-1440 box 1 | `replay-1440/04-draft-1440-pair.png` |
| 4 | Chat | draft-back-to-chat | 420 | differs | By page, A chat and its draft: reopen in Chat. | replay-420-verified / box 2 | `replay-420-verified/04-draft-back-to-chat-420-pair.png` |
| 4 | Chat | draft-back-to-chat | 1440 | differs | By page, A chat and its draft: reopen in Chat. | replay-1440 box 1 | `replay-1440/04-draft-back-to-chat-1440-pair.png` |
| 4 | Chat and draft | draft-first-turn | 420 | differs | D7; By page, Phone: no Split at 420; W4 section-heading rhythm. W4: section-heading line height. | replay-420-verified / box 2 | `replay-420-verified/04-draft-first-turn-420-pair.png` |
| 4 | Chat and draft | draft-first-turn | 1440 | differs | D7; By page, Phone: no Split at 420. W4: section-heading line height. | replay-1440 box 1 | `replay-1440/04-draft-first-turn-1440-pair.png` |
| 4 | Chat | draft-second-turn | 420 | differs | D7 and By page, Chats: retained rail and Stop/Close; scripted conversation differs. | replay-420-verified / box 2 | `replay-420-verified/04-draft-second-turn-420-pair.png` |
| 4 | Chat | draft-second-turn | 1440 | differs | D7 and By page, Chats: retained rail and Stop/Close; scripted conversation differs. | replay-1440 box 1 | `replay-1440/04-draft-second-turn-1440-pair.png` |
| 4 | Goal | goal-how-it-runs | 420 | differs | Ruling 10; By page, Goals: available ways only. | replay-420-verified / box 2 | `replay-420-verified/04-goal-how-it-runs-420-pair.png` |
| 4 | Goal | goal-how-it-runs | 1440 | differs | Ruling 10; By page, Goals: available ways only. | replay-1440 box 1 | `replay-1440/04-goal-how-it-runs-1440-pair.png` |
| 4 | Goal | goal-landing | 420 | differs | By page, A goal: no branch-update line. | replay-420-verified / box 2 | `replay-420-verified/04-goal-landing-420-pair.png` |
| 4 | Goal | goal-landing | 1440 | differs | By page, A goal: no branch-update line. | replay-1440 box 1 | `replay-1440/04-goal-landing-1440-pair.png` |
| 4 | Goal | goal-paused | 420 | differs | D4: Pause stops running tasks immediately. | replay-420-verified / box 2 | `replay-420-verified/04-goal-paused-420-pair.png` |
| 4 | Goal | goal-paused | 1440 | differs | D4: Pause stops running tasks immediately. | replay-1440 box 1 | `replay-1440/04-goal-paused-1440-pair.png` |
| 4 | Goal | goal-report | 420 | differs | D8: goal-owned report; scripted goal waits for direction. | replay-420-verified / box 2 | `replay-420-verified/04-goal-report-420-pair.png` |
| 4 | Goal | goal-report | 1440 | differs | D8: goal-owned report; scripted goal waits for direction. | replay-1440 box 1 | `replay-1440/04-goal-report-1440-pair.png` |
| 4 | Goal | goal-resumed | 420 | matches | Same structure, controls and look; scripted data differs. | replay-420-verified / box 2 | `replay-420-verified/04-goal-resumed-420-pair.png` |
| 4 | Goal | goal-resumed | 1440 | matches | Same structure, controls and look; scripted data differs. | replay-1440 box 1 | `replay-1440/04-goal-resumed-1440-pair.png` |
| 4 | Goal | goal-stop-sheet | 420 | unreachable | Client Confirm sheet shown; mock uses native confirm, accepted by CDP, so its pending dialog is not in the PNG. | replay-420-verified / box 2 | `replay-420-verified/04-goal-stop-sheet-420-pair.png` |
| 4 | Goal | goal-stop-sheet | 1440 | unreachable | Client Confirm sheet shown; mock uses native confirm, accepted by CDP, so its pending dialog is not in the PNG. | replay-1440 box 1 | `replay-1440/04-goal-stop-sheet-1440-pair.png` |
| 4 | Goal | goal-stopped | 420 | matches | Same structure, controls and look; scripted data differs. | replay-420-verified / box 2 | `replay-420-verified/04-goal-stopped-420-pair.png` |
| 4 | Goal | goal-stopped | 1440 | matches | Same structure, controls and look; scripted data differs. | replay-1440 box 1 | `replay-1440/04-goal-stopped-1440-pair.png` |
| 4 | Gallery | goal-task-rows | 420 | unreachable | Final fake has no delegated tasks; gallery shown at 420. | gallery goal-running; box 3 attempt | `04-goal-task-rows-420-pair.png` |
| 4 | Gallery | goal-task-rows | 1440 | unreachable | Final fake has no delegated tasks; gallery shows task rows. | gallery goal-running; box 3 attempt | `04-goal-task-rows-1440-pair.png` |
| 4 | Goal | goal-tasks | 420 | unreachable | Script has no delegates: empty task section; gallery goal-running shows task rows. | replay-420-verified / box 2 | `replay-420-verified/04-goal-tasks-420-pair.png` |
| 4 | Goal | goal-tasks | 1440 | unreachable | Script has no delegates: empty task section; gallery goal-running shows task rows. | replay-1440 box 1 | `replay-1440/04-goal-tasks-1440-pair.png` |
| 4 | Chat | new-chat | 420 | differs | D7 and By page, Chats: retained rail and Stop/Close; scripted conversation differs. | replay-420-verified / box 2 | `replay-420-verified/04-new-chat-420-pair.png` |
| 4 | Chat | new-chat | 1440 | differs | D7 and By page, Chats: retained rail and Stop/Close; scripted conversation differs. | replay-1440 box 1 | `replay-1440/04-new-chat-1440-pair.png` |
| 4 | Rail | rail-menu | 420 | unreachable | By page, Phone: rail hidden; Chats and its + New chat replace it. | replay-420-verified / box 2 | `replay-420-verified/04-rail-menu-420-pair.png` |
| 4 | Rail | rail-menu | 1440 | differs | By page, A chat and its draft: held rail menu; Goal and All ways. | replay-1440 box 1 | `replay-1440/04-rail-menu-1440-pair.png` |
| 4 | Chat and draft | split | 420 | unreachable | By page, Phone: Split intentionally absent; Draft shows the document. | replay-420-verified / box 2 | `replay-420-verified/04-split-420-pair.png` |
| 4 | Chat and draft | split | 1440 | differs | D7: rail folds to a strip. W4: section-heading line height. | replay-1440 box 1 | `replay-1440/04-split-1440-pair.png` |
| 4 | Start goal | start-goal-sheet | 420 | differs | Rulings 3–8 and 10; By page, Goals: offered ways only; Budgets. | replay-420-verified / box 2 | `replay-420-verified/04-start-goal-sheet-420-pair.png` |
| 4 | Start goal | start-goal-sheet | 1440 | differs | Rulings 3–8 and 10; By page, Goals: offered ways only; Budgets. | replay-1440 box 1 | `replay-1440/04-start-goal-sheet-1440-pair.png` |
| 5 | Gallery | configure-proposal | 420 | unreachable | Configuring fake falls back; gallery shown. | gallery configure-chat-proposal; live/ctl attempts on manual boxes 5 and 9 | `05-configure-proposal-420-pair.png` |
| 5 | Gallery | configure-proposal | 1440 | unreachable | Configuring fake falls back; gallery shows proposal and document. | gallery configure-chat-proposal; live/ctl attempts on manual boxes 5 and 9 | `05-configure-proposal-1440-pair.png` |
| 5 | Gallery | held-task | 420 | unreachable | Final fake has no temper-owned held script; gallery shown. | gallery needs-held; live/ctl attempts on manual boxes 5 and 9 | `05-held-task-420-pair.png` |
| 5 | Gallery | held-task | 1440 | unreachable | Final fake has no temper-owned held script; ctl stop made only an owner hold. | gallery needs-held; live/ctl attempts on manual boxes 5 and 9 | `05-held-task-1440-pair.png` |
| 5 | Needs you | held-task-live | 420 | matches | Initial walk-specific goal script exhausted retries; this is a real temper-owned held card. The final cue waits on a question instead. | manual box 8, goal 2; earlier fake cue | `05-held-task-live-420-pair.png` |
| 5 | Home | home-needs-card | 420 | differs | D3: fixed question format; scripted question and counts. | replay-420-verified / box 3 | `replay-420-verified/05-home-needs-card-420-pair.png` |
| 5 | Home | home-needs-card | 1440 | differs | D3: fixed question format; scripted question and counts. | box 5 | `05-home-needs-card-1440-pair.png` |
| 5 | Needs you | later | 420 | differs | By page, Needs you: Later lives only in the tab. | replay-420-verified / box 3 | `replay-420-verified/05-later-420-pair.png` |
| 5 | Needs you | later | 1440 | differs | By page, Needs you: Later lives only in the tab. | box 5 | `05-later-1440-pair.png` |
| 5 | Needs you | needs-count | 420 | differs | By page, Needs you: counts omit answered cards. | replay-420-verified / box 3 | `replay-420-verified/05-needs-count-420-pair.png` |
| 5 | Needs you | needs-count | 1440 | differs | By page, Needs you: count omits answered cards. | box 5 | `05-needs-count-1440-pair.png` |
| 5 | Gallery | proposal | 420 | unreachable | W3: ctl proposal missing; gallery shown. | gallery needs-proposal; live/ctl attempts on manual boxes 5 and 9 | `05-proposal-420-pair.png` |
| 5 | Gallery | proposal | 1440 | unreachable | W3: ctl proposal missing from project inbox; gallery shows the card. | gallery needs-proposal; live/ctl attempts on manual boxes 5 and 9 | `05-proposal-1440-pair.png` |
| 5 | Needs you | proposal-missing | 420 | differs | W3: ctl proposal exists with project 0; project Needs you says nothing needs you. | replay-420-verified / box 3 | `replay-420-verified/05-proposal-missing-420-pair.png` |
| 5 | Needs you | question-with-choices | 420 | matches | Same structure, controls and look; scripted data differs. | replay-420-verified / box 3 | `replay-420-verified/05-question-with-choices-420-pair.png` |
| 5 | Needs you | question-with-choices | 1440 | matches | Same structure, controls and look; scripted data differs. | box 5 | `05-question-with-choices-1440-pair.png` |
| 5 | Needs you | reply-answered | 420 | differs | D5: five-second send delay; reply words restored on Undo. | replay-420-verified / box 3 | `replay-420-verified/05-reply-answered-420-pair.png` |
| 5 | Needs you | reply-answered | 1440 | differs | D5: five-second send delay. | box 5 | `05-reply-answered-1440-pair.png` |
| 5 | Needs you | reply-open | 420 | matches | Same structure, controls and look; scripted data differs. | replay-420-verified / box 3 | `replay-420-verified/05-reply-open-420-pair.png` |
| 5 | Needs you | reply-open | 1440 | matches | Same structure, controls and look; scripted data differs. | box 5 | `05-reply-open-1440-pair.png` |
| 5 | Needs you | undo | 420 | differs | D5; By page, Needs you: Undo restores reply words. | replay-420-verified / box 3 | `replay-420-verified/05-undo-420-pair.png` |
| 5 | Needs you | undo | 1440 | differs | D5: Undo restores reply words. | box 5 | `05-undo-1440-pair.png` |
| 6 | Home | away-on | 420 | differs | D10; By page, Away: implementation/retry/wording only. | replay-420-verified / box 4 | `replay-420-verified/06-away-on-420-pair.png` |
| 6 | Home | away-on | 1440 | differs | D10; By page, Away: implementation/retry/wording only. | box 6 | `06-away-on-1440-pair.png` |
| 6 | Home | decided-for-you | 420 | differs | D10; By page, Away: Home only, 24 hours. | replay-420-verified / box 4 | `replay-420-verified/06-decided-for-you-420-pair.png` |
| 6 | Home | decided-for-you | 1440 | differs | D10; By page, Away: Home only, 24 hours. | box 6 | `06-decided-for-you-1440-pair.png` |
| 6 | Home | goal-decided | 420 | differs | D10: decided tool; By page, Away: Home only, 24 hours. | replay-420-verified / box 4 | `replay-420-verified/06-goal-decided-420-pair.png` |
| 6 | Home | goal-decided | 1440 | differs | D10: decided tool; By page, Away: Home only, 24 hours. | box 6 | `06-goal-decided-1440-pair.png` |
| 6 | Home | overturned | 420 | differs | By page, Away: overturn while the task lives. | replay-420-verified / box 4 | `replay-420-verified/06-overturned-420-pair.png` |
| 6 | Home | overturned | 1440 | differs | By page, Away: overturn while the task lives. | box 6 | `06-overturned-1440-pair.png` |
| 7 | Chats | back-in | 420 | unreachable | Fresh printed link signs the client back in; no mock sign-in page exists. | replay-420-verified / box 2 | `replay-420-verified/07-back-in-420-pair.png` |
| 7 | Chats | back-in | 1440 | unreachable | Fresh printed link signs the client back in; no mock sign-in page exists. | box 4; ctl project-ways after W1 | `07-back-in-1440-pair.png` |
| 7 | Home | project-added | 420 | unreachable | Mock + New project only explains name/Create in a toast; client stock-policy creation shown. | replay-420-verified / box 2 | `replay-420-verified/07-project-added-420-pair.png` |
| 7 | Home | project-added | 1440 | unreachable | Mock + New project only explains name/Create in a toast; client stock-policy creation shown. | box 4; ctl project-ways after W1 | `07-project-added-1440-pair.png` |
| 7 | Home | project-switched | 420 | matches | Same structure, controls and look; scripted data differs. | replay-420-verified / box 2 | `replay-420-verified/07-project-switched-420-pair.png` |
| 7 | Home | project-switched | 1440 | matches | Same structure, controls and look; scripted data differs. | box 4; ctl project-ways after W1 | `07-project-switched-1440-pair.png` |
| 7 | Project switcher | project-switcher | 420 | matches | Same structure, controls and look; scripted data differs. | replay-420-verified / box 2 | `replay-420-verified/07-project-switcher-420-pair.png` |
| 7 | Project switcher | project-switcher | 1440 | matches | Same structure, controls and look; scripted data differs. | box 4; ctl project-ways after W1 | `07-project-switcher-1440-pair.png` |
| 7 | Accounts | settings-accounts | 420 | differs | By page, Settings: tab contents use the mock language; mock tab not drawn. | replay-420-verified / box 2 | `replay-420-verified/07-settings-accounts-420-pair.png` |
| 7 | Accounts | settings-accounts | 1440 | differs | By page, Settings: tab contents use the mock language; mock tab not drawn. | box 4; ctl project-ways after W1 | `07-settings-accounts-1440-pair.png` |
| 7 | Builders | settings-builders | 420 | differs | By page, Settings: tab contents use the mock language; mock tab not drawn. | replay-420-verified / box 2 | `replay-420-verified/07-settings-builders-420-pair.png` |
| 7 | Builders | settings-builders | 1440 | differs | By page, Settings: tab contents use the mock language; mock tab not drawn. | box 4; ctl project-ways after W1 | `07-settings-builders-1440-pair.png` |
| 7 | Forges | settings-forges | 420 | differs | By page, Settings: tab contents use the mock language; mock tab not drawn. | replay-420-verified / box 2 | `replay-420-verified/07-settings-forges-420-pair.png` |
| 7 | Forges | settings-forges | 1440 | differs | By page, Settings: tab contents use the mock language; mock tab not drawn. | box 4; ctl project-ways after W1 | `07-settings-forges-1440-pair.png` |
| 7 | Repositories | settings-repositories | 420 | differs | By page, Settings: tab contents use the mock language; mock tab not drawn. | replay-420-verified / box 2 | `replay-420-verified/07-settings-repositories-420-pair.png` |
| 7 | Repositories | settings-repositories | 1440 | differs | By page, Settings: tab contents use the mock language; mock tab not drawn. | box 4; ctl project-ways after W1 | `07-settings-repositories-1440-pair.png` |
| 7 | Sign out | signed-out | 420 | unreachable | By page, Settings: Sign out is in Accounts. Client signed-out page shown; the mock has no equivalent. | replay-420-verified / box 2 | `replay-420-verified/07-signed-out-420-pair.png` |
| 7 | Sign out | signed-out | 1440 | unreachable | By page, Settings: Sign out is in Accounts. Client signed-out page shown; the mock has no equivalent. | box 4; ctl project-ways after W1 | `07-signed-out-1440-pair.png` |
| 7 | Sign out | signed-out-reloaded | 420 | unreachable | Signed-out client survives reload; mock has no signed-out page. | replay-420-verified / box 2 | `replay-420-verified/07-signed-out-reloaded-420-pair.png` |
| 7 | Sign out | signed-out-reloaded | 1440 | unreachable | Signed-out client survives reload; mock has no signed-out page. | box 4; ctl project-ways after W1 | `07-signed-out-reloaded-1440-pair.png` |

## Notes from the walk

- The 31-minute return was observed after at least 1869 seconds away. Home
  said “you last looked 32 minutes ago”, but delivered no brief. The default
  period had no unreserved funds and the sign-in had made a personal account,
  unlike the passing process test's deployment account and funding setup.
  W6 records the policy/admission question; the gallery is the substitute.
- A nonempty queue was too brief to observe on the fast local landing. Its
  section and the real landed row were walked; `changes-one` supplies the
  nonempty-row proof. The goal cue deliberately creates no delegated tasks;
  `goal-running` supplies the nonempty task rows.
- The first trial of the additional goal cue waited on nothing, exhausted
  retries and supplied the extra live held-card capture. The final cue writes
  its report, asks a question and waits. Its Pause/Resume/Stop and Draft return
  were then walked at both widths. The earlier transient missing Draft link
  on the failing cue is not a client finding.
- Configure temper was compared with both the greenfield configuring chat
  and `727f3523ee`'s `configure-chat/01` plus its overlay. The fake's ordinary
  configuring reply produces neither a document nor a settings proposal;
  the named gallery states show them. A separate ctl proposal was made for
  real and exposed W3. Forges, Accounts, Builders and Repositories are live
  client tabs; the mock's tab acts only explain their undrawn contents.
- At 420, Chats provides the way back from a conversation; Settings is in
  the project switcher once Goals occupies its tab, and Home provides Changes.
  Split and the desktop rail menu are intentionally absent. Sign out was last
  on each Settings box; reload stayed signed out, and a fresh printed link
  supplied the way back. No sign-in link is included in this report.
- The early two-minute Settings stall was seen twice at 1440. The final
  fast-path attempt did not reproduce it. W1 records both the observation and
  that uncertainty; later paths name their ctl fallback where it was used.
- Mock data, titles, model names, counts, clocks and paragraph lengths differ
  on purpose. Retained rail, white Draft composer, available goal ways,
  budget shape, no green-time claim and the disabled Settings controls follow
  the carried decisions. No such decision was reversed.

## Driver and fake choices

The committed Python driver uses only the standard library. It copies inputs,
starts and stops its own processes, handles native dialogs, waits for visible
named controls, and saves text/styles/rectangles alongside each pair. A mock
native Stop dialog is accepted rather than retained as a screenshot, while the
client Confirm sheet is captured. Clean mock references were recaptured beside
unchanged live client PNGs where a Mockup panel had covered the reference.

Three scripts were added, leaving the old cues intact: asked `Land a local hello
status change.`, cued `Add a local hello status line to README.md.` for its coder,
and cued `walk-away-mode` for the report/question/wait goal. The coder reads then
writes the fresh hello README; it does not assume widget's original text.
The final model document has nine cued and four asked scripts, within sixteen.
No limit, client API, stored format or protocol changed. No client was fixed.

The Rust checks passed: 82 fake tests after the first addition, one focused fake
format test, 1686 focused tests across the changed package and its six direct
consumers, then 82 fake tests after the cue correction. Each of the three named
process walks passed before and after that correction (one selected test per
run). All six consumers passed `cargo check --tests`; clippy for temper-fake,
formatting, and temper's old-form count passed. No new test was added, so there
is no new-rule mutation exercise. The non-fake checks belong to `real-checks`.

The replay that treated the mock's + New project toast as an implementation was
corrected. A later waiting replay was interrupted by the command runner after
ten minutes; its owned processes were stopped and its folders removed. A final
fresh one-command replay completed normally. All logs remain beside the images.

All started boxes printed Stopped, all Chromium processes ended, and every
boxes/ and build/ directory was removed. Screenshots, public ctl evidence,
logs, the gallery and the contact pages remain on the builder.
