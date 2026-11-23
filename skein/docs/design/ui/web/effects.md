# The browser parts: fetch, the store, transport and effects

## 1. Fetch, the small store and the transport

- **Exchanges** by fetch, to a path on the origin, never to another
  origin: the method, headers and body the application gives (a POST
  with its body, a GET with none), with same-origin credentials;
  abandoned by aborting; at most the exchanges in flight fetch is given.
  An answer's body is read up to the bound the exchange gives; past it,
  the exchange failed. Which headers an exchange sends, what its body
  holds and what its server answers are the application's (a header
  naming its client, a cookie its server set).
- **An exchange's cache and an answer's headers:** an exchange names its
  cache, the browser's own by default, or none (fetch's `no-store`, so
  the browser's cache neither answers it nor keeps its answer), and the
  answer's headers the application keeps, by name; the platform reports
  those, each value within the exchange's bound, with the status and
  the body, and no other header. An application that reads an answer's
  freshness (its `Cache-Control`, its `Age`) or when to ask again
  (`Retry-After`) asks for none of the browser's cache and keeps those.
- **An exchange's body in pieces,** when the exchange asks for it, for
  an answer whose parts mean something before its end does (a sequence
  of replies, each framed, written as each is ready): its status and
  the headers it keeps are reported first, opened; then its body as the
  stream's is read, each piece at most the bound the exchange gives (a
  longer one handed in parts), handed to the state machine as it comes,
  with the next read asked by the loop after the step (`README.md`, 4.2); then its
  end. The body's whole stays within the exchange's bound on its answer,
  past which the exchange fails, as it fails when the connection does
  after it opened. It holds its place among the exchanges in flight
  until its end, its failure or its abandoning. An exchange that asks no
  pieces is answered whole, as above.
- **The stream** by fetch, a GET of a path with the headers the
  application gives, with same-origin credentials, its body read a piece
  at a time: each piece, at most the bound the application gives (a
  longer one handed in parts), handed to the state machine as it comes,
  the next read asked by the loop after the step (`README.md`, 4.2), so the platform
  paces it. An opening refused with a status other than 200 reports that
  status first, then reads the response body with the same bounded pieces
  and pace until its end or the application's close; the application
  interprets its refusal records. One stream at a time.
