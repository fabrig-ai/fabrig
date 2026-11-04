+++
id = "progress-value"
part = "progress-value"
kind = "interface"
needs = []
files = ["skein/crates/skein-ui/src/tree.rs", "skein/crates/skein-ui/src/builder.rs", "skein/crates/skein-ui/src/diff.rs", "skein/crates/skein-ui/src/lib.rs", "skein/crates/skein-ui/src/tests/tree.rs", "skein/crates/skein-ui/src/tests/diff.rs", "skein/crates/skein-ui-web/src/mapping.rs", "skein/crates/skein-ui-web/src/tests/mapping.rs", "skein/crates/skein-ui-web/src/tests/page.rs", "skein/crates/skein-ui-web/src/tests/golden/**", "skein/crates/skein-ui-bundle/skein-ui.css", "skein/crates/skein-ui-bundle/src/tests.rs", "skein/crates/skein-ui-bundle/src/tests/golden/index.html", "skein/crates/skein-ui-text/src/tests/fixtures.rs"]
effort = "high"
minutes = 55
+++

# A progress's fraction, its own words, and the stylesheet's percent

## Goal

skein-ui's progress node gains a fraction, done of total, and words a platform can read. The web maps both, and `skein-ui.css` turns the percent into a custom property an owner's stylesheet can fill a bar with. This is one change in skein; temper's fetch is its first consumer, later in this plan.

It also repairs a gap the cut found. `mapping.rs` writes `aria-valuetext` from the progress node's own runs, but `Builder::run` refuses every role but text, so a built progress has no such runs and no `aria-valuetext` reaches a page. A text child does not stand in for it: `role="progressbar"` presents no children to assistive technology.

Acceptance:

- `skein_ui::Fraction`, `Builder::fraction`, and `Builder::run` on a progress, as `temper/docs/plan/fetch-progress-1/notes/contracts.md`, section 1, has them;
- the builder refuses a fraction off a progress, with done past total, with a total of zero, and on a progress closed with no runs of its own;
- a change of fraction is an attribute patch and a change of words a text patch, both on the same node id;
- a page shows a progress's `aria-valuetext`, `aria-valuemin`, `aria-valuemax`, `aria-valuenow` and `data-progress`, and none of the last four with no fraction;
- `STYLESHEET` holds the 101 rules, checked against 0 to 100;
- `skein-ui-text`'s goldens are byte for byte what they were, for a progress that now has a fraction;
- this task's **Not yet** entries are gone from the status files named below.

## Inputs

- `skein/docs/design/ui.md`, 3.1, 3.2, 3.4, section 5, section 7.
- `skein/docs/design/ui-web.md`, section 6 (a fraction), section 12.
- `skein/docs/design/ui-text.md`, section 5.
- `skein/docs/design/draft/progress-value.md`, sections 2 to 5 and 8, for its reasons.
- `temper/docs/plan/fetch-progress-1/notes/contracts.md`, section 1, and `notes/common.md`.

## Owns and may change

Owns the files in the header.

May also change:

