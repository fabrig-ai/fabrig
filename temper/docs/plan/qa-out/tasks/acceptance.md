+++
id = "acceptance"
part = "acceptance"
kind = "docs"
needs = []
files = [
  "temper/docs/development/acceptance.md",
  "temper/docs/design/draft/v0.1.0.md",
  "temper/docs/design/domain/testing.md",
  "temper/docs/design/release.md",
  "temper/docs/design/draft/reference-case.execution.md",
  "tools/citations/allow.txt",
]
effort = "high"
minutes = 90
+++

## Goal

temper's own acceptance stories, one page: what a user does and what must hold, for each of the
eight rungs that have a brief, so that they outlive the briefs. The v0.1.0 draft states the bar and
cites the page, without saying who walks it. The design's passages that lean on a rung's "brief"
lean on the page instead.

Acceptance:

1. `temper/docs/development/acceptance.md` has the twelve headings of
   `temper/docs/plan/qa-out/notes/contracts.md`, C2, and each story section has C2's bold leads, in
   order.
2. Every person's words in the briefs stand verbatim in the page's block quotes: the one chat turn
   (rung 0); B0's and B1's request and answer; B2's request and answer; M3's goal; M4's chat; M8's
   three chats. Every "What passes" clause stands in that story's **What must hold** or **No pass**.
3. The fixtures stand exactly, in fenced blocks: the person's gate on Forgejo (`ci.yml` and
   `CONTRIBUTING.md`) and on GitHub (`ci.yml`) in section 7, the person's tier (`slow.yml` and
   `slow.sh`) in section 11.
4. This prints nothing:

   ```sh
   grep -n -E '\bW/|`W`|\bHOST\b|\bREPO\b|\bUNIT\b|\bREPORT\b|give-back|kit(-github|-claude)?\.md|rung-[0-9a-z]+\.md|qlc-|qlg-|fabrig-ai|temper-probe|177453345|walker|QA session|QA driver|\blent\b|\blend|borrow|edition|your task|the report|\bdirector\b' temper/docs/development/acceptance.md
   ```

5. This prints nothing, and `grep -n '^##' temper/docs/design/draft/v0.1.0.md` prints what it
   printed before:

   ```sh
   grep -n -E '\bdirector\b|QA session|QA driver|\bbrief' temper/docs/design/draft/v0.1.0.md
   ```

6. `grep -n -E "(its|the|same) brief|brief's" temper/docs/design/release.md` prints nothing, and in
   `temper/docs/design/domain/testing.md` no line of "The ladder's twins" (2.4) or "The ladder's
   rungs" (2.7) says "brief".
7. Both checks of `temper/docs/plan/qa-out/notes/common.md` pass, with no line added to
   `tests/boundary/outward.txt`.

## Inputs

- `temper/docs/plan/qa-out/inputs/request.md`, section 3.
- `temper/docs/plan/qa-out/notes/common.md`, all of it; it departs from your implementer's text.
- `temper/docs/plan/qa-out/notes/contracts.md`, C1, C2 and C3.
- **The stories:** `temper/docs/plan/v0.1.0-ladder/rung-{0,a,b0,b1,b2}.md`,
  `temper/docs/plan/reference-walks/rung-{m3,m4,m8}.md`.
- **What they share:** `temper/docs/plan/v0.1.0-ladder/kit.md`, `kit-github.md` and `kit-claude.md`
  beside it. Source for sections 2 and 3 only; most of them is the lab's and stays out.
- `temper/docs/development/qa.md`, sections 2 to 6. Read section 3 for what it offers; `qa-page` is
  rewriting it beside you, to the contract (C1).
- `temper/docs/design/domain/testing.md`, 2.4 ("The ladder's twins") and 2.7 ("The ladder's rungs").
- `temper/docs/design/draft/v0.1.0.md`, sections 1 to 4, 7 to 9. `temper/docs/design/release.md`,
  section 6.

## Owns and may change

- **Owns:** `temper/docs/development/acceptance.md`, new.
- **May change, this task's departure from "not yours", because the request names the draft and the
  briefs' readers need it:**
  - `temper/docs/design/draft/v0.1.0.md`: only the passages of step 3.
  - `temper/docs/design/domain/testing.md`: only the passages of step 4.
  - `temper/docs/design/release.md`, section 6's first bullet: only `its brief's scenario`.
  - `temper/docs/design/draft/reference-case.execution.md`: only the lines of step 5.
- **May add to `tools/citations/allow.txt`:** one line for each name in a story's repository that the
  checker reads as a citation, such as `CONTRIBUTING.md`, `README.md` or `changes/1.md`, as testing.md's
  `changes/1.md` line does, under `# Data, examples and names quoted as such.`:
  `temper/docs/development/acceptance.md CONTRIBUTING.md # a story repository's file, as data`.
  Fenced blocks are not read, so the fixtures need none.
