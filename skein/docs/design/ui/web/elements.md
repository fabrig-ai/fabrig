# The tree as elements

Each node is one element, by its role, through the page part's default
mapping, this table (../README.md, 3.1 and 3.2, and ../shells.md, 1); the browser's face finds
the person's roles through the same table (`testing.md`, 2). An application
overrides it for its own nodes only (1). A role, state or property the
light path adds (../README.md, 4.3) adds its row here, with its golden row in
the host tests (`testing.md`).

| Role | Element (ARIA role where the element has none) |
|---|---|
| page | the mount element the page part is given (`bundle.md`, 3) |
| banner, navigation, main, complementary, content info | `<header>`, `<nav>`, `<main>`, `<aside>`, `<footer>` |
| form, search | `<form>` (named), `<search>` |
| region | `<section>`, named |
| group | `<div role="group">` |
| heading | `<h1>` to `<h6>`; deeper, `role="heading"` with `aria-level` |
| paragraph, text | `<p>`, `<span>` |
| list, item | `<ul>`, or `<ol>` with an order, `<li>` |
| table, row, cell, column header, row header | `<table>`, `<tr>`, `<td>`, `<th scope="col">`, `<th scope="row">` |
| term, description | `role="term"`, `role="definition"` |
| button | `<button type="button">` |
| link | `<a href="<path>">`; a checked external Markdown link, `<a href="<url>" rel="noopener noreferrer" target="_blank">` |
| text field | a visible `<label for>` before a named `<input>`, its `type` by its entry kind: `text`, `number`, `email`, `tel` for a telephone, `url`, `date`, `time`, `password` for a secret |
| text area | `<textarea>` |
| choice, choice group | `<input type="radio">` with its label, `role="radiogroup"` |
| check | `<input type="checkbox">` with its label |
| select, option | `<select>`, `<option>` showing its name as its text |
| menu, menu item | `role="menu"` behind its button, `role="menuitem"`; a menu of a field's options (`../README.md`, 3.2), `role="listbox"` with no button, `id="skein-ui-options-N"`, and its items `role="option"`, `id="skein-ui-option-N"`, N the node's id (`README.md`, section 5) |
| disclosure | `<details>` with `<summary>`, holding its name, then its description's element |
| dialog | `<dialog>`, shown modal |
| status, alert, log | `role="status"`, `role="alert"`, `role="log"` |
| tree, tree item | `role="tree"`, `role="treeitem"`; nested tree items in `<div role="group">` |
| tabs, tab | `role="tablist"`, `<a role="tab">` |
| progress | `role="progressbar"` with its text, its own runs, shown in it and as `aria-valuetext` |
| time | `<time>` |
| code, quote, separator | `<pre><code>`, `<blockquote>`, `<hr>` |
| image | one of its owner's pictures: `<img src="<prefix><digest>/pictures/<owner prefix>/<file>" alt="<name>">`; an address: `<img src="<the address>" alt="<name>">`; a picture the look lacks: its name as text |
| figure | `<figure>` |
| slot | `<div>`, its children the owner's nodes |
| drawing | `<figure>` named by its name: the scene's `<svg aria-hidden="true">`, then the fallback's elements, hidden from sight but not from assistive technology while the scene is drawn, shown when it is not (`README.md`, section 6) |

- **An ARIA role alone** in the table is a `<div>` with that role.
- **Nested tree items:** each run of consecutive tree-item children of a
  tree item sits in one unnamed `<div role="group">` with no class.
  Other children stay directly under the tree item, in document order.
  A leaf has no group, and removing the last item removes its group.
  The browser derives levels and set sizes from this nesting where the
  owner gives no explicit count or position.
