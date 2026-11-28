# The shapes reader

Provisional, 2026-11-28. `tools/shapes/shapes.py` reads a deployment's task activity as shapes in the worlds' vocabulary, `smith/3` (`smith/docs/design/shapes.md`), and names the shapes the deployment's runs hit that the worlds' samples didn't reach. It runs on the user's word, after landing, never in a plan: its tests read synthetic pages alone. Written from `temper/docs/design/draft/misbehaving-agents.md`, 8.

## 1. In one page

- **It reads three things:** the vocabulary's file; the worlds' shape lines, from files the user names; and a deployment's activity through `temper ctl`, every page of every task active in a window (section 2).
- **It shapes each completion in the window,** with `unknown` where the activity doesn't carry a field, and counts the completions it can't shape, by why (section 3).
- **It compares them with the worlds' samples,** a word `unknown` on either side matching any (section 4), and prints the shapes no sample reached, by how often, with the samples' sizes and the incomplete cases counted (section 5).
- **Nothing raw escapes:** only the vocabulary's words, counts and the reader's own phrases reach its report and its error lines (section 6).

## 2. What it reads

### 2.1 Its command

`python3 -I -B tools/shapes/shapes.py --project NAME [--project NAME ...] --from TIME --to TIME --worlds PATH [--worlds PATH ...] [--ctl COMMAND]`, from the repository root.

- **TIME** is UTC, `YYYY-MM-DDTHH:MM:SSZ`; `--from` comes before `--to`. The window is `[from, to)`.
- **`--ctl`** is the command that runs temper's ctl, split as shell words, `temper ctl` when omitted; the reader adds each read's arguments after it. The deployment it reaches, and its sign-in, are the user's.
- **Exit:** 0, a report printed; 1, no report, with a line saying why (section 6); 2, usage.

### 2.2 The vocabulary

`smith/testing/smith-shapes/vocabulary.txt`, in the form `smith/docs/design/shapes.md`, 5 gives. Its version names the shapes the reader writes, and its words are the reader's allowlist: a word it would write that the file's line for that field doesn't hold is written `unknown`. A file not in that form is refused.

### 2.3 The worlds' samples

- **Each `--worlds` file** is either a text file of lines or a JUnit report, such as a campaign round's (`tools/tiers/campaign.md`, 6), whose test cases' `system-out` and `system-err` texts are read as lines.
- **Its lines:** `fuzzy shapes: SWEEP VOCABULARY cases N distinct D beyond B`, a sample's header, and `fuzzy shape: SWEEP VOCABULARY N SHAPE`, one of its shapes (`skein/docs/design/testing/sweeps.md`, 4.6). Every other line is passed over.
- **A sample** is a sweep's header and its shapes. Samples of one sweep, from several files or test cases, add their cases, their shape counts and their `beyond`.
- **Refused,** with no report: a line of either kind in another version than the file's, named by both versions; a header or shape line not in its form, or a shape holding a word the vocabulary doesn't, named by its file and line number; no sample at all.

### 2.4 The deployment's activity

- **Its tasks:** every page of `task list --project NAME --json` and of `task list --project NAME --ended --json`, for each project named. A task is in the window when its last activity is at or after `from`.
- **Each task's pages:** `task activity TASK --json`, then each older page by its `before` cursor until a page has none (`temper/docs/design/client/ctl/tools-reads.md`, 1): every page of the task, so that an id issued before the window is seen. Its rows are taken in the order of their positions.
- **A read that fails:** a ctl that exits non-zero, or prints what isn't a page in its form, leaves that task's activity unread, counted as such (3.1), and the reader goes on; its output and its error are never repeated. A task list that can't be read ends the run with no report.
- **Kept only while it shapes.** Texts, inputs and numbers are read from a row while the reader classes its calls; none is kept past the run, and none is written.

## 3. A completion's shape

### 3.1 Completions, and those it can't shape

- **A completion** is each message the model spoke in a decoded turn row whose time is in the window: its calls in block order, or `text` with none (`smith/docs/design/shapes.md`, 2).
- **Not shaped, counted by why:**
  - `task unread`: a task whose activity couldn't be read, counted once;
  - `turn unread`: a turn row with no archive, or one that doesn't decode;
  - `turn cut`: a decoded turn marked cut, whose blocks may be missing;
  - `input unread`: a completion with a call whose input isn't a whole JSON object, withheld or cut.
