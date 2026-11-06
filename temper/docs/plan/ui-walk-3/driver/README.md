# Replay the client walk

Python 3's standard library and Chromium are enough. Run in this checkout on a
builder; `SKEIN_TEST_CHROMIUM` may name Chromium. No account or installation is
needed. Every box uses `--fake-backends`.

Build the inputs, one command at a time:

```sh
heavy cargo build -p temper -p smith-fake-claude
heavy temper/scripts/bundle.sh --loopback "$PWD/target/ui-walk-3-bundle"
heavy cargo run -p temper-client-gallery -- "$PWD/target/ui-walk-3-gallery"
```

Then run all seven steps with one command, choosing 1440 or 420 and a fresh
absolute output directory under `~/src/fabrig/ui-walk-3/`:

```sh
python3 -B temper/docs/plan/ui-walk-3/driver/walk.py \
  --width 1440 --out "$HOME/src/fabrig/ui-walk-3/walk-again-1440" \
  --bundle "$PWD/target/ui-walk-3-bundle" \
  --gallery "$PWD/target/ui-walk-3-gallery"
```

The driver copies the binary, adapter and bundle before starting four private
boxes: change/brief, draft/Settings/sign-out, Needs you, Away. It presses visible
controls by role and name; at 420 it returns through Chats and uses the project
switcher for Settings and Home's link for Changes. It handles browser dialogs
and sets the viewport through CDP, including at 420.

Expect at least 31 minutes: the first box stays running with its page closed
while the other boxes are walked. Home is reopened only after that interval.
For a second-width run, `--width 420 --brief-from ABSOLUTE_FIRST_OUTPUT` reuses
the first box's live `03-home-brief-420-*` evidence, waiting for those files if
necessary. The first walk takes that capture by resizing its original live
1440 box, as the walk notes require. Without this option, a 420 run waits its
own 31 minutes.

`index.html` links every mock/client/pair PNG. Adjacent JSON holds the page's
text, computed styles and rectangles; `rows-*.json` records the path and
provisional verdict. Review those verdicts against the images and the decisions
before replacing the plan's report. `run.json` records durations and counts.
Copy the reviewed report beside the index. No screenshots or box data belong
in git.

A stalled feature-switch read is captured before a named `temper ctl` fallback
sets project ways. A ctl proposal is retained as public JSON; a project-filtered
inbox that misses it is captured too. Unsupported configuring-model output,
nonempty queue/task rows, and a temper-owned held card use the named gallery
states with an explicit `unreachable` row. The driver's log never prints a
sign-in link. All links are read from the fake box's output at runtime.

On completion or an exception, boxes receive SIGTERM and must print `Stopped`;
Chromium stops, and `boxes/` and `build/` are removed. Screenshots, logs, the
index and gallery remain. A fresh output directory is required for another
run. The two non-fake checks belong to `real-checks`; this driver does not run
them.

The fixed-tip replay opens Settings immediately after To begin, without the
old repository/one-second settling wait. The ctl fallback is still evidence of
a failure, never counted as verification of W1. Two additional fake cues after
the document's second turn replace its note with `work:` memory and add four
more working notes, then restore the same document as a later note. Reopening
the chat checks both the no-document case and continuation beyond the first
notes page (W7); the existing two document cues and their literal revisions
stay intact. `working-notes-WIDTH.json` retains the first public ctl notes page.
The phone Configure reference now provisionally matches W5's hidden hint;
review the capture before accepting that verdict.

`change()` waits for “main last moved” before labelling a capture landed; a
queued title alone is insufficient. A live nonempty queue is captured when
available and the gallery remains a fallback. The working-note proof saves
all ctl pages and compares Draft with Draft on the mock.

For the final small focus fix, rewalk the affected state on fresh boxes:

```sh
python3 -B temper/docs/plan/ui-walk-3/driver/focus.py \
  --width 1440 --out "$HOME/src/fabrig/ui-walk-3/walk-again/focus-after/1440" \
  --bundle "$PWD/target/ui-walk-3-focus-bundle"
```

Repeat at 420 with a fresh output directory. This proves that Enter-created
projects focus the heading without outlining it, and that the next keyboard
control retains its ring. It also captures the still-different rename form
and records Escape and Cancel focus. `focus-metrics.json` retains the numbers.
The full walk's client stays frozen in its copied bundle while this fresh
bundle runs; the report identifies which captures were replaced.
