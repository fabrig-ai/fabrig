+++
id = "journeys-glob"
part = "globs"
kind = "feature"
needs = []
files = [".config/gate/read-by-tests.txt"]
effort = "medium"
minutes = 5
+++

# The web journeys' files are test inputs

## Goal

fab's queue takes any file under a `docs/` directory as docs unless `.config/gate/read-by-tests.txt` lists it. `temper/docs/design/client/journeys/` holds non-Markdown test inputs (CSS, JS, recordings, setups, pages, READMEs) read by `temper/tools/journeys/src/tests.rs` and `temper-client-shell-web`'s journey tests, and the list doesn't name it. Two lines of that file change, nothing else.

Acceptance:

1. The line

   ```
   temper/docs/design/client/journeys/**
   ```

   stands directly above the existing `temper/docs/design/client/tui/journeys/**`, the file's other globs unchanged and in their order.
2. The file's first line, now `# Markdown inputs compiled into, or read by, tests.`, reads exactly:

   ```
   # Inputs under docs/ (any format) or elsewhere that tests compile in or read; every non-Markdown input under docs/ must be listed, since the queue takes the rest as docs.
   ```

3. The task's commits change `.config/gate/read-by-tests.txt` alone, one line added and one changed: `git diff --stat BASE..HEAD`, `BASE` the commit the branch started from.

## Inputs

- `temper/docs/plan/docs-read-globs/inputs/request.md`, all of it; it wins over this spec where they differ.
- `.config/gate/README.md`, *The gate's lists*, the `.config/gate/read-by-tests.txt` item: what a listed glob does.

## Owns and may change

Owns `.config/gate/read-by-tests.txt`, those two lines alone.

Nothing else. `.config/gate/README.md` (*The gate's lists*: "globs of Markdown compiled into or read by tests") and `AGENTS.md`, 2 ("Some Markdown is read by tests") still say Markdown; the request keeps them as they are, so leave them and name them under `## Left`.

## Steps

1. Make the two edits of the goal.
2. **Departure from the implementer's text:** no build, no test, no clippy and no `cargo fmt` for this task; no package changes, and the request runs nothing beyond the plan's focused and fuzzy runs on the trunk. `python3 -I -B temper/scripts/old_forms.py` runs once and shows no count raised; `--write` doesn't run.
3. One commit, saying why: the queue takes unlisted files under `docs/` as docs, and these are test inputs. Push it.

## Report

As the implementer's text says: the commit pushed, the diff's stat, the `old_forms.py` output, and `## Left` naming `.config/gate/README.md` and `AGENTS.md`, 2.
