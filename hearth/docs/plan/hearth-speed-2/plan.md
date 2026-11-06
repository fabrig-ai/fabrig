# hearth-speed-2: the plan

hearth's focused suite under its 30 CPU s, aiming for 27 on the queue's figure, by repairs the profile shows and with no build setting changed. The design is `hearth/docs/design/domain/testing.md`, 9.1 and 11; the rule is `skein/docs/foundation/testing-strategy.md`, 8. The standing rules, the aim, the base commit and the method are in `notes/common.md`, and the repairs' rules in `notes/repairs.md`.

## Parts and tasks

| Part | Tasks, in order | Needs | Minutes |
|---|---|---|---|
| `profile` | `profile` | nothing | 90 |
| `builder` | `machine-call`, then `builder-repeats` | `profile` | 45, 45 |
| `tree` | `tree-fs`, then `tree-once`; `path-bytes` beside them | `profile` | 45, 75; 20 |
| `real-git` | `scratch-objects` | `profile` | 60 |
| `render` | `render-lookups`, then `render-repeats` | `profile` | 30, 40 |
| `last-round` | `last-round` | all four families | 60 |
| `close` | `close` | `last-round` | 40 |

After `profile`, five tasks start at once: `machine-call`, `tree-fs`, `path-bytes`, `scratch-objects`, `render-lookups`. Each owns a crate of its own. **The critical path** is `profile`, `tree-fs`, `tree-once`, `last-round`, `close`: about 310 implementer-minutes, and far less if the first wave meets the aim, because every later task starts by checking.

## What was decided

- **The profile goes first and the repairs wait for it,** because the request says what's in it decides the rest, and `hearth-speed` found no profiler and went without one. This cut can't read the profile, since a cut never waits for a run. So each repair is written from a survey of the source, and `notes/repairs.md` makes the profile overrule the spec: an option it shows under 1% of its family isn't done, and a cost it shows over 3% in the task's files is the task's.
- **The first wave is what the surveys found largest and certain,** and later tasks may never change a line. By reading alone, the three likeliest large costs are the fake machine's per-call diff (perhaps 30-50% of a build's run), the tree world's O(N²) scans (1.2-1.6 CPU s) and git's fixture work (about 1.3 CPU s). Together they could reach the 15%, so `builder-repeats`, `tree-once`, `render-repeats` and `last-round` each begin with **the start check**, and stop if the aim is met. That's "keep it small: stop once the aim is met" for a plan that can't see its own results.
- **Most of the cost isn't where the request looked first.** "Work a world repeats whatever its seed" turned out small in the builder world's setup (under 1-2% of a run, by reading), since the fake repository is cheap and the universe depends on the seed. The cost there is in the fake's own bookkeeping. `Fake` caches aren't in the plan unless the profile shows them.
- **`rendered_keys` once in a process is within the request,** the one repair near its "no seed" line. Every seed's build still runs and compares against the value, so a seed whose keys differ fails loudly; the implementer first checks by hand that the eight seeds agree, and falls back to a map by seed if they don't. The request names this repair: "work a world repeats whatever its seed, computed once in a test's process".
- **The object writer is the kit's own, not `hearth-fake-git`'s.** Reusing the fake's encoders would have real git check them, which nothing does today, but they're `pub(crate)`, hold trees in memory, and scan on insert, whereas the writer walks a work tree. A `sha1` dependency line is the smaller change.
- **Checked by this cut, with git 2.47.3 and a throwaway writer, deleted after:** written objects in stored zlib blocks give git's own commit ids and pass `fsck --strict`; `add -A` and `commit` on a repository with no index give git's ids too; a narrow `git add <path>` on one commits that path alone, and `git status` calls everything deleted. No test does those two, and the design and the spec say so. `make` costs 6.25 ms of git's CPU, the writer under 1 ms of its own with C hashing.
- **A survey's bisection was checked by hand.** Path order is task order under `tasks/` only because `class` gives `Declaration` only to `identifier`'s names, whose bytes are all above `/`. `-` would break it, and `identifier` refuses `-`. The spec says why it holds, and its table test holds it.
- **`figures.md` has one writer at a time.** Tasks side by side would meet in it, so repairs report in commits; `profile`, `last-round` and `close` write it, and each runs alone.
- **The method is written out in `notes/common.md`, with its driver,** because `testing.status.md`, 9.1 no longer holds it: the commit that took the `opt-level` lines out cut that text. It's `tools/tiers/design.md`, 7, 7.3 and 10.2 with a caller of its own, as `hearth-speed` measured. `close` puts it back in the status in a sentence.
- **Instructions as the noise-free check.** The same code read 26.9 and 31.1 CPU s across machines, and a 1% repair is lost in that. `notes/repairs.md` lets a repair land on its test's `Ir`, callgrind's count, with the family's figure showing no rise.
- **The base is `39b490f138`,** the main this plan started from, which every slot can reach. The trunk differs from it only in this plan's files.
- **Design:** `testing.md`, 9.1 names the three repairs as how the budget holds, `testing.status.md` has them **Not yet**, and 11 keeps only what's open. Nothing depends on a draft, so no draft's status changes.

