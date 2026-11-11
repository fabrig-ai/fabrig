# temper v0.1.0's acceptance stories

Provisional, 2026-11-04. What a person does with temper, and what must hold, for each story of
v0.1.0's acceptance.

## 1. In one page

A story is a person's acts and what must hold on real backends, read on both sides: temper
through ctl and the forge through its API. The bar is `../design/draft/v0.1.0.md`, sections 2, 3
and 9; the same story, unchanged, is its acceptance.

0 to B2 are the ladder's stories (`../design/draft/v0.1.0.md`, 2.1). M3, M4 and M8 walk milestones
of the reference case (`../design/draft/reference-case.execution.md`, section 3) on the way to C.
The table's pairs are the stories' starting backends; section 3 gives their other pairs.

| Story and section | Backends | Twin | Fixture |
|---|---|---|---|
| 0, section 4 | ChatGPT, owned Forgejo | none | none |
| A, section 5 | ChatGPT, owned Forgejo | none | playground A data, section 5 |
| B0, section 6 | ChatGPT, owned Forgejo | green light, section 6 | B0 data, section 6 |
| B1, section 7 | ChatGPT, owned Forgejo | repair, section 7 | B1 data and gate, section 7 |
| B2, section 8 | ChatGPT, owned Forgejo | none yet | section 7's gate; fake checks, section 8 |
| M3, section 9 | ChatGPT, owned Forgejo | none | reference settings and draft, section 9 |
| M4, section 10 | Claude Code, owned Forgejo | none | none |
| M8, section 11 | ChatGPT, owned Forgejo | none | the person's tier, section 11 |

C, D and W have no story yet; section 12 holds them.

## 2. What every story holds

- **Driven as any user drives temper:** through `temper ctl`, with `temper help` and
  `temper ctl help` the guide, the forge through its API (`qa.md`, sections 4 and 5).
- **Nothing changes but the story's own deployment and repository.**
- **A fresh box:** from empty directories by the walk `box`; configured and funded as `qa.md`,
  section 3 says, with the budget raised after sign-in and before forge setup; its configuration
  checked before every start; ending with a stop whose last line ends `Stopped`.
- **The login in and out:** `account import` moves the person's login into the box, the file
  leaving as the box takes it; the account is waited on until usable. `account export` moves it
  back before the box stops, for any reason, and `stat` alone shows a non-empty regular file,
  mode 0600. A credential's contents are never read or printed. A case that risks the box gives
  the login back first (`qa.md`, section 3). Section 3 has Claude Code's form.
- **Two accounts at each step** (`qa.md`, section 6): "read on both sides" means temper's through
  ctl and the forge's through its API, agreeing.
- **Every step from the help:** a place the help and ctl's help did not get there is a finding
  (`../design/draft/v0.1.0.md`, section 4), never a pass by other means.
- **A finding** is `qa.md`, section 6's, with its reproduction from a fresh box, what was expected
  and what was seen. A story that cannot reach its **What must hold** without a hand on the
  product has not passed. A setup that differs from what the pass set is a setup to redo, never
  a finding.

## 3. The backends

`../design/draft/v0.1.0.md`, 2.2 gives the pairs and their order, one run at a time on a login.

**The first pair,** ChatGPT with an owned Forgejo: every story's steps are written for it, except
M4's, which needs Claude Code. A Forgejo here and its runner are made through ctl.

**Claude with an owned Forgejo:** Claude Code on a person's own login, its signed-in configuration
directory used in place, never moved, copied or read, as `temper help box`, step 5 says. Only one
box uses it at a time. Declare it with `account declare 2 claude_code NAME --key KEY`, verify with
`account verify ACCOUNT --source program`, and wait until usable. There is nothing to export:
before the box stops no chat or task is running; after it, `stat` alone shows the directory there
with its mode unchanged. The account list is read before the stop. The configuration and model
choice are `qa.md`, section 3's.

**ChatGPT with GitHub, connected:** the person's App and repository replace the owned Forgejo,
as `temper help github` walks it. The requests and answers keep their words, including "widget".