- **Attributes:**
  - *a name* is `aria-label` where the element's own text does not give
    it; a button, a tab, a menu item and an option show their name as
    their text; a menu item with a description shows each in a `<span>` of its
    own, the name first, its accessible name the name alone; a named text field has a visible `<label>` just before
    its input, linked by `for` to `id="skein-ui-label-<node id>"`, as a
    choice or check is linked to its label. The label supplies the name
    once, with no `aria-label` on the input; an unnamed field has no label;
  - *a description* is `aria-describedby`, naming an element of the page
    part's that holds its words as text, shown beside the node's
    element: after it; inside it, last, for an item, a cell and a
    header, whose parents take no other child; after the `<select>` for
    an option, which holds text alone. A disclosure's is drawn with its
    name, folded or not (../README.md, section 3): in its `<summary>`, after
    the name, a `<span id="skein-ui-description-<node id>">` holding a
    space and its words, and alone there for a disclosure with no name.
    The summary's words are its accessible name, so the `<details>`
    takes no `aria-describedby`, and a screen reader hears the
    description once, within the name ("3 tool calls read · grep ·
    edit, collapsed"). An owner's look tones it by that id, as
    `summary > [id^="skein-ui-description-"]`;
  - *states:* disabled is `disabled` where the element takes it, else
    `aria-disabled`; expanded `aria-expanded`; busy `aria-busy`; current
    `aria-current`, `page` on a link; invalid `aria-invalid`; pressed
    `aria-pressed`, a check's `checked`; selected `aria-selected`, a
    choice's `checked`, an option's `selected`; required `required`
    where the element takes it, else `aria-required`; submits on enter
    `enterkeyhint="send"`. There is no provisional state (../README.md, 3.2),
    and no attribute stands for one;
  - *completed,* which no attribute stands for either (../README.md, 3.2), is
    the page part's word for it (`README.md`, 3.2), a space before it, in a
    `<span id="skein-ui-state-<node id>">` of its own, last of the
    item's content and before its description's element. `skein-ui.css`
    hides it from sight only (`README.md`, section 10): a screen reader reads the
    item's words and then the word ("Branch pushed completed"), and what
    the eye sees is the owner's look, its style token;
  - *a fraction* on a progress is `aria-valuemin="0"`,
    `aria-valuemax="<total>"` and `aria-valuenow="<done>"`, the
    fraction's numbers in decimal, read with its `aria-valuetext`, and
    `data-progress="<percent>"`, the whole percent, `done × 100 / total`
    rounded down, made in `u128`, 0 to 100, so a bar never shows full
    before its work is done. A whole percent is enough for sight, and
    the words carry what is finer. With no fraction it has none of
    them, and an owner's rule may draw a moving bar on
    `[role="progressbar"]:not([data-progress])`;
  - *a placeholder* is `placeholder` on the `<input>` or the
    `<textarea>`. The name stays the label's or `aria-label`'s: a
    browser exposes the placeholder only while the field holds no
    value, and a screen reader reads it after the name, as a hint
    ("Message, edit text, Message T6…");
  - *a hint* is `title` on the node's element, which a browser shows on
    hover (../README.md, 3.2). The name stays `aria-label`'s or the element's
    text. Where the node has no description, a browser reads the hint
    as one, after the name;
  - *a focus request* is no attribute: the page part moves the focus
    (`README.md`, section 5);
  - *a collection's count and an item's position:* a list's or a tree's
    count is `aria-setsize` on each of its items or tree items, an
    item's position `aria-posinset`; a table's count is `aria-rowcount`,
    a row's position `aria-rowindex`;
  - *a list's order* is `<ol start="N">` in place of `<ul>`, N its first
    number, written at 1 too, and the browser numbers its items. An
    element a look gives the list (1) keeps its tag, and `start`
    goes only on an `<ol>`;
  - *a cell's spans* are `colspan` and `rowspan`, left out at 1; *a
    column header's sort* is `aria-sort`, `ascending` or `descending`;
  - *a region name* is `data-region="<its name>"`; *style tokens* are
    classes, by their names (`README.md`, section 10); *a shortcut* is
    `aria-keyshortcuts`, in ARIA's form (`Alt+K`, `K`, `Escape`), a
    primary one by the primary modifier the page part was given (`README.md`, 4.1):
    `Control+K`, or `Meta+K`, never both.
- **An image's address** is a path on the origin or an `https` URL,
  checked by skein-ui as Markdown's external links are (../README.md, 3.5);
  the `src` takes it as is. An `https` one needs its origin in the
  server's `img-src` (`bundle.md`, 4), else the image shows its name. The name is
  its text alternative, always.
- **Values** are in the element's own formats (../README.md, 3.4): a number in
  decimal digits, a date `YYYY-MM-DD`, a time `HH:MM`. A value the
  element does not accept shows empty, and an `Input` carries what the
  element gives back: an `<input type="number">` holding no number gives
  the empty text, which the owner's view reads as no number.
- **Text runs:** plain as text, emphasis `<em>`, strong `<strong>`, code
  `<code>`, a link run `<a>`.
- **Text is text:** every name, description, run, value and mark's text
  is written as text, never read as markup (../markdown.md); bytes
  that are not UTF-8 show as U+FFFD.
- **What the page part sets, and so reserves:** `data-node`, a node's
  id, on its element; `data-region`; `data-drawn` on a drawing's figure
  while its scene is drawn; `data-act` on a mark that carries an act
  (`README.md`, section 6); and ids starting `skein-ui-` on its own elements, which
  a description's `aria-describedby` and a menu button's `aria-controls`
  name, by which `skein-ui.css` hides a completed item's word, and by
  which an owner's look finds a disclosure's description. It reserves no
  class name. The parts' own elements (the mount element, a
  scene's SVG, a menu's button, a description's element, a completed
  item's word, the restart part's words) carry no `data-node`.
