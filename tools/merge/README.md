# The reference's merge job

`.temper/merge merge <request.json>` runs from a job directory with the merge
in progress in `<repository name>/` (temper/docs/design/protocol/tooling.md, 4.1). It recounts
unmerged tree baselines from the head's comments and the larger side counts,
stages successful recounts, and regenerates known conflicted recordings on the
merged tree. A new `# Raised:` line on the incoming side or a failed recount
leaves the baseline.
The recording rule knows `temper/tests/playground/tests/fixtures/story-a.log`:
it runs only `walk::story_a_walks_from_the_welcome_to_the_csv_change_landed`
in `temper-playground-world`'s `suite` binary through `cargo nextest`, with
`SKEIN_CODEGEN_WRITE=1` and `TEMPER_INTERNAL_BUNDLE=none`. The one-process replay reads
the same log. The generator's source must be present and unconflicted; both
recording sides must be regular files. It removes the conflicted output before
generating, requires a fresh nonempty regular file without conflict markers,
and stages only that output. An unavailable or failed generator restores the
conflicted bytes and leaves the index unmerged, as does invalid output.

The codec rule reads `tools/merge/codecs.txt`: one row per module, its
repository-relative path followed by its schema texts in command order,
space-separated, sorted by module path; `#` starts a comment. Only listed
modules can run. The list and the module's schema texts must be present and
unconflicted. The command writes lean records for every family.
Any conflict under `skein/testing/skein-codegen/` or
`skein/crates/skein-codec/` leaves every listed module unbuilt. Deleted or
renamed modules stay for the resolution, and both sides must be regular files.
For each eligible conflict it builds and runs only the generator's command:
`cargo run -p skein-codegen --bin skein-codegen -- <schema>... <out-dir>`,
with the recording rule's environment. It never runs the family's drift test,
which would build the conflicted codec crates. Arguments come only from the
list, never the conflicted module's contents. A missing or conflicted list
supplies no commands.

The command writes to a fresh scratch directory beside the module. A successful
command must produce a nonempty regular file without conflict markers; the job
preserves the conflicted file's mode, moves the fresh module into place, and
stages that module alone. An unavailable or failed build or command, or invalid
output, leaves the conflicted bytes, mode and index untouched. A staging failure
restores the conflicted bytes and mode. `fabrig-boundary`'s `merge_codecs` test
checks the list against tracked generated headers and reproduces each module
from its listed texts.

The journeys' rule knows the files the journeys' writer alone writes
(temper/docs/design/client/journey-writer.md, 2): under
`temper/docs/design/client/web/journeys/` or
`temper/docs/design/client/tui/journeys/`, a journey directory's
`recording` and its pages, `NN.html` or `NN.txt` with two digits or
more, and at either root `skein-ui.css`, `journey.js`, `theme.css` and
`tui-journey.js`. A conflicted path among them is settled with the
base's copy, the commit merged in, the landing branch's, read from that
commit at that path: where it holds the path, its bytes and mode are
written and staged; where it does not, the path is removed from the
tree and the index. No command runs and no model is asked. A deletion
or a rename there is settled as any other conflict of the path, since
only the writer writes it; the path's every stage goes, and its record
is in `settled`. Main's copy is the writer's, whose next run writes it
again.

Any other conflicted file is left to the resolution (temper/docs/design/protocol/tooling.md, 4.1).

The B0 and B1 `temper/tests/real-loop/tests/fixtures/*/llm.json` recordings
have `regenerate.sh` generators using `temper-twin`, but those require captured
data outside the checkout. The merge job has no such input and leaves them
unmerged. It also leaves unknown recordings and hand-written fixture inputs
unmerged; it never infers a command from a conflicted file's contents or suffix.
The result is `$TEMPER_JOB/result.json`; exit 0 means settled, 3 means conflicts
remain. Left files are records of their kind, with the deleting or renaming
side and commit where known, from the index and each side's history; the 512-byte result
keeps whole records, with `more` for omissions (temper/docs/design/protocol/tooling.md, 4.1).
Baseline recounts require Python's standard library and git; recording and
codec regeneration also require the generators' build tools and dependencies.

Run the scratch-repository tests with
`python3 -I -B tools/merge/test_merge.py`.
