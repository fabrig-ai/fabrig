# The client, walked again

At 1440 and 420, the client closely follows the mock's layout, type and spacing.
First run, chat-to-change, draft-to-goal, questions, Away and sign-out can all be
walked. The earlier copy, heading rhythm, phone hint and working-note faults
are fixed. The retained chat rail, available goal ways, budgets and disabled
Settings controls follow the carried decisions. The remaining functional and
rename differences are listed below, with their reproductions; the fake's and
the mock's missing states are explicit in the table.

Client source: `07f8081aff`, on the fixed integration tip `ab064a7976`, then
A1's sheet at `67f012c1c1`. The full walk ran on the former; the affected
project-creation and rename states were walked again on fresh boxes with the
latter's rebuilt bundle. A1 changes only outlines on focused page main/h1
nodes with tabindex -1. The table selects the final captures of those states.
The full-walk bundle is `952a3a318572a5cd`; the final bundle is
`8ed930c6d4532ff7`. Native builds and copied inputs are recorded in
`walk-again/artifact-identity.json`; no later build changed a running box.

Builder: **debian-32gb-hel1-11**. Evidence: `~/src/fabrig/ui-final-walk/`;
[contact page](index.html), with the new pairs under [walk-again/](walk-again/index.html)
and this report beside the contact page. All pair links below are relative to
that evidence root. The first walk is on **debian-16gb-hel1-8**, in
[walk/](walk/index.html) on that builder. The non-fake evidence is on
**debian-32gb-hel1-11**, in [real-checks/](real-checks/index.html).

Verdicts: **22 matches, 90 differs, 30 unreachable**, across **142 rows**.
The first walk had **25 matches, 72 differs, 30 unreachable**, across 127 rows.
The new totals include working-note, fast-read, rename and repeated-choice
checks, live queued-row evidence, and clearer decision classifications; they
are not a count of regressions. A4 is the new adverse interaction observation.
The earlier settings-switches-stalled case is now settings-read-fast-path;
the additional needs-count-confirmed pair keeps the successful desktop repeat
and the primary phone comparison separate from A4's original capture.

There were **15 fresh boxes**, 11 supplying the selected evidence and four
exploratory/interrupted runs. Each full-walk box repeated first run and enabled
the four features through Settings; none used the settings ctl fallback.

| source | boxes | selected steps |
|---|---:|---|
| `1440/` | 1–4 | 1: change and timed Home return; 2: draft, goal, Configure, Settings and Sign out; 3: Needs you and ctl proposal; 4: Away |
| `420-verified/` | 1–4 | The same paths on four fresh phone boxes; Home's brief uses the first desktop box's live state resized to 420 |
| `focus-final/1440/`, `focus-final/420/` | 1 each | Fresh first run, Enter-created project, Rename, Escape and Cancel after A1 |
| `needs-confirm/1440/` | 1 | Fresh standalone repeat of the question/Undo/choice sequence after A4 |
| `420/`, `focus-before/1440/`, `focus-after/1440/`, `focus-after/1440-verified/` | 1 each | Interrupted phone driver, pre-fix outline proof, an over-specific driver wait, and initial post-fix focus proof |

The desktop landing box ran for 1,937.835 seconds (32.30 minutes); its page
was absent for the required 31 minutes, then Home was given 20 seconds to read
the brief. The phone replay ran for 1,562.385 seconds (26.04 minutes), including
its wait for that same live return. Home said “you last looked 31 minutes ago”
at both widths but showed no brief. The first period still had budget and
reservation 8,117,760, leaving nothing unreserved. The corresponding public
project read is `walk-again/1440/home-brief-project.json`.

“Matches” compares the named state's structure, controls and look; scripted
names, counts, clocks and document bodies differ intentionally. “Unreachable”
names the missing side and its substitute. It does not say the whole client
page is absent. The repeated held-task-live rows preserve the first walk's
extra transient state rather than silently dropping it.

