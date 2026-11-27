# A repository's .temper/

Split from [ways.md](ways.md), its section 8, on 2026-11-27, and part of that draft: what a repository's `.temper/` holds, how temper reads and guards it, a new project's ways, and versioned files. Its status is its own, and its execution is [ways.execution.md](ways.execution.md)'s.

## 1. Its layout

```
.temper/
  ways.toml            which ways the project offers, in what order, and each command's default
  ways/<name>.toml     a way of the project's: new, or picked from the gallery
  charters/<name>.md   a charter of the project's, whole (6.2)
  hooks/               the repository's hooks and their timeouts (hooks.md)
```

- **`ways.toml`:** `format` (8), `offer`, the ways' names in order, each a file in `ways/`; `[default]`, one offered way per start command (`change = "hand-it-over"`, `milestone = "milestone"`); and `[defaults]`, the project's defaults across its ways (`ways.md`, 5.3). Work already running keeps its way (`ways.md`, section 4).
- **All or nothing** (the user, 2026-11-27): a repository with `ways.toml` offers its own ways alone, and the stock ways aren't offered there, so none of their names is reserved. One without `ways.toml` offers the stock ways, each the default for the command it fits. `ways.toml` is the switch, not `.temper/` itself, so that a repository with hooks alone still runs the stock ways.
- **Ways and charters are the project's,** from its home repository, the repository its path is drawn over (`../domain/forge/change/branch-path.md`, section 1). Hooks and the guide are each repository's own. The landing check refuses `ways/`, `ways.toml` and `charters/` in another of the project's repositories, so that a project's method has one home. A context repository's `.temper/` is never read.
- **What temper doesn't read leaves `.temper/`,** so that the directory says exactly what temper does.

## 2. Use, copy, remove, add

- **From the gallery:** picking an entry writes its file into `ways/`, whole, and names it in `offer`. It is the project's own from then on. `temper ctl way copy NAME` does the same from the terminal, writing as well each charter file its recipes name that the repository lacks, with the stock's text, so that the copy resolves; `temper ctl charter copy NAME` writes a stock charter's files into `charters/`, whole. A gallery entry that is the floor's settings becomes a proposal to the project's owners, never a file.
- **Change one:** edit its file.
- **Remove one:** delete its file and its name from `offer`.
- **Add one:** a file with a new name, which `offer` names.
- **No link back:** a file doesn't say where it came from. What a release changes in the gallery or in the stock charters reaches a project's files through Ask temper's review (`ways.md`, section 11), for a person to decide.

## 3. A way's file

```toml
# .temper/ways/milestone.toml: the project's Milestone, as picked from the gallery and changed
format = 1
headline = "Design first; each goal is built on its own branch, and lands when it's done"
line = "You and an agent write a design; agents build each goal on its own branch, and each lands when it's done."
start = "milestone"

[tasks.milestone]
executor = "gathering"
gathers = "lead"

[tasks.lead]
contract = "report"
holds = ["branch"]
lands = "whole"
into = "base"
delegates = { change = { at_most = 40 } }
budget = 1000
landing = { approvals = [{ role = "owner", people = 1, freshness = "clean" }] }

[tasks.lead.executor]
settings = "goal"                    # runs by the deployment's goal charter's settings
model = "claude-opus-5-5"
effort = "xhigh"

[tasks.lead.executor.agent]           # a charter composed in place: one agent, its first run and its later wakes
instructions = ["common", "lead"]
parts = [{ on = ["first"], file = "designer" }, { on = ["question", "result"], file = "coordinator" }]

[tasks.change]
executor = "change"
into = "parent"
producer = "build"
gates = ["review", "safety"]
landing = { ci = false, up_to_date = false }

[tasks.build]
executor = { agent = "careful-implementer", model = "gpt-6.2-sol", effort = "high", speed = "ultrafast" }
contract = "change"

[tasks.review]
executor = { agent = "reviewer" }
contract = "verdict"

[tasks.safety]
executor = { agent = "reviewer" }
contract = "verdict"
lens = "data safety"

[reaches]
wording = "decide"
retry = "decide"
tests = "decide"
implementation = "ask"
dependency = "ask"
departure = "ask"
findings = "ask"
```

- **People's roles by name:** `role` names a project role as the project's policy does. Its form is the next pass's to settle with `temper ctl`'s.
- **Budget in the deployment's unit,** as a start's `--budget` takes it.
- **`careful-implementer`** is `.temper/charters/careful-implementer.md` with `settings = "implementer"`, and `common`, `lead`, `designer`, `coordinator` and `reviewer` are charter files of the project's (`ways-charters.md`, 2), written from the stock's and changed.

## 4. Hooks

- **Moved to [hooks.md](hooks.md)** (the user, 2026-11-27): `deliver`, `finish`, `merge` and `close`, each run from its own branch's version, with timeouts in `.temper/hooks/config.toml` under a project ceiling. They are the repository's, for every way, and a way can't turn one off.
- **Found where they run,** in the job's checkout, not from the revision 5 keeps.

