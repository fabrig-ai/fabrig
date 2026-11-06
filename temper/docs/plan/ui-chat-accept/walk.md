# The configuring chat's Accept, in a browser

2026-11-06. The fresh fake-box walk of
`temper/docs/plan/ui-chat-accept/tasks/chat-walk.md`. The observed code is
`c8cb120e99cc5f6fbbcce94393246cfdfb12b411`, including chat-decides
`30f66ed1b3433f3d37d854db74f74a44be120d73`. No product code changed in this walk.

The six selected runs use independent `temper box --fake-backends` boxes:
Chat Accept, Needs you Accept, and Needs you Reject, each at 1440 and 420.
Each completes ChatGPT first run, opens Settings → Start and sends exactly
“Show drafts and goals for this project.” before any settings edit. No ctl
write substitutes for a browser action. The proposal is number 1, owned by
project 0 with `projects: [1]`; its chat is T1 in Hello, project 1. The pending
public inbox has that proposal; after either decision its rows are empty.

Pictures and page text are outside the worktree, under
`~/src/fabrig/ui-chat-accept/`. Every picture path below is relative to that
folder; its adjacent `.json` records the page's text, controls, viewport and
sheet rectangle. `build.json` in each run records the builder, source commit,
binary hash, bundle and start time. `finished.json` records completion.
The browser is driven through the existing `ui-walk-3/driver/cdp.py`, using
visible controls and real pointer/key events. Both widths use
`Emulation.setDeviceMetricsOverride`, height 1000, scale 1; the 420 captures
are 420 pixels wide. The throwaway script is deleted before the push.

## Chat Accept

| Width | Step | Seen | Picture path |
|---|---|---|---|
| 1440 | 1, first run | ChatGPT plan usable; Hello created; To begin reached. | `chat-walk-1-1440-v2/chat-accept-1440-01-first-run.png` |
| 1440 | 1, Settings | Settings opened without changing a switch. | `chat-walk-1-1440-v2/chat-accept-1440-02-settings.png` |
| 1440 | 1, Start | Configure temper, its introduction and composer. | `chat-walk-1-1440-v2/chat-accept-1440-03-start.png` |
| 1440 | 1, cue sent | Exact cue sent from Configure, before any settings change. | `chat-walk-1-1440-v2/chat-accept-1440-04-cue-sent.png` |
| 1440 | 2 | Pending card: Accept, Change something; rail Needs you 1, no Goals. | `chat-walk-1-1440-v2/chat-accept-1440-05-pending.png` |
| 1440 | 3, sheet | Accept this proposal?; Not yet then primary Accept; centred. | `chat-walk-1-1440-v2/chat-accept-1440-07-accept-sheet.png` |
| 1440 | 3, Escape | Sheet gone; card pending, count 1; public inbox unchanged. | `chat-walk-1-1440-v2/chat-accept-1440-08-escape.png` |
| 1440 | 3, Accept again | Same sheet opened again, then its primary pressed. | `chat-walk-1-1440-v2/chat-accept-1440-09-accept-again.png` |
| 1440 | 4, card | Sheet gone, no refusal; “✓ Accepted · new work follows it”; count gone, Goals and + Draft appear. | `chat-walk-1-1440-v2/chat-accept-1440-10-accepted.png` |
| 1440 | 4, Chats | Goals in rail; Chats offers Drafts; Needs you has no count. | `chat-walk-1-1440-v2/chat-accept-1440-11-rail-and-drafts.png` |
| 1440 | 5 | Same T1: further message, then a new “Hello from the local fake model.” below it. | `chat-walk-1-1440-v2/chat-accept-1440-12-follow-up.png` |
| 420 | 1, first run | ChatGPT plan usable; Hello created; To begin reached. | `chat-walk-1-420/chat-accept-420-01-first-run.png` |
| 420 | 1, Settings | Settings opened without changing a switch. | `chat-walk-1-420/chat-accept-420-02-settings.png` |
| 420 | 1, Start | Configure temper, its introduction and composer. | `chat-walk-1-420/chat-accept-420-03-start.png` |
| 420 | 1, cue sent | Exact cue sent before any settings change. | `chat-walk-1-420/chat-accept-420-04-cue-sent.png` |
| 420 | 2, card | Pending card with visible Accept and Change something. | `chat-walk-1-420/chat-accept-420-05-pending.png` |
| 420 | 2, count | Back through Settings to Chats: bottom navigation Needs you 1; no Goals or Drafts. | `chat-walk-1-420/chat-accept-420-06-pending-rail.png` |
| 420 | 3, sheet | Returned through T1; centred sheet with Not yet and primary Accept. | `chat-walk-1-420/chat-accept-420-07-accept-sheet.png` |
| 420 | 3, Escape | Sheet gone; card pending; public inbox unchanged. | `chat-walk-1-420/chat-accept-420-08-escape.png` |
| 420 | 3, Accept again | Same sheet reopened, then its primary pressed. | `chat-walk-1-420/chat-accept-420-09-accept-again.png` |
| 420 | 4, card | Sheet gone, no refusal; “✓ Accepted · new work follows it”, + Draft. | `chat-walk-1-420/chat-accept-420-10-accepted.png` |
| 420 | 4, Chats | Bottom navigation has Goals, Needs you without count; Chats offers Drafts. | `chat-walk-1-420/chat-accept-420-11-rail-and-drafts.png` |
| 420 | 5 | Same T1: further message followed by a new “Hello from the local fake model.” | `chat-walk-1-420/chat-accept-420-12-follow-up.png` |

