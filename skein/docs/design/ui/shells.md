# Shells and platforms

A shell maps the tree onto its platform and reports a person's acts
back. skein has three: the text shell (text.md), the full-screen shell
(screen.md), and the browser parts an application's browser shell runs
its loop over (web/README.md). Every
other platform's shell is the application's, over the same tree.

- **The only impure code.** A shell is the platform's: its event loop,
  windows, network, storage, clock and entropy. The callbacks a platform
  needs are the shell's, never step code's.
- **One callback, one step.** Each thing the platform reports (an act on
  a node, an answer, a timer, the place changing, an effect's report) is
  one event: the application's loop runs one step for each, then does
  what it asked, in order; once no report waits, it builds and applies
  the patches.
- **Time and the seed are inputs,** read by the shell once per turn.
- **The shell's parts hold the order that protects requests,** so no
  order of the loop breaks it: a step's writes to the small store before
  its exchanges, a navigation away only after; no turn while one runs;
  patches applied only to the tree they are diffed from, a patch naming
  a node the shell does not hold being a panic (web/README.md, 4.3). A shell
  of the application's own holds the same order itself.
- **A panic is fail-stop:** the shell restarts the client, as a
  supervisor restarts a process (../../foundation/programming-model.md, 2.1), and what
  the application keeps in its small store makes what is pending safe to
  send again.
- **Layout and navigation chrome are each shell's, by design.** The tree
  says what each region is; a shell decides where it goes and what frames
  it: a browser's elements and stylesheets, a phone's navigation bar,
  tab bar and back stack, a terminal's lines.
- **Mapped whole.** A shell maps every role, state and property; it
  reports no act on a disabled node, and only those `README.md`, section 6's table
  gives its role; it shows a picture it lacks as its image's name.

## 1. The web

The page part keeps the full mapping (web/elements.md); in brief:

| Role | Element, or ARIA role on a `<div>` |
|---|---|
| page; slot | the mount element; `<div>` |
| banner, navigation, main, complementary, content info | `<header>`, `<nav>`, `<main>`, `<aside>`, `<footer>` |
| form, search, region | `<form>`, named; `<search>`; `<section>`, named |
| heading; paragraph, text | `<h1>` to `<h6>`, deeper `role="heading"`; `<p>`, `<span>` |
| list, item; term, description | `<ul>`, or `<ol>` with an order, `<li>`; `role="term"`, `role="definition"` |
| table, row, cell, column header, row header | `<table>`, `<tr>`, `<td>`, `<th scope="col">`, `<th scope="row">` |
| button, link | `<button type="button">`, `<a href>` |
| text field, text area | `<input>` of its entry kind's type, `<textarea>` |
| choice, choice group, check | `<input type="radio">`, `role="radiogroup"`, `<input type="checkbox">` |
| select, option | `<select>`, `<option>` |
| menu, menu item | `role="menu"` behind a button, `role="menuitem"` |
| disclosure, dialog | `<details>` with `<summary>`; `<dialog>`, shown modal |
| status, alert, log | `role="status"`, `role="alert"`, `role="log"` |
| tree, tree item, tabs, tab | `role="tree"`, `role="treeitem"`, `role="tablist"`, `<a role="tab">` |
| progress, time | `role="progressbar"`, `<time>` |
| code, quote, separator, figure | `<pre><code>`, `<blockquote>`, `<hr>`, `<figure>` |
| image; drawing | `<img src alt>`; `<figure>`: the scene's SVG hidden from assistive technology, then the fallback |

| State or property | Attribute |
|---|---|
| disabled, expanded, busy, current, invalid | `disabled` or `aria-disabled`; `aria-expanded`; `aria-busy`; `aria-current`; `aria-invalid` |
| pressed, selected | `aria-pressed`, a check's `checked`; `aria-selected`, a choice's `checked`, an option's `selected` |
| required, submits on enter | `required` or `aria-required`; `enterkeyhint="send"` |
| completed | the page part's word, last in the item, hidden from sight and read (web/elements.md) |
| continues, an end request | nothing of their own: the page part reads them (web/README.md, section 5) |
| folds away | `data-folds-away` on the `<details>`, read by the page part (web/README.md, section 5) |
| to log end | `data-to-log-end` on the `<button>`, read by the page part (web/README.md, section 5) |
| a name, a description, a level | `aria-label`; `aria-describedby`; `<h1>` to `<h6>`, deeper `aria-level` |
| a placeholder | `placeholder` |
| a hint | `title` |
| an entry kind | `type`: `text`, `number`, `email`, `tel`, `url`, `date`, `time`, `password` |
| a value, a focus request | `value`; `focus()` |
| a count, a position | `aria-setsize`, `aria-posinset`; on a table and its rows, `aria-rowcount`, `aria-rowindex` |
| an order | `<ol start>`, the browser numbering its items |
| spans, a sort | `rowspan`, `colspan`; `aria-sort` |
| a fraction | `aria-valuemin`, `aria-valuemax`, `aria-valuenow`, read with its text's `aria-valuetext`; `data-progress`, the whole percent (web/elements.md and web/README.md, section 10) |
| a shortcut | `aria-keyshortcuts`; Escape, the `<dialog>`'s `cancel` event, or in a form its `keydown` |
| a region name, style tokens | `data-region`; classes |

## 2. iOS

A phone's shell is the application's to write: this shows that the
vocabulary maps, not how such a shell is built. Traits are
`UIAccessibilityTraits`; each control is UIKit's, then SwiftUI's.

| Role | Trait or container | Control |
|---|---|---|
| page; slot | | the shell's root view; its children |
| landmarks, region, figure, group | `accessibilityContainerType` `.landmark`; `.semanticGroup` for a figure, a group or an inner region | a view holding its children; SwiftUI `.accessibilityElement(children: .contain)` |
| heading | `.header` | `UILabel`; `Text` with `.accessibilityHeading(_:)` |
| paragraph, text, term, description, time, code, quote | `.staticText` | `UILabel`; `Text` |
| list, item | container type `.list` | `UICollectionView` cells; `List` |
| table, row, cells, headers | container type `.dataTable` (`UIAccessibilityContainerDataTable`) | cells answering `accessibilityRowRange()`, `accessibilityColumnRange()`; headers by `accessibilityHeaderElements(forColumn:)` and `(forRow:)`; `Grid` |
| button; link | `.button`; `.link` | `UIButton`; `Button`, and `Link` for an external link |
| text field; text area | `.searchField` in a search | `UITextField`, `UIDatePicker` for a date or a time; `TextField`, `SecureField`, `DatePicker`; `UITextView`, `TextEditor` |
| choice, choice group | `.button`, `.selected` | rows with a checkmark in a `.semanticGroup` |
| check | | `UISwitch`; `Toggle` |
| select, option | `.button` | `UIButton` with a `UIMenu` (`showsMenuAsPrimaryAction`), each option a `UIAction`; `Picker` |
| menu, menu item | `.button` | `UIMenu`, `UIAction`; `Menu`, `Button` |
| an item's menu | `accessibilityCustomActions` | `UIContextMenuInteraction`, `UISwipeActionsConfiguration`; `.contextMenu`, `.swipeActions`, `.accessibilityAction(named:_:)` |
| disclosure, tree, tree item | `.button` for its head, its state in `accessibilityValue` | a collection view's outline; `DisclosureGroup`, `OutlineGroup` |
| dialog | `accessibilityViewIsModal`; SwiftUI `.isModal` | a modal presentation; `.sheet` |
| status, alert, log | `.staticText` | a label; the shell posts `UIAccessibility.Notification.announcement` with its new words |
| tabs, tab | `.button`, `.selected` while current | `UISegmentedControl`; `Picker` with `.segmented` |
| progress | `.updatesFrequently` | `UIProgressView`; `ProgressView` |
| separator | not an element | `Divider` |
| image | `.image` | `UIImageView`; `Image`, `AsyncImage` for an address |
| drawing | the scene hidden, the fallback read | Core Graphics, or `Canvas` with `.accessibilityChildren` holding the fallback |

| State or property | iOS |
|---|---|
| disabled | `.notEnabled`; `isEnabled`; `.disabled(true)` |
| current, selected; pressed | `.selected`; a switch's `isOn`, a button's `.selected` |
| expanded, busy, required, completed, a sort | the shell says it in `accessibilityValue`, in its phrasebook's words |
| invalid | the description in `accessibilityHint`, and words in `accessibilityValue` |
| submits on enter | `returnKeyType = .send`; `.submitLabel(.send)` |
| a name, a description | `accessibilityLabel`; `accessibilityHint` |
| a level | `.accessibilityHeading(.h1)` to `.h6`; UIKit has `.header` alone |
| an entry kind | `keyboardType` (`.numberPad`, `.emailAddress`, `.phonePad`, `.URL`), `textContentType`, `isSecureTextEntry`; `UIDatePicker` |
| a value | `text`, `accessibilityValue` |
| a placeholder | a text field's `placeholder`, a `TextField`'s prompt; a text view has none, and the shell lays a label over it, read as the hint |
| a hint | with a pointer, `UIToolTipInteraction`'s text; `.help(_:)`; never `accessibilityHint`, which is the description's |
| a focus request | `becomeFirstResponder()`; `UIAccessibility.post(notification: .layoutChanged, argument:)` naming the node's view; `@FocusState`, `@AccessibilityFocusState` |
| a count, a position, spans | a data table's `accessibilityRowCount()`, `accessibilityRowRange()`, `accessibilityColumnRange()`; a list has none, and the shell may say it in `accessibilityValue` |
| an order | no API: the shell draws each item's number before its words, and its `accessibilityLabel` says it |
| a shortcut | `UIKeyCommand` with `.command` or `.alternate`; `.keyboardShortcut(_:modifiers:)` |
| Escape | `accessibilityPerformEscape()` on the dialog's view, `UIKeyCommand.inputEscape`; `.accessibilityAction(.escape)`, `.keyboardShortcut(.cancelAction)` |
| a source | a picture from the application's assets (`UIImage(named:)`); an address the shell loads |
| a fraction | `UIProgressView.progress` from done over total, its text as `accessibilityValue`; `ProgressView(value:total:)`; with none, the indeterminate view |

**What an iOS shell does itself:** its layout; its navigation chrome
(what the banner and navigation hold placed in a navigation bar or a tab
bar, a back stack made from addresses); an item's menu as its swipe
actions, long-press menu and custom actions; status, alert and log as
announcements; words for the states it says as a value; style tokens
interpreted as hints, by the names it knows.

## 3. Android

The application's to write, as on iOS. A class name is the View class,
its package left out, that a View shell gives
`AccessibilityNodeInfo.setClassName`; a Compose shell gives a `Role` and
semantics.

| Role | View: class name and node info | Compose |
|---|---|---|
| page; slot | the activity's content root; its children | the root composable; its children |
| landmarks, region, figure, group | no landmark: a view group holding its children; a pane titled by its name (`setPaneTitle`) where its content changes whole | `isTraversalGroup`; `paneTitle` |
| heading | `TextView`, `setHeading(true)` | `heading()` |
| paragraph, text, term, description, time, code, quote | `TextView` | `Text` |
| list, item | `ListView`; `CollectionInfo`, `CollectionItemInfo` | `LazyColumn`; `collectionInfo`, `collectionItemInfo` |
| table, row, cells, headers | `GridView`; `CollectionInfo` with rows and columns, `CollectionItemInfo` with row and column spans and a header's `heading` | `LazyVerticalGrid`; `collectionInfo`, `collectionItemInfo` |
| button | `Button` | `Role.Button` |
| link | a clickable node, its text a `ClickableSpan`, which TalkBack lists as a link | a `LinkAnnotation` in its text |
| text field, text area | `EditText`, `setInputType`; a text area `setMultiLine(true)` | `TextField` with `KeyboardOptions`; `singleLine = false` |
| choice, choice group | `RadioButton`, `RadioGroup` | `Role.RadioButton`, `Modifier.selectableGroup()` |
| check | `CheckBox` | `Role.Checkbox`, `toggleableState` |
| select, option | `Spinner`, its options selected or not | `Role.DropdownList`; its options `selected` |
| menu, menu item | `PopupMenu` and its items | `DropdownMenu`, `DropdownMenuItem` |
| an item's menu | custom `AccessibilityAction`s (`addAction`); a long press's context menu; swipes through `ItemTouchHelper` | `customActions`; `combinedClickable(onLongClick = …)`; `SwipeToDismissBox` |
| disclosure, tree, tree item | `ACTION_EXPAND`, `ACTION_COLLAPSE`; a tree's `CollectionInfo` hierarchical | `expand`, `collapse` |
| dialog | `Dialog`, `setPaneTitle` | `Dialog`, `paneTitle` |
| status, log; alert | `setLiveRegion`, polite; assertive | `liveRegion = LiveRegionMode.Polite`; `Assertive` |
| tabs, tab | `TabLayout` and its tabs | `TabRow`, `Tab`, `Role.Tab` |
| progress | `ProgressBar`, `RangeInfo` | `progressBarRangeInfo` |
| separator | not a node | `HorizontalDivider` |
| image | `ImageView`, `setContentDescription` | `Image(contentDescription = …)`, `Role.Image` |
| drawing | a View drawing the scene on `android.graphics.Canvas`, the fallback as virtual nodes (`ExploreByTouchHelper`) | `Canvas` with `clearAndSetSemantics`, the fallback's nodes beside it |

| State or property | View | Compose |
|---|---|---|
| disabled | `setEnabled(false)` | `disabled()` |
| expanded | `ACTION_COLLAPSE` offered, else `ACTION_EXPAND` | `collapse`, else `expand` |
| busy, required, completed, a sort | `setStateDescription`, in its phrasebook's words | `stateDescription` |
| current, selected; pressed | `setSelected(true)`; `setCheckable(true)`, `setChecked(true)` | `selected = true`; `toggleableState = ToggleableState.On` |
| invalid | `setContentInvalid(true)`, `setError(description)` | `error(description)` |
| submits on enter | `setImeOptions(EditorInfo.IME_ACTION_SEND)` | `KeyboardOptions(imeAction = ImeAction.Send)` |
| a name, a description | `setContentDescription`, else its text; `setHintText` | `contentDescription`; the field's supporting text |
| a level | `setHeading(true)` alone | `heading()` alone |
| an entry kind | `setInputType`: `TYPE_CLASS_TEXT`, `TYPE_CLASS_NUMBER`, `TYPE_TEXT_VARIATION_EMAIL_ADDRESS`, `TYPE_CLASS_PHONE`, `TYPE_TEXT_VARIATION_URI`, `TYPE_DATETIME_VARIATION_DATE`, `TYPE_DATETIME_VARIATION_TIME`, `TYPE_TEXT_VARIATION_PASSWORD`; a secret `setPassword(true)`; read back by `getInputType`, `isPassword` | `KeyboardType.Text`, `Number`, `Email`, `Phone`, `Uri`, `Password`; `password()`; a date or a time through a picker |
| a value | `setText` | `editableText` |
| a placeholder | `setHint`, which TalkBack reads while the field is empty | `TextField`'s `placeholder` |
| a hint | `setTooltipText`, never `setHintText`, which is the description's | a `TooltipBox`'s plain tooltip |
| a focus request | `requestFocus()`; `ACTION_ACCESSIBILITY_FOCUS` | `FocusRequester.requestFocus()` |
| a count, a position, spans | `CollectionInfo`'s rows; `CollectionItemInfo`'s row index (from 0: the position less one) and spans | `collectionInfo`, `collectionItemInfo` |
| an order | no API: the shell draws each item's number before its words, and `setContentDescription` says it | the same, in `semantics`' `contentDescription` |
| a shortcut | `onKeyShortcut` | `Modifier.onKeyEvent` |
| Escape | the back gesture (`OnBackPressedDispatcher`); a dialog's dismissal | `BackHandler`; `Dialog(onDismissRequest = …)` |
| a source | a picture from the application's resources; an address the shell loads | `painterResource`; the shell's loader |
| a fraction | `RangeInfo` of a float range, 0 to 1, from done over total, and `ProgressBar`'s own scaled to its integer range; its text by `setStateDescription`; with none, the indeterminate view | `progressBarRangeInfo` of the same range; `stateDescription` |

**What an Android shell does itself:** its layout; its navigation chrome
(a top app bar, a navigation bar or rail, a back stack made from
addresses); an item's menu as its swipe actions, long-press menu and
custom actions; words for the states it says as a state description;
style tokens interpreted as hints, by the names it knows.