- **Touches nothing else:** not `temper/docs/development/qa.md`, not `temper/docs/plan/`, not
  `tests/boundary/outward.txt`, not `temper/docs/help/`, no code.
- **Options:**
  - If a brief and the design disagree on what a story must show, take the brief's words and report
    it. The page records what the briefs hold; it decides nothing new.
  - If a twin or fixture path below does not exist on your trunk, say what is there or "none yet",
    and report it.
  - If a brief holds something you cannot tell as temper's or the lab's, keep it when it is what a
    user does or sees, and drop it when it only runs a sitting (times allowed, where evidence is kept,
    how a report reads). List what you dropped.
  - If the page needs a fact from `temper help`, read the help; don't invent.

## Steps

### 1. Write `temper/docs/development/acceptance.md`

Title `# temper v0.1.0's acceptance stories`, then `Provisional, 2026-11-04.` and a sentence on what
the page is. Wrap at 100 columns. Cite documents from `temper/docs/development/`, as qa.md does
(`../design/draft/v0.1.0.md`, 2.1; `qa.md`, section 6).

**How the briefs translate.** Each row applies throughout:

| In the brief | In the page |
|---|---|
| `W`, `W/...`, `HOST`, `REPO`, `UNIT`, `REPORT` | nothing: say what is meant, or drop it |
| "the four checks `true`" | its configuration checked (`qa.md`, section 3) |
| `W/account.json`, `W/give-back`, "the login in/out", a held or lent login | the person's login moved in by `account import` and back by `account export` (section 2) |
| `W/programs/claude` | the person's Claude Code login, used in place (section 3) |
| "as a user unit", the ssh and `systemd-run` lines | nothing: the box is started as the walk `box` says |
| "Keep all of `W`", wall cap, "say in the report", the report's shape | nothing, but what a user sees |
| "your verdict says so", "did not walk B1" | **No pass**: "A run where ... is no pass." |
| edition notes, walk ids, dates of editions | the rule alone, its reason in the present tense |
| "qa.md 5", "testing.md 2.4" | a citation by path and section |
| "A QA session", "you", "the walker" | "the person", or the passive |
| "Time each phase ..." | **Its phases**, a sentence listing them; `testing.md`, 2.7 takes a run's bounds from them |
| the repository kept for QA, an App's name or id | "the person's repository", "the person's App" |

**1. In one page.** What a story is: what a person does and what must hold, on real backends, read
on both sides. Where the bar is (`../design/draft/v0.1.0.md`, sections 2, 3 and 9). A table with a
row per story: the story and its section, its backends, its twin, its fixture. 0 to B2 are the
ladder's (`../design/draft/v0.1.0.md`, 2.1). M3, M4 and M8 walk milestones of the reference case
(`../design/draft/reference-case.execution.md`, section 3) on the way to rung C. C, D and W have no
story yet: they say so here, and section 12 holds that.

**2. What every story holds.** From `kit.md`, generic:

- **Driven as any user drives temper:** through `temper ctl`, with `temper help` and `temper ctl
  help` the guide, the forge through its API (`qa.md`, sections 4 and 5).
- **Nothing changes but the story's own deployment and repository.**
- **A fresh box:** from empty directories by the walk `box`; configured and funded as `qa.md`,
  section 3 says; its configuration checked before every start; ending with a stop whose last line
  ends `Stopped`.
- **The login in and out:** `account import` moves the person's login into the box, the file leaving
  as the box takes it; `account export` moves it back before the box stops, for any reason, and `stat`
  alone shows a non-empty regular file, mode 0600. A credential's contents are never read or printed.
  A case that risks the box gives the login back first (`qa.md`, section 3). Section 3 has Claude
  Code's form.
- **Two accounts at each step** (`qa.md`, section 6): "read on both sides" means temper's through ctl
  and the forge's through its API, agreeing.
- **Every step from the help:** a place the help and ctl's help did not get there is a finding
  (`../design/draft/v0.1.0.md`, section 4), never a pass by other means.
- **A finding** is `qa.md`, section 6's, with its reproduction from a fresh box, what was expected and
  what was seen. A story that cannot reach its **What must hold** without a hand on the product has not
  passed. A setup that differs from what the pass set is a setup to redo, never a finding.

