# The gallery's files

Draft, 2026-11-27, part of [ways.md](ways.md): each entry of the gallery ([ways-recipes.md](ways-recipes.md), section 5) as the whole files a project would hold, written by [ways-recipes.md](ways-recipes.md), sections 1 to 3, [ways.md](ways.md), 5.3, and [ways-temper-dir.md](ways-temper-dir.md), so that each is checked against the format before it is built, and built as a fixture and a world test (`ways.execution.md`, W9). Each way's charters are files of the project's in `.temper/charters/`, written whole from the stock's when the way is picked ([ways-charters.md](ways-charters.md), section 2): `implementer`, `reviewer`, `goal`, `pair` and the ones an entry names. A model an entry names must be one the deployment's catalogue holds and the charter's `settings` charter offers, or it is ignored ([ways-recipes.md](ways-recipes.md), section 2); the fixtures set both. An entry's "no review" holds where the floor asks none: a project whose floor still has today's rule 1, with its gate, keeps that review whatever a way says ([ways.md](ways.md), 5.5).

## 1. The stock three

### 1.1 Pair

You and one agent in its own copy, saved as it goes; when it delivers checked work, "Land this" opens its change, by code. No review: you're there, and the agent can open a sub-agent for a review in place when you ask. Needs [pair.md](pair.md)'s goal, released by W9 once it lands; its model choice is W6's. It adds no gate: a floor's review still holds.

```toml
# .temper/ways/pair.toml
format = 1
headline = "You and one agent, in its own copy"
line = "You work with one agent in its own copy, saved as it goes; when it delivers, you say Land this."
start = "pair"

[tasks.pair]
executor = { agent = "pair", model = "claude-opus-5-5", effort = "high" }
contract = "report"
conversation = true
holds = ["copy"]
delegates = { land = {} }

[tasks.land]
executor = "change"
into = "base"
producer = "handed"
gates = []
landing = { ci = true, up_to_date = true }

[reaches]
```

### 1.2 Hand it over

You describe it, an agent writes it, another reviews it, and CI and code land it.

```toml
# .temper/ways/hand-it-over.toml
format = 1
headline = "An agent writes it, another reviews it, and code lands it"
line = "You describe the change; an agent writes it, another reviews it, and it lands when CI passes."
start = "change"

[tasks.change]
executor = "change"
into = "base"
producer = "build"
gates = ["review"]
questions = "up"
landing = { ci = true, up_to_date = true }

[tasks.build]
executor = { agent = "implementer" }
contract = "change"

[tasks.review]
executor = { agent = "reviewer" }
contract = "verdict"

[reaches]
wording = "decide"
retry = "decide"
tests = "decide"
implementation = "ask"
dependency = "ask"
departure = "ask"
findings = "ask"

[[reaches.narrow]]
kind = "findings"
words = "a repair goal for a routine red after landing"
class = "decide"
```

### 1.3 Milestone

Design first; each goal is built on its own branch and lands when it's done, as Draft first lands today.

```toml
# .temper/ways/milestone.toml
format = 1
headline = "Design first; each goal is built on its own branch, and lands when it's done"
line = "You and an agent write a design; agents build each goal on its own branch, and each lands when it's done."
start = "milestone"

[tasks.milestone]
executor = "gathering"
gathers = "goal"

[tasks.goal]
executor = { agent = "goal" }
contract = "report"
holds = ["branch"]
lands = "whole"
into = "base"
gates = []
landing = { ci = true, up_to_date = true }
delegates = { change = { at_most = 40 } }

[tasks.change]
executor = "change"
into = "parent"
producer = "build"
gates = []
landing = { ci = false, up_to_date = false }

[tasks.build]
executor = { agent = "implementer" }
contract = "change"

[reaches]
wording = "decide"
retry = "decide"
tests = "decide"
implementation = "ask"
dependency = "ask"
departure = "ask"
findings = "ask"

[[reaches.narrow]]
kind = "findings"
words = "a repair goal for a routine red after landing"
class = "decide"
```

## 2. Ways of their own

