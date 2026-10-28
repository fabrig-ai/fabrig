# Review ideas from DeepPattern

Draft, 2026-10-23. Ideas worth keeping from DeepPattern's agent-quality
toolkit, read on 2026-10-23 when its makers invited the user to a trial,
which the user declined. The source is its public repositories,
`github.com/deeppatternai/agent-quality-gates` (AQG: skills, hooks and
rule blocks layered on another vendor's coding agent, MIT) and
`github.com/deeppatternai/decision-engine` (the client of a hosted panel
of reviewers; the panel itself is not public). Nothing was installed or
run. Nothing here is decided: each idea says where it would fit in
temper's design and what is open.

## 1. In one page

- **What temper already has.** DeepPattern offers evidence that a
  person or an agent inspects before an explicit accept, reject or
  person's decision. temper's gates are that, held as facts rather than
  claims: a verdict at the change's exact head, its freshness, blocking
  or advisory, a person's approval as a gate (`domain/forge.md`, 8.3).
  AQG asks an agent it does not control to write its own evidence;
  temper's procedures judge it from the forge.
- **Reviewers from different model families** (section 2): a review's
  verdict counts for more when its model shares no lineage with the
  producer's, and two families disagreeing on a change is where to look.
- **A disposition for each remark** (section 3): a repair says, for each
  of a gate's remarks, whether it acted on it or rejected it with a
  reason, and a review can ask a person what it cannot settle.
- **A lens for tests** (section 4): a review that judges whether a
  change's tests assert behaviour rather than shape, and whether they
  are deterministic.
- **Smaller ones** (section 5): a result that cannot be verified, said
  apart from one partly done; what a run chose not to do, said; a plan a
  person rearranges by hand.

## 2. Reviewers from different model families

The Decision Engine's premise: a model reviewing its own output, or
another trained on the same data, shares its blind spots, so their
agreement is weak evidence. Reviewers from different vendors are less
correlated: where they agree independently it is a signal, and where
they split, a person or a further review should look. The engine also
labels how real an agreement is, telling convergence on the substance
from convergence on a shared phrase; how it does so is on its server,
not public.

In temper, a gate's review is a task for an agent with a lens
(`domain/forge.md`, 8.3), and a task makes its LLM choice among those
its charter allows (`domain/agent.md`, 4.6). Two shapes would fit:

- **Another family than the producer's.** A review gate asks for a
  model whose provider differs from the one that produced the head it
  judges, choosing among its charter's others, and says so when the
  deployment has only one.
- **One lens, two families.** A gate's review runs on two models of
  different families. Both approving is the verdict; a split is a
  request for changes whose remarks name it, or is advisory, as the
  project chooses.

Open: whether the gain is worth a second provider's account and spend;
how a split is settled without a third opinion; what a deployment with
one provider, the first milestone's, does instead.

## 3. A disposition for each remark

AQG's adjudication turns each finding of a review into one of three:
accepted, rejected with a reason, or needing a person's decision, in a
table the next step reads.

In temper, a blocking review that asks for changes becomes a repair
given the gate's remarks (`domain/forge.md`, 8.4), and the gate is asked
again at the repair's head. Nothing records what the repair did with
each remark:

- **The repair answers each remark:** acted on, or rejected with its
  reason. The gate's next review reads the answers with the new diff, so
  a rejected remark is judged on its reason rather than raised afresh,
  and repairs stop circling on a remark the producer will not take.
- **A remark only a person can settle:** a review that cannot decide (a
  product choice, a trade-off beyond its lens) asks a person through a
  person task (jig's `domain/people.md`, section 8), rather than
  requesting changes a repair cannot make.

Open: where the answers live (the pull request's thread, the change's
state, the repair's result); whether a rejected remark the reviewer
insists on holds the change at once or counts against the repairs'
limit.

## 4. A lens for tests

AQG's test-quality review flags tests that assert a result's shape (its
type, that a key exists) rather than its behaviour (its value, its
effects, its errors); tests the change weakened, skipped or deleted;
and tests that depend on sleeps, the wall clock, unseeded randomness or
a live network.

In temper, it is a review's lens like memory safety or performance
(`domain/forge.md`, 8.3), in a project's landing rules or added to one
change by a coordinator. For temper's own repositories it would review
against skein's `testing-strategy.md` (worlds, seeds, the simulated
clock), so the lens's text is the project's, not a fixed list.

## 5. Smaller ones

- **Unverifiable is not partial.** AQG's closeout keeps `UNVERIFIABLE`,
  work that may be done but that no fresh check can confirm, apart from
  `PARTIAL`, some evidence and not all. A run's result could say so: a
  change whose behaviour no check in the repository covers, said in its
  result and its pull request rather than reported done.
- **What a run chose not to do.** The closeout names the actions
  deliberately not taken: no deploy, no secret touched, a file left
  alone. temper's authority already refuses what a task may not do; a
  run's result naming what it left out of scope would help the
  reviewers and a person reading the change.
- **A plan a person rearranges.** The Decision Engine's discussion
  board lays a topic out as cards; a person drags, ranks, strikes and
  annotates them, and the agent reads their intent from the edits
  rather than from text. People direct temper at the level of goals and
  plans (`README.md`, section 1), and a goal's plan shows in the client
  as a tree of its delegates (`client/README.md`, 5.8): editing that
  tree by hand, sent back as the person's answer, is a candidate for
  the client.
