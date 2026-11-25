# The browser parts: patches and acts

## 1. Patches and acts

- **Patches applied in place** by the page part (../README.md, section 7):
  - *an insert* creates a subtree's elements under its parent, before
    its sibling or at the end; each link gets its `href` in the turn
    (`README.md`, section 7), each drawing its scene (`svg.md`, section 1) or its component
    (`elements.md`, 5);
  - *a remove* removes the element and its subtree;
  - *a move* moves them, and gives back the focus and the selection of a
    field it moved;
  - *a text* replaces a node's runs;
  - *an attribute* sets or removes what changed: a drawing whose data
    changed is drawn again, and a role or a list's order whose element
    differs makes the new element, the children moved into it;
  - *a value* writes a field's value, which comes only from the owner's
    write (../tree.md, 1.4).
- **Never replaced while it can be updated:** focus, caret, selection,
  composition and scroll stay. A removal that takes the focus moves it
  to the nearest remaining ancestor that can take it, a disclosure on
  its `<summary>`, else the main.
- **A focus request:** after the apply, a node inserted with one, or
  whose count moved, takes the focus. An element that cannot take it by
  itself (a heading, a region) gets `tabindex="-1"` first, so it takes
  the focus without joining the tab order. A disclosure takes it on its
  `<summary>`. The node then comes into view by the least scrolling that
  shows it, even where it already held the focus, when a browser's own
  focus moves nothing. A focus moved by a removal or by keys doesn't ask this.
- **Acts,** each reported with its node's id (../README.md, section 6), none on
  a disabled node; the view refuses one that races a patch disabling
  its node, and an option of a disabled select is disabled:
  - a press on a button, a tab or a menu item: `Press`, which only
    they, a link and an option take (../README.md, section 6);
  - a plain press on a link: `Press`, and the page does not navigate. A
    press with a modifier, or the middle button, on a link is the
    platform's: it opens the link's path in a new tab, a new start of
    the application; so is any press on an external link;
  - an input's whole value, on each `input` event outside a composition,
    and once at its end: `Input`;
  - Enter, outside a composition, in a field that submits on enter:
    `Submit`, its whole value sent first as `Input`; Shift+Enter in such
    a text area inserts a line break where its owner allows it, and does
    not submit;
  - a check's or a choice's change: `Toggle`; a select's change:
    `Press` on the chosen option's node. The page part then puts the
    element's choice back to the tree's, which the next patch moves if
    the owner followed;
  - a press in a drawing's scene: `Drawing` (`svg.md`, section 1).
- **Shortcuts** (../README.md, 6.1): the platform's keys turned into a press of
  the first bound, enabled node in preorder naming the key; a shortcut
  with no modifier never while a field has the focus. The page part's
  platform half gives each `keydown` as the event holds it, and its pure
  half decides:
  - *the modifier,* from the event's `ctrlKey`, `metaKey`, `altKey` and
    `shiftKey`, by the primary modifier (`README.md`, 4.1): none held, none; the
    primary one alone, the primary; Alt alone, Alt; any other set, no
    shortcut;
  - *the key,* the event's `key`, lowercased, when it is a letter `a` to
    `z` or a digit; else the letter or digit its `code` names (`KeyK`,
    `Digit5`), as when Alt on macOS composes another character, or a
    layout gives other letters;
  - *a key that presses a node* has its default prevented, so that the
    browser does not act on it as well.
- **Escape:** the platform's Escape key in an open dialog, which the
  browser gives the `<dialog>` as its cancel event, presses the first
  bound, enabled node in the dialog naming Escape; with none, it does
  nothing. The page part prevents the dialog's own closing: a dialog
  closes when the tree removes it, and one the browser closes on its own
  while it is in the tree is shown again. Outside every open dialog,
  the Escape key's `keydown` while the focus is in a `<form>` presses
  the first bound, enabled node naming Escape in the innermost form
  holding the focus (`../README.md`, 6.1), its default prevented; with none,
  it does nothing. An open menu takes Escape first, to close itself; a
  component that takes Escape for itself keeps it.