| Owned-Forgejo step | GitHub step |
|---|---|
| The box's forge | Connect the person's App; adopt its repository as the project's home, below. |
| A copy of `playground/widget` | Reset the person's repository to the widget, below. |
| CI on the box's runner | GitHub Actions on GitHub's runners, read through the Actions API. |
| The person's gate | Section 7's GitHub fixture, in one commit through the git data API. |
| Before and after | Close open pull requests and temper's issues; delete branches but `main`. |

Connect with `forge connect github-app APP FILE --key KEY`, adding `--read-only` with an App
another person or tool also uses. Then `forge list --until settled`,
`forge accept FORGE INSTALLATION`, `forge repositories FORGE --installation INSTALLATION`, and
`repository adopt --project PROJECT --forge FORGE --owner OWNER --name NAME --landing main
--key KEY`
adopt it as the project's home. Rung 0 leaves the repository's files as they are and reads them.

For a copy of widget, reset the person's repository to `README.md` holding `# widget` and
`src/main.rs` holding exactly the widget temper seeds (`PLAYGROUND_MAIN`,
`temper/crates/temper-forge-setup/src/lib.rs`), with no other file but its workflow. Read CI
through the Actions API (`qa.md`, section 5, GitHub's reads): runs at each head, jobs and logs.
The gate is one commit through the git data API because the contents API writes one file per
commit; a write under `.github/workflows/` needs a token with the Workflows permission. Its push
run must pass, `ci` green and `changes` skipped. After the story, put back the repository's own CI
and remove the gate's `CONTRIBUTING.md` and `changes/`.

**GitHub managed,** temper's App made from its manifest, needs a person's clicks in a browser on
the engine's machine (`../design/draft/github.md`, 6.1 and 6.5).

## 4. Rung 0, the ground

**Backends:** ChatGPT with an owned Forgejo; GitHub and Claude use section 3's replacements.

**What the person does**

1. Configure empty box directories with `temper init`, start and sign in by the printed link on
   stdin, as `temper help box`, steps 1 to 3 say, with section 2's configuration and funding.
2. Set up Forgejo here through ctl, as the help's step 4 says: org `work`, repository `hello`,
   the project `me` lists. Read the repository, its files, `main`'s head and the runner's starter
   workflow through the forge API, using the read header the box names. Read through ctl the
   forge, repository and project's home.
3. Import the login as section 2 says and wait until usable. Start one chat in the project:

   > What does this project's repository hold? Answer in two sentences.

   Wait for the reply through ctl, read it whole, and read the chat's task, tree and activity.
   Close the chat.
4. Export the login, check it by `stat`, and read the account list: the account is no longer
   signed in. Stop the box.

**What must hold**

The configuration checked before the start; the box started and signed in; the owned Forgejo
made through ctl alone, temper's account of its forge, repository and project's home agreeing
with the forge API. The login imported and usable; the reply a real model's, true to what the
forge shows, naming what the repository holds; no change started or made. The login exported
back to its file, non-empty, regular and 0600, before the stop; the box's last line `Stopped`.
Every step comes from `temper help` and `temper ctl help`, with each place they did not get there
a finding, as section 2 says.

**Its phases** are configuration, start and sign-in, forge setup, login import, the chat turn,
login export and stop.

**Twin and fixtures:** none; twins start at B0 (`../design/testing/real-loop.md`).

## 5. Rung A, the playground

**Backends:** ChatGPT with an owned Forgejo; GitHub and Claude use section 3's replacements.

**What the person does**

1. Make the box as section 4 says. Through ctl, make a copy of `playground/widget`, adopted as
   the project's home, as `temper help forge` and `temper ctl help forge` name it. Read its files
   and `main`'s head through the API. Find the three suggested requests from temper itself:
   its help, ctl, or the client's suggestions as ctl shows them.
2. Import the login as section 2 says. Ask each suggested request in its own chat, in temper's
   words, nothing appended. Each is answered in words or by delegating a change. Any change
   delegated for the other two goes on and may land. "Add CSV export to widget" must delegate a
   change, push it, open its pull request, pass CI on the box's runner and an agent's review,
   and land on `main`.
3. Follow each through ctl: replies, tree, change panel and waits. Read the forge's pull
   request, files, reviews, CI at its head, `main`'s head and landed `src/main.rs`. Close each chat
   once answered. Export the login and stop as section 4 says.

**What must hold**

Read on both sides: all three requests answered by the model through ctl truthfully, every claim
about the repository checked on the forge; the CSV change delegated, its pull request opened,
CI green at its head on the box's runner, an agent's review passed, landed on `main`, its new
head holding the CSV export. Temper's account of the change's step, pull request, merge commit
and chat's words agrees with the forge at each step; the chat says it landed. The login is
exported back, non-empty and 0600, before the stop. Every step comes from the help, with each gap
a finding, as section 2 says.

**Its phases** are the reply, change made, pull request, CI passed, review, landing and chat's
last word, for each request.

**Twin and fixtures:** no twin; its data is the playground's story,
`temper/crates/temper-playground/stories/a/` (`../design/playground.md`, 5.2).

## 6. Rung B0, the green light

**Backends:** ChatGPT with an owned Forgejo; GitHub and Claude use section 3's replacements.

**What the person does**

1. Make the box as section 4 and the playground's repository as section 5; read its files and
   `main`'s head. Import the login as section 2 says.
2. Start a chat in widget's project with these words:

   > Add CSV export to widget. Before anything is changed, have the work tell me its plan and
   > wait for my go-ahead.

   Through ctl alone follow the reply, tree and person's inbox, waiting on it rather than
   reading it again and again. The chat delegates the work; the work asks its plan and whether
   to go ahead, and the question waits in the inbox, its words readable through ctl.
3. Before answering, read both sides: no change in the chat's tree; no branch but `main` and no
   pull request on the forge. Wait 2 minutes and check again: nothing moved.
4. Answer from the inbox through ctl:

   > Go ahead. The CSV's first line must be the header fruit,count.

   Follow the change as section 5 says: panels, waits, chat's answers, pull request, CI on the
   box's runner, review and landing. Read `src/main.rs` at `main`'s new head for `fruit,count`.
5. Close the chat, export the login and stop as section 4 says.

**What must hold**

Read on both sides: the work's question about its plan reaches the inbox before anything changes,
whether the work asks it or the chat asks for the work; the person's answer reaches the work
before its change is made. Nothing pushed and no change made before the answer. The change
lands with CI green at its head and an agent's review passed; `main`'s `src/main.rs` holds the
header `fruit,count`; the chat says it landed. The login is exported back, non-empty and 0600,
before the stop; temper and the forge agree at every step.

**No pass:** a run where the question is asked only in a chat reply, never in the inbox, or where
the work goes ahead without asking, is no pass.

**Its phases**, from the request, are the question in the inbox, answer sent, change made, pull
request, CI passed, review, landing and chat's last word.

**Twin and fixtures:** `suite::fuzzy::green_light` in `temper/tests/real-loop`, with fixtures in
`temper/tests/real-loop/tests/fixtures/b0/`. The live story is
`rung_b0_a_person_agrees_a_plan_before_the_change_lands` in `temper/bin/temper-box/tests/live.rs`,
run by `temper/scripts/proof-live.sh rung-b0`, with its offline proofs `rung-b0-offline`
(`../design/testing/real-loop.md` and `../design/testing/live-tier.md`).

## 7. Rung B1, the repair

**Backends:** ChatGPT with an owned Forgejo; GitHub and Claude use section 3's replacements.

**What the person does**

1. Make the box and repository as section 6 says. Before importing the login, commit the two
   Forgejo gate files below to `main`, exactly, in one commit through the person's write API,
   message "CI: each pull request adds changes/<its number>.md". Replace the workflow and add
   `CONTRIBUTING.md`; wait for the push's combined status at `main`'s head to pass, and read what
   ctl shows of it, if anything. GitHub uses section 3's one-commit route and the GitHub fixture.
2. Import the login and start a chat in widget's project:

   > Add CSV export to widget. Before anything is changed, have the work tell me its plan and
   > wait for my go-ahead.

   Follow the reply, tree and inbox as section 6 says. Before answering, check no change in the
   tree, no branch but `main`, no pull request; wait 2 minutes and check the same again.
3. Answer through ctl's inbox:

   > Go ahead. The CSV's first line must be the header fruit,count.

   Follow the change through ctl and the API: pull request, first CI failure, repair and new
   head on that same pull request, passing CI, passing review and landing. Read the commits,
   runs at each head, jobs and every failed job's log (`qa.md`, section 5). At `main`'s new head
   read `src/main.rs` and `changes/`.
4. Close the chat, export the login and stop as section 4 says.

**What must hold**

Read on both sides: the work's plan question reaches the inbox before anything changes, asked by
the work or the chat for it; the answer reaches the work before its change. Nothing pushed and
no change made before the answer. CI at the first head is red on the box's runner, its `changes`
job failing for the missing or incomplete `changes/<number>.md` alone. Temper's change shows the
failure and repair through ctl. The repair is temper's, delegated by the change rather than the
chat or person, with the failure CI showed in the repair's instructions. Both jobs are green at
the repaired head on the same pull request. `main` holds `changes/<number>.md` and the header
`fruit,count`; an agent's review passed at the landed head; the chat says it landed. The login
is exported back, non-empty and 0600, before the stop; temper and the forge agree at each step.

**No pass:** a run where the work goes ahead without asking is no pass for B0 or B1; a run whose
first head passes CI is no pass for B1. A log ending `No pull request time in the event` is a
setup to redo, never a finding.

**Its phases**, from the request, are the question in the inbox, answer sent, change made, pull
request, first CI failure, repair's start and push, CI passed, review, landing and chat's last
word.

**Twin and fixtures:** `suite::fuzzy::repair`, `temper/tests/real-loop/src/repair.rs`, with
fixtures in `temper/tests/real-loop/tests/fixtures/b1/` (`../design/testing/real-loop.md`).
The person's gate follows in full.

The note must name when its pull request was opened, which no push can know before the pull
request exists, so a first push cannot meet it, whatever pull request number it guesses. A log
ending `Missing changes/<number>.md with the line: opened <time>` is the red this story wants.

On Forgejo, `.forgejo/workflows/ci.yml`:

```yaml
on:
  push:
    branches: [main]
  pull_request:
jobs:
  check:
    runs-on: temper-host
    steps:
      - run: echo "temper's starter check passed; replace this step with your own checks"
  changes:
    if: ${{ github.event_name == 'pull_request' }}
    runs-on: temper-host
    steps:
      - name: Each pull request adds changes/<its number>.md, naming when it was opened
        run: |
          n='${{ github.event.pull_request.number }}'
          opened='${{ github.event.pull_request.created_at }}'
          echo "pull_request.number=$n opened=$opened"
          test -n "$opened" || { echo 'No pull request time in the event'; exit 2; }
          auth=$(printf 'x-access-token:%s' "${{ github.token }}" | base64 -w0)
          git -c "http.extraHeader=Authorization: basic $auth" clone --quiet --no-checkout "${{ github.server_url }}/${{ github.repository }}.git" repo
          git -C repo checkout --quiet '${{ github.event.pull_request.head.sha }}'
          f="repo/changes/$n.md"
          test -f "$f" && grep -qxF "opened $opened" "$f" || { echo "Missing changes/$n.md with the line: opened $opened"; exit 1; }
```

On both forges, `CONTRIBUTING.md`:

```markdown
# Contributing

Each pull request adds `changes/<its number>.md`: one line saying what it
changes, for the next release's notes, then the line `opened <when the pull
request was opened>`, exactly as CI prints it. CI checks it.
```

On GitHub, `.github/workflows/ci.yml`:

```yaml
on:
  push:
    branches: [main]
  pull_request:
jobs:
  ci:
    runs-on: ubuntu-latest
    steps:
      - run: echo "the probe's starter check passed"
  changes:
    if: ${{ github.event_name == 'pull_request' }}
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ github.event.pull_request.head.sha }}
      - name: Each pull request adds changes/<its number>.md, naming when it was opened
        run: |
          n='${{ github.event.pull_request.number }}'
          opened='${{ github.event.pull_request.created_at }}'
          echo "pull_request.number=$n opened=$opened"
          test -n "$opened" || { echo 'No pull request time in the event'; exit 2; }
          f="changes/$n.md"
          test -f "$f" && grep -qxF "opened $opened" "$f" || { echo "Missing changes/$n.md with the line: opened $opened"; exit 1; }
```

## 8. Rung B2, two delegates

**Backends:** ChatGPT with an owned Forgejo; GitHub and Claude use section 3's replacements.

**What the person does**

1. Make the box, repository and person's gate as section 7 says, the gate's push CI passed and
   ctl's view of it read. Import the login.
2. Start a chat in widget's project:

   > Add CSV and JSON export to widget. First have an advisor read widget and report how the
   > two exports should share the table's rows, without changing anything. Then tell me the
   > plan and wait for my go-ahead. Make CSV first, and JSON only after CSV has landed, reusing
   > what CSV adds.

   Follow through ctl the reply, tree, advisor's task and result, and inbox, waiting on it.
   The chat delegates the work; the work delegates an advisor; only after its report does the
   plan question reach the inbox, readable through ctl and naming what the advisor found.
3. Before answering, check no change in the tree, no branch but `main`, no pull request; the
   advisor ended with its report and made nothing on the forge. Wait 2 minutes and check the
   same again: nothing moved.
4. Answer from the inbox through ctl:

   > Go ahead. CSV's first line is the header fruit,count. JSON is an array of objects with the
   > keys fruit and count.

   Follow both changes through their panels, waits, tree and chat's answers, and through the
   API. CSV goes through its pull request, first red CI, repair, green CI, review and landing;
   JSON's task starts only after CSV's merge, read on both sides, then goes through its own red
   CI, repair, green CI, review and landing. Read each pull request's commits, runs at each head
   and failed jobs' logs (`qa.md`, section 5), and `src/main.rs` and `changes/` at `main`'s new
   head. Read the work's spend in its task document through ctl, and its result before the chat's.
5. Close the chat, export the login and stop as section 4 says.

**What must hold**

A deep task's page (a change's producer, repairer or reviewer, three ancestors down) may show
its lineage cut at the box's lines and say so: expected on the small profile, not a finding, as
long as its own row reads whole.

