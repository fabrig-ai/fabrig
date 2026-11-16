# using-temper: execution

2026-11-16. How [using-temper.md](using-temper.md) gets built. It designs nothing: each milestone writes its own design from its draft as it runs, and lands whole, with its own tests. Each is a goal, and none starts until the user approves it. Plans use the focused and fuzzy suites alone; walks and render checks come after landing. A stored format or a wire that a milestone changes moves its version and brings its patch or its bridge (`redeploys.md`, sections 3 and 4).

## 1. The milestones

| Milestone | Draft parts | Needs | Its walk | State |
|---|---|---|---|---|
| U1, readiness: a card on a project's Home after adoption saying whether a first change can pass, from adoption's reads and main's last CI run, each line with its act | `using-temper.md`, 5.2 step 3; 11, G2 | none | a repository adopted on GitHub with protection that refuses temper's merge, then with none | not started |
| U2, "Teach temper this repository": a stock first change, offered on Home, whose agent writes the delivery check and `AGENTS.md`'s part from the repository's CI and docs | `using-temper.md`, 5.2 step 4; 11, G2 | U1 | its pull request on a real repository, reviewed at Pair | not started |
| U3, a new project's foundation: the drafter's words for a project with no code, and its goal's first change replacing the starter workflow with the real build and tests | `using-temper.md`, 5.3; 11, G3 | none | an empty Forgejo repository to its foundation's goal landed | not started |
| U4, reached away from the machine: the forge asked for the person's review when a change waits for it, and a sign-off's push | `using-temper.md`, 5.2 step 7; 11, G5 | none | an approval reaching a phone through GitHub's app, and through the push | not started |
| U5, the newcomer's reading: temper's README, the install script and the help's GitHub walk made true | `using-temper.md`, 8; 11, G8 | none | the help's walk from a fresh box | not started |
| U6, the second door: a help page that is the words a person gives their coding agent | `using-temper.md`, 4; 11, G7 | none | a coding agent handing a goal to a box from its page alone | not started |
| U7, the climb's offers: How work runs' suggestions over counts, and the chat's "Make it a draft?" | `using-temper.md`, 3.4; 11, G9 | none | a run of approvals given unchanged, then the offer | not started |
| J1, J2, the journeys: J1 from the welcome to a first change landed after its review; J2 from an empty repository to its foundation's goal started | `using-temper.md`, 12 | U1 to U4 | the user takes their frames | not started |

## 2. Not here

Each needs a draft of its own before a goal can carry it out, in its own field:

- **A project's gates and tiers as settings** (`using-temper.md`, 11, G4), out of the deployment's document.
- **Placement by what a project needs** (`using-temper.md`, 11, G4).
- **From a box to a machine that stays on, and the terminal and `temper ctl` beyond loopback** (`using-temper.md`, 11, G5 and G7).
- **A large draft as a milestone in the client** (`using-temper.md`, 11, G6), after `home-for-a-person.execution.md`'s M3.
- **A change's cost said before it starts** (`using-temper.md`, 11, G10).

## 3. Order

- **Five at once:** U1, U3, U4, U5 and U6 touch different parts: Home's adoption card, the drafter's words, the forge's review request and the push, the help and README, and a help page.
- **Then U2** once U1 has landed, since its offer is a line of U1's card; **and U7**, which touches How work runs and the chat's charter.
- **Then J1 and J2,** recorded once U1 to U4 have landed.
- **Seams:** U2 and U3 both write stock charters' words, the change's and the drafter's, so the second merges. U1 and U7 both add to pages, Home and How work runs, which `home-for-a-person.execution.md`'s H1 and M3 change too: whichever lands second merges.
- **Priority mark:** the UI with looks is one of v0.1.0's critical pieces; U1, U2, U4 and J1 carry its mark, since they are the first day's pages.

## 4. When it is done

Every milestone landed, its walk taken after landing, the gaps of section 2 each in a draft of its own, and the draft's status holding no lag, this file goes with its draft (`DESIGN.md`, section 3).