### 2.1 Owner-approved milestone

As Milestone, each goal's branch landing on an owner's approval. One milestone asks for two owners, set at its start, the work's own layer ([ways.md](ways.md), 5.4).

```toml
# .temper/ways/owner-approved-milestone.toml
format = 1
headline = "Design first; each goal lands when an owner approves it"
start = "milestone"

[tasks.milestone]
executor = "gathering"
gathers = "goal"

[tasks.goal]
contract = "report"
holds = ["branch"]
lands = "whole"
into = "base"
gates = []
landing = { ci = true, up_to_date = true, approvals = [{ role = "owner", people = 1, freshness = "clean" }] }
delegates = { change = { at_most = 40 } }

[tasks.goal.executor]
agent = "goal"                      # the goal charter whole: designer on its first run, coordinator after
model = "claude-opus-5-5"
effort = "xhigh"

[tasks.change]
executor = "change"
into = "parent"
producer = "build"
gates = ["review"]
landing = { ci = false, up_to_date = false }

[tasks.build]
executor = { agent = "implementer", model = "gpt-6.2-sol", effort = "high" }
contract = "change"

[tasks.review]
executor = { agent = "reviewer" }
contract = "verdict"
```

### 2.2 Second opinion

The stock review and a second one, by a lens of its own, with a charter of the project's, `.temper/charters/data-safety-reviewer.md`:

```
+++
format = 1
settings = "reviewer"
+++
You review a change for data safety alone: what it stores, migrates, deletes or exposes ...
```

```toml
# .temper/ways/second-opinion.toml
format = 1
headline = "Two reviews: the usual one and one for data safety"
start = "change"

[tasks.change]
executor = "change"
into = "base"
producer = "build"
gates = ["review", "safety"]
landing = { ci = true, up_to_date = true }

[tasks.build]
executor = { agent = "implementer" }
contract = "change"

[tasks.review]
executor = { agent = "reviewer" }
contract = "verdict"

[tasks.safety]
executor = { agent = "data-safety-reviewer", model = "claude-opus-5-5", effort = "xhigh" }
contract = "verdict"
lens = "data safety"
freshness = "exact"
```

### 2.3 Hotfix

Straight into main, first among the work waiting in its queue (a turn already running finishes first), at Ultrafast, decided now and told after, with CI still required. Needs gpt-6.2-sol with its Ultrafast speed in the catalogue (`model-speeds.md`).

```toml
# .temper/ways/hotfix.toml
format = 1
headline = "A fix, fast: first in line, decided now, told after"
start = "change"

[tasks.change]
executor = "change"
into = "base"
producer = "build"
priority = 0
gates = []
landing = { ci = true, up_to_date = true }

[tasks.build]
executor = { agent = "implementer", model = "gpt-6.2-sol", effort = "high", speed = "ultrafast" }
contract = "change"

[reaches]
wording = "decide"
retry = "decide"
tests = "decide"
implementation = "decide"
dependency = "decide"
departure = "decide"
findings = "decide"
```

### 2.4 Night batch

Many small goals on a cheaper model, at low priority, their branches landing in turns of up to four as they're ready, in a project that lands by turn with a `batch` of at least four; nothing schedules it by the clock.

```toml
# .temper/ways/night-batch.toml
format = 1
headline = "A batch of small goals, landing together in turns"
start = "batch"

[tasks.batch]
executor = "gathering"
gathers = "goal"

[tasks.goal]
executor = { agent = "goal", model = "claude-sonnet-5-5", effort = "medium" }
contract = "report"
holds = ["branch"]
lands = "whole"
into = "base"
priority = 900
deadline = 259200
gates = []
landing = { ci = true, up_to_date = true }
share = { with = ["night-batch"], at_most = 4, combine = "squash" }
delegates = { change = { at_most = 10 } }

[tasks.change]
executor = "change"
into = "parent"
producer = "build"
gates = []
landing = { ci = false, up_to_date = false }

[tasks.build]
executor = { agent = "implementer", model = "gpt-6.2-sol", effort = "medium" }
contract = "change"
```

