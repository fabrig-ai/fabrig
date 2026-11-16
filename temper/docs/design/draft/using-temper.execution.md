# using-temper: execution

2026-11-16. How [using-temper.md](using-temper.md) gets built. It designs nothing: each milestone writes its own design from its draft as it runs, and lands whole, with its own tests. Each is a goal, and none starts until the user approves it. Plans use the focused and fuzzy suites alone; walks and render checks come after landing. A wire a milestone changes moves its version and brings its bridge, or is declared breaking (`redeploys.md`, section 4). A registered store family's schema moves no version and brings no patch (jig's `protocol/store/layouts.md`); any other stored format follows `redeploys.md`, section 3.

## 1. The milestones

| Milestone | Draft parts | Needs | Its walk | For | State |
|---|---|---|---|---|---|
| U6, the second door from another machine: `temper ctl` and the terminal at a deployment's public `https://` address over TLS with the machine's roots; the terminal's credential and kept requests per deployment's origin, never sent to another; ctl's sign-in by the person's browser, as the terminal's already is; the approval between hosts: who may ask and how often, so sustained anonymous asks can't keep the person out, the code typed on the page, what the page attests told apart from what the client says, and whether approving needs more than the person's credential; a client of another version refused naming the deployment's build; the help's walk for the driver, but for milestones, and the line for a person's `AGENTS.md` | `using-temper.md`, section 16; 11, G7 | none | from a laptop, `temper ctl` signed in to dev.fabrig.ai by the browser, a pushed branch opened as a change and landed | v0.1.0 | approved by the user, 11-16 |
| U5, the newcomer's reading: temper's README, the install script and the help's GitHub walk made true; the help's index saying the ladder, the five verbs and the rules of being simple at scale (the site's words are the user's) | `using-temper.md`, 3, 7, 8; 11, G8 | none | the help's walk from a fresh box | v0.1.0 | approved by the user, 11-16, among "the easy picks" |
| U8, the repository's `.temper/`: `.temper/AGENTS.md` in a run's prompt after the root's guide, as temper's and winning where they differ; a reviewer's guides read from the branch the change lands into; the stock ways' rule by path asking the person for a change to `.temper/`; `temper help repository` saying what `.temper/` holds, each file with an example, and that the rule is no security boundary | `using-temper.md`, 15.2 to 15.4; 11, G11 | none | a run's prompt on a repository with both guides; a change to `.temper/pre-pr` under Agents review reaching the person | v0.1.0 | not started |
| U1, readiness: a card on a project's Home after adoption saying whether a first change can pass, from adoption's reads and warnings, main's last CI run and what `.temper/` holds, each line with its act; "unknown" for a read that failed, a CI never run or a file not fetched; the workers' toolchain said to be unknown until G4 | `using-temper.md`, 5.2 step 3; 11, G2 | U8, for its `.temper/` line's page | a repository adopted on GitHub with protection that refuses temper's merge, then with none | v0.1.0 | not started |
| U4, reached away from the machine: the forge asked for the person's review when a change waits for it, and a sign-off's push | `using-temper.md`, 5.2 step 7; 11, G5 | none | an approval reaching a phone through GitHub's app, and through the push | v0.1.0 | not started |
| U3, a new project's foundation: the drafter's words for a project with no code, and its goal's first change replacing the starter workflow with the real build and tests, with `AGENTS.md` and `.temper/pre-pr`, approved by the person since it changes `.temper/`, the person a reviewer the forge counts | `using-temper.md`, 5.3; 11, G3 | none | an empty Forgejo repository to its foundation's goal landed | v0.1.0 | not started |
| J1, J2, the journeys: J1 from the welcome to a first change landed after its review; J2 from an empty repository to its foundation's goal started | `using-temper.md`, 12 | J1: U1 and U4; J2: U3 | the user takes their frames | v0.1.0 | not started |
| U2, "Teach temper this repository": a stock first change, offered on Home, whose agent writes `.temper/pre-pr` and `.temper/AGENTS.md` from the repository's CI and docs, and nothing outside `.temper/` | `using-temper.md`, 5.2 step 4; 11, G2 | U1, U8 | its pull request on a real repository, reviewed at Pair | after v0.1.0 | not started |
| U7, the climb's offers: How work runs' suggestions over counts, the chat's "Make it a draft?", and "Ask you again for changes like this", a rule by path asking again; not "Start it as a milestone", which is G6's | `using-temper.md`, 3.4; 11, G9 | none | a run of approvals given unchanged, then the offer | after v0.1.0 | not started |

## 2. Not here

Each needs a draft of its own before a goal can carry it out, in its own field:

- **Where a project's gates and tiers live** (`using-temper.md`, 15.2; 11, G4): in `.temper/`, read from the branch a change lands into, as this draft's default has it, or in the store, as the kinds that `../protocol/configuration/engine.md` plans; beside `config.md`, which moves the rest of the deployment's document. It settles how a landing cuts over when a change to them lands: a train holding one beside other changes, and a proof made before it and still queued (`../domain/landing.md`).
- **Placement by what a project needs** (`using-temper.md`, 11, G4), the need said in `.temper/` (15.2).
- **From a box to a machine that stays on** (`using-temper.md`, 11, G5).
- **A large draft as a milestone in the client** (`using-temper.md`, 11, G6), with section 3.4's "Start it as a milestone", after `home-for-a-person.execution.md`'s M3; and the driver's walk for a milestone from an execution file, with M2's `temper ctl milestone`.
- **A change's cost said before it starts** (`using-temper.md`, 11, G10).

## 3. Order

- **The easy picks first, at once:** U6, the user's own pick, and U5, small and apart from it. The user approved them on 2026-11-16: "we could start with the easy picks, like temper (ctl) working against a remote repo".
- **Then, on the user's word, apart from each other:** U8, U4 and U3; U1 once U8 has landed, since its `.temper/` line opens U8's page.
- **Then J1 and J2,** recorded once their milestones have landed.
- **After v0.1.0:** U2, once U1 and U8 have landed, since its offer is a line of U1's card and it writes U8's guide; and U7.
- **Seams:** U6 and U5 both write help pages and the README's words, so the second merges. U2 and U3 both write stock charters' words, the change's and the drafter's, so the second merges. U1 and U7 both add to pages, Home and How work runs, which `home-for-a-person.execution.md`'s M3 changes too: whichever lands second merges.
- **Priority mark:** the UI with looks is one of v0.1.0's critical pieces; U1, U4 and J1 carry its mark, since they are the first day's pages.

## 4. When it is done

Every milestone landed, its walk taken after landing, the gaps of section 2 each in a draft of its own, and the draft's status holding no lag, this file goes with its draft (`DESIGN.md`, section 3).
