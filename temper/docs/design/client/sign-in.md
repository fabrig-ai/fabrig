# Signing in and the small store

Signing in and the client's small store are split from [the client's overview](README.md).

## 1. Signing in and the small store

How jig signs a client in, and how a native client keeps its credential
and its small store, are jig's (jig's `protocol/client/sign-in.md` and `jig/docs/design/protocol/client/README.md`, sections 7 and 9
; jig's `client/domain/client/README.md`, sections 8 and 9).
temper's:

- **A sign-in's address in the small store** (`pages/accounts.md`, 1, `ConnectModel`, *A reload
  while it waits*): a browser's shell keeps, in one record of its own under
  the key `sign-in`, beside jig's, the latest answered sign-in's source, its
  address and whether its tab was refused, so that a reload draws the same
  page. The address is the provider's authorization address, with its state
  and challenge and no verifier, which the page's link and the tab's history
  show anyway. A later sign-in's record replaces it, and it goes with the tab.
  The record is read only for that source: another's is no sign-in of the page's.

- **The printed link** (jig's `protocol/client/sign-in.md`): at
  start, `temper-box serve`, or `temper box` before it starts serve
  (`../protocol/setup/box-supervision.md`, 1), prints a sign-in link for the deployment's
  first owner (`../protocol/binaries.md`), at the link's page (below),
  `http://HOST:PORT/sign-in?secret=SECRET`. `temper-box open
  DIRECTORY LINK` checks that the link names the engine's address on
  loopback and that route or the link's page (below), and gives the secret to `jig-client` with the
  credential DIRECTORY holds, if any (`SignIn { with: Link { secret,
  held } }`). The secret is in the command line only until the sign-in
  spends it; the client root passes it on and keeps nothing of it. The
  running engine also keeps a local link of its own at rest, never the
  printed one, for a TUI on the same machine (`tui/directory.md`, 1, *The engine's link,
  kept at rest*).
- **The link's page, `/sign-in`** (the user's request of 2026-11-18,
  "a more terse URL, e.g. http://127.0.0.1:8367/sign-in?secret=xxx"):
  temper gives jig's mapping its link page, `/sign-in`, beside its
  prefix, `/jig` (jig's `protocol/client/default-http.md`, 1), so the
  link reads `http://HOST:PORT/sign-in?secret=SECRET`, its secret spent
  by the `POST` to the link's route, `/jig/v1/sign-in/link`, as before.
  - **Printed and shown at the page:** every link temper makes is in
    the page's form, by jig's ends given that page
    (`jig/docs/design/protocol/client/default-http.md`, 4): the box's block and every fresh link
    (`../protocol/setup/box-supervision.md`, 1), serve's line, and at a public
    origin `https://NAME/sign-in?secret=SECRET`; the engine's local
    link at rest (`tui/directory.md`, 1); the link the box hands the TUI
    in front of it, which its welcome and `/web`'s note show as given
    (`tui/box.md`, *The welcome*); ctl's asked link (`ctl/command-line.md`,
    1.1); and the TUI's link field's placeholder
    (`tui/pages/sign-in.md`, 1). The help that shows the link shows it
    so, and the operator's journal search finds it so.
  - **The route's form still taken** ("let's keep backward compat"):
    a link in the route's form,
    `http://HOST:PORT/jig/v1/sign-in/link?secret=SECRET`, printed by a
    build before, still signs in wherever a link is taken, and its
    `GET` still answers the page (`web/serving.md`, 1), until the user
    says otherwise (2). Every client of temper takes either form: the
    browser opening it (`web/sign-in.md`, 1), the TUI's paste and the
    engine's link file (`tui/pages/README.md`), `temper ADDRESS`
    refusing it as an argument (`tui/README.md`, section 1),
    `temper-box open` (`line.md`, 1), `temper ctl DIRECTORY sign-in`
    (`ctl/command-line.md`, 1.1) and the TUI journeys' redaction
    (`tui/journeys.md`, section 5); each reads it with jig's mapping
    made with that page.
  - **Named once:** the page's path is `LINK_PAGE` in
    `temper-client-protocol-wire`, the lowest crate both the engine,
    which prints the link, and the clients, which read it, name;
    `temper-client-domain-forge`'s `sign_in` module re-exports it, and
    every client names it from there.
  - **No version moves:** the link is no versioned wire, and every
    client of temper since dev.fabrig.ai's 0.0.2 takes both forms. A
    client of a build before that one says the page's form is no link;
    it is a client of another build, which an engine refuses and tells
    to upgrade (`../draft/redeploys.md`, section 8).