### 2.5 Careful newcomer

Every decision waits for the person who asked, away or not.

```toml
# .temper/ways/careful-newcomer.toml
format = 1
headline = "Everything asks you first"
start = "change"

[tasks.change]
executor = "change"
into = "base"
producer = "build"
gates = ["review"]
questions = "up"
landing = { ci = true, up_to_date = true }

[tasks.build]
executor = { agent = "implementer" }
contract = "change"

[tasks.review]
executor = { agent = "reviewer" }
contract = "verdict"

[reaches]
wording = "ask-away"
retry = "ask-away"
tests = "ask-away"
implementation = "ask-away"
dependency = "ask-away"
departure = "ask-away"
findings = "ask-away"
```

### 2.6 One-landing milestone

A milestone on a branch of its own: each goal lands into it, the branch catches up with main whenever main moves, and the whole lands at once, on an owner's approval. Needs W13: a gathering holding a branch, with a closer of its own landing it into the gathering's `into`.

```toml
# .temper/ways/one-landing-milestone.toml
format = 1
headline = "One big landing: goals gather on the milestone's branch"
start = "milestone"

[tasks.milestone]
executor = "gathering"
gathers = "goal"
holds = ["branch"]
lands = "whole"
into = "base"
gates = []
catch_up = "after-each-landing"
landing = { ci = true, up_to_date = true, approvals = [{ role = "owner", people = 1, freshness = "clean" }] }

[tasks.goal]
executor = { agent = "goal" }
contract = "report"
holds = ["branch"]
lands = "whole"
into = "parent"
gates = ["review"]
landing = { ci = true, up_to_date = true }
delegates = { change = { at_most = 40 } }

[tasks.change]
executor = "change"
into = "parent"
producer = "build"
gates = []
landing = { ci = false, up_to_date = false }

[tasks.build]
executor = { agent = "implementer" }
contract = "change"

[tasks.review]
executor = { agent = "reviewer" }
contract = "verdict"
```

## 3. Not ways

### 3.1 Docs fast lane, a default across ways

A change touching only documentation skips CI; any other waits for it. It lives in `ways.toml`'s `[defaults]` ([ways.md](ways.md), 5.3): its rule joins every change's criteria, and no way lifts it. Two things make it work: the floor mustn't ask CI itself, so the project's owners take CI out of its rule 1, a settings decision; and the offered ways leave `ci` out of their changes into the landing branch, inheriting the default's `false`, where the stock files above set it.

```toml
# .temper/ways.toml
format = 1
offer = ["hand-it-over", "milestone"]

[default]
change = "hand-it-over"
milestone = "milestone"

[[defaults.sets]]
name = "docs"
patterns = ["**/*.md", "docs/**"]

[defaults.landing.base]
ci = false                          # ways leave ci out, and inherit this
up_to_date = true

[[defaults.rules]]
unless_only = ["docs"]
ci = true
```

### 3.2 Sensitive paths, the floor's

Whatever the way, and whatever a landing says, a change touching these paths waits for two owners. It is the floor's, a project's kind 2 rule and kind 8 path set in settings, set by its owners with `temper ctl settings apply` (`../client/ctl/tools-acts.md`, section 1), shown here loosely in TOML. A project that accepts that a landing may change it writes the same as a `[[defaults.rules]]` with `sets`.

```toml
# settings, not .temper/: the project's classes and its rule
[[classes.sets]]
number = 2
name = "auth"
patterns = ["auth/**", ".temper/hooks/**"]

[[landing_rules]]
pattern = "*"
enforce = true
approvals = [{ role = "owner", people = 2, freshness = "clean" }]
sets = [2]
```

### 3.3 Standing triage, a grant

A standing triager starts Hand it over for small findings, and milestones only through its person ([ways.md](ways.md), 7.3). Its grant is the store's, given at `temper ctl role start`, never a file: the ways it may start, `hand-it-over`; the way it may propose to its person, `milestone`, made as theirs on acceptance; and no value it may set. "Small findings" is its charter's guidance, not a rule temper checks.
