# A progress value

Draft, 2026-11-04, from a session with the user that day. skein-ui's
progress node says how far a piece of work has gone in words alone
(`../ui/README.md`, 3.1). This draft gives it a value, done of total, which
each shell maps to its platform's progress, and which a browser's
stylesheet can draw as a bar that fills. Its first consumer is temper's
fetch of Forgejo, whose percentage the clients say in words
(`temper/docs/design/draft/fetch-progress.md`, section 6).

## 1. In one page

- **A fraction on progress:** done of total, two whole numbers, done at
  most total and total above zero. Absent, the progress is
  indeterminate, as every progress node is today (section 2).
- **Words still say it:** the node's text says how far ("42%", "41
  MB"), and every shell gives that text to its platform as the value's
  words; the fraction is the platform's number, never a person's only
  way to know (section 3).
- **The web:** `aria-valuemin`, `aria-valuemax` and `aria-valuenow` on
  the progress bar, and `data-progress`, the whole percent, from which
  `skein-ui.css` sets a custom property an owner's stylesheet fills a
  bar with: attributes alone, nothing inline, under the page's
  `style-src 'self'` (section 4).
- **The text shell is unchanged:** `NAME: TEXT`, its text the words
  (section 5).
- **A phone's shells** map it onto their progress views (section 6).
- **One skein change,** near the light path: the property, its rows,
  the mappings, the stylesheet's rules and their goldens; then temper
  fills the fetch's bar from it (section 9).

## 2. The fraction

- **`Fraction { done, total }`,** both `u64`, a property of the progress
  role alone (`../ui/README.md`, 3.2): done at most total, total above zero.
  The builder asserts both, as it asserts an attribute off its roles
  (`../ui/README.md`, section 5).
- **Absent, indeterminate:** a progress node with no fraction is work
  whose end is not known, as today; a shell draws or says it so.
- **The owner's numbers:** bytes, steps, items, as its domain counts
  them; skein-ui never divides them for the tree. A shell that needs a
  ratio makes it, checked, from the two (sections 4 and 6).
- **Not a meter.** A meter, a value in a known range (a disk's use),
  still waits (`../ui/README.md`, 4.2): progress is how far a piece of work
  has gone toward its end.

## 3. Words

- **The node's text is the value's words,** as today: the owner writes
  the percentage, the bytes or the step, from its phrasebook
  (`../ui/README.md`, section 9). A fraction with no text is the owner's bug:
  the builder asserts a progress node with a fraction carries text.
- **No style is a fact** (`../ui/web/README.md`, section 10): the bar's width
  repeats what the words say, for sight.

## 4. The web

The page part's mapping (`../ui/web/elements.md`), for a progress node
with a fraction, beside the `aria-valuetext` it writes today:

- **`aria-valuemin="0"`, `aria-valuemax="TOTAL"`, `aria-valuenow="DONE"`,**
  the fraction's numbers in decimal: a progress bar's value, read with
  its words.
- **`data-progress="P"`,** the whole percent, `done × 100 / total`
  rounded down, so a bar never shows full before its work is done; 0 to
  100.
- **`skein-ui.css` sets `--skein-ui-progress`** from it, one rule per
  whole percent, `[data-progress="42"] { --skein-ui-progress: 42%; }`,
  101 rules: the page sets no style inline, and CSS's typed `attr()`
  is not in every browser the page supports. An owner's rule draws the
  bar with `var(--skein-ui-progress, 0%)`.
- **With no fraction,** none of them: the element is today's, and an
  owner's rule may draw a moving bar on `[role="progressbar"]:not([data-progress])`.
- **A whole percent is enough for sight:** a bar's width moves by one
  pixel per percent at most widths; the words carry what is finer.

## 5. The text shell

- **Unchanged:** `NAME: TEXT` (`../ui/text.md`, section 5). The text is
  the figure the owner chose; the shell draws no bar of characters,
  which a screen reader would read as noise.

## 6. A phone's shells

- **iOS:** `UIProgressView.progress`, or SwiftUI's
  `ProgressView(value:total:)`, from done over total; the text as
  `accessibilityValue` (`../ui/shells.md`, 2).
- **Android:** `RangeInfo` of a float range, 0 to 1, from done over
  total, and `ProgressBar`'s own scaled to its integer range; the text
  as its state description (`../ui/shells.md`, 3).
- **Absent,** each platform's indeterminate view, as today.

## 7. The diff

- **The fraction is an attribute** (`../ui/README.md`, section 7): a change is
  an attribute patch, which keeps the node's id, so a screen reader
  that follows it hears the new words.
- **The owner moves it at its own pace:** a view builds again when its
  state's count moved, so an owner that changes its fraction once per
  whole percent (as temper's fetch does, one line per percent) sends no
  more patches than that.

## 8. Its tests

- **The builder's:** a fraction on a progress node; refused off it,
  with done past total, with a total of zero, and with no text.
- **The web mapping's goldens:** a progress node with a fraction, its
  `aria-value*` and `data-progress`; done at total; a percent rounded
  down; none without a fraction.
- **The stylesheet's:** its 101 rules, checked against the numbers 0
  to 100 by a test that reads `skein_ui_bundle::STYLESHEET`.
- **The text shell's golden:** unchanged for a progress with a
  fraction.
- **The fake person** sees a progress by its words, as today.

## 9. Its order

1. **skein:** the property, its row in `../ui/README.md`, 3.2, and the rows of
   `../ui/shells.md`; the web mapping and `skein-ui.css`'s rules; the goldens.
   One change.
2. **temper** (`temper/docs/design/draft/fetch-progress.md`, section 6):
   its view sets the fetch's fraction from the bytes written and the
   size, and its meter's rule reads `var(--skein-ui-progress, 0%)` in
   place of its own `--temper-flow-meter-fill`; with no size, no
   fraction, and the moving bar stays.

## 10. Open questions

None. The width's channel is `data-progress` with `skein-ui.css`'s
rules, not the page part setting `--skein-ui-progress` through the
CSSOM: the mapping stays attributes alone and its goldens see all of
it (the user left the choice to this draft, 2026-11-04).
