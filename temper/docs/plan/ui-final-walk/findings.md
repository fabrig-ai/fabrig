# First-walk findings

The client walked is `a040302420`; only the fake's additional scripts and the
walk driver changed. Widths are CSS pixels. Evidence is on
`debian-16gb-hel1-8`, under `~/src/fabrig/ui-final-walk/walk/`.
The state column is filled in by each finding’s fix task.

| id | where (step, page, state, width) | seen | the mock or rule says | kind | size | owner | state |
|---|---|---|---|---|---|---|---|
| W1 | 2, Settings, feature switches after a fast first run, 1440 | All four feature inputs remained disabled for over 120 seconds, including after Home → Settings. No error explained them. Reproduced on manual boxes 4 and 5; applying project-ways through ctl made them live. | The four switches are available to an owner; they disable together while a change is being sent. No change had been sent. | stale-read | small | fix-flow | Fixed in this commit; red first: `a_transient_settings_read_backs_off_and_recovers_without_a_revision_change` (no retry deadline after a transient read failure) and `leaving_the_settings_scope_cancels_its_read_retry`. |
| W2 | 1, Home, To begin, 1440 and 420 | “Tell temper how you work” says “temper runs its default way of working until you change it.” The repository row says checked changes land through the merge queue. | The mock explains proposing how work runs and accepting it before anything changes, and says the starter CI ran on main and passed. These copy differences are not in the carried decisions. Preserve truthful claims when reconciling the words. | word | small | fix-look | Fixed in `0430ee52b1`; red first: `tests::home::home_ready_headline_and_done_steps_hold_in_both_shells`. Repository readiness makes no unsupported CI claim. |
| W3 | 5, Needs you and its count, settings proposal, 1440 and 420 | A real project-ways proposal waits for project 1's owners in ctl, but its inbox row and proposal have project 0. The project's browser inbox says “Nothing needs you” and its rail has no count. Gallery shows the card instead. | The mock's configuring proposal appears in Needs you and its count. The rule says project owners decide proposals for their project's settings. The project-filtered rail must provide a way to find it. | other | large | fabrig/ui | |
| W4 | 4, draft pane section headings, 1440 and 420 | Section headings use 17px/25.5px (line-height 1.5). | The mock's document section headings use 17px/28.9px (inherited line-height 1.7). No carried decision changes this rhythm. | look | small | fix-look | Fixed in `ea27ed419d`; red first: `draft_section_headings_keep_the_document_line_height` (1.5 instead of 1.7). |
| W5 | 2, Configure temper introduction, 420 | Composer has the hint below it; its textarea starts at y=919.8125 in a 1000px viewport. | Look request 727f3523ee / configure-chat / 01 hides the hint on the phone; its textarea starts at y=945. The client design currently says the hint stays, so reconcile that line with the reference rather than silently treating them as equal. | look | small | fix-look | Fixed in `99dc682b1f`; red first: `phone_session_hides_the_keyboard_hint` (phone hide rule absent). Reconciled the design with the look request. |
| W6 | 3, Home, return after over 31 minutes, 1440 and the same live state at 420 | A real landing remains in Changes, but Home returns to “Nothing needs you” with no brief, error or way to fund it. The project's whole period is reserved by the owner's pool; period left is 0. | D2 asks for the stock brief on a new sitting after something landed. The mock shows its brief. No carried decision explains a fresh project's funding preventing it silently. | other | large | fabrig/ui | |

| W7 | Addition, chat draft | A lower-numbered `work:` note displaces the document. | Draw the lowest-numbered non-`work:` note in the chat’s own scope; only working notes means no document. | other | small | fix-flow | Fixed in `05d2eca302` and this commit’s continuation past working-note pages; red first: `a_work_note_never_displaces_the_lowest_document_in_the_chats_scope` (selected note 1 instead of 12) and `a_chat_with_only_work_notes_has_no_document` (drew working memory); `the_chats_list_excludes_working_notes_from_documents` also failed on the working title; `a_page_of_working_notes_does_not_hide_a_later_document` failed because no continuation was requested. |

## W1 reproduction and evidence

The domain reproduction supplies a transient `Limit`, `Busy`, `Unreachable`
or a `SignedOut` read result while the session is live. The old code lost the
read, retained no kind-7 value and scheduled no retry; opening Settings did
not restore it without a changed settings line. The fix retries without a
revision change, with bounded backoff, and drops retries with their scope.
The original browser timing has not been replayed here; `walk-again` should
repeat the fast first-run path without the ctl fallback.

Use a fresh fake box. Complete Forgejo's empty `work/hello`, ChatGPT and Ready.
As soon as Home renders To begin, open Settings without an additional settling
pause. Observe all four disabled feature inputs. Waiting more than two minutes
and opening Home then Settings did not repair manual box 4. `settings show`
through ctl still reported revision 1. A ctl apply of project-ways at that
revision made the inputs live after reopening Settings.

