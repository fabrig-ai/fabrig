+++
id = "numbered-lists"
part = "numbered-lists"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-ui/src/markdown.rs",
  "skein/crates/skein-ui/src/tree.rs",
  "skein/crates/skein-ui/src/builder.rs",
  "skein/crates/skein-ui/src/diff.rs",
  "skein/crates/skein-ui/src/tests/markdown.rs",
  "skein/crates/skein-ui/src/tests/markdown/17.md",
  "skein/crates/skein-ui/src/tests/tree.rs",
  "skein/crates/skein-ui/src/tests/diff.rs",
  "skein/crates/skein-ui-web/src/mapping.rs",
  "skein/crates/skein-ui-web/src/tests/mapping.rs",
  "skein/crates/skein-ui-web/src/tests/page.rs",
  "skein/crates/skein-ui-web/src/tests/golden/markdown-numbered.html",
  "skein/crates/skein-ui-text/src/render.rs",
  "skein/crates/skein-ui-text/src/tests.rs",
  "skein/crates/skein-ui-text/src/tests/bound.rs",
  "skein/crates/skein-ui-text/src/tests/fixtures.rs",
  "skein/crates/skein-ui-text/src/tests/golden/**",
  "skein/docs/design/ui.status.md",
  "skein/docs/design/ui-web.status.md",
  "skein/docs/design/ui-text.status.md",
  "skein/docs/design/STATUS.md",
]
effort = "high"
minutes = 70
+++

# A numbered list is a numbered list

## Goal

skein's Markdown reader reads an ordered list as a bullet list whose items begin with their number, so a numbered item shows a bullet and "1. ". A list node comes to say it is ordered and from what number; the web mapping draws `<ol start>`, and the text shell the numbers. Every number shows as written, and a line that merely starts with a number stays text.

Acceptance:

