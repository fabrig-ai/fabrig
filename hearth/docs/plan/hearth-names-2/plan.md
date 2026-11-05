# hearth-names-2: the plan

The last details of a real deployment leave hearth's files and one skein transcript, by the request (`inputs/request.md`). Only example values change, and this plan spells none of the old ones.

## Parts and tasks

| Part | Task | What | Needs |
|---|---|---|---|
| `hearth-values` | `hearth-values` | items 1 to 4: the design, the fixtures and the goldens | none |
| `forgejo-transcript` | `forgejo-port` | item 5: the captured transcript in skein | none |

Both go out at once. **Critical path:** `hearth-values` alone, about 40 implementer-minutes, then the trunk's end check.

## What was decided

- **Two tasks, because a branch stays in its tree.** Within hearth one task: `render.md`, 9.2, the fixtures, the goldens and the literal in `hearth/tests/builder/tests/suite/goldens.rs` say one thing, so they change together.
- **The task writes the design, not the cut.** The design's text changes only in its example values and the requested removals, so nothing here is design work to do first. The task states its departure from the implementer's text, including the one line of a draft the request names.
- **Three symlinks hold the zone, and no text search finds them:** the fixture `tasks/locale/tree/etc/localtime` and two `rendered/` witnesses. The task names them and its acceptance checks their targets.
- **Two chance hits stay:** a package's SHA256 in `hearth/testing/mirror/universe` holds the port's digits, and a float in `smith/benchmarks/summaries/` holds the forge port's. Both tasks replace and search with word boundaries.
- **The area follows the zone only where it's the fleet's:** the two `doc` strings in `format.md`, 5.3 and `locale.json` name an area and a zone in it, so both change. A bare area word in hearth's other fixtures isn't the zone and stays.
- **Beyond the request's list, and why:**
  - The bullets in `hearth/docs/design/domain/store.md`, 12 and `hearth/docs/design/README.md`, 9 go with the one in `format.md`, 13. The three were added together to hold this one question, and left alone two of them would ask what the request answers. Documents agree with one another (`DESIGN.md`, 1).
  - `build.md`, 14 and `format.md`, 3.7 and 5.3 hold the zone in lines the request's list doesn't point at, which its "everywhere under `hearth/`" covers.
  - The `.expect`'s header says its bytes are as captured but for the host and the chunk size, which stops being true (`AGENTS.md`, 2), so it gains the port.
- **No status file changes:** none names any of these, and none says a gap closes.
- **Memory ceilings stay:** every new value is shorter or the same length. A ceiling moves only on a measurement, and the task says how.
- **Goldens bless from `hearth-builder-world`,** the only world that calls `bless`, and only the fleet's test is blessed.

## Overlaps with other drafts

- `debt-21/hearth-builder-switch` owns `hearth/tests/builder/tests/suite/goldens.rs` and changes one line there, not the literal this plan changes. This overlap is unavoidable, since that file holds the literal.
- `debt-21/hearth-kit-render` moves `golden.rs` into `hearth/testing/hearth-test-render/` and leaves the fleet's files alone. The task's bless step says what to do if it has landed.
- `forge-no-registration` is still being cut, so its files are unknown.
- No other listed task touches the `hearth/docs/design/` files this plan edits, or `skein/tests/http/`. The queue plans list no files.

## Left, and put to the user

The question `container-name`: one container's name stays in `format.md`, 3.5, since the request says nothing else changes. No task waits for the answer.
