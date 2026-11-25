# pre-pr-failure-list: status

2026-11-17. Where the design lags or departs from [pre-pr-failure-list.md](pre-pr-failure-list.md).

The design carries this draft (`tools/pre-pr/design.md`, 1, 6, 7 and 9): the failures file, its place, form and writing; the summary's failures line, second after the verdict; pre-pr's cap told apart from the host's tail and a session's later fitting; a cache hit's file marked `cached`; and the tests, in `tools/pre-pr/test_pre_pr.py` alone. Where the code lags is that design's status (`tools/pre-pr/design.status.md`).

- **Design lags:** none.
- **Design departs:** 1: the file is `target/temper/pre-pr-failures.json` under the checkout's root, ignored by git, not `temper/pre-pr-failures.json` in the git directory, and the failures line names it by that path from the root: an agent's `read` refuses any path with a `.git` part, so a list in the git directory could not be read (`small-fixes-1116.md`, 1; the user, 2026-11-16).
- **Open:** none.
