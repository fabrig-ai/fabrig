+++
id = "copy-once"
part = "copy-once"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/pages/welcome_copy.rs",
  "temper/crates/temper-client-view/src/tests/welcome_copy.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "medium"
minutes = 20
+++

# The welcome's Copy reads once

## Goal

In a browser the welcome's command button draws its word twice, as `Copy<span>Copy</span>`: skein's web mapping writes a
button's name as its text, and `block()` also gives it a text child. Remove the text child, keeping the name, the styles and
the binding. Hold it with a test of the three labels and a walk of every gallery state.

Acceptance:

- **The fix:** `block()` in `pages/welcome_copy.rs` has no `w.text(b, word, &[])`. `skein/crates/skein-ui-web/src/mapping.rs` is
  unchanged.
- **The labels:** a test in `temper-client-view`'s `src/tests/welcome_copy.rs`, in the browser shell: for "Copy", "Copied" and
  "Couldn’t copy", the button's name is the word and it has no text child. Seen red without the fix.
- **The walk:** a test in the gallery's `src/tests.rs`: in every sample, a `Button`, `Tab` or `Option` with a name has no text
  under it, but a recommended choice's tag. Seen red without the fix.
- **The proof:** one throwaway render of `welcome-model-claude`, with the button's markup quoted in the report, reading "Copy"
  once.

## Inputs

- `temper/docs/plan/ui-copy-once/inputs/request.md`: the request, and *For every task of this plan*, which binds. Only focused and
  fuzzy suites run; no tier, no `Look-request` trailer.
- `skein/docs/design/ui-web.md`, 6, *Attributes*: a button, a tab and an option show their name as their text.
- `temper/docs/design/client/README.md`, 3.5: *The command's Copy*; and *Needs you*, `.choices`, for the `.rec` tag.
- `temper/docs/design/client/browser.md`, 7.1, for the gallery's one command.

No design changes: the design already says this, and the code comes to it.

## Owns and may change

**Owns:** the `files` above.

**Survey, on the trunk this spec was cut from; the walk is what checks it:**

- `temper-client-view` has 22 sites that open a role held in a variable, and 4 `b.open(match ...)` or `b.open(if ...)` forms. None
  draws a name and a text child in a `Button`, `Tab` or `Option`. The 17 `text()` helpers take no such role from any caller. The 4
  `control()` and `action()` sites open a `Button` with a name alone; their text is on the `Link` arm. `proposal_card` takes `Item`
  or `Region`. The match and if forms name `Figure`, `Image`, `Paragraph`, `Slot` and `Text`. `Role::Tab` isn't opened at all.
- **These pass the walk as they stand, and stay:** Home's Skip (`pages/home.rs`, `begin_head`), whose text is terminal-only; Away's
  toggle (`frame.rs`, `away`), whose child is an empty `Slot`; the rail's project row (`frame.rs`, `project_row`) and Configure's
  starter cards (`pages/configure.rs`), which have text and no name in a browser.
- **The one exception, by design:** a recommended choice's button has a name and, in a browser, a `Role::Text` child saying
  "recommended" (`pages/ask.rs`, `pages/goal.rs`, `pages/needs.rs`; `tests/needs.rs` holds it). Those are different words, not the
  name drawn twice.
- **Readers of the removed child:** none. `tests/welcome_copy.rs`, `temper-client-shell-web/src/host_tests_copy.rs` and the gallery's
  tests find the button by its name. `temper.css` styles `.temper-welcome-copy` itself and no child. No golden or print holds the
  doubled word.
- **A label still changes:** a name patch rewrites a button's own text (`skein-ui-web/src/page.rs`, `Patch::Attribute`;
  `page/web.rs`, `refresh` and `content`), as Away's toggle already relies on.

**This departs from the request, on the plan's judgement:** the request's walk says no `Button`, `Tab` or `Option` has both a name
and a text child. That is false of the recommended tag, which the design draws, so the walk excepts that one word. It stays as
strict as it can be otherwise.

**Pre-approved options:**

- If the walk trips on a control whose text repeats its name, it's the same bug: remove the text child, keeping the name, styles and
  binding. If that site serves the terminal too, keep the child there behind `w.shell == Shell::Terminal`, as Home's Skip does. The
  site's file is yours for that edit; name it in the report.
- If the walk trips on a control with other words than its name and not "recommended": don't change the page. Add that word to the
  walk's exception, and list it under `## Left` with the site.
- If taking "recommended" out of the walk's exception fails nothing (step 5), no sample draws the tag: remove the exception and say so.
- If an existing test reads the Copy's text run, make it read the name.

**Not yours:** `skein/crates/skein-ui-web/src/mapping.rs`, `temper.css`, the gallery's `names.txt` and `samples.rs`, `docs/design/`,
`docs/help/`.

## Steps