Read on both sides: the advisor's report arrives before the question, with no branch, pull
request or change made by the advisor. The work's plan question is in the inbox, asked by the
work or the chat for it, and the person's answer reaches the work before its change. Nothing
pushed and no change made before the answer. There are two changes, JSON made only after CSV
landed, each with CI red at its first head for missing or incomplete `changes/<number>.md` alone,
its repair temper's, then green at the repaired head and an agent's review passed. Both land in
that order. `main` holds the CSV header `fruit,count`, the JSON export as an array of objects with
keys `fruit` and `count`, and a `changes/<number>.md` for each pull request. The work's spend is
read through ctl; the chat says both landed. The login is exported back, non-empty and 0600,
before the stop; temper and the forge agree at every step.

**No pass:** a run where anything changes before the answer, or JSON starts before CSV lands, is
no pass. A run whose first heads pass CI is no pass for B2's gate. Section 7's setup-red rule
holds here too.

**Its phases**, from the request, are the advisor's start and report, question in the inbox,
answer sent, each change made, each pull request, each first CI failure, each repair's push,
each green run, each review, each landing and the chat's last word.

**Twin and fixtures:** the person's gate is section 7's; no twin yet. What holds it on fakes is
`temper/tests/engine/domain/tests/suite/forge_routes/b2_tree.rs`, and its pages are held by
`temper/tests/client/tests/suite/b2.rs`.

