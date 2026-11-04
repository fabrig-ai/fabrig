+++
id = "execution-lines"
part = "execution-lines"
kind = "docs"
needs = ["notes-and-example"]
files = [
  "temper/docs/design/draft/reference-case.execution.md",
  "temper/docs/design/draft/configuring.execution.md",
  "temper/docs/design/client/browser.status.md",
  "temper/docs/design/playground.status.md",
  "tests/boundary/outward.txt",
]
effort = "medium"
minutes = 30
+++

# The execution files say what, not who; six entries leave the inventory

## Goal

`reference-case.execution.md` and `configuring.execution.md` keep what each milestone delivers, its
order and dependencies, its acceptance and what is done, and lose who runs it and where. Two status
lines lose their name. The six files then leave `tests/boundary/outward.txt`. This is sections 1 and
4 of `temper/docs/plan/execution-own/inputs/request.md`, and its last bound.

Acceptance:

- Neither execution file holds a counted name (`notes/common.md`, The counted names), and this
  prints nothing:

  ```sh
  grep -nE "trunk|plan of its own|temper/docs/plan|separate sessions|retire|Old code|[Oo]ld branches|in place of|one pipeline|X5" \
    temper/docs/design/draft/reference-case.execution.md temper/docs/design/draft/configuring.execution.md
  ```

- `\bcut\b` is left only in M3's table row, heading and entry, as the reference's cut, and in P4's
  "cut into obligations".
- Every milestone and phase keeps its entry: Executes, Design, Builds, Configures, Tests, Walk and
  Done, as far as it has them. Every section number stays, section 2 of each file included.
- `tests/boundary/outward.txt` has no line for `temper/docs/design/client/browser.status.md`,
  `temper/docs/design/playground.status.md`, or `configuring.execution.md`, `dashboards.md`,
  `notes.md` and `reference-case.execution.md` in `temper/docs/design/draft/`, and no `# Raised:`
  line left without its entry. Its other entries are unchanged, and no count rises.
- The outward check is green, and `tools/citations/citations.py --all` is at 0.

## Inputs

- `temper/docs/plan/execution-own/notes/common.md`: its rules hold, and its departures from the
  implementer's text.
- `DESIGN.md`, 3 and 4.

## Owns and may change

- **Owns:** `temper/docs/design/draft/reference-case.execution.md`,
  `temper/docs/design/draft/configuring.execution.md` and `tests/boundary/outward.txt`.
- **One line each, as step 3 says:** `temper/docs/design/client/browser.status.md` and
  `temper/docs/design/playground.status.md`. Another draft, `playground-browser`, is changing both
  files. Touch nothing else in them.
- **Who reads section 2:** `temper/docs/plan/reference-walks/plan.toml` names
  `reference-case.execution.md, 2` as its `happy_path`, and `configuring.execution.md` cites it.
  So section 2 stays, with its walk. Other citers rely on `reference-case.execution.md`'s M3, M10
  and section 6, and on the `notes.md` sections it cites, all of which stay.
- **Pre-approved:**
  - if a status line reads otherwise on your base, because `playground-browser` has landed, replace
    the name in the sentence as it stands; if the file holds no name, leave it alone and say so;
  - if a sentence of an execution file holds a lab word Steps does not list, apply
    `notes/common.md` and name it in the report.

## Steps

### 1. `reference-case.execution.md`

- **Section 1, second bullet:** "**Each writes its own design** (section 2): its input is its entry
  here and the draft sections it cites. Shared design is kept to the least: no milestone designs
  ahead for another, and where several touch one section, each adds its own part (section 4)."
