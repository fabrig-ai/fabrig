# A provider's outage is not a run's failure

Draft, 2026-11-13. A model provider unreachable for two minutes held two
goals until a person released them, with nothing in the hold saying why.
This draft has such an outage wait like any infrastructure failure, and
the hold name it.

## 1. What happened

- **The provider's edge refused every call** for about two minutes: each
  call from one worker got HTTP 403 with an edge's HTML page ("Attention
  Required"), not the provider's own JSON error. Calls a few minutes
  later were answered as usual.
- **smith read it as the run's own failure.** The program backend's
  `result_failure`
  (`smith/crates/smith-protocol-program/src/backend/conversation.rs`)
  maps an `is_error` result by its `api_error_status`: 401 to
  `SignedOut`, 429 to `Exhausted`, and any other status, 403 and every
  5xx included, to `Invalid`. The run ended failed, its failure's
  evidence `unsent`: no call reached the model.
- **The task spent its tries in seconds.** As the run's own failure it
  had the ordinary bound, its backoff from 1 second up to 30
  (jig's `domain/tasks/lifecycle.md`, *Tries, within a task*), so two
  goals' coordinators each failed four attempts in about 90 seconds and
  were held `out of tries: run failures`.
- **The hold said nothing.** Its words were empty. Finding the cause took
  the engine's log, then the worker's trace, then the program's own
  session file on the worker.

jig's `domain/tasks/lifecycle.md` already has the class for this: an *infrastructure*
failure, "a forge or a network down for minutes", waits longer within a
bound of its own (12 tries, 10 seconds doubling to 5 minutes, about half
an hour) and holds with its last failure's reason as its words. A
provider unreachable is that kind of failure.

## 2. Wanted, ranked

1. **An unreachable provider is unavailable, not invalid.** A failed
   result whose call reached no model counts as the provider being
   unavailable or overloaded: a 403 whose body is not the provider's own
   error (an edge's page), any 5xx, and 529 (overloaded). In
   `result_failure` they become `Unavailable` or `Overloaded`, skein's
   `Failure` (`skein/crates/skein-llm/src/types.rs`), not `Invalid`. A
   403 carrying the provider's own error stays as it is. The direct
   backends follow the same rule where they differ.
2. **Such a failure is an infrastructure failure.** A run ended because
   its model was unavailable or overloaded, with evidence `unsent`,
   counts as *infrastructure* in jig's `domain/tasks/lifecycle.md`, 6: its own tries, its
   longer backoff, none of the run's own tries spent. An outage of a few
   minutes then holds no task; one of half an hour still does.
   That section names it in the class's list.
3. **The hold names the failure, from structured fields only.** A task
   held for failures whose last failure was a model's names it in the
   hold's words: the endpoint, the model, the failure's kind, the
   evidence, and the HTTP status where one came, for instance
   `anthropic claude-opus-5-5: unavailable, status 403, unsent`. No
   provider text, body or tool output goes into the words, so nothing a
   provider echoed reaches a reader of the hold
   (`../domain/worker/README.md`, *A hold naming an agent run's last
   failure*). That open question is settled for this structured case
   (the user, 2026-11-13): its entry says so, and free text stays out
   as it stands.

## 3. Out

- How the program labels the error, and why the provider's edge refused.
- The retry bounds' figures; every other failure class.
- Free-text failure reasons in holds.

## 4. Tests

Cheap, focused and fuzzy suites only; walk = false.

- smith: `result_failure` for a 403 with an edge's body, a 403 with the
  provider's own error, a 500 and a 529, each to its failure; 401 and 429
  as today.
- jig or temper's domain, wherever runs' failures are classed: a run
  ended `Unavailable` with evidence `unsent` is retried as an
  infrastructure failure and spends none of the run's own tries; past
  its bound the task is held with words naming the endpoint, model, kind,
  evidence and status, and no provider text.
- Run only the new and affected tests by name.

## 5. Formats

If the hold's words or a failure's record need a field no store keeps
today, the store's version moves and its patch comes with it: upgrades
are supported (`redeploys.md`).
