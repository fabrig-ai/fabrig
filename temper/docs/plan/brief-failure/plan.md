# brief-failure: a brief that can't be gathered says why, ends, and survives one section

From the first real goal on a deployment: a CI repair `Retrying`, `retry: refused`, count 25 and rising, with no reason anywhere and no end. The request (`inputs/request.md`) has the route, confirmed by reading: a forge read error falls to `BriefRead::Failed`, a required section's `Missing` fails the brief, `tag_brief` sends `PreparationFailed`, and `run::pause` has no bound.

## Parts and tasks

| Part | Task | Needs | What |
|---|---|---|---|
| `brief-words` | `brief-words` | | `Fate`, `Failure`: each ending of the brief child says why and how it counts |
| `retry-reason` | `retry-reason` | | the retry notice's reason, the counted event, the hold's words, `tasks-v9` |
| `forge-answer` | `forge-answer` | | the cause: a real answer as a fixture, by question; nothing waits for it |
| to come | `brief-counts`, `reason-shown`, `forge-sections` | | the core's routing, the client's wire and chip, the forge's words |

Critical path as it stands: `brief-words` and `retry-reason`, then `brief-counts`, then `forge-sections`.

jig is shared with hearth, whose manifests name no jig crate, so this change doesn't reach it. `skein-forge-client` isn't on hearth's surface.

## What was decided

- **A failure carries `Fate`: capacity, transient or permanent,** and its words. `jig/docs/design/domain/engine.md`, 9.1 names each.
- **Reasons are `&'static [u8]` in the kit's types.** "No LLM-written or secret words" then holds by construction.
