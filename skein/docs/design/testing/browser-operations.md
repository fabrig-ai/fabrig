# Browser: the operations

This page sets out the steps from browser requests to replies and events.
The rest of `skein-browser` is in [`browser.md`](browser.md).

## 1. The operations

Each operation is an entity with its own state, stepping from one reply
to the next command, never a closure waiting on a reply:

| Operation | Its steps |
|---|---|
| Viewport | `Emulation.setDeviceMetricsOverride` with the requested width and height, `mobile: false`; `Done` after its reply, or `Refused` |
| a person | a browser context created; with a downloads directory, its downloads allowed there under the browser's own names, with their events (`Browser.setDownloadBehavior`, `allowAndName`, `eventsEnabled`), else denied |
| opening a page | create a target in the person's context, attach to it, enable the domains it reports on, set the viewport, navigate; `Opened` once the load event fires |
| go | navigate, reload, or move through the history; the page's `beforeunload` prompt, if it opens meanwhile, accepted, and `Warned`, or Chromium's entry that it blocked one taken as `Warned` (section 1); `Done` once the navigation commits: the load event for a new document, the same-document event for one that stays |
| find | the page brought to the front (`Page.bringToFront`); the accessibility nodes matching the role and the name, under the scope or the page's current document, either asked by its object in that document (below): exactly, both asked of CDP; by words, the role's nodes asked without a name, their names compared in the kit; at most `Limits::matches` kept, the rest counted in `more`; their properties; their boxes, if asked |
| await | a find, again every `Limits::poll`, until the expectation holds (`Met`) or `within` passes (`Missed`) |
| press | the node scrolled into view, its named point with it; its box's centre, or the named point in the box; layout metrics turn the viewport point into document coordinates for the hit test; refused `Hidden` without a box or with a point outside it, `Disabled` if its accessibility node says so, `Covered` unless the hit test at that point lands on the node or inside it; then the mouse moved, pressed and released at the viewport point |
| type | text with CR/LF first describes its node; for a single-line `input`, CR and LF are removed as on a person’s paste (CDP’s `Input.insertText` would change them to spaces). Text areas and editable content keep their line endings. Then the node is focused and the text inserted |
| compose | the node focused; a composition set and left open, which a later `Type` commits |
| keys | the node focused; for each character of the text, in order, its key down and up, the character its key and its text |
| key | the key down and up, with the text it makes; with `shift`, Shift's modifier on both; select-all, A's key down and up with Control's modifier and no text; a letter, its code (`KeyS`), its key and Windows key code (upper case with Shift), its text unless `alt`, and with `alt` Alt's modifier on both |
| download | not an operation: the browser-level `Browser.downloadWillBegin` names the frame, which is a page's when it is the page's target, and the suggested name, kept by the download's id; `Browser.downloadProgress` completed is `Downloaded`, its path the person's directory and the id; canceled is the page's `Trouble`; past `Limits::downloads`, `Browser.cancelDownload` |
| requests | not an operation: on a page opened with `requests`, `Network.enable` among the domains it reports on, before it navigates; each `Network.requestWillBeSent` of its main frame without a `redirectResponse` is `Sent`, `document` where its `type` is `Document`; the first `Network.dataReceived`, `Network.loadingFinished` or `Network.loadingFailed` of a request the kit keeps is `Answered`, and the request leaves what it keeps; any other of them is skipped, a kept request's later ones and those of a request it never kept alike |
| address | the page's navigation history (`Page.getNavigationHistory`); its current entry's URL, `Address` |
| tabs | the browser's targets (`Target.getTargets`), those of type `page` in the page's person's context that no `Page` request opened, in the reply's order; each one's URL, title and `canAccessOpener`, within `Limits::matches`, the rest counted; `Tabs`. No script runs in them, and the kit attaches to none: what a person's tab strip and address bar show, and the browser's own word on the opener |
| attribute | the current document's root (`DOM.getDocument`, its root alone); the first element under it the selector finds (`DOM.querySelector`), `NoElement` if none; its attributes (`DOM.getAttributes`), the named one's value, or `NoAttribute`; refused `Protocol` for a selector Chromium cannot parse, and `Limit` for a value past `Limits::text`, which is never cut, since a test uses it whole |
| elements | the current document's tree (`DOM.getDocument`, its whole depth), and with a selector the elements it selects among them (`DOM.querySelectorAll`); refused `Protocol` for a selector Chromium cannot parse |
| style | the node's computed style (`CSS.getComputedStyleForNode`, `CSS` enabled at the page's first) |
| rect | the node's box model (`DOM.getBoxModel`), its border quad's bounds; none where Chromium computes none |
| files | not an operation: on a page opened with `files`, `Fetch.enable` for every URL before it navigates; each `Fetch.requestPaused` continued for a `file://` URL, else failed (`BlockedByClient`) |
| snapshot | the page brought to the front; the accessibility tree under the document, as indented lines of role, name, value and states, cut at `Limits::snapshot` |
| screenshot | the page brought to the front; the viewport as PNG, under `Limits::screenshot` |