- **A sign-in held is kept** (jig's `domain/people/README.md`, section 3): a
  person still signed in who opens a fresh link with the same DIRECTORY
  keeps their sign-in, its number and its expiry, and the deployment's
  sign-ins don't grow. The engine checks the credential offered as any
  exchange's and answers with none; `jig-client` goes on holding the
  one offered, and `DIRECTORY/token` is not written. A credential the
  engine does not know, or one ended or expired, is no refusal: the
  link signs the person in afresh, and its credential is written. A
  link refused, answered not ready, busy or full, or an engine not
  reached, leaves `DIRECTORY/token` as it was.
- **The bearer credential,** the sign-in's number beside 32 random
  bytes, is `DIRECTORY/token`, a private file (`0o600`), which
  `jig-client-shell-native` writes where the host says, when it says.
  Once signed in by a link, the host reads the credential from
  `jig-client`'s `link()`, through the client root, and has the native
  shell keep it, before anything is told of the sign-in; a new sign-in
  by a link has it written anew, and a sign-in kept writes nothing,
  its credential unchanged; signed out, the host has the native
  shell keep none, which erases it. A later `temper-box open DIRECTORY
  ADDRESS` has the native shell read it before its loop (`line.md`, 1), and the
  host gives it back as held (`SignIn { with: Held(credential) }`). It
  is never printed and never an argument, and neither temper's root
  nor its view keeps it.
- **The small store** is `DIRECTORY/store`, `jig-client-shell-native`'s
  file, holding `jig-client`'s records alone: the requests it keeps,
  jig's and the forge's adoptions, pending. temper's pages keep none
  (`root.md`, 1.4). It never holds an LLM account's key, token or record, nor a
  forge's pasted value: a request carrying a value is never kept (`root.md`, 1.6),
  and a sign-in's record and a forge's credentials are the engine's
  alone (`../protocol/credentials.md`). A pending FORGE request that
  carries no value (a manifest's `Connect`, `Install`, `Forget`,
  `SetUp`, `Reachable`) is kept as any keyed request; one carrying a
  value is not.
- **Loopback, or TLS by name,** so a bearer token never crosses a
  network in plaintext: plain HTTP reaches an engine on loopback alone
  (`../protocol/configuration/README.md`, section 2; `line.md`, 1), and
  the TUI reaches a deployment by name over HTTPS
  (`tui/README.md`, 1; jig's `protocol/client/README.md`, section 9),
  each deployment's credential and small store in a directory of its
  own (`tui/directory.md`, 1). `temper-box open` and `temper ctl` stay on
  loopback.
- **One person.** The deployment's link signs in its first owner;
  signing in through the forge is open (`README.md`, section 5).

### 1.1 A terminal signed in from the browser

jig's third built-in way (jig's `protocol/client/sign-in.md`, *A
sign-in approved by a browser*; `../draft/tui.md`, 4.2): a native
client on the person's machine, the TUI, asks; the person's browser,
signed in by its cookie, approves at the approval page (`pages/shared-behaviour.md`, 1, *The
approval page*); the client takes the sign-in with the secret its ask
was answered with. A box enables it beside the printed link
(`../protocol/configuration/engine.md`, clients).

- **The root's half,** for a native client, in `temper-client-domain`,
  a module of its own, `asked`:
  - *Asking:* `Event::AskApproval { port }`, the host's, while the link
    is signed out, `port` the ask's loopback return, `Option<u16>`: the
    port the host's listener on `127.0.0.1` took, for the one click at a
    name (`tui/pages/sign-in.md`, 3 and 4), or none, on loopback and for
    a typed code. At any other time, or while an ask is in flight, it
    changes nothing. The root sends `jig-client`'s `AskApproval { port }`
    and holds `asked::State::Asking`.
  - *Asked:* on `jig-client`'s `Approval::Asked { secret, code, expires
    }`, the root signs in at once, `SignIn { with: Approval { secret,
    grant: None } }`, and holds `Asked { code, expires, port }`, `port`
    its ask's, for its host to draw. The secret goes to `jig-client`,
    which holds it while the take runs (jig's
    `client/domain/client/README.md`, section 9), never into
    `asked::State`, a print or a log. An ask that named a port keeps it
    in the root too, in a field no reader is given, for the take with
    the grant (*The grant*, below), until the link leaves signed out or
    another ask replaces it. An unlinked client (jig's
    `client/domain/client/README.md`, 4.4) leaves the take to its host's link,
    which made the ask and holds the secret: the root holds the code
    and the expiry, and drops the secret it was told.
  - *The grant:* `Event::Granted { grant }`, the host's, `grant` a
    `skein_lib::Secret` of the 32 bytes its loopback listener read
    (`tui/pages/sign-in.md`, 4). While the root holds `Asked` with a port
    and its secret, and the link is signed out, it signs in again,
    `SignIn { with: Approval { secret, grant: Some(grant) } }`, whose take
    replaces the one before; `jig-client` sends it on as that section
    has it, again without the grant after `not_granted`. At any other
    time it changes nothing, and the grant is dropped. The root keeps no
    grant: it is `jig-client`'s while the take runs.
  - *Not asked:* `NotAsked(Refused | NotReady | Busy | Lost)`, held as
    it came, for its host's line.
  - *The take's outcomes are the link's,* in `jig-client`'s `link()`:
    while it waits, `not_signed_in` is `Approval(Waiting)`, and
    `jig-client` sends the take again each second; refused, expired,
    used and unknown end it, `Approval(..)`; full, `Full { .. }`, its
    `link_until` the approval's own expiry; signed in, the link starts,
    its credential in `link().credential`, which the host keeps as it
    keeps a link's (section 1, *The bearer credential*). Once the link
    leaves signed out, the root drops what it held.
  - *Asking again* is another `AskApproval`: its take replaces the one
    before.
  - `Domain::asked()` gives the state to the host and the view, and the
    root's shown count moves with it.
- **The TUI's flow,** on the TUI's sign-in screen (`../draft/tui.md`,
  4.1), its host and its view, the TUI's crates:
  - *It asks* once each time its sign-in screen opens signed out (`tui/pages/README.md`:
    at its start with no credential kept, and again after a sign-out or
    a kept credential the engine refused), while the root holds no ask:
    the host steps `Event::AskApproval`. A held credential's sign-in in
    flight is not signed out, so nothing is asked until the engine has
    answered it. On loopback the screen still takes the printed link,
    pasted, as `../draft/tui.md`, 4.1 has it; at a name it holds the
    approval alone, by one click where it can open a browser, else by a
    typed code (`tui/pages/sign-in.md`, 3).
  - *Asking:* until the engine answers, "Asking temper for a code…",
    dim, under the screen's first line.
  - *The tab's address* is the TUI's own, made from its own parts alone
    by `temper-client-tui`'s `approval_address(origin: &Origin, code:
    Option<ApprovalCode>)`: the origin it was given or kept, printed
    (`tui/README.md`, 1), then the approval page's printed path,
    `address::print_approve(code)`. The host gives the code on loopback,
    `http://127.0.0.1:8367/terminal?code=WDJB-MJHT` (`web/addresses.md`,
    1), and for the one click at a name,
    `https://dev.fabrig.ai/terminal?code=WDJB-MJHT`, the code an
    `ApprovalCode`, so of the code's alphabet by its type; and none for
    a typed code's ask at a name, `https://dev.fabrig.ai/terminal`, whose
    page asks for the code typed (`tui/pages/sign-in.md`, 3). An answer
    whose code is off the alphabet is refused by jig's reader, so
    nothing is opened for it. It opens no address the engine sent.
  - *Opening the tab:* on each ask answered, once, the host's duty
    `open` carries the address, and the view draws it as opened until
    the opener's report says otherwise. The run gives the address to the
    opener as its one argument, with no shell between: `xdg-open
    ADDRESS` on Linux, `open ADDRESS` on macOS, a departure, on purpose,
    from `temper-box open`, which opens no browser (`line.md`, 5). The opener's
    stdin, stdout and stderr are none of the terminal's, so nothing it
    says reaches the screen; its environment is the TUI's. Its exit
    ended, the run reports `Opened { opened }`, done on 0. Closing
    aborts and reaps the opener alone, without waiting for it to finish
    by itself or signalling the browser's group (skein's `runtime/io/processes.md`
    , `SpawnAlone`).
  - *The opener* is the binary's choice, at start: none on an SSH
    session (`SSH_CONNECTION` or `SSH_TTY` set) or, on Linux, with no
    display (neither `DISPLAY` nor `WAYLAND_DISPLAY` set); else the
    program of that name in the first absolute directory of `PATH` that
    holds it as an executable file; none when no directory does. With
    none, the run reports `Opened { opened: false }` at the duty itself.
  - *Its lines,* the code and the address drawn, the code as shown:

    ```
    Sign in this terminal
    A tab opened in your browser. Check it shows
    WDJB-MJHT, then press Approve.
    No tab? Open http://127.0.0.1:8367/terminal?code=WDJB-MJHT
    The code expires in 10 min.
    ```

    "The code expires in SPAN." is `view.md`, 6's *span* to the ask's expiry,
    with its tick. On the screen, "Sign in this terminal" is `tui/pages/README.md`'s
    first line, "Sign in this terminal to HOST:PORT", said once; the
    approval's lines follow it, then the paste's, "Paste the link temper
    printed, then Enter." and its field.
  - *No browser to open:* an SSH session, no display, or the opener
    failing (not found, or exiting other than 0): the TUI opens nothing
    more and prints the address and the code, "Open this address in a
    browser that reaches temper:", the address, "and check it shows
    CODE, then press Approve.", then "From another machine, tunnel port
    PORT to this one first.", then the expiry's line. The engine is on
    loopback, so a browser elsewhere reaches it through a tunnel to its
    listener on the same port, which jig's guards require of the
    address (jig's `protocol/browser/README.md`, 12.2). The tunnel's
    line is loopback's alone: a deployment by name is reached from any
    browser, and its lines are `tui/pages/sign-in.md`, 3's.
  - *Asking again:* Enter in the Link field while it is empty, once the
    approval has ended or was not asked, or a pasted link has replaced
    it, asks again (`may_ask_again`), and its line says "Press Enter to
    ask again." The field has the focus on this screen and takes every
    character, `q` and `a` among them (`tui/pages/README.md`), so no letter can be the
    key. While an approval waits, Enter in the empty field is the
    paste's "not a link" refusal, as before.
  - *A pasted link, sent,* replaces the approval's take (jig-client's
    sign-in in flight is the newest one asked): the approval's lines
    give way to the paste's progress and its refusal, and Enter in the
    emptied field asks again.
  - *Each outcome's line,* in place of the screen's last line:

    | Outcome | Its line |
    |---|---|
    | waiting, or a grant that isn't the approval's (`not_granted`) | the screen as drawn |
    | refused | "The browser refused this sign-in. Press Enter to ask again." |
    | expired | "The code expired before it was approved. Press Enter to ask again." |
    | used | "That code was already used. Press Enter to ask again." |
    | unknown | "temper doesn't know that code any more: it restarted, or the code is old. Press Enter to ask again." |
    | full | "All N sign-ins are in use; the earliest ends MOMENT UTC. Press Enter to ask again then." (`view.md`, 1's full, its moments so said) |
    | the take busy, not ready or lost | "temper is busy; still trying…", while `jig-client` sends it again |
    | not asked: refused | "This temper doesn't sign terminals in from the browser. Paste the link it printed instead."; at a name, "PLACE doesn't sign terminals in from the browser." (`tui/pages/sign-in.md`, 3) |
    | not asked: busy | "Too many terminals are waiting to sign in. Press Enter to ask again in a moment." |
    | not asked: not ready or lost | "temper is starting or stopping, or out of reach. Press Enter to ask again." |

    An ended approval's code, address and expiry are no longer drawn:
    the outcome's line stands under the screen's first line.
  - *Signed in:* the TUI keeps the credential in its directory
    (`../draft/tui.md`, 4.1), as `temper-box open` keeps a link's (section
    1), before anything is told of the sign-in, and draws its first
    screen; the page says "Signed in. You can close this tab." (`pages/README.md`).
  - *Not asked while the TUI signs in from the engine's link* (`tui/pages/README.md`):
    the approval is asked when that ends with the TUI signed out, as the
    sign-in screen opens.
  - *The browser's way in beside the TUI's link:* a TUI signed in from
    the engine's link (`tui/pages/README.md`) spends the engine's local link, never the
    printed link (`tui/directory.md`, 1; jig's `protocol/client/sign-in.md`, *The local
    link*), so the link the box's terminal shows stays live for the
    browser, before the TUI signs in or after it. That link is the
    browser's way in, and the approval page (`pages/README.md`) asks for the browser
    signed in first.
  - *Never shown:* the secret and the credential are never drawn on
    the screen or logged. A recording keeps them in the person's
    private file; `temper-client-journeys print FILE OUT` redacts them
    before writing pages outside the repository's journeys
    (`tui/journeys.md`, 3.1 and section 5; `web/journeys.md`, 5.1).