- Markdown's `4. d`, `5. e` make one list whose node has `ordered == Some(4)`, its items' words holding no marker.
- A number that is not the next opens another list, at the same nesting, from that number; a blank line or a line under an item ends a list, and the next says the number it goes on from.
- The lines in `a_line_that_merely_starts_with_a_number_stays_text` below make no list, and their characters are all kept.
- The web mapping gives `<ol data-node=".." start="N">`, with `<ul>` for a list with no order; a list changing between them is one `Op::Replace`, its items moved into it, and a first number changing is one `Op::Attributes`.
- The text shell prints `4. ` where a bullet would be, one more for each item after, and `- ` under a list with no order.
- Each new test was red first, the report saying what it showed, and bites (the implementer's text).
- `skein/docs/design/ui.status.md` has lost its 3.2 and 10 entries, keeping its 3 entry and its **Departs**; `ui-web.status.md` and `ui-text.status.md` are back to `No gap.`; in `skein/docs/design/STATUS.md`, `ui-text.md` and `ui-web.md` are back among those with no gap.

## Inputs

- `skein/docs/design/ui.md`, 3.2 (*an order*), 5, 10 (*numbered lists*, *bounded*, *fed real text*) and 17.
- `skein/docs/design/ui-web.md`, 6 (*a list's order*), 6.1 (*Another element*) and 7 (*an attribute*).
- `skein/docs/design/ui-text.md`, 5 (the item's row) and 8.
- `skein/AGENTS.md`; `skein/docs/foundation/programming-model.md`, 10, for step code.
- `skein/docs/foundation/testing-strategy.md`, 8.1, for the tier's budget.

## Owns and may change

Owns, as `files` lists them.

**One departure from the implementer's text:** the four status files in `files` are this task's to bring back, as the acceptance says, and nothing else in `docs/design/`. What the code teaches about the design goes in the report.

May also change, only for what is named:

- `skein/crates/skein-ui-web/src/tests/golden/mapping.html` and the text shell's other goldens: hand-edited, only for what a fixture's new list adds. A line you can't explain by that is a red.
- `skein/tests/ui/**`, `skein/tests/ui-text/**`, `skein/examples/ui/**`, `skein/testing/skein-fake-person/**`: a compiler-forced line alone. The survey found none: `Node` and `Attribute` are written whole only in `tree.rs` and `diff.rs`.

Not to touch: `skein/crates/skein-ui-web/src/page/web.rs` (`Op::Replace` is already what a changed tag takes), anything in `temper/`, `jig/` or `smith/`, and every stylesheet. `temper/crates/temper-client-view/src/look.rs`'s `WELCOME_NUMBERED`, a look's `ol` for a list of its own, reads as it did.

Who reads this, from the survey: `markdown::read` is called only from `temper-client-view` (`pages/{document,task,agent,goal,chat}.rs`, `cards.rs`, `cards/delegation.rs`), `temper-client-shell-terminal`'s fixtures and skein's own tests. No golden in any tree holds a `- N. ` line. `temper-client-view`'s `tests/home.rs` ("1. Tell temper how you work") and `tests/accounts.rs` ("3. API key") read numbers of the view's own words, not the reader's.

Options, each pre-approved:

- **A test in another tree that reads a number out of a Markdown item's words,** or a printed `- N. `, which the survey did not find: change only its expected words, one line each, and name it in the report.
- **A generated print in `temper-client-journeys` or `temper-client-gallery` that changes:** regenerate it by its own tool if each changed line is a numbered list; otherwise it's a red to explain.
- **The bound test can't be made to fail by taking the ten bytes away** (the room is wide): keep the ten bytes, keep `b.ordered(u32::MAX)` in `full_node`, and say so in the report, rather than contrive a page for it.
- **An item not directly under the list** (under a slot, a group): prints `- `. The reader never builds one; say so in the report if a fixture of skein's does.

## Steps

1. **The surface, no behaviour yet,** so the tests compile. `skein/crates/skein-ui/src/tree.rs`: `Node` gains, after `count`, and in both of its literals:

   ```rust
   /// A list's order: its items are numbered, and this is the first one's number (ui.md, 3.2).
   pub ordered: Option<u32>,
   ```

   `skein/crates/skein-ui/src/builder.rs`, beside `count`:

   ```rust
   /// Number a list's items, the first one `first` (ui.md, 3.2).
   pub fn ordered(&mut self, first: u32) {
       assert!(self.node().role == Role::List, "orders belong to lists");
       self.node_mut().ordered = Some(first);
   }
   ```

   `skein/crates/skein-ui/src/diff.rs`: `Attribute` gains `ordered`, `Attribute::of` copies it, and `attributes` compares it.

2. **The real text.** `skein/crates/skein-ui/src/tests/markdown/17.md`, a reply as an agent writes one, exactly:

   ```markdown
   The release needs three things, in this order:

   1. **Update the manifest.** Bump the version.
   2. **Run the checks.**
      - the focused tier
      - the slow tier
   3. **Tag it.**
      Only once the checks are green.
   4. Push the tag.

   7. Write the notes.
   1. Tell the team.

   It shipped in
   2026. That was late.

   3.14 is not a list, nor is 3 apples.
   ```

   The cases of step 3 that nest read with `nesting` 3. `reports` in `agent_reports_and_malformed_inputs_fit_the_byte_bound` becomes `[&[u8]; 17]` and takes it, its comment saying 17 holds numbered lists.

3. **The reader's tests, red.** `skein/crates/skein-ui/src/tests/markdown.rs`, with a helper giving each list node's `ordered` in preorder, cases as rows:

   ```rust
   #[test]
   fn a_numbered_list_says_its_order_and_its_marker_leaves_its_words() {
       // input, each list's order, the words
       // "1. a\n2. b\n"          [Some(1)]               "ab"
       // "4. d\n5. e\n"          [Some(4)]               "de"
       // "0. z\n"                [Some(0)]               "z"
       // "999999999. n\n"        [Some(999_999_999)]     "n"
       // "- a\n- b\n"            [None]                  "ab"
       // "1. \n"                 [Some(1)]               ""
   }

   #[test]
   fn every_number_shows_as_written() {
       // input, each list's order
       // "1. a\n3. b\n"                  [Some(1), Some(3)]
       // "1. a\n1. b\n"                  [Some(1), Some(1)]
       // "3. a\n2. b\n"                  [Some(3), Some(2)]
       // "1. a\n\n2. b\n"                [Some(1), Some(2)]
       // "1. a\n   under\n2. b\n"        [Some(1), Some(2)], a paragraph between
       // "3. a\n\n   under\n   it\n4. b\n"  [Some(3), Some(4)], a paragraph between
       // "1. a\n   - x\n2. b\n"          [Some(1), None], the first holding two items
       // "1. a\n   1. x\n   2. y\n2. b\n" [Some(1), Some(1)], the first holding two items
       // "- a\n1. b\n"                   [None, Some(1)]
       // "1. a\n- b\n"                   [Some(1), None]
       // "Steps:\n1. a\n2. b\n"          [Some(1)], after its paragraph
       // "1. a\n\nbefore\n9. after\n"   [Some(1)], "9. after" staying in the paragraph
   }

   #[test]
   fn a_line_that_merely_starts_with_a_number_stays_text() {
       // each makes no list, and text(&tree) is the input
       // "3.14 is pi", "3 apples", "1.5 million", "3) three", "1.", "1.five",
       // "007. bond", "1234567890. ten digits", "before\n42. after",
       // "before\n2. after"
   }

   #[test]
   fn reports_read_their_numbers_as_written() {
       // 01.md's first list is Some(3), 03.md's is Some(1);
       // 05.md, a passage cut before its list, keeps "4. " in its paragraph's words;
       // 17.md's lists are [Some(1), None, Some(4), Some(7), Some(1)]
   }
   ```

   `skein/crates/skein-ui/src/tests/tree.rs`, beside `a_position_of_zero_is_an_assertion`: `#[should_panic(expected = "orders belong to lists")]` on an item. `skein/crates/skein-ui/src/tests/diff.rs`: a list going from `Some(1)` to `Some(4)` keeps its id and gives one `Patch::Attribute` with `ordered == Some(4)`, and its items no patch.

4. **The reader,** `skein/crates/skein-ui/src/markdown.rs`:
   - `ListLevel`'s `ordered: bool` becomes `number: Option<u32>`: from `list_item`, the item's number; on the stack, the number that goes on in this list. `Item` keeps `level` and `body`.
   - `list_item` reads the number with checked arithmetic: one to nine digits, and no leading zero unless it is `0`; otherwise it is no item.
   - `list_step`: an item goes on in its list when the indent is the same and its number is the top's, the top's moving to the next; otherwise the existing branch closes that list and a new one opens. A new list calls `builder.ordered(first)` where it has a number.
   - The item's marker text in `line` goes, so the item holds only `inline` of its body.
   - `Blocks` gains `follows: Option<u32>`, the next number of the last numbered list `close_lists` closed: where it closes several, the outermost numbered one. In `line`, a list item with a number, when `blocks.paragraph` is open, is read as a list item only if the number is 1 or `blocks.follows` holds it; otherwise it falls through to the paragraph's lines below.
   - Keep the module's bounds: no recursion, one more word in `Blocks`, `markdown_bound` as it is priced. `ListLevel` stays two words.

5. **The web,** `skein/crates/skein-ui-web/src/mapping.rs`, `collection`, which runs after `override_element`:

   ```rust
   if let Some(first) = a.ordered {
       if e.tag == "ul" {
           e.tag = "ol".into();
       }
       if e.tag == "ol" {
           e.set("start", &first.to_string());
       }
   }
   ```

   Tests. `src/tests/mapping.rs`: `Property::Order`, setting `made.attribute.ordered = Some(4)`, and the row `(Role::List, Property::Order, r#"<ol data-node="1" start="4"></ol>"#)`. `src/tests/page.rs`, beside `a_changed_tag_replaces_the_element`, in its form: a list with two items gaining an order gives kinds `["replace"]`, its `html()` an `<ol ... start="1">` holding the items; a first number changing gives `["attributes"]` under whatever name that helper gives it. Beside `a_markdown_table_builds_the_golden_page`, `numbered_markdown_builds_the_golden_page` reads `17.md` with `markdown_depth: 2` and six reader levels (ui.md, 10), against a hand-written `golden/markdown-numbered.html`.

6. **The text shell,** `skein/crates/skein-ui-text/src/render.rs`:
   - `Parent` gains the next number, `next: Option<u64>`, set from a list's `ordered` as it is pushed. An item takes the top parent's and moves it on by one (`u64`, so `u32::MAX` and any count fit), and `node` writes it with `Decimal::of` and `. ` in place of `- `. A tree item and any other parent print `- `.
   - `room`: ten more bytes a node: a number holds up to ten digits, past the two of `- `. Its comment says so.

   Tests. `src/tests/bound.rs`, `full_node`: a list also `b.ordered(u32::MAX)`. `src/tests/fixtures.rs`, `property_collections`: one more list in that function's idiom, `b.ordered(4)`, two items, the second with a position, printing `4. ` and `5. ... (2)`, and the goldens that page reaches (`properties.txt`, `every.txt`) hand-edited for those lines. `src/tests.rs`, beside `a_markdown_table_prints_each_row_with_its_cells`, in its form, `numbered_markdown_prints_its_numbers_as_written` reads `17.md` with `markdown_depth: 2`, against a hand-written `golden/markdown-numbered.txt`, which should read, past the first line:

   ```text
   The release needs three things, in this order:
   1. Update the manifest. Bump the version.
   2. Run the checks.
     - the focused tier
     - the slow tier
   3. Tag it.
      Only once the checks are green.
   4. Push the tag.
   7. Write the notes.
   1. Tell the team.
   It shipped in
   2026. That was late.
   3.14 is not a list, nor is 3 apples.
   ```

   If the shell's existing rules for lines print otherwise, keep the rules and say what differed in the report.

7. **The status files,** as the acceptance says.

8. **Check.** The implementer's text, and because the reader changes what an item's words mean for each page that reads Markdown:

   ```text
   heavy cargo nextest run -p skein-ui -p skein-ui-text -p skein-ui-web -p skein-ui-tests -p skein-ui-text-tests -p skein-ui-web-tests -p skein-fake-person -p skein-ui-example-view -p jig-client-shell-web -p temper-client-view -p temper-client-shell-terminal -p temper-client-shell-web -p temper-client-world -p temper-client-journeys -p temper-client-gallery
   ```

   No browser tier: `page/web.rs` is untouched and `<ol start>` is the browser's own, and the user reads this on a real chat. `python3 -I -B tools/citations/citations.py --base origin/main`. Push.

## Report

The implementer's report, with: the red each test showed; the rule each test broke when you checked it bites; the print `17.md` gave if it differed from the sketch; anything a stylesheet of skein's made you notice.