- **A page cut** (the page's `cut`) is counted as `page cut`. It shapes on, but its task's activity isn't whole (3.3).

### 3.2 Each call

- **Its tool** is its name where that is a word of the vocabulary's tools other than `delegated` and `unknown`; `unknown` for a call answered `unknown_tool`; `delegated` for any other.
- **Its answer** is the call's `answer` (`jig/docs/design/client/domain/client/conversations.md`, 2), or `unanswered` where it is null. A call to `read_task`, `message`, `subscribe` or `mark_read` answered `delegated` is `read`, `sent`, `subscribed` or `marked` where it didn't fail, and `unknown` where it did: the activity doesn't carry which refusal it was.
- **Its references,** read from its input as each tool's input names them, in `smith/docs/design/shapes.md`, 2's order, each present one with its class (3.3).

### 3.3 A reference's class

From the task's own rows before the call, its whole activity read. A task's activity is **whole** when every page was read, none cut, and none of its turns was unread or its inputs unread; where the rules give `unissued` and the activity isn't whole, the class is `unknown`.

- **A path or an offset:** `unknown`, since the activity carries no workspace.
- **A job:** issued in the call's own attempt where an earlier call's outcome reads `started as job N` (`jig/docs/design/client/domain/client/conversations.md`, 2). Then `consumed` where an earlier `job` call naming it was answered with its end (`exited`, `exit_nonzero`, `signalled`, `command_timed_out`, `stopped` or `cancelled`), and `unknown` where not, since the activity can't tell a live job from a held one. Issued only in an earlier attempt: `before_opening`. Never issued: `unissued`.
- **A task:** seen where it is the task's own number, or a task reference or a number in any field or text of the rows before the call. Seen, it is `ended` where the task lists show it ended by the call's turn, `live` where they show it live then, and `unknown` where neither list holds it. Never seen: `unissued`.
- **A question:** `live` where the rows before the call carry its number as a question's; else `unknown`, since one task's rows can't tell a foreign question from one never issued.
- **A message:** `live` where the rows before the call carry a message by its number; else `unissued`.

### 3.4 Its run

The run fields' words are `smith/docs/design/shapes.md`, 3's, taken by their tags; a tag the vocabulary doesn't hold is `unknown`.

- **`end`:** the `run_ended` row of the turn's attempt; `unknown` where none was read.
- **`begin`:** the `run_began` row of the task's next attempt; `none` where there is none, the task is in the ended list and its activity is whole; else `unknown`.
- **`wake`:** for a `message` begin, the kind of the message it names, from the task's rows, or `unknown` where none carries it; `none` for any other begin; `unknown` where the begin is.

## 4. The comparison

- **A deployment's shape matches a world's** where both have the same calls in the same order, with the same tools and the same references, and each answer, class and run word is the same or is `unknown` on either side.
- **Reached** where it matches a shape of any sample; **missed** where it matches none. A reached shape that matched only through an `unknown` is counted as such.
- **Absence isn't proof.** A sample keeps at most its kept shapes, and a nonzero `beyond` says some cases reached more (`skein/docs/design/testing/sweeps.md`, 4.6), so a missed shape may be among them; the report names those samples. A shape no world drew may still be one a world can draw.

## 5. The report

Plain text on stdout, for the user: never written to a file, never into the repository. In order:

- `shapes: VOCABULARY`;
- `deployment: tasks T, turns U, completions C, shaped S`;
- `incomplete: task unread A, turn unread B, turn cut D, input unread E, page cut F`;
- `worlds: samples K, cases N, shapes M, samples cut P`;
- `missed: X shapes in Y completions`, then a line `N SHAPE` for each missed shape, N the completions that hit it, by N falling, then by the shape's bytes;
- `reached through unknown: Z completions`;
- `sample SWEEP cases N distinct D beyond B`, one for each sample, by sweep.

## 6. Nothing raw

- **What it writes:** the vocabulary's version and words, counts, the sweeps' names from the worlds' lines, its own fixed phrases, and, in an error line only, a `--worlds` path and line number the user gave.
- **What it never writes,** in its report or its error lines: a row, a page, a text, an input, a value, a path or a number of the deployment, a task's number included, or a ctl's output or error. A refusal says what kind of input failed and why, in its own words.
- **Its fixtures hold none either:** an expected report holds only what this section allows.

## 7. Testing

- **`tools/shapes/test_shapes.py`,** unittest, beside the reader, which the gate's tooling route runs when the reader changes (`.forgejo/scripts/gate_scope.py`). It never reaches a deployment or runs a `temper` binary: a fake ctl serves synthetic task lists and pages.
- **Its pages hold** a cut page, an unread turn, a job and a task issued before the window and named by a call inside it, a refused call, a job's end read and a resumed attempt's stale job, run ends, begins and waking messages; their texts, inputs and numbers carry sentinels.
- **It checks** each completion's shape, each count of the report and each incomplete case; that `unknown` on either side matches, and a sample's `beyond` is reported; and that no sentinel and no number of the input reaches the report, an expected report, or the error line of any refusal: a page not in its form, a failing ctl, a worlds' line in another version, a shape line not in its form.

## 8. Why

- **Every page of a task,** not the window's alone: an id issued before the window would read as never issued.
- **`unknown` where the activity is silent,** never a guess: a wrong class would report as missed a shape the worlds reach.
- **`unknown` matches either way:** the activity carries no workspace, so without it every deployment shape that names a path would be missed; and the worlds write `unknown` in the run fields (`smith/docs/design/shapes.md`, 7).
- **jig's tools' refusals `unknown`:** their answers are stored as text, and classing them by the product's words would copy those words into the reader.
- **The activity's pages alone,** as the draft asks: a cut turn is counted, not read again whole.
- **A script:** it reads JSON and JUnit with Python's standard library and links no Rust, and the vocabulary's file keeps its words one list.

## 9. Later

- **A cut turn read whole** with `task turn` (`temper/docs/design/client/ctl/tools-reads.md`, 1), so that its completions are shaped.
- **Where and how often it runs** against the reference deployment, and who reads its report: the user's (`temper/docs/design/draft/misbehaving-agents.status.md`, Open).