- **Menus and disclosures are the page part's to fold,** with no step. A
  disclosure is `<details>` and `<summary>`. An open disclosure that folds
  away (`../tree.md`, 1.2) folds on a press outside it, the press still going to
  what it lands on, and on Escape, after an open menu and before a form's
  Escape; a press inside it leaves it open. A menu is a button of the
  page part's, named by the menu's name, or reading and named by the
  menu's button where it has one (`../tree.md`, 1.2; `elements.md`), that opens its items with the
  platform's keys (arrows, Escape): ARIA's menu-button pattern. A patch
  that changes a menu's button changes the button's words and name in
  place, the menu open or closed as it was. A press
  on an item closes the menu. A press outside an open menu closes it, the
  press still going to what it lands on, and the focus returns to the
  menu's button, as on Escape, unless that press took it to another
  control. **A field's options** (`../tree.md`, 1.2, *options*) are no
  menu-button: the menu is a `listbox`, shown while it is in the tree, its
  items `option`s, and its field the text box that controls it,
  `aria-autocomplete="list"`, `aria-controls` naming the list and
  `aria-activedescendant` its active option, `aria-selected` on that
  option alone. The active option starts at the item the tree marks
  selected, or the first enabled one, and stays while that item stays in
  the list across an apply; otherwise it starts again. While the field
  holds the focus and its list an enabled option: ArrowDown and ArrowUp
  move the active option, wrapping and skipping disabled ones; Enter,
  with no Shift and no composition under way, presses the active option
  and gives the field no `Submit` for that Enter; Tab, where the field's
  words are not empty, differ from the active option's name and begin
  it, writes that name into the field and reports it as the field's
  `Input`, the focus staying, and otherwise moves the focus as the
  browser does; Escape is the form's, as above. A press on an option
  presses it, and its pointer's down keeps the focus in the field.
  PageUp and PageDown scroll the log as from any field (below).
  **An address changed folds the
  disclosures:** when the path or the query the page shows moved since
  the page part's last apply (a new entry, one replaced, back or
  forward), it folds every open `<details>` in that apply, so a
  disclosure whose link led elsewhere (a ☰ in the banner) is closed on
  the page it led to. Its platform half reads the address after the
  patches; the first apply only remembers it, and an apply at the same
  address folds nothing. The shell does nothing for it. **A dialog**
  is shown modal while it is in the tree. **A log** keeps its end in
  view while the person has not scrolled away from it; away, its element
  has `data-log-away`, for a stylesheet, until it is back at its end. A
  press of a button with the state *to log end* (../tree.md, 1.2) takes every
  log of the page to its end, keeping it there, before the press goes on
  to its owner, where the button is bound. A pointer's press on it leaves
  the focus where it was: the person is reading, and may be writing too.
- **Going to another page** (a new entry a step shows): after the turn's
  patches, its disclosures folded (above), the reference starts the
  page at its top, the focus on its
  main's first level-1 heading, else the main, unless a focus request
  moved in the same apply; back and forward restore their scroll,
  history's (`README.md`, section 7).
- **Reading a log.** The page part measures a log, its scroll position,
  its height and its screen, at each scroll, and a bound log after each
  apply that patched anything, a log's first among them, and from those
  alone:
  - *A bound list:* a `list` bound for `Near` (../README.md, section 6) is measured
    the same, at each scroll and after each apply that patched it, and reports
    its foot alone, while it continues, by the rules of *Near* below. It has no
    head to report and nothing follows: its scroll stays the person's.
  - *Near:* a bound log's head is near when what lies above the view is
    under one and a half of its screens; the foot of one that continues
    (../tree.md, 1.2), when what lies below it is. The page part reports `Near`
    (../README.md, section 6) when the edge comes near, again when the log's height
    moves while it stays near, and again for each quarter of a screen moved
    toward it, the head before the foot, one act a measure. So a page opened
    at its end reads earlier lines until its head is that far off, and a
    long scroll asks no more than it needs. An owner marks a log *busy*
    (../tree.md, 1.2) while what it reads is out: when that state goes and the
    edge is still near, the page part reports `Near` again, height moved or
    not, so that a page that drew nothing doesn't end the reading. An owner
    that keeps it through a failure hears nothing until the person moves.
  - *Following:* a log follows its end while the person is at it and it
    doesn't continue; one that continues is away, as one scrolled up is. An
    end request that moves (../tree.md, 1.2), or a press *to log end*, has it
    follow, across whatever reads bring its end, until the person scrolls
    up.
  - *The view stays:* across an apply, a log not following keeps the words
    being read where they were on the screen, to the pixel a scroll position
    can say, whatever the patches inserted or removed above them, in every
    browser. Before the apply the platform half notes the first entries that
    reach into the view and where each stands; after it, it moves the scroll
    by as far as the first of them still there has moved. A browser's own
    scroll anchoring (`overflow-anchor`) has by then done its part or, as
    in Safari, has none to do, so the two never add up. A status is no entry:
    it may stay at the head while entries arrive below it. Any other node
    there would be what holds, so an owner draws none.
  - *Keys:* PageUp and PageDown, with no modifier, scroll the page's first
    log with more than its screen to show, by its screen less a line, from
    anywhere on the page, the focus in a field or on nothing among it, their
    default prevented. A menu or a dialog open, a component holding the
    focus, or no such log leaves them to the platform. Home and End stay the
    platform's, which takes them while the focus is on the log. Such a scroll
    is a person's at once: the page part measures the log in the same apply,
    since the platform's scroll event waits for its next frame and a
    delivery can come first. So a PageUp has stopped the following before
    that delivery, and a PageDown that reaches the end has started it.
  - *Wheel:* an unmodified, cancelable vertical wheel over a log with room
    to scroll moves and measures it in the same event, before a delivery can
    miss its reaching the end. Pixel deltas are pixels, line deltas use the
    log's computed line height, and page deltas use its screen. A component,
    a nested scrolling pane or field, modified or horizontal input, and an
    event already prevented keep their platform handling.
- **The title,** as the reference sets it: the main's first level-1
  heading's text, then ` · ` and the application's name; the name alone
  when there is none. `<html lang>` is the phrasebook's language.