## 5. How temper reads it

- **Whole, from one commit, or not at all:** the forge reads `.temper/` of the home repository's landing branch at one tip, every file at that commit, within bounds of its own: files, bytes per file and in all. A read cut short, refused, or answered for another commit is no read. The CI workflows' directory is read at a landing tip in the same way, as a precedent for when, though not for how whole (`../domain/forge/adoption.md`, section 11).
- **When:** at adoption; at each move of the landing branch whose diff touches `.temper/`, or can't be told, an outside push and a rewind included; when the project's home changes; and at an engine's start, which reads again rather than trusts a mark held in memory.
- **Kept as the project's repository revision:** what a read gave, validated as the landing check validates it (6), with its commit, saved as a revision of the project's own that no person sets but by landing. Each read is stamped with the observation of the landing branch it was made for, in order, and with the project's home; an answer for an observation older than the revision's, or for another home, is dropped, whatever its commit's age, so that a rewind is taken as any move.
- **The last good revision holds:** a read that fails or doesn't validate keeps the last good one, and the project's record says so, with the commit it couldn't take. Work started meanwhile takes the last good revision.
- **Stock only on a verified absence** (the review's ninth finding): a project whose read found no `ways.toml` runs the stock ways. Before its first good read, after its home changes, or while a read is tried again, a start naming a way or taking a default one waits, saying why, and a start of today's kind goes on.
- **Hooks are found where they run,** in the job's checkout, not from the revision (4).

## 6. Guarded

- **Checked before it lands, on the tree that lands** (the review's third finding): a change touching `.temper/` waits for a check of temper's own, by code, of the tree that will land: under queue, its head merged onto its base's tip, the change made up to date first whatever its rules say; under turn, the candidate; a branch landing whole, the branch's tree at its close. The pass names that head, base and tree, and a change whose head or base moves is checked again before it lands. It is a requirement of temper's own, not a landing rule's, so removing a rule or one of jig's requirements doesn't remove it. Each increment checks what it can parse: the files' form, formats, names, links and home first, then charters, then ways (`ways.execution.md`). The files must parse; names must be valid and unique (`ways.md`, section 9); every way named must exist and be offered; every task's executor, contract, resource and procedure must be temper's, and the guardrails of `ways-recipes.md`, 1 must hold; every charter a way names must have its file, and each file its `settings` charter; every recipe must agree with itself (`ways-recipes.md`, 2); charters must fit their bounds; a hook may be a symbolic link only to another hook beside it ([hooks.md](hooks.md), 3.1); every file must name a format the build reads (8); and no `ways/`, `ways.toml` or `charters/` may sit outside the home repository. A failure refuses with the file and its line, as a gate's verdict does.
- **Signed off, where the owners turn it on** (off by default, the user, 2026-11-27): a change touching `.temper/` then also waits for one owner's approval on the forge, a sign-off of temper's own beside the floor's rules, taking none of a project's rule or path set slots. It is a project setting, refused where no owner can sign, a playground's forge or a project with no owner on its forge.
- **Off, the floor alone guards `.temper/`:** ways can ask for less review than the project's defaults, charters shape what agents do, and hooks run code on workers, so a change an agent writes can edit them and land under the project's rules alone. A rule by path naming `.temper/` is how owners ask for a person meanwhile.
- **The floor stays in settings,** where landing can't reach it.

## 7. A new project's ways

- **At its creation, or its adoption,** a project whose home repository has no `ways.toml` is shown the gallery, each entry drawn, and the stock ways it would run meanwhile.
- **It may start on the stock ways,** writing nothing: they don't ask a person to buy in yet. Or it picks one or more entries, or writes a way from scratch, and may then change them before going on.
- **temper writes them by a change** into the home repository, "Add .temper/ with your ways": `ways.toml`, the ways' files, and the charter files their agents run by, with the stock's text (`ways-charters.md`, 2). It lands by the project's own rules (6), so that the first thing temper does in a repository is a change a person can read. Until it lands, the stock ways run.
- **A starter is offered,** Hand it over preselected, so that the step is one click, with the advice to start small and grow, the whole gallery in sight.

## 8. Versioned files

- **Each file names its format:** `format = 1` in `ways.toml`, in each way's file and in `hooks/config.toml`, and in a charter file's front matter, where leaving it out reads as 1.
- **Every format shipped keeps reading:** a build reads each format temper has released, converting it as it reads. A file lives in a repository temper doesn't control, so this is longer than the store's rule, which keeps no code for a layout older than the deployment runs ([layouts-both-ways.md](layouts-both-ways.md)).
- **A newer format than the build knows** fails the read, naming the file and the format, and the last good revision holds (5).
- **Rewriting a file to the newest format** is a change a person asks for, or Ask temper proposes (`ways.md`, section 11); nothing rewrites it by itself.
- **temper.toml and the other files a person writes** take the same rule, by [formats.md](formats.md).