## The survey

Who reads what changes, from the trunk at this cut. Each spec's `Owns and may change` names its own.

- **`hearth-fake-machine`:** `hearth-fake-git`, `hearth-fake-mmdebstrap`, `hearth-builder-world`, `hearth-conformance-world`. The tree, render, git and mmdebstrap worlds don't depend on it. `machine-call` changes no public API, and runs all four packages' tests.
- **`hearth-tree-world`:** only its own `tests/suite/*`.
- **`hearth-test-repository`:** only `hearth-builder-world` and `hearth`'s integration tests. `Repository::make` and `next` have many callers, whose signatures don't change. `hex_digest` is the one item another task changes in this crate, in another file.
- **`hearth_lib::path`:** `hearth-protocol-tree`, `hearth-domain-tree`, and `hearth/tests/tree/tests/suite/trees.rs`.
- **`Documents::declaration`:** across `hearth-domain-render`, and no public signature changes. `hearth/tests/render/src/schemas.rs` builds a `Documents` through its constructor.
- **`hearth-test-render`:** the render and builder worlds.
- **Markdown:** `.config/gate/read-by-tests.txt` names no hearth file, and no test in these crates reads Markdown. `tests/boundary/tests/outward.rs` skips this plan's directory, so a machine's name may stand in `figures.md`, but not in a status file.

## Overlaps

No draft beside this one lists a file under `hearth/`, so no spec avoids one. The one shared file is **`Cargo.lock`**: `scratch-objects` adds `sha1` to `hearth-test-repository`'s dependency list there, a line that no other draft's change touches, while `bundle-loading/gzip-skein` has already merged its own. This plan's own overlap is `hearth/testing/hearth-test-repository`, between `scratch-objects` (`src/lib.rs`, `src/objects.rs`) and `builder-repeats` (`src/resolution.rs`, one function's body): different files, stated in both.

## What this plan can't do

- **The queue's record after landing:** nothing in a plan runs after it lands, so no task can write that figure. `close` says so in `figures.md`, and the status adds that the budget reads the queue's figure, unknown for this revision until it lands. Whoever retires the plan states the record, as `hearth-speed`'s retirement did.
- **A cost in a shared kit:** the request makes that a question, not a task, and this cut can't see whether the profile shows one. `skein_lib::sha256`'s unoptimized `compress` and `skein-heap`'s counting allocator are the likely names, both by reading. `profile` names any under `### Shared kits`, `last-round` measures whether it would close the gap, and `close` carries it to the user in the status.
- **A process's floor:** about 11% of the suite on a builder, by `hearth-speed`'s estimate. libtest filters its whole list per process, which is std's, and how a test binary starts is a build setting. `profile` splits it; nothing here repairs it unless hearth's own code is in it.