- **Section 1, third bullet:** "sends its findings back as fixes".
- **Section 2,** the heading stays and its body becomes:

  ```
  1. **Its input:** its entry in section 3 and the draft sections it cites.
  2. **Design and build together.** Its designers, one per family, write
     the family's design: the sections the entry's Design names, each
     document's status with a Not yet for what is still to build, its
     Later, and the entries they close in the drafts' statuses. The work
     that cites it follows, family by family. Implementers never write the
     design. A section once written binds the work that cites it: later
     work adds compatible detail or its own part, and an incompatible
     change is a numbered revision.
  3. **Landed whole:** checked whole, it lands on main with its statuses
     current.
  4. **Done** when it has landed on main. A milestone that needs it may
     start then.
  5. **Its walk, where there is something to walk:** a rung of the
     ladder, which blocks nothing. Whether what landed makes a story
     walkable through `temper ctl` is judged as it lands; if not yet, the
     walk folds into a later milestone's. A brief is in a rung's form
     (`v0.1.0.md`, section 3): the case's steps the milestone makes
     possible, with the case's names, on the case's backends; a step that
     needs a later milestone is walked in its plainest form or left out,
     and the brief says which. Its findings go back as fixes. Walks grow,
     each brief extending the earlier ones, so M10's is the case.
  ```

  Delete "Its designers, its implementers and its walker are separate sessions." and the whole "Old
  branches, for reference only" paragraph.
- **Section 3's table:** a done cell holds the date and the commit, and what follows them goes.
  - M0: `Ground: the playground's reset closed out; designers write the design`, its state
    `done 2026-11-02`. The row also has one cell too few today: give it its empty sixth, so that
    Executes, Needs and Walks are empty.
  - M1 to M3, M5, M6, M9 stay. M4 and M8 stop after their commit's closing parenthesis.
  - M7 stays. M10: `done 2026-11-03 (`4c739315c4`)`.
- **The paragraph under the table:** end at "Configures, the settings of
  `reference-code-production.md`, section 14 it adds."
- **Every `Old code` bullet goes,** M1, M2, M3, M5, M7, M8, M9 and M10.
- **M0:** delete "(the user's X5)". The second bullet: "**Designers write the design** (the user,
  2026-11-02): a milestone's designers write their family's design with the words of its changes, as
  section 2 has it, the project's design rules (`DESIGN.md`) among their documents." The last:
  "**Done** 2026-11-02."
- **M3:** "moved here from M1 (the user, 2026-11-02)"; "M4's sub-agents reachable from temper: the
  box's smith profile ..."; "**A charter's documents** are built by `configuring.execution.md`'s P1,
  not here: ..."; "M4's walk folds in here: a Claude Code chat ...".
- **M7:** "which M7a found missing".
- **Section 4:** "**Up to five at once,** in wave 2."
- **Section 6, After the case:** "fabrig's own drafts carried out by temper, on fabrig's forge,
  GitHub, a rung behind (`v0.1.0.md`, 2.2 and section 6)." Its last sentence stays.

### 2. `configuring.execution.md`

- **Section 1, first bullet:** "**Five phases,** P1 to P5 (section 3), set with the user on 2026-11-02
  so that no two designers write one section at once: ...". The rest stays.
- **Section 1, third bullet:** "P2 and P3 after M3: P2 ...; P3 the configuring agent. They build on
  M3's design."
- **Section 2,** the heading stays and its body becomes: "As a milestone of the reference runs
  (`reference-case.execution.md`, section 2): its entry here its input, this file and
  `configuring.md` its sources; its design and its build together; landed whole; its walk, where
  there is one, a pass of QA through `temper ctl` that blocks nothing."
- **Section 3's table:** P1 and P4 stop after their commit's closing parenthesis.
- **Every `Old code` bullet goes,** P1, P2 and P3.
- **P1:** "**Done** when it lands."
- **P2:** delete the "**Starts early,**" bullet.
- **P4, Builds:** "the golden, the check and the judge; the walk's brief."

### 3. The status lines

The same words in both: `fab's gate list (`.config/gate/wasm32.txt`)` becomes
`the gate's list (`.config/gate/wasm32.txt`)`. `browser.status.md` line 9, `playground.status.md`
line 29. Reflow nothing else.

### 4. `tests/boundary/outward.txt`

Delete the six entries and the `# Raised: outward-citations/repair-1, ...` line directly above
`dashboards.md`'s. If `notes-and-example`'s merge left any count of those files above zero, that is
a question, not a raised entry.

### 5. Finish

Commit, run `notes/common.md`'s checks, and push.

## Report

As the implementer's text says, and also: each sentence you changed beyond Steps, what you found in
the two status files on your base, and the outward check's count of tests.