## 9. Rung M3, a goal for a draft

**Backends:** ChatGPT with an owned Forgejo; the goal's work at `gpt-6.1-sol`.

**What the person does**

1. Make the box as section 4 says, with a budget sized as `temper help budget` says for a goal
   of up to a hundred changes. Through ctl, set up an owned Forgejo with a first repository
   `fabrig` in a new project (`forge setup --org fabrig --repository fabrig`, as the help names
   it). As the person, push fabrig at the commit under test to its `main`. Read through the API
   that head and `temper/docs/design/draft/archive/playground-reset.md` in it.
2. Apply the reference settings named below through ctl as `temper help configure` walks it,
   with the authority `docs/examples/reference/README.md` gives:
   coordinator's charter 5, designer's charter 6, coding agent's charter 2, template 4 for a
   draft's goal, and landing rules for `main` and goal branches. Adapt only where the help says:
   models to what the box offers, offering `gpt-6.1-sol` for the goal's work, and landing rules
   to this box's repository. `settings show` names each; `settings show template 4` names its own
   goal branch. Import the login.
3. Start a goal from template 4 in the `fabrig` project through ctl:

   > Implement `temper/docs/design/draft/archive/playground-reset.md`.

   Read `task show`: template 4's charter and LLM choice, tracked; the goal's own branch g,
   `temper/<G>/g<G>`, shown by the forge at `main`'s tip.