- **The transport** takes a step's requests whole: the small store's
  writes first, then the exchanges and the stream, then the navigation
  away, if any, to a path on the origin or an `https` address the step
  names (a provider's sign-in, another site). When a write fails, every
  exchange of the step is reported failed, none is sent, and there is
  no navigation (`README.md`, 4.3).
- **The navigation away** sends the page's own tab to the address, once,
  in the turn whose step named it: the state machine names it from an
  event (an answer naming a provider's sign-in), never from its state,
  so that no build, rebuild or reload names it again. A reload starts
  a new page, which navigates only when its own steps name an address.
  Its targets, a rule (`README.md`, section 14): a path on the origin (one leading
  `/`, not `//`, no `\`), or an `https` address. A target outside them
  is not followed and nothing is reported: the page stays as it is, and
  an application that may name one shows the address otherwise too, as
  a link. **Plaintext loopback** (`http://127.0.0.1:PORT/…` or
  `http://[::1]:PORT/…`) is a target only when the application makes
  the browser's platform with loopback navigation on, a setting of the
  platform, off by default, which an application turns on only for a
  test's page, where its fake peer (an issuer on loopback) has no TLS;
  skein-ui-web has no feature for it. The check is a pure function of
  the target and the setting, tested natively (`testing.md`).
- **A tab of the page's own,** for what a person does beside a page that
  stays (a provider's sign-in): the platform's three acts, one tab at a
  time, the shell's to ask and outside the transport's whole.
  - **Opened,** `open_tab`, with a line of words, in the turn of a person's
    act (section 2, A person's gesture), before the transport carries that
    turn's writes and exchanges: `window.open` with no address and no
    `noopener`, which would hide the tab from the page; the words its title
    and its body. It answers whether a tab opened: a browser that refuses
    (a pop-up blocker, an act's activation spent) gives no tab, `false`,
    and the application draws what the person does instead. A tab held and
    not yet pointed is closed first.
  - **Pointed,** `point_tab`, once, from an event (an answer), never from
    the state, under the navigation's targets (above): its opener cut
    first (`opener` set to null), so that the page at the address has no
    hold on this one, then sent to the address in place of its blank
    entry. A target outside them closes the tab, and nothing is reported,
    as the navigation away: the application shows the address otherwise
    too. The platform lets a pointed tab go, and keeps no hold on it.
  - **Closed,** `close_tab`: what an answer refusing the act asks. With no
    tab held, closing and pointing do nothing.
  - **A platform with no tabs** refuses each opening: the trait's default,
    which the worker's takes (`README.md`, section 15), and so does a test's unless it
    says otherwise. A reload ends the hold, and the tab stays as it is.
- **A hidden tab lets its stream go** a minute after it is hidden (a
  default, the application's parameter): fetch closes the stream and
  reports its end, as any drop; an opening while hidden waits until the
  tab is shown. A browser allows about six connections to one origin
  over HTTP/1.1, each tab's stream holding one.
- **The small store** is the tab's session storage: it survives a reload
  and goes with the tab. Each record and its key are base64 text
  (../../runtime/lib-bytes.md, 1.1), the key under the application's prefix, read whole at
  start. A write fails when the platform refuses it, its quota full, or
  past the small store's own bound, the application's parameter,
  unbounded by default, which a story sets low (`testing.md`, 2).
- **The kept store** is the browser's local storage, beside the small
  store: what a person chooses once for every tab of the origin (a
  look), never a request, a credential or a stream's place. Its records
  are the small store's form, base64 text under the application's prefix,
  read whole at start; a write fails as the small store's does, and a
  page that has no local storage (a private window that refuses it, the
  worker, a test's platform unless it says otherwise) reads none and
  refuses every write: the trait's default. The transport never carries
  it: the shell writes a kept record when its application asks, outside a
  step's whole, and a failed write changes nothing but that record.
  Another tab of the origin reads the record at its own start; no tab is
  told of another's write.

## 2. Effects in a browser

An effect (../README.md, section 11) is data an application's step emits, in
its own closed enum, asking its platform for what is outside the tree (a
share sheet, a clipboard write, a file picked).

- **Carried out by its shell:** its own code over the glue, or a part of
  skein's where one exists (the file picker, when it comes: `README.md`, section 12),
  giving back **exactly one report per effect,** carrying its token:
  done, refused, failed, or its result (a file's name, type and bytes,
  within the bound the effect states). One the shell cannot carry out is
  reported refused at once.
- **Through the inbox, like any report:** the code that carries out an
  effect is given the inbox and a function wrapping its report into `R`,
  as each part is (`README.md`, 4.1). **No re-entry still holds:** a platform
  callback (a promise settled, a picker's change) raises a report, and
  the inbox calls the turn only when none runs.
- **A person's gesture:** an effect the platform allows only during one
  (a file picker, a share sheet, a clipboard write, a tab: section 1) is asked in the turn
  the person's act started, which the inbox runs within the act's event,
  so the gesture holds. Asked in a turn a timer or an answer started, it
  is refused by the platform, and reported refused.
- **The clipboard,** a part of skein's: `navigator.clipboard.writeText`,
  its promise's end reported through the inbox. The clipboard is there
  only in a secure context, so on plain http it is missing, and the write
  is reported refused at once, as is a text that isn't UTF-8. A promise
  rejected with `NotAllowedError` is refused, and any other rejection
  failed; both are the application's to say, and neither touches the
  page.
- **Full screen,** a part of skein's, asked in the turn of a person's
  act, as a tab is:
  - *Entered:* `requestFullscreen` on the document's element. Once the
    page is in full screen, the part asks `navigator.keyboard.lock` for
    `Escape` alone, where the browser has it (Chromium's, in a secure
    context), so that a short Escape reaches the page as any key does and
    a long one still leaves full screen, as the browser's own notice
    says; the desktop's own shortcuts are the browser's and stay so. A
    lock refused or missing changes nothing else: there a browser keeps
    Escape, and it leaves full screen, as Firefox and Safari do. The
    report `Entered` follows the browser's change; a request refused, by
    a browser with no full screen, an activation spent or a rejected
    promise, is reported `Refused`, at once where nothing was asked.
  - *Left:* by the browser (its Escape, its long Escape, its own control)
    or by the application's `leave`, which exits full screen when the
    page is in it and otherwise does nothing. Leaving unlocks the keyboard
    and is reported `Left`, once for each `Entered`, whoever left. The
    page's tree is untouched by either: what full screen means to a
    person is the application's, which draws it.
  - *A page with none,* the worker's or a test's unless it says
    otherwise, reports each request `Refused`.
- **A reload ends what is in flight:** an effect asked before it gives
  no report after it; the state machine starts again from its small
  store (`README.md`, section 4), which holds no effect in flight. A reload leaves
  full screen, as the browser does, with no report.
