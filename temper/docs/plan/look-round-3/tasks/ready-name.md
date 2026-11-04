+++
id = "ready-name"
part = "ready-name"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/pages/welcome.rs",
  "temper/crates/temper-client-domain/src/tests/pages/welcome.rs",
  "temper/crates/temper-client-view/src/pages/welcome.rs",
  "temper/crates/temper-client-view/src/tests/welcome.rs",
]
effort = "medium"
minutes = 60
+++

# Ready offers the first repository's name

## Goal

Ready's field "Your first project" holds the first repository's name with its first letter in
capital ("Hello" for `work/hello`), or "My project" with none to adopt, until the person writes
another. The empty-name line names the offered one. "Default" leaves the client, Ready's act
otherwise unchanged.

Acceptance:

- With a first repository made and adopted nowhere, Ready's field shows it capitalized; with none,
  "My project"; the value follows the `forges` watch until the person writes, and what they wrote,
  emptied too, stays theirs.
- Create asks for the shown name without its outer blanks, as now; the taken-name path compares
  with that same name.
- Empty, Create says "A name for it: NAME, or one of yours.", NAME the offered name.
- A first repository's name past jig's 64 bytes offers "My project".
- `DEFAULT_NAME` is gone and no test or gallery state reads "Default" as a first project's name.
- Every commit carries the request's trailer (`notes/common.md`).

## Inputs

- `temper/docs/plan/look-round-3/notes/common.md` and `notes/contracts.md` (The first repository,
  Ready's name).
- `temper/docs/design/client/README.md`, 3.5, The welcome: *Ready*, *The offered name*, and section
  11, *A taken name at Ready*.
- `temper/docs/design/draft/first-run-projects.md`, 4.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-domain/src/pages.rs`: `make_default`, `default_told` (their reads of
  the name), the `welcome_first` field, its getter and `Act::DefaultName`'s arm.
- `temper/crates/temper-client-domain/src/route.rs`: `playground_facts`, one line.
- `temper/crates/temper-client-view/src/words.rs`: `WelcomeDefaultUnnamed` gains a `Name` hole.
- `temper/tools/gallery/src/worlds.rs`, line 35: `FirstRun { .. } => "Default"` is the name of a
  project a world seeds. Change it only if a gallery test fails, to `"Hello"`.
- `temper/tests/client/**`, `temper/tests/client-browser/**`, `temper/tests/one-process/**`,
  `temper/tests/real-loop/**`: an assertion or a helper reading "Default" as the first project's
  name, or `Creating Default…`, the word only.
- `temper/docs/design/client/README.status.md`: drop this task's **Not yet** entry.

Options, each pre-approved:

- If the `forges` watch is refused or not delivered, `welcome_first` is `None`, and "My project" is
  offered: leave it so. A person who holds no forges reaches no Ready's act anyway.
- If `Act::DefaultName`'s handler is reached with the same bytes as the offered name, it still
  counts as written: a person typed it. Leave it so.

`temper/docs/design/draft/first-run-projects.md`'s section 9 stays open; change no taken-name
behaviour.

## Steps

1. **The domain.** As `notes/contracts.md`: `First`, `offered`, `OFFERED_NAME`, `written`, `shown`,
   `asked`. `offered`: `first.name` with its first byte ASCII-uppercased, a non-ASCII first byte
   left as it is; `OFFERED_NAME` for `None`, for an empty name and for one past `NAME_BYTES`.
   `Pages::welcome_first()`, `playground_facts` sets it from `forge::listed::unadopted_first`.
   `make_default` and `default_told` take the offered name from `pages.welcome_first`.
2. **The view.** `first_project` draws `state.shown(&offered)`, passes the name to
   `WelcomeDefaultUnnamed`, and to "Creating NAME…".
3. **Tests.** `temper-client-domain`, `src/tests/pages/welcome.rs`: `offered` for `work/hello`,
   `work/éa`, an empty and a 65-byte name, and none; the field following rows that arrive late; a
   written name and an emptied one surviving a new row. `temper-client-view`,
   `src/tests/welcome.rs`: the existing tests at 1347, 1404 and 1423 read "My project" with no
   repository in their fixture, or "Hello" with one.
4. **Check,** `notes/common.md`: `heavy cargo nextest run -p temper-client-domain -p
   temper-client-view -p temper-client-world -p temper-client-gallery`, clippy on the same, then push.

## Report

As `notes/common.md`, Your report.

## Answers

2026-11-04. May the gallery's first-run seed change to "Hello" with its tests passing, or does its
"Default" stand as an exception to the acceptance?

Change it. The acceptance governs: `temper/tools/gallery/src/worlds.rs`, line 35, becomes
`FirstRun { .. } => "Hello"`, gallery tests passing or not; "Change it only if a gallery test fails"
is withdrawn. No test reads that word and no sample presses Create, so nothing the task owns moves
with it. That one word is all that is left: keep the commit `66fb352e2d`, add the change under the
request's trailer, run the gallery's tests, `temper-client-gallery` alone, and report the task's
acceptance whole. `FirstRun { step: 22 } => "widget"` stays, and the gallery's playground worlds
showing this name in place of "Demo" are not this task's.
`temper/crates/temper/tests/fake_backends/walk/open_default.rs`, whose walk reads a "Default"
button, is `ready-adopts`'s (its step 6): leave it, under `## Left`.