4. Read the goal's activity and its run's sub-agent sessions through ctl: the first run reads
   the draft and touched designs, asks inspect-only surveys side by side, each at its LLM
   choice, what it reads, none refused and nothing changed by them.
5. Follow through ctl (`task show`, `change show`, the goal's tree) and the forge (g's head,
   pull requests into g and merges): S, the shared shape; one designer's change per family
   that needs one, each after S; and the changes the goal specifies itself. S lands into g,
   then each designer writes its family's design, lists changes in its result and lands into
   g. Each result wakes the goal to make the family's batch. Each producer is under its
   charter at its LLM choice, designers under charter 6. `main` does not move.
6. Once all changes land into g, the goal makes X, the close-out under charter 6. X lands into
   g last; the goal reports and ends, g kept unlanded. Read the goal's spend. Export the login
   and stop as section 4 says.

**What must hold**

Read on both sides: the goal from template 4 on its own branch g made at `main`'s tip; inspect-only
surveys side by side, none refused; S, designers' changes and each family's batch made and landed
into g as above, each producer under its charter at its LLM choice. The happy path has no
questions, early starts or restarts: each designer starts once S lands. X lands into g last;
the goal reports; g stays and `main` is unmoved; temper and the forge agree at each step; the
login is back before the stop. Every step comes from the help or each gap is a finding. Where
ctl has no way to a step and the help says so, that is a blocking finding; the nearest way the
help names, if any, lets the remaining steps be observed, with that way stated.