| step | page | state | width | verdict | why | box or way | pair |
|---|---|---|---|---|---|---|---|
| 1 | Forge | forge | 420 | matches | Same structure, controls and look; scripted data differs. | 420-verified / box 1 | [pair](walk-again/420-verified/01-forge-420-pair.png) |
| 1 | Forge | forge | 1440 | matches | Same structure, controls and look; scripted data differs. | 1440 / box 1 | [pair](walk-again/1440/01-forge-1440-pair.png) |
| 1 | Forge | forge-empty-hello | 420 | matches | Same structure, controls and look; scripted data differs. | 420-verified / box 1 | [pair](walk-again/420-verified/01-forge-empty-hello-420-pair.png) |
| 1 | Forge | forge-empty-hello | 1440 | matches | Same structure, controls and look; scripted data differs. | 1440 / box 1 | [pair](walk-again/1440/01-forge-empty-hello-1440-pair.png) |
| 1 | Home | home-to-begin | 420 | differs | By page, First run: ready headline, no Connected; W2 acceptance copy fixed. | 420-verified / box 1 | [pair](walk-again/420-verified/01-home-to-begin-420-pair.png) |
| 1 | Home | home-to-begin | 1440 | differs | By page, First run: ready headline, no Connected; W2 acceptance copy fixed. | 1440 / box 1 | [pair](walk-again/1440/01-home-to-begin-1440-pair.png) |
| 1 | Model | model-chatgpt-signed-in | 420 | differs | By page, First run: automatic fake return to Ready; project name and ChatGPT plan omit email. | 420-verified / box 1 | [pair](walk-again/420-verified/01-model-chatgpt-signed-in-420-pair.png) |
| 1 | Model | model-chatgpt-signed-in | 1440 | differs | By page, First run: automatic fake return to Ready; project name and ChatGPT plan omit email. | 1440 / box 1 | [pair](walk-again/1440/01-model-chatgpt-signed-in-1440-pair.png) |
| 1 | Model | model-claude | 420 | differs | By page, First run: retained forge pane and Claude command/sign-in explanation. | 420-verified / box 1 | [pair](walk-again/420-verified/01-model-claude-420-pair.png) |
| 1 | Model | model-claude | 1440 | differs | By page, First run: retained forge pane and Claude command/sign-in explanation. | 1440 / box 1 | [pair](walk-again/1440/01-model-claude-1440-pair.png) |
| 1 | Model | model-offers | 420 | differs | By page, First run: the forge pane stays on Model. | 420-verified / box 1 | [pair](walk-again/420-verified/01-model-offers-420-pair.png) |
| 1 | Model | model-offers | 1440 | differs | By page, First run: the forge pane stays on Model. | 1440 / box 1 | [pair](walk-again/1440/01-model-offers-1440-pair.png) |
| 1 | Ready | ready | 420 | differs | By page, First run: Hello and ChatGPT plan, no email; fake forge offers no browser sign-in. | 420-verified / box 1 | [pair](walk-again/420-verified/01-ready-420-pair.png) |
| 1 | Ready | ready | 1440 | differs | By page, First run: Hello and ChatGPT plan, no email; fake forge offers no browser sign-in. | 1440 / box 1 | [pair](walk-again/1440/01-ready-1440-pair.png) |
| 1 | Welcome | welcome | 420 | matches | Same structure, controls and look; scripted data differs. | 420-verified / box 1 | [pair](walk-again/420-verified/01-welcome-420-pair.png) |
| 1 | Welcome | welcome | 1440 | matches | Same structure, controls and look; scripted data differs. | 1440 / box 1 | [pair](walk-again/1440/01-welcome-1440-pair.png) |
| 2 | Configure | configure-answer | 420 | unreachable | The configuring fake has no settings/document script; gallery configure-chat-proposal is shown in step 5. | 420-verified / box 2 | [pair](walk-again/420-verified/02-configure-answer-420-pair.png) |
| 2 | Configure | configure-answer | 1440 | unreachable | The configuring fake has no settings/document script; gallery configure-chat-proposal is shown in step 5. | 1440 / box 2 | [pair](walk-again/1440/02-configure-answer-1440-pair.png) |
| 2 | Configure | configure-look-request | 420 | matches | Compared with the configure-chat look request, including its overlay; W5 phone hint fixed. | 420-verified / git 727f3523ee / 01; saved live Configure introduction | [pair](walk-again/420-verified/02-configure-look-request-420-pair.png) |
| 2 | Configure | configure-look-request | 1440 | matches | Compared with the configure-chat look request, including its overlay; W5 phone hint fixed. | 1440 / git 727f3523ee / 01; saved live Configure introduction | [pair](walk-again/1440/02-configure-look-request-1440-pair.png) |
| 2 | Configure | configure-temper | 420 | differs | By page, Settings: full page; configure-chat look request. | 420-verified / box 2 | [pair](walk-again/420-verified/02-configure-temper-420-pair.png) |
| 2 | Configure | configure-temper | 1440 | differs | By page, Settings: full page; configure-chat look request. | 1440 / box 2 | [pair](walk-again/1440/02-configure-temper-1440-pair.png) |
| 2 | Settings | rail-features-on | 420 | differs | D11; Ruling 9; Configure first in Settings. By page, Phone: Changes is reached from Home. | 420-verified / box 1 | [pair](walk-again/420-verified/02-rail-features-on-420-pair.png) |
| 2 | Settings | rail-features-on | 1440 | differs | D11; Ruling 9; Configure first in Settings. | 1440 / box 1 | [pair](walk-again/1440/02-rail-features-on-1440-pair.png) |
| 2 | Settings | settings-how-work-runs | 420 | differs | D11; By page, Settings: Configure temper first. | 420-verified / box 1 | [pair](walk-again/420-verified/02-settings-how-work-runs-420-pair.png) |
| 2 | Settings | settings-how-work-runs | 1440 | differs | D11; By page, Settings: Configure temper first. | 1440 / box 1 | [pair](walk-again/1440/02-settings-how-work-runs-1440-pair.png) |
| 2 | Settings | settings-read-fast-path | 420 | matches | W1 fixed on the fast first-run path: all four feature controls available; no ctl fallback. | 420-verified / box 1 | [pair](walk-again/420-verified/02-settings-how-work-runs-420-pair.png) |
| 2 | Settings | settings-read-fast-path | 1440 | matches | W1 fixed on the fast first-run path: all four feature controls available; no ctl fallback. | 1440 / box 1 | [pair](walk-again/1440/02-settings-how-work-runs-1440-pair.png) |
| 3 | Changes | changes-landed | 420 | differs | By page, Changes: no green time; queue and landed rows keep the mock structure. | 420-verified / box 1 | [pair](walk-again/420-verified/03-changes-landed-420-pair.png) |
| 3 | Changes | changes-landed | 1440 | differs | By page, Changes: no green time; queue and landed rows keep the mock structure. | 1440 / box 1; queued row retained; landed list recaptured through browser history without reopening Home | [pair](walk-again/1440/03-changes-landed-1440-pair.png) |
| 3 | Changes | changes-queue | 420 | differs | By page, Changes: no green time; queue and landed rows keep the mock structure. | 420-verified / box 1 | [pair](walk-again/420-verified/03-changes-queue-420-pair.png) |
| 3 | Changes | changes-queue | 1440 | differs | By page, Changes: no green time; queue and landed rows keep the mock structure. | 1440 / box 1; queued row retained; landed list recaptured through browser history without reopening Home | [pair](walk-again/1440/03-changes-queue-1440-pair.png) |
| 3 | Changes | changes-queued-row | 420 | differs | By page, Changes: no green time; live nonempty queue observed. | 420-verified / box 1 | [pair](walk-again/420-verified/03-changes-queued-row-420-pair.png) |
| 3 | Gallery | changes-queued-row | 1440 | differs | By page, Changes: no green time; live nonempty queue observed. | 1440 / box 1; queued row retained; landed list recaptured through browser history without reopening Home | [pair](walk-again/1440/03-changes-queued-row-1440-pair.png) |
| 3 | Chat | chat-change | 420 | differs | D7; By page, Chats: retained rail at 1440 and Stop/Close controls; scripted conversation and tool content differ. | 420-verified / box 1 | [pair](walk-again/420-verified/03-chat-change-420-pair.png) |
| 3 | Chat | chat-change | 1440 | differs | D7; By page, Chats: retained rail at 1440 and Stop/Close controls; scripted conversation and tool content differ. | 1440 / box 1 | [pair](walk-again/1440/03-chat-change-1440-pair.png) |
| 3 | Home | home-after-change | 420 | differs | D2; By page, Home: brief waits for a new sitting. | 420-verified / box 1 | [pair](walk-again/420-verified/03-home-after-change-420-pair.png) |
| 3 | Home | home-after-change | 1440 | differs | D2; By page, Home: brief waits for a new sitting. | 1440 / box 1 | [pair](walk-again/1440/03-home-after-change-1440-pair.png) |
| 3 | Home | home-brief | 420 | unreachable | W6: no brief after 31 minutes; computed Home shown; gallery row supplies the intended brief. | 420-verified / first desktop box; same live return resized to 420 | [pair](walk-again/420-verified/03-home-brief-420-pair.png) |
| 3 | Home | home-brief | 1440 | unreachable | W6: no brief after 31 minutes; computed Home shown; gallery row supplies the intended brief. | 1440 / box 1; live return after 31 minutes | [pair](walk-again/1440/03-home-brief-1440-pair.png) |
| 3 | Gallery | home-brief-gallery | 420 | unreachable | W6: gallery home-morning supplies the brief missing from the live return. | 420-verified / first desktop box; same live return resized to 420 | [pair](walk-again/420-verified/03-home-brief-gallery-420-pair.png) |
| 3 | Gallery | home-brief-gallery | 1440 | unreachable | W6: gallery home-morning supplies the brief missing from the live return. | 1440 / box 1; live return after 31 minutes | [pair](walk-again/1440/03-home-brief-gallery-1440-pair.png) |
| 3 | Chat | tool-lines | 420 | differs | D7; By page, Chats: retained rail at 1440 and Stop/Close controls; scripted conversation and tool content differ. | 420-verified / box 1 | [pair](walk-again/420-verified/03-tool-lines-420-pair.png) |
| 3 | Chat | tool-lines | 1440 | differs | D7; By page, Chats: retained rail at 1440 and Stop/Close controls; scripted conversation and tool content differ. | 1440 / box 1 | [pair](walk-again/1440/03-tool-lines-1440-pair.png) |
| 4 | Draft | document-after-working-notes | 420 | differs | D7; W7 fixed: reopened document follows five lower-numbered work: notes across three ctl pages. | 420-verified / box 2 | [pair](walk-again/420-verified/04-document-after-working-notes-420-pair.png) |
| 4 | Draft | document-after-working-notes | 1440 | differs | D7; W7 fixed: reopened document follows five lower-numbered work: notes across three ctl pages. | 1440 / box 2 | [pair](walk-again/1440/04-document-after-working-notes-1440-pair.png) |
| 4 | Draft | draft | 420 | differs | By page, A chat and its draft: composer stays on white; W4 heading rhythm fixed. | 420-verified / box 2 | [pair](walk-again/420-verified/04-draft-420-pair.png) |
| 4 | Draft | draft | 1440 | differs | By page, A chat and its draft: composer stays on white; W4 heading rhythm fixed. | 1440 / box 2 | [pair](walk-again/1440/04-draft-1440-pair.png) |
| 4 | Chat | draft-back-to-chat | 420 | differs | By page, A chat and its draft: reopen in Chat. | 420-verified / box 2 | [pair](walk-again/420-verified/04-draft-back-to-chat-420-pair.png) |
| 4 | Chat | draft-back-to-chat | 1440 | differs | By page, A chat and its draft: reopen in Chat. | 1440 / box 2 | [pair](walk-again/1440/04-draft-back-to-chat-1440-pair.png) |
| 4 | Chat and draft | draft-first-turn | 420 | differs | D7; By page, Phone: no Split at 420; W4 heading rhythm fixed. | 420-verified / box 2 | [pair](walk-again/420-verified/04-draft-first-turn-420-pair.png) |
| 4 | Chat and draft | draft-first-turn | 1440 | differs | D7; By page, Phone: no Split at 420; W4 heading rhythm fixed. | 1440 / box 2 | [pair](walk-again/1440/04-draft-first-turn-1440-pair.png) |
| 4 | Chat | draft-second-turn | 420 | differs | D7: rail stays. | 420-verified / box 2 | [pair](walk-again/420-verified/04-draft-second-turn-420-pair.png) |
| 4 | Chat | draft-second-turn | 1440 | differs | D7: rail stays. | 1440 / box 2 | [pair](walk-again/1440/04-draft-second-turn-1440-pair.png) |
| 4 | Goal | goal-how-it-runs | 420 | differs | Ruling 10; By page, Goals: available ways only. | 420-verified / box 2 | [pair](walk-again/420-verified/04-goal-how-it-runs-420-pair.png) |
| 4 | Goal | goal-how-it-runs | 1440 | differs | Ruling 10; By page, Goals: available ways only. | 1440 / box 2 | [pair](walk-again/1440/04-goal-how-it-runs-1440-pair.png) |
| 4 | Goal | goal-landing | 420 | differs | By page, A goal: no branch-update line. | 420-verified / box 2 | [pair](walk-again/420-verified/04-goal-landing-420-pair.png) |
| 4 | Goal | goal-landing | 1440 | differs | By page, A goal: no branch-update line. | 1440 / box 2 | [pair](walk-again/1440/04-goal-landing-1440-pair.png) |
| 4 | Goal | goal-paused | 420 | differs | D4: Pause stops running tasks immediately. | 420-verified / box 2 | [pair](walk-again/420-verified/04-goal-paused-420-pair.png) |
| 4 | Goal | goal-paused | 1440 | differs | D4: Pause stops running tasks immediately. | 1440 / box 2 | [pair](walk-again/1440/04-goal-paused-1440-pair.png) |
| 4 | Goal | goal-report | 420 | differs | D8: goal-owned report; scripted goal waits for direction. | 420-verified / box 2 | [pair](walk-again/420-verified/04-goal-report-420-pair.png) |
| 4 | Goal | goal-report | 1440 | differs | D8: goal-owned report; scripted goal waits for direction. | 1440 / box 2 | [pair](walk-again/1440/04-goal-report-1440-pair.png) |
| 4 | Goal | goal-resumed | 420 | matches | Same structure, controls and look; scripted data differs. | 420-verified / box 2 | [pair](walk-again/420-verified/04-goal-resumed-420-pair.png) |
| 4 | Goal | goal-resumed | 1440 | matches | Same structure, controls and look; scripted data differs. | 1440 / box 2 | [pair](walk-again/1440/04-goal-resumed-1440-pair.png) |
| 4 | Goal | goal-stop-sheet | 420 | unreachable | Client Confirm sheet shown; mock native confirm accepted by CDP, so its pending dialog cannot be captured in the PNG. | 420-verified / box 2 | [pair](walk-again/420-verified/04-goal-stop-sheet-420-pair.png) |
| 4 | Goal | goal-stop-sheet | 1440 | unreachable | Client Confirm sheet shown; mock native confirm accepted by CDP, so its pending dialog cannot be captured in the PNG. | 1440 / box 2 | [pair](walk-again/1440/04-goal-stop-sheet-1440-pair.png) |
| 4 | Goal | goal-stopped | 420 | matches | Same structure, controls and look; scripted data differs. | 420-verified / box 2 | [pair](walk-again/420-verified/04-goal-stopped-420-pair.png) |
| 4 | Goal | goal-stopped | 1440 | matches | Same structure, controls and look; scripted data differs. | 1440 / box 2 | [pair](walk-again/1440/04-goal-stopped-1440-pair.png) |
| 4 | Gallery | goal-task-rows | 420 | unreachable | Scripted goal has no delegated tasks; gallery shows their rows. | 420-verified / gallery goal-running; box 2 attempt | [pair](walk-again/420-verified/04-goal-task-rows-420-pair.png) |
| 4 | Gallery | goal-task-rows | 1440 | unreachable | Scripted goal has no delegated tasks; gallery shows their rows. | 1440 / gallery goal-running; box 2 attempt | [pair](walk-again/1440/04-goal-task-rows-1440-pair.png) |
| 4 | Goal | goal-tasks | 420 | unreachable | Script has no delegates: empty task section; gallery goal-running shows task rows. | 420-verified / box 2 | [pair](walk-again/420-verified/04-goal-tasks-420-pair.png) |
| 4 | Goal | goal-tasks | 1440 | unreachable | Script has no delegates: empty task section; gallery goal-running shows task rows. | 1440 / box 2 | [pair](walk-again/1440/04-goal-tasks-1440-pair.png) |
| 4 | Chat | new-chat | 420 | differs | D7; By page, Chats: retained rail at 1440 and Stop/Close controls; scripted first turn. | 420-verified / box 2 | [pair](walk-again/420-verified/04-new-chat-420-pair.png) |
| 4 | Chat | new-chat | 1440 | differs | D7; By page, Chats: retained rail at 1440 and Stop/Close controls; scripted first turn. | 1440 / box 2 | [pair](walk-again/1440/04-new-chat-1440-pair.png) |
| 4 | Rail | rail-menu | 420 | unreachable | By page, Phone: rail hidden; Chats and its + New chat replace it. | 420-verified / box 2 | [pair](walk-again/420-verified/04-rail-menu-420-pair.png) |
| 4 | Rail | rail-menu | 1440 | differs | By page, A chat and its draft: held rail menu; Goal and All ways. | 1440 / box 2 | [pair](walk-again/1440/04-rail-menu-1440-pair.png) |
| 4 | Chat and draft | split | 420 | unreachable | By page, Phone: Split intentionally absent; Draft shows the document. | 420-verified / box 2 | [pair](walk-again/420-verified/04-split-420-pair.png) |
| 4 | Chat and draft | split | 1440 | differs | D7: rail folds to a strip; W4 heading rhythm fixed. | 1440 / box 2 | [pair](walk-again/1440/04-split-1440-pair.png) |
| 4 | Start goal | start-goal-sheet | 420 | differs | Rulings 3–8 and 10; By page, Goals: offered ways only; Budgets. | 420-verified / box 2 | [pair](walk-again/420-verified/04-start-goal-sheet-420-pair.png) |
| 4 | Start goal | start-goal-sheet | 1440 | differs | Rulings 3–8 and 10; By page, Goals: offered ways only; Budgets. | 1440 / box 2 | [pair](walk-again/1440/04-start-goal-sheet-1440-pair.png) |
| 4 | Chat | working-notes-only | 420 | differs | D7; W7 fixed: no Draft when only work: notes exist. Mock retains its different document fixture. | 420-verified / box 2 | [pair](walk-again/420-verified/04-working-notes-only-420-pair.png) |
| 4 | Chat | working-notes-only | 1440 | differs | D7; W7 fixed: no Draft when only work: notes exist. Mock retains its different document fixture. | 1440 / box 2 | [pair](walk-again/1440/04-working-notes-only-1440-pair.png) |
| 5 | Gallery | configure-proposal | 420 | unreachable | Configuring fake replies with fallback words; gallery shows its proposal/document state. | 420-verified / gallery configure-chat-proposal; box 3 attempt | [pair](walk-again/420-verified/05-configure-proposal-420-pair.png) |
| 5 | Gallery | configure-proposal | 1440 | unreachable | Configuring fake replies with fallback words; gallery shows its proposal/document state. | 1440 / gallery configure-chat-proposal; box 3 attempt | [pair](walk-again/1440/05-configure-proposal-1440-pair.png) |
| 5 | Gallery | held-task | 420 | unreachable | A ctl stop placed an owner hold; no temper-owned held card. Gallery shows the requested kind. | 420-verified / gallery needs-held; box 3 attempt | [pair](walk-again/420-verified/05-held-task-420-pair.png) |
| 5 | Gallery | held-task | 1440 | unreachable | A ctl stop placed an owner hold; no temper-owned held card. Gallery shows the requested kind. | 1440 / gallery needs-held; box 3 attempt | [pair](walk-again/1440/05-held-task-1440-pair.png) |
| 5 | Gallery | held-task-live | 420 | unreachable | The first walk used a transient failing cue for this extra live card; the final fake cannot reproduce it. Current gallery needs-held is shown. | 420-verified / gallery needs-held; box 3 attempt | [pair](walk-again/420-verified/05-held-task-420-pair.png) |
| 5 | Gallery | held-task-live | 1440 | unreachable | The first walk used a transient failing cue for this extra live card; the final fake cannot reproduce it. Current gallery needs-held is shown. | 1440 / gallery needs-held; box 3 attempt | [pair](walk-again/1440/05-held-task-1440-pair.png) |
| 5 | Home | home-needs-card | 420 | differs | D3: fixed question format; scripted question and counts. | 420-verified / box 3 | [pair](walk-again/420-verified/05-home-needs-card-420-pair.png) |
| 5 | Home | home-needs-card | 1440 | differs | D3: fixed question format; scripted question and counts. | 1440 / box 3 | [pair](walk-again/1440/05-home-needs-card-1440-pair.png) |
| 5 | Needs you | later | 420 | differs | By page, Needs you: Later lives only in the tab. | 420-verified / box 3 | [pair](walk-again/420-verified/05-later-420-pair.png) |
| 5 | Needs you | later | 1440 | differs | By page, Needs you: Later lives only in the tab. | 1440 / box 3 | [pair](walk-again/1440/05-later-1440-pair.png) |
| 5 | Needs you | needs-count | 420 | differs | By page, Needs you: counts omit answered cards. | 420-verified / box 3 | [pair](walk-again/420-verified/05-needs-count-420-pair.png) |
| 5 | Needs you | needs-count | 1440 | differs | A4: first choice after Undo did not answer; question/count 1 remained. Fresh repeat is a separate row. | 1440 / box 3 | [pair](walk-again/1440/05-needs-count-1440-pair.png) |
| 5 | Needs you | needs-count-confirmed | 420 | differs | By page, Needs you: choice answered and count cleared in the primary phone walk; A4 was absent here. Standalone repeat was at 1440 only. | 420-verified / box 3; primary phone comparison for A4 | [pair](walk-again/420-verified/05-needs-count-420-pair.png) |
| 5 | Needs you | needs-count-confirmed | 1440 | differs | By page, Needs you: answered cards leave the count. A4 did not reproduce on this fresh desktop box. | needs-confirm/1440 / fresh box 1; standalone repeat after A4 | [pair](walk-again/needs-confirm/1440/05-needs-count-1440-pair.png) |
| 5 | Gallery | proposal | 420 | unreachable | W3: project settings proposal unreachable; needs-proposal gallery supplies a goal-proposal card only; configure-proposal shows the settings card. | 420-verified / gallery needs-proposal; box 3 attempt | [pair](walk-again/420-verified/05-proposal-420-pair.png) |
| 5 | Gallery | proposal | 1440 | unreachable | W3: project settings proposal unreachable; needs-proposal gallery supplies a goal-proposal card only; configure-proposal shows the settings card. | 1440 / gallery needs-proposal; box 3 attempt | [pair](walk-again/1440/05-proposal-1440-pair.png) |
| 5 | Needs you | proposal-missing | 420 | differs | W3: ctl proposal has project 0 and is absent from project Needs you. No question remains. | 420-verified / box 3 | [pair](walk-again/420-verified/05-proposal-missing-420-pair.png) |
| 5 | Needs you | proposal-missing | 1440 | differs | W3: ctl proposal has project 0 and is absent from project Needs you. A4: unanswered question still counted. | 1440 / box 3 | [pair](walk-again/1440/05-proposal-missing-1440-pair.png) |
| 5 | Needs you | question-with-choices | 420 | matches | Same structure, controls and look; scripted data differs. | 420-verified / box 3 | [pair](walk-again/420-verified/05-question-with-choices-420-pair.png) |
| 5 | Needs you | question-with-choices | 1440 | matches | Same structure, controls and look; scripted data differs. | 1440 / box 3 | [pair](walk-again/1440/05-question-with-choices-1440-pair.png) |
| 5 | Needs you | reply-answered | 420 | differs | D5: five-second send delay; reply words restored on Undo. | 420-verified / box 3 | [pair](walk-again/420-verified/05-reply-answered-420-pair.png) |
| 5 | Needs you | reply-answered | 1440 | differs | D5: five-second send delay; reply words restored on Undo. | 1440 / box 3 | [pair](walk-again/1440/05-reply-answered-1440-pair.png) |
| 5 | Needs you | reply-open | 420 | matches | Same structure, controls and look; scripted data differs. | 420-verified / box 3 | [pair](walk-again/420-verified/05-reply-open-420-pair.png) |
| 5 | Needs you | reply-open | 1440 | matches | Same structure, controls and look; scripted data differs. | 1440 / box 3 | [pair](walk-again/1440/05-reply-open-1440-pair.png) |
| 5 | Needs you | undo | 420 | differs | D5; By page, Needs you: Undo restores reply words. | 420-verified / box 3 | [pair](walk-again/420-verified/05-undo-420-pair.png) |
| 5 | Needs you | undo | 1440 | differs | D5; By page, Needs you: Undo restores reply words. | 1440 / box 3 | [pair](walk-again/1440/05-undo-1440-pair.png) |
| 6 | Home | away-on | 420 | differs | D10; By page, Away: implementation/retry/wording only. | 420-verified / box 4 | [pair](walk-again/420-verified/06-away-on-420-pair.png) |
| 6 | Home | away-on | 1440 | differs | D10; By page, Away: implementation/retry/wording only. | 1440 / box 4 | [pair](walk-again/1440/06-away-on-1440-pair.png) |
| 6 | Home | decided-for-you | 420 | differs | D10; By page, Away: Home only, 24 hours. | 420-verified / box 4 | [pair](walk-again/420-verified/06-decided-for-you-420-pair.png) |
| 6 | Home | decided-for-you | 1440 | differs | D10; By page, Away: Home only, 24 hours. | 1440 / box 4 | [pair](walk-again/1440/06-decided-for-you-1440-pair.png) |
| 6 | Home | goal-decided | 420 | differs | D10: decided tool; By page, Away: Home only, 24 hours. | 420-verified / box 4 | [pair](walk-again/420-verified/06-goal-decided-420-pair.png) |
| 6 | Home | goal-decided | 1440 | differs | D10: decided tool; By page, Away: Home only, 24 hours. | 1440 / box 4 | [pair](walk-again/1440/06-goal-decided-1440-pair.png) |
| 6 | Home | overturned | 420 | differs | By page, Away: overturn while the task lives. | 420-verified / box 4 | [pair](walk-again/420-verified/06-overturned-420-pair.png) |
| 6 | Home | overturned | 1440 | differs | By page, Away: overturn while the task lives. | 1440 / box 4 | [pair](walk-again/1440/06-overturned-1440-pair.png) |
| 7 | Chats | back-in | 420 | unreachable | Mock has no signed-out page; client signed back in using a fresh printed link. | 420-verified / box 2 | [pair](walk-again/420-verified/07-back-in-420-pair.png) |
| 7 | Chats | back-in | 1440 | unreachable | Mock has no signed-out page; client signed back in using a fresh printed link. | 1440 / box 2 | [pair](walk-again/1440/07-back-in-1440-pair.png) |
| 7 | Home | project-added | 420 | unreachable | A1 fixed: Enter-created project focuses h1 without an outline; next keyboard link retains its 2px ring. Mock new-project creation is a toast. | focus-final/420 / fresh box 1; A1 sheet at 67f012c1c1 | [pair](walk-again/focus-final/420/07-project-added-420-pair.png) |
| 7 | Home | project-added | 1440 | unreachable | A1 fixed: Enter-created project focuses h1 without an outline; next keyboard link retains its 2px ring. Mock new-project creation is a toast. | focus-final/1440 / fresh box 1; A1 sheet at 67f012c1c1 | [pair](walk-again/focus-final/1440/07-project-added-1440-pair.png) |
| 7 | Project switcher | project-rename | 420 | differs | A2: rename form sits below the rows with a visible Cancel; mock replaces the selected row. | focus-final/420 / fresh box 1; A1 sheet at 67f012c1c1 | [pair](walk-again/focus-final/420/07-project-rename-420-pair.png) |
| 7 | Project switcher | project-rename | 1440 | differs | A2: rename form sits below the rows with a visible Cancel; mock replaces the selected row. | focus-final/1440 / fresh box 1; A1 sheet at 67f012c1c1 | [pair](walk-again/focus-final/1440/07-project-rename-1440-pair.png) |
| 7 | Home | project-switched | 420 | matches | Same structure, controls and look; scripted data differs. | 420-verified / box 2 | [pair](walk-again/420-verified/07-project-switched-420-pair.png) |
| 7 | Home | project-switched | 1440 | matches | Same structure, controls and look; scripted data differs. | 1440 / box 2 | [pair](walk-again/1440/07-project-switched-1440-pair.png) |
| 7 | Project switcher | project-switcher | 420 | matches | Same structure, controls and look; scripted data differs. | 420-verified / box 2 | [pair](walk-again/420-verified/07-project-switcher-420-pair.png) |
| 7 | Project switcher | project-switcher | 1440 | matches | Same structure, controls and look; scripted data differs. | 1440 / box 2 | [pair](walk-again/1440/07-project-switcher-1440-pair.png) |
| 7 | Project switcher | rename-cancel-focus | 420 | differs | A3: Cancel focuses main instead of the switcher; A1 removes the page target outline. | focus-final/420 / fresh box 1; A1 sheet at 67f012c1c1 | [pair](walk-again/focus-final/420/07-rename-cancel-focus-420-pair.png) |
| 7 | Project switcher | rename-cancel-focus | 1440 | differs | A3: Cancel focuses main instead of the switcher; A1 removes the page target outline. | focus-final/1440 / fresh box 1; A1 sheet at 67f012c1c1 | [pair](walk-again/focus-final/1440/07-rename-cancel-focus-1440-pair.png) |
| 7 | Project switcher | rename-escape-focus | 420 | differs | A3: Escape closes the editor but focus goes to the new-project field instead of the switcher. | focus-final/420 / fresh box 1; A1 sheet at 67f012c1c1 | [pair](walk-again/focus-final/420/07-rename-escape-focus-420-pair.png) |
| 7 | Project switcher | rename-escape-focus | 1440 | differs | A3: Escape closes the editor but focus goes to the new-project field instead of the switcher. | focus-final/1440 / fresh box 1; A1 sheet at 67f012c1c1 | [pair](walk-again/focus-final/1440/07-rename-escape-focus-1440-pair.png) |
| 7 | Accounts | settings-accounts | 420 | differs | By page, Settings: tab contents use the mock language; mock tab not drawn. | 420-verified / box 2 | [pair](walk-again/420-verified/07-settings-accounts-420-pair.png) |
| 7 | Accounts | settings-accounts | 1440 | differs | By page, Settings: tab contents use the mock language; mock tab not drawn. | 1440 / box 2 | [pair](walk-again/1440/07-settings-accounts-1440-pair.png) |
| 7 | Builders | settings-builders | 420 | differs | By page, Settings: tab contents use the mock language; mock tab not drawn. | 420-verified / box 2 | [pair](walk-again/420-verified/07-settings-builders-420-pair.png) |
| 7 | Builders | settings-builders | 1440 | differs | By page, Settings: tab contents use the mock language; mock tab not drawn. | 1440 / box 2 | [pair](walk-again/1440/07-settings-builders-1440-pair.png) |
| 7 | Forges | settings-forges | 420 | differs | By page, Settings: tab contents use the mock language; mock tab not drawn. | 420-verified / box 2 | [pair](walk-again/420-verified/07-settings-forges-420-pair.png) |
| 7 | Forges | settings-forges | 1440 | differs | By page, Settings: tab contents use the mock language; mock tab not drawn. | 1440 / box 2 | [pair](walk-again/1440/07-settings-forges-1440-pair.png) |
| 7 | Repositories | settings-repositories | 420 | differs | By page, Settings: tab contents use the mock language; mock tab not drawn. | 420-verified / box 2 | [pair](walk-again/420-verified/07-settings-repositories-420-pair.png) |
| 7 | Repositories | settings-repositories | 1440 | differs | By page, Settings: tab contents use the mock language; mock tab not drawn. | 1440 / box 2 | [pair](walk-again/1440/07-settings-repositories-1440-pair.png) |
| 7 | Sign out | signed-out | 420 | unreachable | Mock has no signed-out page; client signed out from Accounts. | 420-verified / box 2 | [pair](walk-again/420-verified/07-signed-out-420-pair.png) |
| 7 | Sign out | signed-out | 1440 | unreachable | Mock has no signed-out page; client signed out from Accounts. | 1440 / box 2 | [pair](walk-again/1440/07-signed-out-1440-pair.png) |
| 7 | Sign out | signed-out-reloaded | 420 | unreachable | Mock has no signed-out page; client remained signed out after reload. | 420-verified / box 2 | [pair](walk-again/420-verified/07-signed-out-reloaded-420-pair.png) |
| 7 | Sign out | signed-out-reloaded | 1440 | unreachable | Mock has no signed-out page; client remained signed out after reload. | 1440 / box 2 | [pair](walk-again/1440/07-signed-out-reloaded-1440-pair.png) |

