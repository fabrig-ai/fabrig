# The reference's merge job

`.temper/merge merge <request.json>` runs from a job directory with the merge
in progress in `<repository name>/` (temper/docs/design/protocol/tooling.md, 4.1). It recounts
unmerged tree baselines from the head's comments and the larger side counts,
stages successful recounts, and leaves other conflicts for a resolution. A new
`# Raised:` line on the incoming side or a failed recount leaves the baseline.
The result is `$TEMPER_JOB/result.json`; exit 0 means settled, 3 means conflicts
remain. It requires only Python's standard library and git.

Run the scratch-repository tests with
`heavy python3 -I -B tools/merge/test_merge.py`.