`02-settings-switches-stalled-1440-pair.png` and its client JSON retain the
state. The four input `disabled` properties were true; no feature change was
in flight. Later driver runs wait for the repository adoption text and allow a
one-second Home settle. The driver retains a captured ctl fallback if the read
still stalls. This is an observation of a timing-dependent path, not a claim
that every fresh box fails. A final fast-path attempt on the corrected fake
script did not reproduce it; `stall-proof/` retains that attempt.

## W2 evidence

`01-home-to-begin-1440-pair.png` and `01-home-to-begin-420-pair.png` show the
words. Their adjacent JSON preserves the text. The intro heading typography
matches: 32px/40px, weight 500 at 1440; 25px/32.5px, weight 500 at 420. The
changed ready headline, white To begin card, Optional/Skip, lack of Connected,
project name and account line are carried departures, not findings.

## W3 reproduction

1. Complete the fresh fake welcome, enable the features and start a chat.
2. In a private ctl directory, sign in with a fresh link printed by that box.
3. Read `settings show --json` and its `newest` revision. Write a private file
   containing one `project-ways` item for project 1, setting `drafts`, `goals`,
   `changes` and `away` to true and `way` to 0.
4. Run `settings propose FILE "Try a settings proposal for the UI walk."
   --against REVISION --key walk-proposal`. It prints a proposal for project
   1's owners.
5. Read that proposal and the unfiltered inbox through ctl. Both retain
   `project: 0`. Open the browser's Needs you through its rail/tab, which filters
   to project 1: it says nothing needs you and offers no proposal.

`proposal-1440.json`, `inbox-proposal-1440.json` and their 420 counterparts
retain public ctl evidence. `05-proposal-missing-420-pair.png` shows the empty
project inbox; `05-proposal-*-pair.png` shows the gallery substitute. The replay
writes the same files in its own output directory.

This needs a decision about how settings proposals spanning deployment and
multiple projects belong in project inboxes and counts, then a consistent
projection/read rule. A visual card change alone cannot repair the missing
entry. The proposal in the reproduction touches only project 1, so the problem
is visible without a mixed-scope proposal.

## What was explained or corrected during the walk

- The fake forge reports `offer: not_offered`. Its missing Ready “Open Forgejo”
  link follows the page-facts condition in `temper/docs/design/client/README.md`,
  3.5; it is not a missing-link finding.
- An initial walk-specific goal script called wait with nothing pending,
  exhausted its retries and produced a real temper-owned held card at 420.
  That capture remains. The final script asks a question before waiting;
  replayed Pause, Resume and Draft return work. The early missing Draft link
  on the moving, failing goal was not retained as a client finding.
- Native mock Stop confirmation is accepted through CDP; the client uses its
  own Confirm sheet. The mock does not implement its other Settings tabs,
  new-project creation or Sign out. Those limits are named in the table.

## W4 and W5 measurements

W4 is in `replay-1440/04-draft-1440-pair.png` and the 420 draft pair. The
adjacent JSON records the mock's H2 and the client's corresponding Markdown
H3: both 17px and weight 600; the line heights are 28.9px and 25.5px. Their
640px document columns match at 1440 after accounting for the retained rail
(D7). The text, heading level and tint's section are scripted data.

W5 is `02-configure-look-request-420-pair.png`, with the reference and overlay
extracted from the specified commit. Both textareas are x=31, width=318,
height=26, 15px/23.25px. The client is 25.1875px higher because its hint is
visible. The introduction's heading is identical on both: x=16, y=77,
26px/32.5px. The reference has no Goals or Changes in its rail; the live
project's enabled features are data/decision D1, not a finding.

## W6 reproduction

1. Complete the fresh fake welcome using its personal ChatGPT plan and empty
   `work/hello`. Enable the features through Settings.
2. In the first chat send `Land a local hello status change.`. Follow Changes
   until `Record the local hello status change` is in Landed recently.
3. Open Home, then close the page, keeping the box running. Wait at least
   31 minutes by the wall clock. Reopen the same Home address.
4. Home says it was last looked at 32 minutes ago, but its headline is
   “Nothing needs you.” There is no brief, byline or error. Changes still says
   main moved 32 minutes ago.
5. Read the project through ctl. On this box the period budget was 8117760,
   reserved 8117760, left 0. The owner's pool retained the reservation.

`03-home-brief-1440-pair.png` and `03-home-brief-420-pair.png` capture the live
return; the latter is the same box and state resized through CDP.
`home-brief-project.json` retains the public balances. The two
`03-home-brief-gallery-*-pair.png` pairs show the intended brief as a fallback.
The process-tier status test passes, but its setup cuts the owner's role's
funding to leave period money for the deployment-started status run; the
person's path did not make that hidden adjustment.

The balances and the status-run admission path indicate a funding problem;
this is an inference, not a proven single cause. The remedy needs a decision
about the default allocation for stock background work and its account, and
how the person sees an admission failure. It is larger than a view or stale
read repair. No budget or policy was raised or changed to conceal it.
