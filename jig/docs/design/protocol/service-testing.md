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
  focused checks read the real, fake and refusing configurations through
  startup's readers, without network or a person.
  After the sign-in, the story prints a line at each change of the
  chat's run, and a run that can no longer answer (a failed or refused
  attempt, a hold, another ending) fails it at once in those words. A
  failed run prints its agent trace's failures (class, status, code)
  before the trace is erased. The test catches nothing
  (`temper/docs/design/draft/panics.md`, 5): the story's run and its
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
