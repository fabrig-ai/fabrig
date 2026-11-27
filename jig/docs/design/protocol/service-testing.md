# The engine's and the worker's services: testing

## 1. Testing

Each within jig's suites' budgets.

- **The domain worlds** run without any of this (`../domain/testing.md`).
- **The components' worlds** test each component alone (`channel/testing.md`,
  section 1; `llm.md`, section 8; `store/testing.md`, section 1; `client/testing.md`,
  section 1), and the default HTTP mapping's engine end and its guards
  in their step tests (`browser/testing.md`).
- **The protocol world:** the reference service (`service.md`, section 7), every
  component in it, over skein's simulated io (skein/docs/foundation/testing-strategy.md, 2.7):
  a store on the simulator's crashing disk, scripted parties'
  connections to its server, through jig's default HTTP mapping, skein's
  fake LLM peer, smith's agent processes over skein's
  fake LLM
  spawned on skein's fake machine beneath the test worker (smith's
  `protocol/host.md`, section 7), and the application's systems' fakes,
  the process killed and started again at drawn moments, and drained by
  a shutdown. Its referee is the conformance world's, seen through bytes:
  nothing a party, a host or a system saw depended on a commit that was
  lost; and a drained shutdown ends with nothing waiting for a commit.
- **Drawn turns in the protocol world's chat**
  (`temper/docs/design/draft/misbehaving-agents.md`, 5.2): its answered
  chat keeps its story, one person's chat answered by the in-process
  worker and the answer seen, and plays a model's own turns inside it,
  in stretches (skein's `testing/fakes/drawn-agent.md`, 5).
  - **Its script:** the agent delegates one task to the person, then
    plays a stretch, cancels that task, plays a stretch, and finishes.
    A stretch is a bounded number of rounds, the world's round count,
    of `message` and `subscribe` alone, so drawn turns never deliver,
    finish, cancel or amend. The chat's role holds the delegate,
    message, control and subscribe families in this story alone, so
    the world's other stories keep their requests and their cuts.
  - **Its menu:** bodies of both tools' fields, each in the words,
    question and answer forms, naming a task by a slot over creation
    replies (`Delegated tasks: T at`) and an answer's question by one
    over `Sent message N`, each with a never-issued value. The fake
    draws by request from the parts of the world's seed (skein's
    `testing/sweeps.md`, 2: `agent/model`, and `llm` of the faults
    and the scheduling).
  - **What a stretch adds to the story's checks:** each drawn call
    answered once, in the conversation the fake last saw, and none of
    another tool; and each refusal its draws can reach counted, at
    least once at the story's seed: a task never issued (`Refused;
    missing reference`, and the subscription's refusal), and the
    cancelled task as an ended recipient. The world classifies each
    drawn target as live, ended or never issued, beside its answer.
  - **What it can't reach,** and why: an answered question, since no
    party here asks the chat a question; and `read_task`, which the
    reference service doesn't declare (`service.md`, section 7).
    temper's composed workload draws its producer's `read_task`
    (temper's `testing/workloads.md`, 5).
- **End to end** (skein/docs/foundation/testing-strategy.md, 2.9): the reference binary,
  `jig-test`, started on a scratch directory with skein's fake LLM peer
  as a process of its own on loopback (skein's `testing/fakes/llm.md`) as its
  endpoint, a person's requests answered through a native client on
  loopback, over jig's default HTTP mapping,
  runs taken by smith's real agent processes, then the binary killed and
  restarted, and the work found where it was. The process tier has three
  paths through `main`: that restart story and a binary told to stop
  with a sign-in under way, each through the real loop, whose shared
  `Deadline` classifies unanswered waits under measured starvation; and
  a table of its agent modes (`service.md`, section 7), with no loop, a
  row each: the relay's usage, a program child refused with no parent,
  and a program child running a scripted Claude Code's `auth status`,
  signed in, whose output smith reads as its status check does. The
  child hands the program's exit and answer on as they are, so what
  another status reads as is smith's status reading's alone (smith's
  `protocol/programs.md`, 3). The fixture bundle's page under the
  reference policy is the reference server's step tests' and the
  bundle reader's, in `jig-test-service` and `jig-test`, and the
  shipped bundle booting from the printed link is the browser tier's.
  A binary that ends before its story asks it to fails the story with
  its exit status, the last step the story saw and the end of its
  stderr, read to its end or for a bounded while, never a sign-in's
  address, in words a step test of `jig-test` holds. Account sign-in,
  refresh, sign-out and moves are covered in their lower worlds.
- **Startup's reads, as steps:** `jig-test`'s startup reads its
  configuration, its agent's document and the private files the
  configuration names through one face: the files themselves in the
  binary, documents and answers its step tests supply in
  `jig-test::step`. The agent's document takes its trust roots from the
  same face, through smith's document reader (smith's
  `shell/README.md`, 2.6). The step tests hold that a worker's secret
  refused as unsafe ends the start, naming the file and the refusal and
  none of its bytes, before any root or descriptor is opened; and that
  the live layer's real, fake and refusing configurations are ones
  startup accepts. The native private-file reader's refusals (group
  bits, a leaf link, a linked directory) stay in the real loop, in
  `jig-test::real`.
- **The live layer:** `jig-test` with a real LLM account, a key added
  by a request, a sign-in made as the test starts, or a program's login
  on its test worker's host (`llm.md`, 5.4 and 5.5), a native client
  signing in by the printed link, and a request answered by a real
  model, within a bound on calls. No record is read from a file.
  Run `scripts/live-sign-in.sh` for a fresh ChatGPT sign-in, a model answer
  and confirmed sign-out; it prints the browser link and the SSH tunnel
  needed for the fixed loopback callback on port 1455, and however the
  run ends, the sign-in's record does not outlive it.
  The live story declares low effort on the engine's and agent's models;
  its real, fake and refusing configurations are checked as startup
  reads them, without a file, a network or a person (above).
  After the sign-in, the story prints a line at each change of the
  chat's run, and a run that can no longer answer (a failed or refused
  attempt, a hold, another ending) fails it at once in those words. A
  failed run prints its agent trace's failures (class, status, code)
  before the trace is erased. The test catches nothing
  (`temper/docs/design/draft/archive/panics.md`, 5): the story's run and its
  final check return their findings, and on one the test prints those
  failures and its wall time, erases the records and traces and keeps
  the rest of its scratch, as it does now, then fails. Whatever else
  ends it, a panic of our own code or a signal, the script, once the
  test process has ended (on a signal, having stopped the runner with
  TERM and reaped it), erases the records and traces under its scratch,
  keeps the rest on a failure and removes it on a pass. The refused-plan run-failure words are
  checked by the focused step test
  `live_sign_in::tests::a_refused_plan_is_reported_with_the_run_failure`.
- **The browser tier:** `jig-test` serving jig's test application's
  bundle and the mapping at `/jig`, and Chromium, children of the
  harness's loop, a person in a page, and jig's attack stories against
  its server (`browser/testing.md`); a suite of its own, outside the
  default and fuzzy suites.