**No pass:** a run that needs a hand on the product to reach these outcomes is no pass, as
section 2 says; going on after a blocking ctl gap does not make it a pass.

**Its phases** are the goal's start, surveys, S's landing, designers' landings, families' batches
and X's landing, including the whole interval from the goal's start to X's landing.

**Twin and fixtures:** no twin; the configuration is
`temper/docs/examples/reference/settings.json`; the goal's draft is
`temper/docs/design/draft/archive/playground-reset.md`; the repository holds fabrig at the commit
under test.

## 10. Rung M4, surveys as sub-agents

**Backends:** Claude Code with an owned Forgejo, using the login in place as section 3 says.

**What the person does**

1. Make the box and playground's repository as sections 4 and 5 say, on Claude Code; read its
   files and `main`'s head. Declare and verify the login as section 3 says.
2. Start a chat through ctl in widget's project with a model and two sub-agent choices from
   `settings show charter 1`, two different choices if it offers two:
   `chat start --project PROJECT --model MODEL --sub-agent MODEL_A[:EFFORT]
   --sub-agent MODEL_B[:EFFORT]`,
   as `temper ctl help chat start` names it. Its words are:

   > Have two sub-agents read `src/main.rs` and `README.md` side by side, then tell me what each
   > found.

3. Read through ctl the activity, sub-agents' sessions and choices, and their answer. Read the
   forge: `main`'s head unchanged, no new branch or pull request. End use of the login, stop the
   box and check the login directory by `stat`, as section 3 says.

**What must hold**

Two inspect-only sub-agents run side by side, each a session of its own at its LLM choice, none
refused as too many, each ended on its budget, read through ctl. The answer is true to both
files; nothing changes on the forge. Use of the login ends before the stop, and `stat` checks its
directory, mode unchanged. Every step comes from the help or each gap is a finding.

**Its phases** are the chat's start, the sub-agents' sessions and their ends, and the chat's
answer, including the whole interval from the chat's start to its answer; each child's spend is
read with its session.

**Twin and fixtures:** none.

## 11. Rung M8, tiers and verdicts

**Backends:** ChatGPT with an owned Forgejo.

**What the person does**

1. Make the box and playground's repository as sections 4 and 5 say; read its files and
   `main`'s head. As the person, commit the two tier files below exactly through Forgejo's write
   API, one commit, message "A slow tier: the README checks". Set `tests/slow.sh` to mode 0755 if
   the API permits it; otherwise the workflow runs it with `sh`.
