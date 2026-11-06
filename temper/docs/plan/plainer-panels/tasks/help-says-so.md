+++
id = "help-says-so"
part = "help-says-so"
kind = "docs"
needs = ["person-answered", "held-waiting"]
files = ["temper/docs/help/task.md", "temper/crates/temper-docs/src/tests/rules.rs", "temper/docs/design/client/tools.md", "temper/docs/design/client/tools.status.md"]
effort = "high"
minutes = 30
+++
# The help says so

## Goal

`temper help task` says what `taken by:` and `waiting on:` print, including an
answered person task and a held task. `temper/docs/help/task.md` holds 8179 bytes
of its concept page's 8192, so make room there.

This task also closes the design these three tasks built.

This departs from your run's text: `docs/help/task.md` and the design lines below
are yours here.

Acceptance:

- The page says, in the words the merged code prints:
  - `taken by:` names who holds a person task, or prints `none`;
  - `answered by:` takes its place once an answer has ended the task, and a
    closing person task with no holder prints neither;
  - `waiting on:` prints the hold, in the merged form, for a held task that awaits
    nothing else.
- The page is within 8192 bytes, and has lost no fact.
- A new test in `rules.rs` holds those sentences, and bites.
- `tools.md`'s open question is closed and `tools.status.md`'s entry is gone.

## Inputs

- `temper/docs/design/client/tools.md`, 5.5 (area 3), 6.1 (`waiting on:`) and 15.
- `temper/docs/plan/plainer-panels/notes/common.md` and `notes/contracts.md`.
- The merged `waiting` and `person_text` in
  `temper/crates/temper-client-tools/src/jig/render/document.rs`: read the form
  there, since it is the user's answer.

## Owns and may change

Owns:

- `temper/docs/help/task.md`: any line, to make room and to add these facts.
- `temper/crates/temper-docs/src/tests/rules.rs`: one new test.
- `temper/docs/design/client/tools.md`: in 6.1, replace `In which words is open
  (section 15).` with the form as built; in 15, remove the question's bullet.
- `temper/docs/design/client/tools.status.md`: remove the `Not yet` bullet naming
  `answered by:`, and date the file with `date -u`.

Options:

- **Where the room comes from:** rewording that loses no fact, anywhere on the
  page. Start with the **Delegates** bullet, whose long middle repeats itself most,
  then **Acting**.
- **Keep these strings,** which `rules.rs` reads across line breaks: `invalid
  results`, `assignment was refused before any model turn`, `serve's stderr`,
  `` `DATA/box/logs/serve.log` ``, `precise refusal reason`, `` `Cancelled · its
  early start conflicts with SHA` ``, `goes on with a new producer`.
- **Limits' first line** says temper does not let ctl take a task or hand it back,
  though `task take` and `task hand-back` do so for a person task. `tools.status.md`
  records that gap, and the request says nothing else changes: leave it, don't
  mention `task take` beside `taken by:`, and list it under `## Left`.

## Steps

1. Read the merged code, then write the facts into the page's **What it waits on**
   bullet and a person-task line under **What it is**.
2. Condense until `wc -c temper/docs/help/task.md` is at most 8192.
3. Add to `rules.rs`, in the style of `task_help_explains_refused_invalid_results`:

   ```rust
   #[test]
   fn task_help_says_what_taken_by_and_waiting_on_print() {
       let text = find(b"task").expect("task page").text.split_whitespace().collect::<Vec<_>>().join(" ");
       for instruction in [/* `taken by:`, `none`, `answered by:`, `waiting on:`, the merged held form */] {
           assert!(text.contains(instruction), "task help must explain: {instruction}");
       }
   }
   ```

4. Edit the design lines above, then, from the repository root:

   ```
   python3 -I -B tools/citations/citations.py --base origin/main
   ```

5. The page is read by tests (`.config/gate/read-by-tests.txt`):

   ```
   heavy cargo nextest run -p temper-docs
   heavy cargo nextest run -p temper --lib -E 'test(help)'
   ```

   Commit and push.

## Report

As your run's text says. Give the page's bytes before and after, and say where the
room came from.