## Earlier findings, checked again

| finding | result in this walk |
|---|---|
| W1 | Verified: fast first-run Settings has all four controls available at both widths, with no ctl fallback. The transient-failure retry is also covered by its passing domain test; no live read fault was injected. |
| W2 | Verified: Home explains proposing how work runs and accepting it before anything changes, at both widths. Repository readiness makes no unsupported CI claim. |
| W3 | Still open: the ctl proposal retains project 0 and never appears in project 1's Needs you. At 1440 its question remains because of A4; the missing proposal is a separate fault. |
| W4 | Verified: client and mock document section headings are 17px/28.9px at both widths. |
| W5 | Verified: the phone hint is absent; both Configure textareas are x=31, y=945, width=318, height=26. Desktop textareas also align at x=491, y=913.8125, width=650, height=26. |
| W6 | Still open: the 31-minute return showed computed Home with no brief at both widths; the entire 8,117,760 period remained reserved. |
| W7 | Verified: only working notes give no Draft control. A document after five lower-numbered working notes is found on reopening, across three ctl pages; the goal's Draft link returns to its chat. Both widths passed. |
| R1 | Still large and open; no fix is claimed. The real-fetch repeat is conditional on that repair, which has not happened. |

## fabrig/ui

- **W3 — large:** first run, enable features, start a chat; use ctl to propose project 1's project-ways at the current settings revision. Read the proposal and unfiltered inbox: project 0. Open project Needs you: no settings-proposal card or usable count for it. Evidence: `walk-again/1440/proposal-1440.json`, `inbox-proposal-1440.json`, their phone counterparts, and both proposal-missing pairs.
- **W6 — large:** land the local hello change, close Home and leave its box running for 31 minutes, then reopen it. Home still says “Nothing needs you”, without a brief, byline or admission error; the real landing remains in Changes. Evidence: both Home-return pairs and the named gallery substitute; funding/admission remains the earlier inference, not a proven sole cause.
- **R1 — large:** start a real box with explicit config/data/cache directories and a fresh home, choose Claude at Model, and run its displayed `temper program-login claude-owner` verbatim. It looks in the home's defaults and refuses before downloading. Exporting the box's directory variables reaches the pinned fetch. Reproduction and the default/custom-directory command requirement are in `real-checks.md`; that repair remains owed.
- **A4 — large:** at 1440, start the local away goal while here, choose Later, Reply, send Use the small implementation, Undo, then immediately press that offered choice. In the first run the question/count 1 remained after six seconds, and ctl still retained the unanswered question. A fresh desktop repeat and the phone run passed. Keep both `walk-again/1440/05-needs-count-1440-pair.png` and `walk-again/needs-confirm/1440/05-needs-count-1440-pair.png`; the cause and reliable reproduction remain unknown.
- **A2 — small, left:** open the project switcher and Rename at either width. The client keeps the selected row and places the editor below it, with secondary Save, visible Cancel and a 25.1875px input; the mock replaces the row, uses primary Save, no Cancel, and a 38px field (34px minimum). Both final project-rename pairs reproduce it.
- **A3 — small, left:** from Rename, press Escape: it cancels but focuses New project. Open Rename again and press Cancel: main gets focus. The carried answer asks for the opener or switcher summary. Both final closing-state pairs and `focus-metrics.json` retain it. A1 repairs the page outline only.

