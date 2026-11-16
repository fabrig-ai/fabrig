# home-for-a-person: execution

2026-11-15. How [home-for-a-person.md](home-for-a-person.md) and [milestones.md](milestones.md) get built. It designs nothing: each milestone writes its own design from its draft as it runs, and lands whole, with its own tests. Each is a goal, and the user approved them all on the mockups (2026-11-15), so each starts once what it needs has landed. Plans use the focused and fuzzy suites alone; walks and render checks come after landing. A stored format or a wire that a milestone changes moves its version and brings its patch or its bridge (`redeploys.md`, sections 3 and 4). Its look is `../client/web/home-for-a-person/`.

## 1. The milestones

| Milestone | Draft parts | Needs | Its walk | State |
|---|---|---|---|---|
| H1, Home's parts for a person: Needs you first; findings in the headline; severities counted and drawn; a finding's row; the cards' order; "One more" with its line; the why line of a finding the person holds, said once where every card has the same; Decided for you's shorter words and ticks; goals' rows with their bars, number words and cut titles | `home-for-a-person.md`, 3 | none | Home against `home.html` at 1366 and 390 px, by day and by night | not started |
| A1, agents write for a person: a goal's title as a short first line in the chat's `goal` and `change` calls and in `temper ctl goal start`'s help; one sentence and one line in every charter's `decided`; report headlines with no numbers | `home-for-a-person.md`, 4 | none | a goal started from a chat, its title and its headline on Home | not started |
| B1, the brief reads findings: the open findings in the status agent's words; a run when they change; its instructions | `home-for-a-person.md`, 6 | none | a brief that names findings | not started |
| N1, people by name: a brief's requester named, and the charters writing the name | `home-for-a-person.md`, 7 | none | a decision naming who asked | landed; its walk after landing not yet run |
| M1, jig's part of a milestone: a party making a delegate of a task it may act on; a procedure task ended by its maker's word once its delegates have ended; dependencies named at a task's start | `milestones.md`, 3 and 7 | none | none: no page draws it yet | not started |
| F1, the triager's groups: the line form's `Findings:` and `Group:` lines; the triager's charter grouping and asking; a group's card on Home and on Needs you, counted in the headline and the head | `home-for-a-person.md`, 5 | H1 | a triager role's questions on Home | not started |
| M2, temper's milestone: its template, label and identity; its goals asking its maker; `temper ctl milestone`, and `goal start` with `--milestone`, `--after` and `--no-deadline`; the reads, and the status agent's input | `milestones.md`, 3, 6, 7 and 8 | M1 | a milestone made through `temper ctl`, printed by `milestone show` | not started |
| F2, why a group reached you: the reason kind for starting work on findings, its class in the stock, its why line and its Stop asking | `home-for-a-person.md`, 5 | F1 | a group's why line and its offer | not started |
| M3, the pages' milestones: goals that wait; Home's grouped Goals; Work's milestone card, placed recursively; a milestone's page; a goal's "in NAME"; Needs you's from line and filter | `milestones.md`, 4 and 5 | M2, H1 | Home, Work, a milestone's page and Needs you against the mockups | not started |

## 2. Order

- **Five at once:** H1, A1, B1, N1 and M1 touch different parts: the client's Home and its words, the stock charters, the status agent's words, a brief's people line, and jig's tasks and admission.
- **Then F1** once H1 has landed, since it draws in H1's cards and rows; **and M2** once M1 has.
- **Then F2** after F1, **and M3** after M2 and H1.
- **Seams:** A1 and F1 both change stock charters, the triager's among them, so whichever lands second merges the other's words. F1 and M3 both change Home's view, one its cards and the other its Goals, so the second merges. B1 and M2 both feed the status agent's words, B1 findings and M2 milestones, so the second merges.
- **Priority mark:** the UI with looks is one of v0.1.0's critical pieces, and every plan here carries its mark: each is part of Home, Work or Needs you's look, the milestones' engine parts included, since the pages draw them.
- **What a project sets, the person's:** F1's groups appear only where a triager role stands (kind 15, `../domain/areas.md`, section 6); B1's brief only where the status agent's charter is funded (`../domain/forge/attention.md`, section 3). Neither is a plan.

## 3. When it is done

Every milestone landed, its walk taken after landing, and the drafts' statuses holding no lag, this file goes with its drafts (`DESIGN.md`, section 3).
