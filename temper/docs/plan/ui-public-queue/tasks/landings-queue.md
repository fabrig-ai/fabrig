+++
id = "landings-queue"
part = "public-queue"
kind = "feature"
needs = []
files = [
  "temper/tests/client-browser/src/public.rs",
  "temper/tests/client-browser/tests/browser_all/browser_public.rs",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 30
+++

# The public story reads a queued change on Landings

## Goal

`browser_public::public_board_and_task_lines_are_readable_at_desktop_and_phone_widths` fails on a page that is right:

    public landings page: temperLandingsQueuetask 18c22892 · Goal 18c22892 · Ready just nowNo landings to show yet.

A stock queue's change may be read while it waits, and the story's `Page::Landings` check doesn't take that form. Make it take a queued member too, as strictly as it takes the rest. This is the repair of the red `red-56`, and it changes no product code.

Acceptance:

1. `Tier::public_landings_reads` takes exactly the forms under *Implementation*, and `check_page_fields` asserts it for `Page::Landings`, with the same failure message.
2. Every other check in `browser_public.rs` stands as it is.
3. A table test in the tier's library holds those forms and refuses the others, and passes in the focused profile.
4. `browser_all` compiles and clippy is clean.
5. The story has been run by name, alone, as *The one browser run* says, and the report gives what each run's landings page read.
6. `temper/docs/design/client/browser.status.md` has lost this plan's `Not yet` entry.

## Inputs

- `temper/docs/design/client/browser.md`, 8.3, *The landings*, and 8.4, *The browser tier*'s last sentences. 8.4 already says what this task builds.
- `temper/docs/plan/ui-public-queue/inputs/request.md`: *One task*, and *For every task of this plan*, whose rule against running a tier's stories this task's one run departs from, as below.
- `temper/crates/temper-client-view/src/public/forge.rs`, `queues` and `landings`: what the page draws. `temper/crates/temper-client-view/src/words.rs`, `Times::public_time`: its time.

## Owns and may change

**Owns:** the files in the header.

**Departures from the implementer's text:**

- You remove this plan's entry from `temper/docs/design/client/browser.status.md`, and make no other edit under `docs/design/`.
- You run one story in a browser, which a plan's runs otherwise never do: a red's repair may run the test it repairs, by name, alone (`AGENTS.md`, 5). Nothing else runs in a browser: no other story, no whole tier, no walk, no render check.

**Options, each decided:**

- `lowercase_hex` in `browser_public.rs` has no other reader: it moves to `public.rs` with the check. If something else reads it by then, keep a copy where it stands and say so.
- `temper/tests/client-browser/src/lib.rs` stays untouched: `browser-cap` is adding lines beside its exports. That is why the check is an associated function on `Tier`, and `Tier` is already imported in `browser_public.rs`.
- If a run's assert fails on a form that `public/forge.rs` draws and this spec left out, add the form and its row, and say so. If `public/forge.rs` can't draw it, the page is wrong: report `Failed:` with the text, and widen nothing.
- If a run stops at any other check, such as the missing rectangle that `browser.status.md`, 8.4, records for this story, that isn't this repair's. Read its log, and run the story the second time. If that stops elsewhere too, list both logs under `## Left`, report `Done:` if the rest of the acceptance holds, and change no other check.
- If the machine can't start the run (no Chromium, say), list that under `## Left`. After-landing runs the story.

**Not yours:** any product crate (`temper-client-view`, `temper-engine-domain-forge` and the rest), `temper.css`, `docs/help/`, and the story's other pages and stages.

## Steps

### Implementation

```rust
// temper/tests/client-browser/src/public.rs, in `impl Tier`
/// Whether a public landings page's text, run together, is a form the demo's page takes
/// (temper/docs/design/client/browser.md, 8.4): the queue's one member while the change
/// waits, then no landing or the stock queue's merge.
#[must_use]
pub fn public_landings_reads(shown: &str) -> bool;
```

```rust
// browser_public.rs, check_page_fields
if matches!(self.page, Page::Landings) {
    // The demo lands by the stock queue (temper/docs/design/client/browser.md, 8.4).
    let shown = self.text.join("");
    assert!(Tier::public_landings_reads(&shown), "public landings page: {shown}");
}
```

The forms, whole and in order, with `ID` eight lowercase hexadecimal digits and `COMMIT` seven:

- `temperLandings`, then **the end**.
- `temperLandingsQueuetask ID · Goal ID · Ready TIME`, then **the end**.
- **The end** is `No landings to show yet.`, or `landed · task ID · COMMIT`, and nothing after it.
- `TIME` is `just now`, `a minute ago`, or `N minutes ago` with `N` decimal digits. There is no `1 minute ago`: the page says `a minute`.

The text is joined with no separator, so `TIME` runs straight into the end. Split on the page's fixed words, as the current check does. How you do it inside is yours.

A phase other than `Ready`, or an hour's or a day's time, is refused: a queue's line draws only `Ready`, and the tier's `global-timeout` is 120 s.

### Tests

One table test at the end of `public.rs`, in a `#[cfg(test)] mod tests`, `a_public_landings_page_reads_the_forms_it_draws_and_no_other`.

Taken:

- `temperLandingsNo landings to show yet.`
- `temperLandingslanded · task 18c22892 · 789ada1`
- `temperLandingsQueuetask 18c22892 · Goal 18c22892 · Ready just nowNo landings to show yet.`, the red's own text
- `temperLandingsQueuetask 18c22892 · Goal 3f2a9c01 · Ready a minute agoNo landings to show yet.`
- `temperLandingsQueuetask 18c22892 · Goal 3f2a9c01 · Ready 2 minutes agolanded · task 18c22892 · 789ada1`

Refused, each a taken row with one thing wrong:

- no end: `temperLandings`, and a member with nothing after its time
- no member: `temperLandingsQueueNo landings to show yet.`
- two members
- a task or goal of seven or nine digits, or with an upper-case digit
- a commit of six or eight digits
- another phase: `Held just now`
- another time: `Ready 1 minute ago`, `Ready an hour ago`, `Ready minutes ago`
- words after the end, and words before `temper`

It must bite, as your text says: for instance, let the member's goal take any width, and see its row fail.

### The one browser run

From the slot's root, after the tests above pass:

```
TIERS_WRAPPER=heavy TEMPER_BUNDLE=none temper/scripts/browser.sh -E 'test(=browser_public::public_board_and_task_lines_are_readable_at_desktop_and_phone_widths)' --no-capture
```

- `TIERS_WRAPPER=heavy` puts `heavy` before each build and nextest run inside the runner (`tools/tiers/design.md`, 3). The story needs the bundles and binaries that runner builds, so a bare `heavy cargo nextest` can't run it.
- Check the count: one test selected.
- Read the page's text from the `public page Landings:` lines of its output.
- Run it once. If that run didn't read a queued member, run it once more, unchanged: the request allows two, and which form a run reads depends on timing. Don't try to force a form, and don't run it a third time. A pass on any form meets acceptance, since the table holds the queued form either way.

### Finish

Remove this entry from `temper/docs/design/client/browser.status.md`, the one naming `ui-public-queue`.

`cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; then:

```
heavy cargo clippy -p temper-client-browser-world --all-targets -- -D warnings
heavy cargo nextest run -p temper-client-browser-world --lib
heavy cargo nextest run -p temper-client-browser-world --test browser_all --no-run
```

Nothing else depends on this package. Commit and push.

## Report

As your text says. Also:

- each browser run's result and what its landings page read, with the log's path
- the rule you broke for the table test and the row that failed
- anything under `## Left`
