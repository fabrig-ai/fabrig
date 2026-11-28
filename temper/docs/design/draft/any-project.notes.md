# any-project: notes

The reasons behind [any-project.md](any-project.md), tested on real projects. On 2026-11-28 the user asked: "can we take a couple of real world third party projects and make a 'simulation' of how temper would work in each case? (what the owner would practically do when trying out temper for that project)", "based on your ideas". Two walks follow, hono and vector, from opposite ends of the spectrum, and a third project that stopped at its first step, click. Each project's facts come from its repository at a named commit, read by a survey; those a walk leans on were checked by hand, and each links to its lines. What the owner does is imagined, step by step, as the draft would have it, and so are the issues it starts from. Each step names the draft section it exercises, and **Gap** marks what the draft lacked. Section 5 gathers the gaps, and the draft takes them up in its 7.5 and its open questions 7 to 9.

## 1. The projects

| Project | What it is | Where it sits |
|---|---|---|
| [honojs/hono](https://github.com/honojs/hono/tree/717dc17cb80bff270ddc557614fe3c3e1b916a1f), at `717dc17` | a TypeScript web framework for many JavaScript runtimes, 589 files, nine packages | rungs 1 and 2: no guide, no design records, ideas as issues |
| [vectordotdev/vector](https://github.com/vectordotdev/vector/tree/83cc0a23b61b60b67141bdbc49cc3bb0d0b30c9c), at `83cc0a2` | a Rust observability pipeline, 4,494 files, a workspace of 41 crates | rung 3: a long `AGENTS.md`, an AI policy, 86 RFCs, changelog fragments, a merge queue |
| [pallets/click](https://github.com/pallets/click/tree/08b803cf32ceb2ae6a772c62d5da42757a897d44), at `08b803c` | a Python library for command lines, 179 files | stopped: its organisation bans generated pull requests |

## 2. hono

### 2.1 What its owner starts from

- **No guide and no design records.** Ideas come as issues, and its founder writes "I want to write the code as I like" ([CONTRIBUTING:15](https://github.com/honojs/hono/blob/717dc17cb80bff270ddc557614fe3c3e1b916a1f/docs/CONTRIBUTING.md#L15)). "Pull requests are limited to maintainers" ([:29](https://github.com/honojs/hono/blob/717dc17cb80bff270ddc557614fe3c3e1b916a1f/docs/CONTRIBUTING.md#L29)).
- **AI is allowed with a bar:** "You may use AI to contribute, but it must never waste a maintainer's time or make their work unpleasant" ([:23](https://github.com/honojs/hono/blob/717dc17cb80bff270ddc557614fe3c3e1b916a1f/docs/CONTRIBUTING.md#L23)).
- **Four tools:** Node 24, Bun 1.2, Deno 2.4 and pnpm, pinned in [.tool-versions](https://github.com/honojs/hono/blob/717dc17cb80bff270ddc557614fe3c3e1b916a1f/.tool-versions).
- **CI across runtimes:** [ci.yml](https://github.com/honojs/hono/blob/717dc17cb80bff270ddc557614fe3c3e1b916a1f/.github/workflows/ci.yml) runs `pnpm run check`, the build and the tests, then Bun (on Windows too), Deno, Node 20, 22 and 24, workerd and Fastly; `ci-adapters.yml` runs the adapters, and `perf.yml` the speed and size checks, on a label.
- **A bot that pushes:** on every ready pull request, [autofix.yml](https://github.com/honojs/hono/blob/717dc17cb80bff270ddc557614fe3c3e1b916a1f/.github/workflows/autofix.yml#L33-L37) runs `pnpm run check:fix` and commits "ci: apply automated fixes".
- **What a pull request carries:** tests, `pnpm run check:fix` and TSDoc, if applicable ([pull_request_template](https://github.com/honojs/hono/blob/717dc17cb80bff270ddc557614fe3c3e1b916a1f/.github/pull_request_template.md)). There is no changelog: release notes are drafted from commit subjects.

### 2.2 Day one, nothing written (7.1)

1. **Ten minutes:** a maintainer connects GitHub through temper's App and adopts `honojs/hono`, landing on `main`. Ask temper offers "Set up temper here". They skip it, to see what temper does as it is.
2. **A first change:** Hand it over, from an imagined issue: "c.json drops a header a middleware set". The implementer has no guide, reads `package.json`, and runs `pnpm run test`. **Gap, the worker's tools:** a worker with Node alone runs the core's tests, but not Bun's or Deno's. CI runs them later, and a red comes back as a repair.
3. **Delivery checks nothing,** since there is no hook (2.4). The pull request opens and about ten CI jobs start.
4. **The bot pushes onto temper's branch** "ci: apply automated fixes". **Gap, the forge's other actors:** the head temper checked and reviewed has moved under it. The review, with clean freshness, runs again, and temper's next push builds on a commit it didn't make. The cheap answer is the project's: a delivery hook running `pnpm run check`, and a guide line to run `check:fix`, leave the bot nothing to fix (2.3).
5. **The reviewer approves, CI passes, and temper lands it.** Its subject becomes a line of the next release's notes, which the stock implementer already writes for the history.

The bottom rung is real: a project with no guide and no design runs on the first day. The first friction is tools and bots, not design.

### 2.3 "Set up temper here" (7.3)

Ask temper reads `package.json`'s scripts, `.tool-versions`, the CI workflows, the pull request template and the contributing guide. It proposes one change, part by part:

- **A new `AGENTS.md`,** short, all of it found in the repository:

```markdown
# hono, for agents

- **Setup:** `pnpm install --frozen-lockfile`; Node, Bun, Deno and pnpm are pinned in `.tool-versions`.
- **Before you finish:** `pnpm run check:fix`; `pnpm run test` for the core; `pnpm run test:bun`, `test:deno` or `test:node` where a change touches that runtime; `pnpm --filter @hono/NAME run test` for an adapter.
- **Every change:** tests that fail without it, and TSDoc on what it makes public.
- **Size and speed are features:** no runtime dependency in `src/`; say what a change does to the bundle's size.
- **A commit's subject is a line of the release notes:** what changed, for someone using hono.
```

- **`.temper/hooks/deliver`:**

```sh
#!/bin/sh
set -e
pnpm install --frozen-lockfile --prefer-offline
pnpm run check
pnpm run build
pnpm run test
```

- **No ways:** the stock ways run, Hand it over the default.
- **No design line:** nothing found, so rung 2 for goals. The change's words say why: "hono keeps its design in issues; a goal will plan in `.goal/`, and the plan goes in its pull request, beside the issue it answers."
- **For the deployment, not the repository:** "your workers need Bun and Deno for these checks", an owner's step on their machines. **Gap, the worker's tools:** the draft says "a worker" and nothing of what it must hold.

### 2.4 A bigger piece: a goal at rung 2 (5, 6.1)

1. **From an issue's words** (6.1), an imagined one: "an option on the CORS middleware, honoured by the Bun and Deno adapters".
2. **The designer writes `.goal/plan.md`:** the option, where it is read, and each adapter's half; and `.goal/contracts.md`, the option's type, which all three changes share. It cuts three changes, the two adapters `after` the core.
3. **Its questions reach the founder:** Hand it over and Milestone ask on `implementation` and `departure` (`ways-recipes.md`, section 4), so the API's shape is theirs before any code is. "I want to write the code as I like" fits the way's `[reaches]` as it stands.
4. **The closer takes `.goal/` out,** opens one pull request into `main`, and writes the plan into its description. The maintainer reviews one pull request, with its design above the diff.

### 2.5 What hono shows

- Day one needs nothing written. A guide of five lines and a hook of four pay first.
- Rung 2 fits a project whose design lives in issues: the plan lands in the pull request, beside the issue.
- A project's own bots push to the branches temper works on.
- The worker's tools are part of plugging in.

## 3. vector

### 3.1 What its owner starts from

- **An AI policy that welcomes agents with a person accountable:** "You own what you submit" ([AI_POLICY:11](https://github.com/vectordotdev/vector/blob/83cc0a23b61b60b67141bdbc49cc3bb0d0b30c9c/AI_POLICY.md#L11)). A reviewer's question is answered "in your own words rather than relaying AI output" ([:13](https://github.com/vectordotdev/vector/blob/83cc0a23b61b60b67141bdbc49cc3bb0d0b30c9c/AI_POLICY.md#L13)). A pull request "substantially or fully generated by an agent" should "mention the model used" in its commit message, or at least its description ([:37](https://github.com/vectordotdev/vector/blob/83cc0a23b61b60b67141bdbc49cc3bb0d0b30c9c/AI_POLICY.md#L37)). Pull requests "often receive automated AI code review", to be read before a person's ([:27-29](https://github.com/vectordotdev/vector/blob/83cc0a23b61b60b67141bdbc49cc3bb0d0b30c9c/AI_POLICY.md#L27-L29)).
- **A guide of 259 lines,** [AGENTS](https://github.com/vectordotdev/vector/blob/83cc0a23b61b60b67141bdbc49cc3bb0d0b30c9c/AGENTS.md): Clippy after every change, `make fmt`, the narrowest tests by feature; a pre-push hook it recommends ([:179-197](https://github.com/vectordotdev/vector/blob/83cc0a23b61b60b67141bdbc49cc3bb0d0b30c9c/AGENTS.md#L179-L197)); "Do NOT include co-authoring information from coding agents (i.e. avoid "Co-Authored-By: Claude" attribution)" ([:242](https://github.com/vectordotdev/vector/blob/83cc0a23b61b60b67141bdbc49cc3bb0d0b30c9c/AGENTS.md#L242)); and, while a pull request is open, "never rewrite published commits or force-push the branch" ([:255](https://github.com/vectordotdev/vector/blob/83cc0a23b61b60b67141bdbc49cc3bb0d0b30c9c/AGENTS.md#L255)). It says nothing of RFCs.
- **RFCs:** 86 dated ones under `rfcs/`, from a template. One is needed for an architectural or data-model change, a new component's behaviour, a removal, a substantial change users see. "At least 3 Vector team members must approve your RFC" ([rfcs/README:98](https://github.com/vectordotdev/vector/blob/83cc0a23b61b60b67141bdbc49cc3bb0d0b30c9c/rfcs/README.md#L98)), after a last call a week long, and its implementation is broken into issues.
- **What each pull request carries:** a fragment `changelog.d/<name>.<type>.md`, which "must end with an `authors:` line" ([changelog.d/README:131](https://github.com/vectordotdev/vector/blob/83cc0a23b61b60b67141bdbc49cc3bb0d0b30c9c/changelog.d/README.md#L131)); a CLA signed through CLA Assistant ([cla.yml](https://github.com/vectordotdev/vector/blob/83cc0a23b61b60b67141bdbc49cc3bb0d0b30c9c/.github/workflows/cla.yml)); and a template asking for the configuration used and the test results.
- **Landing:** one team member reviews a minor change, two a major one, three an RFC, plus one from each set of code owners; "All pull requests are squashed and merged", and those "over 300-500 lines of diff" are discouraged ([CONTRIBUTING:178-191](https://github.com/vectordotdev/vector/blob/83cc0a23b61b60b67141bdbc49cc3bb0d0b30c9c/CONTRIBUTING.md#L178-L191)). GitHub's merge queue runs the costly suites ([master_merge_queue.yml](https://github.com/vectordotdev/vector/blob/83cc0a23b61b60b67141bdbc49cc3bb0d0b30c9c/.github/workflows/master_merge_queue.yml)): an ordinary pull request's checks pass with them skipped.
- **Cost:** the component-features job "Usually takes 2h" ([component_features.yml:36](https://github.com/vectordotdev/vector/blob/83cc0a23b61b60b67141bdbc49cc3bb0d0b30c9c/.github/workflows/component_features.yml#L36)); integration needs Docker.

### 3.2 Setting up first (7.2, 7.3)

1. **Adoption:** a maintainer adopts `vectordotdev/vector`, landing on `master`. A way's `into = "base"` takes it; Draft first's literal `main` (2.1) would have refused it.
2. **The policy before anything.** **Gap, the policy:** no step of the draft reads one. Ask temper reads the AI policy first and says what it means here, before it proposes a line: you own each change temper opens; temper's pull requests name their models; temper adds no co-author line; reviewers' questions come to you, for you to answer in your own words.
3. **The change it proposes:**
   - **`AGENTS.md`, two lines added, nothing rewritten:**

```markdown
- **Design:** a change that needs an RFC by `rfcs/README.md` starts with one, from `rfcs/_YYYY-MM-DD-issue#-title.md`, and lands before its code; other changes need none.
- **Size:** keep a pull request under about 300 lines of diff, and split larger work (`CONTRIBUTING.md`, Reviews & Approvals).
```

   - **`.temper/hooks/deliver`,** the guide's own pre-push hook, less what costs a build: `make check-fmt`, `make check-markdown` and `make check-changelog-fragments`. Clippy and the narrow tests stay the implementer's, as the guide's cycle already says.
   - **Its ways:** Hand it over with one owner's approval in its landing, and Milestone for RFC work.
   - **For the floor, a proposal to the owners, not a file:** changes touching `rfcs/**` wait for three owners' approvals, as the gallery's sensitive paths ([ways-gallery.md](ways-gallery.md), 3.2).
   - **The rung:** 3, from the design line; the RFC is the record that stays.

A rich guide is most of the setup: temper brings two lines and a hook the guide already wrote.

### 3.3 Day one, a fix

1. **From an issue's words,** an imagined one: a sink retrying on a status it shouldn't. Hand it over.
2. **The implementer follows the guide's cycle,** writes a fragment of type `fix` under `changelog.d/`, named for its issue, and ends it `authors:` and the requester's login, which temper knows. The hook passes.
3. **The pull request opens.** **Gap, what each change carries:** the stock implementer's report doesn't fill the repository's pull request template, and here the template asks for the configuration and the tests run.
4. **CLA Assistant asks the commit's author to sign.** **Gap, whose name:** temper's commits are authored and committed by its forge identity ([../protocol/git.md](../protocol/git.md), section 5), which can't sign a CLA as a person. Authored as the requester, who has signed, with temper as committer, the change passes.
5. **The policy asks for the model.** **Gap, disclosure:** the guide forbids a co-author trailer, and the policy asks for "Generated with MODEL". temper knows each run's model; a line naming them, written by code into the landing's words, is exact where a model's own words might not be.
6. **Two reviewers that are agents:** temper's, and vector's own automated review. **Gap, the forge's other actors:** a review from another reviewer on the forge doesn't reach the change's implementer as a repair; the policy asks each comment to be answered or dismissed with a reason.
7. **A person approves** on the forge. temper's own review is a first pass, not the landing's gate.
8. **Landing, as temper already does it:** vector lands through GitHub's merge queue, which runs the suites the pull request skipped. A branch whose rules require the forge's queue lands through it, temper's own queue deciding only what enters and when ([../domain/forge/queues.md](../domain/forge/queues.md), section 1). The pull request's green checks are only part of the evidence, as vector's own aggregate says.
9. **Catching up with `master`** by a merge, never a rebase or a force-push. **Gap, history:** the guide forbids rewriting a published branch, and the project's word must hold for temper's own pushes too.

### 3.4 An RFC-sized piece: a milestone at rung 3 (6.2)

1. **The RFC first.** The maintainer and the drafter write it in the template; the guide's design line says where and how (4.1). It lands as a goal of its own, the RFC alone, waiting a week and for three approvals under the floor's rule.
2. **Then its goals,** each `after` the RFC's. vector breaks an RFC's work into issues, which is the draft's "issues as goals" (6.1). It is wanted here now, not later.
3. **Each goal** cites the RFC as its lasting design and keeps the shapes its changes share in `.goal/contracts.md`. It lands as one pull request under 300 lines, sized so by the guide's line, through human review and the merge queue.
4. **Where the time goes:** a week of last call, and each pull request's reviewers. temper doesn't shorten either. What it saves is the RFC's draft, the checks before a person looks, and the repairs.

### 3.5 What vector shows

- Rung 3 works as drafted: the RFC is the record that stays, and `.goal/` holds the rest.
- The hard parts are the forge's: whose name is on a commit, a CLA, disclosure, approvals by people, other reviewers, no rewritten history.
- The forge's own merge queue already fits: temper lands through it.

## 4. click, stopped at the first step

- **Its organisation's rule:** "Using LLM or AI tools to generate PRs is not allowed. They are closed on sight, without review or discussion. This applies regardless of code quality or correctness" ([Pallets' pull request guide](https://palletsprojects.com/contributing/pr/)), which click's [docs/contributing](https://github.com/pallets/click/blob/08b803cf32ceb2ae6a772c62d5da42757a897d44/docs/contributing.md) defers to.
- **What its owner could do:** change their organisation's policy first, or keep temper to a copy of their own where nothing reaches upstream. The draft offers neither, and temper shouldn't hide the question. **Gap, the policy:** the setup reads it first, and says plainly what it allows before it proposes anything.
- **Two landing branches:** `main` for the next feature release, `stable` for fixes ([branches](https://github.com/pallets/click/branches)). **Gap, the base:** a piece of work's base among a repository's branches, a fix starting on `stable`.

## 5. The gaps

| Gap | Seen in | Where the draft takes it |
|---|---|---|
| The project's policy on AI, read first and said plainly | click, vector | 7.5; 7.3's first step |
| What each change carries: the pull request template filled, a changelog entry, by the guide and checked by the hook | vector, hono | 7.5 |
| Whose name is on a commit, for a CLA, and the models named by code | vector | 7.5; questions 7 and 8 |
| Approvals by people as a way's landing; the forge's merge queue already fits | vector | 7.5 |
| The forge's other actors: a bot pushing to temper's branch, another reviewer's review | hono, vector | 7.5; question 9 |
| No rewritten history on a published branch | vector | 7.5 |
| The worker's tools | hono, vector | 7.1; [any-project.md](any-project.md), section 11 |
| A piece of work's base among a repository's branches | click | [any-project.md](any-project.md), section 11 |
| Issues as goals, wanted now | vector, hono | [any-project.md](any-project.md), section 11, moved up |
