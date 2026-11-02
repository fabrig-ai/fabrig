# live-temper-stories: the plan

From `inputs/request.md` (11-02): fab's after-landing pair failed both owned-forge landing
stories at main c2922ded2. Two direct parts, three tasks; shared rules in `notes/common.md`.
The design is written with the specs: `temper/docs/design/client/tools.md, 8.1` (two new
bullets) and `temper/docs/design/domain/testing.md, 2.7` (the landing story's DIRECTORY and
paging, and what the stories need of the runner), each with its **Not yet** entry, which the
task that builds it removes.

## What failed, and why

- **ChatGPT, 6.6 s.** The cause is the story's harness, not ctl or the environment. Every
  ctl call of a sitting runs with the run's root (`LivePlan::client`, `<store>/first`) as its
  working directory and names DIRECTORY `client` beneath it (`support/ctl.rs`, `Calls`'s
  `client`). The held record's import guard (`LiveJudge::drive`, ~2194) and its give-back
  hook (`LivePlan::install_return_hook`, ~1327) are handed `self.plan.client` itself, so ctl
  ran `temper ctl <store>/first account import FILE`: `first` exists, is 0700 and locks, but
  holds no `address`, and `directory.rs`'s catch-all answered the failed `address` load with
  "DIRECTORY could not be read", which `key_file_fail` then glued to "record file". The rung
  (`forge_land/rung.rs`, ~613) hands the hook its `client` directory and is right. The
  directories' modes along the path play no part: ctl checks DIRECTORY, its files and FILE
  alone.
- **Claude Code, 56.6 s.** The live judge (`check_landing`, ~1821-1834) read each parent's
  ended delegates once and required the page uncut, with no cursor after it. ctl pages the
  ended group by `--after` (`tools.md`, 5.2), as the fake GitHub story already reads it
  (`github_land.rs`, `delegate_page`). One page is not the design: the judge pages.

## Parts and tasks

- **client-words** (`temper/docs/design/client/tools.md, 8.1`), slice 1.
  - `tasks/directory-words.md`, high, ~60 minutes, needs nothing: an absent `address` is
    "DIRECTORY is not signed in"; any other failed step prints `DIRECTORY could not be read:
    STEP: ERROR` (or `written` at sign-in's store); a FILE's failure is the FILE's, a key
    file's names its error; `key_file_fail` passes DIRECTORY's reasons whole. Frame tests in
    `temper-client-shell-cli`, the import's red before with the live run's exact line.
- **landing-story** (`temper/docs/design/domain/testing.md, 2.7`), slice 2.
  - `tasks/held-login-client.md`, high, ~40 minutes, needs nothing: `LivePlan::client_directory`,
    and the guard and the hook built through `LivePlan` methods that name it; a suite test
    with a stub ctl, red with the run's root, green with the client directory.
  - `tasks/delegate-pages.md`, high, ~60 minutes, needs `held-login-client` (same file):
    `EndedDelegates` pages the chat's and the change's ended delegates by `--after` within 32
    pages, in a new `LivePhase::Ended` before the closing reads; two suite tests.

## Critical path

`held-login-client` (~40 minutes) then `delegate-pages` (~60), ~100 minutes, up to ~200 with
the buffer; then fab's end check on the trunk, and fab's after-landing pair on a main holding
the three, which is the request's proof. `directory-words` runs beside them.

## Overlaps

- `temper/docs/design/client/tools.status.md`: `reference-m8`'s trunk adds a `5.4, 5.5`
  line just above the `8.1: GROUP VERB --help` line, and `v0.1.0-ladder`'s
  `ready/v0.1.0-ladder/finding-4` appends at the file's end; this plan's entry goes after
  `FORGE, which temper-client-tools' own step tests hold alone.`, apart from both.
  `tools.md` 8.1 is touched by neither (their hunks are in 5.2, 5.5, 6.1 and 6.4).
- `temper/docs/development/v0.1.0-ladder.md`'s finding-110 is this request's first failure
  (the wording). `directory-words` settles it; a ladder task cut for finding-110 would
  overlap `temper-client-shell-cli` and should not be cut.
- No neighbouring branch touches `temper-client-shell-cli`, `tests/support/forge_land.rs`,
  `tests/suite/forge_land.rs` or `testing.md`/`testing.status.md`.

## Calls made

- **The import is repaired in the harness, not in ctl.** The request asks to reproduce the
  failure in `temper-client-shell-cli`'s tests and repair it there unless the cause is the
  pair's environment. It is neither: ctl did what it was told on the directory it was given.
  `directory-words` reproduces the line in the crate's tests (an import on a DIRECTORY with no
  `address`, red with exactly `record fileDIRECTORY could not be read`) and makes it name the
  cause; `held-login-client` repairs the cause where it is, red before, green after.
- **What the stories need of the runner** is written in 2.7 even though the environment is not
  the cause, since the request asks for it and fab's pair gives it: an absolute
  `TEMPER_LIVE_STORE`, a 0600 record in a directory the user may write, the Claude login and
  `claude` on `PATH`, a private `TMPDIR`; no mode or name above them.
- **An absent `address` is "not signed in".** `address` is written by `sign-in` alone
  (8.1), so a DIRECTORY without it has never been signed in, as one without `token`; this is
  the line the live run would have printed.
- **The page bound is 32 pages in all,** as the judge's conversation pages (`read_page`'s
  `pages <= 32`).
- **No contracts note:** the two parts share no type, field or signature.
- **Each task removes its own Not yet text** from the status file, a departure the specs
  state, so that the status says what stands when the plan lands.