- **A look brings its page to the front,** as a person chooses a tab
  before looking: each find (each poll of an await), snapshot and
  screenshot sends `Page.bringToFront` on that page's session and waits
  for its answer before reading. A page already in front takes the same
  step, with no extra polling. Its other tabs keep loading; the kit
  attaches to none of them.
  - A story's page counts as hidden while Chromium reports it hidden:
    after another tab takes the front, until this page is brought back.
    Looking brings it back and makes it visible to its own
    `visibilitychange` listener (../ui/web/effects.md, section 1). A story about a
    hidden page leaves it without these looks for the hidden interval;
    listing tabs and reading attributes do not bring it to the front.

- **A find by words reads each name whole,** as the reply carries it,
  before a kept name is cut to `Limits::text`. A plain name is compared
  in the wire buffer; an escaped name is decoded once. Once `Limits::matches`
  is reached, the kit still tests membership and counts matching nodes with
  a backend id, but does not build their properties or measure their boxes.
  A words reply holds every node of the role under its scope. A reply past
  `Limits::message` fails the browser (`browser.md`, section 4): a test scopes a find by
  words `within` a node, or raises the limit.
- **A find asks only under the current document.** Its root is never a
  node id, whose document the query does not check, but an object that
  Chromium checks against the document that holds it:
  - without a scope, the object of `document`, evaluated in the page's
    current execution context (`browser.md`, section 4);
  - with a scope, the scope's object, resolved in the current document
    (`DOM.resolveNode`). Chromium refuses a node of an earlier document,
    and the find is refused `Gone` (`browser-vocabulary.md`, section 1).

  The kit releases a scope's object after the query. It can keep one
  document object per page, for unscoped finds and polls after the page
  has been brought to the front. Navigation or a session's execution
  context being cleared or destroyed retires that object; it is no longer
  used and the kit queues its release when a command slot is free. Closing
  the page drops the handle and ends its context. An object arriving for
  a read superseded by navigation or context loss is not cached and is
  released. Chromium refuses an object once its execution context is
  gone; a find encountering that refusal looks again at its next poll,
  and an `Await` keeps its deadline.
- **A find waits out a navigation.** While a navigation of the page is
  in flight, the kit's go or the page's own, the kit holds its DOM and
  accessibility commands to the page, and the evaluation of `document`,
  until the navigation commits, as a go's does. A find in flight when the
  navigation begins starts again, and asks once it has committed. An
  attribute read, and each read of a page's rendering, waits and starts
  again the same way, so that it reads one document from its root to
  its answer.
- **A press at a point reaches a shape of an inline SVG.** The hit test
  there lands on the shape, inside the node that holds it, and the mouse
  goes down on the shape; something over the shape covers it, as it would
  cover a button. The kit checks only that the hit lands inside the node:
  where no shape is painted, such as inside a shape with no fill, the
  press goes to what is beneath, and the test sees that by its effect.
- **A press is done when the browser has taken it,** not when its
  effects show. What follows a press (a request, a page redrawn) the test
  awaits.
  A node detached before mouse input is `Gone`. Errors from dispatching
  mouse input are `Protocol`, since a partial press cannot safely be repeated.
- **The fake person's press re-finds a node gone before the mouse acts.**
  It repeats the whole semantic find, including its scopes, after the
  poll interval, until the original answer deadline. Other refusals
  still fail the act. Repeated semantic finds also wait for that interval,
  whether the target was present or absent. Box measurements keep their
  own polling clock.
- **Covered is not a failure.** A dialog that has not closed yet, or a
  toast over a button, covers it for a moment. The test decides whether
  to wait and press again.
- **Typing inserts; keys press.** `type` inserts its text as one edit,
  and commits a composition left open. A native date or time field
  takes no inserted text. Each of its parts, a number or the half of
  the day, takes keys, and a part filled moves the focus to the next.
  `keys` reaches it as a person's keyboard does.
  - The kit types the characters it is given, in order. Which
    characters a field's parts take, and in which order, is the
    test's: the browser's locale sets them (`browser.md`, section 3), and the face
    checks them as its facts (../ui/web/testing.md, 1 and 2).
  - A key goes where the focus is. Focusing a field that does not have
    the focus puts it on the field's first part; a field that has it
    keeps it in the part it is in.
- **A go answers its page's closing warning.** A person who navigates,
  reloads or moves back or forward has chosen to leave the document. So
  the kit accepts a `beforeunload` prompt that opens on the page while a
  go of it is in flight (CDP's `Page.javascriptDialogOpening`, answered
  by `Page.handleJavaScriptDialog`), and reports `Warned` before the
  go's `Done`.
  - A test sees by `Warned`, or by its absence, whether the page warned
    when the person left it.
  - Chromium shows the prompt only on a page that has had a person's
    input since it loaded. On any other page it leaves without one, and
    logs at error level, from source `intervention`, that it blocked it:
    the entry's text begins `Blocked attempt to show a 'beforeunload'
    confirmation panel`. While a go of the page is in flight, the kit
    takes that entry as the prompt it would have accepted: `Warned`,
    once, before the go's `Done`, and no `Trouble`. So a page that warns
    is seen to warn whether or not the person acted on it first.
  - Closing a page closes its target, which runs no `beforeunload`
    handler, so a close is never warned.
  - No request answers a dialog. A person who stays on a page that
    warns, or who answers a confirm, waits for a test that needs one.
- **Every command has a deadline,** `Limits::answer`. A browser that does
  not answer in time fails the operation with what was pending, so a
  hung browser fails a test rather than holding it up.