## Real checks

These are the **2026-11-05 non-fake observations at `a040302420`**, carried
forward from `real-checks.md`, with their limits. They were not rerun here.

| check | verdict |
|---|---|
| Claude offered with no claude on PATH | Passed on a fresh real box. |
| Displayed program-login command with explicit deployment directories | Failed R1; still unresolved. |
| Real fetch with the box's directory variables supplied | Passed: 293,776,752 bytes, SHA-256 `86dc1fcc69498159eaf15f446613e847010f9036bfbbdffd4c49350be8489ada`, version 2.1.311; launched to its first sign-in prompt and stopped there. A second run fetched nothing. Command start to final file write was 5.266–6.267 seconds; network-only time was not measured. |
| Browser Sign out on real serve | Passed 7/7 across 1440 and 420: reload and a typed address stayed signed out, credentialed stream requests returned 401, no 503 was observed. Five repetitions exercised small client-call limits within the standard engine profile. |
| Repeat fetch at this tip | Not required yet: `real-checks.md` requires it after R1's command repair. R1 remains moved and unfixed. The successful exported-variable run does not prove the displayed command. |

Still unproved: provider sign-in or paid calls; TLS/proxy/remote deployments;
a populated real Accounts list and every redraw race; the full small engine
profile; other download platforms, corruption, failed downloads or concurrent
fetches; copying an external public repository. No credentials or provider
sign-in were used. The two fake Sign outs in this walk add no non-fake claim.

