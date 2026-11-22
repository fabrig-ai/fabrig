# The gallery's journey and interaction states

The gallery's journey and interaction states are split from [the gallery](gallery.md).

## 1. Journeys and interactions

| Name | The state |
|---|---|
| `chat-solo-T10` | the solo world's chat T10, its proposal in place |
| `inbox-solo` | the solo world's *Needs you* (`../pages/README.md`): the first of a proposal and a question |
| `needs-one` | *Needs you*: a question in the line form, the first of three |
| `needs-saw-open` | `needs-one` with "What the agent saw" open |
| `needs-reply-open` | `needs-one` with "Reply…" pressed: the reply form |
| `needs-second-answered` | the first answered, its line and toast showing: the second of three |
| `needs-later` | the first marked later: the next shown, the toast's words |
| `needs-last` | the last card: "The last one" |
| `needs-done` | every card answered: "Nothing needs you." and the answered lines |
| `needs-proposal` | a proposal's card |
| `needs-held` | a held task's card |
| `needs-funds` | a held task out of budget: "S of L used." |
| `needs-plain-words` | a question in no line form: "Reply…" alone |
| `needs-decided` | the unfiltered *Needs you*: a question, then the region "Decided for you · 2", "for 24 hours, then in each task's history" at its head's right, with rows of two projects, each "PROJECT · TITLE · 01:05" |
| `needs-decided-project` | *Needs you* filtered by one project: a question, then the region "Decided for you · N" with "Limits" at its head's right, each row "TITLE · 01:05" (`../pages/home.md`, 1, *Decided for you*) |
| `needs-why` | a question whose asked row says why: "Reached you because it is an implementation choice, which asks you unless you’re away." then "Keep asking" and "Stop asking: let agents decide implementation choices" on the line |
| `needs-stop-asking` | `needs-why` with its rule open for an owner: "Let agents decide implementation choices, then tell you." with "Apply"; "Keep asking" on the line, none under the offers |
| `needs-stop-asking-set` | a question that changes a path set, its rule "Stop asking: let agents decide some changes to protocol core" open for a member: the words offer's sentence over its field, the split, each with "Propose" |
| `needs-sign-off-part` | a sign-off's card open for a member, its rule by path asking two members, one signed: "This project's landing rule 1 asks for 2 approvals by members.", "Signs off changes to payments.", "1 of 2 signed: Sam; you or Kit can give the second." and "Sign off on Forgejo ↗" first (`../forge/README.md`, 4.1, *Its context*; `whole-story/team/needs.html`) |
| `needs-only-you` | the unfiltered *Needs you* (`../pages/README.md`, **Only you**): a question, then "Only you · 2", a signed-out ChatGPT sign-in and an expiring Claude Code login, each with "Sign in again" |
| `needs-only-you-done` | no card open: "Nothing needs you.", "Home", then "Only you · 1", a signed-out ChatGPT sign-in |
| `night-needs-only-you` | `needs-only-you` with the night look on |
| `accounts-expiring` | Accounts (`../pages/accounts-words.md`, 1, *Each state*): a ChatGPT sign-in read as expiring, "Expiring: sign in again before it ends." and "Continue with ChatGPT" |
| `goal-running` | first-run-projects, running: six of nine tasks in, its report, Left and Risk, the bar, three waiting lines, "6 landed" folded |
| `goal-needs-funds` | first-run-projects held for funds: "needs funds", its band with Release and Leave held |
| `goal-needs-question` | welcome-stall with an open question: its band and its parsed choices, the recommended one primary |
| `goal-planning` | forge-cards, no report and no tasks: the planning words, the dots and "Proposing tasks from the draft…" |
| `goal-paused` | first-run-projects paused by the person: "paused by you", Resume |
| `goal-landed` | pane-chat-line landed: its result, "Landed at 01:12.", no acts and no composer |
| `goal-landed-open` | `goal-landed` with "N landed" pressed: its rows under it |
| `goal-closed` | welcome-stall closed: "Closed by you…", "Closed before it landed." |
| `goal-failed` | forge-cards failed: "Failed: " and its reason, "Failed before it landed." |
| `goal-no-report` | first-run-projects running with tasks and no report: "N of M tasks done.", no byline |
| `goal-amending` | `goal-running` with Amend pressed: the editor, between the header and the report |
| `goal-closing` | `goal-running` with Close pressed: its confirmation |
| `goal-toast` | `goal-running` paused, its toast showing |
| `goal-sending` | `goal-running` with the person's words sent and one being sent, the composer holding its next words |
| `start-goal` | Goals with "Start one from a draft" pressed: the sheet with no draft, its Draft field and two ways |
| `start-goal-draft` | the sheet opened with ../../draft/first-run-projects.md, its name, its default way and "When it needs you", as the project says |
| `start-goal-refused` | `start-goal-draft` after a refused start: "Always ask" chosen, `.sheet-error` in jig's words |
| `task-solo-T3` | the solo world's goal T3, on `goal.html`'s page |
| `accounts-solo` | the solo world's accounts |
| `chats-first-run-1` | the first run's Chats at step 1: an empty list |
| `welcome-first-run-0` | the first run's welcome: its two cards, their note, then temper box's paragraph |
| `welcome-go-on` | Welcome asked for with a real forge under way: Go on setting up, no playground or note, temper box's paragraph after the card |
| `connect-forge-first-run-6` | the first run's `ConnectForge` at step 6: Forgejo here setting up, the box preparing |
| `forges-first-run-7` | the first run's Forges at step 7: Forgejo here connected |
| `welcome-forge-choose` | the welcome's Forge choices, before any forge is connected |
| `welcome-forge-details` | the welcome's Forgejo Details, the sheet "Local forge" with the sample chosen by default and no fields |
| `welcome-sheet-copies` | Forgejo's sheet with public copy addresses, in the order written |
| `welcome-sheet-copies-refused` | Forgejo's sheet refuses an invalid copy address above its foot |
| `welcome-sheet-nothing` | Forgejo's sheet with Nothing for now selected |
| `welcome-forge-under-way` | the Forge step's status in the column, Each step, and Continue while Forgejo sets up |
| `welcome-forge-connected` | the Forge step's connected Forgejo and first repository, with Continue |
| `welcome-forge-github` | the Forge step's accepted GitHub installation, with Continue |
| `github-app-details` | GitHub's own-App form, with masked key input |
| `github-install` | GitHub's connected App, installed nowhere yet |
| `github-picker` | the installation's reached repositories, ready to adopt |
| `github-adopted` | the adopted GitHub repository and its primary Open act |
| `welcome-model-choose` | the welcome's Model choices, Claude first and the default, then ChatGPT, with no terminal to lend |
| `welcome-model-claude-terminal` | Model at first run step 7, Forgejo connected, a terminal to lend: Claude's card first, "Signs in from the terminal where temper runs" and the primary "Sign in to Claude Code", then ChatGPT's (`claude-sign-in/`, 01) |
| `welcome-model-claude-signing-in` | the same, pressed: "Claude Code is signing in, in the terminal where temper runs", its muted line, Cancel and the folded "No tab opened, or a code to paste?" (02) |
| `welcome-model-claude-signed-in` | the same, signed in: ticked "Claude subscription", "through Claude Code, signed in", Continue, and the pane's Model line in the same words (04) |
| `welcome-model-claude-no-terminal` | the same with no terminal to lend: Claude's card first, "One command in a terminal, on this machine" and the primary "Use your Claude subscription" (05) |
| `welcome-model-claude-not-finished` | the cards with a terminal to lend again, and "Claude Code’s sign-in didn’t finish." under the lead |
| `welcome-model-forge-under-way` | Model at first run step 6, Forgejo setting up, its Forge pane and Each step |
| `welcome-model-forge-fetching` | Model at first run step 6, Forgejo fetching at 42%, its bar filled and the figure at its end |
| `welcome-model-forge-fetching-bytes` | Model at first run step 6, Forgejo fetching with no size, its moving bar and 41 MB at its end |
| `welcome-model-forge-done` | Model at first run step 7, Forgejo connected, its marked Forge pane without Each step |
| `welcome-model-forge-failed` | Model at first run step 9, Forgejo failed, its Forge pane and Each step |
| `welcome-model-claude` | Claude’s welcome command and I’ve signed in, with no Name field |
| `welcome-model-key-details` | the welcome's API key form, without an unsupported display-name field |
| `welcome-model-connected` | a usable ChatGPT plan, its ticked status and Continue to Ready |
| `welcome-model-away` | the page waiting for ChatGPT’s sign-in, its note, "Open it again" and folded paste |
| `welcome-model-signing-in` | the same note while signing in to ChatGPT inside the welcome |
| `welcome-model-signin-ready` | the connected state after signing in to ChatGPT |
| `welcome-model-declined`, `welcome-model-renewed`, `welcome-model-refused` | the other sign-in endings inside the welcome |
| `welcome-paste-waiting`, `welcome-paste-written` | the waiting page’s folded paste, with an empty or supplied secret address |
| `welcome-paste-too-long`, `welcome-paste-address`, `welcome-paste-state` | the length and address refusals, ready for a fresh paste |
| `welcome-paste-sending`, `welcome-paste-finishing` | the finish request in flight, then accepted |
| `welcome-paste-too-many`, `welcome-paste-ended`, `welcome-paste-expired` | the restart endings, with no paste field |
| `welcome-paste-lost`, `welcome-paste-unsent` | the address cleared after a lost or unsent finish request |
| `welcome-ready-starting` | a usable subscription's trial at Starts: its first step ticked, Claude Code starts on this machine under way |
| `welcome-ready-answering` | the trial past Starting: its first two steps ticked, Claude answers under way |
| `welcome-ready-answered` | an answered trial: ticked, answered in N s, no Each step |
| `welcome-ready-stopped-starting` | a trial whose run ended before starting: stopped at its second step, the run's why, Try again; Ready and Create stay usable |
| `welcome-ready-stopped-answering` | a trial whose run ended before answering: stopped at its third step, the run's why and Try again |
| `welcome-ready-waiting` | a running trial with a wait that won't settle in a moment: stopped at its step, the wait's words, no Try again until it ends |
| `welcome-ready-waited` | a trial ended after a minute: the last wait's words, or no answer within a minute; Try again |
| `welcome-ready-plan` | a plan's trial: Signed in to your PLAN plan ticked, then PLAN answers under way |
| `welcome-ready-key` | a key's trial: Key added ticked, then PLAN answers under way |
| `welcome-ready-checking` | Ready from its first frame, the declaration or host check under way, Model's line at its first step, its bar and Each step |
| `welcome-ready-copies` | Ready offers Widget and names the two repositories its act will copy into the project |
| `welcome-ready-none` | Ready with neither forge nor model, the pane's two lines, their inline connect links and the offered playground |
| `welcome-ready-not-yet` | Almost ready, the settled check without a usable login, Model's line stopped at its first step, Try again, Sign in again and Each step |
| `welcome-ready-signing` | Ready while ChatGPT signs in, Model's line in the pane as it is |
| `welcome-ready-under-way` | Ready while Forgejo sets up, Forge before Model, Each step and the note below the form |
| `welcome-ready-adopted` | Ready with Forge and Model connected, the project form and Back to Model |
| `forge-setup-failed` | the failed setup's reason and neutral retry act |
| `forges-github-install` | GitHub's Forges row before installation, with its primary Install act |
| `forge-admin-details` | a managed Forgejo's admin Details, with its first-repository fieldset and legend |
| `connect-forge-choose` | the shared Forgejo and GitHub cards, with Your Forgejo below |
| `connect-forge-connected` | shared cards with the connected Forgejo marked and GitHub primary |
| `connect-forge-yours-admin` | Your Forgejo’s admin form, with the other form absent |
| `connect-forge-yours-tokens` | Your Forgejo’s bot tokens, with the admin form absent |
| `adoption` | the adoption form, empty |
| `adoption-filled` | the adoption form, filled in |
| `adoption-refused` | the adoption form after a refusal, its alert |
| `sheet-adopt` | Add repositories with a long list, search, count and disabled project rows |
| `sheet-searching` | the sheet searching with a pick retained |
| `sheet-none` | the completed search with no match and Not listed opened |
| `sheet-few` | a small completed list, no search or count, More open |
| `sheet-empty` | the sheet with no reachable repository |
| `sheet-copy` | Copy public, one address per line |
| `sheet-copy-lines` | invalid copy lines and their repairs |
| `sheet-empty-source` | Make empty, with organization, name and the destination forge |
| `sheet-from-home` | `home-add-repository`'s deliveries, To begin's "Adopt another…" pressed: Add repositories over the project's Repositories, as that entry opens it for a project with no repository (`../forge/family.md`, 3) |
| `sheet-from-chats` | `chats-no-repository`'s deliveries, its band's "Adopt a repository" pressed: Add repositories over Repositories, as a project with no repository opens it (`../forge/family.md`, 3) |
| `repositories-making` | copies under way and failed, the forge’s reason, Copy again and Dismiss |
| `chats-reference` | the reference world's chats |
| `chat-reference-T1` | the reference world's chat T1 |
| `chat-solo-T1` | the solo world's chat T1, closed: no composer and no chooser |
| `chat-solo-T6` | the solo world's chat T6 |
| `chat-held-task` | the solo world's chat T10, its proposal's reason saying a task is held |
| `chat-first-run-2-T1` | the first run's chat T1 at step 2, its change running |
| `chat-first-run-3-T1` | the first run's chat T1 at step 3, its change landed |
| `chat-first-run-4-T4` | the first run's chat T4 at step 4, waiting at rest for the person’s words |
| `playground-resetting` | the first run's Chats while the playground resets: "Leaving the playground…", and no playground's start |
| `chats-first-run-5` | Chats after the first plan connects |
| `inbox-reference` | the reference world's *Needs you* |
| `accounts-reference` | the reference world's accounts |
| `accounts-first-run-1` | the first run's accounts at step 1: the playground's account |
| `signed-out` | signed out: the top bar's mark alone, and the column "Signed out" with the printed link's words (`look.md`, *Outside a project*) |
| `out-of-date` | a bundle older than the engine: the column "Out of date" |
| `no-project` | signed in, `me` naming no project, on a deployment with no welcome: the top bar with the person and "Sign out", and the column "No project yet" with the form "New project" (`../pages/README.md`, *With no project yet*) |
| `approve-typing` | the approval page at `/terminal`, no code: "Sign in a terminal?", the field holding a code typed in lower case with a space, `wdjb mjht`, and Check (`../pages/README.md`, *The approval page*) |
| `approve-unread` | the same field after Check with a code that doesn't read, `WDJB-MJH`: its alert |
| `approve-asking` | `/terminal?code=WDJB-MJHT` read waiting, asked two minutes before: the code, "Asked 2 min ago", the warning, Approve and Refuse |
| `approve-approved` | the same approved, waiting for the terminal's take: "Approved" and the code |
| `approve-signed-in` | the same taken: "Signed in. You can close this tab." |
| `approve-refused` | the same refused |
| `approve-expired` | the same read expired |
| `approve-used` | the same decided twice: Approve answered used |
| `approve-unknown` | a code the engine doesn't know: "Unknown code" and "Type a code" |
| `approve-signed-out` | the approval page with the browser signed out: "Sign in first", the printed link first, then the page's address again |
| `approve-signed-out-google` | the same with Google's way given: "Sign in, and this page comes back with your terminal's code." and "Continue with Google", its `then` `/terminal?code=WDJB-MJHT` |
| `approve-busy` | a read answered busy: "temper couldn't answer just now" and Try again |
| `chats-empty` | the chats with none, outside the playground |
| `inbox-empty` | nothing needs the person |
| `team` | hand-made deliveries after the whole story's `team/team.html`: two people, both owners of the project, the person reading with two questions and a review waiting, "Three things wait on you.", and Sam with a question waiting on one of Sam's chats, "One question waits on Sam."; "Two of you."; "owners of shop" at People's head's right, and each box "Owner"; a service member left out |
| `team-alone` | one person, the project's owner, nothing waiting: "Just you.", People's head its title alone, and one box, "Owner" and "Nothing waits on you." |
| `night-team` | `team` with the night look on |
| `team-role` | hand-made deliveries after the whole story's `team.html`: one person, the project's owner, nothing waiting; a standing role, Prover, its task 52, which they started on 2 November; the project's `findings` document with two areas, `core`, "The protocol, its state machine and its document", owned by the reader, and `tests`, "Tests, fuzzing and the tiers", owned by task 52, and kind 15 naming task 52; the lede "Just you." then "Each area has one owner. Prover hands each finding and red to its area's owner. An owner that can't decide asks you, as How work runs says."; "Areas" with "+ Hand an area to a role", `core`'s row with "You", "Nothing waits on you." and "–", `tests`' row with Prover's chip, current, its Now's first line and "21 of 40", and "Anything else", "What fits none of these", Prover, its Now's first line and "in Prover's"; Who routes, "Prover reads what comes in against these words and its rules, and hands each to its owner." and "Change by proposal"; no Standing roles; Prover's card: "A standing role · owns tests, and Anything else, since 2 November · started by you", "Talk to it", "Pause" and "Give the area back"; "Its Now · written at 02:41" and its four lines, each a list item; "Its rules · change by proposal" and its charter's "Asked to"; "Wakes on": "News from the forge, on 2 topics", "every 30 minutes", a timer, "Mondays 06:00", and "Task 61's result", "at once"; "Sessions this week": one with a run live, "Sun 01:52 · running", and two ended, "Sat 16:20 · 22 min" with its `Stood` words and "Fri 23:05 · 31 min" with none, each with its cost, and the line under them; "Its weekly report · written Mon 06:00" and its lines; then People, one box, "Owner · core"; and two follows of the project's: one Prover owns, on Prover's card under "Follows" after its Now, its line, "Follows acme/upstream · 2 behind · check passes at 9c04d7e1a2b3", and its two entries, one "held by its owner: waits for the handoff fix", one "waits for change 812"; one the person reading owns, its line on their box, "Follows acme/tools · caught up · no check yet" (`../pages/team.md`, 1, *Follows*) |
| `night-team-role` | `team-role` with the night look on |
| `team-hand` | `team-role` with "+ Hand an area to a role" pressed: the sheet "Hand an area to a role", `core` chosen, then `tests` and "A new area"; Prover chosen, then "Prover, from the stock" and "Porter, from the stock" with their words, and "Write one with Ask temper"; "Not now" and "Hand it over" |
| `role-talk` | Prover's page, task 52, standing, with the reader's talk open: their words "From now on, reproduce a flake twice before routing it.", Sent, and its reply as its run's turn, "Add it to my rules?"; the composer named "Talk", its note and the terminal's command "Or in a terminal: temper ctl DIR role talk 52 TEXT --key KEY"; "Leave" among the taskbar's acts |
| `chat-closing-below` | a chat at rest with one task below it: a singular count and the hairline |
| `chat-closing-proposal` | a chat at rest with one proposal: a singular count and the hairline |
| `task-releasing-uncertain` | a release with an uncertain effect and its two choices |
| `chat-closing-work` | a run, two delegates with opposite fates, and two proposals: what closing changes |
| `chat-closing-failed` | the same close refused: the sheet keeps its choices and says why |
| `chat-closed` | the requester closed it: one end line at 21:05, with a quiet bar |
| `chat-paused` | a chat its person paused: "✦ Paused" above the composer, Resume and Close in the bar, and the hint "Sending resumes this chat." |
| `chat-closing-start-over` | an Ask temper chat's Start over confirmation |
| `task-closing` | an agent's task Close confirmation, with a task below it |
| `task-paused` | an agent's task its person paused: "Paused", Resume and Close, and no own held card |
| `proposal-accepting` | a proposal's Accept confirmation |
| `proposal-rejecting` | a proposal's Reject confirmation with its reason |
| `chat-closing` | `chat-solo-T6` with Close pressed: its confirmation |
| `chat-rejecting` | `chat-solo-T10` with Reject pressed: the reason's box |
| `task-held` | an agent's task's plan with a held task, Release offered |
| `task-releasing` | `task-held` with Release pressed: its confirmation |
| `connect-model-choose` | shared plan cards and key rows, with the owner’s quiet holder switch |
| `connect-model-connected` | a connected plan marked on its card, with the first unconnected plan primary |
| `connect-model-key-details` | a key's secret field and Add key |
| `connect-model-program-details` | a program’s panel, command, Copy, login name and I’ve signed in |
| `connect-model-signing-in` | the waiting note, "Open it again" and folded paste |
| `connect-model-declined` | plan usage declined, with another sign-in and a key way |
| `connect-model-refused` | another holder's account refused, with another sign-in |
| `connect-model-key-lost` | a key request's answer lost, its alert asking for the value again |
| `task-reference-T28` | the reference world's change T28, queued 2nd after T26 |
| `task-reference-T24` | the reference world's change T24, landed, its ended task T25 |
| `task-reference-T12` | the reference world's goal T12, its tasks with its ended delegates T13, T14 and T24 folded |
| `task-solo-T9` | the solo world's agent T9, its run's committed calls and words within the shipped window |

All `welcome-*` states keep the welcome's bar: temper's mark, Steps and
its link region alone, no person, Sign out or sign-out alert. Only the
Welcome step draws temper box's paragraph, under the cards' note, or
under the cards where there is no note (`../pages/welcome.md`, 1;
`welcome-top`). On Model and Ready, "Open Forgejo ↗", when an address is
known, is a quiet link at the forge text's 24 px indent, muted, 13.5 px
(`forgejo-link`); with no address there is no link.

## 2. Open questions

None.