- **One listener per kind of event,** on the mount element, finds the
  node of the element acted on, by its nearest `data-node`. A click
  finds instead the nearest element whose node a person presses, a
  link, a button, a tab or a menu item, as a browser's own `<a>` and
  `<button>` take a click on their content (../README.md, section 3); a click
  inside none of them, its nearest `data-node`.

## 1. An application's elements and components

- **Another element,** for a role carrying one of its owner's style
  tokens (an `<article>` for a region carrying `notes-card`), keeps the
  node's role and name: the page part adds ARIA's role where the
  element's own differs. States, `data-node`, listeners and patches
  stay the page part's; nodes with no override keep the table.
- **A component** (../README.md, section 10), chosen by the node's role and one
  of its owner's style tokens, or by a drawing's kind, from the look's
  component table, is the application's ordinary Rust, given:
  - a host element the page part keeps in the node's place, with its
    `data-node`;
  - the node, on insert and on each patch to it;
  - the inbox and a function wrapping an `Act` on its node into the
    application's `R` (`README.md`, 4.1).

  It builds what it likes inside the host, reports only the acts its
  node's role takes, on its node (an `Input` with a field's whole value,
  a `Drawing` with its owner's bytes), and is removed with its node. The
  view refuses any other act, and any on a disabled node (../README.md,
  section 6), so a component on a role that does not fit is inert: **it
  renders a node and never stands in for a role.** The node and its
  fallback stay in the tree, which is all the text shell, the fake
  person and a screen reader see (../README.md, section 10); a drawing's
  fallback stays beside the host, hidden from sight only (`README.md`, section 6).
- **Inside the host element is the component's:** the page part's
  listeners leave its events to it; while the focus is inside, a
  shortcut with no modifier is not taken, as in a field.
- **What a component builds is the application's to keep accessible
  and safe:** the node's role and name on the platform (a code editor is
  a text area to a screen reader), checked by its tier; never a node's
  text read as markup (`README.md`, section 14); its styles from the origin's
  stylesheets, since the strict policy refuses a `<style>` element or a
  `style` attribute it writes (`bundle.md`, 4).
