# Walk: a copy made and adopted through `temper ctl` on the fake backends

Read `temper/docs/design/qa.md` first: whom a pass plays, the box, the hands, reading the forge, what
makes a finding. This brief is the walk's acceptance, and is rerun unchanged.

## What this is

A person's agent copies a public repository into a Forgejo and has it adopted into a project, on
`temper box --fake-backends`, through `temper ctl` alone, with `temper help` and `temper ctl help` as
its guide. The source is one of the two the fake forge seeds. **No request reaches a real host.**
Where you stall, guess, or read a design or temper's source to get on, that is a finding against the
help or ctl: note it with the minutes it cost, then go on.

## What your task gives you

- `W/bin/temper`, built at the walk's commit.
- Nothing to sign in to: the box's fake model and fake issuer stand in, and every value you need the
  box prints on its stderr as `box: ...` lines, which go to your box log.

## The scenario

1. **The box:** `temper box --fake-backends`, as a user unit, its output in your log. Read from the log:
   `box: fake forgejo http://ADDR`, `box: Forgejo elsewhere admin token file FILE`, and the two
   `box: copy source https://HOST/OWNER/NAME` lines.
2. **A forge and a project:** connect the fake Forgejo as its admin, with a first repository so that a
   project exists, following `temper help forgejo`. `temper ctl forge list` shows it connected and
   managed.
3. **A copy:** `temper ctl repository copy --project PROJECT --forge FORGE ADDRESS --until adopted
   --key KEY`, with the first source's address. It answers once adopted: the repository is `OWNER/NAME`
   on the forge, its default branch the source's, and not the project's home unless you asked.
4. **It says it as it goes:** a second copy, of the second source, without `--until`: it answers at once,
   and `temper ctl forge list` shows `Copying OWNER/NAME from HOST` until it is gone from the list and
   adopted. Time it.
5. **The checks are there:** through the forge's API, the copy's default branch holds
   `.forgejo/workflows/ci.yml`, whose commit is the bot's, on top of the source's commits. The source
   is unchanged.
6. **Taken:** the first copy again, under a new key. It stops, naming the name as taken, and the
   repository is untouched. Then the same address with `--as ORG/NAME`: it lands there.
7. **Refused:** an address of no source in the catalogue, under a host of `.invalid`. It stops with
   the forge's words; `forge list` keeps the row until it is asked again. `http://` and a path of more
   segments are refused before anything is sent, naming the form to write.

A restart mid-copy is no step of this walk: a fake peer's memory lasts one launch
(`temper/docs/design/protocol/setup.md`, 7.5), so the world's story holds it (`copy-restart`).

## The report

For each step: passed or the finding, with the command, what it printed and the minutes it cost.
Findings first, ordered by what they cost a person.