## Notes and implementation choices

The committed driver now follows the fixed pages, takes the fast Settings path,
and distinguishes a queued title from an observed landing. The first desktop
landed capture was corrected through the browser's existing Changes history
entry, without reopening Home during the timed absence. The queue's earlier
live row is retained. One phone attempt was stopped and cleaned up while this
wait was corrected, then replayed on a fresh box. The initial focus-only wait
required one particular Home headline; it was narrowed to the selected project,
since the focused target is the rule being checked.

Two additive fake cues replace the chat's current document with five working
notes, then restore its document as a later note. Existing draft cues and their
literal revisions remain unchanged. Public ctl note pages retain the numbers.
No stored format, protocol or public interface changed. The only client repair
here is A1: a local stylesheet rule for programmatic main/h1 targets, under the
plan's Home marker, preserving control rings. The new CSS test failed first
because that selector was absent, then passed; fresh-browser proofs checked the
computed styles and the following Tab's visible control ring.

At 420, Chats supplies the way back from a conversation, Settings is in the
project switcher once Goals occupies its tab, and Home supplies Changes. Split
and the desktop rail menu remain intentionally absent. Sign out was last on
each Settings box, reload stayed signed out, and a fresh printed link supplied
the way back. No client address was typed as a shortcut. Gallery/file addresses
and the historical Configure reference are named explicitly on their rows.