## 2. Open questions

- **Approval between hosts and managing sign-ins** (1.1): a deployment
  at a name, Google on the web, the code typed there with the ask's origin
  and age shown, and the web's list of a person's sign-ins, each ended
  from there (`../draft/tui.md`, section 7). Default: loopback, approved
  by a browser signed in by the printed link, or by Google's way where it
  is given (`pages/shared-behaviour.md`, 1, *The approval page*).
- **A failed link beside a live cookie** (`view.md`, 1): the used, expired or
  unknown paragraph remains until another sign-in, even if the cookie
  still signs in on a reload. Falling back to the transport is jig's
  client question (jig's `client/domain/client/README.md`, section 15).
  Default: no fallback; reload to use the cookie.
- **The route's form retired** (section 1, *The route's form still
  taken*; jig's `protocol/client/default-http.md`, 7): once every tool
  and person has moved to the page's form, temper might stop taking a
  link at the route and stop answering its `GET` with the page. The
  user's request of 2026-11-18 keeps it "at first, then all tooling gets
  migrated"; the user decides when. Default: both taken, the route's
  page kept.

- **The forge as people's sign-in provider:** temper's own routes on its
  clients' listener run the forge's OAuth flow, read who the person is,
  and ask jig's clients' component to mint a credential for that
  identity (`Mint`, with the forge's provider number, a provider of
  kind application in jig's `providers` member, the person's id there,
  their login and name; jig's `protocol/client/sign-in.md`).
  A browser then holds it as the default mapping's cookie; a terminal
  needs a flow of its own, such as a device code. Default: the
  deployment's link only.