**3. The backends.** From `../design/draft/v0.1.0.md`, 2.2, `kit-github.md` and `kit-claude.md`:

- **The first pair,** ChatGPT with an owned Forgejo: every story's steps are written for it.
- **Claude with an owned Forgejo:** Claude Code on a person's own login, its signed-in configuration
  directory used in place, never moved, copied or read, as `temper help box`, step 5 says. Declared
  with `account declare 2 claude_code NAME --key KEY`, verified with `account verify ACCOUNT --source
  program`, waited on until usable. Nothing to export: before the box stops no chat or task is
  running; after it, `stat` alone shows the directory there with its mode unchanged.
- **ChatGPT with GitHub, connected:** a table mapping each owned-Forgejo step to GitHub's:
  - the box's forge: `forge connect github-app APP FILE --key KEY` (`--read-only` with an App another
    person or tool also uses), `forge list --until settled`, `forge accept FORGE INSTALLATION`, `forge
    repositories FORGE --installation INSTALLATION`, `repository adopt --project PROJECT --forge FORGE
    --owner OWNER --name NAME --landing main --key KEY`, as the project's home, as `temper help github`
    walks it.
  - a copy of `playground/widget`: the person's repository reset to `README.md` holding `# widget`
    and `src/main.rs` holding the widget temper seeds (`PLAYGROUND_MAIN`,
    `temper/crates/temper-forge-setup/src/lib.rs`), with no other file but its workflow.
  - CI: GitHub Actions, read through the Actions API (`qa.md`, section 5, GitHub's reads).
  - the person's gate: section 7's GitHub fixture, one commit through the git data API, since the
    contents API writes one file per commit; a write under `.github/workflows/` needs a token with
    the Workflows permission.
  - before and after a story: open pull requests and temper's issues closed, branches but `main`
    deleted, the repository's own CI put back.
- **GitHub managed,** temper's App made from its manifest, needs a person's clicks in a browser on the
  engine's machine (`../design/draft/github.md`, 6.1 and 6.5).

**4 to 11. The stories,** from `rung-0.md`, `rung-a.md`, `rung-b0.md`, `rung-b1.md`, `rung-b2.md`,
`rung-m3.md`, `rung-m4.md`, `rung-m8.md`, each in C2's form. **Backends** is one line. A step that
repeats an earlier story's says so by section ("the box, as section 4") rather than again; B1's and
B2's steps may lean on B0's, and B2's fixture on B1's. Keep the checks made before answering (nothing
in the tree, no branch, no pull request, then a wait of 2 minutes and the same again) and the deep-task
paragraph of B2's "What passes".

**Twin and fixtures**, by story:

| Section | Twin and fixtures |
|---|---|
| 4, 0 | none: twins start at B0 (`../design/domain/testing.md`, 2.4) |
| 5, A | none; its data is the playground's story, `temper/crates/temper-playground/stories/a/` (`../design/playground.md`, 5.2) |
| 6, B0 | twin `suite::fuzzy::green_light` in `temper/tests/real-loop`, fixtures `temper/tests/real-loop/tests/fixtures/b0/`; live story `rung_b0_a_person_agrees_a_plan_before_the_change_lands` (`temper/crates/temper/tests/live.rs`), `temper/scripts/proof-live.sh rung-b0`, its offline proofs `rung-b0-offline` |
| 7, B1 | twin `suite::fuzzy::repair`, `temper/tests/real-loop/src/repair.rs`, fixtures `temper/tests/real-loop/tests/fixtures/b1/`; the person's gate, in full here, on Forgejo and on GitHub |
| 8, B2 | the person's gate, section 7's; no twin yet; what holds it on fakes: `temper/tests/engine/domain/tests/suite/forge_routes/b2_tree.rs`, and its pages `temper/tests/client/tests/suite/b2.rs` |
| 9, M3 | the configuration `temper/docs/examples/reference/settings.json`; the goal's draft `temper/docs/design/draft/archive/playground-reset.md`; the repository holds fabrig at the commit under test |
| 10, M4 | none |
| 11, M8 | the person's tier, in full here |

Say with the gate what its rule is for, in the present tense: the note must name when its pull
request was opened, which no push can know before the pull request exists, so a first push cannot
meet it, whatever pull request number it guesses. A log ending `Missing changes/<number>.md with the
line: opened <time>` is the red the story wants. One ending `No pull request time in the event` is a
setup to redo.

**12. Open questions.** The stories of rungs C, D and W are written once their parts are; which of
the case's acts wait for v0.2.0 is `../design/draft/v0.1.0.md`, section 10.

### 2. Reread each story against its brief

Clause by clause: every "What passes" clause has its place. This is acceptance 2 and 3.

### 3. Edit `temper/docs/design/draft/v0.1.0.md`

Section numbers stay. Replace only these passages, as worded here; a line's wrapping is yours.

**Section 1**, the three bullets from "A rung is climbed" to "The director triages":

```markdown
- **A rung is an acceptance story** (section 3): what a person does and
  what must hold (`../../development/acceptance.md`). The same story,
  unchanged, is the acceptance.
- **QA after landing runs on real backends too** (section 3).
- **Every finding is weighed against the happy path** (section 4): one
  on v0.1.0's happy path, the ladder, is fixed with a test in the lowest
  world that reproduces it; the rest waits until v0.1.0 is out.
```

**Section 3**, its first two bullets:

```markdown
- **A story per rung,** in `../../development/acceptance.md`: what a
  person does and what must hold, on the rung's backends, with the
  login's move in and out.
- **The story is driven through `temper ctl`,** as
  `../../development/qa.md` says, from a fresh box, changing nothing
  but its own deployment and repository.
```

Its third bullet:

```markdown
- **Its fixes make it pass:** the rung's work ends when the same story,
  unchanged, passes. It fixes what the story needs, and leaves a finding
  outside it to section 4.
```

Its last bullet:

```markdown
- **QA after landing** climbs `../../development/qa.md`'s rungs (its
  section 3) on real backends too: fakes, a real model, GitHub, one
  live run at a time on a login.
```

**Section 4**, the bullets from "The director triages" to "A rung's own walk":

```markdown
- **A finding on the happy path is fixed before the release;** any
  other waits until v0.1.0 is out.
- **A rung's own findings** are on the path by definition.
```

Its last bullet:

```markdown
- **The tool's friction counts:** a place where a story stalls, is
  guessed at or needs the source to get on is a finding against
  `temper ctl` or the help, since people's agents drive temper the same
  way.
```

**Section 7** and **section 8**, one phrase each:

| Old | New |
|---|---|
| words are a script a QA driver types through `temper ctl` | words are a script, typed through `temper ctl` |
| rung A's brief, on the first pair | rung A, on the first pair |

### 4. Edit `temper/docs/design/domain/testing.md`

Match by words: these phrases cross lines in the file. Nothing else changes.

| Where | Old | New |
|---|---|---|
| 2.4, "The ladder's twins" | added once its brief passes on real backends (its section 3) | added once its acceptance story (`../../development/acceptance.md`) passes on real backends (`../draft/v0.1.0.md`, section 3) |
| 2.4 | **The person's acts and the checks are the brief's,** | **The person's acts and the checks are the story's,** |
| 2.4 | read where the brief reads them | read where the story reads them |
| 2.7, "The ladder's rungs" | a story of their own each once its brief passes on real backends (`../draft/v0.1.0.md`, 3): the brief's scenario as a test | a live story of their own each once its acceptance story (`../../development/acceptance.md`) passes on real backends (`../draft/v0.1.0.md`, 3): the acceptance story's steps as a test |
| 2.7 | **Its acts are the brief's,** | **Its acts are the story's,** |
| 2.7 | as the brief's person acts | as the story's person acts |
| 2.7 | **Its Musts are the brief's "What passes",** | **Its Musts are the story's "What must hold",** |

### 5. Edit the two other readers

| File and place | Old | New |
|---|---|---|
| `temper/docs/design/release.md`, section 6 | its brief's scenario from the help's install step | its story's steps (`../development/acceptance.md`, section 5) from the help's install step |
| `temper/docs/design/draft/reference-case.execution.md`, section 1 | its walk, a QA brief of the case's steps | its walk, a story of the case's steps |
| section 2, step 5 | A brief is in a rung's form | A story is in a rung's form |
| section 2, step 5 | and the brief says which | and the story says which |
| section 2, step 5 | each brief extending the earlier ones | each story extending the earlier ones |
| section 6 | It gets a brief of its own | It gets a story of its own |

### 6. Check, commit, push

Commit first: both checks read what git tracks, and a new page counts only once tracked. Run
acceptance 1 and 4 to 6, then the two checks of `notes/common.md`. Add `allow.txt` lines only for what
the checker names, commit, and run it again. Push your branch.

## Report

As your implementer's text says. Also: what you dropped as the lab's and any clause you could not
place; each twin or fixture that differs from the table; each brief and design disagreement; the
`allow.txt` lines you added.
