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

The job also settles `.temper/known-reds.md` by its entries
(temper/docs/design/protocol/tooling.md, 4.1): it keeps incoming prose and entry
order, removes entries the head removed since the merge base, and inserts the
head's new entries after the incoming side's last entry (in its place if removed),
or at the end when there is no entry. It leaves the file unmerged on a deletion,
a non-regular stage or work tree file, invalid UTF-8, a result over 64 entry lines
or 8,192 bytes, or a write or staging failure. Conflicted bytes are restored on
failure; the reason is one standard error line, `.temper/known-reds.md: left: WHY`.

The B0 and B1 `temper/tests/real-loop/tests/fixtures/*/llm.json` recordings
have `regenerate.sh` generators using `temper-twin`, but those require captured
data outside the checkout. The merge job has no such input and leaves them
unmerged. It also leaves unknown recordings and hand-written fixture inputs
unmerged; it never infers a command from a conflicted file's contents or suffix.
The result is `$TEMPER_JOB/result.json`; exit 0 means settled, 3 means conflicts
remain. Baseline recounts require Python's standard library and git; recording
regeneration also requires the generator's build tools and dependencies.

Run the scratch-repository tests with
`python3 -I -B tools/merge/test_merge.py`.
