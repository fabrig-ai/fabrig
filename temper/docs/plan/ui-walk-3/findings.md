# Third-walk findings

Client source: `fa5b518db6093a96a90d3507a4e18ee1ed27eea6`.
Evidence is on **debian-32gb-hel1-14**, under
`~/src/fabrig/ui-walk-3/walk/`; the [report](report.md) selects the final
pairs and identifies every exploratory run. No client code was changed.

| id | where (step, page, state, width) | seen | the mock or rule says | kind | size | owner | state |
|---|---|---|---|---|---|---|---|
| W3 | 5, Configure, project Needs you, Home and acceptance, 1440 and 420 | The exact fresh-box prompt produces a live proposal. The inbox row retains `project: 0` and now also has `projects: [1]`; the project projection and count use that membership. Accept changes the ways and clears the count. | Project settings proposals appear in each affected project's inbox, count and Home; acceptance updates the original chat card. | other | large | fabrig/ui | fixed, seen at both widths: exact cue on a fresh revision-1 box; `final-1440/05-configure-proposal-accepted-1440-pair.png`, `final-420-v3/05-configure-proposal-accepted-420-pair.png`; inbox JSON before and after acceptance. |
| W6 | 3, Home after a real landing and 31 minutes away, 1440 and the same live box at 420 | Stock sitting and personal ChatGPT plan, without a funding edit. Separate unfunded box uses a shortened sitting and ctl period cut. | A funded return has sentence, lede and byline; a refused run leaves the computed landing sentence without brief prose, byline or error. | other | large | fabrig/ui | fixed, seen: `final-1440/03-home-brief-1440-pair.png` and the same live box at 420 after 1861.756 seconds; stock sitting and personal plan, no funding step. Separate unfunded rows at both widths have the minute-old computed landing sentence and no brief/byline/error. |
| N1 | 5, Accept proposal sheet; 4, Stop goal sheet, 1440 and 420 | The confirmation is pinned to the viewport's upper-left corner. At 1440 Accept is x=0, y=0, width=460, height=214.09375; computed margin is 0px, position fixed, inset 0px. At 420 it is x=0, y=0, width=388, height=242.5625. | `temper/docs/design/client/README.md`, 3.6, The sheet: the mock's modal sheet, narrower. The mock's start sheet is centred. No decision moves a confirmation to the corner. | look | small | fix-look | |
| N2 | 2, Settings, How work runs, Status, 1440 and 420 | The disabled “The brief on Home” select says “Off”; the row describes what temper does today. The view hardcodes `SettingsOff`. | D11 disables these controls while describing current behaviour. The stock status agent is enabled and may write a brief; “Off” does not describe that behaviour. | word | small | fix-look | |
| N3 | 4, Chat/Draft after working-note restoration, 420 | Start goal… is offered on the first document turn, then absent after the working-note round trip although the restored document is visible. No error explains it. Two earlier fresh phone boxes reached the sheet. | The same chat still has a nonempty document, enabled drafts and the home repository that made the first offer valid (`temper/docs/design/client/README.md`, 3.5). Its goal entry must stay available. | link | small | fix-flow | |

N1 reproduction: fresh fake box, finish first run, open Configure temper before
changing settings, send “Show drafts and goals for this project.” Open the
project's Needs you and press Accept. The sheet's Not yet and Accept work, but
the sheet starts at the screen's top-left. The same placement occurs when
stopping the draft's goal. Adjacent client JSON records its rectangle and
computed styles. This is a look fault, not a failed proposal decision.

N2 reproduction: finish first run, open Settings → How work runs and read the
Status section. Compare its “Off” with the funded Home return in the report.
The decision to keep the select disabled is retained; the finding is its word.

The rename form, focus return, repeated A4 sequence, Owner menu detail, account
trial and new-tab looks belong to `additions`. Their existing ids remain
reserved. This walk's seventh step still captures rename, Escape, Sign out,
reload and return; it makes no claim about the additions' repeated checks.

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
