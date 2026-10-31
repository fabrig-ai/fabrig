<!-- Saved by agent-handover-coordination on 2026-10-31 at the request of
fabrig-handover-execution. Source: the user's message of
2026-10-31T02:13:36Z in session 936a6a50 (greeting-session-start), where the
user pasted this kickoff. Text below is verbatim from that paste.
Later user decision (fab/handover.md, "Decided by the user (10-31)"): the
api gate is OUT of scope for this run; the Rust gates are files, ratchets
(old_forms) and size-as-flag only. -->

# Kickoff: fab's first fabrig run, "skein component kit, phase 1"

From the user, 2026-10-31. You drive fab (agents repository, fab/, docs/design/fab.md). This prompt starts fab's first run on fabrig. Read it whole before acting; it is self-contained.

1. Why this run
Purpose: prove fab on Rust and on fabrig, not on fab's own Python. The run should exercise:
skeleton calls that write Rust;
Rust gates and ratchets;
crate-graph ordering;
landing through fabrig's queue (fab/queue.py on hetzner8).
It should also produce a quality result that can be measured without judgement.
Subject: the first slice of work from the skein code-quality review of 2026-10-31. That review is a separate document; ask the user for its path if you don't have it. This run uses only the parts named in section 4 below, and everything it needs from the review is quoted here.
Mode: mostly Migration, with one Feature contract part, one Feature part and one Fix part. It is not a test of Feature mode's flow or slices. A Feature draft (temper/docs/design/draft/landing-model.md, slice 1) comes after this run.
2. Entry criteria: check these before starting anything on fabrig

Check each and record the result as a note in the log. If one is unmet, design and build it as fab parts in docs/plan/agents-rebuild, through the loop, not by hand. Start the fabrig run only when all hold.

lean-briefs and part-measures have landed and have run on at least three finished fab parts. The code report shows skeleton share and Claude to Codex weight per part.
The designer's calls are live (docs/design/fab.md 13.3): the cut runs as a call, not by hand. Your own session is not the designer for this run.
Calls and the foreman can land in fabrig. Skeleton, contract, review, answer and foreman branches for a fabrig plan reach fabrig's main through its merge queue (today section 14 of fab.md lists both as Later).
Rust gates, minimal set, run on every fabrig branch of this plan before ready:
files: changed paths are within the task's globs;
api: the touched crates' public items against the skeleton's, read from rustdoc JSON or cargo public-api, through heavy. An unplanned public item is red; it needs a skeleton amendment;
ratchets: the counts in section 5 may only fall, using fabrig's existing old-forms mechanism (<tree>/scripts/old_forms.py and old-forms.txt, lowered by the queue);
size: new or grown functions over 60 lines are flagged in the digest, not red, for this run.
A Rust clone detector is welcome but not required; the ratchets stand in for it.
A Rust digest for each branch: public items added or changed, new modules, changed lines by production, tests and docs, functions over 60 lines, ratchet deltas.
Your session is measured. Its transcript's weight is counted in the run's Claude total from the start, and you step in only when the loop is blocked.
3. The draft
Drafts are the user's: agents never write into docs/design/draft/. Write the proposed draft as component-kit.md with the content of sections 4 to 6 below, in fabrig's house style. Give the user its path, and the user lands it in skein/docs/design/draft/.
Once it lands, start the cut as a call (fab call cut) with this draft and the review as sources, plan directory skein/docs/plan/component-kit in fabrig (or wherever fab's plans for fabrig live by then).
4. Scope
Part	Mode	What
kit-contract	Feature, contract	lib's missing primitives and the io stage, designed and landed alone as a skeleton. lib: an earliest deadline helper (written privately in 4 crates and also in lib), Queue::clear and Stack::clear (today for _ in 0..len { pop } loops), one shared Lifecycle enum (identical copies in llm-connection and oauth-accounts), one growth rule (written 5 times in lib, each with its own GROWTH_FLOOR), slab iteration, hex and decimal parsing (HTTP's client and server decimal are identical). io: skein_io::stage::{up, down} as free functions, modelled on examples/echo/service/src/io_stage.rs, tested once in tests/io.
echo-first	Migration	One root end to end first: the echo example's host on the shared stage. This is slice 1.
stage-sites	Migration	The 39 io::resume call sites outside skein-io move to the shared stage, one crate per task, in crate-graph order (expand, move, contract). Only some of them check MAX_OUT before taking work today; after the move, every site does, as testing-strategy 6 requires.
lib-sites	Migration	The private copies of the lib primitives move to lib, one crate per task.
world-surfaces	Feature	Referee gets defaults for act, next_deadline and overdue (testing/skein-world/src/referee.rs:49-60), as starved and last_word have. Host::next_deadline becomes provided: the earlier of the mechanism deadline and the policy deadline (next_policy_deadline stays required). A forwarding Host implementation lets the about 10 test enum Process dispatchers shrink to one accessor. See the question in section 6.
abort	Fix	skein-forge-connection and skein-forge-setup's host take an Abort request, as programming-model 5.2 requires. The TLS close_notify is not part of this run.