1. In `temper/crates/temper-client-view/src/tests/welcome_copy.rs`, the labels' test, beside the clipboard one and reading the same
   way:

   ```rust
   #[expect(clippy::disallowed_methods, reason = "semantic assertions locate the button")]
   fn copy_button(tree: &skein_ui::Tree) -> usize {
       tree.nodes()
           .iter()
           .position(|n| n.role == Role::Button && n.styles.as_slice().contains(&look::WELCOME_COPY))
           .unwrap()
   }

   #[test]
   fn the_button_reads_each_label_once_in_the_browser() {
       for (written, label) in
           [(None, "Copy"), (Some(Written::Done), "Copied"), (Some(Written::Refused), "Couldn’t copy")]
       {
           let (mut root, _) = welcome::model_with_forge(d::WayKind::Program);
           let mut view = f::browser_page_view(&root);
           if let Some(written) = written {
               let id = f::find(view.tree(), Role::Button, b"Copy").unwrap();
               let event = crate::act_event(&mut view, id, Act::Press).unwrap();
               f::root_event(&mut root, event);
               f::root_event(&mut root, d::Event::Copied { token: 1, written });
               f::build(&mut view, &root);
           }
           let index = copy_button(view.tree());
           assert_eq!(view.tree().nodes()[index].name.as_deref(), Some(label.as_bytes()));
           assert!(f::node_text(view.tree(), index).is_empty(), "{label}: a text child draws the name again");
       }
   }
   ```

2. In `temper/tools/gallery/src/tests.rs`, the walk, right after `no_state_reads_of_zero` and not at the end of the file. The gallery's
   `view` builds in the browser shell:

   ```rust
   // A browser's button, tab and option show their name as their text (skein/docs/design/ui-web.md, 6), so text under one
   // is more words; a recommended choice's tag is the one the design draws (temper/docs/design/client/README.md, 3.5).
   #[test]
   fn no_named_control_draws_a_text_child_in_any_state() {
       for sample in samples::samples() {
           let view = view(&sample);
           let nodes = view.tree().nodes();
           for (index, node) in nodes.iter().enumerate() {
               if node.name.is_none() || !matches!(node.role, Role::Button | Role::Tab | Role::Option) {
                   continue;
               }
               let mut text = Vec::new();
               for inner in nodes.iter().skip(index).take(usize::try_from(node.size).unwrap()) {
                   for run in &inner.text {
                       text.extend_from_slice(&run.text);
                   }
               }
               assert!(
                   text.is_empty() || text == b"recommended",
                   "{}: {:?} {:?} also draws the text {:?}",
                   sample.name,
                   node.role,
                   node.name.as_deref().map(String::from_utf8_lossy),
                   String::from_utf8_lossy(&text),
               );
           }
       }
   }
   ```

3. Show both red, one command at a time. Keep each failure for the report:

   ```
   heavy cargo nextest run -p temper-client-view --lib -E 'test(=tests::welcome_copy::the_button_reads_each_label_once_in_the_browser)'
   heavy cargo nextest run -p temper-client-gallery --lib -E 'test(=tests::no_named_control_draws_a_text_child_in_any_state)'
   ```

   The first fails at "Copy". The second fails on a sample that draws the command block, `welcome-model-claude` or one before it. If
   the walk fails first on any other site, that is the survey's miss: take the pre-approved option.

4. **The fix:** delete `w.text(b, word, &[]);` from `block()`. `w` and `Run` stay in use. Run both tests: green.

5. **The exception bites:** take `|| text == b"recommended"` out of the walk, run it once, see it fail on a sample with a recommended
   choice, restore it. The report names the sample.

6. **The proof,** with the fix in. `OUT` sits outside the slot and is never committed:

   ```
   heavy cargo run -p temper-client-gallery -- ~/src/fabrig/ui-copy-once/gallery
   ```

   Find the button in `welcome-model-claude.html` by its class, `temper-welcome-copy`. Quote its whole `<button ...>...</button>` in
   the report: its text is `Copy`, with no `<span>`. Take one screenshot of that page with a throwaway headless Chromium, serving `OUT`
   over a local `python3 -I -m http.server` if its bundle links need it, and leave it in `~/src/fabrig/ui-copy-once/`. Any script for
   this is untracked and gone before the push.

7. `cargo fmt`. From `temper/`, `python3 -I -B scripts/old_forms.py`: it reads its baseline relative to the working directory, so from the
   repository root it says the baseline is missing; don't run `--write` there.
   `heavy cargo clippy -p temper-client-view -p temper-client-gallery --all-targets -- -D warnings`.

8. The focused tests of the changed packages and their direct dependents:

   ```
   heavy cargo nextest run -p temper-client-view -p temper-client-gallery -p temper-client-shell-web -p temper-client-shell-terminal -p temper-client-world -p temper-playground-world
   ```

9. Commit, in small steps that say why, and push. No `Look-request` trailer.

## Report

As your text says, and:

- both tests' failures from step 3, and the rule each bit on;
- the sample the exception bit on in step 5;
- the button's markup from step 6, which builder holds the screenshot, and its path;
- each site beyond `welcome_copy.rs` the walk made you change, or "none";
- that the labels past "Copy" were seen in the view's tree alone: the gallery's page is static, and no browser tier runs in a plan.
