# hearth-names: the plan

hearth's documents and fixtures stop naming the real machine layout, the hosting providers and the VPN key, by the request's mapping (`inputs/request.md`). Only example names change.

## Parts and tasks

| Part | Task | What | Needs |
|---|---|---|---|
| `names` | the cut's own commit | `hearth/docs/design/`: the sentences and the mapping (`notes/names.md`, 3) | none |
| `names` | `fixtures` | hearth's tests, fixtures and goldens: the mapping, by script | none |

**Critical path:** `fixtures` alone, about 60 implementer-minutes, then the trunk's end check.

## What was decided

- **The cut writes the design, one task does the rest.** Which sentences describe the deployment is a judgement, and `notes/names.md`, 3 records each one, as the request asks. The fixtures and goldens must change together, since the builder and render worlds read one set, so they stay one task.
- **The plan's own directory is outside the search.** Its files spell the old names to say what changes, and they leave main when the plan retires. Until then the search excludes `hearth/docs/plan/hearth-names`.
- **The search misses `RHI_PROD_CONTAINERS` and `BAD_ION_LAB`,** since `_` is a word character. They are renamed too, and the task has a second search for them.
- **The diff's padded column and the `1:92` position are spelled out in the task,** since a plain replacement breaks both. Sort-order moves are named there too.
- **Goldens bless from `hearth-builder-world`,** the only world that calls `bless`.
- **The open question in `hearth/docs/design/README.md`, 9 on private-looking names is removed:** the user's decision answers it.
- **No status file changes,** since the trunk lands whole.

## Overlaps with other drafts

None in the open work. The only other hearth work beside this draft is `debt-20/mmdebstrap-once`, already merged, and its files (`hearth/testing/hearth-fake-mmdebstrap/src/**`, `hearth/tests/builder/src/hosted/mmdebstrap.rs`, a debt item) hold no name this plan changes.

## Left, and put to the user

The plan's question `port-and-zone`: the SSH port, the zone and a sentence in `domain/store.md`, 9.2 are outside the mapping and stay. No task waits for the answer.
