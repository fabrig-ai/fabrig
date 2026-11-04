# bugs-jig: the plan

The jig bugs the debt backfill routed (`inputs/bugs-jig.md`), cut for fab by one lead call
(2026-11-04). Low priority: small tasks, no reshaping. The standing rules are `notes/common.md`.

## Parts and tasks so far

| Part | Bug | Tasks | State |
|---|---|---|---|
| `source-name` | 6 | `source-name-refused` | released |
| `chat-status` | 4 | `chat-status-total` | released |
| `charter-rail` | 3 | `rail-jig`, then `rail-temper` | released |
| `close-grace` | 1 | `close-grace` | released |

Bugs 2, 5, 8 and 9 are under survey and come in later batches. Bug 7 is dropped: see below.

## Design written by this cut

- `jig/docs/design/domain/hosts.md`, 8: `modify` only to a run that can deliver.
- `temper/docs/design/domain/agent.md`, 4.3: jig's translation holds the rail, not temper's.
- `jig/docs/design/client/domain/client.md`, 19: the helper reads any chip.
- `jig/docs/design/protocol/client.md`, 3: the link's refusal of a chip that lacks what its phase
  names, which the code already does.
- `jig/docs/design/examples.md`, 7.3: an environment's close has the grace `ops` configures for it.

### Decisions

- **Bug 3 was a design disagreement:** temper's run is safe today, because
  `temper-engine-smith/src/start.rs` narrows `modify`; jig's `hosts.md`, 8 says a root decides none
  of the workspace tools, and no other root has the rail. The rail goes into
  `jig_protocol_charter::charter`, where `deliver` is decided, and temper's line goes. Both designs
  changed; `assignment` still says what the policy says.
- **Bug 4 stays, smaller than it was stated:** the link refuses both records as malformed, so the
  abort is past the link only. `chat_status` still becomes total, since it is a public helper over
  public types.
- **Bug 1 needs no template identity:** a close's grace is the request's (`people.md`), and jig's
  own `close_chat` reads the chat template offered now, not the one that made the chat
  (`client.md`, 19). So the deployment's close names its grace from `ops`'s configuration, beside
  its reason, and the root stops reading templates. Carrying identity was 100-odd literal sites and
  the client, people and tasks formats; matching the chat's `label` kept the root's decision, needed
  a change to `engine.md`'s "No decision reads one", and left a shared label wrong. A chat's own
  template's grace is honoured by no close; giving a task its grace as a value would do it, and is a
  stored format, so it is the user's, and not cut here.
- **Bug 7 is dropped:** `d8e8eb62a8` gave `Resources` a `Debug` of counts, with tests.

## Overlaps

- `chat-status-total` edits one arm of `jig-client-view/src/words/words/status.rs`, inside the
  glob of debt-3's `jig-ui-callers`.
- `temper/docs/design/domain/agent.md` is also edited on `fabrig-scale/integration`, in other
  hunks than 4.3.

## Critical path

Each part stands alone; `rail-jig` then `rail-temper` is the longest so far.
