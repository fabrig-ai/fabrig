+++
id = "chat-and-help"
part = "says"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-service/src/stock/configuring.txt",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-service/src/tests/stock_brief.rs",
  "temper/crates/temper-service/src/tests/limits.rs",
  "temper/crates/temper-service/src/limits.rs",
  "temper/docs/help/open-public-reading.md",
  "temper/docs/help/project.md",
  "temper/crates/temper-docs/src/lib.rs",
  "temper/crates/temper-docs/src/tests/rules.rs",
  "temper/docs/design/domain/agent.status.md",
]
effort = "medium"
minutes = 40
+++

# The configuring chat and the help say what may show and what the pages hold

## Goal

Asked about going public, the configuring chat says who does it and what may show, and never proposes it. `temper help open-public-reading` says what the public pages now show, and both it and `temper help project` give the words.

This task departs from the implementer's text: `docs/help/` is yours here, because the request's section 3 is a help change, and so is the one status entry below.

Acceptance:

1. `configuring.txt` holds the paragraph in `notes/contracts.md`, whole, where the note puts it, and a test holds it.
2. The stock's fit figures agree with it, and `temper-service`'s focused tests pass.
3. `open-public-reading.md` gives the words before the command that makes a project public, and states every fact under *The walk's facts* below.
4. `project.md` gives the words where it says how a project is made public.
5. A test in `temper-docs` holds both pages' words and the walk's facts.
6. The **Not yet** entry on 8.1 is gone from `temper/docs/design/domain/agent.status.md`.

## Inputs

- `temper/docs/plan/ui-public-3/notes/contracts.md`: the words, the paragraph and its figures. They bind.
- `temper/docs/plan/ui-public-3/notes/common.md`.
- `temper/docs/design/domain/agent.md`, 8.1 (*Its instructions*).
- `temper/docs/design/client/browser.md`, 8.3: what the pages show, the walk's source.
- `temper/docs/design/client/tools.md`, 5.3 (the `project amend` row).
- `jig/docs/design/domain/people.md`, 13.3 (*A title can carry a chat's words*).

## Owns and may change

Owned: the `files` above.

Readers, surveyed:

- `temper-client-tools/src/tests/help.rs`, `public_reading_walk_names_every_figure_and_visibility_remedy`, needs every string it lists to stay in the walk. Keep them, on one line each as they stand. It is `ctl-and-pages`' crate, so don't edit that test.
- `temper-docs/src/tests/checks.rs` holds the walk's headings, an `Expect:` in each numbered step, and the capital placeholders in a fenced `temper` line. Add no new placeholder.
- `temper/tests/engine/domain/tests/suite/configuring.rs` reads `configuring.txt` by `include_bytes!`, so it follows.
- `temper/docs/design/protocol/configuration.md`, section 11, and its status already say 355,965.

Pre-approved:

- **If a fit test measures another figure than the note's table,** take the test's, for example if smith's framing counts a byte differently. Then change `configuration.md`'s and `configuration.status.md`'s figure to match, which departs from your text, and say so in the report.
- **If another figure in `limits.rs` or `tests/limits.rs` is tied to `brief_bytes`,** move it with it, the least that passes, and name it.
- **On a catch-up with `model-catalogue-2`,** which also grows `configuring.txt` (`configuring-learns`) and may move these figures (`stock-models`): keep both paragraphs and recompute every figure in the note's table from the tests, rather than merging numbers by hand.
- **If `temper help` or a golden prints the walk's summary elsewhere,** regenerate it.

## Steps

### The chat

Add the paragraph, change the figures in the note's table, and add to `temper-service/src/tests/stock.rs`, beside `configuring_txt_speaks_the_persons_terms`:

`configuring_txt_says_what_a_public_project_shows`: `words` contains "Making the project public is no setting: never propose it" and the words whole. Those words are a literal here, since this crate doesn't read ctl's constant.

### The walk

`temper/docs/help/open-public-reading.md`:

- **Its summary,** line 5 and `temper-docs/src/lib.rs`'s `summary`, promise too much: a title can hold a chat's words. Both become: "Enable anonymous reading at the deployment's entrance, then make a project public and read what its public pages show."
- **Step 2,** before its command block, gives the words whole, then says that ctl asks nothing first and prints them again with its answer.
- **Step 3's** `Expect:` is rewritten from the facts below, keeping the page addresses in its first paragraph. Its line of what stays private stays: specifications beyond titles, instructions, results, conversations and people's names.

The walk's facts (`temper/docs/design/client/browser.md`, 8.3), in the walk's plain register, quoting the pages' own words where given:

1. **A goal's spend alone:** a goal's row shows its spend; no task's, and no budget.
2. **A row's words:** a goal's or task's identity and title, then its phase, for how long, and for a goal its spend: "running · for 14 minutes · 126 spent". An ended row says how it ended and how long ago: "landed · 2 hours ago · 126 spent". A goal reads "landed", "failed" or "ended"; a task "done", "failed" or "ended".
3. **The board's two limits,** which it states itself: goals that ended before the project went public aren't shown, and a restart empties the goals that ended. Quote its line: "Work that ended before this project went public, or before temper last restarted, isn't shown."
4. **Landings:** the board links to them only when the forge holds a queue or a landing. The Landings page holds a stock queue's merges as recent landings: its line says landed, its change and its commit, and its own page holds its change, with no gates. Until one has, "No landings to show yet."
5. **No link goes to the forge:** a commit, a pull request and a CI run are never links. A commit shows its first seven digits.

The page's bound is 12,288 bytes and it stands at 4,208, so nothing needs shortening.

### The project page

`temper/docs/help/project.md`:

- In *How it changes*, the bullet **Made public or private:** gives the words whole, after "stay private."
- In *Limits*, the bullet on what a public project's pages show keeps its list and adds that a title can hold what was said in a chat, pointing at that bullet. Don't repeat the words a third time.

### The test

In `temper-docs/src/tests/rules.rs`, beside `deployment_explains_tool_payload_and_its_dependencies`, whose whitespace it normalizes the same way:

`public_reading_says_what_may_show_and_what_the_pages_hold`:

- both pages contain the words whole;
- in the walk, the words stand before `--visibility public --key KEY`;
- the walk contains, normalized: "126 spent", "2 hours ago", "No landings to show yet.", "Work that ended before this project went public, or before temper last restarted, isn't shown.", "no budget", "first seven digits", "never links";
- it no longer contains "without exposing".

### Close

Drop the **Not yet** entry. Then narrow tests, `cargo fmt`, clippy on `temper-service` and `temper-docs`, and the focused tests of those two, `temper-engine-domain-world`, and their other direct dependents, as your text says. A change of what the stock's largest charter holds moves `standard`'s brief, so `temper-service`'s whole focused set runs. Commit and push.

Run `python3 -I -B tools/citations/citations.py --base origin/main` before pushing, since this task edits design status.

## Report

As your text says. Name the figures you changed and whether the note's table held. Name each rule you broke by hand and what failed.