The final fake cannot produce delegated goal tasks, a temper-owned held task or
a configuring proposal. ctl stop produces an owner hold. The gallery supplies
the named alternatives. Its needs-proposal card is a goal proposal, so it is
only a substitute for the missing project-settings inbox card; the separate
configuring-gallery capture shows settings-proposal words. The mock's other
Settings tabs, new-project creation and Sign out have no implementation, and
its native Stop dialog is accepted by CDP. The desktop W7 reference was aligned
to Draft mode beside the unchanged live capture before this final review.

All builds/tests used heavy, one at a time, with `TEMPER_BUNDLE=none` except
the separate bundle builder. Verification passed: one focused fake-format test;
1,689 focused tests across temper-fake and its six direct consumers; five
existing W1/W7 domain tests; one process-profile document walk; the new focus
test green after its recorded red; 46 terminal-golden tests; and 526 focused
tests across the web shell and its direct consumers. Clippy passed for both
changed packages with all targets and warnings denied. Formatting and temper's
old-form ratchet passed. The gallery and journey prints were regenerated;
there was no generated tracked diff. Citations found zero new broken references against origin/main.

All 15 boxes printed Stopped; all Chromium processes started by these drivers
ended, and every boxes/ and build/ directory under walk-again was removed.
Screenshots, public read evidence, logs and contact pages remain. Both full
walks wrote their final run.json records and all selected captures, and their
logs contain no exception. Their surrounding command sessions nevertheless
reported exit 143 afterward; the cause is unknown. Those exit statuses are
retained as a runner limitation, alongside the completed artifacts and
verified cleanup, rather than represented as successful command exits.