The follow-up is “Please confirm you can still answer in this chat.” The
observation requires the fake reply to occur **after** that message; the
same words also occur in the model's response to the acceptance itself and
do not establish continuation. The accepted card remains after returning
through Chats. No reload is used for that return: ended cards are held for
the session (`temper/docs/design/client/README.md`, 5.13).

The sheets' rectangles are x=490, y=392.953125, width=460,
height=214.09375 at 1440; x=18, y=378.71875, width=384,
height=242.5625 at 420. Escape leaves the public inbox byte-for-byte equal
to its read before Escape. Pending/accepted inbox reads and the follow-up
result are retained in each run's JSON.

## Accept from Needs you

These additional fresh boxes repeat the same first run and cue, then decide
in Needs you. Each also returns through Chats to T1 and receives a new fake
reply to the same follow-up.

| Width | Step | Seen | Picture path |
|---|---|---|---|
| 1440 | Pending | Needs you has the proposal, Accept and count 1. | `chat-walk-1-needs-1440/needs-accept-1440-07-needs.png` |
| 1440 | Sheet | Accept sheet opens; Escape dismisses it with the inbox unchanged. | `chat-walk-1-needs-1440/needs-accept-1440-07-accept-sheet.png` |
| 1440 | Escape | Pending proposal still present after Escape. | `chat-walk-1-needs-1440/needs-accept-1440-08-escape.png` |
| 1440 | Primary | Reopened sheet's Accept completes; count gone. | `chat-walk-1-needs-1440/needs-accept-1440-09-needs-accepted.png` |
| 1440 | Chat | Original chat's card says “✓ Accepted · new work follows it”. | `chat-walk-1-needs-1440/needs-accept-1440-10-accepted.png` |
| 1440 | Continuation | New fake reply below the further message in T1. | `chat-walk-1-needs-1440/needs-accept-1440-12-follow-up.png` |
| 420 | Pending | Needs you has the proposal, Accept and count 1. | `chat-walk-1-needs-420/needs-accept-420-07-needs.png` |
| 420 | Sheet | Accept sheet opens; Escape dismisses it with the inbox unchanged. | `chat-walk-1-needs-420/needs-accept-420-07-accept-sheet.png` |
| 420 | Escape | Pending proposal still present after Escape. | `chat-walk-1-needs-420/needs-accept-420-08-escape.png` |
| 420 | Primary | Reopened sheet's Accept completes; count gone. | `chat-walk-1-needs-420/needs-accept-420-09-needs-accepted.png` |
| 420 | Chat | Original chat's card says “✓ Accepted · new work follows it”. | `chat-walk-1-needs-420/needs-accept-420-10-accepted.png` |
| 420 | Continuation | New fake reply below the further message in T1. | `chat-walk-1-needs-420/needs-accept-420-12-follow-up.png` |

## Reject from Needs you

