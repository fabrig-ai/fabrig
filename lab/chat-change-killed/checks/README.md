# End-state evidence

Python 3.11+ standard library only, under the temporary lab-tooling exception.
The runner's `command_seconds` bounds each command.

```sh
checks/end-state collect OBSERVATIONS 127.0.0.1:PORT OUT
checks/end-state judge OUT
```

`collect` reads OBSERVATIONS once and writes its exact bytes to
`OUT/observations.jsonl`. The story has ended and the fake flushes before
replying, so its prefix through the last whole checkpoint holds every landing
temper saw. Lines after that checkpoint belong to an unfinished flush and are
ignored by the projection. No checkpoint, malformed lines before it, or any
nonzero loss count leaves the result undecided.

If the prefix names a landing C, collection asks `POST /.fake/git/objects`
for `{"commits":[C]}` to learn first parent P, then `{"commits":[C,P]}`.
It keeps `objects-N.request.json` and `objects-N.response.body`, including
received partial bytes, and prints HTTP statuses on stderr. No credentials,
retries, redirects or polling are used. Collection exits 0 even on failed
object reads; only unreadable observations or unwritable output fail it.

`judge` reads retained files only, with no network or clock. Objects must be
whole in the response: graph entries cannot supply missing trees, and the
second response must contain both C and P. It checks the last merge of
`fake/widget` into `main` agrees with main's last move, C's NOTICE equals the
22-byte `../NOTICE`, P has no NOTICE, and the squash adds only that file.
Deletion clears the last move. Paths, content and messages use the fake's
base64 byte encoding; commit numbers remain exact u64 integers.

Status 0 passes silently; 1 prints stable `fail`/`summary` JSON lines for
`not-landed/fake/widget/main`, `content/fake/widget/main/NOTICE` or
`changed/fake/widget/main`; 2 prints `undecided` JSON lines when evidence is
missing, lost or unreadable (the lab's `lab.md`, 7.2). C, P, tree paths and
NOTICE bytes go to stderr.

The separate stopped `promises` check reads the forge's observations at rest
and the engine's stopped store copy. It derives its profile and decisions
from temper records; the end-state helper judges no forge promise.
