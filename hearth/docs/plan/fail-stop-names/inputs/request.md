# Request: the fail-stop check for every binary, and the domain out of skein, smith and temper's docs

From fabrig/hearth, 2026-11-04, on the user's word. Two small direct parts, independent of each other. No behaviour of a
product changes.

## 1. Every binary installs the fail-stop hook, checked by a test

`skein/docs/foundation/programming-model.md`, 2: a panic is fail-stop, and each binary's `main` installs `skein_fail_stop`'s
hook before anything else, because the dev profile unwinds. `hearth/scripts/fail_stop.py` checks this for hearth's binaries and
no gate runs it. The user's decision: the check becomes a test in `tests/boundary`, over the whole workspace.

- Add it to `fabrig-boundary`, beside `direction.rs` and `lints.rs`, in their manner: a failure names the package, binary and
  path; a file it can't read is a failure, never a skip; a test that the check catches a breach, so it can't pass by reading
  nothing. It keeps the script's rule: in each `fn main`, only `use` and `const` items and comments may come before
  `skein_fail_stop::install();`. Read `.config/gate/README.md` and add the check to its list.
- The script's sweep over the workspace finds 2 of 30 binary targets breaching it, both to fix:
  `temper-stage-one` (`temper/tools/stage-one/src/main.rs`) and `playground-serve`
  (`temper/tests/client-browser/src/bin/playground-serve.rs`). Each needs `skein-fail-stop` in its manifest and the call. Say
  if either has a reason it can't take the hook: that's a question, not an exemption.
- `hearth/scripts/fail_stop.py` goes. Anything that names it goes with it.

## 2. The domain, outside `hearth/`

The user's decision: the private domain leaves current files; no git history is rewritten. `hearth/` is plan `hearth-docs`'s,
running now: leave it alone.

- `skein/tests/http/transcripts/forgejo-repository-captured.{http,expect}`: `git.example.org` for `git.ekanayaka.io`, as the
  sibling `forgejo-issues-page-chunked` transcript has it, its header comment too. It's chunked: the chunk size changes with the
  bytes, and the `.expect` must still say what the `.http` gives. The test passes as it did, on equal terms.
- `smith/benchmarks/tasks/repository/shell-cwd/grader/Cargo.toml`: two git dependencies on `git.ekanayaka.io/ai/skein.git`. Find how
  this grader is built. If a run still fetches that URL, use what the task can build from today, and don't merely rename it to
  a host that doesn't resolve. `smith/benchmarks/tasks/**` is read by tests.
- `temper/docs/design/domain/testing.md`, 1238, and `temper/docs/design/draft/github.md`, 620 to 629: these name a person's GitHub
  account as a probe owner. Say "the probe account" or the like. Keep section numbers and meaning.

**Leave alone:** every `freeekanayaka` in temper's code and scripts (`temper/crates/temper/tests/`, `temper/scripts/`). It's a real
account in the live probes' safety allowlist, and fabrig/temper decides what happens to it. Also leave
`free@localhost` in skein's captured transcripts, and `tests/boundary/tests/outward.rs`, which spells such names on purpose.

## Rules

- `tools/citations/citations.py --all` stays at 0 and `tests/boundary/tests/outward.rs` passes.
- Nothing changes outside what this request names.
- All parts must be in the plan when the cut ends.
