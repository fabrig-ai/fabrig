# The core: its world

## 1. The world

Each child domain has a world of its own, its parent and neighbours
scripted in it, its expectations in a referee; authority, which keeps no
state, is tested by its step tests.

The core's world runs the core with every child beneath it, under a root
of jig's own for testing, against fakes that share none of its types
(../testing.md, section 4): jig's test connector, with resources, holds,
pools, priced effects, requirements, procedures and faults drawn from
the seed; one to three scripted workers and the engine's own slots,
whose runs play scripts; scripted parties; the store on skein-kv's
in-memory mode, slow or failing a commit; and the engine restarting
cold, at drawn moments or at one a scenario chooses.

**Every story ends with a cold restart,** the restart oracle (`restart.md`
): the engine restarts over the story's store, restores with every
link made, and reads back what the story wrote (the parties' documents,
the tasks' and the children's views), equal in their durable part to
their last answers before it. Each document type declares once, in the
one projection the oracle applies, the fields a restart rebuilds rather
than keeps, each with the step that makes it so: run-time observations
such as an account's last end, or a task's runtime on a host that died
with the engine. A field in doubt is durable; no story opts out but a
deliberately broken fixture; and a difference left after the projection
is a finding, a writer not keeping what its document shows. A link
that cannot be made refuses the restart (`restart.md`), and the oracle
fails the story naming the refusal's phase, record and link.
This is where the cross-record rules a restore once checked live, one
writer at a time: each child's rules are the oracle's to prove over
every writer of its records, never the restore's to judge.

Its stories: a conversation that answers, parks and resumes; a plan of
reports, a choice and procedure tasks, revised mid-way, and done; a
procedure's effect waiting for another connector's verdict, then made; a
delegate held past its tries, escalated to a person and released; a goal
cancelled with runs live and effects in flight, closing deepest first; a
tool call whose answer is lost with the channel and the engine, asked
again after a restart and answered from its record, made once; a run
handing a held resource to a procedure while it still holds the writer
slot; two tasks queued for a pool's last slot, and the pool shrinking
under them; an uncertain effect across restarts made before its retry
deadline; an effect of a kind that cannot be recovered, uncertain and
held for a person; a judge's facts changing between its verdict and the
effect; a policy narrowed while a proposal waits, an entry is committed
and a run is live; a note written, corrected by a person and recalled; a
recurring task across a period's reset; a person's words to a running
run; a conversation resumed from its current window after a compaction;
a message a run left unread, waking its task again as the run ends;
drift holding a task, and the client told why.

Its stories of what the core computes and serves: a task query watched
while tasks enter it, leave it, change its order and end, each delivery
after its commit, the ended ones then read through the ended indexes,
by requester and, for a listed task, by project, a filter past the
prefix coming back cut; what a task waits on moving from its delegates
to a party as a delegate asks its person a question, and back once it
is answered; a conversation's turns counted unread until its person
marks them seen, the conversation ended and read after a restart with
the position its person left, and one forgotten once read leaving
nothing that shows; `me` marked as a pool is spent from, deliveries
coalesced; a budget given `AsAllowed` answered with what was given.

Its stories of added dependencies (../tasks/lifecycle-endings.md, 1): an agent task given
a dependency while its run is live, the run finishing done and parked,
its activity naming what it waits for, woken when the dependency is
done and finishing again, after a restart too; an idle agent task kept,
a message to it waiting in its inbox and never waking it, then
delivered as it goes on; a kept task's run asking an effect that waits
for dependencies, refused; a run amending a delegate to wait for
another, and one whose add would close a circle, refused naming it; the
reordering of two tasks queued for one write hold, ended done in the
new order.

Its stories of agents and procedures doing what parties do: a run
amending a delegate a party amended since it was shown, answered moved
with what it is now, then amending again; a run releasing a held
delegate, naming its hold's revision, and making an uncertain effect
again; a run proposing an amendment and an effect beyond its authority,
each accepted by a person; a run subscribing to a connector's topic and
woken by its news; a run cancelling a delegate with a grace it finishes
within; a procedure amending, cancelling and messaging its delegates,
one amendment refused as moved and the procedure stepping again on what
it is told.

Its stories of settings (`settings.md`): an owner's change of a charter,
a template and a policy in one set, committed in one decision, then
`me` re-read with the template, the next `MakeProject` copying the
policy, a narrowed rule refusing the next effect of a task made before
it, and a restart restoring the same settings; a change refused as
stale, as out of shape, and by a connector judging an application's
kind, each committing nothing; a restart with other defaults, recorded
as the release's, a changed setting kept over them and a reset
returning one to the new default; a charter changed while its task's
run is live, the run finishing on its version and its delegates made on
the newest; a conversation moved at its next run, and one whose charter
was removed staying where it is; a change moving running work, and a
removed charter living while a task holds it, a delegation to it
refused as not offered; a restart with old revisions held and a run
adopted on its version; the bound reached, the change refused naming
the oldest revision's tasks, those moved with `MoveToNewest`, and the
change then made; a revision leaving memory as its last holder ends.
Its stories of a proposal of settings (`settings-proposals.md`, 1): one proposed by a run and
accepted by an owner, moving running work; one made stale by another
change and withdrawn, its proposer told.

Its stories of crossed proofs and closed windows (`executors-transcripts.md`, 1)
plant that evidence in the store (`jig/tests/core/tests/support/closed_proofs.rs`):
a task mid-run on a worker, its window resumed from an earlier attempt,
its proof crossed and its window closed, the worker
listing its run at its hello, the run stopped, the claim lost, its late
turn refused, its writer slot held until the connector reads afresh,
and its next attempt fresh with the closed window's tail; the same with
no worker listing it, lost once the grace has passed; an in-process
claim yet to commit a turn, settled at once as not started; a crash
before adoption, the next restart settling the claim the same way;
the next attempt claimed and the engine restarted before its first
turn, adopted as any claim; and a successor's proof crossed before
that turn, settling it again.

Its stories of the restore before the switch (`restart.md`) use
registry changes and declared fixture jobs: an upgraded copy restored
whole, its restores' commits in the copy, no secret served and no run
adopted before the switch, switched, and the manifest naming the
position then; a live set past limits lowered since, refused before
the switch, the source generation still active and unchanged, and a
start of the build before opening it as it was; a task naming a project
the copy does not hold, a link that cannot be made, refusing the start
before the switch with the task and the project named, the source
generation the same; a crash during the restore, the next start
upgrading another copy; and a start with nothing pending, whose switch
step switches nothing and commits the wires alone.

Its referee holds the core to ../model.md, section 11, seen from outside;
applications' worlds reuse it on jig's conformance world (../testing.md,
section 5).