2. Configure the person's tier through CONFIG and `temper help tier` alone: the repository's
   landing configuration gets `"tiers": [{"name": "slow", "workflow": "slow.yml", "ignored": [],
   "artifact": "report", "format": "ids"}]`, or the help's format name for one test id per line,
   beside its gates. Restart the box with its configuration checked and read what ctl shows
   of the tier, if anything. Import the login.
3. Ask through a chat in widget's project:

   > Add the line "Tier walk: first." to the end of README.md in widget.

   Follow it as section 5 says: pull request, CI on the box's runner, agent's review, landed on
   `main`. Read `verifying`, the tier run's task (`task show`), its dispatched run on the forge,
   branch `temper/tier/slow.N` and `report` artifact. While this run continues, do step 4.
4. While the tier's first full run still runs (it sleeps ten minutes), start two more chats,
   one after the other:

   > Add the line "BREAK the slow check" to the end of README.md in widget.

   > Add the line "Tier walk: third." to the end of README.md in widget.

   Follow both to landed. The next tier run starts at the newest tip once the first ends,
   covering both landings. Read the first fix with `change show CHANGE --until cleared`: the
   first run is green, the fix `cleared`, its result `Cleared by slow run N.`. Read the next run
   red on `no-break` alone, its rerun alone at the same tip failing
   again, then bisection running `no-break` alone at the second fix's commit, failing and blaming
   that fix; the third fix clears.
5. Read both sides: change panels (`red` naming the run and failed test, `cleared`), run tasks,
   dispatched runs, inputs `temper_commit` and `temper_tests`, artifacts, and each run's branch
   gone when it ends. Read each chat's words after landing: it landed, and whether it says
   cleared or red. Close the chats, export the login and stop as section 4 says.

**What must hold**

Read on both sides: the person configures the tier through CONFIG and the help alone. The first
fix lands, is `verifying`, then `cleared` by the green tier run, its result saying so. The second
and third fixes land in one run's range; that run is red on `no-break`, its failed test rerun
alone at the same tip fails again, and bisection runs it alone at the second fix's commit and
blames the second fix alone. The third fix clears. Each run has its own `temper/tier/slow.N`
branch, deleted when it ends. Temper's panels, run tasks and results agree with the forge's
dispatched runs, inputs and artifacts at every step. The login is exported back, non-empty and
0600, before the stop. Every step comes from the help or each gap is a finding.

**No pass:** a run where the tier never runs, or runs but temper never judges it, is no pass.
If the second and third landings do not fall in one range, blame is direct and bisection is
unobserved; that run does not pass the bisection requirement.

**Its phases**, from the first request, are the landings, each tier run's start and end, the
rerun, bisection's runs, and each change's `cleared` or `red`.

**Twin and fixtures:** no twin; the person's tier follows in full.

`.forgejo/workflows/slow.yml`:

```yaml
on:
  workflow_dispatch:
    inputs:
      temper_commit:
        required: true
      temper_key:
        required: true
      temper_tests:
        required: false
        default: ""
jobs:
  slow:
    runs-on: temper-host
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ inputs.temper_commit }}
      - name: The README checks
        env:
          TEMPER_TESTS: ${{ inputs.temper_tests }}
        run: sh tests/slow.sh
      - uses: actions/upload-artifact@v3
        if: always()
        with:
          name: report
          path: report/failed.txt
```

`tests/slow.sh`:

```sh
#!/bin/sh
# Two README checks; with TEMPER_TESTS (one id per line) only those, at once; else all, slowly.
set -u
mkdir -p report
: > report/failed.txt
tests=${TEMPER_TESTS:-}
if [ -z "$tests" ]; then
    tests="readme-heading
no-break"
    sleep 600
fi
status=0
for t in $tests; do
    case $t in
        readme-heading) head -n 1 README.md | grep -q '^# ' ;;
        no-break) ! grep -q 'BREAK' README.md ;;
        *) false ;;
    esac || { echo "$t" >> report/failed.txt; status=1; }
done
exit $status
```

## 12. Open questions

The stories of rungs C, D and W are written once their parts are. Which of the case's acts wait
for v0.2.0 is `../design/draft/v0.1.0.md`, section 10.
