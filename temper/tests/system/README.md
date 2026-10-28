# temper's composed system world

`composed::World` drives the actual engine root, actual worker root and
smith's actual agent domain over fake LLM IO. The store is jig's real store
over test files, and the authenticated person is jig's fake. skein's fake
forge serves both the engine's API and the worker's git operations from one
repository state. This follows
jig's `domain/testing.md`, sections 2 and 8, and temper's `domain/testing.md`,
section 2.1.

The world plays what the two sides' own worlds play. Between the roots it
is the channel and the protocol layers (`composed/link.rs`): jig's link
records cross as they are, since both roots speak the hub's vocabulary; an
assignment crosses with smith's charter encoded by smith's codec from
temper's charter translation, and temper's workspace items beside it; an
answer crosses with temper's parts, what landed and what was saved, in the
engine's terms. The engine's hand-outs are answered as its protocol layer
would: a relayed call decoded into the engine's tools, a decided answer
rendered as smith takes it, an ended run's outcome read back by smith's
result codec; accounts and core reads as the engine world plays them. The
worker's hand-outs are answered with smith's codecs, as the worker world
answers them.

Below the worker the world is the agent process (`composed/process.rs`):
smith's agent world behind the worker's process face, its directories
mounted from the trees the worker prepared. The fake LLM scripts only
completion contents: smith emits its own calls, turns, waits and answer.
Calls and turns cross smith's process host, the worker's hub, the engine
and its journal before returning.

Focused cases cover a committed chat report, a durable host-call reply,
live human words, cancellation, a read-only adopted repository prepared by
the worker, park/resume from the committed concrete smith prefix, and
withheld store acknowledgements. Assertions independently read durable
task and call records and native smith observations. The worker's answer
must carry smith's actual result, turns and cumulative spend; every turn the
worker sends is kept by the engine, and the process is reaped.

The narrower fixtures (`world`, and the `forge` and `goals` stories) drive
the engine root and smith's agent directly, the hub's records played by the
engine world's protocol layer, and keep the forge and goal checks.

The fuzzy sweep runs 24 seeds twice, comparing durable rows, concrete turns
and crossing observations. Seeds vary fake LLM timing, store commit delay,
host calls and repository preparation. The composed clock advances one
millisecond per iteration; each smith iteration yields to the roots. Both
root configurations must admit their declared worst-case bounds.