Out of scope for this run, each a later run:

kv's store as a 5.4 machine;
the shared transport (theme 1a) and its design note;
JSON parse and navigation (1e);
the forge family's dialect and fake consolidation (theme 3);
test consolidation (theme 4);
the layout and lint decisions (D1, D2, D4, D5).

Expected size: about 25-35 tasks and a net reduction of a few thousand lines. The cut sets the real numbers.

5. Ratchets (counts may only fall)

Count these on main before the first task, and record them as the baseline:

io::resume sites outside skein-io (39 by the review);
private earliest/earlier helpers outside lib;
enum Lifecycle definitions outside lib;
GROWTH_FLOOR definitions;
for _ in 0.. pop loops over lib containers;
fn decimal and fn hex definitions outside lib;
test enum Process dispatchers that forward Host method by method.

The cut may add counts; it may not remove these. The run is done when each count reaches its target in the plan, zero where the part removes every copy.

6. Questions the cut must raise, not settle
The forwarding Host. impl<H: Host + ?Sized> Host for Box<H> is a generic item, and the programming model favours concrete composition over traits and generic items. Options:
(a) the generic impl, as an explicit, documented exception;
(b) a concrete delegate type;
(c) leave the dispatchers, taking only the defaults and the provided deadline.
Recommend one, and if you can measure it, make it a probe: the line count of each option on two of the dispatchers.
A "component kit" role in programming-model section 4 (shared mechanism between lib and components) is a change to the code's rules, and so is the user's to decide. Propose the wording; don't land it unasked.
Anything else that is hard to reverse (a public interface other trees use, a stored format) is the user's.
7. How the run proceeds
Mode presets:
Migration parts: codemod-sized tasks of 30-60 implementer minutes, one crate each, in crate-graph order, under the ratchets.
Feature parts: skeleton first. Signatures and short contracts, never the order of steps; docstrings under about a quarter of the part's expected lines.
The Fix part: Codex only, unless a gate wakes Claude.
The skeleton of kit-contract lands alone before any migration task, and its public API becomes the api gate's baseline.
Slices: S1 is echo-first (the echo host runs on the shared stage, its story test green on main). S2 is stage-sites and lib-sites done (ratchets at their targets). S3 is world-surfaces and abort. None is walked by QA. The after-landing tiers are the check.
Structure review and consolidation run on fab's cadence (about 10 landed tasks or 4 hours). They look in particular for new copies appearing while old ones are removed.
Answers in batches (fab's answerer thresholds); only the blocked task parks.
Your role: you watch the log and the reports. You do not design, write specs or fix by hand. A blocked loop is a fab task to fix, then the run resumes.
8. What to measure and report

Before the first task (baseline):

the ratchet counts;
production and test lines of the crates in scope;
the review's scorecard rows for those areas (Clean, Org, Taste, Naming, Prim, Elegance, Economy, Complexity).

During, in the code report:

per part: skeleton weight against its tasks' Codex weight, skeleton share, weight per landed production line;
the whole run's Claude to Codex weight, your session included, and shown apart;
questions per task and the foreman's share;
rejections by class, collateral rejections, ready-to-landed time;
time to the first implementer and to each slice.

At the end, one report to the user, in one page:

the ratchets and lines before and after;
the Claude to Codex weight, with and without your session, against the dynamic-memory run's 1.09;
skeleton share;
rework (lines rewritten within 7 days, reported when the 7 days have passed);
a re-review of the touched areas with the review's rubric, by a fresh read-only reviewer that does not see this run's plan, and the scorecard delta;
what fab had to change to get through, as task specs already landed or queued.
9. Stop and ask the user when
an entry criterion in section 2 cannot be met without a decision of the user's;
a ratchet would have to rise;
a task needs a change outside the scope of section 4;
the run's Claude to Codex weight, your session included, is still above 1.0 after the first 10 landed tasks.

In each case, report what you found and the options, with your recommendation.