| Width | Step | Seen | Picture path |
|---|---|---|---|
| 1440 | Pending | Fresh proposal in Needs you, count 1. | `chat-walk-1-reject-1440-v3/reject-1440-07-needs.png` |
| 1440 | Sheet | Reject reason “Keep the current project settings.”; Reject sheet opens. | `chat-walk-1-reject-1440-v3/reject-1440-08-reject-sheet.png` |
| 1440 | Primary | Reject completes; sheet gone; card says Rejected; count gone. | `chat-walk-1-reject-1440-v3/reject-1440-09-rejected-needs.png` |
| 1440 | Original chat | Proposal title and reason remain, with no decision label or acts. | `chat-walk-1-reject-1440-v3/reject-1440-10-rejected-chat.png` |
| 420 | Pending | Fresh proposal in Needs you, count 1. | `chat-walk-1-reject-420/reject-420-07-needs.png` |
| 420 | Sheet | Same reason; Reject sheet opens. | `chat-walk-1-reject-420/reject-420-08-reject-sheet.png` |
| 420 | Primary | Reject completes; sheet gone; card says Rejected; count gone. | `chat-walk-1-reject-420/reject-420-09-rejected-needs.png` |
| 420 | Original chat | Proposal title and reason remain, with no decision label or acts. | `chat-walk-1-reject-420/reject-420-10-rejected-chat.png` |

At both widths the rejected chat card reads:

> HOW TEMPER WILL RUN HELLO
>
> Show drafts and goals
>
> temper enforces
>
> Drafts and goals are available in this project.
>
> your agents follow
>
> Keep each change checked.
>
> Running work stays on its held revision.

It says neither “Rejected” nor “Accepted”; Accept and Change something are
absent. Above it, the reason sent by the person and the fake model's reply
are visible. Goals and Drafts remain unavailable. This records what the
card says; it does not establish a new rejection-wording contract.

## Differences from the mock

The same cue is also sent through the first-run mock's configuring chat.
Adjacent `-mock.png` and `-mock.json` files retain Start, pending and accepted
states for comparison with `temper/docs/design/client/greenfield/chat.html`.

- The live project is Hello with a repository; the mock calls it Default.
  Live Configure has the full introduction, the three topics, Ways to begin,
  Models by role and temper's own way. The mock has its older short
  introduction and three sentence starters.
- The live desktop keeps its rail and Settings breadcrumb; the mock chat
  has no rail and says Chats. The live title becomes the cue; the mock keeps
  “Tell temper how you work”. The live model line says “ChatGPT plan” and no
  high effort. Live Stop, Close and waiting status are present. At 420 those
  acts occupy a second bar and the model line is hidden. The live phone
  composer hides its Enter/Shift+Enter hint; the mock shows it.
- The live conversation draws the propose_settings call, a “Decision made”
  line while the proposal still waits, and its waiting age. The mock draws
  neither the call nor those lines. The fake model says “I proposed showing
  drafts and goals. Accept the proposal when you are ready.”; the mock says
  “Here’s how I’ll run that. Nothing changes until you accept.”
- The live card has the reason's six plain paragraphs. It lacks the mock's
  item/value rows in small mono type and its ruled, bold “temper enforces” /
  “Agents follow” summary. The live item-value gap is already recorded in
  `temper/docs/design/client/README.status.md`, 5.13. Both have the title,
  primary Accept and quiet Change something.
- Live Accept opens the confirmation; the mock accepts immediately. Both
  end with “✓ Accepted · new work follows it” and offer + Draft. Live Goals,
  Drafts and Away appear, but Changes stays hidden: the fake proposal sets
  changes false, whereas the mock's scripted proposal sets it true. The
  live fake replies “Hello from the local fake model.”; the mock adds its
  longer acceptance explanation and toast.
- The live card stays after the latest assistant reply, including the
  follow-up reply. The mock's accepted card precedes its acceptance reply.
  Needs you keeps its decided card in the flow while clearing the count;
  the mock has an answered stack. The live rejected chat card has no status
  line, as recorded above; the mock chat has no direct Reject act.

These differences were observed at both widths. No product repair was
needed for this task. B1's refusal was not reproduced; W3's project count
and navigation change were seen again. `finding-400` was not met: all four
Accept boxes answered the subsequent message with a new reply.

## Driver limits and retained attempts

The selected six runs complete successfully and record box `Stopped` in
their copied `box-1-lifecycle.log`; cleanup warnings are null and both box
and Chromium exits are 0. The inherited cleanup JSON's `stopped_marker`
describes stdout, not the lifecycle log.

Earlier attempts remain beside the selected runs. `chat-walk-1-1440`
completed acceptance but its driver reloaded the chat, losing its
session-held card. `chat-walk-1-reject-1440` toggled the Reject disclosure
instead of pressing the second control with the same accessible name.
`chat-walk-1-reject-1440-v2` completed rejection, then waited for an older
reply hidden behind Older. Their `fault.json` files retain those script
failures. The script was repaired in each case and replayed on a fresh box;
those failed attempts do not supply the selected proof. No “key mark caught
a moving page” occurred and no mark wait was widened.