- `skein/testing/skein-fake-person/**` and any other reader in `skein/`, for what the compiler requires and nothing else. The survey found no `Attribute { .. }` or `Node { .. }` literal outside `skein-ui`, so none is expected.
- A departure from the implementer's text: **three status files.** Delete from `skein/docs/design/ui.status.md` the sentence beginning `3.2, 3.4, 5: a progress's fraction`. Replace the whole **Not yet** line of `skein/docs/design/ui-web.status.md` with `No gap.`. In `skein/docs/design/STATUS.md`, put `ui-web.md` back among the documents that have no gap, and take it out of those that lag. Nothing else under `docs/design/`.

Pre-approved:

- A text child inside a progress stays allowed for a progress with no fraction. `temper-client-shell-terminal/src/tests/fixtures.rs` builds one. Leave it; no assertion against children, and no temper file changes on this branch.
- If `skein-ui.css`'s rules fit better beside its other rules than at its end, put them there.
- The digest in `skein-ui-bundle/src/tests/golden/index.html` moves with the stylesheet's bytes: update it. If a golden outside the files in the header names that digest, update it too and say so in the report; the cut's search found none.
- If a page test needs a helper in `skein-ui-web/src/tests/fixtures.rs`, add it there.

## Steps

1. `skein-ui`:

   ```rust
   // tree.rs, beside `Spans`
   /// Done of total, in its owner's numbers (ui.md, 3.2).
   #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
   pub struct Fraction { pub done: u64, pub total: u64 }

   // builder.rs
   /// Say how far a progress has gone, its runs saying it in words.
   pub fn fraction(&mut self, fraction: Fraction) {
       assert!(self.node().role == Role::Progress, "fractions belong to progress");
       assert!(fraction.total > 0, "a fraction's total is above zero");
       assert!(fraction.done <= fraction.total, "a fraction's done is at most its total");
       self.node_mut().fraction = Some(fraction);
   }
   ```

   `Node` and `Attribute` gain `fraction: Option<Fraction>`; `Node::vacant`, `Node::new` and `Attribute::of` carry it; `diff::attributes` compares it. `open` reserves runs for a progress as it does for a text node. `run`'s assertion takes a text node or a progress. `close` asserts `"a fraction has its words"` on a node whose fraction is set and whose `text` is empty. `lib.rs` exports `Fraction`.
2. Tests in `skein-ui/src/tests/tree.rs`: `a_progress_holds_its_fraction_and_its_words`, and one `#[should_panic(expected = ..)]` each for the four refusals above. In `tests/diff.rs`: `a_moved_fraction_keeps_its_node_and_patches_its_attribute_and_text`, two builds of a progress, 42% then 43%, one `Patch::Attribute` and one `Patch::Text`, one id.
3. `skein-ui-web/src/mapping.rs`, in `core_element`'s `Role::Progress` arm:

   ```rust
   if let Some(fraction) = a.fraction {
       e.set("aria-valuemin", "0");
       e.set("aria-valuemax", &fraction.total.to_string());
       e.set("aria-valuenow", &fraction.done.to_string());
       e.set("data-progress", &percent(fraction).to_string());
   }
   ```

   `percent` is `done × 100 / total` in `u128`, rounded down. The mapping stays attributes alone: nothing inline, no CSSOM.
4. Tests in `skein-ui-web/src/tests/mapping.rs`: a `Property::Fraction` in the `PROPERTIES` table with rows for a fraction with words, done at total (`data-progress="100"`), a percent rounded down (`1` of `3` gives `33`, and `2` of `3` gives `66`), `u64::MAX` of `u64::MAX` (nothing overflows), and none with no fraction. In `tests/page.rs`: `a_progress_that_moves_keeps_its_element_and_says_its_new_words`, through `View` and `Page` as the `a_text_patch_replaces_the_runs` test does: the element's `aria-valuetext` and `data-progress` are the new ones. And `a_progress_built_with_runs_has_its_value_text`, from `Builder`, not a `Made` literal: this is the gap above.
5. `skein-ui-bundle/skein-ui.css`: 101 rules, `[data-progress="N"] { --skein-ui-progress: N%; }`, N from 0 to 100, under a one-line comment. `src/tests.rs`: `the_stylesheet_sets_a_progress_for_every_whole_percent`, which reads `STYLESHEET`, finds each exact rule, and counts 101 uses of `[data-progress=`. Update the golden's digest.
6. `skein-ui-text/src/tests/fixtures.rs`, `remaining`: the progress takes `b.run(Run::Plain, b"half")` and `b.fraction(Fraction { done: 1, total: 2 })`, in place of its `b.text`. `every.txt` and `roles.txt` do not change: that is the test.
7. The status lines above; `cargo fmt`; the old-forms check; `heavy cargo clippy -p skein-ui -p skein-ui-web -p skein-ui-bundle -p skein-ui-text --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-ui -p skein-ui-web -p skein-ui-bundle -p skein-ui-text -p skein-fake-person`; `heavy cargo check --tests -p jig-client-world -p temper-client-view -p temper-client-shell-web -p temper-client-shell-terminal`. Commit and push.

## Report

As the implementer's text has it, with: each rule broken for a new test and the failure seen; whether any reader outside `skein-ui` needed an edit; the stylesheet's bytes before and after.
